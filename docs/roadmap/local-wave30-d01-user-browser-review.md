# D01 independent Essentials password browser checkpoint

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original D01 task
`a96a1977-3210-4284-8f7d-645793369301`; existing worktree
`7c85f5ef-3fac-4f72-aaed-08474d7fb454`. Observation date: 2026-10-02.
Only this report was reserved. The root owns integration and task status.

Result: **the bounded password browser path passed**. A fresh synthetic
administrator signed in through the printed Essentials section 2 form,
reached the signed-in portal, opened and closed Sign-in and security,
signed out, and signed in again. Final cleanup signed out again. No product
or printed-guide failure was observed in these steps, and no corrective
retry was performed. This is an independent worker checkpoint against the
guide, not a recruited inexperienced human's usability study or a completion
claim for every D01 task. D05's independent user/operator acceptance gate
remains a root decision. O06 remains DONE and closed.

## Exact inputs and provenance

The published guide was read from Git objects at
`6b4db4f0317e4427c187f55e063989ccba124217`, also the observed local main ref.
No merge or guide edit was performed. The authorized prebuilt Essentials
artifacts were the root's c01-source build, with source pin
`c01c39ab4e092423d5522bedc50fff87656d8c0a`. Artifact bytes were independently
hashed before execution; this worker did not reproduce their build.

| Input | Exact observation |
| --- | --- |
| Server | `riauth`, 183038592 bytes, SHA-256 `7abf745c10691a012a1918c83089168d0e0d88b42764dbf8ed538b90c828b606` |
| Maintenance | `riauth-maintenance`, 56499296 bytes, SHA-256 `86490c7f71de6b7ae9d4dabdf9060a8757100f3aedbdaa670284d9e1206a2a95` |
| Binary directory | `/Users/dominik/orca/projects/riAuth-public-preview-local-module-boundaries-wave27/target/d01-essentials-c01c39a/aarch64-apple-darwin/debug` |
| Published Essentials guide | `docs/essentials-guide.md`, blob `e48371d3a34a7b6441cbcfce700daf954aa5f2ae`, SHA-256 `9df3c2861984751d48b28d284cfb47c07801fd3934072e2ccb829edeb35579a0` |
| Published portal guidance | `docs/PORTAL.md`, blob `7c529fcf698e0e07b083d4cfb520bf340f32857e`, SHA-256 `386fa4ca53f4607133b3f30071b7e2714ff4504cb0879b514fe2b52d86ddc015` |
| Older c01 guide, distinct input | Blob `2b8408f90d82bb35d49fc1ebd3512d9a1a853962`, SHA-256 `80f54441cd5cca1cd3c8521ba91b125db0ac8a86c8ee4055986b8f69402c67ee`; its hash was read, its historical observations were not refreshed |
| Live GUI provider | RiWork Cua.ai Driver MCP, daemon 0.30.4, `com.trycua.driver`, `/Applications/CuaDriver.app/Contents/MacOS/cua-driver`; health `ok` on macOS 26.2 arm64 |
| Permission readback | Accessibility and Screen Recording granted; read-only checks did not run the prompt-capable direct ScreenCaptureKit probe |

The local cua-driver skill pack identifies itself as 0.29.1, distinct from
the live daemon. SKILL, MACOS, BROWSER and RUNTIME instructions were consulted,
and advertised tool descriptions/current state were read before browser
input. Instruction-file SHA-256 pins:

| File under `/Users/dominik/.agents/skills/cua-driver/` | SHA-256 |
| --- | --- |
| `SKILL.md` | `593466f73db9e063b3d18a7a939fec79e7e171fe1f2da486f0b780e223426abe` |
| `MACOS.md` | `c3fe476a0321fef996e82cbe0ab06d75a44cf1298daaec24ed681efb9a3b2cc5` |
| `BROWSER.md` | `865da3b26f49f9a4dcdcdcfd245783e604ba81563b19e566579a17a1b46c3ba7` |
| `RUNTIME.md` | `ad032aecfbee63812a2fda201aeb3c8b0ebc56dc8418690c71d2fc2c7850fe20` |

These are instruction and artifact hash checks, not additional body reviews
or runtime tests. The guide's section 2 operator/password path and browser
instructions, section 4 navigation entry, and PORTAL sign-in, sign-out and
security-panel guidance support the recorded actions. Selected c01
`src/cli/local.rs` init and `src/cli.rs` hidden-password code were also read
to deliver the printed interactive setup without adding noninteractive flags.
No broader source audit was performed.

## Disposable operator setup

The ownership ledger's `wave30_D01_independent_user_browser` reservation was
read: approved, runtime released, this exact worktree/shell, one report,
RiWork Cua only, at most 20 minutes and no other desktop controller.

Initial free space was 10.829 GiB and `lsof` found no listener on port 9000.
A new owner-private directory under this worktree's ignored
`deployment-private/d01-user-browser-isnden7n` and a private `xdg` directory
were created with umask 077. A newly generated synthetic password was kept
out of the report and visible command arguments. The printed interactive
command was executed with absolute authorized binary/config paths and the
fresh XDG directory:

```text
riauth-maintenance --config <fresh-private-lab>/riauth.toml init
  --issuer http://localhost:9000 --listen 127.0.0.1:9000
  --data-dir data --admin admin
```

Both hidden password prompts completed; init exited 0 and reported
`initialized: true`. No `--password-stdin`, CLI login, session approval,
HTTP authentication substitute or previous operator fixture was used.

`riauth --config <fresh-private-lab>/riauth.toml serve` owned PID 9986.
The listener readback was exactly PID 9986 on `127.0.0.1:9000`.
The guide's `curl --fail http://127.0.0.1:9000/readyz` entry check, bounded
with `--max-time 5`, exited 0: status `ok`, role `integrated`, version
`0.1.1`. This HTTP read was only the printed readiness prerequisite.
The browser performed all user authentication and navigation.

## Actual browser observations

Driver session `wave30-d01-user-browser` used one explicitly authorized
`browser_prepare` with `allow_launch: true`, profile `isolated_new`.
It returned `launched_isolated_browser`, prepared/owned PID 6760, and
reported no personal profile copying, changed preferences, permission
prompt, existing-browser termination, foregrounding or global input.
`list_windows` returned ordinary window 102201 titled `about:blank`.
Binding reported `status: ok`, `binding_quality: exact`,
`mutation_allowed: true`, endpoint access `driver_owned`:

```text
target: bt-c792e95c-4d86-4c75-b477-134900f71f89
tab:    tab-d0cab2bb-6bed-4ce4-8fa9-6739d7380e6f
```

These were returned session capabilities, not guessed raw CDP identifiers.
Every ref action used a fresh snapshot from this same target/tab/session.
Typing used `browser_type`. Button clicks explicitly selected
`input_route: dom_event`, the documented macOS background route. Those
dispatches report `unverifiable`; their application effects were independently
checked by subsequent page snapshots. They do not prove physical mouse input
or a trusted-click browser ceremony. No raw CDP, Playwright, AppleScript,
legacy page script, personal-profile attachment or provider switch was used.

| Step | Fresh observed evidence |
| --- | --- |
| Open printed `http://localhost:9000/apps` | Snapshot `p845`: “Not signed in”, Username, Password, optional Authenticator or recovery code, and instruction to leave it empty |
| Enter credentials | Username entered through `p845:6`, confirmed as synthetic `admin` in `p846`; password typed through `p846:8`; `p847` confirmed the authenticator field remained empty; no secret value was printed or captured |
| Submit Sign in | `p847:53`; `p848` showed submission pending, then `p849` showed “Signed in as admin (@admin)”, “Connected”, “Welcome back, admin”, and 0 applications |
| Open Sign-in and security | `p849:55`; `p850` showed the named dialog, Password/Change password, Passkeys with “You have no passkeys yet”, and Authenticator app “Off” |
| Close security dialog | `p850:22`; `p851` returned to the signed-in portal |
| Sign out | `p851:92`; `p852` showed “Not signed in”, “You’re signed out”, and the password sign-in form |
| Sign in again | Username remained `admin`; fresh password input via `p852:8`, empty code checked at `p853`, submit `p853:53`; pending `p854`, then `p855` restored signed-in identity, “Connected” and the empty catalogue |
| Final UI cleanup sign-out | `p855:92`; `p856` again confirmed “Not signed in” and “You’re signed out” |

An empty catalogue is expected: this fresh lab had no registered application.
The portal also displayed the missing-email and extra-verification notices;
the latter remained informational here and no application access was attempted.
No passkey, Touch ID, authenticator enrollment, password change or recovery
button was activated. Sessions and consent was visible as a link but was not
opened; session-list/consent management is not claimed.

No screenshot or video was requested at any point, including secret entry.
Raw passwords, cookies, session tokens, private signing material and browser
storage were not copied into durable evidence. Browser snapshots were used
for page state, not an accessibility conformance test.

## Cleanup, release and limitations

Finally cleanup used Driver `kill_app` only on the proven driver-owned
PID 6760; it reported SIGKILL sent. `end_session` reported inactive.
Subsequent Driver `list_windows` returned zero windows for that PID and
`list_apps` no longer listed it. No personal browser process was terminated.
Driver-managed profile-file erasure was not separately inspected or claimed.

The owned server/controller received Ctrl-C/SIGINT and exited 0. The private
lab, store, logs and XDG directory were removed in the controller's finally
block. Server elapsed time was **186.997 seconds**; minimum monitored free
space **9.598 GiB**, above the 8.5 GiB stop margin and 8 GiB floor. Cleanup
readback at `2026-10-02T12:26:34.616383+00:00` found the lab absent, owned
server PID absent, no listener on 9000 (`lsof` exit 1), and 9.904 GiB free.
The complete checkpoint stayed within the authorized 20-minute bound; an
exact overall browser/preflight duration was not separately instrumented.
Immediate result and sole desktop/runtime resource release were sent through
the explicit project orchestrator before this report was committed.

Preflight discovery initially checked a guessed `riauth-server` filename;
the authorized server is `riauth`, which was located and hash-verified before
execution. A read-only implicit session-state query returned
`session_not_started`; starting the declared named lifecycle then succeeded.
Neither was a product/guide failure or a permission/binding refusal. All
actual product commands and required page transitions above succeeded.

This checkpoint leaves install/download/release provenance execution, an
ordinary non-administrator user, an independent recruited human, application
OIDC redirect/consent, physical/synced/phone passkeys, Safari/Firefox/mobile,
invitation/mail, password lifecycle, session-list/consent management,
LDAP, SCIM, PostgreSQL, HTTPS/proxy deployments, backup/recovery and the
other Essentials/Platform tasks untested here. It supplies bounded password
browser evidence only and does not imply all-D01, all-D05, current CI,
release, HA, interoperability or deployment acceptance.

No Cargo, build, other product binary, existing source/test/config/guide edit, merge/reset,
main/push/status operation, new worker/task/worktree, service installation,
settings/security change or other-worker contact occurred. The only tracked
change is this reserved report. Existing accepted receipt-secret, header,
PAM, held Group and nonrenewed-lease/paused-I/O limits remain undisturbed.

Report verification: `python3 scripts/check-docs.py` exited 0 with
“Markdown links and build-directory layout checked”; `git diff --check`
passed. These documentation checks are separate from the browser checkpoint.

## Source-first section 3 application checkpoint proposal — 2026-10-02

**Proposal only; application source and all runtime remain held.** Root reserved
only this append in `wave30_D01_confidential_browser_plan`, existing project,
worktree and shell. This phase starts no listener, browser, binary, helper or
test and changes no guide, D05 artifact or product source. The entire preceding
180-line / 11586-byte report remains the dated `6837b745` observation.

The proposed checkpoint supplies one additional task input: an application for
the printed confidential `local-demo` registration. One fresh browser would
use that application's sign-in action, sign in with the synthetic password,
approve actual consent, return to **`http://localhost:3000/callback`**, and
reach a cookie-protected application page after validated OIDC exchange.
Neither this plan nor earlier administrator portal access is evidence that
this application journey passed. Root retains original D01/D05 gate decisions.

### Fixed source and earlier evidence actually inspected

Current guide/protocol/fixture source was read through Git objects at published
**`9cefe7a56425bb73c17753e8766d92320b77da3b`**; no merge into this worktree
was performed. Whole-file identities below are separate from the stated body
review ranges, and are not tests or build-provenance reproduction.

| Fixed input | Blob / SHA-256 | Body inspected for this proposal |
| --- | --- | --- |
| `docs/essentials-guide.md` | `e48371d3a34a7b6441cbcfce700daf954aa5f2ae` / `9df3c2861984751d48b28d284cfb47c07801fd3934072e2ccb829edeb35579a0` | Section 3 completely; relevant section 2 setup/sign-in prerequisites. |
| `docs/oidc-profiles.md` | `d9f222a7eec7b2998db800ec15d6b1bfc2b7cee5` / `ed1ae74c792e5e35da8d8a939cbe1b37a58392519f808d79066957451528d29b` | Complete protocol guidance. |
| `tools/browser/fixture.js` | `cb9254826d69096cd1c78472078cffd1cdf82294` / `b886dac64616015a791f48e2784a6adf84733e7f2f5a1b2dabb2e68e3172bc8b` | Complete fixture; its RP answers every path with the same stub page. |
| `tools/browser/signin.spec.js` | `50bf9a0623d4c72211d14ce0cccf1ac8afbd4dfa` / `d24ed29b9dcad2d143999c0df2fd328da8a62239bec9f3f8a0124df402518bb5` | Selected authorization/PKCE/password/consent/callback/token assertions; not a complete body review. |
| `tests/browser.rs` | `82292b8d6e91b107797d7fef622ecd8e7168e710` / `e3de993b1bb369bf181b9e103ec0715e66c7fc7ade9fb214d0fefe55ce8f3736` | Complete real-browser RP fixture, including native `jsonwebtoken` verification. |
| `scripts/recovery-drill-oidc.py` | `3be747d03146f1bcaa3ec012ee8d173b61fa737d` / `f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d` | Complete module: bounded request/JSON/JWK/native OpenSSL verification, callback, protected route, service-session login and cleanup. |
| `crates/riauthctl/src/admin.rs` | `58b316890bb6bd04d331088164a94ae39d2d6a9c` / `d67ceb258bd3e2b66b8aebce4cc8af65630bb4b7c22ca2379e4b2b4122811cfd` | Selected `ClientCommand::Create`, `SecretFile` and `protect_secret` bodies. Identical blob in c01. |

The executable source baseline is **`c01c39ab4e092423d5522bedc50fff87656d8c0a`**.
Selected c01 `src/management.rs` direct-create response, `src/model.rs`
`Client::view`, `src/oidc.rs::authenticate_client`, and riauthctl global/session
options were read. Exact blobs: management `8ff588258fd859949015604518003c5950bcd2ad`,
model `8daa94fa28ae30efce78afe587a0bd1524d4f297`, OIDC
`326072f73e1a24cf079d4dd39fe42380e49ddd13`, client main
`d91107cf667e9a2ff60b261e7edc41270367eb52`, client session
`51995aeae62692b5ed3f3a0c69fa80bb1b0aad3d`. The protocol guide also has the
same blob in c01; the newer Essentials prose has a distinct older c01 guide
blob already recorded above. This is a newer printed guide against matching
accepted c01 product artifacts, not a claim of binaries built at 9cefe7a.

The complete added original D01 disposition was read at
`33d4b76fa0a036cd7701663810ee138eccd9d6a3`, walkthrough-report blob
`49f11d5fec28643afafc3ec1c1ab97f8ddf55df1`, as a separate root-reviewed input.
It recommends original-scope completion without asserting every optional
workflow executed. The accepted actual R05 appendix at
`0cc3515eb54a3bcbef83e7fbe03a00ddbcea6cbc`, report blob
`e07cba637b9108aff0c146b9d58e7135feb51a55`, was read for its 19-check outcome,
native verifier/provider and public-client/service-session limitations. Its
helper blob is exactly the fixed module above. No R05 check was repeated.
The earlier D05 addendum `bc3fd18e610b352e782c7015b3d815c33ee88603` and
all c01-dated evaluation values remain unchanged.

### Why a small adapter is needed; exact prospective reservation

The JavaScript RP stub does not exchange/validate tokens or create protected
access. The Rust real-browser fixture validates signed tokens, but requires a
test build, uses a public client/random callback, starts Chrome directly and
completes via terminal approval. The recovery module has executed signature,
claims, userinfo and protected-cookie primitives, but its constructor starts a
random-port RP for a `127.0.0.1` issuer and its `login` uses a service session.
None directly supplies this printed confidential/browser task.

Request root's **source reservation for exactly one new file**:
**`scripts/d01-confidential-browser-demo.py`**, plus continued append-only
evidence in this report. This helper path is absent from fixed 9cefe7a and the
current worktree. No existing helper/test/example, manifest, dependency,
feature, workflow, approval, config, state, diagnostic, guide or D05 artifact
needs editing. The helper's future commit/hash must be reviewed separately
before runtime; no source hash or passing helper result exists yet.

Proposed surgical semantics, all confined to the new local-only helper:

1. Fixed issuer `http://localhost:9000`, client `local-demo`, bind
   `127.0.0.1:3000`, public origin `http://localhost:3000`, callback `/callback`,
   scopes `openid profile`. Refuse an occupied port; never select another port
   or retry. No client registration, IdP login/approval, CLI session input,
   workflow, tenant, passkey or management API implementation in this helper.
2. Import the **hash-checked fixed recovery module** without calling its
   constructor, `login` or public-client `callback`. Import starts nothing.
   Use a minimal verifier context with issuer/client/workspace and checked
   native OpenSSL properties; delegate to its existing `discovery`,
   `callback_fields`, `verify_id_token`, `request`, strict JSON/base64 and
   constant-time helpers. Existing JWK-to-SPKI conversion feeds native
   `openssl dgst -sha256 -verify`; add no signature algorithm or custom crypto.
3. Read the CLI-issued file with no-follow/regular-file/owner/0600 checks,
   at most 256 KiB and duplicate-key rejection. The direct response schema is
   `{client: {...}, client_secret: ...}`: require `client.client_id` equal to
   `local-demo`, confidential true, service false, enabled true, exact callback
   and scopes, unset token-auth override, and a bounded nonempty secret. The
   CLI reserves a new 0600 file and saves the complete response before removing
   `client_secret` from stdout. Do not invent a top-level `client_id` schema or
   disclose the credential. Never place a secret in argv or environment.
4. Serve a fixed, self-contained “Local demo” page and a real sign-in action.
   One same-origin form POST initiates one authorization-code flow. Generate
   fresh unpredictable state, nonce and S256 verifier/challenge in memory;
   bind the pending attempt to a distinct HttpOnly, SameSite=Lax browser flow
   cookie. Use a 180-second pending deadline. No authorization request from a
   service process: the browser follows the redirect and performs sign-in and
   consent. No terminal/session bearer or `decision=approve` substitution.
5. Enforce exact Host/origin, bounded request/header/query inputs, singleton
   callback parameters and state/issuer/flow-cookie binding; consume the
   pending callback once. Reuse the bounded no-proxy/no-redirect request helper
   for confidential code exchange, adding `client_secret` to the POST form
   with client ID, exact callback and verifier. **`client_secret_post` is
   accepted for the printed unset-auth-method confidential client in c01**;
   this needs no settings file or advanced auth configuration.
6. Require validated discovery issuer/endpoints, advertised S256/RS256 and
   confidential POST support. Reuse the checked bounded JWKS selection,
   public RSA key/native RS256 signature verification, issuer, scalar audience
   `local-demo`, nonce, issuance/expiry times, subject and `at_hash` checks.
   Fetch userinfo with the new access token and require subject equality.
   Establish the app cookie only after every check succeeds. JWT payload
   decoding or a nonempty JWKS is insufficient.
7. Generate a fresh, distinct HttpOnly/SameSite=Lax/Path=/ application cookie.
   Check only that app cookie in constant time, rejecting duplicate own-cookie
   inputs while tolerating unrelated localhost cookies: browser IdP cookies
   also cross localhost ports. Deny `/protected` without a valid app session
   (403); accept it only after verified callback (200). This HTTP-only fixture
   is no production Secure-cookie, TLS or application authorization claim.
8. Callback responds with a redirect to query-free `/protected`, not a page
   echoing code/claims. Use fixed HTML, no external assets, no-store,
   no-referrer and restrictive CSP. Silence HTTP access/error logging and
   native-verifier output; fail with finite tags, never raw exceptions or
   request contents. Clear in-memory secret/protocol/session references and
   remove private verifier temporaries in finally; do not claim Python memory
   erasure. A single redacted result contains verification booleans, stage
   IDs/statuses, public source/provider hashes and cleanup data only.

### Matching artifacts and exact proposed commands — NOT EXECUTED

Use only the already accepted matching Essentials/base-client artifacts under
`/Users/dominik/orca/projects/riAuth-public-preview-local-module-boundaries-wave27/target/d01-essentials-c01c39a/aarch64-apple-darwin/debug`:

| Executable | Required SHA-256 before future use |
| --- | --- |
| `riauth` | `7abf745c10691a012a1918c83089168d0e0d88b42764dbf8ed538b90c828b606` |
| `riauth-maintenance` | `86490c7f71de6b7ae9d4dabdf9060a8757100f3aedbdaa670284d9e1206a2a95` |
| `riauthctl` | `bfbbb322f1a66d0ac9998beb9fb5838097cea0442cea3fd3a1d057fcb3f600cf` |

These hashes are accepted earlier observations, not fresh artifact hash
checks in this source-only phase. Future preflight must rehash all three,
the newly reviewed adapter, and the fixed imported helper; refuse mismatch.
Use Python 3.11+ stdlib and the existing native `/opt/homebrew/bin/openssl`:
accepted R05 provider **OpenSSL 3.6.4 25 Aug 2026**, executable SHA-256
`67a83dd6d6d747d50c5d296dffb23e32bae9a2c588c93ae2d77e4c607b455c72`.
Recheck resolved provider identity/version with a 5-second deadline. No
dependency installation, Cargo or substitute binaries.

After separate source and runtime release, the following is the exact proposed
argv/path schedule, supervised in the existing worker lifecycle. It is a plan,
not a runnable helper already delivered. `$D01_BIN` denotes the directory
above; `$D01_LAB` denotes one fresh private own-worktree directory below.

```sh
umask 077
D01_LAB=$(mktemp -d "$PWD/deployment-private/d01-confidential-browser.XXXXXXXX")
mkdir -m 700 "$D01_LAB/xdg" "$D01_LAB/deployment-private" "$D01_LAB/rp"
git show 9cefe7a56425bb73c17753e8766d92320b77da3b:scripts/recovery-drill-oidc.py > "$D01_LAB/recovery-drill-oidc.py"
env XDG_CONFIG_HOME="$D01_LAB/xdg" "$D01_BIN/riauth-maintenance" \
  --config "$D01_LAB/riauth.toml" init --issuer http://localhost:9000 \
  --listen 127.0.0.1:9000 --data-dir data --admin admin
env XDG_CONFIG_HOME="$D01_LAB/xdg" "$D01_BIN/riauth" \
  --config "$D01_LAB/riauth.toml" serve
curl --fail --max-time 5 http://127.0.0.1:9000/readyz
env XDG_CONFIG_HOME="$D01_LAB/xdg" "$D01_BIN/riauthctl" \
  --server http://localhost:9000 login admin
env XDG_CONFIG_HOME="$D01_LAB/xdg" "$D01_BIN/riauthctl" \
  --server http://localhost:9000 client create local-demo --name 'Local demo' \
  --confidential --redirect-uri http://localhost:3000/callback \
  --scope openid,profile --secret-file "$D01_LAB/deployment-private/local-demo-secret.json"
env XDG_CONFIG_HOME="$D01_LAB/xdg" "$D01_BIN/riauthctl" \
  --server http://localhost:9000 discovery
env XDG_CONFIG_HOME="$D01_LAB/xdg" "$D01_BIN/riauthctl" \
  --server http://localhost:9000 whoami
python3 -B scripts/d01-confidential-browser-demo.py \
  --workspace "$D01_LAB/rp" \
  --secret-file "$D01_LAB/deployment-private/local-demo-secret.json" \
  --verifier-helper "$D01_LAB/recovery-drill-oidc.py" \
  --openssl /opt/homebrew/bin/openssl --deadline-seconds 600 \
  --evidence "$D01_LAB/rp-result.json"
```

`serve` and the demo are independently owned foreground child processes of a
single transient private guard, not sequential blocking shell lines or new
RiWork shells. Hash-checked fixed helper materialization is a disposable input,
not an existing tracked-helper edit or source merge. The private parent directory
must exist; create it if absent, without changing permissions of an existing
directory. The guard removes inherited credential/session/config/agent overrides
for these children, sets fresh XDG, and preserves HOME. It generates one synthetic
password privately and supplies the printed hidden init/login prompts; no raw
stdout/session/secret or password input is retained. CLI login is **operator
registration setup only**, never passed to the demo or credited as user sign-in.

### One bounded browser attempt and cleanup contract

The proposed whole fixture is **at most 900 seconds including cleanup**.
Stop active work at 840 seconds and reserve 60 seconds for finally cleanup.
Demo lifetime is at most 600 seconds, pending OIDC attempt 180 seconds,
each protocol/OpenSSL request 5 seconds, each CLI/setup child at most 60
seconds, service/helper readiness at most 30 seconds, and each browser page
transition at most 30 seconds. Entry requires at least **8.5 GiB** free;
one-second monitoring stops at that margin and preserves the **8 GiB floor**.
Check no listener on **9000 or 3000**, including other loopback address families,
before launching, and prove each resulting listener belongs to the recorded
owned process. Never stop an unrelated occupant or change callback ports.

At separately released actual runtime, first read RiWork Cua.ai Driver MCP
descriptions/current state and the applicable cua-driver instructions. One
sole controller may request `browser_prepare` with `isolated_new` and launch
permission, then bind **exactly its returned window/target/tab**. No personal
profile copying/attachment, app install, settings/security changes, permission
automation, accessibility test, alternate provider or raw CDP/Playwright/
AppleScript. Refuse uncertain binding/permissions and report the actual
prerequisite through root; do not silently switch provider.

Through that exact browser, first navigate to query-free
`http://localhost:3000/protected` and observe the expected signed-out refusal;
open the demo page and use its sign-in action. In the fresh isolated profile,
use actual riAuth Username/Password input with an empty optional authenticator
field, submit sign-in, observe actual `Local demo` consent and its requested
scopes, then approve using a fresh returned action ref after the page enables
it. Require arrival at the clean protected page plus the demo's redacted
verification result. No API/CLI session, terminal approval, portal shortcut or
programmatic authentication stands in for those user actions. No physical
passkey or Touch ID prompt is requested. A missing consent step or unexpected
page is a failed checkpoint to report, not permission to invent a bypass.

No screenshots/video or raw credential/token/code/state/nonce/verifier/cookie/
subject/query URL logging. Sanitize Driver result metadata and page URLs before
any visible or durable output; retain only public path, fixed page text, fresh
snapshot/action identifiers and outcomes. Do not dump tool responses, input
values, browser storage, response bodies, exception messages or process argv.
Actual dispatch route/trusted-input limitations must be reported truthfully.

Stop on the first unexpected product/printed-step/protocol/Driver failure and
propose a correction before any retry. One fixture, one registration/flow,
no automatic rerun or replacement isolated profile. In finally: stop the demo
and server using only recorded owned PID/start identities (SIGINT, wait up to
8 seconds, TERM up to 5 seconds, KILL only if still proven owned); reap children,
close RP socket/thread, and close only the proven Driver-owned browser/session
through Driver. Read back absence of owned processes and both listeners. Retain
only redacted result/cleanup metadata in a new exclusive 0600
`deployment-private/d01-confidential-browser.redacted.json` (refuse an existing
file at preflight), remove the lab/XDG/store/
credential/session/verifier temporaries, and prove removal. Driver profile-file
erasure is not claimed without separate evidence. Send immediate actual exit,
first failure if any, cleanup/disk observations and **runtime/desktop release**
to the explicit project orchestrator before a separate evidence append/commit.

### Gate interpretation and actual checks in this phase

Section 3 explicitly says to start the application and that registering a
client does not start it. Supplying a local RP is an **explicit task input**;
the missing turnkey launch command is not an observed product/printed-step
defect. This proposal makes that input concrete without silently requiring
advanced configuration or changing the accepted original D01 disposition.
If separately implemented/released and passed, it would add one confidential
browser application journey; it would not refresh any historical evidence or
prove installation wrappers, invitations, ordinary non-admin users, hardware
passkeys, all optional tasks, tenants, current whole CI, release or deployment.
This worker's previous independent password browser run and proposed app run
are authored actions, not independent verification of its own evidence.

Actual work here: fixed Git-object/source reads above, whole-file blob/SHA-256
identity checks, exact c01 CLI/protocol blob comparison, prospective-helper
absence, and original-report byte equality/prefix inspection. One source grep
first used an unquoted zsh glob and was corrected with quoted paths; a read-only
`riwork orchestrator --help` returned exit 2 (unsupported command), followed by
the documented top-level help. Neither was product execution or a checkpoint
failure. No product binary/provider rehash or invocation, helper execution, test, Cargo,
browser/Driver call, listener or runtime resource was acquired in this phase.
All accepted receipt/header/PAM/removal/audit protections, held Group and
nonrenewed 60-second/paused-I/O limits remain unchanged. Root alone reserves
implementation/runtime, reviews/integrates/publishes and decides original gates.

Append verification actually passed: Python static checks preserved all 11586
prior bytes and their SHA-256, checked the seven tabled blob/SHA-256 identities
and three commit object types, confirmed both prospective/unmaterialized helpers
absent and only this report changed, and checked balanced fences/final newline.
`git diff --check` passed. These are document/source-identity checks, not helper,
protocol, desktop or product-runtime evidence.

## Reserved confidential demo implementation: source-only evidence — 2026-10-02

Root explicitly reserved `wave30_D01_confidential_browser_helper`: exactly one
new `scripts/d01-confidential-browser-demo.py`, plus append-only evidence here,
in the same project/worktree. **Source is implemented; all execution remains
HELD pending root source review and a separate explicit runtime release.**
No helper import/execution, CLI setup, listener, native provider, Driver, Cargo
or product test was invoked. No runtime/desktop slot was acquired or released.

The implementation is committed separately from this evidence append:

| Source artifact | Exact identity |
| --- | --- |
| Source commit, parent `9e0f0dbeff260d674187ddd23399bcb413498904` | `0b0d15cda6e6c61388ce40338de707f84851989f` |
| Sole added file | `scripts/d01-confidential-browser-demo.py`, mode `100644`, 651 lines, 29933 bytes |
| Source blob | `cbbaaea8afa667112546c5ee6b85e945d6f306bd` |
| Source SHA-256 | `d8446bfdf22f2a345b019673fe93828d5f75024a87b530b100eb6c027da9a863` |
| Imported verifier, unchanged fixed Git object | Commit `9cefe7a56425bb73c17753e8766d92320b77da3b`, blob `3be747d03146f1bcaa3ec012ee8d173b61fa737d`, SHA-256 `f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d` |

The existing recovery helper is absent from this older worktree and was not
materialized, imported or edited. The new script checks private file mode and
the exact verifier hash before compiling/executing those verified bytes **at
future runtime**; it does not reread arbitrary code or create bytecode files.
Only `LocalRelyingParty.discovery`, `callback_fields` and `verify_id_token`
are delegated with the minimal context they require. Neither its constructor,
public-client callback nor service-session `login` is called. Native RS256
verification and JWK conversion remain in that pinned, previously checked
module; no custom signature implementation or dependency was added.

### Implemented source behavior, not observed runtime results

- Fixed issuer/client/origin/callback/scopes and confidential POST match the
  accepted plan. The 0600, owner-only, regular, no-follow CLI credential file
  is bounded to 256 KiB, parsed with the pinned duplicate-key guard, and
  validated against the direct `{client, client_secret}` response schema.
  The secret is used only in token POST data, never argv, environment, HTML,
  output or evidence. Existing creation receipts/headers are untouched.
- One same-origin empty form POST produces the browser authorization redirect
  with fresh state, nonce, S256 and a browser flow cookie. One callback consumes
  that pending state before exchange and requires the matching flow cookie,
  exact issuer and state. The code exchange includes the confidential secret,
  exact callback and verifier. There is no API/session/terminal approval path.
- Discovery endpoints and algorithm/auth-method advertisements are checked.
  The pinned method verifies JWKS/public RSA/RS256, scalar audience, issuer,
  nonce, issuance/expiry, subject and access-token hash; userinfo subject must
  agree. Only then is the fresh app cookie created. The response clears the
  flow cookie and redirects to query-free `/protected`.
- `/protected` responds 403 without a valid app cookie and 200 only with the
  verified app session. Its fixed HTML exposes no identity/token data. Own
  cookie duplicates are rejected; unrelated localhost IdP cookies are ignored.
  The final pass condition requires the before-cookie refusal, authorization
  redirect, exchange/validation/userinfo and after-cookie acceptance. Numeric
  HTTP observations are stored separately from boolean checks. They initialize
  to null/false and are updated only after the corresponding future action.
- Request line, header total, header line/count, cookie size/count, body and
  query limits are explicit; only bodyless GET and the exact empty form POST
  are accepted. Host is exact, authorization/transfer/expect headers are
  refused, Origin is exact for POST, and callback parsing is the pinned bounded
  singleton parser. Unknown auxiliary paths get fixed 404 HTML without starting
  another flow. HTTP request/error logging is silent, and HTML/headers use
  no-store/no-referrer/CSP with no external assets or raw request interpolation.
- One synchronous HTTP server binds only `127.0.0.1:3000`, with no alternate
  port or retry. A POSIX main-thread alarm samples disk at intervals of at most
  one scheduled second, enforcing the 8.5 GiB stop margin, overall helper cap
  of 600 seconds, pending cap of 180 seconds and operation deadlines. Header
  input/response, provider/discovery/token/userinfo and **combined JWKS/native
  signature verification** each have a five-real-second alarm; the delegated
  module's existing socket/OpenSSL caps remain. Nested operations retain the
  earlier deadline. No new thread, external service or browser controller exists.
- Fixed failure tags and controlled stages replace raw exceptions. The original
  failure is retained if cleanup/evidence writing also fails, with separate
  cleanup/evidence failure fields. Finally closes the active connection and
  listener, restores owned alarms, clears private references and checks the
  initially empty verification workspace for leftover temporaries. No Python
  memory erasure is claimed. A new exclusive 0600 result file contains public
  source/provider hashes, fixed status/check metadata and resource observations;
  stdout contains only readiness PID/port and finite result fields.

### Actual static checks and limits

All assigned source checks passed. Python read the new file as bytes and used
`ast.parse` plus in-memory `compile` on the AST; **the resulting code object was
never executed**, and the target module was never imported. Checks verified
stdlib-only imports, fixed issuer/client/origin/cookie/bounds/provider/verifier
literals, the guarded entry, the three permitted native-method references,
absence of RP construction/service-session approval paths, and the pinned
verifier Git-object SHA-256. The preceding report exactly matched the complete
`9e0f0db` input before this append. `git diff --cached --check` passed, and
the source commit's sole path is the added helper. No static check failed;
draft refinements to deadline nesting, combined verification bounds, cleanup,
first-failure retention and argument matching were made before that commit.

These are source/identity/scope checks, not evidence that native verification,
signals, token exchange, any browser transition, refusal, cleanup or timing
has executed successfully. The earlier seven whole-file identity checks and
the historical R05/D01 outcomes retain their dates and scope. No current
whole-CI, installation, release, non-admin, invitation, physical-passkey,
tenant, all-workflow or independent verification of this worker's own actions
is inferred from this implementation.

Root's source review should assess these concrete limits before runtime:

1. Python 3.11+, a fresh 0700 lab/empty RP workspace, POSIX alarms on the main
   thread and the exact native OpenSSL path/hash/version are required. A changed
   provider, occupied port, invalid private input or expired/duplicate flow
   fails closed; there is no compatibility/dependency/provider fallback.
2. Verification deliberately uses the existing fixture's strict RS256/JWT,
   scalar audience, RSA key-size/exponent and `at_hash` checks, rather than a
   general OIDC library for arbitrary issuers or token profiles. The combined
   five-second verification cap is more restrictive than allowing two successive
   five-second calls. None of this has been exercised with the new adapter yet.
3. Helper metadata proves only its future HTTP/verification observations.
   Actual password entry and consent cannot be inferred from those booleans:
   the separately released **RiWork Cua.ai Driver MCP-only** trace must observe
   them on the exact isolated target. Driver descriptions/current state remain
   deferred to that release; no GUI provider or permission was accessed here.
4. The external owner still owns IdP/browser processes, the fresh lab/XDG and
   the inclusive 900-second fixture deadline with the 840-second active stop
   and 60-second cleanup reserve. The helper closes its own socket and clears
   references; it does not delete the caller's lab, stop IdP/personal browsers
   or prove a vanished listener/PID tree. Its elapsed field is sampled after
   socket cleanup, before final evidence writing; whole-fixture timing and
   final listener/process/lab readbacks belong to the parent guard.

The exact argv and matching c01 artifact hashes in the prior proposal remain
the runtime request, with source commit/hash above added to preflight. Root
must separately review this implementation and explicitly release that one
fixture; this source reservation grants no execution. Existing guide/D05
artifacts, product/config/state/approval/workflow/tests and accepted contracts
remain unchanged. Root alone integrates/pushes/statuses and decides D01/D05
original gates; O06 remains closed. No new worker/task/worktree/shell, other
worker contact, main mutation or merge/reset occurred.

Evidence-append checks actually passed: all 31296 bytes of the prior plan/report
and all 11586 original browser bytes remain unchanged; balanced Markdown fences,
final newline, unchanged committed source SHA-256/blob and single-report pending
diff were verified. `git diff --check` passed. No link/build/test or helper
runtime was substituted for these static checks.

## Released fixture refused during prerequisite checks — 2026-10-02

**No application/browser checkpoint ran.** Root reviewed the source and evidence,
published them at `ae8937800254a1ad4296ea257de1eccc4780e45b`, and explicitly
released the single `wave30_D01_confidential_browser_helper` fixture. One
preflight invocation then exited **1** at a native-provider return/version
assertion. It stopped before operator setup or any helper/browser launch.
There was no retry, alternate provider, source correction or substituted pass.
Desktop/operator reservation release was sent immediately before this append;
the independent remote A09 Cargo lane was neither acquired nor released here.

### Exact inputs and observed checks

Execution input remained own HEAD
`6b3a1790ff80cb205bb403854725a779541adbac`; no merge to published main occurred.
The published helper is blob `cbbaaea8afa667112546c5ee6b85e945d6f306bd`, exactly
the source reviewed above. Published Essentials is unchanged blob
`e48371d3a34a7b6441cbcfce700daf954aa5f2ae`, SHA-256
`9df3c2861984751d48b28d284cfb47c07801fd3934072e2ccb829edeb35579a0`;
its sections 2/3 were reread for the planned normal operator registration and
browser task. These current reads do not refresh earlier executions.

All three matching prebuilt c01 artifact hashes were recomputed successfully:

| Artifact | Bytes / matching SHA-256 |
| --- | --- |
| Essentials `riauth` | 183038592 / `7abf745c10691a012a1918c83089168d0e0d88b42764dbf8ed538b90c828b606` |
| Essentials `riauth-maintenance` | 56499296 / `86490c7f71de6b7ae9d4dabdf9060a8757100f3aedbdaa670284d9e1206a2a95` |
| Base `riauthctl` | 20406784 / `bfbbb322f1a66d0ac9998beb9fb5838097cea0442cea3fd3a1d057fcb3f600cf` |
| New demo source, read only | 29933 / `d8446bfdf22f2a345b019673fe93828d5f75024a87b530b100eb6c027da9a863` |
| Fixed verifier, Git object only | Blob `3be747d03146f1bcaa3ec012ee8d173b61fa737d` / `f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d` |

The native provider also **matches its required artifact hash**:
`67a83dd6d6d747d50c5d296dffb23e32bae9a2c588c93ae2d77e4c607b455c72`.
Resolved file: `/opt/homebrew/Cellar/openssl@3/3.6.4/bin/openssl`, 880512 bytes.
This was checked again read-only after refusal to disambiguate the assertion;
that metadata read was not a second provider invocation or fixture retry.

The actual first failing statement followed **one**
`/opt/homebrew/bin/openssl version` invocation, bounded by five seconds and
using the planned restricted environment. It tested both return code zero
and stdout stripped to exactly `OpenSSL 3.6.4 25 Aug 2026`. The combined
assertion failed at Python line 16, and the preflight process exited 1.
Its captured native return code/stdout/stderr were not retained before that
assertion terminated the process. Consequently **the returned version and
native exit code are unknown**: output mismatch and nonzero exit cannot be
distinguished from this evidence. No exact version mismatch string, changed
library, installation defect or provider failure cause is invented. The
preflight tool measured 0.381 seconds for its status/hash/version command.

### Driver prerequisites and unused resources

The cua-driver skill and relevant MACOS/BROWSER/RUNTIME instructions and live
tool descriptions were consulted before Driver reads. Instruction hashes
remain the four values recorded in the original browser report; those files
were rehashed. Initial large combined output was truncated, so bounded
tool-specific schemas/instruction sections were read next. No GUI action was
attempted. RiWork's read-only `cua status` reported daemon **0.30.4**, the
expected CuaDriver executable and ready state. Driver MCP permission read
reported Accessibility and Screen Recording granted; direct ScreenCaptureKit
probe was explicitly skipped. MCP recording state reported disabled/inactive.

No fixture `start_session`, isolated profile preparation, browser binding,
navigation, screenshot, typing, consent or logout occurred. No fresh lab/XDG,
synthetic password, CLI session/secret, store, server/helper process or socket
was created. No maintenance/server/client binary was executed. Only the native
provider version subprocess ran; `subprocess.run` completed and reaped it,
but its PID was not separately recorded. There was no owned persistent process
or browser/session to stop, end or remove. This does not claim cleanup or
erasure of any shared/personal Driver resource.

After refusal, bounded read-only `lsof` returned **exit 1/no listener** for both
9000 and 3000. The planned
`deployment-private/d01-confidential-browser.redacted.json` does not exist;
no helper result was fabricated. The post-refusal capacity sample was
**8.282 GiB free**, below the 8.5 GiB launch/stop margin and above the 8 GiB
floor. This is an additional observed prerequisite preventing launch. The
initial ordered preflight had failed before its disk/socket entry checks and
before starting the one-second fixture guard; no continuous disk minimum or
entry-margin pass is claimed. No resource-consuming setup continued after
that observation, and no cache/artifact/evidence was removed to recover space.

### Release, reporting correction and remaining input

First clock read: `2026-10-02 14:07:20 UTC`. Immediate release and subsequent
read-only resource observations were complete by the `14:08:18 UTC` clock
read. A corrected root handoff was delivered by `14:10:09 UTC`. These are
observation timestamps, not a claimed precisely instrumented whole-fixture
duration. Both explicit-project orchestrator sends exited 0 before this
report append. **DESKTOP/OPERATOR RUNTIME RELEASED; Cargo unaffected.**

The first commentary/handoff incorrectly classified the assertion as an
OpenSSL hash mismatch and said the provider had not executed. That was my
source-line interpretation error. The read-only digest matched, and inspection
of the submitted preflight statement identified the version/exit assertion.
The inaccurate classification was explicitly withdrawn in commentary and a
second immediate root handoff. This appendix preserves that reporting error
and correction rather than silently replacing the first claim.

The next root-coordinated input is an actual bounded provider exit/version
observation retained before any assertion, plus a fresh capacity check at or
above 8.5 GiB before another released fixture. No new diagnostic/provider call
or rerun is authorized by this report. The missing captured values do not
justify relaxing the reviewed version/hash guard or editing the helper, and
the capacity sample does not justify deleting other work. Root decides any
concrete correction/reservation and new runtime release.

No password sign-in, Local demo consent, callback, exchange, native token
validation, userinfo or protected 403/200 result is claimed. Helper invocation
count is **zero**. All earlier D01/R05 observations remain dated, D01/D05
completion remains root-owned, and accepted O06/I10/R05 DONE rows stay closed.
The only tracked change in this turn is this append. No source/helper/guide/
D05/config/state/approval/workflow/test edit, Cargo/build/test, provider switch,
new worker/task/worktree/shell, other-worker contact, merge/reset, main/push or
status change occurred. Accepted receipt/header/PAM/removal/audit/held Group
and nonrenewed 60-second/paused-I/O contracts remain unchanged.

Evidence verification passed: all 40877 bytes of the prior `6b3a179` report
remain an exact prefix; the committed helper bytes/SHA-256 are unchanged.
The pending diff contains only this report append, Markdown fences are
balanced and the final newline is present. `git diff --check` passed. These
document/source checks did not execute the helper or repeat the provider call.

## Separately authorized native-provider diagnostic — 2026-10-02

**The one diagnostic exited 0; the confidential fixture remains held.** Root
received `93643763a1fc3360491a0a4a32424fd353e92cc0`, preserved its first refusal
and reporting correction, and authorized exactly one new native-provider
version observation. That authorization did not release the helper, operator
fixture, desktop or Cargo. This phase is separate from the lost first native
invocation; its return/output values remain unknown.

### Retained observation and provenance

Own source input remains helper commit
`0b0d15cda6e6c61388ce40338de707f84851989f`, blob
`cbbaaea8afa667112546c5ee6b85e945d6f306bd`, SHA-256
`d8446bfdf22f2a345b019673fe93828d5f75024a87b530b100eb6c027da9a863`.
The exact guard and sanitized environment in `setup()` were read directly;
the helper was neither imported nor executed. Ancestor instruction checks
found no `AGENTS.md`. No task/worktree/shell was created.

The diagnostic rehashed `/opt/homebrew/bin/openssl` after resolving it to
`/opt/homebrew/Cellar/openssl@3/3.6.4/bin/openssl`. Its 880512 bytes match
`67a83dd6d6d747d50c5d296dffb23e32bae9a2c588c93ae2d77e4c607b455c72`.
It then invoked the resolved executable with the sole argument `version`,
using exactly the helper's inherited-environment filter: `PATH`, `HOME`,
`TMPDIR`, `LANG`, `LC_ALL` only. All five keys were present; their values were
not printed or copied into evidence. The native process had a five-second
timeout, completed in 0.003956 seconds and was reaped. There was no timeout,
second invocation or provider substitution.

| Retained field | Actual observation |
| --- | --- |
| Numeric native return code | `0` |
| ASCII stdout | `OpenSSL 3.6.4 25 Aug 2026 (Library: OpenSSL 3.6.4 25 Aug 2026)` followed by one newline |
| Full stdout length | 63 bytes; no truncation |
| ASCII stderr | Empty |
| Full stderr length | 0 bytes; no truncation |
| Started UTC | `2026-10-02T14:23:52.077151+00:00` |
| Completed UTC | `2026-10-02T14:23:52.082796+00:00` |

The wrapper opened an exclusive, no-follow, owner-private metadata file before
the invocation, then retained the numeric return code, bounded ASCII outputs,
full byte lengths, digest, timing and disk sample and flushed/fsynced it
**before any output assertion or comparison**. The per-stream retained limit
was 4096 bytes. Readback verified mode `0600`, 1103 bytes and SHA-256
`8f47d8fbd384ccf9746de938ef5c6e9cbba8d0e072484b443e2c2cbc4fa7439d`.
The metadata is
`deployment-private/d01-provider-version-diagnostic-20261002.json`, under the
existing owned `0700` private directory, ignored by Git and not committed.
Its contents contain provider metadata rather than credentials. The wrapper
itself also exited 0. No exception, timeout or cleanup failure was recorded.

After persistence, source-only AST extraction of `OPENSSL_VERSION` and metadata
readback established that the current exact equality is false. The other
existing version predicates pass: return code zero, stdout at most 256 bytes,
stderr at most 4096 bytes and strict ASCII stdout. Source bytes/hash remain
unchanged. These comparisons did not invoke any executable a second time.

Root's prior read-only format-string observation is consistent with this new
stdout, but the new diagnostic does not recover the first invocation's lost
values or prove its exact failure cause. The earlier refusal and my corrected
reporting error remain intact above.

### Exact proposed source reservation, not an edit

The concrete returned evidence warrants only this one-line change at
`scripts/d01-confidential-browser-demo.py:41`:

```diff
-OPENSSL_VERSION = "OpenSSL 3.6.4 25 Aug 2026"
+OPENSSL_VERSION = "OpenSSL 3.6.4 25 Aug 2026 (Library: OpenSSL 3.6.4 25 Aug 2026)"
```

This would retain exact full-text equality after `.strip()` and the existing
path, regular-file/size, artifact hash, ASCII/length and timeout guards. It
would also make the helper's provider-version evidence field contain the
observed complete version text. No prefix match, normalization, alternate
provider, hash relaxation, dependency or other source hunk is proposed. Root
must reserve the source change separately; this turn makes no helper edit.

### Capacity, bounded scope and handoff

The authorized fresh read-only disk sample measured **11969429504 bytes /
11.1474 GiB free**, above the 8.5 GiB launch margin. It is a point observation,
not a continuous floor check or permission to start the fixture. No deletion
or cache/artifact cleanup was performed here.

An ancillary read-only `riwork orchestrator --help` request returned 2 because
that subcommand is unsupported; no state changed. The explicit-project
diagnostic/proposed-hunk handoff used the supported `orchestrator send`
command and exited 0 before this append.

No maintenance/server/CLI/helper, HTTP request, listener, browser, Driver,
session, Cargo/build/test or whole-fixture runtime ran. No fixture reservation
was acquired or released in this diagnostic phase. There was no second native
provider invocation, source correction, guide/product/D05 edit, merge/reset,
main/push/status change, new worker/task/worktree/shell or other-worker contact.
RiWork Cua.ai Driver remains the sole future desktop provider. Root alone
reserves any source correction and a new fixture release and decides D01/D05
completion; accepted O06/I10/R05 DONE rows and prior dated evidence remain
unchanged. The only tracked change is this report append.

Evidence checks passed: all 48637 bytes of prior report commit `9364376` remain
an exact prefix; helper bytes/SHA-256 and the private metadata bytes/hash/mode
are unchanged. The pending diff contains only this report, Markdown fences
and final newline pass, and `git diff --check` passed. These checks repeated
no native invocation or fixture activity.

## Reserved full-version guard correction, source only — 2026-10-02

Root reserved `wave30_D01_provider_full_version_guard` with source approval
true and runtime release false after independently reading the retained
1103-byte `0600` diagnostic. This phase implements only the exact full-text
literal proposed above. It neither repeats that diagnostic nor releases or
runs the confidential fixture. The first invocation's lost values and my
initial reporting error remain preserved in their dated phase.

Source commit **`16395f1be5a65b0d3cb0a1ad2d88daa0c17b7e1b`**, parent
`5c7665c0e1cd9f23ad6c8d912cdc67ab27609a2c`, changes exactly one line in
`scripts/d01-confidential-browser-demo.py:41`:

```diff
-OPENSSL_VERSION = "OpenSSL 3.6.4 25 Aug 2026"
+OPENSSL_VERSION = "OpenSSL 3.6.4 25 Aug 2026 (Library: OpenSSL 3.6.4 25 Aug 2026)"
```

The resulting helper is mode `100644`, 651 lines / 29970 bytes, blob
`37c9136850c2522da1eaa5f31f8b94e70acc0886`, SHA-256
`f94af72ab613bc332ea684e5c4244c38d53c6540e1ea3d883938aecd265ec6fc`.
It still requires exact complete version equality after `.strip()`. Its
provider evidence field will record that full constant if a later separately
released helper run succeeds. This source-only check claims no such run.

Actual static checks passed before the source commit:

- Starting helper bytes were exactly the `0b0d15c` source blob and SHA-256
  recorded above. Diagnostic readback matched its 1103 bytes, `0600` mode,
  SHA-256 `8f47d8fbd384ccf9746de938ef5c6e9cbba8d0e072484b443e2c2cbc4fa7439d`,
  exit 0 and exact 63-byte stdout. These were reads of retained data.
- Original and changed literal each occur once, at line 41. Replacing the
  original with the new literal reconstructs the entire new source; reversing
  that replacement reconstructs **all original `0b0d15c` bytes exactly**.
- Both sources parsed with `ast.parse`. Replacing only the changed AST string
  value with its prior value produced the exact original AST dump without
  source-position attributes.
- `compile(source, filename, "exec", dont_inherit=True)` succeeded in memory.
  Its code object was not executed; the helper was not imported, and no bytecode
  artifact was written.
- Pending scope was exactly the helper path; the whole prior report remained
  unchanged. `git diff --check` passed. The source commit contains only that
  path, one insertion/one deletion; tracked state was clean after commit.

The byte-reversal proof covers every other equality/path/hash/ASCII/length,
sanitized-environment, timeout, native verification, browser-flow and cleanup
guard: their source bytes did not change. No guard relaxation, provider switch,
custom crypto, dependency or second hunk was introduced.

This phase makes no new provider call, helper/maintenance/CLI/server execution,
HTTP request, listener, browser/Driver/session interaction, Cargo/build/test or
fixture acquisition/release. The prior disk sample remains a dated point
observation; it was not refreshed or substituted for a future fresh launch
check. The independent A09 Cargo owner is unaffected.

Root must review/publish this source and separately approve any one new
confidential fixture with a fresh disk sample above 8.5 GiB. Runtime remains
held. Existing receipt/header/PAM/removal/audit and held Group/nonrenewed
60-second/paused-I/O contracts, D01/D05 root-owned decisions and closed
O06/I10/R05 rows remain unchanged. No guide/product/D05/test/config/state edit,
main/push/status action, merge/reset, new worker/task/worktree/shell or other
worker contact occurred. This evidence is an append to the existing report,
committed separately from the source.

Evidence-append verification passed: all 54467 bytes of the preceding report
remain an exact prefix; committed source and retained diagnostic bytes/hash/mode
are unchanged. Pending scope contains only this report, balanced Markdown
fences and final newline pass, and `git diff --check` passed. No runtime was
used for these document checks.

## Separately released full-version fixture: stopped before browser — 2026-10-02

**The one released fixture exited 1; no application/browser checkpoint ran.**
Root reviewed/published source `16395f1` and evidence `fb9f474` at main
`9b8956f7b2a9961b14e313fa57c0f5214a136772`, with root review `3a7aa16`, then
released exactly one new confidential Local demo fixture. This phase used that
reviewed helper without edits. Operator setup succeeded, but the helper failed
with `request_invalid` before any isolated browser/profile/session action.
The outer guard stopped immediately, cleaned its owned resources and exited 1.
There was no retry, correction, alternate provider or substituted browser pass.
Desktop/operator release was sent before this evidence append; Cargo was
neither acquired nor released.

### Exact inputs and successful setup observations

Own HEAD at entry was `fb9f474db779fb655b43026f1089be19661bb645`; no merge to main
occurred. The helper remained blob `37c9136850c2522da1eaa5f31f8b94e70acc0886`,
651 lines / 29970 bytes, SHA-256
`f94af72ab613bc332ea684e5c4244c38d53c6540e1ea3d883938aecd265ec6fc`.
All three c01 artifact hashes in the table above were freshly recomputed and
matched before use. The pinned `9cefe7a` verifier bytes were read from Git,
matched `f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d`,
and were materialized only as a disposable owner-private lab input.

Published guide sections 2/3 were reread directly for normal init/password
setup and confidential registration. Guide blob remains
`e48371d3a34a7b6441cbcfce700daf954aa5f2ae`, SHA-256
`9df3c2861984751d48b28d284cfb47c07801fd3934072e2ccb829edeb35579a0`.
The published OIDC profiles artifact was identity-checked as blob
`d9f222a7eec7b2998db800ec15d6b1bfc2b7cee5`, SHA-256
`ed1ae74c792e5e35da8d8a939cbe1b37a58392519f808d79066957451528d29b`;
this identity check is not a new whole-body review or protocol execution.

The guard required unused 9000/3000 listeners and fresh capacity at least
8.5 GiB before setup, then created exactly one fresh owned `0700` lab with
private XDG, credential directory and RP workspace. Child environment retained
only the five sanitized inherited keys plus fresh `XDG_CONFIG_HOME`. A synthetic
password was supplied through hidden PTY prompts, never argv/environment or
printed output. CLI stdout/stderr were discarded after bounded in-memory
handling. No policy/config/review/header/receipt bypass was supplied.

| Operator setup command | Actual exit / observation |
| --- | --- |
| `riauth-maintenance ... init --issuer http://localhost:9000 --listen 127.0.0.1:9000 --data-dir data --admin admin` | 0; two hidden password prompts |
| `riauth ... serve` | Started owned PID 15737; only listener owner on 9000 |
| `GET http://127.0.0.1:9000/readyz` | 200 |
| `riauthctl --server http://localhost:9000 login admin` | 0; one hidden password prompt; operator setup only |
| `riauthctl ... client create local-demo --name 'Local demo' --confidential --redirect-uri http://localhost:3000/callback --scope openid,profile --secret-file ...` | 0; new credential file mode 0600 |
| `riauthctl ... discovery` | 0 |
| `riauthctl ... whoami` | 0 |
| Reviewed helper, exactly one invocation | Owned PID 15749; only listener owner on 3000; later exit 1 |

The native provider resolved to the same pinned 880512-byte binary and matched
`67a83dd6d6d747d50c5d296dffb23e32bae9a2c588c93ae2d77e4c607b455c72`.
One five-second preflight `version` call returned 0, 63 ASCII stdout bytes
containing the exact full app/library text plus newline, and zero stderr bytes;
there was no timeout. Before exact comparison, the guard flushed/fsynced that
observation into exclusive `0600` metadata:
`deployment-private/d01-confidential-browser-provider-20261002.json`,
359 bytes, SHA-256
`6fad035f1897b5287b2ebd812405fcabf6a33b72d2d4f498f3bca7081eb719ab`.
The helper independently performed its existing version check and reported
`provider_identity_verified: true` with the full pinned version. Thus this
fixture includes the retained preflight version call and the helper's own
version call; it is one fixture, not a claim of one total native invocation.
No RS256 verification execution is credited.

### Actual helper failure and scope of diagnosis

The helper's redacted result is **failed**, exit **1**, stage **`request`**,
tag **`request_invalid`**, elapsed **9.247 seconds**. Its first three checks
passed: `credential_private_validated`, `provider_identity_verified` and
`discovery_verified`. Every subsequent check is false:

| Browser/protocol check | Actual value |
| --- | --- |
| `protected_without_cookie_denied` | false |
| `authorization_redirect_issued` | false |
| `state_issuer_flow_cookie_verified` | false |
| `confidential_s256_exchange_verified` | false |
| `rs256_jwks_issuer_audience_nonce_time_access_hash_verified` | false |
| `userinfo_subject_verified` | false |
| `protected_with_fresh_cookie_accepted` | false |

All six helper HTTP status fields are null: protected before/after,
authorization redirect, callback, token exchange and userinfo. No worker browser
request to the RP, password sign-in, empty-factor submission, Local demo consent,
callback or fresh protected 200 occurred. The readyz/discovery/operator calls
above are not user authentication substitutes.

Source-only inspection after cleanup read `Handler.handle_one_request`,
`send_error`, cookie parsing and routing. `request_invalid` can arise from HTTP
parsing or several finite header/target/method/cookie guards. The retained result
does not distinguish those branches, identify a request origin or retain a
request line/header. It does not justify attributing the failure to the product,
an external actor, a specific browser request or a particular predicate.
No exact guard correction is proposed without that missing cause evidence, and
no raw request/query logging or relaxed guard is authorized here. Root must
adjudicate the helper failure and any separately scoped diagnostic/source
reservation before another fixture. This phase performed no diagnostic rerun.

### Driver preparation, budget and cleanup

Before any GUI action, the cua-driver skill, relevant MACOS/BROWSER/RUNTIME
instructions and live tool schemas were consulted. Initial large tool/file
output was truncated; narrower schema and relevant-section reads followed.
The four instruction hashes remain the values recorded above. RiWork read-only
status reported Driver **0.30.4**, the expected CuaDriver executable and ready.
MCP permission read reported Accessibility/Screen Recording granted, with
direct capture unprobed; MCP recording state was disabled/inactive. No history
tools were advertised.

Because the helper failed first, no `start_session`, `browser_prepare`, binding,
navigation, click/type, screenshot, recording, browser logout or `end_session`
was issued. No Driver-owned browser/session was created to kill or end. This
does not claim cleanup of shared or personal Driver resources. The sole future
desktop provider remains RiWork Cua.ai Driver MCP.

The outer active/cleanup budgets were 840/60 seconds; the helper cap was 600,
pending cap 180, native/HTTP caps five seconds and CLI caps 60. The inclusive
outer elapsed value was **323.837 seconds**, measured from a conservative start
that includes a 60-second allowance for initial instruction/tool reads before
the first recorded clock. The actual cleanup completed at
`2026-10-02T14:57:48.022112+00:00`, within the 900-second bound. It is not a claim
that the helper itself ran for that duration.

One-second outer disk monitoring began at guard preflight, with **11 samples**
and minimum **11915264000 bytes / 11.096954 GiB**. The helper's own scheduled and
phase-entry observations recorded **53 samples**, minimum **11915100160 bytes /
11.096802 GiB**. Both observed minima exceeded the 8.5 GiB stop margin and 8 GiB
floor. No disk deletion was performed. No continuous minimum is claimed during
the earlier instruction-read period before the guard.

The guard's `finally` reaped its recorded children, closed processes/listeners
and removed the whole fresh lab/XDG/store/credential/password/session/verifier
input. The helper reported `listener_closed`, `connection_closed`,
`private_references_cleared` and `verifier_temporaries_removed` all true.
Fresh readbacks found guard PID 15561, server PID 15737 and helper PID 15749
absent, both port checks exit 1/no listeners, and the lab path absent. Server
termination/reaping is proven; its numeric exit was not retained and is not
claimed. No unrelated process was killed. No Python memory or Driver profile
erasure is claimed.

The retained outer/helper result is exclusive `0600`,
`deployment-private/d01-confidential-browser.redacted.json`, 3374 bytes,
SHA-256 `04074d4aed82df1127666305c9ab6e41faaa36cab52f34c61562b29da7ebf560`.
It preserves the helper failure beneath outer stage `browser_checkpoint`,
tag `rp_nonzero`; no result was rewritten as passed. Both new metadata files
are private/ignored, and no secret/query/raw exception or screenshot is retained.

### Immediate release and remaining gate

The explicit-project orchestrator release send exited 0 immediately after
cleanup/readbacks and before this append: **DESKTOP/OPERATOR RUNTIME RELEASED**.
No Cargo slot was acquired or released; the independent CI lane is unaffected.
The source/provider/helper remain unchanged and this phase grants no rerun.
The remaining bounded application journey has no browser outcome yet. This
failure does not replace the earlier dated D01 password-browser/R05 evidence,
recover the first lost version values, erase the reporting correction or
change root-owned D01/D05 classification. O06/I10/R05 remain DONE.

Only this report is appended. No source/helper/product/guide/D05/test/config/
state edit, Cargo/build/test, policy reload, main/push/status/merge/reset, new
worker/task/worktree/shell or other-worker contact occurred. All accepted
receipt/header/PAM/removal/audit and held Group/nonrenewed 60-second/paused-I/O
contracts remain unchanged.

Evidence-append checks passed: all 58457 bytes of preceding `fb9f474` report
remain an exact prefix, reviewed helper bytes/hash are unchanged, and both new
private metadata files retain their actual bytes/hash and mode 0600. Only this
report differs, Markdown fences/final newline pass, and `git diff --check`
passed. These checks invoked no fixture or provider again.

## Reserved request-rejection diagnostic, source only — 2026-10-02

Root fully read the preceding actual fixture evidence/private JSON and request
guards, found no identifiable failed predicate or sender, and reserved
`wave30_D01_request_rejection_diagnostic` for source only. This phase adds one
private evidence field, `request_invalid_reason`, initially null. It records
only the first fixed whitelisted request-rejection label. No helper, provider,
operator command, listener, Driver, browser or runtime was executed here. The
failed fixture remains failed and its historical reason remains unidentified.

Source commit **`7461ab50f5a5fdf0aa5d7bd6c4b309575e7a4238`**, parent
`3b85ee2d000fb691c81218b9ae2cd7179b5e7102`, changes only
`scripts/d01-confidential-browser-demo.py` (36 insertions / 16 deletions).
Result: mode `100644`, 671 lines / 31162 bytes, blob
`62460c2f4515a325bb65fc430649b904e8bb2c10`, SHA-256
`ac3c350d0ea5f1c7448d533524e5db2f99ee37ad61be313c6f36e0c4025a2fcd`.
The original source for equivalence is reviewed `16395f1`, blob
`37c9136850c2522da1eaa5f31f8b94e70acc0886`, SHA-256 `f94af72a...` above.

### Exact finite mapping and first-reason preservation

| Fixed label | Original rejected predicate / unchanged control-flow position |
| --- | --- |
| `http_parse` | Parser `send_error`; original code/status/failure-tag assignment and reply follow unchanged |
| `host` | `headers.get_all("Host") == [AUTHORITY]` |
| `authorization` | `headers.get_all("Authorization") is None` |
| `transfer_encoding` | `headers.get_all("Transfer-Encoding") is None` |
| `expect` | `headers.get_all("Expect") is None` |
| `content_length` | `headers.get_all("Content-Length") in (None, ["0"])` |
| `target_scheme` | `not target.scheme` |
| `target_netloc` | `not target.netloc` |
| `target_fragment` | `not target.fragment` |
| `method` | Same existing `GET` / `POST` branches, otherwise the original `Failure("request_invalid")` |
| `post_target` | `target.path == "/login" and not target.query`, preserved as one compound condition |
| `origin` | `headers.get_all("Origin") == [ORIGIN]` |
| `content_type` | `headers.get_all("Content-Type") == ["application/x-www-form-urlencoded"]` |
| `cookie_header_count` | `len(headers) <= 1`, before existing cookie-size/piece limits and own-cookie loop |
| `own_cookie_shape` | Same compound separator / absence of duplicate own name / exact 43-character cookie regex, in the existing own-cookie loop |

The header predicates still run in the five-row order above, followed by the
three target predicates. POST retains target, Origin, type order. The own-cookie
compound retains all three operands and their short-circuit order. Each failed
new `require_request` call records its fixed label then immediately raises the
same `Failure("request_invalid")`, so later predicates are not evaluated after
an earlier rejection. Parser `send_error` records only `http_parse`; method
rejection records only `method` before its unchanged raise. Existing request
limits, parser behavior, exceptions, timeouts and accepted/refused sets remain.

The recorder assigns only when the in-memory reason is null **and** the supplied
label belongs to the 15-name whitelist. Every call site supplies a fixed literal
from that set, with only the internal recorder forwarding its `reason` argument.
There are exactly two writes to the in-memory property: initialization to null
and this guarded first assignment. Main initializes the private JSON field to
null and copies that property once in finally before clearing private references.
No response/cleanup branch resets or replaces it. Unknown labels cannot be
stored. No request line, header, Host, path, query, method, cookie, Origin, error
text or sender information is added to output. Existing stdout summaries are
unchanged; only the owner-private JSON gains the one field.

### Actual static proofs, no execution

The source-only verifier parsed both full modules, then normalized the new AST
by removing exactly the whitelist, two diagnostic methods, null initialization,
one private JSON key/copy and two fixed parser/method marks. It regrouped only
the five approved predicate runs into their prior `require(..., "request_invalid")`
conjunctions, flattening the POST-target compound into the original four-operand
conjunction. The resulting **entire module AST exactly equals the original AST**
without source-position attributes. This covers original predicates, their
ordering/branch nesting, public responses, other outputs and all other logic.
The normalization collector visits nested bodies first; its internal run list
is a traversal detail, not a claim that own-cookie shape executes before the
header-count guard. Original branch nesting/order is established by the full
restored-AST equality.

The new rejection helper, with its diagnostic call removed, also has exactly
the old `require` function body after substituting the fixed original tag.
Whitelist membership, all fixed-label call sites and null-first assignment
shape were checked directly in the AST. The private key has exactly one added
initialization and one final copy; no additional output-field change survives
the normalization proof.

Byte comparisons independently verified **28 existing functions** unchanged,
including credential/verifier loading, every Budget method, Demo provider setup,
authorization begin/callback/clear, HeaderReader, DemoServer, protected access
`Handler.get`, replies/CSP/cookies in `Handler.reply`, quiet logging and
`QuietParser.error`. Only Demo initialization, three instrumented Handler methods and main's
new private field/copy differ, plus the two new diagnostic methods. The full
pinned version/hash, flow/native crypto, five-second operations, real deadlines,
cleanup, protected 403/200 predicates and client-secret contracts remain exact.

In-memory `compile(source, filename, "exec", dont_inherit=True)` passed. The
code object was not executed; the helper was not imported and no bytecode was
written. Scope and `git diff --check` passed before source commit. All prior
68899 report bytes were unchanged at that point. Retained failed-fixture JSON
`04074d4a...`, separate diagnostic `8f47d8fb...` and fixture provider metadata
`6fad035f...` matched their complete SHA-256 pins and mode 0600 on readback.
No metadata was augmented retrospectively with a new reason.

### Remaining input and held runtime

This source provides a bounded predicate category for a future separately
released rejection; it cannot identify sender or reconstruct the prior failure.
No concrete controller defect or product cause is established by these static
proofs, and no controller behavior/guard relaxation is changed or proposed here.
Root must immutably review/publish this source and separately release any exact
fixture before execution. This source reservation acquires/releases no runtime
or Cargo slot. RiWork Cua.ai Driver MCP remains the sole future desktop provider.

Only the reserved helper source and this append-only report change. No other
helper/product/guide/D05/config/state/test edit, provider/CLI/server/listener/
HTTP/Driver/browser/session/Cargo/build/test run, main/push/status/merge/reset,
new worker/task/worktree/shell or other-worker contact occurred. First-refusal
lost values/reporting correction, actual failed fixture and dated D01/R05
observations remain preserved. Root owns D01/D05 interpretation; O06/I10/R05
stay DONE. Receipt/header/PAM/removal/audit and held Group/nonrenewed
60-second/paused-I/O protections remain unchanged.

Evidence-append checks passed: all 68899 bytes of `3b85ee2` remain an exact
prefix; committed diagnostic source and all three private historical metadata
files retain their bytes/hash/mode. Only this report differs, Markdown fences
and final newline pass, and `git diff --check` passed. No source import,
provider call or runtime execution was used for these document checks.

## Separately released diagnostic fixture: Authorization rejection — 2026-10-02

**The one new fixture exited 1 before browser launch.** Root fully reviewed and
published diagnostic source `7461ab5` and evidence `6e4739d` at main
`d5b60bc12aaafa258daddd64b40879afacb87bc1`, accepted review `cebdcd1`, then
released one fresh fixture. It used the exact diagnostic helper without edits.
The helper retained first reason **`authorization`**, original failure tag
**`request_invalid`**, stage **`request`**. The outer guard stopped with
`rp_nonzero`, cleaned its resources and exited 1. No browser authentication,
consent or application checkpoint is credited. No retry or correction ran.

### Source/artifact inputs and unchanged controller behavior

Own entry HEAD was `6e4739d473569d95ef3468012b0a020b3e05d9e7`; no main merge
occurred. Fresh helper hash matched
`ac3c350d0ea5f1c7448d533524e5db2f99ee37ad61be313c6f36e0c4025a2fcd`,
blob `62460c2f4515a325bb65fc430649b904e8bb2c10`. All three matching c01 binary
hashes were recomputed before use and equal the complete pins recorded above.
The fixed verifier was read from its `9cefe7a` Git object, matched SHA-256
`f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d`,
and was materialized only inside the fresh private lab. Published guide and
OIDC-profile blob/hash identities remain `e48371d...` / `9df3c286...` and
`d9f222a...` / `ed1ae74c...` respectively. These identity readbacks do not
claim a new whole-body review or historical observation refresh.

The transient controller retained the prior setup/commands/readiness behavior:
fresh `0700` own lab/XDG/RP directories, hidden synthetic password prompts,
same normal confidential public-CLI registration, exact issuer 9000 and callback
3000, five-second readiness request and `lsof` listener ownership reads.
Changes were only the required diagnostic source pin, fresh exclusive metadata/
lab names, finite reason retention and numeric owned-child exit recording.
No new HTTP probe/request, Host tolerance, guard relaxation, authorization
substitution or source correction was introduced to avoid the prior refusal.
No private password, client secret or CLI-session value was printed or passed
through argv/environment; captured CLI output was discarded.

### Native observation and actual operator setup

The resolved native provider again matched
`67a83dd6d6d747d50c5d296dffb23e32bae9a2c588c93ae2d77e4c607b455c72`.
Its preflight `version` invocation used only inherited `HOME`, `LANG`, `LC_ALL`,
`PATH`, `TMPDIR`, with a five-second timeout. It returned **0**, no timeout,
**63 ASCII stdout bytes** containing
`OpenSSL 3.6.4 25 Aug 2026 (Library: OpenSSL 3.6.4 25 Aug 2026)` plus newline,
and **0 stderr bytes**. That complete exit/text/length/hash observation was
flushed/fsynced into a new exclusive `0600` file before exact equality assertion:
`deployment-private/d01-confidential-browser-diagnostic-provider-20261002.json`,
359 bytes, SHA-256
`6fad035f1897b5287b2ebd812405fcabf6a33b72d2d4f498f3bca7081eb719ab`.
Its bytes happen to equal the earlier provider metadata, but it is a distinct
new file; the old file was never overwritten. The helper's own unchanged native
version check also passed. No RS256 signature-verification result is claimed.

| Owned command/process | PID | Actual numeric exit / observation |
| --- | --- | --- |
| Maintenance init | 56111 | 0; two hidden password prompts |
| Essentials server | 56161 | 0 after owned cleanup; only owner of listener 9000 while running |
| Operator CLI login | 56167 | 0; one hidden password prompt |
| Confidential Local demo creation | 56168 | 0; exact callback/scopes; new secret-file mode 0600 |
| CLI discovery | 56169 | 0 |
| CLI whoami | 56170 | 0 |
| Diagnostic helper, one invocation | 56171 | 1; only owner of listener 3000 while running |
| Outer guard | 55984 | 1 |

The existing `GET http://127.0.0.1:9000/readyz` returned **200**. CLI login and
registration were operator setup only, never passed to the RP as a user session.
No service-session/API sign-in/approval stood in for browser behavior.

### Observed reason, checks and limits of attribution

The helper failed after **3.013 seconds**. Both its private result and the outer
result retain `request_invalid_reason: "authorization"`. Under the reviewed
first-only recorder and guard order, this identifies only failure of:

```python
self.headers.get_all("Authorization") is None
```

Thus the parsed request's Authorization header list was non-null; no header
value or sender information was retained. Parsing and the preceding Host guard
completed before this first rejection. Later request guards were not reached
and are not credited. No raw request line, Host, method, path, query, cookie,
Origin, error contents or header value is disclosed. The reason does not name
an actor, prove product fault or retroactively assign a reason to the prior
unclassified fixture. That prior failure remains independently unknown.

Actual helper checks: credential/private-file, provider identity and discovery
are true. All seven subsequent protected/authorization/state/exchange/native
RS256/JWKS/issuer/audience/nonce/time/access-hash/userinfo/fresh-cookie checks are
false. All six helper HTTP status fields remain null. The original failure tag,
stage, booleans and statuses are retained without converting the result to pass.
No protected-before 403, password/empty-factor sign-in, Local demo consent,
callback, userinfo or protected-after 200 was performed.

A bounded read-only inspection of this transient controller's source found
listener readbacks through `lsof`, its single explicit readiness URL on 9000,
and the unchanged CLI issuer/registration schedule. It establishes no concrete
controller defect or request origin. No new controller request, source edit,
diagnostic rerun or sender inference followed the refusal. Root owns the next
cause adjudication/reservation; this phase grants no further runtime.

### Resource budget, cleanup and release

First clock read: `2026-10-02 15:32:15 UTC`, with the inclusive timer started
immediately before it. Final cleanup completed at
`2026-10-02T15:34:48.232499+00:00`, **153.252 seconds** inclusive, under the
900-second cap. Active/cleanup were capped at 840/60 seconds, helper 600,
pending 180, native/HTTP five and CLI 60. No conservative historical allowance
was added to this new timer.

Fresh capacity passed the 8.5 GiB entry gate. Outer one-second monitoring began
at guard preflight: **5 samples**, minimum **14558846976 bytes / 13.558983 GiB**.
The helper's scheduled and phase-entry checks recorded **22 samples**, minimum
**14558842880 bytes / 13.558979 GiB**. Both minima exceed the 8.5 stop margin and
8 GiB floor. No continuous minimum is claimed before guard preflight, and no
disk deletion was performed.

Owned children were stopped/joined and their numeric exits retained as above.
The helper's listener/connection closure, private-reference clearing and
verifier-temporary cleanup booleans are all true. Fresh readbacks found all
eight listed PIDs absent, both 9000/3000 `lsof` checks exit 1/no listeners, and
the entire fresh lab/XDG/store/password/credential/session/verifier input absent.
No unrelated process was killed, and no Python memory erasure is claimed.

The helper refused before the browser branch. This phase made **zero Driver
calls** and no session/profile/binding/navigation/input/screenshot/recording/
logout actions. No Driver-owned browser/session existed to kill or end, and no
cleanup of a personal/shared Driver resource is claimed. RiWork Cua.ai Driver
MCP remains the sole allowed desktop provider for any separately released run.

Actual new retained result:
`deployment-private/d01-confidential-browser-diagnostic.redacted.json`, exclusive
`0600`, **4046 bytes**, SHA-256
`aefdc34827486859e08a1e28de0a09505af65b4afdf9f1ba5f952fa0208595fa`.
Both new metadata files are private/ignored. All old metadata/lost-value and
reporting-error evidence remain intact.

The explicit-project orchestrator release send exited 0 immediately after
cleanup/readbacks and before this append: **DESKTOP/OPERATOR RUNTIME RELEASED**.
No Cargo slot was acquired/released; I04 preparation's independent lane is
unaffected. Source and original guards remain unchanged, and no retry/fallback
ran. Only this report is appended; no product/guide/D05/source/test/config/state
edit, Cargo/build/test, main/push/status/merge/reset, new worker/task/worktree/
shell or other-worker contact occurred. Root owns D01/D05 gate adjudication;
O06/I10/R05 remain DONE and every prior accepted protection remains unchanged.

Evidence-append checks passed: all 76829 preceding `6e4739d` report bytes remain
an exact prefix; helper bytes/hash and every old/new private metadata hash,
length and mode 0600 match their pins. The actual reason is the finite literal
`authorization`, and all recorded owned-child exits are numeric. Only this
report differs, Markdown fences/final newline pass, and `git diff --check`
passed. These checks repeated no provider or fixture execution.

## Controller Authorization source audit, read only — 2026-10-02

Reservation: `wave30_D01_controller_authorization_source_audit`. Only this
existing report is appended. **No configured own source call site was found
that constructs Authorization to port 3000. The actual sender remains unknown.**
The observed `authorization` refusal is preserved; this is no product pass,
sender attribution or retry. Fixture/runtime remains held.

### Exact retained controller and match to the actual attempt

The just-executed controller is available as the exact retained
`d01_diag_controller_text` session-store value. The preceding tool call stored
that value, passed the same value as `exec_command.cmd`, and retained returned
exec session **34263**. The command payload is **13380 UTF-8 bytes**, SHA-256
`ab0a9c5d8f8fb1c4f5fb5db8b3c396e4dfbdf56bb5a7f0f7e7178b485d77040e`.
Its Python body is **13357 bytes / 177 lines**, SHA-256
`a3748aaa8aee61fa302050daa78867b1a251fca6cd1294047c43808db71486da`.
The full public source payload is archived below so this source input is not
lost at a later session compaction. It contains code/constant inputs, no actual
password/session/header/token values.

Static AST/readback matched `START = 1790955134.981` to the actual redacted
metadata, the exact `ac3c350d...` helper pin, new exclusive evidence/provider
paths and the observed command schedule. Retained stdout/metadata identify
guard 55984, server 56161 and helper 56171. These are the same attempt recorded
above. Payload hashes were computed **in this audit**, not captured before
execution or attested from process memory. The actual private JSON does not
contain a controller-source hash; metadata alone would not independently prove
the entire payload. The source match uses the retained tool-call input/value
chain plus these concrete readbacks.

No controller payload is missing. Source for the implementation behind
`tools.exec_command` (including any implicit process/listener integration, if
one exists), and Python interpreter/startup customizations, is not a supplied
or pinned source input here. No implicit hook is asserted to exist. This audit
did not inspect broader process arguments, environment values, private files
or unrelated processes to fill that gap, and cannot identify network origin
through those unreviewed implementations.

### Every configured initial request/readiness construction

| Source construction | Destination / Authorization behavior |
| --- | --- |
| Controller `listeners()` / `wait_listener()` | `lsof` readbacks; no HTTP request builder in that source |
| Controller's sole explicit `urlopen()` | Fixed `http://127.0.0.1:9000/readyz`, no explicit headers; synchronous block completes before CLI setup and RP startup |
| Operator CLI login | Selected issuer 9000; `/api/login` has no bearer argument |
| Operator client creation / revision / whoami | Authenticated API requests bound to the selected issuer 9000; creation callback 3000 is registration data, not a contacted URL |
| Operator CLI discovery | Selected issuer 9000, no bearer |
| Adapter setup / delegated discovery | Fixed issuer 9000; no bearer argument and no RP self-request |
| Adapter sign-in action | Constructs the future browser authorization redirect at validated issuer 9000; no HTTP client call or Authorization header construction |
| Adapter confidential token exchange | Validated 9000 token endpoint; client secret in form, no bearer argument |
| Delegated JWKS verification request | Validated 9000 JWKS URI; no bearer argument |
| Adapter userinfo | Validated 9000 userinfo endpoint, bearer argument only after callback/exchange/verification; never reached in this failed fixture |
| Parent recovery `__init__` / `login` RP requests | Not invoked by this adapter; source possibility in the unused library, not an executed initial RP builder |

The retained controller has one AST `urlopen` call and no Authorization literal.
It uses the same readyz/default-opener construction as the failed attempt; no
proxy/redirect/opener correction was made. The default opener's transitive or
ambient behavior is not attested by the absence of an explicit header in this
source. That synchronous request and every CLI child nevertheless complete in
the controller's source order before the helper/RP is launched.

Fixed c01 CLI source reads were bounded to the selected login/discovery/whoami/
client-create dispatch and transport/base/header construction. Whole-file hashes
below identify the sources, not whole-body reviews or a new binary/source-build
attestation. The previously accepted c01 binary pins remain the runtime inputs.

| c01 source at `c01c39ab4e092423d5522bedc50fff87656d8c0a` | Blob / SHA-256 |
| --- | --- |
| `crates/riauthctl/src/transport.rs` | `dea115cb8e23bd54911cdb66d50b6efa7e9c940b` / `d0f64e747b667c9f3a7eea18ce84e0dd16b7c0b6d695f2fbc8ed70c553fe7240` |
| `crates/riauthctl/src/main.rs` | `d91107cf667e9a2ff60b261e7edc41270367eb52` / `e75f26eedb1b740d6cbeacf32663f01fb5b802330be6a612eecfca35f29a7da6` |
| `crates/riauthctl/src/admin.rs` | `58b316890bb6bd04d331088164a94ae39d2d6a9c` / `d67ceb258bd3e2b66b8aebce4cc8af65630bb4b7c22ca2379e4b2b4122811cfd` |

`Remote::new` disables redirects/proxies. Discovery carries no bearer,
`verify_issuer` refuses a management-base mismatch and retains the selected
issuer, and `request_at` builds base plus a validated API path, adding bearer
only when supplied. There is no followed redirect forwarding that bearer to
the registered callback. The actual CLI children all exited before RP startup.

Pinned verifier source remains blob `3be747d03146f1bcaa3ec012ee8d173b61fa737d`,
SHA-256 `f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d`.
Its top level contains definitions/imports/constants, no HTTP invocation.
Adapter AST references to `LocalRelyingParty` are only `discovery`,
`callback_fields` and `verify_id_token`; its constructor/login are not called.
The adapter is a separate Demo object and binds its own fixed 3000 server.

The verifier's generic `request(url, bearer=...)` **can** construct Authorization
for any caller-supplied URL if given a bearer. That is a source capability, not
an observed call to 3000. Its explicit header starts with Accept JSON, optionally
adds form Content-Type, and only adds Authorization after a string-length bound
of 1..32768 and the existing token-character regex. Default request opener has
proxies disabled and redirects refused. Every configured adapter use omits
`agent`; the sole bearer use is the validated 9000 userinfo request. Discovery
requires issuer equality and each authorization/token/userinfo/JWKS endpoint to
equal fixed issuer plus the exact suffix. The unused recovery `login` code also
creates a new opener/request without the IdP bearer for its RP callback and
protected reads; its existence does not mean that branch executed here.

### First-request timing: unavailable in actual evidence

The current JSON has `request_invalid_reason` but no initial request timestamp,
connection/request count or request index. The rejection happened after the
RP listener existed and before any worker Driver/browser action, within the
helper's **3.013-second total lifetime**. That lifetime includes private setup,
provider/discovery work and cleanup; it is **not** the first request time.
Source allows empty EOF and root/404 requests without recording a first-request
counter, so null protocol statuses do not prove the rejection was the first
connection or request. Those are source possibilities, not observed earlier
traffic. No first-request timing, header value or sender can be recovered from
the retained metadata, and none was fetched here.

### ONE proposed bounded observation, not an implementation

Propose only one new private field,
`request_first_line_ms_since_listen`: null or an integer bounded to 0..600000.
Capture it once after the existing first **nonempty request-line read** returns,
using a monotonic origin immediately after listener construction. This defines
line-read completion, not TCP acceptance time, request contents or sender.
The observation would add no request/probe, header inspection, tolerant Host/
Authorization/cookie behavior or new caller attribution. It closes only the
missing initial-line timing observation; it does not solve sender identity or
make the application journey pass. Root decides whether that observation is
useful, and must separately reserve source/runtime before any implementation.

Exact prospective helper-only insertions (current source still unchanged):

```diff
@@ Demo.__init__ (after existing reason initialization)
         self.request_invalid_reason = None
+        self.request_first_line_ms_since_listen = None
+        self.listener_started_at = None
@@ DemoServer.__init__ (after existing bind/listen construction)
         super().__init__(("127.0.0.1", 3000), Handler)
+        demo.listener_started_at = time.monotonic()
@@ Handler.handle_one_request (after existing empty-line return)
                 if not self.raw_requestline:
                     return
+                if demo.request_first_line_ms_since_listen is None:
+                    demo.request_first_line_ms_since_listen = min(600000, max(0, int(
+                        (time.monotonic() - demo.listener_started_at) * 1000)))
                 require(len(self.raw_requestline) <= MAX_REQUEST_LINE, "request_limit")
@@ main private record
               "request_invalid_reason": None,
+              "request_first_line_ms_since_listen": None,
@@ main finally (before existing reference clearing)
             record["request_invalid_reason"] = demo.request_invalid_reason
+            record["request_first_line_ms_since_listen"] = demo.request_first_line_ms_since_listen
```

For review only, those five insertions were constructed **in memory**, parsed
and compiled to an unexecuted code object. Candidate SHA-256:
`66254e52e2d7f0421586348d03d42b08eefbacd97758a873b9d04e81aecb6b68`.
Removing exactly the three new clock/state assignments, one first-only observer
block, one private JSON key and one final copy restores the **entire original
`ac3c350d...` AST exactly** without source-position attributes. Existing guard
predicates/order, first reason, failure tag, deadline/flow/native crypto,
public responses/protected behavior and cleanup logic all remain in that
restored AST. No proposed source was written, imported or executed. This is the
preservation proof before any possible change, not an approved source commit.

### Scope and held outcome

This audit used retained source reads, bounded fixed Git-object body reads,
static AST/payload matching and the unexecuted in-memory candidate. No raw
credential/token/header value, broader process argument or environment value
was inspected. No additional HTTP/native/provider/helper/CLI/server/listener/
Driver/browser/Cargo/version invocation, rerun, deletion, helper/controller/
product/guide/D05/test edit, main/push/status/merge/reset, new worker/task/WT/
shell or other-worker contact occurred. All old failures, lost-value correction,
metadata and accepted gate limits remain preserved. RiWork Cua.ai Driver MCP
remains the sole desktop provider. Runtime remains held; root owns D01/D05
interpretation and any next reservation; O06/I10/R05 stay DONE.


### Retained executed command payload — source archive

The fenced block preserves the historical command input, not execution
permission. Its final block newline is Markdown formatting: remove that one
newline to reconstruct the 13380-byte command ending in `PY`. Its Python body
is the 177-line body between the shell wrapper and final `PY`; no actual
private credential/header/environment value is included.

```bash
python3 -u - <<'PY'
import datetime,fcntl,hashlib,json,os,pathlib,pty,secrets,select,shutil,signal,stat,subprocess,sys,tempfile,termios,time,urllib.request
ROOT=pathlib.Path.cwd()
START=1790955134.981
ACTIVE=START+840
BIN=pathlib.Path('/Users/dominik/orca/projects/riAuth-public-preview-local-module-boundaries-wave27/target/d01-essentials-c01c39a/aarch64-apple-darwin/debug')
PRIVATE=ROOT/'deployment-private'
OUT=PRIVATE/'d01-confidential-browser-diagnostic.redacted.json'
PROVIDER_META=PRIVATE/'d01-confidential-browser-diagnostic-provider-20261002.json'
record={'schema':'riauth.d01-confidential-browser-outer/v1','result':'failed','failure_stage':None,'failure_tag':None,'request_invalid_reason':None,'commands':[],'artifacts':{},'cleanup':{},'owned_child_exits':[],'started_epoch':START,'active_seconds':840,'cleanup_seconds':60,'minimum_free_bytes':None,'disk_samples':0}
lab=None;children=[];names={};server=None;helper=None;password=None;stage='preflight';resultfd=None;lastsample=0
def require(ok,tag):
    if not ok:raise RuntimeError(tag)
def tick():
    global lastsample
    now=time.monotonic()
    if now-lastsample>=1:
        free=shutil.disk_usage(ROOT).free
        record['disk_samples']+=1
        record['minimum_free_bytes']=free if record['minimum_free_bytes'] is None else min(record['minimum_free_bytes'],free)
        lastsample=now
        require(free>=8.5*1024**3,'disk_margin')
    require(time.time()<ACTIVE,'active_deadline')
def write_exclusive(path,value):
    fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    raw=(json.dumps(value,sort_keys=True,indent=2)+'\n').encode('ascii')
    with os.fdopen(fd,'wb') as f:
        f.write(raw);f.flush();os.fsync(f.fileno())
    return hashlib.sha256(raw).hexdigest()
def listeners(port):
    p=subprocess.run(['/usr/sbin/lsof','-nP','-t','-iTCP:'+str(port),'-sTCP:LISTEN'],capture_output=True,timeout=3)
    require(p.returncode in (0,1),'socket_observation_failed')
    return set(int(v) for v in p.stdout.split())
def own(p,name):
    children.append(p);names[p.pid]=name;return p
def stop_child(p):
    if p.poll() is None:
        for sig,wait in [(signal.SIGINT,8),(signal.SIGTERM,5),(signal.SIGKILL,2)]:
            if p.poll() is not None:break
            p.send_signal(sig)
            try:p.wait(timeout=wait)
            except subprocess.TimeoutExpired:pass
    require(p.poll() is not None,'owned_child_reap_failed')
def controlling_tty():
    os.setsid();fcntl.ioctl(0,termios.TIOCSCTTY,0)
def cli(name,args,prompts=0):
    tick()
    master,slave=pty.openpty()
    p=own(subprocess.Popen(args,cwd=lab,env=environment,stdin=slave,stdout=slave,stderr=slave,preexec_fn=controlling_tty),name)
    os.close(slave);seen=0;raw=b'';started=time.monotonic()
    try:
        while p.poll() is None:
            tick();require(time.monotonic()-started<60,'cli_deadline')
            if select.select([master],[],[],0.2)[0]:
                try:chunk=os.read(master,4096)
                except OSError:chunk=b''
                raw+=chunk;require(len(raw)<=131072,'cli_output_limit')
                for prompt in ([b'Password: ',b'Confirm password: '] if prompts==2 else [b'Password: ']):
                    if seen<prompts and prompt in raw:
                        require(prompt==([b'Password: ',b'Confirm password: '][seen] if prompts==2 else b'Password: '),'cli_prompt_order')
                        os.write(master,password.encode()+b'\n');seen+=1;raw=b''
        while select.select([master],[],[],0)[0]:
            try:
                chunk=os.read(master,4096)
                if not chunk:break
                raw+=chunk
            except OSError:break
        code=p.wait()
        record['commands'].append({'name':name,'exit':code,'password_prompts':seen})
        require(code==0,'cli_nonzero');require(seen==prompts,'cli_prompt_missing')
    finally:
        raw=b'';os.close(master)
        if p.poll() is None:stop_child(p)
def wait_listener(p,port):
    deadline=time.monotonic()+30
    while time.monotonic()<deadline:
        tick();require(p.poll() is None,'owned_service_early_exit')
        owners=listeners(port)
        if owners:
            require(owners=={p.pid},'listener_owner_mismatch');return
        time.sleep(0.2)
    raise RuntimeError('listener_deadline')
try:
    tick()
    require(PRIVATE.is_dir() and not PRIVATE.is_symlink() and stat.S_IMODE(PRIVATE.stat().st_mode)==0o700,'private_directory_invalid')
    require(not OUT.exists() and not PROVIDER_META.exists(),'evidence_already_exists')
    require(not listeners(9000) and not listeners(3000),'port_occupied');record['ports_preflight_empty']=True
    pins={'riauth':'7abf745c10691a012a1918c83089168d0e0d88b42764dbf8ed538b90c828b606','riauth-maintenance':'86490c7f71de6b7ae9d4dabdf9060a8757100f3aedbdaa670284d9e1206a2a95','riauthctl':'bfbbb322f1a66d0ac9998beb9fb5838097cea0442cea3fd3a1d057fcb3f600cf'}
    for name,pin in pins.items():
        tick()
        with (BIN/name).open('rb') as f:digest=hashlib.file_digest(f,'sha256').hexdigest()
        record['artifacts'][name]=digest;require(digest==pin,'artifact_hash_mismatch')
    helper_path=ROOT/'scripts/d01-confidential-browser-demo.py'
    require(hashlib.sha256(helper_path.read_bytes()).hexdigest()=='ac3c350d0ea5f1c7448d533524e5db2f99ee37ad61be313c6f36e0c4025a2fcd','helper_hash_mismatch')
    verifier=subprocess.check_output(['git','show','9cefe7a56425bb73c17753e8766d92320b77da3b:scripts/recovery-drill-oidc.py'],timeout=5)
    require(hashlib.sha256(verifier).hexdigest()=='f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d','verifier_hash_mismatch')
    provider=pathlib.Path('/opt/homebrew/bin/openssl').resolve(strict=True)
    with provider.open('rb') as f:provider_hash=hashlib.file_digest(f,'sha256').hexdigest()
    require(provider_hash=='67a83dd6d6d747d50c5d296dffb23e32bae9a2c588c93ae2d77e4c607b455c72','provider_hash_mismatch')
    provider_env={k:v for k,v in os.environ.items() if k in {'PATH','HOME','TMPDIR','LANG','LC_ALL'}}
    p=subprocess.Popen([str(provider),'version'],stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=provider_env)
    try:stdout,stderr=p.communicate(timeout=5);timeout=False
    except subprocess.TimeoutExpired:
        p.kill();stdout,stderr=p.communicate(timeout=2);timeout=True
    provider_record={'sha256':provider_hash,'exit':p.returncode,'timeout':timeout,'stdout_ascii':stdout[:4096].decode('ascii',errors='backslashreplace'),'stderr_ascii':stderr[:4096].decode('ascii',errors='backslashreplace'),'stdout_bytes':len(stdout),'stderr_bytes':len(stderr),'environment_keys':sorted(provider_env)}
    record['provider_metadata_sha256']=write_exclusive(PROVIDER_META,provider_record)
    require(not timeout and p.returncode==0 and len(stdout)<=256 and len(stderr)<=4096 and stdout.decode('ascii').strip()=='OpenSSL 3.6.4 25 Aug 2026 (Library: OpenSSL 3.6.4 25 Aug 2026)','provider_version_failed')
    tick()
    resultfd=os.open(OUT,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    lab=pathlib.Path(tempfile.mkdtemp(prefix='d01-confidential-browser-diagnostic.',dir=PRIVATE));os.chmod(lab,0o700)
    for name in ('xdg','deployment-private','rp'):(lab/name).mkdir(mode=0o700)
    (lab/'recovery-drill-oidc.py').write_bytes(verifier);os.chmod(lab/'recovery-drill-oidc.py',0o600);verifier=None
    password=secrets.token_urlsafe(30)
    fd=os.open(lab/'browser-password',os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
    with os.fdopen(fd,'w') as f:f.write(password)
    environment=dict(provider_env);environment['XDG_CONFIG_HOME']=str(lab/'xdg')
    stage='init';cli('maintenance_init',[str(BIN/'riauth-maintenance'),'--config',str(lab/'riauth.toml'),'init','--issuer','http://localhost:9000','--listen','127.0.0.1:9000','--data-dir','data','--admin','admin'],2)
    stage='serve'
    server=own(subprocess.Popen([str(BIN/'riauth'),'--config',str(lab/'riauth.toml'),'serve'],cwd=lab,env=environment,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL),'server')
    record['server_pid']=server.pid;wait_listener(server,9000);record['server_listener_owned']=True
    stage='readyz'
    with urllib.request.urlopen('http://127.0.0.1:9000/readyz',timeout=5) as response:
        record['readyz_status']=response.status;response.read(4096)
    require(record['readyz_status']==200,'readyz_failed')
    stage='cli_login'
    base=[str(BIN/'riauthctl'),'--server','http://localhost:9000']
    cli('operator_login',base+['login','admin'],1)
    stage='client_create';secretpath=lab/'deployment-private/local-demo-secret.json'
    cli('confidential_client_create',base+['client','create','local-demo','--name','Local demo','--confidential','--redirect-uri','http://localhost:3000/callback','--scope','openid,profile','--secret-file',str(secretpath)])
    require(stat.S_IMODE(secretpath.stat().st_mode)==0o600,'cli_secret_mode_invalid')
    stage='discovery';cli('discovery',base+['discovery'])
    stage='whoami';cli('whoami',base+['whoami'])
    stage='rp_start'
    helper=own(subprocess.Popen([sys.executable,'-B',str(helper_path),'--workspace',str(lab/'rp'),'--secret-file',str(secretpath),'--verifier-helper',str(lab/'recovery-drill-oidc.py'),'--openssl','/opt/homebrew/bin/openssl','--deadline-seconds','600','--evidence',str(lab/'rp-result.json')],cwd=ROOT,env=environment,stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,text=True),'helper')
    record['helper_pid']=helper.pid;record['helper_invocations']=1
    wait_listener(helper,3000);record['helper_listener_owned']=True
    print(json.dumps({'fixture_ready':True,'guard_pid':os.getpid(),'server_pid':server.pid,'helper_pid':helper.pid,'lab':str(lab),'commands':record['commands'],'provider_metadata_sha256':record['provider_metadata_sha256'],'minimum_free_bytes':record['minimum_free_bytes']}),flush=True)
    stage='browser_checkpoint';announced=False
    while not (lab/'stop').exists():
        tick();require(server.poll() is None,'idp_early_exit')
        if helper.poll() is not None:
            code=helper.wait()
            if not announced:
                record['helper_exit']=code;record['helper_result']=json.loads((lab/'rp-result.json').read_bytes())
                record['request_invalid_reason']=record['helper_result']['request_invalid_reason']
                print(json.dumps({'helper_completed':True,'exit':code,'result':record['helper_result']['result'],'failure_tag':record['helper_result']['failure_tag'],'request_invalid_reason':record['request_invalid_reason']}),flush=True);announced=True
            require(code==0,'rp_nonzero')
        time.sleep(0.2)
    if (lab/'ui-failure').exists():raise RuntimeError('browser_checkpoint_failed')
    require(helper.poll()==0,'rp_checkpoint_incomplete');record['result']='passed'
except Exception as error:
    record['failure_stage']=stage
    tags={'disk_margin','active_deadline','private_directory_invalid','evidence_already_exists','port_occupied','socket_observation_failed','artifact_hash_mismatch','helper_hash_mismatch','verifier_hash_mismatch','provider_hash_mismatch','provider_version_failed','cli_deadline','cli_output_limit','cli_prompt_order','cli_prompt_missing','cli_nonzero','owned_service_early_exit','listener_owner_mismatch','listener_deadline','readyz_failed','cli_secret_mode_invalid','idp_early_exit','rp_nonzero','browser_checkpoint_failed','rp_checkpoint_incomplete'}
    record['failure_tag']=str(error) if str(error) in tags else 'outer_unexpected_failure'
finally:
    for p in reversed(children):
        try:stop_child(p)
        except Exception:record['cleanup']['child_reap_failure']=True
        record['owned_child_exits'].append({'name':names[p.pid],'pid':p.pid,'exit':p.poll()})
    record['cleanup']['owned_children_reaped']=all(p.poll() is not None for p in children)
    if lab is not None:
        if helper is not None:
            record['helper_exit']=helper.poll()
            if (lab/'rp-result.json').exists():
                record['helper_result']=json.loads((lab/'rp-result.json').read_bytes())
                record['request_invalid_reason']=record['helper_result']['request_invalid_reason']
        password=None;shutil.rmtree(lab);record['cleanup']['lab_removed']=not lab.exists()
    else:record['cleanup']['lab_removed']=True
    try:
        record['cleanup']['port9000_absent']=not listeners(9000);record['cleanup']['port3000_absent']=not listeners(3000)
    except Exception:record['cleanup']['socket_observation_failure']=True
    record['completed_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();record['elapsed_seconds']=round(time.time()-START,3)
    if resultfd is None and not OUT.exists():resultfd=os.open(OUT,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    if resultfd is not None:
        with os.fdopen(resultfd,'wb') as f:
            f.write((json.dumps(record,sort_keys=True,indent=2)+'\n').encode('ascii'));f.flush();os.fsync(f.fileno())
    print(json.dumps({'fixture_finished':True,'result':record['result'],'failure_stage':record['failure_stage'],'failure_tag':record['failure_tag'],'request_invalid_reason':record['request_invalid_reason'],'cleanup':record['cleanup'],'owned_child_exits':record['owned_child_exits'],'minimum_free_bytes':record['minimum_free_bytes'],'disk_samples':record['disk_samples'],'elapsed_seconds':record['elapsed_seconds']}),flush=True)
sys.exit(0 if record['result']=='passed' else 1)
PY
```

Audit-append verification passed: all 85920 preceding `cfacf75` bytes remain an
exact prefix; extracting the archived command/body reproduces both full hashes
and the 177-line body parses without execution. Committed helper and all five
historical metadata files retain their bytes/hash/mode. Scope is this report
only; Markdown fences/final newline and `git diff --check` pass. No source or
proposed observer was written/executed and no runtime was acquired/released.

## 2026-10-02: source-only preflow Authorization rejection design

Reservation: wave30_D01_preflow_authorization_rejection_design. This phase owns
only an append to this report. The candidate below exists only in memory and
as a reviewable diff in this report. Disk helper/controller are unchanged; no
helper import/execution, request, provider invocation, CLI fixture, listener,
Driver/browser action, Cargo or runtime acquisition/release occurred. The
timestamp-only candidate in the prior phase is not implemented or proposed for
release. Root must review this immutable report before separately reserving
source ownership or runtime.

Recommendation: this narrow candidate is suitable for source review. A bounded,
otherwise structurally valid Authorization-bearing request during strict
preflow is still refused with fixed 403; only termination of the listening demo
is deferred for the first three such refusals. The fourth 403 is terminal with
the existing fixed request_invalid failure tag. This deliberately interprets
**at most four refusals across the entire fixture** as three nonterminal
refusals plus a fourth terminal refusal; it does not permit four continuations
and a fifth refusal. This policy is explicit for root's review.

There is no new accepted Authorization-bearing request. A rejected request
cannot dispatch to login, callback or protected access, read an application
credential/cookie value, create a redirect/cookie/form, or satisfy a journey
check. Reject any implementation allowing those paths or treating a 403 as
journey success.

### Immutable inputs and prior failures

The complete helper was read from disk and matched source commit
7461ab50f5a5fdf0aa5d7bd6c4b309575e7a4238, helper blob
62460c2f4515a325bb65fc430649b904e8bb2c10, SHA-256
ac3c350d0ea5f1c7448d533524e5db2f99ee37ad61be313c6f36e0c4025a2fcd,
31162 bytes / 671 lines. This is the source published at fixed main
d5b60bc12aaafa258daddd64b40879afacb87bc1; no main update/merge occurred.
Report parent is 3221600b1f1f89194118d07c30d25c24dc6e81fc, blob
851a98844b720ee8a666aa22be2ef4731920d4a6, SHA-256
f75ff6c5b851d66c4599675aae35418316d9459b0bf2fb4528fcb29486554236,
111511 bytes / 1726 lines.

The prior report's retained executed controller payload remains unchanged:
SHA-256 ab0a9c5d8f8fb1c4f5fb5db8b3c396e4dfbdf56bb5a7f0f7e7178b485d77040e;
177-line Python body SHA-256
a3748aaa8aee61fa302050daa78867b1a251fca6cd1294047c43808db71486da.
This phase does not execute/change that controller. Previously reviewed source
possibilities are not proof of the actual request origin.

The actual diagnostic fixture remains failed before browser activity:
request_invalid, stage request, first fixed reason authorization, helper exit 1,
all later journey checks false and their HTTP-status entries null. Sender,
Authorization value and exact first-request timing remain UNKNOWN. There is
no auto-probe/external-origin assertion or product failure/pass inference.
The first fixture's unclassified request refusal, lost first provider-version
output, initial incorrect reporting and later correction remain preserved.
This design creates no actual fixture observation and closes no D01/D05 gate.

### Exact proposed helper diff

In-memory candidate SHA-256 7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0,
32723 bytes / 695 lines.
No candidate helper file was written. The only prospective source file is
scripts/d01-confidential-browser-demo.py; no controller/product/guide/D05/config
or existing test file is proposed for change. The six hunks add the private
integer's initialization/record/final-copy, a local control-flow exception,
and the one rejection branch/catch.

```diff
--- a/scripts/d01-confidential-browser-demo.py
+++ b/scripts/d01-confidential-browser-demo.py
@@ -234,0 +235 @@
+        self.preflow_authorization_refusals = 0
@@ -381,0 +383,4 @@
+class PreflowAuthorizationRefusal(Exception):
+    pass
+
+
@@ -422,0 +428,14 @@
+                if (self.headers.get_all("Authorization") is not None
+                        and demo.attempted is False and demo.pending is None
+                        and demo.cookie is None and demo.subject is None
+                        and demo.preflow_authorization_refusals < 4):
+                    self.record_request_reason("authorization")
+                    self.require_request(self.headers.get_all("Transfer-Encoding") is None, "transfer_encoding")
+                    self.require_request(self.headers.get_all("Expect") is None, "expect")
+                    self.require_request(self.headers.get_all("Content-Length") in (None, ["0"]), "content_length")
+                    target = urllib.parse.urlsplit(self.path)
+                    self.require_request(not target.scheme, "target_scheme")
+                    self.require_request(not target.netloc, "target_netloc")
+                    self.require_request(not target.fragment, "target_fragment")
+                    demo.preflow_authorization_refusals += 1
+                    raise PreflowAuthorizationRefusal
@@ -444,0 +464,4 @@
+        except PreflowAuthorizationRefusal:
+            if demo.preflow_authorization_refusals == 4:
+                demo.failure, demo.done = "request_invalid", True
+            self.reply(403, "Local demo could not complete this request.")
@@ -536 +559 @@
-              "request_invalid_reason": None,
+              "request_invalid_reason": None, "preflow_authorization_refusals": 0,
@@ -623,0 +647 @@
+            record["preflow_authorization_refusals"] = demo.preflow_authorization_refusals
```

### Request and lifecycle invariants

1. Existing request-line/header byte, line and count limits, parser error
   handling, request deadline, connection close and exact Host check execute
   before the new branch. Authorization presence uses the old get_all(...)
   is None distinction: empty and duplicate Authorization headers are present
   and cannot pass the old header-free guard. No Authorization value is
   examined, formatted, logged or retained.
2. Strict eligibility requires attempted is False, pending is None, cookie
   is None, subject is None, and count below four. No state is reset to
   manufacture eligibility. The counter is initialized once per Demo and
   never reset by flow/response/cleanup.
3. Before a nonterminal refusal, the exact old Transfer-Encoding, Expect,
   Content-Length and target scheme/netloc/fragment guards are copied into
   this branch. Bodies remain unread; only absent Content-Length or exactly
   one 0 remains structurally valid. A failed parser/Host/body/target bound
   is terminal through existing handlers and receives no continuation/count.
   Parser/Host and malformed-input responses retain their failure handling
   rather than being advertised as valid 403 continuation cases. The first
   authorization reason is recorded before the additional bounds and cannot
   be overwritten by their rejection. No method/path/Origin/cookie can gain
   accepted routing from this branch.
4. For a bounded eligible request count increases once in range 1..4, then
   the private exception leaves the request budget and bypasses all dispatch.
   Its handler sends only reply(403, "Local demo could not complete this
   request."). No kwargs enable a start form, Location, Set-Cookie or flow-cookie
   clearing. The old fixed response header/body writer is byte-exact. Neither
   cookies(), Cookie-header lookup, credential/subject lookup, begin(),
   callback, token, userinfo or protected check is called by branch/catch.
   Testing required demo.cookie is None is not reading a submitted cookie.
5. Counts 1..3 leave failure, done, attempted, pending, cookie, subject, every
   journey check and every recorded HTTP status unchanged. Existing stage
   assignment to request and first reason recording are diagnostic effects.
   Count 4 sets only failure="request_invalid" and done=True before its same
   403. The unchanged listening loop stops and raises that failure before
   the all-checks success gate. No fifth eligible continuation is possible.
6. Any Authorization-bearing request with one of the four required preflow
   state conditions false falls through to the byte-exact old Authorization
   guard. It remains terminal request_invalid with existing 400. An
   artificially invoked request at count 4 fails that unchanged guard too.
   Header-free requests retain exact old guard order, routing, cookie and
   failure behavior; the extra presence/state test cannot enter this branch.
7. First reason recording is byte-exact and first-only within the finite
   whitelist. The only new retained datum is private integer
   preflow_authorization_refusals (initial 0, maximum 4), copied before clear()
   alongside the old first reason. No sender/raw header/Host/path/query/method/
   Origin/cookie/error is retained. Response/cleanup cannot overwrite the
   reason or reset count. Count records a refusal decision, not proof that
   a peer read the response if a write/deadline failure occurred.
8. The new catch is outside the five-second request budget, so it calls the
   unchanged five-second response budget after unwinding, as the existing
   failure reply does. Response failure or Halt remains failure; no exception
   is suppressed/retried. The 600-second maximum, 180-second pending deadline,
   one-second disk checks, 8.5 GiB stop margin, native/HTTP/CLI budgets, Host/
   ports, confidential exchange, verifier/provider guards and outer 840+60
   plan are unchanged.

### Outcome versus authentication acceptance

The private reason/count describe refused traffic. After 1..3 such 403s a
later separate, header-free browser journey could pass only by satisfying
every unchanged check: pre-cookie protected 403, actual password sign-in and
consent, state/issuer/flow-cookie validation, confidential S256 exchange,
native RS256/JWKS issuer/audience/nonce/time/access-hash verification, matching
userinfo and fresh protected-cookie 200. A success record could therefore
contain first reason authorization and count 1..3 alongside no failure tag;
that describes historical refusal, not accepted Authorization traffic.
Consumers must keep that distinction explicit. No refusal writes a journey
check or recorded HTTP status, and require(all(record["checks"].values()), ...)
remains necessary for success.

There is no observation that the UNKNOWN sender will recur or that this
design will enable the real checkpoint to complete. It defers automatic
teardown for a bounded refused preflow request; it does not authenticate that
request or prove a user task. Physical passkey, tenant, invitation, ordinary
nonadmin and all-category gates remain outside this design; dated evidence
stays dated.

### Static proof actually performed

Python-stdlib source analysis read the pinned helper/report, applied the six
unique replacements to an in-memory string, parsed its AST, and compiled an
in-memory code object with dont_inherit=True. It did not import/execute the
helper/candidate/controller. Reversing all six unique replacements reproduced
the **entire original 31162-byte helper**, with the entire location-free AST
equal too, including every guard/constant/accepted-refused predicate/crypto/
native/deadline/output/cleanup/main entry point outside the additions.

Source-segment comparison of every original function found only Demo.__init__,
Handler.handle_one_request and main changed. These other
31 original functions were byte-exact:

Failure.__init__, Halt.__init__, require, private_directory, private_bytes, load_verifier, client_secret, Budget.__init__, Budget.__enter__, Budget.interrupt, Budget.tick, Budget.limit, Budget.close, Demo.invoke, Demo.setup, Demo.begin, Demo.callback, Demo.clear, HeaderReader.__init__, HeaderReader.readline, DemoServer.__init__, DemoServer.process_request, DemoServer.handle_error, Handler.log_message, Handler.record_request_reason, Handler.require_request, Handler.send_error, Handler.cookies, Handler.get, Handler.reply, QuietParser.error.

The original Authorization guard and entire original body/target block remain
byte-exact. The copied branch block is exactly the old block with one extra
indentation level. AST assertions verified the branch's final statement raises
the private exception, the catch has only the fourth-count failure assignment
and fixed 403 reply without keywords, and neither calls route/cookie/flow/
credential methods. The branch's only assignments are bounded target parsing
and integer increment; it has no check/status/flow assignment. This is source
control-flow proof, not executed protocol validation.

The static analysis exited 0. An initial report-orchestration string had a
JavaScript syntax error before any nested tool/file operation; corrected
quoting generated this append. The first append's static whitespace assertion
then failed on three blank unified-diff context lines containing a single
space; the diff was regenerated with zero context, keeping the exact candidate
and prior report bytes unchanged. These are actual tooling/check failures,
not helper/runtime observations.
The full diff/hash above is reproducible from the six unique replacements.
Complete report prefix and five historical private metadata files were
hash/byte/mode checked without printing their bodies; identity checks are
not added body reviews or runtime evidence.

### Exact future validation and release conditions

Source ownership is HELD. Root should review the strict four-total policy,
new fixed 403 lifecycle and precise six-hunk diff/hash. If separately reserved,
implementation should match this candidate byte-exact, repeat full reversal/
AST/function/guard proofs, and commit source/evidence separately without
changing controller/guide/product/limits.

Before any real fixture, separately reserve ONE focused Python-stdlib
lifecycle check (no dependency/native/provider/CLI/IdP/browser/network call).
Use controlled request input and a stub reply sink; inspect only fixed
statuses/labels/counts/boolean state, never print input values. Verify:

- Bounded Authorization-bearing GET/POST under strict preflow produce fixed
  403 without Location, Set-Cookie or form; no dispatch/cookie/credential/
  protected spy fires. Cover absent versus empty/duplicate Authorization
  presence without logging synthetic values.
- From count 0, refusals 1, 2 and 3 keep all prior flow/check/status snapshots
  identical and the fixture live. Refusal 4 ends with request_invalid, first
  reason authorization and count 4, without journey credit. Counter survives
  connections without reset.
- Each of attempted True, non-None pending/cookie/subject and count already
  4 retains old terminal Authorization rejection. Header-free GET/login/
  callback/protected and invalid method/POST target/Origin/type/cookie cases
  retain the original control-flow/guard outcomes.
- Request-line/header/parser/Host limits and Transfer-Encoding/Expect/nonzero
  or duplicate Content-Length/absolute or fragmented targets fail closed.
  Invalid target parsing fails closed too; no malformed request continues.
  First reason survives response/cleanup failure. Count stays bounded;
  403 write failure/timeout is not success and is not ignored.
- Unchanged reply construction supplies no form/Location/Set-Cookie for a
  bare 403. All checks false remains failure; only the full unchanged journey
  gate can pass. Four-refusal failure cannot pass. Any in-memory fake success
  is check logic, not browser evidence.
- Count survives final private record copying/cleanup. No raw request/error/
  value fields, extra stdout or undocumented flow are introduced.

None of those future cases ran here and no test-file reservation is inferred.
After source review and separately released validation, root may consider
exactly ONE freshly bounded real Driver-only confidential fixture: mandatory
c01 artifacts/verifier/full provider identity and retained metadata, private
lab/XDG, port/PID ownership; pre-cookie 403 then actual browser password/
consent/callback/fresh protected 200; unchanged 900s/840+60/600s limits, disk
floor and owned cleanup. Add no readiness HTTP probe or manufactured
Authorization request to enable that fixture. Unexpected failure stops
without rerun/fallback. Historical failures remain failed and root alone
interprets D01/D05 gates. This design releases no runtime.


Final design-phase readback: the 111511-byte parent report remains an exact
prefix; extracted zero-context candidate diff matches its generated diff;
disk helper remains ac3c350d0ea5f1c7448d533524e5db2f99ee37ad61be313c6f36e0c4025a2fcd;
all five historical private metadata files retain their recorded byte counts,
SHA-256 and 0600 modes. Final AST/in-memory compile/reconstruction/control-flow
and report-only scope checks exited 0; Markdown fence/final-newline/whitespace
and git diff --check passed. No source/helper/controller implementation or
runtime occurred; source/fixture remain HELD pending root review/reservation.

## 2026-10-02: exact bounded refusal source and one failed memory verification

Reservation: wave30_D01_bounded_preflow_authorization_refusals. Root released
the exact source candidate and ONE <=30-second stdlib-only in-memory lifecycle
verification. The source was materialized/committed separately; verification
failed, so it stopped immediately with no rerun or candidate correction.
This append records partial observations and source diagnosis, not a complete
verification pass. The real confidential browser fixture remains HELD.
No desktop/operator/Cargo resource slot was acquired or released.

### Source materialized exactly

Source-only commit 470690cad0cd93c9f25c5bc40b982e1b91679b49, parent
2d2baa58f9a56b6d0370a30597bb7981b61df46d. Its sole changed file is
scripts/d01-confidential-browser-demo.py, blob
3d3379399126423a6a5588a6bd55ea4ac1723ebd, SHA-256
7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0,
32723 bytes / 695 lines. It matches the entire root-reviewed six-hunk
in-memory candidate byte-exact, including three stateless nonterminal 403
refusals, fourth terminal 403, count 0..4 and old post-flow terminal 400.
The prior report was unchanged in the source commit.

Before writing, the entire original ac3c helper was matched to immutable
7461ab50f5a5fdf0aa5d7bd6c4b309575e7a4238. Candidate reversal reconstructed all
31162 original bytes and the whole location-free AST; 31 original functions
were byte-exact. Only Demo.__init__, Handler.handle_one_request and main
changed, plus the new two-line exception class. The exact diff extracted from
the immutable design report matched the generated zero-context diff.
Parser/Host/body/target guards, header-free dispatch/cookie/flow/protected
methods, native/crypto/provider methods, first reason recorder, response writer,
budgets, cleanup and success gate remain as proved in the prior phase.

After the failed memory run, static parse/in-memory compile/reconstruction
and function-segment assertions were repeated against the actual disk helper
and exited 0. Disk bytes still match the reviewed candidate and source commit.
These are static proofs, distinct from the failed execution below; no helper
module, main function, verifier/native/provider routine or controller ran in
those source checks.

### Exactly one executed lifecycle verification

Executed payload SHA-256
81b659352cedd0cc7fce37e32e447d04b0e373ebd9fea5c33f993c8b35b50d96,
89348 bytes / 431 physical lines, including embedded immutable baseline and
candidate source strings. Both source strings were read using Git before
the verification; their identities were asserted inside it. The verification
itself invoked no subprocess/CLI, helper module main, network/socket/listener,
native verifier/provider, product CLI, Driver/browser or Cargo.

The harness used io.BytesIO request/body streams, selected constant/class/
method ASTs and controlled synthetic headers, the actual stdlib request parser,
the helper's unchanged HeaderReader/Handler methods, stub five-second budget/
reply/write/dispatch/flow sinks and a credential-read counter. Only Demo's
constructor and clear method were included; begin/callback were local stubs,
so routing outcomes do not claim OIDC or cryptographic validation. It executed
selected private record-initialization/final-copy and existing outcome-gate
statements directly, never module main. SIGALRM enforced a 30-second cap.
No controlled input/header/cookie/state/subject values were printed or retained
in the result.

Actual command exit: **1**. Harness elapsed 0.019471 seconds; tool wall time
0.137263541 seconds. Result failed/assertion_failed at fixed case
header_free_bound_netloc. It completed 61 of 76 planned cases; one case failed,
14 subsequent cases were never attempted. It made 35 baseline and 60 candidate
in-memory Handler calls (95 total). Expected injected response-write failures
and response timeouts both remained 0 because those cases were later in the
sequence and did not run.

Complete sanitized result emitted by that single execution:

```json
{
  "completed_cases": 61,
  "deadline_seconds": 30,
  "elapsed_seconds": 0.019471,
  "expected_response_timeouts": 0,
  "expected_write_failures": 0,
  "failed_case": "header_free_bound_netloc",
  "failure_tag": "assertion_failed",
  "groups": {
    "authorization_bounds": 9,
    "authorization_presence": 6,
    "bounded_sequence": 4,
    "header_free_bounds": 9,
    "header_free_routes_guards": 25,
    "postflow": 7,
    "record_copy": 1
  },
  "handler_calls": {
    "baseline": 35,
    "candidate": 60
  },
  "helper_sha256": "7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0",
  "main_invoked": false,
  "network_listener_native_provider_cli_driver_browser_cargo": false,
  "result": "failed"
}
```

| Verification group | Completed | Planned | Actual scope |
| --- | ---: | ---: | --- |
| Initial count/final-copy/outcome gate | 1 | 1 | Count 0 copied; incomplete checks cannot pass |
| Cumulative first-three/fourth refusals | 4 | 4 | Counts 1..4, fixed 403, live/live/live/terminal, no dispatch or journey credit |
| Authorization presence/route refusal | 6 | 6 | Empty, duplicate, POST, protected/cookie, callback and other method refused without dispatch |
| Strict state/exhaustion | 7 | 7 | Four state disqualifiers, two non-False attempted values and existing count 4 retain terminal 400 |
| Header-free routes/route guards | 25 | 25 | Baseline/candidate snapshots, replies and stub traces equal |
| Header-free structural bounds | 9 | 12 | First nine matched; tenth case failed before candidate comparison |
| Authorization structural bounds | 9 | 12 | First nine terminal; no continuation/count/flow/check/status changes |
| Parser/request/header limits | 0 | 5 | Not run |
| Counter across header-free flow/cleanup | 0 | 1 | Not run |
| Preexisting first-reason preservation | 0 | 1 | Not run |
| 403 write-failure/response-timeout propagation | 0 | 2 | Not run |

Completed sequence cases copied reason/count at counts 1..4, verified no form/
Location/Set-Cookie, checked all refused-request dispatch/credential counters
remained zero, and invoked the unchanged incomplete-check outcome gate.
The six presence cases also called actual clear(), observing that count 1 and
first reason authorization survived. This is not full counter-through-flow
or response/cleanup-failure coverage; those scheduled cases were not reached.

The 25 header-free parity cases covered root/unavailable/query routing,
protected before/after/mismatched-cookie and missing precondition, callback
routing with stub pending state, login/repeated login, method, POST target/
query, Origin/type missing/wrong/duplicate and cookie count/shape/duplicate/
unrelated handling. Stub protected 200/callback/login outcomes are memory
control-flow observations, never browser sign-in/consent/token success.

The nine structural cases completed for both header-free parity and preflow
Authorization covered missing/wrong/duplicate Host, Transfer-Encoding, Expect,
nonzero/duplicate/noncanonical Content-Length and absolute target scheme.
The failed next case was a network-path target. Its candidate comparison,
Authorization counterpart, fragment and invalid target-parse cases, all raw
parser/line/header limits, flow counter persistence, first-only preset reason,
403 write failure and response timeout remain **unverified by execution**.

### Source-only diagnosis of the stopped assertion

The failed header-free structural case first invokes the baseline, appends
its in-memory outcome, then requires:

```python
assert d.done and d.failure is not None and h.responses == [(400, ())]
```

That case did not complete or reach its candidate request/comparison. The
sanitized result retains its fixed case/failure tag and counts, not the
individual false predicate or the baseline response/status. No actual status
is asserted from the lost in-memory sink.

After stopping, read-only inspection of the installed stdlib source found
BaseHTTPRequestHandler.parse_request normalizes a leading network-path prefix
before returning to the helper. Python version 3.14.6, executable
/opt/homebrew/opt/python@3.14/bin/python3.14; inspected source
/opt/homebrew/opt/python@3.14/Frameworks/Python.framework/Versions/3.14/lib/python3.14/http/server.py,
53122 bytes, SHA-256
b917e19d333ae8aa1995063c9bddf5a15391f8f7caf32e41a22067824d643429.
The parse_request source segment SHA-256 is
2e3a1f90b0ab739e3c2b191b01f0dbf52ab13d4f77e3695bae662db8b2277be4.
Its normalization at line 396 is:

```python
if self.path.startswith('//'):
    self.path = '/' + self.path.lstrip('/')
```

Source inference: the harness's expectation that this parsed network-path
request must hit the helper's netloc rejection is incorrect. The helper
checks the parser's normalized path, not the original raw target. Both old
and new header-free paths still use exactly that same parser and guards.
This is a concrete test-expectation defect identifiable in source, not an
observed product failure or a complete parity pass for the stopped case.
No helper/Host/Authorization/parser guard is relaxed and no sender/probe
origin is inferred. The earlier actual fixture failures stay failed.

### Next validation proposal, not executed or released

Keep source SHA exactly unchanged. If root separately releases a corrected
memory check, classify the normalized network-path example as header-free
old/new routing parity instead of requiring terminal netloc rejection.
Its Authorization-bearing counterpart should expect the bounded preflow
403 refusal when all normalized bounds/state pass, with no dispatch/journey
credit. Check the unchanged target.netloc guard independently against a
controlled parser-result stub retaining a nonempty netloc. This distinguishes
actual stdlib normalization from the helper predicate without changing either.
Report all new observed cases and the preserved first failed verification;
do not silently count this failed run as a pass.

The other planned, unexecuted cases still require separately released
validation: parser/line/header bounds, remaining target failures, counter/
reason persistence across flow/cleanup, and actual propagation from a 403
write sink/response timeout. No correction/rerun happened here, and no new
test/helper file was created. No entire browser fixture is released.
Application/password/consent/callback/protected and native crypto gates
remain unproven by this memory check. Root alone decides source acceptance,
further validation releases and D01/D05 outcome; O06/I10/R05 remain closed.

### Evidence preservation and scope

The entire 128522-byte / 2001-line 2d2baa report is a byte-exact prefix of this
append. All five historical private metadata files retain byte counts,
SHA-256 and 0600 modes; checks read only identities and did not print bodies.
All prior provider refusal/lost values/reporting correction and both failed
fixtures remain intact. The helper remains exactly committed after the failed
check. Controller/product/guide/D05/config/test files and main/status/push were
not changed. Evidence is a separate report-only commit. Final report scope,
source identity, Markdown fences/final newline/whitespace and git diff --check
are checked independently of the failed lifecycle verification.

## 2026-10-02: corrected complete memory harness, design only

Reservation: wave30_D01_corrected_memory_harness_design. Only this existing
report is appended. Root confirmed the prior network-path expectation defect
from source. The harness below is a static proposal: it was parsed/compiled
to in-memory code objects, but never executed or imported. No helper/module
main, controller, native/provider, product CLI, HTTP/socket/listener, Driver/
browser or Cargo ran. No runtime slot was acquired/released; real browser
fixture remains HELD. Root must review this immutable archive before releasing
ONE <=30-second verification.

The first memory run remains **failed**, exit 1, 61 of 76 cases completed,
case header_free_bound_netloc, failure assertion_failed. Its baseline response
and exact false predicate were not retained. Root's source-confirmed diagnosis
does not retroactively attribute an observed status. The proposed expectations
below are future assertions, not results; no pass or D01/D05 closure is claimed.
All earlier fixture/provider/reporting failures remain in their dated phases.

### Source identities and exact payload construction

The helper remains source commit
470690cad0cd93c9f25c5bc40b982e1b91679b49, blob
3d3379399126423a6a5588a6bd55ea4ac1723ebd, SHA-256
7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0,
32723 bytes / 695 lines. No helper/controller/source change is proposed.

The embedded baseline is Git object
7461ab50f5a5fdf0aa5d7bd6c4b309575e7a4238:scripts/d01-confidential-browser-demo.py,
blob 62460c2f4515a325bb65fc430649b904e8bb2c10, SHA-256
ac3c350d0ea5f1c7448d533524e5db2f99ee37ad61be313c6f36e0c4025a2fcd,
31162 bytes / 671 lines. The embedded candidate is the entire exact 470690
object above. Both strings retain their final newline and were source/hash
matched to those objects without execution. No runtime credential material
is embedded.

The readable archive below is complete harness logic. Reconstruct the exact
executable payload by prepending these two lines, in this order, without any
extra separator, then appending the archived logic bytes including its final
newline:

```text
BASELINE_TEXT=<json.dumps(baseline_source, ensure_ascii=True)>\n
SOURCE_TEXT=<json.dumps(candidate_source, ensure_ascii=True)>\n
```

Angle-bracket expressions are assembly instructions, not literal payload
contents; each evaluates to the JSON string literal of the corresponding
whole ASCII source. The displayed backslash-n denotes one actual LF. The
two original serialized source lines are byte-exact and unchanged from
the retained 81b659 payload. Their combined length is 66953 bytes.

| Identity | Bytes | Physical lines | SHA-256 |
| --- | ---: | ---: | --- |
| Original complete payload | 89348 | 431 | 81b659352cedd0cc7fce37e32e447d04b0e373ebd9fea5c33f993c8b35b50d96 |
| Original readable logic | 22395 | 429 | e37a2dd56ed269f2ee8bda90ddc37920e21497544a126832be59a53cb8ce80df |
| Corrected complete payload | 91163 | 463 | ce79aeb99fafaaeb265d6b7bff55f415e91825d83e9c650eed412825a5a9525f |
| Corrected readable logic | 24210 | 461 | 98b19b4d422067b0d49ea7eb1035a719d88bda672dc36579c6cf5c47cb0924eb |
| Exact zero-context diff | 3262 | 57 | ae31333e31149558e01ed45d0e3c1e410699406134d74ee9b5a5be0b3746c815 |

### Narrow correction and proposed case counts

Only five unique replacements change the harness. The request sink gains a
parser_path=None hook which always calls the actual parser first, injecting a
controlled result only when explicitly requested. All 76 existing cases omit
that argument and retain actual stdlib wire parsing.

For the existing normalized network-path case, header-free OLD/NEW routing
must compare equal after the actual parser strips the leading prefix; the
test asserts the normalized path and unchanged generic 404/live outcome.
Its Authorization counterpart must satisfy the existing refused() assertions:
fixed 403, no dispatch/cookie/credential/flow/check/status/Location/Set-Cookie/
form/journey credit, first authorization reason and count 1.

Two added cases separately inject a parser-result path with nonempty netloc
after successfully parsing an ordinary root request: one header-free, one with
Authorization. They require terminal 400, request_invalid, correct finite first
reason (target_netloc or authorization), zero dispatch/credential counters,
unchanged flow/check/status snapshots and candidate count 0, with OLD/NEW
outcomes equal. This explicitly tests the helper's netloc predicate rather
than pretending CPython wire parsing preserves that raw form. The hook changes
only a memory test sink, never helper guards or accepted traffic.

All other cases/assertions remain, including the previously unattempted
parser/request/header limits, remaining target failure cases, counter across
header-free flow/cleanup, first-only reason, 403 write-failure propagation and
response-timeout propagation. Planned counts are expectations only:

| Group | Original planned | Corrected planned |
| --- | ---: | ---: |
| Initial count/final-copy/outcome gate | 1 | 1 |
| First-three/fourth refusals | 4 | 4 |
| Authorization presence/refused routes | 6 | 6 |
| Strict state/exhaustion | 7 | 7 |
| Header-free routes/guards | 25 | 25 |
| Actual-parser header-free structural cases | 12 | 12 |
| Actual-parser Authorization structural cases | 12 | 12 |
| Controlled parser-result netloc rejection | 0 | 2 |
| Parser/request/header limits | 5 | 5 |
| Counter across header-free flow/cleanup | 1 | 1 |
| Preexisting first-reason preservation | 1 | 1 |
| 403 write-failure/response-timeout propagation | 2 | 2 |
| Total | 76 | 78 |

A complete proposed run would make 44 baseline and 80 candidate in-memory
Handler calls (124 total). No such run occurred. begin()/callback() remain
stubs; protected/login/callback memory results cannot prove OIDC, native
crypto, browser consent or actual user success. Output remains fixed sanitized
labels/counts/booleans; controlled inputs are source constructions below,
not printed request/header/cookie/token values.

### Exact original-to-corrected diff

41 added / 9 removed lines, net +32 lines. The original terminal assertions
remain byte-equivalent ASTs inside the non-normalized branches.

```diff
--- original-memory-harness.py
+++ corrected-memory-harness.py
@@ -137 +137 @@
-def request(ns,demo,version,wire,*,write_fail=False,timeout=False):
+def request(ns,demo,version,wire,*,write_fail=False,timeout=False,parser_path=None):
@@ -139,0 +140,5 @@
+        def parse_request(self):
+            parsed=super().parse_request()
+            if parsed and parser_path is not None:
+                self.path=parser_path
+            return parsed
@@ -319 +324,5 @@
-                assert d.done and d.failure is not None and h.responses==[(400,())]
+                if target.startswith("//"):
+                    assert h.path=="/"+target.lstrip("/")
+                    assert not d.done and d.failure is None and h.responses==[(404,())]
+                else:
+                    assert d.done and d.failure is not None and h.responses==[(400,())]
@@ -327,7 +336,11 @@
-            assert d.done and d.failure is not None and h.responses==[(400,())]
-            assert d.preflow_authorization_refusals==0
-            assert d.request_invalid_reason==("host" if host!="normal" or any(n=="Host" for n,v in headers)
-                                             else "authorization")
-            after=flow_snapshot(d)
-            assert after[:4]==before[:4] and after[6:]==before[6:]
-            assert all(v==0 for v in d.calls.values())
+            if target.startswith("//"):
+                assert h.path=="/"+target.lstrip("/")
+                refused(new,d,h,before,1,False)
+            else:
+                assert d.done and d.failure is not None and h.responses==[(400,())]
+                assert d.preflow_authorization_refusals==0
+                assert d.request_invalid_reason==("host" if host!="normal" or any(n=="Host" for n,v in headers)
+                                                 else "authorization")
+                after=flow_snapshot(d)
+                assert after[:4]==before[:4] and after[6:]==before[6:]
+                assert all(v==0 for v in d.calls.values())
@@ -334,0 +348,19 @@
+    for authorization in (False,True):
+        def controlled(authorization=authorization):
+            outcomes=[]
+            parser_path="//"+new["AUTHORITY"]+"/"
+            for ns,version in [(old,"baseline"),(new,"candidate")]:
+                d=make_demo(ns);before=flow_snapshot(d)
+                headers=[("Authorization","ignored")] if authorization else []
+                h=request(ns,d,version,build_wire(ns,headers=headers),parser_path=parser_path)
+                assert h.path==parser_path
+                assert h.responses==[(400,())] and d.done and d.failure=="request_invalid"
+                assert d.request_invalid_reason==("authorization" if authorization else "target_netloc")
+                after=flow_snapshot(d)
+                assert after[:4]==before[:4] and after[6:]==before[6:]
+                assert all(v==0 for v in d.calls.values())
+                if version=="candidate":assert d.preflow_authorization_refusals==0
+                outcomes.append(normalized(ns,d,h))
+            assert outcomes[0]==outcomes[1]
+        verify("controlled_parser_result_"+("authorization" if authorization else "header_free"),
+               "controlled_parser_result",controlled)
```

### Complete corrected readable harness logic

Archive marker: D01_CORRECTED_MEMORY_HARNESS_LOGIC_V1. The following Python
fence is complete logic, not executed by this design phase. Extract all fence
content with its terminating newline; prepend the two pinned serialized-source
lines described above to obtain the exact corrected payload hash.

Root publication encoding: the one `handle_error` indexing call below has a
space before its call parentheses so the repository Markdown checker does not
interpret it as a link. Remove only that space when reconstructing the archived
logic/payload. This rendering normalization restores the exact executed source;
no helper or verification payload changed. All dated author-prefix proofs refer
to their immutable author commits before this publication-only encoding.

```python
import ast, contextlib, copy, hashlib, http.server, io, json, re, signal, time, urllib.parse
from types import SimpleNamespace

started=time.monotonic()
SOURCE_SHA="7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0"
BASE_COMMIT="7461ab50f5a5fdf0aa5d7bd6c4b309575e7a4238"
groups={}
completed=0
case="source_pin"
handler_calls={"baseline":0,"candidate":0}
expected_write_failures=0
expected_timeouts=0

class VerificationDeadline(BaseException):
    pass

def deadline(signum,frame):
    raise VerificationDeadline

def verify(name,group,body):
    global case,completed
    case=name
    body()
    completed+=1
    groups[group]=groups.get(group,0)+1

class BudgetSink:
    def __init__(self):
        self.events=[]
        self.pending_deadline=None
        self.fail_response=False
    @contextlib.contextmanager
    def limit(self,seconds,tag):
        self.events.append((seconds,tag,"enter"))
        if tag=="response_timeout" and self.fail_response:
            raise self.halt("response_timeout")
        try:
            yield
        finally:
            self.events.append((seconds,tag,"exit"))

class WriteFailure(Exception):
    pass

class WriteSink(io.BytesIO):
    def __init__(self):
        super().__init__()
        self.fail=False
    def write(self,value):
        if self.fail:
            raise WriteFailure
        return super().write(value)

def load_selected(text,version):
    tree=ast.parse(text)
    constants=[node for node in tree.body if isinstance(node,ast.Assign)]
    assert all(not any(isinstance(item,ast.Call) for item in ast.walk(node)) for node in constants)
    selected=constants[:]
    for node in tree.body:
        if isinstance(node,ast.ClassDef) and node.name in {
                "Failure","Halt","HeaderReader","Handler","PreflowAuthorizationRefusal"}:
            selected.append(node)
        elif isinstance(node,ast.FunctionDef) and node.name=="require":
            selected.append(node)
        elif isinstance(node,ast.ClassDef) and node.name=="Demo":
            node=copy.deepcopy(node)
            node.body=[method for method in node.body if isinstance(method,ast.FunctionDef)
                       and method.name in {"__init__","clear"}]
            selected.append(node)
    ns={"http":http,"re":re,"urllib":urllib}
    exec(compile(ast.fix_missing_locations(ast.Module(body=selected,type_ignores=[])),
                 "<selected-request-methods-"+version+">","exec"),ns)
    main=next(node for node in tree.body if isinstance(node,ast.FunctionDef) and node.name=="main")
    ns["record_node"]=copy.deepcopy(next(node for node in main.body if isinstance(node,ast.Assign)
        and any(isinstance(t,ast.Name) and t.id=="record" for t in node.targets)))
    main_try=next(node for node in main.body if isinstance(node,ast.Try))
    demo_final=next(node for node in main_try.finalbody if isinstance(node,ast.If)
        and isinstance(node.test,ast.Compare) and isinstance(node.test.left,ast.Name)
        and node.test.left.id=="demo")
    ns["copy_nodes"]=[copy.deepcopy(node) for node in demo_final.body
                     if isinstance(node,ast.Assign) and isinstance(node.targets[0],ast.Subscript)
                     and isinstance(node.targets[0].value,ast.Name)
                     and node.targets[0].value.id=="record"]
    loop_index=next(i for i,node in enumerate(main_try.body) if isinstance(node,ast.While))
    ns["gate_nodes"]=copy.deepcopy(main_try.body[loop_index+1:loop_index+4])
    server=next(node for node in tree.body if isinstance(node,ast.ClassDef) and node.name=="DemoServer")
    error=copy.deepcopy(next(node for node in server.body if isinstance(node,ast.FunctionDef)
                            and node.name=="handle_error"))
    exec(compile(ast.Module(body=[error],type_ignores=[]),"<server-error-method>","exec"),ns)
    return ns

def make_demo(ns):
    names=("credential_private_validated","provider_identity_verified","discovery_verified",
        "protected_without_cookie_denied","authorization_redirect_issued",
        "state_issuer_flow_cookie_verified","confidential_s256_exchange_verified",
        "rs256_jwks_issuer_audience_nonce_time_access_hash_verified",
        "userinfo_subject_verified","protected_with_fresh_cookie_accepted")
    class DemoSink(ns["Demo"]):
        def __getattribute__(self,name):
            if name=="secret":
                access=object.__getattribute__(self,"calls")
                access["credential"]+=1
            return object.__getattribute__(self,name)
        def begin(self):
            self.calls["begin"]+=1
            ns["require"](not self.attempted,"flow_already_started")
            self.stage="flow"
            self.attempted=True
            self.pending={}
            return ns["ISSUER"]+"/authorize","a"*43
        def callback(self,query,cookie):
            self.calls["callback"]+=1
            ns["require"](self.pending is not None,"callback_already_consumed")
            self.pending=None
            self.cookie="b"*43
            self.subject="synthetic"
            return self.cookie
    budget=BudgetSink()
    budget.halt=ns["Halt"]
    demo=DemoSink(SimpleNamespace(equal=lambda left,right:left==right),None,object(),budget,
                  {name:i<3 for i,name in enumerate(names)},
                  {name:None for name in ("authorization_redirect","callback","token_exchange",
                   "userinfo","protected_before","protected_after")})
    demo.calls={"get":0,"cookies":0,"begin":0,"callback":0,"credential":0}
    return demo

def flow_snapshot(demo):
    return (demo.attempted,demo.pending,demo.cookie,demo.subject,
            demo.failure,demo.done,copy.deepcopy(demo.checks),copy.deepcopy(demo.statuses))

def build_wire(ns,method="GET",target="/",headers=(),host="normal"):
    prefix=[] if host=="absent" else [("Host",ns["AUTHORITY"] if host=="normal" else "invalid")]
    return ((method+" "+target+" HTTP/1.1\r\n"
            +"".join(name+": "+value+"\r\n" for name,value in prefix+list(headers))
            +"\r\n").encode("ascii"))

def request(ns,demo,version,wire,*,write_fail=False,timeout=False,parser_path=None):
    handler_calls[version]+=1
    class HandlerSink(ns["Handler"]):
        def parse_request(self):
            parsed=super().parse_request()
            if parsed and parser_path is not None:
                self.path=parser_path
            return parsed
        def get(self,target):
            demo.calls["get"]+=1
            return super().get(target)
        def cookies(self):
            demo.calls["cookies"]+=1
            return super().cookies()
        def reply(self,status,text,**options):
            self.responses.append((status,tuple(sorted(options))))
            return super().reply(status,text,**options)
        def send_response(self,status,message=None):
            self.sent_statuses.append(status)
        def send_header(self,name,value):
            self.sent_headers.append((name,value))
        def end_headers(self):
            self.header_ends+=1
    handler=object.__new__(HandlerSink)
    handler.server=SimpleNamespace(demo=demo)
    handler.rfile=io.BytesIO(wire)
    handler.wfile=WriteSink()
    handler.wfile.fail=write_fail
    handler.responses=[]
    handler.sent_statuses=[]
    handler.sent_headers=[]
    handler.header_ends=0
    demo.budget.fail_response=timeout
    handler.handle_one_request()
    return handler

def normalized(ns,demo,handler):
    return (flow_snapshot(demo),demo.stage,demo.request_invalid_reason,demo.calls,
            handler.responses,handler.sent_statuses,handler.sent_headers,handler.wfile.getvalue(),
            demo.budget.events)

def record_copy(ns,demo):
    env=dict(ns)
    env["demo"]=demo
    exec(compile(ast.Module(body=[ns["record_node"]],type_ignores=[]),"<record-initialization>","exec"),env)
    exec(compile(ast.Module(body=ns["copy_nodes"],type_ignores=[]),"<record-final-copy>","exec"),env)
    return env["record"]

def gate_fails(ns,demo):
    env=dict(ns)
    env["demo"]=demo
    env["record"]={"checks":demo.checks,"result":"failed"}
    try:
        exec(compile(ast.Module(body=ns["gate_nodes"],type_ignores=[]),"<existing-outcome-gate>","exec"),env)
    except ns["Failure"] as failure:
        assert failure.tag in {"request_invalid","unexpected_failure"}
    else:
        raise AssertionError
    assert env["record"]["result"]=="failed"

def refused(ns,demo,handler,before,count,terminal):
    assert handler.responses==[(403,())] and handler.sent_statuses==[403]
    assert handler.header_ends==1
    assert all(name not in {"Location","Set-Cookie"} for name,value in handler.sent_headers)
    assert b"<form" not in handler.wfile.getvalue()
    assert demo.calls=={"get":0,"cookies":0,"begin":0,"callback":0,"credential":0}
    after=flow_snapshot(demo)
    assert after[:4]==before[:4] and after[6:]==before[6:]
    assert demo.preflow_authorization_refusals==count
    assert demo.request_invalid_reason=="authorization"
    assert demo.failure==("request_invalid" if terminal else before[4])
    assert demo.done==(True if terminal else before[5])
    assert all(not value for key,value in demo.checks.items() if key not in
               {"credential_private_validated","provider_identity_verified","discovery_verified"})
    assert all(value is None for value in demo.statuses.values())
    copied=record_copy(ns,demo)
    assert copied["preflow_authorization_refusals"]==count
    assert copied["request_invalid_reason"]=="authorization"
    gate_fails(ns,demo)

def run():
    global expected_write_failures,expected_timeouts
    text=SOURCE_TEXT
    assert hashlib.sha256(text.encode()).hexdigest()==SOURCE_SHA
    baseline_text=BASELINE_TEXT
    assert hashlib.sha256(baseline_text.encode()).hexdigest()=="ac3c350d0ea5f1c7448d533524e5db2f99ee37ad61be313c6f36e0c4025a2fcd"
    old=load_selected(baseline_text,"baseline")
    new=load_selected(text,"candidate")
    demo=make_demo(new)
    assert record_copy(new,demo)["preflow_authorization_refusals"]==0
    verify("initial_count_copy","record_copy",lambda:gate_fails(new,demo))
    for count in range(1,5):
        def check(count=count):
            before=flow_snapshot(demo)
            handler=request(new,demo,"candidate",build_wire(new,headers=[("Authorization","ignored")]))
            refused(new,demo,handler,before,count,count==4)
        verify("bounded_refusal_"+str(count),"bounded_sequence",check)
    for label,headers,method,target in [
        ("empty",[("Authorization","")],"GET","/"),
        ("duplicate",[("Authorization","ignored"),("Authorization","")],"GET","/"),
        ("post",[("Authorization","ignored")],"POST","/login"),
        ("protected",[("Authorization","ignored"),("Cookie","d01_local_demo="+"b"*43)],"GET","/protected"),
        ("callback",[("Authorization","ignored")],"GET","/callback"),
        ("other_method",[("Authorization","ignored")],"PUT","/")]:
        def check(headers=headers,method=method,target=target):
            d=make_demo(new);before=flow_snapshot(d)
            h=request(new,d,"candidate",build_wire(new,method,target,headers))
            refused(new,d,h,before,1,False)
            d.clear()
            assert d.preflow_authorization_refusals==1 and d.request_invalid_reason=="authorization"
        verify("presence_"+label,"authorization_presence",check)
    for field,value in [("attempted",True),("pending",{}),("cookie","b"*43),
                        ("subject","synthetic"),("attempted",0),("attempted",None)]:
        def check(field=field,value=value):
            d=make_demo(new);setattr(d,field,value);before=flow_snapshot(d)
            h=request(new,d,"candidate",build_wire(new,headers=[("Authorization","ignored")]))
            after=flow_snapshot(d)
            assert h.responses==[(400,())] and d.failure=="request_invalid" and d.done
            assert d.preflow_authorization_refusals==0 and d.request_invalid_reason=="authorization"
            assert after[:4]==before[:4] and after[6:]==before[6:]
            assert all(v==0 for v in d.calls.values())
        verify("strict_state_"+field+"_"+str(groups.get("postflow",0)+1),"postflow",check)
    def exhausted():
        d=make_demo(new);d.preflow_authorization_refusals=4
        h=request(new,d,"candidate",build_wire(new,headers=[("Authorization","ignored")]))
        assert h.responses==[(400,())] and d.done and d.failure=="request_invalid"
        assert d.preflow_authorization_refusals==4 and all(v==0 for v in d.calls.values())
    verify("defensive_exhaustion","postflow",exhausted)
    login=[("Origin",new["ORIGIN"]),("Content-Type","application/x-www-form-urlencoded"),("Content-Length","0")]
    specifications=[
        ("root","GET","/",[],None),("protected_before","GET","/protected",[],None),
        ("unavailable","GET","/unavailable",[],None),("root_query","GET","/?x",[],None),
        ("protected_query","GET","/protected?x",[],None),
        ("callback_empty","GET","/callback",[],None),
        ("callback_ready","GET","/callback",[],{"pending":{}}),
        ("login","POST","/login",login,None),
        ("login_repeated","POST","/login",login,{"attempted":True}),
        ("protected_after","GET","/protected",[("Cookie","d01_local_demo="+"b"*43)],
            {"cookie":"b"*43,"subject":"synthetic","protected_before":True}),
        ("protected_before_missing","GET","/protected",[("Cookie","d01_local_demo="+"b"*43)],
            {"cookie":"b"*43,"subject":"synthetic"}),
        ("protected_cookie_mismatch","GET","/protected",[("Cookie","d01_local_demo="+"c"*43)],
            {"cookie":"b"*43,"subject":"synthetic"}),
        ("method","PUT","/",[],None),
        ("post_target","POST","/wrong",login,None),("post_query","POST","/login?x",login,None),
        ("origin_missing","POST","/login",[("Content-Type","application/x-www-form-urlencoded")],None),
        ("origin_wrong","POST","/login",[("Origin","invalid"),("Content-Type","application/x-www-form-urlencoded")],None),
        ("origin_duplicate","POST","/login",login+[("Origin",new["ORIGIN"])],None),
        ("type_missing","POST","/login",[("Origin",new["ORIGIN"])],None),
        ("type_wrong","POST","/login",[("Origin",new["ORIGIN"]),("Content-Type","text/plain")],None),
        ("type_duplicate","POST","/login",login+[("Content-Type","application/x-www-form-urlencoded")],None),
        ("cookie_duplicate","GET","/",[("Cookie","other=x"),("Cookie","other=x")],None),
        ("cookie_shape","GET","/",[("Cookie","d01_demo_flow=x")],None),
        ("cookie_own_duplicate","GET","/",[("Cookie","d01_demo_flow="+"a"*43+"; d01_demo_flow="+"a"*43)],None),
        ("cookie_unrelated","GET","/",[("Cookie","unrelated=x")],None)]
    structural=[
        ("host_absent","/","absent",[]),("host_wrong","/","wrong",[]),
        ("host_duplicate","/","normal",[("Host",new["AUTHORITY"])]),
        ("transfer","/","normal",[("Transfer-Encoding","chunked")]),
        ("expect","/","normal",[("Expect","100-continue")]),
        ("length_nonzero","/","normal",[("Content-Length","1")]),
        ("length_duplicate","/","normal",[("Content-Length","0"),("Content-Length","0")]),
        ("length_format","/","normal",[("Content-Length","00")]),
        ("scheme","http://localhost:3000/","normal",[]),
        ("netloc","//localhost:3000/","normal",[]),("fragment","/#x","normal",[]),
        ("parse_target","http://[","normal",[])]
    for label,method,target,headers,initial in specifications:
        def check(method=method,target=target,headers=headers,initial=initial):
            outcomes=[]
            for ns,version in [(old,"baseline"),(new,"candidate")]:
                d=make_demo(ns)
                for field,value in (initial or {}).items():
                    if field=="protected_before":d.checks["protected_without_cookie_denied"]=value
                    else:setattr(d,field,copy.deepcopy(value))
                h=request(ns,d,version,build_wire(ns,method,target,headers))
                outcomes.append(normalized(ns,d,h))
                if version=="candidate":
                    assert d.preflow_authorization_refusals==0
            assert outcomes[0]==outcomes[1]
        verify("header_free_"+label,"header_free_routes_guards",check)
    for label,target,host,headers in structural:
        def check(target=target,host=host,headers=headers):
            outcomes=[]
            for ns,version in [(old,"baseline"),(new,"candidate")]:
                d=make_demo(ns)
                h=request(ns,d,version,build_wire(ns,target=target,headers=headers,host=host))
                outcomes.append(normalized(ns,d,h))
                if target.startswith("//"):
                    assert h.path=="/"+target.lstrip("/")
                    assert not d.done and d.failure is None and h.responses==[(404,())]
                else:
                    assert d.done and d.failure is not None and h.responses==[(400,())]
                if version=="candidate":assert d.preflow_authorization_refusals==0
            assert outcomes[0]==outcomes[1]
        verify("header_free_bound_"+label,"header_free_bounds",check)
        def auth_check(target=target,host=host,headers=headers):
            d=make_demo(new);before=flow_snapshot(d)
            h=request(new,d,"candidate",build_wire(new,target=target,host=host,
                headers=[("Authorization","ignored")]+headers))
            if target.startswith("//"):
                assert h.path=="/"+target.lstrip("/")
                refused(new,d,h,before,1,False)
            else:
                assert d.done and d.failure is not None and h.responses==[(400,())]
                assert d.preflow_authorization_refusals==0
                assert d.request_invalid_reason==("host" if host!="normal" or any(n=="Host" for n,v in headers)
                                                 else "authorization")
                after=flow_snapshot(d)
                assert after[:4]==before[:4] and after[6:]==before[6:]
                assert all(v==0 for v in d.calls.values())
        verify("authorization_bound_"+label,"authorization_bounds",auth_check)
    for authorization in (False,True):
        def controlled(authorization=authorization):
            outcomes=[]
            parser_path="//"+new["AUTHORITY"]+"/"
            for ns,version in [(old,"baseline"),(new,"candidate")]:
                d=make_demo(ns);before=flow_snapshot(d)
                headers=[("Authorization","ignored")] if authorization else []
                h=request(ns,d,version,build_wire(ns,headers=headers),parser_path=parser_path)
                assert h.path==parser_path
                assert h.responses==[(400,())] and d.done and d.failure=="request_invalid"
                assert d.request_invalid_reason==("authorization" if authorization else "target_netloc")
                after=flow_snapshot(d)
                assert after[:4]==before[:4] and after[6:]==before[6:]
                assert all(v==0 for v in d.calls.values())
                if version=="candidate":assert d.preflow_authorization_refusals==0
                outcomes.append(normalized(ns,d,h))
            assert outcomes[0]==outcomes[1]
        verify("controlled_parser_result_"+("authorization" if authorization else "header_free"),
               "controlled_parser_result",controlled)
    raw_bounds=[
        ("request_line",b"GET /"+b"x"*8192+b" HTTP/1.1\r\n\r\n"),
        ("header_line",b"GET / HTTP/1.1\r\nHost: localhost:3000\r\nX: "+b"x"*4096+b"\r\n\r\n"),
        ("header_count",b"GET / HTTP/1.1\r\nHost: localhost:3000\r\n"+b"X: x\r\n"*33+b"\r\n"),
        ("header_total",b"GET / HTTP/1.1\r\nHost: localhost:3000\r\n"+(b"X: "+b"x"*3000+b"\r\n")*3+b"\r\n"),
        ("parser",b"GET / HTTP/9.0\r\nHost: localhost:3000\r\n\r\n")]
    for label,wire in raw_bounds:
        def check(wire=wire):
            outcomes=[]
            for ns,version in [(old,"baseline"),(new,"candidate")]:
                d=make_demo(ns)
                h=request(ns,d,version,wire)
                outcomes.append(normalized(ns,d,h))
                assert d.done and d.failure in {"request_invalid","request_limit"}
                assert all(v==0 for v in d.calls.values())
            assert outcomes[0]==outcomes[1]
        verify("raw_bound_"+label,"parser_header_limits",check)
    def persist():
        d=make_demo(new)
        request(new,d,"candidate",build_wire(new,headers=[("Authorization","ignored")]))
        request(new,d,"candidate",build_wire(new))
        assert d.preflow_authorization_refusals==1
        request(new,d,"candidate",build_wire(new,"POST","/login",login))
        assert d.attempted and d.preflow_authorization_refusals==1
        before=flow_snapshot(d)
        request(new,d,"candidate",build_wire(new,headers=[("Authorization","ignored")]))
        after=flow_snapshot(d)
        assert after[:4]==before[:4] and after[6:]==before[6:]
        assert d.preflow_authorization_refusals==1 and d.request_invalid_reason=="authorization"
        copied=record_copy(new,d)
        d.clear()
        assert copied["preflow_authorization_refusals"]==1
        assert d.preflow_authorization_refusals==1 and d.request_invalid_reason=="authorization"
    verify("counter_across_header_free_flow_cleanup","counter_persistence",persist)
    def first_only():
        d=make_demo(new);d.request_invalid_reason="http_parse"
        h=request(new,d,"candidate",build_wire(new,headers=[("Authorization","ignored")]))
        assert h.responses==[(403,())] and d.request_invalid_reason=="http_parse"
        assert d.preflow_authorization_refusals==1
        d.clear()
        assert d.request_invalid_reason=="http_parse" and d.preflow_authorization_refusals==1
    verify("first_reason_preserved","first_reason",first_only)
    def write_failure():
        global expected_write_failures
        d=make_demo(new);before=flow_snapshot(d)
        try:
            request(new,d,"candidate",build_wire(new,headers=[("Authorization","ignored")]),write_fail=True)
        except WriteFailure:
            expected_write_failures+=1
        else:
            raise AssertionError
        assert d.preflow_authorization_refusals==1 and d.request_invalid_reason=="authorization"
        assert flow_snapshot(d)==before and all(v==0 for v in d.calls.values())
        new["handle_error"] (SimpleNamespace(demo=d),None,None)
        assert d.failure=="unexpected_failure" and d.done
        assert d.preflow_authorization_refusals==1 and d.request_invalid_reason=="authorization"
        assert record_copy(new,d)["preflow_authorization_refusals"]==1
    verify("403_write_failure_propagates","response_failures",write_failure)
    def response_timeout():
        global expected_timeouts
        d=make_demo(new)
        try:
            request(new,d,"candidate",build_wire(new,headers=[("Authorization","ignored")]),timeout=True)
        except new["Halt"] as failure:
            assert failure.tag=="response_timeout"
            expected_timeouts+=1
        else:
            raise AssertionError
        assert d.preflow_authorization_refusals==1 and d.request_invalid_reason=="authorization"
        assert all(v==0 for v in d.calls.values())
        gate_fails(new,d)
    verify("403_response_timeout_propagates","response_failures",response_timeout)

result="failed"
failure=None
signal.signal(signal.SIGALRM,deadline)
signal.setitimer(signal.ITIMER_REAL,30)
try:
    run()
    result="passed"
except VerificationDeadline:
    failure="verification_deadline"
except AssertionError:
    failure="assertion_failed"
except BaseException:
    failure="harness_exception"
finally:
    signal.setitimer(signal.ITIMER_REAL,0)
elapsed=round(time.monotonic()-started,6)
print(json.dumps({"result":result,"failure_tag":failure,"failed_case":case if failure else None,
 "completed_cases":completed,"groups":groups,"handler_calls":handler_calls,
 "expected_write_failures":expected_write_failures,"expected_response_timeouts":expected_timeouts,
 "elapsed_seconds":elapsed,"deadline_seconds":30,"helper_sha256":SOURCE_SHA,
 "main_invoked":False,"network_listener_native_provider_cli_driver_browser_cargo":False},sort_keys=True))
raise SystemExit(0 if result=="passed" else 1)
```

### Static-only preservation proof and limits

The original retained complete payload was hash/length matched to 81b659 and
reconstructed from the unchanged serialized-source prefix plus original logic.
Applying the five unique replacements in memory created the corrected logic/
payload. Reversing all five reconstructed the **entire original payload
byte-exact**, including both source strings and final newline; its full AST
matched too. No source or harness was imported or executed.

Top-level source-segment comparison found only request and run changed;
these other 14 functions/classes remained byte-exact:

VerificationDeadline, deadline, verify, BudgetSink, WriteFailure, WriteSink, load_selected, make_demo, flow_snapshot, build_wire, normalized, record_copy, gate_fails, refused.

The prior auth_check assertions are AST-identical to the corrected
non-normalized else branch. Existing runtime result/exception/signal/deadline/
sanitized output tail is byte-exact; the 30-second alarm, stop-on-first-failure,
finite failure tags, expected injected failures and no-main behavior remain.
The embedded-source prefix, setup factories, credential counters, refusal/
outcome-gate/record-copy assertions, memory write sink and stub budgets remain
exact. The parser-path hook is absent from every original call and used only
for the two new controlled-result cases. AST name inspection found no
subprocess/socket/os/Path references in the harness logic. These static
checks and unexecuted AST/in-memory compilation exited 0; they establish
preservation, not success of any proposed case.

Before any future execution, root should review the entire archived logic,
the exact diff and both embedded source identities. A separately released
run must use the exact corrected payload hash, retain its complete fixed
result/exit/counts, and stop on its first unexpected failure without correction
or retry. Record the original failed 61-of-76 run separately even if a later
run passes. No browser/runtime release is implied by this report.

This append preserves the entire eb322 report (139821 bytes / 2209 lines,
SHA-256 dfe2d894b43a6b8ca507dc5f55d87b3cd41723604e9959bffd89716dc7f5186f)
as an exact prefix and all five private historical metadata files with their
recorded bytes/hash/0600 modes. Hash checks add no body reviews or external
evidence. Helper470 SHA7fbc is unchanged; controller/product/guide/D05/config/
tests/main/status/push remain untouched. Report-only scope, archive extraction/
hash reconstruction, Markdown fence/final-newline/whitespace and git diff
--check are verified before this separate report-only commit.

## 2026-10-02: released corrected memory verification, actual complete result

Reservation: wave30_D01_corrected_memory_verification. Root fully reviewed
the corrected archive and independently reconstructed its payload before
explicitly releasing exactly ONE standard-library memory verification. This
phase executed that exact payload once, retained its complete fixed JSON/
numeric exit/timing/hash before expectation comparisons, immediately handed
off the actual result to the explicit project orchestrator, and then appended
this evidence. No correction, retry, new case or source/harness change occurred.
The entire real confidential browser fixture remains HELD for root review.

### Pins, execution and retention order

Preflight started from clean
ab3fbace55465e9725ad230ff9eca160cdcc2856. The report matched that Git object,
SHA-256 0d5520f3fa7212757b595dc44fdd5a51a089700683b86f652b3be06f97d42904,
176534 bytes / 2897 lines. Extracted complete readable logic matched SHA-256
98b19b4d422067b0d49ea7eb1035a719d88bda672dc36579c6cf5c47cb0924eb.
The immutable ac3c baseline at 7461ab50 and the exact 7fbc candidate at
470690cad0cd93c9f25c5bc40b982e1b91679b49 were read before verification and
serialized as the two reviewed source lines. Payload reconstruction matched
SHA-256 ce79aeb99fafaaeb265d6b7bff55f415e91825d83e9c650eed412825a5a9525f,
91163 bytes / 463 physical lines. Disk helper matched the immutable candidate
and SHA-256
7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0.
All five prior private metadata hashes/lengths/0600 modes matched before
execution. Source preflight parsed/compiled only; it did not run the harness.

A small outer Python controller launched one owned Python child with the exact
payload on stdin and a 35-second outer communicate timeout. The reviewed
payload retained its unchanged 30-second SIGALRM deadline and stop-on-first-
unexpected-failure behavior. The outer controller alone used subprocess to
enforce the bound and reap the child; the payload made no subprocess/CLI/
network/socket/listener/native/provider/Driver/browser/Cargo calls and did
not import a helper module or invoke module main. It only executed selected
request methods and the reviewed memory sinks.

Outer controller source SHA-256 ab966e651f0a3f572f2d3ac2c27e39faf5cb63ef6c2f86be4a942ab6c8fc8b6d,
99002 bytes / 37 physical lines.
The exact outer tool result, then the complete captured child stdout/stderr,
were retained in session memory before parsing the fixed child JSON and before
any success/count comparisons. No private deployment evidence file was
created or overwritten. The exact fixed stdout is durably archived below;
741 UTF-8 bytes, SHA-256 81c7a85c6b278bb60412be566a0bc07732fbc50c1ead9aa6030d07c14648dc5f, including its final LF. Stderr was empty,
0 bytes. No request/header/cookie/code/state/subject/private-value output was
printed.

### Actual result and complete fixed JSON

**Child exit 0; result passed; all 78 cases completed.** Actual Handler calls:
44 baseline / 80 candidate, 124 total. No first failure and no unreached case.
The injected 403 write-failure branch ran once and the injected response-
timeout branch ran once; both met their reviewed propagation assertions.
These expected injected failures are successful check observations, not
unexpected fixture failures.

Child elapsed 0.021527 seconds with the 30-second cap. Outer elapsed 0.064020
seconds with the 35-second timeout, timeout false, outer failure tag null,
child reaped true. Tool wall time 0.168740042 seconds; outer tool exit 0.
The stdout/exit/hash/timing values above were retained before comparison.

Exact emitted child JSON (fence content preserves the entire 741-byte stdout):

```json
{"completed_cases": 78, "deadline_seconds": 30, "elapsed_seconds": 0.021527, "expected_response_timeouts": 1, "expected_write_failures": 1, "failed_case": null, "failure_tag": null, "groups": {"authorization_bounds": 12, "authorization_presence": 6, "bounded_sequence": 4, "controlled_parser_result": 2, "counter_persistence": 1, "first_reason": 1, "header_free_bounds": 12, "header_free_routes_guards": 25, "parser_header_limits": 5, "postflow": 7, "record_copy": 1, "response_failures": 2}, "handler_calls": {"baseline": 44, "candidate": 80}, "helper_sha256": "7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0", "main_invoked": false, "network_listener_native_provider_cli_driver_browser_cargo": false, "result": "passed"}
```

| Actual group | Completed | What the reviewed assertions established in memory |
| --- | ---: | --- |
| Initial count/final-copy/outcome gate | 1 | Count 0 copy; incomplete checks cannot pass |
| Cumulative first-three/fourth refusals | 4 | First 3 fixed 403/live, fourth fixed 403/terminal request_invalid, count 1..4 and no journey credit |
| Authorization presence/refused routes | 6 | Empty/duplicate/POST/protected/callback/other method refusal; no dispatch/credential/cookie use |
| Strict state/exhaustion | 7 | Four lifecycle disqualifiers, two non-False attempted values and defensive count 4 preserve terminal 400 |
| Header-free routes/route guards | 25 | OLD/NEW state/reply/stub-call parity |
| Actual-parser header-free structural cases | 12 | Prior guards match, including normalized network-path routing parity |
| Actual-parser Authorization structural cases | 12 | Bounds fail closed; normalized network-path counterpart refused with bounded 403 and no credit |
| Controlled parser-result netloc rejection | 2 | Injected nonempty netloc separately yields OLD/NEW terminal 400, correct fixed first reason and zero dispatch/count |
| Parser/request/header limits | 5 | Request-line/header-line/header-count/header-total/parser refusal parity |
| Counter across header-free flow/cleanup | 1 | Counter/reason survive root/login/post-flow refusal and clear |
| Preexisting first-reason preservation | 1 | Existing first fixed reason is not overwritten |
| 403 write-failure/response-timeout propagation | 2 | Writer failure escapes refusal catch and existing error sink fails closed; response Halt propagates |
| Total | 78 | Complete released check; no case pending |

All previously unattempted parser limits, remaining target failure assertions,
counter-through-flow, first-only reason, write-failure and timeout branches
executed in this one corrected run. The count/final-copy assertions covered
0..4. The reviewed branch assertions checked no refusal dispatch, secret/cookie
read, flow/check/status mutation, form, Location, Set-Cookie or journey credit.

The wire network-path case used the actual stdlib parser with no override;
its normalized path assertions and OLD/NEW generic routing outcomes passed
in this new memory run. Its Authorization counterpart stayed a refused 403.
The two separately labeled controlled-result cases invoked the actual parser
first and then explicitly injected only a path value in the test sink to
exercise the unchanged nonempty-netloc predicate. They do not claim wire
parsing exposes that netloc. No helper parser/Host/Authorization/target guard
was changed.

### Evidence limits and preserved failed run

This result verifies the exact bounded lifecycle/guard/sink assertions above.
begin()/callback() remain controlled stubs and module main never ran. Memory
login/callback/protected results do not prove actual browser sign-in, consent,
OIDC exchange, native verifier/JWKS/RS256, issuer/userinfo, a physical passkey,
ordinary nonadmin/invitation/tenant workflow or full D01/D05 acceptance.
No real browser, provider, IdP, CLI fixture or listening process was started.
No desktop/operator/Cargo resource slot was acquired or released; no service/
listener/profile cleanup was necessary for this memory-only run. The one
owned Python child was reaped.

The original memory execution still failed: exit 1, 61 of 76 completed,
header_free_bound_netloc/assertion_failed, with its response/predicate not
retained. The now-passing corrected expectations do not attribute an actual
response to that old run or rewrite its outcome. All historical provider
refusal/lost-value/reporting correction and actual browser-fixture failures
remain unchanged in prior phases. Sender of the historical Authorization
request remains UNKNOWN. Root alone reviews this actual complete result and
decides any further browser release or D01/D05 gate; O06/I10/R05 remain closed.

### Report-only preservation checks

The complete 176534-byte / 2897-line ab3fbace report remains a byte-exact
prefix. The exact archived fixed JSON parses to the retained complete result,
reproduces its 741-byte stdout hash and matches the retained exit/count/hash/
elapsed metadata. Expected group counts sum to 78 with 44/80 calls, injected
failures/timeouts 1/1 and no failure/unreached cases. These comparisons occurred
after complete retention and immediate root handoff.

Payload reconstruction from the unchanged archived logic and source objects
still matches ce79 SHA. Disk helper remains exactly immutable 470690/7fbc;
all five historical private metadata files retain byte counts, hash and 0600
modes without body output. Source/controller/product/guide/D05/config/tests/
main/status/push remain untouched. Report scope, archive extraction, prefix,
source identity, Markdown fences/final newline/whitespace and git diff --check
are checked before the separate evidence-only commit. No broader campaign
or additional check execution occurred.

## 2026-10-02: bounded confidential browser release refused by fresh capacity

Reservation: wave30_D01_bounded_confidential_browser_checkpoint. Root released
one fresh confidential localhost fixture on the unchanged reviewed helper,
assigned the sole desktop/operator controller, and retained Cargo with other
owners. This phase **refused before fixture execution**: the fresh shared-volume
capacity sample was below both the 8.5 GiB start/stop margin and 8 GiB floor.
No automatic repeat, capacity cleanup, provider fallback, source correction,
or entire fixture execution occurred. Immediate desktop/operator RELEASE was
sent to the explicit project orchestrator after ending the one named Driver
lifecycle session and obtaining cleanup readbacks, before this appendix.

### Actual prerequisite and refusal

The read-only preflight command exited 0 and emitted:

```json
{"free_bytes":4741165056,"private_mode":"0o700","fresh_result_absent":true,"fresh_provider_absent":true}
```

4741165056 bytes is 4.415554 GiB. Required start/stop margin:
9126805504 bytes (8.5 GiB); absolute floor 8589934592 bytes (8 GiB).
This actual sample was already below the floor. The task stopped before
controller preparation/execution, native/provider version invocation, any
maintenance/server/base-client/helper command, secret/password generation,
lab/XDG creation, listener launch, browser/profile preparation or GUI action.

This is a shared-capacity prerequisite refusal, not a product/guide/protocol
failure or a browser pass. Fixture command count is 0, helper invocations 0,
browser preparations/actions 0. There is no fixture START or active/cleanup
timer, provider exit/stdout/stderr, signing result, browser target/tab binding,
authorization/refusal counter, cookie/redirect/callback/token/userinfo/protected
status to attribute to this phase. Those values are unobserved/not applicable,
not zero-success substitutes.

The requested fresh paths remained absent and were not populated with an
invented controller/helper result or provider metadata:

- deployment-private/d01-confidential-browser-bounded.redacted.json
- deployment-private/d01-confidential-browser-bounded-provider-20261002.json

A new updated public controller payload was not prepared or executed after
the capacity failure. The prior retained 177-line controller remains unchanged,
command SHA-256
ab0a9c5d8f8fb1c4f5fb5db8b3c396e4dfbdf56bb5a7f0f7e7178b485d77040e.
No new payload hash/execution/protocol observation is claimed.

### Driver-only preflight and lifecycle cleanup actually observed

Read the installed cua-driver SKILL.md (skill source version 0.29.1), MACOS.md
and BROWSER.md, plus advertised MCP descriptions before any GUI action.
Provider stayed RiWork Cua.ai Driver MCP. History tools were not advertised;
none were invoked. No screenshots, recordings, native AX tests, raw CDP/
Playwright/AppleScript, browser settings/security changes or permission-dialog
automation occurred.

Read-only check_permissions(prompt=false) observed Accessibility granted and
Screen Recording granted, attributed to the driver daemon
com.trycua.driver, PID 33430. Direct-capture readiness was not probed.
list_sessions observed 0 visible sessions; get_session returned the expected
session_not_started state. health_report restricted to version/platform/
session/bundle/TCC checks reported overall ok, driver 0.30.4, darwin macOS
26.2 arm64 and com.trycua.driver. AX capability/capture checks were skipped.
These are provider prerequisite observations, not application accessibility
or browser evidence.

Started named lifecycle metadata session d01-bounded-confidential; returned
active true, capture_scope auto/effective_scope window, desktop_capture_authorized
false and desktop_unlocked false. This did not launch or bind an application.
The external capacity sample was then evaluated; no browser_prepare followed.
Cleanup end_session for that exact label returned active false. No browser,
window, profile or target was created and no unrelated session/process was
terminated.

The initial guide lookup used a shell glob which failed before lookup with
no matches; it was corrected using rg --files to locate the existing guide.
No fixture/product action occurred during that tooling correction. Some broad
description/read outputs were truncated; applicable tool descriptions and
platform/browser instruction portions were subsequently read in bounded
calls before any planned interaction. These tooling observations do not
change the capacity refusal or supply browser evidence.

### Fresh cleanup readback and immediate resource release

Cleanup readback command exit 0, UTC 2026-10-02T17:42:16.722017+00:00. It sampled
4446744576 bytes (4.141354 GiB) free.
This second sample was a cleanup readback, not a retry/start gate. Actual
readback JSON:

```json
{
  "browser_created": false,
  "cleanup_readback": true,
  "epoch": 1790962936.722017,
  "free_bytes": 4446744576,
  "lab_created": false,
  "new_outer_absent": true,
  "new_provider_absent": true,
  "ports": {
    "3000": {
      "absent": true,
      "exit": 1,
      "listener_count": 0
    },
    "9000": {
      "absent": true,
      "exit": 1,
      "listener_count": 0
    }
  },
  "server_helper_created": false
}
```

Exact lsof observations for ports 9000 and 3000 both exited 1, returned no
listener PID and confirmed absent. No owned server/helper/browser PID existed
to join. No private lab path had been allocated, so there was no lab to remove
or credential file to delete. Driver session cleanup completed. No unrelated
process/file was touched; no Cargo slot was acquired or released.

Then the explicit project orchestrator received immediate actual refusal and
DESKTOP/OPERATOR RELEASE with both capacity samples and resource readbacks;
that send exited 0 before this report append. The released fixture attempt
will not be resumed on a later sample. Shared-volume remediation belongs to
root, and a future attempt requires a separate explicit release after the
capacity prerequisite is met. No end-user approval or alternate provider
flow was introduced.

### Source/evidence pins and unexecuted scope

Root's published pin for the reviewed helper is main
b619fe25269ccc150e473bbcde47cdb3623ef810 / accepted 0a243c9 / tree a627df2.
Local history remained unchanged at 772c8cdbfd02e2bf3f28212884aebe5ef9f03849
until this report-only commit; no alignment/merge/main edit was performed.
The actual local helper matches immutable source
470690cad0cd93c9f25c5bc40b982e1b91679b49, blob
3d3379399126423a6a5588a6bd55ea4ac1723ebd, SHA-256
7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0,
32723 bytes / 695 lines.

Read local Essentials section 3 registration commands and source-identified
the published guide objects at b619fe:

| Published document | Bytes | SHA-256 |
| --- | ---: | --- |
| docs/essentials-guide.md | 52032 | 9df3c2861984751d48b28d284cfb47c07801fd3934072e2ccb829edeb35579a0 |
| docs/oidc-profiles.md | 25350 | 50e749a35ebed44e7430eb8400debbbce89d15444079913237a349b6fb21ca21 |

Guide identity checks are source/hash observations, not execution or full
independent body reviews of the published objects. No application setup step
was attempted. The expected c01 server7abf745, maintenance86490c7, baseclient
bfbbb322, verifierf6dd1aa and provider67a83dd/full-version pins remain
requirements for any future fixture; **none of those artifacts was freshly
rehashed/invoked in this phase**, because capacity failed first. Do not carry
historical artifact checks forward as new runtime evidence.

The original 78-case corrected memory pass remains dated and limited to
memory sinks; the earlier failed 61-of-76 memory run, actual failed fixtures,
provider refusal/lost values/reporting correction and UNKNOWN request sender
remain preserved. No source/controller/harness/guide/product/D05/test/Cargo
change or new worker/task/worktree was made. No real password sign-in,
Local demo consent, callback, protected 403/200, native verifier/JWKS/RS256,
userinfo, physical passkey, ordinary nonadmin, invitation, tenant or full
D01/D05 gate is credited. Root alone adjudicates future releases and original
acceptance; O06/I10/R05 remain closed.

Preservation checks before committing this append: the complete 185976-byte /
3034-line 772c8cd report remains a byte-exact prefix; helper470 SHA7fbc and the
retained original controller hash are unchanged; all five dated private
metadata files retain their bytes/hash/0600 modes. Fresh requested output
paths remain absent. Only this report changes; Markdown fences/final newline/
whitespace, git diff --check, source/prefix/readback identity and report-only
commit scope are checked without additional runtime.

## 2026-10-02: separately released capacity-ready confidential fixture — actual failure

Reservation: wave30_D01_bounded_confidential_browser_capacity_re_release,
project 891e7443-8dac-4c1b-897f-9e53cb59c7ee, D01 a96a1977, existing WT
7c85f5ef-3fac-4f72-aaed-08474d7fb454. This was one separately authorized fresh
fixture. The earlier 3f70cb1 capacity refusal remains refused/not started;
this observation does not revise it. Root alone owns original D01/D05 gates.

**Actual outcome: failed.** The one helper exited 1 with request_invalid,
first fixed request_invalid_reason authorization, and finite
preflow_authorization_refusals 4. The outer controller exited 1 with
failure_stage browser_checkpoint and failure_tag rp_nonzero. No sender,
raw Authorization value, request target or request time was recorded, so
the sender remains UNKNOWN. The bound browser's one navigation returned
net::ERR_CONNECTION_REFUSED. These are separate observed failures; the
browser error alone does not identify the refused requests' sender/cause.
No retry, extra RP request, correction or fallback followed.

### Fresh capacity and immutable inputs actually checked

Before setup, the read-only fresh sample at epoch 1790964132.870771 measured
14904586240 bytes free (about 13.88 GiB), exceeding the required 9126805504
bytes (8.5 GiB). The 8 GiB floor was 8589934592 bytes. Both requested new
evidence paths were absent and deployment-private had mode 0700. Tracked
and staged diffs were empty at local HEAD
3f70cb1f9cffb19ea3f4e537030083b1ebda0ab3. No unrelated process arguments,
environment, signals or files were inspected/changed. No Cargo slot was
acquired or released.

The controller freshly rehashed all of these before their permitted use:

| Input | Actual matching SHA-256 |
| --- | --- |
| c01 riauth server | 7abf745c10691a012a1918c83089168d0e0d88b42764dbf8ed538b90c828b606 |
| c01 riauth-maintenance | 86490c7f71de6b7ae9d4dabdf9060a8757100f3aedbdaa670284d9e1206a2a95 |
| c01 riauthctl | bfbbb322f1a66d0ac9998beb9fb5838097cea0442cea3fd3a1d057fcb3f600cf |
| Unchanged helper470 | 7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0 |
| Imported pinned verifier | f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d |
| Resolved OpenSSL provider | 67a83dd6d6d747d50c5d296dffb23e32bae9a2c588c93ae2d77e4c607b455c72 |

The three executables came from the previously authorized
riAuth-public-preview-local-module-boundaries-wave27/target/
d01-essentials-c01c39a/aarch64-apple-darwin/debug artifact directory, matching
source c01c39ab4e092423d5522bedc50fff87656d8c0a. No build was run.
Helper source remained immutable
470690cad0cd93c9f25c5bc40b982e1b91679b49, blob
3d3379399126423a6a5588a6bd55ea4ac1723ebd (32723 bytes, 695 lines).
The verifier was obtained from commit
9cefe7a56425bb73c17753e8766d92320b77da3b, blob
3be747d03146f1bcaa3ec012ee8d173b61fa737d. Published reviewed source pin
remains b619fe25269ccc150e473bbcde47cdb3623ef810 / accepted 0a243c9 /
tree a627df2; no alignment, merge or source edit was needed.

The guide identities already source-identified in the preceding phase remain
docs/essentials-guide.md SHA
9df3c2861984751d48b28d284cfb47c07801fd3934072e2ccb829edeb35579a0 and
docs/oidc-profiles.md SHA
50e749a35ebed44e7430eb8400debbbce89d15444079913237a349b6fb21ca21.
This attempt used section 2 administrator setup and section 3 confidential
Local demo registration, fixed localhost:9000 issuer and localhost:3000/callback.
Artifact identity checks do not constitute independent whole-body reviews
or a current full CI/release observation.

### Exact retained public controller and bounded setup

The complete public controller command below was retained and hashed before
execution: 13789 UTF-8 bytes, SHA-256
1194c3e111d877280d26c9b45235862854541f05d040711d2cc2905991a7dda1.
Its Python body has 179 lines, SHA-256
aa61d7408507b8d4ce29d9e7727058298c7e49e86be583ee5e4e258e76cb1da9.
Before execution, AST parsing and in-memory compilation of that body exited 0.
The retained original public controller remains SHA
ab0a9c5d8f8fb1c4f5fb5db8b3c396e4dfbdf56bb5a7f0f7e7178b485d77040e;
the updated payload changes only the fresh start timestamp, two fresh output
names, required helper hash and propagation of the existing fixed numeric
preflow refusal count. No RP readiness request or other request/controller
behavior was added. The executable archive contains generated-value source,
not any actual password, client secret, code, token, cookie or private URL.

The one controller used active 840 seconds plus cleanup 60 (inclusive 900),
helper 600, pending flow 180, HTTP/native operations 5, CLI 60 and readiness/
page budget 30 seconds. Its disk monitor required 8.5 GiB on one-second
samples. It created one fresh private lab and XDG directories with 0700
permissions, synthetic password via private stdin and 0600 private password/
CLI-secret files. Ports 9000 and 3000 were initially unused, then their socket
owners were proven to be exactly owned server PID 42135 and helper PID 42146.

The provider's one outer five-second version invocation retained complete
numeric exit/full ASCII stdout/stderr plus lengths and hash in an exclusive
0600 file before exact comparison. It returned exit 0, stdout 63 bytes and
stderr 0 bytes, no timeout, under only HOME/LANG/LC_ALL/PATH/TMPDIR.
Full stdout (the trailing newline is represented explicitly):

```text
OpenSSL 3.6.4 25 Aug 2026 (Library: OpenSSL 3.6.4 25 Aug 2026)\n
```

The helper's existing pinned provider/discovery/credential prerequisite
checks were true in its retained result; their existing calls/guards were
unchanged. No custom crypto or alternate verifier/provider was used.

| Printed operator setup command | Actual exit | Password prompts |
| --- | ---: | ---: |
| maintenance init | 0 | 2 |
| riauthctl login (administrator) | 0 | 1 |
| confidential Local demo client creation | 0 | 0 |
| discovery | 0 | 0 |
| whoami | 0 | 0 |

Those were operator prerequisites, not a browser user sign-in or application
approval. The original controller's bounded GET /readyz on port 9000 observed
200. There was no additional readiness HTTP request to the RP on port 3000.
The one helper main was invoked, and both listeners had exact owned PID
binding before fixture_ready was emitted.

Complete exact command archive:

```sh
python3 -u - <<'PY'
import datetime,fcntl,hashlib,json,os,pathlib,pty,secrets,select,shutil,signal,stat,subprocess,sys,tempfile,termios,time,urllib.request
ROOT=pathlib.Path.cwd()
START=1790964236.25
ACTIVE=START+840
BIN=pathlib.Path('/Users/dominik/orca/projects/riAuth-public-preview-local-module-boundaries-wave27/target/d01-essentials-c01c39a/aarch64-apple-darwin/debug')
PRIVATE=ROOT/'deployment-private'
OUT=PRIVATE/'d01-confidential-browser-bounded.redacted.json'
PROVIDER_META=PRIVATE/'d01-confidential-browser-bounded-provider-20261002.json'
record={'schema':'riauth.d01-confidential-browser-outer/v1','result':'failed','failure_stage':None,'failure_tag':None,'request_invalid_reason':None,'preflow_authorization_refusals':None,'commands':[],'artifacts':{},'cleanup':{},'owned_child_exits':[],'started_epoch':START,'active_seconds':840,'cleanup_seconds':60,'minimum_free_bytes':None,'disk_samples':0}
lab=None;children=[];names={};server=None;helper=None;password=None;stage='preflight';resultfd=None;lastsample=0
def require(ok,tag):
    if not ok:raise RuntimeError(tag)
def tick():
    global lastsample
    now=time.monotonic()
    if now-lastsample>=1:
        free=shutil.disk_usage(ROOT).free
        record['disk_samples']+=1
        record['minimum_free_bytes']=free if record['minimum_free_bytes'] is None else min(record['minimum_free_bytes'],free)
        lastsample=now
        require(free>=8.5*1024**3,'disk_margin')
    require(time.time()<ACTIVE,'active_deadline')
def write_exclusive(path,value):
    fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    raw=(json.dumps(value,sort_keys=True,indent=2)+'\n').encode('ascii')
    with os.fdopen(fd,'wb') as f:
        f.write(raw);f.flush();os.fsync(f.fileno())
    return hashlib.sha256(raw).hexdigest()
def listeners(port):
    p=subprocess.run(['/usr/sbin/lsof','-nP','-t','-iTCP:'+str(port),'-sTCP:LISTEN'],capture_output=True,timeout=3)
    require(p.returncode in (0,1),'socket_observation_failed')
    return set(int(v) for v in p.stdout.split())
def own(p,name):
    children.append(p);names[p.pid]=name;return p
def stop_child(p):
    if p.poll() is None:
        for sig,wait in [(signal.SIGINT,8),(signal.SIGTERM,5),(signal.SIGKILL,2)]:
            if p.poll() is not None:break
            p.send_signal(sig)
            try:p.wait(timeout=wait)
            except subprocess.TimeoutExpired:pass
    require(p.poll() is not None,'owned_child_reap_failed')
def controlling_tty():
    os.setsid();fcntl.ioctl(0,termios.TIOCSCTTY,0)
def cli(name,args,prompts=0):
    tick()
    master,slave=pty.openpty()
    p=own(subprocess.Popen(args,cwd=lab,env=environment,stdin=slave,stdout=slave,stderr=slave,preexec_fn=controlling_tty),name)
    os.close(slave);seen=0;raw=b'';started=time.monotonic()
    try:
        while p.poll() is None:
            tick();require(time.monotonic()-started<60,'cli_deadline')
            if select.select([master],[],[],0.2)[0]:
                try:chunk=os.read(master,4096)
                except OSError:chunk=b''
                raw+=chunk;require(len(raw)<=131072,'cli_output_limit')
                for prompt in ([b'Password: ',b'Confirm password: '] if prompts==2 else [b'Password: ']):
                    if seen<prompts and prompt in raw:
                        require(prompt==([b'Password: ',b'Confirm password: '][seen] if prompts==2 else b'Password: '),'cli_prompt_order')
                        os.write(master,password.encode()+b'\n');seen+=1;raw=b''
        while select.select([master],[],[],0)[0]:
            try:
                chunk=os.read(master,4096)
                if not chunk:break
                raw+=chunk
            except OSError:break
        code=p.wait()
        record['commands'].append({'name':name,'exit':code,'password_prompts':seen})
        require(code==0,'cli_nonzero');require(seen==prompts,'cli_prompt_missing')
    finally:
        raw=b'';os.close(master)
        if p.poll() is None:stop_child(p)
def wait_listener(p,port):
    deadline=time.monotonic()+30
    while time.monotonic()<deadline:
        tick();require(p.poll() is None,'owned_service_early_exit')
        owners=listeners(port)
        if owners:
            require(owners=={p.pid},'listener_owner_mismatch');return
        time.sleep(0.2)
    raise RuntimeError('listener_deadline')
try:
    tick()
    require(PRIVATE.is_dir() and not PRIVATE.is_symlink() and stat.S_IMODE(PRIVATE.stat().st_mode)==0o700,'private_directory_invalid')
    require(not OUT.exists() and not PROVIDER_META.exists(),'evidence_already_exists')
    require(not listeners(9000) and not listeners(3000),'port_occupied');record['ports_preflight_empty']=True
    pins={'riauth':'7abf745c10691a012a1918c83089168d0e0d88b42764dbf8ed538b90c828b606','riauth-maintenance':'86490c7f71de6b7ae9d4dabdf9060a8757100f3aedbdaa670284d9e1206a2a95','riauthctl':'bfbbb322f1a66d0ac9998beb9fb5838097cea0442cea3fd3a1d057fcb3f600cf'}
    for name,pin in pins.items():
        tick()
        with (BIN/name).open('rb') as f:digest=hashlib.file_digest(f,'sha256').hexdigest()
        record['artifacts'][name]=digest;require(digest==pin,'artifact_hash_mismatch')
    helper_path=ROOT/'scripts/d01-confidential-browser-demo.py'
    require(hashlib.sha256(helper_path.read_bytes()).hexdigest()=='7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0','helper_hash_mismatch')
    verifier=subprocess.check_output(['git','show','9cefe7a56425bb73c17753e8766d92320b77da3b:scripts/recovery-drill-oidc.py'],timeout=5)
    require(hashlib.sha256(verifier).hexdigest()=='f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d','verifier_hash_mismatch')
    provider=pathlib.Path('/opt/homebrew/bin/openssl').resolve(strict=True)
    with provider.open('rb') as f:provider_hash=hashlib.file_digest(f,'sha256').hexdigest()
    require(provider_hash=='67a83dd6d6d747d50c5d296dffb23e32bae9a2c588c93ae2d77e4c607b455c72','provider_hash_mismatch')
    provider_env={k:v for k,v in os.environ.items() if k in {'PATH','HOME','TMPDIR','LANG','LC_ALL'}}
    p=subprocess.Popen([str(provider),'version'],stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=provider_env)
    try:stdout,stderr=p.communicate(timeout=5);timeout=False
    except subprocess.TimeoutExpired:
        p.kill();stdout,stderr=p.communicate(timeout=2);timeout=True
    provider_record={'sha256':provider_hash,'exit':p.returncode,'timeout':timeout,'stdout_ascii':stdout[:4096].decode('ascii',errors='backslashreplace'),'stderr_ascii':stderr[:4096].decode('ascii',errors='backslashreplace'),'stdout_bytes':len(stdout),'stderr_bytes':len(stderr),'environment_keys':sorted(provider_env)}
    record['provider_metadata_sha256']=write_exclusive(PROVIDER_META,provider_record)
    require(not timeout and p.returncode==0 and len(stdout)<=256 and len(stderr)<=4096 and stdout.decode('ascii').strip()=='OpenSSL 3.6.4 25 Aug 2026 (Library: OpenSSL 3.6.4 25 Aug 2026)','provider_version_failed')
    tick()
    resultfd=os.open(OUT,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    lab=pathlib.Path(tempfile.mkdtemp(prefix='d01-confidential-browser-diagnostic.',dir=PRIVATE));os.chmod(lab,0o700)
    for name in ('xdg','deployment-private','rp'):(lab/name).mkdir(mode=0o700)
    (lab/'recovery-drill-oidc.py').write_bytes(verifier);os.chmod(lab/'recovery-drill-oidc.py',0o600);verifier=None
    password=secrets.token_urlsafe(30)
    fd=os.open(lab/'browser-password',os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
    with os.fdopen(fd,'w') as f:f.write(password)
    environment=dict(provider_env);environment['XDG_CONFIG_HOME']=str(lab/'xdg')
    stage='init';cli('maintenance_init',[str(BIN/'riauth-maintenance'),'--config',str(lab/'riauth.toml'),'init','--issuer','http://localhost:9000','--listen','127.0.0.1:9000','--data-dir','data','--admin','admin'],2)
    stage='serve'
    server=own(subprocess.Popen([str(BIN/'riauth'),'--config',str(lab/'riauth.toml'),'serve'],cwd=lab,env=environment,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL),'server')
    record['server_pid']=server.pid;wait_listener(server,9000);record['server_listener_owned']=True
    stage='readyz'
    with urllib.request.urlopen('http://127.0.0.1:9000/readyz',timeout=5) as response:
        record['readyz_status']=response.status;response.read(4096)
    require(record['readyz_status']==200,'readyz_failed')
    stage='cli_login'
    base=[str(BIN/'riauthctl'),'--server','http://localhost:9000']
    cli('operator_login',base+['login','admin'],1)
    stage='client_create';secretpath=lab/'deployment-private/local-demo-secret.json'
    cli('confidential_client_create',base+['client','create','local-demo','--name','Local demo','--confidential','--redirect-uri','http://localhost:3000/callback','--scope','openid,profile','--secret-file',str(secretpath)])
    require(stat.S_IMODE(secretpath.stat().st_mode)==0o600,'cli_secret_mode_invalid')
    stage='discovery';cli('discovery',base+['discovery'])
    stage='whoami';cli('whoami',base+['whoami'])
    stage='rp_start'
    helper=own(subprocess.Popen([sys.executable,'-B',str(helper_path),'--workspace',str(lab/'rp'),'--secret-file',str(secretpath),'--verifier-helper',str(lab/'recovery-drill-oidc.py'),'--openssl','/opt/homebrew/bin/openssl','--deadline-seconds','600','--evidence',str(lab/'rp-result.json')],cwd=ROOT,env=environment,stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,text=True),'helper')
    record['helper_pid']=helper.pid;record['helper_invocations']=1
    wait_listener(helper,3000);record['helper_listener_owned']=True
    print(json.dumps({'fixture_ready':True,'guard_pid':os.getpid(),'server_pid':server.pid,'helper_pid':helper.pid,'lab':str(lab),'commands':record['commands'],'provider_metadata_sha256':record['provider_metadata_sha256'],'minimum_free_bytes':record['minimum_free_bytes']}),flush=True)
    stage='browser_checkpoint';announced=False
    while not (lab/'stop').exists():
        tick();require(server.poll() is None,'idp_early_exit')
        if helper.poll() is not None:
            code=helper.wait()
            if not announced:
                record['helper_exit']=code;record['helper_result']=json.loads((lab/'rp-result.json').read_bytes())
                record['request_invalid_reason']=record['helper_result']['request_invalid_reason']
                record['preflow_authorization_refusals']=record['helper_result']['preflow_authorization_refusals']
                print(json.dumps({'helper_completed':True,'exit':code,'result':record['helper_result']['result'],'failure_tag':record['helper_result']['failure_tag'],'request_invalid_reason':record['request_invalid_reason'],'preflow_authorization_refusals':record['preflow_authorization_refusals']}),flush=True);announced=True
            require(code==0,'rp_nonzero')
        time.sleep(0.2)
    if (lab/'ui-failure').exists():raise RuntimeError('browser_checkpoint_failed')
    require(helper.poll()==0,'rp_checkpoint_incomplete');record['result']='passed'
except Exception as error:
    record['failure_stage']=stage
    tags={'disk_margin','active_deadline','private_directory_invalid','evidence_already_exists','port_occupied','socket_observation_failed','artifact_hash_mismatch','helper_hash_mismatch','verifier_hash_mismatch','provider_hash_mismatch','provider_version_failed','cli_deadline','cli_output_limit','cli_prompt_order','cli_prompt_missing','cli_nonzero','owned_service_early_exit','listener_owner_mismatch','listener_deadline','readyz_failed','cli_secret_mode_invalid','idp_early_exit','rp_nonzero','browser_checkpoint_failed','rp_checkpoint_incomplete'}
    record['failure_tag']=str(error) if str(error) in tags else 'outer_unexpected_failure'
finally:
    for p in reversed(children):
        try:stop_child(p)
        except Exception:record['cleanup']['child_reap_failure']=True
        record['owned_child_exits'].append({'name':names[p.pid],'pid':p.pid,'exit':p.poll()})
    record['cleanup']['owned_children_reaped']=all(p.poll() is not None for p in children)
    if lab is not None:
        if helper is not None:
            record['helper_exit']=helper.poll()
            if (lab/'rp-result.json').exists():
                record['helper_result']=json.loads((lab/'rp-result.json').read_bytes())
                record['request_invalid_reason']=record['helper_result']['request_invalid_reason']
                record['preflow_authorization_refusals']=record['helper_result']['preflow_authorization_refusals']
        password=None;shutil.rmtree(lab);record['cleanup']['lab_removed']=not lab.exists()
    else:record['cleanup']['lab_removed']=True
    try:
        record['cleanup']['port9000_absent']=not listeners(9000);record['cleanup']['port3000_absent']=not listeners(3000)
    except Exception:record['cleanup']['socket_observation_failure']=True
    record['completed_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();record['elapsed_seconds']=round(time.time()-START,3)
    if resultfd is None and not OUT.exists():resultfd=os.open(OUT,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    if resultfd is not None:
        with os.fdopen(resultfd,'wb') as f:
            f.write((json.dumps(record,sort_keys=True,indent=2)+'\n').encode('ascii'));f.flush();os.fsync(f.fileno())
    print(json.dumps({'fixture_finished':True,'result':record['result'],'failure_stage':record['failure_stage'],'failure_tag':record['failure_tag'],'request_invalid_reason':record['request_invalid_reason'],'preflow_authorization_refusals':record['preflow_authorization_refusals'],'cleanup':record['cleanup'],'owned_child_exits':record['owned_child_exits'],'minimum_free_bytes':record['minimum_free_bytes'],'disk_samples':record['disk_samples'],'elapsed_seconds':record['elapsed_seconds']}),flush=True)
sys.exit(0 if record['result']=='passed' else 1)
PY
```

### RiWork Cua.ai Driver-only browser observations

Read the installed cua-driver SKILL.md, MACOS.md and BROWSER.md and applicable
advertised MCP descriptions/current state before GUI actions. Provider stayed
RiWork Cua.ai Driver MCP; driver 0.30.4 was healthy on macOS 26.2 arm64, with
Accessibility and Screen Recording granted and no visible preexisting active
sessions. Direct capture was not probed. This was prerequisite checking, not
application accessibility testing.

Named session d01-bounded-capacity used browser_prepare with allow_launch true
and profile mode isolated_new. It returned prepared true, action
launched_isolated_browser, exact driver-owned PID 44123 and ownership method
spawned_by_driver. Side effects reported new profile/browser creation,
no profile data copying, no preference changes, no modification/termination
of a personal browser, no foregrounding and no consent prompt.

A fresh list_windows for that exact PID identified the normal about:blank
window 103317. get_browser_state with exact PID/window/session and screenshots
disabled returned binding_quality exact, mutation_allowed true,
endpoint_access_class driver_owned, one selected tab, target
bt-4446edb7-e136-446d-9059-585c63beb1d3 and tab
tab-f2630790-5fe0-4dec-84c9-d48e46696b36. These exact returned handles were
used for the sole browser navigation to the public protected path.

browser_navigate returned tool_invocation_failed with
navigation failed: net::ERR_CONNECTION_REFUSED. One fresh semantic readback,
screenshots disabled, showed the browser error page heading “This site can't
be reached”, localhost refused to connect and ERR_CONNECTION_REFUSED.
No Reload action, retry, alternate tab/provider, HTTP sign-in or service/
terminal approval substitute followed. No secret was typed or displayed.
There was no sign-in, empty-factor password submission, consent, callback,
protected 403/200 or authenticated page observation.

### Actual helper/outer results and private evidence identities

Polling/joining the owned controller returned exit 1 and the fixed helper
completion record:

```json
{"helper_completed":true,"exit":1,"result":"failed","failure_tag":"request_invalid","request_invalid_reason":"authorization","preflow_authorization_refusals":4}
```

The existing first-three preflow refusal allowance did not produce any
journey credit. The fourth refusal was terminal, count 4; no counter reset,
fifth allowance or guard/Host/Authorization tolerance was introduced.
The fixed first authorization reason was preserved. Sender/origin and raw
values remain unavailable; neither the Driver nor an external probe is
asserted to be the sender. No request timing attribution is inferred.

| Helper check | Retained actual boolean |
| --- | --- |
| credential_private_validated | true |
| discovery_verified | true |
| provider_identity_verified | true |
| protected_without_cookie_denied | false |
| authorization_redirect_issued | false |
| state_issuer_flow_cookie_verified | false |
| confidential_s256_exchange_verified | false |
| rs256_jwks_issuer_audience_nonce_time_access_hash_verified | false |
| userinfo_subject_verified | false |
| protected_with_fresh_cookie_accepted | false |

All six helper HTTP result fields were null: protected_before,
authorization_redirect, callback, token_exchange, userinfo, protected_after.
A false/null check is an unmet checkpoint, not an observed successful denial
or cryptographic failure. Helper elapsed 48.5 seconds, disk_samples 257,
minimum_free_bytes 12599562240. Outer elapsed 52.323 seconds, disk_samples 50,
minimum_free_bytes 12617760768; completed UTC
2026-10-02T18:04:48.572651+00:00. All sampled minima exceeded 8.5 GiB.
No capacity failure or timeout was reported in this attempt.

| New exclusive private evidence | Bytes | Mode | SHA-256 |
| --- | ---: | --- | --- |
| deployment-private/d01-confidential-browser-bounded.redacted.json | 4125 | 0600 | be9831b0c125ab81a7af4854156fb90a468c065d8581088f5e127f2ddf663342 |
| deployment-private/d01-confidential-browser-bounded-provider-20261002.json | 359 | 0600 | 6fad035f1897b5287b2ebd812405fcabf6a33b72d2d4f498f3bca7081eb719ab |

Both files were created fresh, never overwrote old metadata, and remain
deployment-private rather than committed. Identical provider metadata hashes
across dated files do not imply reuse; this new invocation produced its own
exclusive file. Complete fixed outer/helper results are retained privately;
no raw credentials, headers, codes, cookies, tokens, state, nonce, verifier,
subject or query URL appear in this evidence.

### Owned cleanup, numeric exits and immediate resource release

The outer finally completed owned-child joins, helper connection/listener/
private-reference/verifier-temporary cleanup, own-lab removal and listener
absence on ports 9000/3000. Numeric child exits:

| Owned child | PID | Exit |
| --- | ---: | ---: |
| helper | 42146 | 1 |
| whoami | 42145 | 0 |
| discovery | 42144 | 0 |
| confidential_client_create | 42143 | 0 |
| operator_login | 42140 | 0 |
| server | 42135 | 0 |
| maintenance_init | 42110 | 0 |

Only the proven driver-owned browser PID 44123 was targeted. Cooperative
background cmd+q returned effect unverifiable / delivery_failed; fresh
list_windows still found its window. Exact-owned Driver kill_app then sent
SIGKILL to PID 44123. end_session for d01-bounded-capacity returned active
false. A fresh Driver list_windows for that exact PID returned zero windows.
No unrelated browser was closed. UI logout was unavailable because no
application session had been established.

Fresh cleanup readback at epoch 1790964520.564815 confirmed own controller
PID 42074, server 42135, helper 42146 and browser 44123 all absent. lsof on
each exact port 9000/3000 exited 1 with empty stdout/stderr and no listener.
The exact own lab was absent. Fresh free capacity was 12568104960 bytes.
This readback was about 284.315 seconds after the retained start timestamp,
within the inclusive 900-second budget. No extra RP HTTP call was used to
verify cleanup.

After joined exit and those fresh readbacks, the explicit project
orchestrator send exited 0 and immediately handed root the actual failure,
reason/count, numeric exits, disk minima, private evidence hashes and
DESKTOP/OPERATOR RELEASE **before this report append**. No Cargo acquisition/
release occurred. The attempt is closed; no automatic continuation is allowed.

### Preserved history, actual review scope and remaining limits

The complete prior 194701-byte / 3208-line report remains a byte-exact prefix,
SHA-256 905b74a58764778720136383436c36addd84485c5f0a2d9a7cead27df384f3f1,
blob 2d0e7158e6da27ffdfccd108316b49eea9f86689 at 3f70cb1.
The five historical private 0600 metadata files were rechecked for exact
bytes/hash/mode and all remain unchanged. The initial lost provider output/
wrong reporting correction, earlier actual request refusals with UNKNOWN
sender, failed 61-of-76 memory run, dated corrected 78-case memory pass and
capacity refusal are preserved verbatim. The new fixture does not revise
any earlier outcome or make memory-sink evidence into a real journey.

This phase directly observed the five local operator prerequisite commands,
artifact/provider checks, one helper failure and one isolated browser error,
plus owned cleanup. It did not observe application password authentication,
consent, callback, code exchange, RS256/JWKS/nonce/audience/time validation,
userinfo or protected access; no confidential browser checkpoint pass is
claimed. Physical passkeys, hardware, tenant/application installation,
invitation/LDAP/SCIM/ordinary-nonadmin journeys and full D01/D05 acceptance
remain outside this bounded evidence. Independent reviewer/browser-worker
dual role remains explicit; this is authored actual evidence of my own action,
not independent verification of it. O06/I10/R05 remain DONE.

Only this report is appended. No helper/controller behavior, product, guide,
D05 artifact, config, test, source, worker/task/worktree, main, push or status
change is made. Source identity and hash checks are distinct from full body
reviews. Remaining request origin/cause requires root adjudication; no new
observation, source correction or runtime is proposed/executed here.

Report-only validation checks exact prefix, unchanged immutable helper470,
historical private metadata, archive payload/body hashes, new private metadata
hashes/modes, Markdown fences/final newline/whitespace and git diff --check.
Commit scope and clean tracked/staged handoff are read back without runtime.

Report preparation first encountered an unavailable TextEncoder in the tool's
JavaScript isolate before any file write. Serialization was corrected without
changing the retained controller, helper, runtime or observations.

## 2026-10-02: prepared-browser controller ordering — source-first design only

Reservation wave30_D01_prepared_browser_controller_design, project
891e7443-8dac-4c1b-897f-9e53cb59c7ee, existing WT
7c85f5ef-3fac-4f72-aaed-08474d7fb454. Only this report is appended.
This phase invokes no Driver/provider/helper/CLI/server/listener/browser/
HTTP/native/Cargo runtime and acquires/releases no resource slot.
DESKTOP remains FREE and HELD. Root owns source/runtime reservation and
D01/D05 disposition; O06/I10/R05 stay closed.

**Concrete prospective seam:** after the printed CLI prerequisites and
immediately before the existing helper launch, pause at a bounded private
browser-preparation gate. Prepare exactly one Driver-owned isolated browser
and bind its exact returned blank window/target/tab before releasing that
gate. Then launch the unchanged helper, prove the exact owned listener as
before and dispatch the existing first protected-page navigation through
that prepared target. This moves browser launch/window discovery/binding
out of the helper's listening interval. It does not attribute unknown traffic,
change HTTP acceptance, increase/reset the refusal counter or guarantee
success against later unexpected requests.

### Exact reviewed inputs and own chronology actually available

Read the entire retained 13789-byte public controller from my own prior
execution record and its byte-identical archive in immutable report 500b2b6.
Command SHA-256:
1194c3e111d877280d26c9b45235862854541f05d040711d2cc2905991a7dda1.
The 179-line Python body SHA-256 is
aa61d7408507b8d4ce29d9e7727058298c7e49e86be583ee5e4e258e76cb1da9.

Read only the retained results of my own session d01-bounded-capacity:
browser_prepare, list_windows, get_browser_state binding, the error-page
semantic readback, cooperative close/readback, exact-owned kill_app,
end_session and final window absence, plus fixture-ready and controller
completion records. No other session history or process arguments/environment
were inspected. No historical Driver tool was invoked in this design phase.

| Observed own-source/result order | Available fixed evidence |
| --- | --- |
| Controller performs printed setup and operator login/client/discovery/whoami | Five commands exited 0; original source order unchanged |
| Controller launches helper and proves exact owned port 3000 | Source command lines 133–137; helper PID 42146; listener-owned true; fixture_ready emitted |
| Driver prepares isolated browser | launched_isolated_browser; exact owned PID 44123 |
| Driver lists that PID's windows | about:blank window 103317 selected |
| Driver binds exact PID/window/session | exact binding, driver_owned endpoint, target bt-4446edb7-e136-446d-9059-585c63beb1d3; tab tab-f2630790-5fe0-4dec-84c9-d48e46696b36 |
| Sole first protected browser navigation | net::ERR_CONNECTION_REFUSED |
| Fresh semantic readback | localhost connection-refused browser error page |
| Next owned-controller poll/join | Helper exit 1, request_invalid/authorization/count 4; outer exit 1, rp_nonzero |
| Owned cleanup readbacks | Children reaped, ports/lab/PIDs absent; browser killed only after cooperative close failed; session ended |

No retained Driver action result has an action-start/completion/elapsed/
monotonic timestamp field. No independently timed browser_prepare duration,
listener-bind instant, navigation dispatch instant or refused-request instant
was retained. The available numeric timing is the controller's start epoch
1790964236.25, completion UTC 2026-10-02T18:04:48.572651+00:00 and elapsed
52.323 seconds; helper elapsed 48.5 seconds; cleanup readback epoch
1790964520.564815. Those totals cannot be converted into a measured GUI
preparation delay or request ordering within the listening interval.

The source and call sequence do support the limited conclusion that browser
launch/discovery/binding happened after fixture_ready and therefore while
the helper had already been launched and its listener proven. Moving those
steps before helper creation removes that source-ordered preparation work
from the listening interval. The actual duration removed is UNKNOWN.
The failed navigation does not establish who sent the four refused requests.
No new request, sender inference, external-probe assertion or retrospective
timing attribution is made.

### One exact prospective controller delta

The candidate below is **in memory only**, not materialized as a controller/
helper/source file, imported or executed. Reconstruct it from the exact 500b
controller archive and this complete zero-context diff. It intentionally
inherits the old historical START and already-used output names so only the
ordering delta is reviewed here. It must not be executed: any future root
reservation must separately pin a fresh start and unused exclusive result/
provider paths, as the existing evidence-already-exists guard requires.

Candidate complete command: 14899 UTF-8 bytes, SHA-256
35d1ad33420b2fccbaab61001f34a300ad1002a4347d500776ccb621dfaceab8.
Candidate body: 14876 bytes, 193 lines, SHA-256
89a0987ea53451889e6e93bd369e4cdb7a838d8795e91b8e23798748554d1176.
There are two hunks: a 14-line gate comprising 11 AST statements, and addition
of two finite failure labels. No other old controller line changes.

```diff
--- retained-500b-controller
+++ PROSPECTIVE-prepared-browser-controller
@@ -132,0 +133,14 @@
+    stage='browser_prepare'
+    prepare_deadline=time.monotonic()+30
+    prepare_marker=lab/'browser-prepared'
+    print(json.dumps({'browser_prepare_required':True,'guard_pid':os.getpid(),'server_pid':server.pid,'lab':str(lab)}),flush=True)
+    while not prepare_marker.exists():
+        tick();require(server.poll() is None,'idp_early_exit')
+        require(not (lab/'ui-failure').exists() and not (lab/'stop').exists(),'browser_checkpoint_failed')
+        require(time.monotonic()<prepare_deadline,'browser_prepare_deadline')
+        time.sleep(0.2)
+    tick();require(server.poll() is None,'idp_early_exit')
+    require(not (lab/'ui-failure').exists() and not (lab/'stop').exists(),'browser_checkpoint_failed')
+    require(time.monotonic()<prepare_deadline,'browser_prepare_deadline')
+    prepare_info=prepare_marker.lstat()
+    require(stat.S_ISREG(prepare_info.st_mode) and stat.S_IMODE(prepare_info.st_mode)==0o600 and prepare_info.st_uid==os.getuid() and prepare_info.st_nlink==1 and prepare_info.st_size==0,'browser_prepare_marker_invalid')
@@ -154 +168 @@
-    tags={'disk_margin','active_deadline','private_directory_invalid','evidence_already_exists','port_occupied','socket_observation_failed','artifact_hash_mismatch','helper_hash_mismatch','verifier_hash_mismatch','provider_hash_mismatch','provider_version_failed','cli_deadline','cli_output_limit','cli_prompt_order','cli_prompt_missing','cli_nonzero','owned_service_early_exit','listener_owner_mismatch','listener_deadline','readyz_failed','cli_secret_mode_invalid','idp_early_exit','rp_nonzero','browser_checkpoint_failed','rp_checkpoint_incomplete'}
+    tags={'disk_margin','active_deadline','private_directory_invalid','evidence_already_exists','port_occupied','socket_observation_failed','artifact_hash_mismatch','helper_hash_mismatch','verifier_hash_mismatch','provider_hash_mismatch','provider_version_failed','cli_deadline','cli_output_limit','cli_prompt_order','cli_prompt_missing','cli_nonzero','owned_service_early_exit','listener_owner_mismatch','listener_deadline','readyz_failed','cli_secret_mode_invalid','idp_early_exit','rp_nonzero','browser_checkpoint_failed','rp_checkpoint_incomplete','browser_prepare_deadline','browser_prepare_marker_invalid'}
```

The preparation deadline is 30 seconds total from entering this gate, including
Driver preparation and exact blank-target binding. It does not restart the
outer clock or extend any deadline. Existing tick() samples disk once per
second, stops below 8.5 GiB, and retains the original active START+840 cutoff.
The gate checks that the exact owned server is still live, stops on the
existing private failure/stop signals, and sleeps only 0.2 seconds between
checks. A missed 30-second preparation bound produces the fixed
browser_prepare_deadline failure; no helper has been invoked in that branch.

The marker is only a local readiness interlock in the already-owned 0700 lab.
It must be a zero-byte regular file, owned by the current UID, mode 0600,
single link; symlink/nonempty/wrong mode/owner/link-count is refused with the
fixed browser_prepare_marker_invalid label. It carries no credentials,
browser endpoint, cookie, target, URL or sender data. The driver binding is
verified by the outer worker from the actual returned Driver result before
creating this marker; the file itself is not evidence of a successful
application journey or proof of a live browser.

Exact prospective marker writer, invoked only after future root release,
successful exact Driver binding and a fresh check that this own controller
is still running/waiting:

```python
# Prospective outer-controller marker writer; NOT executed in this phase.
# OWN_LAB is only the exact path returned by this same owned controller.
import os
from pathlib import Path
marker=Path(OWN_LAB)/'browser-prepared'
fd=os.open(marker,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
os.close(fd)
```

This writer uses exclusive creation and no-follow, contains no request,
does not read credentials, and does not reset/rewrite an existing marker.
Failure means stop/cleanup; it is not retried. OWN_LAB is a symbolic task input
here, not a hardcoded prior lab or path copied from another fixture.

### Exact prospective Driver ordering and stop/cleanup contract

The one future outer workflow would retain the existing operator setup and
append the following sequence at the new event, rather than at fixture_ready:

1. Before any setup, fresh capacity at least 8.5 GiB and exclusive output/
   provider-path absence remain mandatory. Establish the one inclusive
   900-second fixture clock (840 active, 60 cleanup); the clock includes
   preparation and is never restarted. Read Driver skills/descriptions/current
   state/permissions only in a separately released future runtime.
2. Start the reviewed controller once. Let it complete its existing pinned
   artifact/provider/version retention, private lab/XDG/password and printed
   CLI prerequisites. Receive browser_prepare_required from the exact own
   controller. Port 3000/helper creation has not occurred at that event.
3. With the one named future session, call Driver browser_prepare using
   allow_launch true and profile mode isolated_new. Retain the exact
   spawned_by_driver PID and ownership proof. Call list_windows for that PID,
   then get_browser_state on its exact blank window with screenshots disabled.
   Require exact binding, mutation_allowed true, driver_owned endpoint and
   the exact returned selected target/tab. No localhost navigation, RP probe,
   personal profile copying or profile/settings change occurs during this step.
4. Read the owned controller's available output/liveness without sending HTTP.
   On any failure/deadline, stop immediately. Only after successful binding
   and a still-live waiting controller, write the single zero-byte private
   marker with the exact prospective writer above.
5. The same controller now runs its existing one helper Popen, same helper
   arguments/deadline 600, same fixed localhost:3000 callback, and same exact
   wait_listener(helper,3000) ownership proof with its existing 30-second
   bound. It emits the existing fixture_ready record.
6. Retrieve that readiness/output promptly and stop if the controller/helper
   has already failed. Dispatch the existing first public protected-page
   navigation through the stored exact returned Driver target/tab. Do not
   repeat browser_prepare, window discovery or binding between listener
   readiness and this first navigation. Any stale/closed/binding failure
   ends the attempt; no rebind/retry or alternate target/provider is allowed.
7. Only if the actual protected denial is observed, continue the unchanged
   planned password/empty-factor sign-in, Local demo consent, exact callback,
   native validation/userinfo and fresh protected access. Every semantic
   outcome is verified from fresh state. No rejected traffic is journey credit;
   no raw HTTP sign-in/session/terminal approval substitute is allowed.

Readiness/liveness checks cannot make the last step atomic with a network
request. The helper may still stop between the latest controller observation
and browser navigation. Unexpected Authorization traffic can still exhaust
the unchanged count 4 at any time. This proposal removes preparation work
from the live interval, not those failure cases.

On a Driver preparation/binding failure, the outer worker writes the existing
private ui-failure/stop signals only to its exact owned lab if still present,
joins the existing controller, and preserves the first fixed failure. The
new gate checks those signals; no helper is started on that failure branch.
If the preparation deadline expires first, the controller's unchanged finally
reaps the existing own server/CLI children, removes only its own lab, checks
ports and retains its fixed result. A subsequent late marker write fails
rather than recreating the removed lab. No retry follows.

The outer finally also ends its exact named Driver session and closes only a
proven driver-owned PID: bounded cooperative close/readback first, then exact
owned Driver kill_app if cooperative close fails. If a failed prepare call
does not return sufficient browser ownership evidence, no guessed PID is
killed; retain that actual cleanup prerequisite failure for root. Known owned
server/helper children use the original bounded join/escalation sequence.
Fresh exact owned PID, port 9000/3000, lab and browser/session absence readbacks
precede immediate release, all within the retained cleanup budget. No unrelated
process/profile/service is touched. This is a future cleanup contract, not
an actual cleanup action in this design phase.

### Timing choice and preservation proof

No helper timestamps or new timing source are proposed in this smallest seam.
Numeric listener/first-navigation gap measurement is not required to prove
the preparation ordering: helper Popen is source-ordered after the validated
marker, and that marker is created only after the exact Driver preparation/
binding result. The retained original results lack the clock inputs needed
to quantify the old gap; adding timing now would not recover them. Any future
root request for relative listener/navigation timing requires its own exact
controller observation reservation. This design does not fabricate old times
or add an unrelated timestamp-only observation.

Static-only review actually performed: AST parse and in-memory compile of the
candidate code object, no execution/import/main; complete byte reversal after
removing the 14-line gate and only the two new literal labels; complete AST
reversal; all nine original controller function ASTs equal; original try-body
prefix before the gate and suffix after it equal; original finally AST equal.
The exact command/body hashes above were computed from those in-memory bytes.
The future marker writer was parsed/compiled only, not run.

All original controller request builders, provider environment/hash/full
version retention-before-comparison, CLI printed registration, owned socket
checks, helper Popen arguments, readiness request, helper-result handling and
cleanup remain byte-for-byte after that reversible insertion. No RP readiness
HTTP request, header construction, authorization request or new dispatch route
is added. Existing helper470 SHA7fbc remains byte-exact, so all parser/Host/Auth/
body-target/cookie/crypto/native/post-flow/accepted-route guards, first-three
stateless403/fourth-terminal403 limit and post-flow terminal400 behavior remain
unchanged. No counter reset, extra allowance or change to an accepted/refused
HTTP set is proposed.

Future validation, only after separate root source/runtime ownership: review
the complete reconstructed candidate and marker writer; check old-source
reversal and helper/artifact pins; confirm preparation failure/deadline never
creates a helper; confirm marker false/malformed/duplicate handling fails
closed; verify one prepared exact target precedes one helper invocation;
confirm the inclusive/disk/deadline bounds and finally cleanup still hold;
then, only if separately released, one fresh actual browser checkpoint under
the existing acceptance protocol. This design supplies no lifecycle test,
browser/crypto/journey pass or D01/D05 gate credit.

### Immutable history and report-only handoff

At phase entry, report HEAD was
500b2b67ac5630a9feb10d21be6d97ab997ef6f0, blob
a0e98d9fda347b76d52396a422397e39b3d706a5, 223611 bytes / 3668 lines,
SHA-256 a092ed00d348952a960e3d8ec83b812c4ca8f6b00d37d29ba7ba0552f461c935.
That complete content remains a byte-exact prefix. Rechecked all five old plus
two new private metadata files for their pinned bytes/SHA/mode 0600; unchanged.
The actual bounded result remains 4125 bytes / SHA
be9831b0c125ab81a7af4854156fb90a468c065d8581088f5e127f2ddf663342 and its
distinct provider result 359 bytes / SHA
6fad035f1897b5287b2ebd812405fcabf6a33b72d2d4f498f3bca7081eb719ab.
No raw private request/header/target/sender/credential data was read or logged.

The original controller archive, helper470, provider refusal/lost-output and
reporting correction, earlier failures, UNKNOWN sender, failed 61/76 and dated
78-case memory pass, capacity refusal, actual fourth-refusal/browser error
and all cleanup evidence are preserved. This is the same author's prospective
source analysis of their own failed attempt, not independent verification of
that action. Historical hashes are identities, not current runtime/body reviews.

Only this report changes. Source/controller/helper files are not materialized
or edited; no guide/product/D05/test/build/Cargo/service/desktop resource,
worker/task/worktree, main/push/status/merge or contact with another worker.
Root receives one immutable report-only commit with exact diff/hashes/static
proofs and retains all implementation, future release and gate decisions.

## 2026-10-02: separately released prepared-browser checkpoint — actual gate timeout

Reservation wave30_D01_prepared_browser_single_checkpoint, project
891e7443-8dac-4c1b-897f-9e53cb59c7ee, existing WT
7c85f5ef-3fac-4f72-aaed-08474d7fb454. One newly released fixture used the
reviewed controller ordering; this was not an uncorrected repeat of the
earlier fourth-Authorization-refusal attempt. Root owns D01/D05 gates.

**Actual outcome: failed at browser_prepare_deadline.** The controller exited
1 at stage browser_prepare before the exclusive marker was written. The
still-live controller check then raised ProcessLookupError; that command
exited 1 before marker creation. No helper Popen, helper listener, RP request,
protected navigation, password sign-in, consent, callback, crypto/userinfo
verification or protected-access result was reached. No retry, correction,
new readiness request, alternative target/provider or counter change followed.

### Fresh prerequisites, clock and retained executable payload

Fresh capacity before setup at epoch 1790966009.8264651 was 23004610560 bytes,
above the 8.5 GiB start/stop threshold 9126805504 bytes and 8 GiB floor
8589934592. Both new output paths were absent; deployment-private was 0700.
The immediate pre-execution static check sampled 23024545792 bytes and again
confirmed both paths absent. No unrelated process arguments/environment/
signals/files were inspected or changed; no Cargo resource was acquired.

The one inclusive fixture start was 1790966050.847, before Driver state/session
setup and actual controller execution. Active deadline remained START+840,
cleanup allowance 60, inclusive budget 900. It was never reset/extended.
Helper600/pending180/HTTPnative5/CLI60/listener-page30 and the new preparation
gate's total30 remained as reviewed. The unchanged tick() monitored capacity
once per second during controller setup/gate execution.

Before execution the exact public command was retained in memory and hashed:
14902 UTF-8 bytes, SHA-256
f18fab70ddf21d151c6db745fc3263c43c680a3f77bccd3d58b0ed10731f9473.
Its 193-line body is 14879 bytes, SHA-256
7bc7270c4230b7dc760b319d43b9b14a40005aad030e54fe0a3f926bf903ebb5.
Static AST parse/code-object compile passed without executing/importing that
check. Reversing only fresh START and the two fresh output paths reconstructed
the exact reviewed 14899-byte command SHA35d1ad and body SHA89a098, byte-for-byte
and AST-equivalent. That reviewed candidate's only ordering change remained
the 14-line/11-AST gate plus two finite failure labels. No controller/source/
helper file was materialized or edited.

These exact new exclusive paths were used and preserved:

- deployment-private/d01-confidential-browser-prepared.redacted.json
- deployment-private/d01-confidential-browser-prepared-provider-20261002.json

Complete exact retained public command, recorded before its one execution and
archived here afterwards; generated-value source is not a runtime secret:

```sh
python3 -u - <<'PY'
import datetime,fcntl,hashlib,json,os,pathlib,pty,secrets,select,shutil,signal,stat,subprocess,sys,tempfile,termios,time,urllib.request
ROOT=pathlib.Path.cwd()
START=1790966050.847
ACTIVE=START+840
BIN=pathlib.Path('/Users/dominik/orca/projects/riAuth-public-preview-local-module-boundaries-wave27/target/d01-essentials-c01c39a/aarch64-apple-darwin/debug')
PRIVATE=ROOT/'deployment-private'
OUT=PRIVATE/'d01-confidential-browser-prepared.redacted.json'
PROVIDER_META=PRIVATE/'d01-confidential-browser-prepared-provider-20261002.json'
record={'schema':'riauth.d01-confidential-browser-outer/v1','result':'failed','failure_stage':None,'failure_tag':None,'request_invalid_reason':None,'preflow_authorization_refusals':None,'commands':[],'artifacts':{},'cleanup':{},'owned_child_exits':[],'started_epoch':START,'active_seconds':840,'cleanup_seconds':60,'minimum_free_bytes':None,'disk_samples':0}
lab=None;children=[];names={};server=None;helper=None;password=None;stage='preflight';resultfd=None;lastsample=0
def require(ok,tag):
    if not ok:raise RuntimeError(tag)
def tick():
    global lastsample
    now=time.monotonic()
    if now-lastsample>=1:
        free=shutil.disk_usage(ROOT).free
        record['disk_samples']+=1
        record['minimum_free_bytes']=free if record['minimum_free_bytes'] is None else min(record['minimum_free_bytes'],free)
        lastsample=now
        require(free>=8.5*1024**3,'disk_margin')
    require(time.time()<ACTIVE,'active_deadline')
def write_exclusive(path,value):
    fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    raw=(json.dumps(value,sort_keys=True,indent=2)+'\n').encode('ascii')
    with os.fdopen(fd,'wb') as f:
        f.write(raw);f.flush();os.fsync(f.fileno())
    return hashlib.sha256(raw).hexdigest()
def listeners(port):
    p=subprocess.run(['/usr/sbin/lsof','-nP','-t','-iTCP:'+str(port),'-sTCP:LISTEN'],capture_output=True,timeout=3)
    require(p.returncode in (0,1),'socket_observation_failed')
    return set(int(v) for v in p.stdout.split())
def own(p,name):
    children.append(p);names[p.pid]=name;return p
def stop_child(p):
    if p.poll() is None:
        for sig,wait in [(signal.SIGINT,8),(signal.SIGTERM,5),(signal.SIGKILL,2)]:
            if p.poll() is not None:break
            p.send_signal(sig)
            try:p.wait(timeout=wait)
            except subprocess.TimeoutExpired:pass
    require(p.poll() is not None,'owned_child_reap_failed')
def controlling_tty():
    os.setsid();fcntl.ioctl(0,termios.TIOCSCTTY,0)
def cli(name,args,prompts=0):
    tick()
    master,slave=pty.openpty()
    p=own(subprocess.Popen(args,cwd=lab,env=environment,stdin=slave,stdout=slave,stderr=slave,preexec_fn=controlling_tty),name)
    os.close(slave);seen=0;raw=b'';started=time.monotonic()
    try:
        while p.poll() is None:
            tick();require(time.monotonic()-started<60,'cli_deadline')
            if select.select([master],[],[],0.2)[0]:
                try:chunk=os.read(master,4096)
                except OSError:chunk=b''
                raw+=chunk;require(len(raw)<=131072,'cli_output_limit')
                for prompt in ([b'Password: ',b'Confirm password: '] if prompts==2 else [b'Password: ']):
                    if seen<prompts and prompt in raw:
                        require(prompt==([b'Password: ',b'Confirm password: '][seen] if prompts==2 else b'Password: '),'cli_prompt_order')
                        os.write(master,password.encode()+b'\n');seen+=1;raw=b''
        while select.select([master],[],[],0)[0]:
            try:
                chunk=os.read(master,4096)
                if not chunk:break
                raw+=chunk
            except OSError:break
        code=p.wait()
        record['commands'].append({'name':name,'exit':code,'password_prompts':seen})
        require(code==0,'cli_nonzero');require(seen==prompts,'cli_prompt_missing')
    finally:
        raw=b'';os.close(master)
        if p.poll() is None:stop_child(p)
def wait_listener(p,port):
    deadline=time.monotonic()+30
    while time.monotonic()<deadline:
        tick();require(p.poll() is None,'owned_service_early_exit')
        owners=listeners(port)
        if owners:
            require(owners=={p.pid},'listener_owner_mismatch');return
        time.sleep(0.2)
    raise RuntimeError('listener_deadline')
try:
    tick()
    require(PRIVATE.is_dir() and not PRIVATE.is_symlink() and stat.S_IMODE(PRIVATE.stat().st_mode)==0o700,'private_directory_invalid')
    require(not OUT.exists() and not PROVIDER_META.exists(),'evidence_already_exists')
    require(not listeners(9000) and not listeners(3000),'port_occupied');record['ports_preflight_empty']=True
    pins={'riauth':'7abf745c10691a012a1918c83089168d0e0d88b42764dbf8ed538b90c828b606','riauth-maintenance':'86490c7f71de6b7ae9d4dabdf9060a8757100f3aedbdaa670284d9e1206a2a95','riauthctl':'bfbbb322f1a66d0ac9998beb9fb5838097cea0442cea3fd3a1d057fcb3f600cf'}
    for name,pin in pins.items():
        tick()
        with (BIN/name).open('rb') as f:digest=hashlib.file_digest(f,'sha256').hexdigest()
        record['artifacts'][name]=digest;require(digest==pin,'artifact_hash_mismatch')
    helper_path=ROOT/'scripts/d01-confidential-browser-demo.py'
    require(hashlib.sha256(helper_path.read_bytes()).hexdigest()=='7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0','helper_hash_mismatch')
    verifier=subprocess.check_output(['git','show','9cefe7a56425bb73c17753e8766d92320b77da3b:scripts/recovery-drill-oidc.py'],timeout=5)
    require(hashlib.sha256(verifier).hexdigest()=='f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d','verifier_hash_mismatch')
    provider=pathlib.Path('/opt/homebrew/bin/openssl').resolve(strict=True)
    with provider.open('rb') as f:provider_hash=hashlib.file_digest(f,'sha256').hexdigest()
    require(provider_hash=='67a83dd6d6d747d50c5d296dffb23e32bae9a2c588c93ae2d77e4c607b455c72','provider_hash_mismatch')
    provider_env={k:v for k,v in os.environ.items() if k in {'PATH','HOME','TMPDIR','LANG','LC_ALL'}}
    p=subprocess.Popen([str(provider),'version'],stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=provider_env)
    try:stdout,stderr=p.communicate(timeout=5);timeout=False
    except subprocess.TimeoutExpired:
        p.kill();stdout,stderr=p.communicate(timeout=2);timeout=True
    provider_record={'sha256':provider_hash,'exit':p.returncode,'timeout':timeout,'stdout_ascii':stdout[:4096].decode('ascii',errors='backslashreplace'),'stderr_ascii':stderr[:4096].decode('ascii',errors='backslashreplace'),'stdout_bytes':len(stdout),'stderr_bytes':len(stderr),'environment_keys':sorted(provider_env)}
    record['provider_metadata_sha256']=write_exclusive(PROVIDER_META,provider_record)
    require(not timeout and p.returncode==0 and len(stdout)<=256 and len(stderr)<=4096 and stdout.decode('ascii').strip()=='OpenSSL 3.6.4 25 Aug 2026 (Library: OpenSSL 3.6.4 25 Aug 2026)','provider_version_failed')
    tick()
    resultfd=os.open(OUT,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    lab=pathlib.Path(tempfile.mkdtemp(prefix='d01-confidential-browser-diagnostic.',dir=PRIVATE));os.chmod(lab,0o700)
    for name in ('xdg','deployment-private','rp'):(lab/name).mkdir(mode=0o700)
    (lab/'recovery-drill-oidc.py').write_bytes(verifier);os.chmod(lab/'recovery-drill-oidc.py',0o600);verifier=None
    password=secrets.token_urlsafe(30)
    fd=os.open(lab/'browser-password',os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
    with os.fdopen(fd,'w') as f:f.write(password)
    environment=dict(provider_env);environment['XDG_CONFIG_HOME']=str(lab/'xdg')
    stage='init';cli('maintenance_init',[str(BIN/'riauth-maintenance'),'--config',str(lab/'riauth.toml'),'init','--issuer','http://localhost:9000','--listen','127.0.0.1:9000','--data-dir','data','--admin','admin'],2)
    stage='serve'
    server=own(subprocess.Popen([str(BIN/'riauth'),'--config',str(lab/'riauth.toml'),'serve'],cwd=lab,env=environment,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL),'server')
    record['server_pid']=server.pid;wait_listener(server,9000);record['server_listener_owned']=True
    stage='readyz'
    with urllib.request.urlopen('http://127.0.0.1:9000/readyz',timeout=5) as response:
        record['readyz_status']=response.status;response.read(4096)
    require(record['readyz_status']==200,'readyz_failed')
    stage='cli_login'
    base=[str(BIN/'riauthctl'),'--server','http://localhost:9000']
    cli('operator_login',base+['login','admin'],1)
    stage='client_create';secretpath=lab/'deployment-private/local-demo-secret.json'
    cli('confidential_client_create',base+['client','create','local-demo','--name','Local demo','--confidential','--redirect-uri','http://localhost:3000/callback','--scope','openid,profile','--secret-file',str(secretpath)])
    require(stat.S_IMODE(secretpath.stat().st_mode)==0o600,'cli_secret_mode_invalid')
    stage='discovery';cli('discovery',base+['discovery'])
    stage='whoami';cli('whoami',base+['whoami'])
    stage='browser_prepare'
    prepare_deadline=time.monotonic()+30
    prepare_marker=lab/'browser-prepared'
    print(json.dumps({'browser_prepare_required':True,'guard_pid':os.getpid(),'server_pid':server.pid,'lab':str(lab)}),flush=True)
    while not prepare_marker.exists():
        tick();require(server.poll() is None,'idp_early_exit')
        require(not (lab/'ui-failure').exists() and not (lab/'stop').exists(),'browser_checkpoint_failed')
        require(time.monotonic()<prepare_deadline,'browser_prepare_deadline')
        time.sleep(0.2)
    tick();require(server.poll() is None,'idp_early_exit')
    require(not (lab/'ui-failure').exists() and not (lab/'stop').exists(),'browser_checkpoint_failed')
    require(time.monotonic()<prepare_deadline,'browser_prepare_deadline')
    prepare_info=prepare_marker.lstat()
    require(stat.S_ISREG(prepare_info.st_mode) and stat.S_IMODE(prepare_info.st_mode)==0o600 and prepare_info.st_uid==os.getuid() and prepare_info.st_nlink==1 and prepare_info.st_size==0,'browser_prepare_marker_invalid')
    stage='rp_start'
    helper=own(subprocess.Popen([sys.executable,'-B',str(helper_path),'--workspace',str(lab/'rp'),'--secret-file',str(secretpath),'--verifier-helper',str(lab/'recovery-drill-oidc.py'),'--openssl','/opt/homebrew/bin/openssl','--deadline-seconds','600','--evidence',str(lab/'rp-result.json')],cwd=ROOT,env=environment,stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,text=True),'helper')
    record['helper_pid']=helper.pid;record['helper_invocations']=1
    wait_listener(helper,3000);record['helper_listener_owned']=True
    print(json.dumps({'fixture_ready':True,'guard_pid':os.getpid(),'server_pid':server.pid,'helper_pid':helper.pid,'lab':str(lab),'commands':record['commands'],'provider_metadata_sha256':record['provider_metadata_sha256'],'minimum_free_bytes':record['minimum_free_bytes']}),flush=True)
    stage='browser_checkpoint';announced=False
    while not (lab/'stop').exists():
        tick();require(server.poll() is None,'idp_early_exit')
        if helper.poll() is not None:
            code=helper.wait()
            if not announced:
                record['helper_exit']=code;record['helper_result']=json.loads((lab/'rp-result.json').read_bytes())
                record['request_invalid_reason']=record['helper_result']['request_invalid_reason']
                record['preflow_authorization_refusals']=record['helper_result']['preflow_authorization_refusals']
                print(json.dumps({'helper_completed':True,'exit':code,'result':record['helper_result']['result'],'failure_tag':record['helper_result']['failure_tag'],'request_invalid_reason':record['request_invalid_reason'],'preflow_authorization_refusals':record['preflow_authorization_refusals']}),flush=True);announced=True
            require(code==0,'rp_nonzero')
        time.sleep(0.2)
    if (lab/'ui-failure').exists():raise RuntimeError('browser_checkpoint_failed')
    require(helper.poll()==0,'rp_checkpoint_incomplete');record['result']='passed'
except Exception as error:
    record['failure_stage']=stage
    tags={'disk_margin','active_deadline','private_directory_invalid','evidence_already_exists','port_occupied','socket_observation_failed','artifact_hash_mismatch','helper_hash_mismatch','verifier_hash_mismatch','provider_hash_mismatch','provider_version_failed','cli_deadline','cli_output_limit','cli_prompt_order','cli_prompt_missing','cli_nonzero','owned_service_early_exit','listener_owner_mismatch','listener_deadline','readyz_failed','cli_secret_mode_invalid','idp_early_exit','rp_nonzero','browser_checkpoint_failed','rp_checkpoint_incomplete','browser_prepare_deadline','browser_prepare_marker_invalid'}
    record['failure_tag']=str(error) if str(error) in tags else 'outer_unexpected_failure'
finally:
    for p in reversed(children):
        try:stop_child(p)
        except Exception:record['cleanup']['child_reap_failure']=True
        record['owned_child_exits'].append({'name':names[p.pid],'pid':p.pid,'exit':p.poll()})
    record['cleanup']['owned_children_reaped']=all(p.poll() is not None for p in children)
    if lab is not None:
        if helper is not None:
            record['helper_exit']=helper.poll()
            if (lab/'rp-result.json').exists():
                record['helper_result']=json.loads((lab/'rp-result.json').read_bytes())
                record['request_invalid_reason']=record['helper_result']['request_invalid_reason']
                record['preflow_authorization_refusals']=record['helper_result']['preflow_authorization_refusals']
        password=None;shutil.rmtree(lab);record['cleanup']['lab_removed']=not lab.exists()
    else:record['cleanup']['lab_removed']=True
    try:
        record['cleanup']['port9000_absent']=not listeners(9000);record['cleanup']['port3000_absent']=not listeners(3000)
    except Exception:record['cleanup']['socket_observation_failure']=True
    record['completed_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();record['elapsed_seconds']=round(time.time()-START,3)
    if resultfd is None and not OUT.exists():resultfd=os.open(OUT,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    if resultfd is not None:
        with os.fdopen(resultfd,'wb') as f:
            f.write((json.dumps(record,sort_keys=True,indent=2)+'\n').encode('ascii'));f.flush();os.fsync(f.fileno())
    print(json.dumps({'fixture_finished':True,'result':record['result'],'failure_stage':record['failure_stage'],'failure_tag':record['failure_tag'],'request_invalid_reason':record['request_invalid_reason'],'preflow_authorization_refusals':record['preflow_authorization_refusals'],'cleanup':record['cleanup'],'owned_child_exits':record['owned_child_exits'],'minimum_free_bytes':record['minimum_free_bytes'],'disk_samples':record['disk_samples'],'elapsed_seconds':record['elapsed_seconds']}),flush=True)
sys.exit(0 if record['result']=='passed' else 1)
PY
```

### Pinned operator setup and provider checks actually completed

All three c01 prebuilt binaries were freshly rehashed by this controller:

| Artifact | Matching SHA-256 |
| --- | --- |
| riauth | 7abf745c10691a012a1918c83089168d0e0d88b42764dbf8ed538b90c828b606 |
| riauth-maintenance | 86490c7f71de6b7ae9d4dabdf9060a8757100f3aedbdaa670284d9e1206a2a95 |
| riauthctl | bfbbb322f1a66d0ac9998beb9fb5838097cea0442cea3fd3a1d057fcb3f600cf |

The artifact directory and c01c39ab4e092423d5522bedc50fff87656d8c0a source pin
remain the accepted Essentials inputs from the prior phase. No build ran.
The unchanged helper470 was freshly hash-checked as
7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0,
32723 bytes, source470690cad0cd93c9f25c5bc40b982e1b91679b49.
Verifier source9cefe7a56425bb73c17753e8766d92320b77da3b/blob
3be747d03146f1bcaa3ec012ee8d173b61fa737d was freshly source-read/hash-checked as
f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d and copied
privately. It was not imported/executed by a helper in this failed phase.

The resolved provider hash matched
67a83dd6d6d747d50c5d296dffb23e32bae9a2c588c93ae2d77e4c607b455c72.
Its one five-second outer version invocation returned exit0/no timeout,
stdout63 ASCII bytes, stderr0, with only HOME/LANG/LC_ALL/PATH/TMPDIR.
Full stdout and numeric exit/lengths were retained in the new exclusive0600
metadata BEFORE the exact full-version assertion:

```text
OpenSSL 3.6.4 25 Aug 2026 (Library: OpenSSL 3.6.4 25 Aug 2026)\n
```

Normal printed section2/3 prerequisites created one fresh0700 lab/privateXDG,
synthetic stdin password and0600 password/client-secret files. There was no
policy/review/header/receipt bypass or service-session application approval.
Both ports were initially absent; port9000 was proven owned by server58067.
The unchanged IdP GET /readyz observed200. No RP readiness HTTP was added.

| Printed prerequisite | Actual exit | Password prompts |
| --- | ---: | ---: |
| maintenance init | 0 | 2 |
| operator administrator login | 0 | 1 |
| confidential Local demo client creation | 0 | 0 |
| discovery | 0 | 0 |
| whoami | 0 | 0 |

These commands are operator setup, not browser user authentication. Matching
guide/source/artifact identities do not supply a full independent body review,
current CI/release/HA observation or confidential application journey pass.

### One Driver-owned preparation and exact binding actually observed

Refreshed installed cua-driver SKILL.md (skill0.29.1), MACOS.md/BROWSER.md
instructions and advertised MCP tool descriptions before actions. Some combined
read/description outputs were truncated; applicable browser/platform/schema
portions were read in bounded follow-ups. History tools were not advertised.
No history search, screenshots, recording, rawCDP/Playwright/AppleScript,
native accessibility testing, permission automation or security/profile
changes occurred.

Fresh check_permissions(prompt=false) observed Accessibility and Screen
Recording granted for com.trycua.driver; direct capture was not probed.
Restricted health_report observed driver0.30.4/macOS26.2 arm64/overallok,
bundleidentity/TCC checks pass, AX/capture capability checks skipped.
list_sessions returned0 visible sessions. The one named lifecycle
d01-prepared-checkpoint then started active.

The controller emitted browser_prepare_required with exact own guard58030,
server58067 and its fresh private lab. By the reviewed source gate, helper
creation/port3000 binding could not yet occur at that event.

Exactly one browser_prepare with allow_launch true/profile isolated_new returned
prepared true, launched_isolated_browser, spawned_by_driver owner PID58336.
Its side-effects report showed new profile/browser creation and no copied
personal data, preference change, personal process modification/termination,
foregrounding or consent prompt. list_windows for that exact PID had one
about:blank candidate, window103442; get_browser_state bound it with screenshots
disabled, statusok, binding_qualityexact, mutation_allowedtrue and
endpoint_access_classdriver_owned. Returned target
bt-e2a86575-3875-437e-9d63-51a3844d95c6, selected tab
tab-87c25d09-01da-49ec-985a-dd8ada33c9c2, URL about:blank.

The binding result did not release the gate by itself. The next exact owned
liveness check os.kill(58030,0), before the reviewed exclusive marker writer,
failed with ProcessLookupError. The writer was never reached and no marker
was created. Poll/join returned the controller's already-completed fixed
browser_prepare_deadline failure. No helper/fixture_ready/navigation occurred.
The browser remained blank. The proposal's actual readiness prerequisite
therefore failed; this is not a product authentication/callback failure.

Own call wrappers retained these wall-clock observations, not new controller/
helper timing fields and not network/request arrival timestamps:

| Recorded own event | Epoch seconds |
| --- | ---: |
| Inclusive START | 1790966050.847 |
| Before Driver prepare invocation | 1790966144.644 |
| After Driver prepare returned | 1790966146.127 |
| After exact blank binding returned | 1790966146.424 |
| Controller completed UTC converted by its retained start+elapsed | about1790966154.987 |
| Fresh cleanup readback | 1790966224.7524512 |

The observed prepare call interval was about1.483s and preparation-through-
binding interval about1.780s from that invocation. These are wall-clock call
intervals. Binding returned before the controller completion timestamp, but
the marker/liveness check happened after the controller had exited. No exact
marker-call timestamp or gate-entry timestamp was retained. Do not invent
those times, attribute refused traffic or derive an old GUI delay from them.
The fixed recorded failure shows the marker was not delivered within the
30s gate. No deadline extension/rebind/retry or helper timestamp change followed.

### Actual fixed result, capacity, evidence and owned cleanup

Outer result: failed, stagebrowser_prepare, tagbrowser_prepare_deadline,
elapsed104.14s, disk_samples32, minimum_free_bytes23009157120,
completed UTC2026-10-02T18:35:54.986681+00:00.
request_invalid_reason and preflow_authorization_refusals remained null.
No helper_invocations/helper_pid/helper_exit/helper_result fields were produced:
the helper never started. Null does not mean counterreset, zero refused
traffic or successful denial. There is no current Authorization observation.
The sender of earlier failed attempts remains UNKNOWN.

| New exclusive private evidence | Bytes | Mode | SHA-256 |
| --- | ---: | --- | --- |
| deployment-private/d01-confidential-browser-prepared.redacted.json | 2077 | 0600 | 062f8ea18abe84db1dd8f2e1235a286b586ad2ab473e8570523e790cc76ccc1b |
| deployment-private/d01-confidential-browser-prepared-provider-20261002.json | 359 | 0600 | 6fad035f1897b5287b2ebd812405fcabf6a33b72d2d4f498f3bca7081eb719ab |

The controller finally reaped every owned child, removed only its own private
lab, and reported port9000/3000 absent. All numeric exits were0:

| Owned child | PID | Exit |
| --- | ---: | ---: |
| whoami | 58081 | 0 |
| discovery | 58080 | 0 |
| confidential_client_create | 58079 | 0 |
| operator_login | 58075 | 0 |
| server | 58067 | 0 |
| maintenance_init | 58063 | 0 |

Driver cleanup targeted only proven owned browser58336. Background cmd+q
returned effectunverifiable/delivery_failed; fresh exact-PID list_windows
still found11 windows. Exact-owned Driver kill_app sentSIGKILL, then
end_session returnedactivefalse. Final exact-PID list_windows found0 windows.
No authenticated application session existed to log out. No unrelated
process/profile was closed.

Fresh readback epoch1790966224.7524512 found own controller58030, server58067
and browser58336 all absent; exact ports9000/3000 lsof each exited1 with
empty stdout/stderr; ownlab absent. Free capacity23016632320 bytes remained
above8.5GiB. No HTTP request was used for cleanup proof.

This fresh readback was173.9054512s after inclusiveSTART, within900s.
It was about69.766s after the controller's completion timestamp. The browser
cleanup action instants were not retained; a complete60s cleanup measured
from controller completion is therefore **not demonstrated** by these records.
Do not claim the readback met that stricter60s interval. This timing limitation
is preserved for root adjudication alongside the gate failure.

After joined exit/cleanup readbacks and before this appendix, commentary
explicitly released DESKTOP/OPERATOR with the actual stage/tag, no-helper/
no-navigation outcome and resource absence. No Cargo acquisition/release
occurred. The attempt ended; no automatic continuation or correction is
authorized.

### Prefix, historical failures and handoff limits

The complete prior4b86774 report,241893bytes/3947lines, blob
601d84f9d7c78863b29936337adfafe49d6e3ae1, SHA
05fa9cd076f77f3922ee93edce2ae08a8a7342d8172ea3debcbb654b096625ff remains
a byte-exact prefix. All7 earlier private metadata files retain their dated
bytes/hashes/0600 modes; the new2 are separate exclusive records. Original
controller archives and reviewed candidate remain unchanged.

The prior4b86774 postcommit handoff command exited2 with
“riwork: leave copy mode and enable terminal input before submitting”.
That refusal remains retained; no terminal input mode, desktop workaround,
provider switch or automatic send retry was performed. This runtime's immediate
resource release is the commentary receipt above. Any postcommit orchestrator
receipt must obey the current fresh-prompt restriction; delivery cannot be
claimed from a refused send.

All earlier lost-provider-output/reporting correction, first failed fixtures,
senderUNKNOWN,61/76 failed memory run, dated78case pass, capacity refusal and
fourth-Authorization-refusal/browser-error evidence remain unchanged. This
new gate timeout neither replaces those failures nor credits the prepared blank
browser as a user journey. The author's reviewer/browser-worker dual role
remains explicit; this is actual evidence of their own action, not independent
verification of it.

No password sign-in, consent, callback, confidential code exchange,
RS256/JWKS/issuer/audience/nonce/time/accesshash, userinfo or protected403/200
was observed. Physical passkey/hardware/tenant/application installation/
invitation/LDAP/SCIM/nonadmin/fullD01/D05/CI/release gates receive no credit.
D01/D05 remain root-owned; accepted closed rows stay closed.

Only this report is appended and committed. No source/helper/product/guide/
D05/test/config edits, builds/Cargo, new task/worker/worktree/shell,
main/push/status/merge or other-worker contact. Static payload reversal,
archive/prefix/helper identity, all9 private metadata hashes/modes,
fences/finalnewline/whitespace and git diff --check are checked; report-only
commit and clean tracked/staged handoff are read back without fixture/runtime.

## 2026-10-02: preparation budget and measured cleanup — source-only proposal

Reservation wave30_D01_preparation_budget_and_cleanup_design, project
891e7443-8dac-4c1b-897f-9e53cb59c7ee, existing WT
7c85f5ef-3fac-4f72-aaed-08474d7fb454. Only this report is appended.
No Driver/native/provider/CLI/server/helper/listener/browser/network/Cargo
fixture is invoked and no resource slot is acquired/released. Root separately
reviews/reserves any implementation or runtime. D01/D05 remain open;
O06/I10/R05 and the other accepted closed rows stay closed.

**Controller proposal:** change exactly one literal, preparation deadline
30 to180 seconds. The original active START+840 and inclusive900/cleanup60
budgets remain unchanged. No reset, extension on retry, helper change, new
accepted request or extra counter allowance is proposed.

**Cleanup proposal:** cache proven owned handles when returned/bound; execute
one immediate exact-owned Driver kill_app, then end_session, then fresh exact
window/PID/ports/lab absence readbacks, recording start/end clocks and child
exits. Do this as one contiguous outer cleanup sequence without skill/schema
loading or a model turn between cleanup actions. The user explicitly requests
direct kill for this disposable owned browser; it replaces the cooperative
cmd+q action that failed and created an additional window in both retained
attempts. No manual process termination or substitute GUI provider is proposed.

The sequence has finite action counts and measurable budgets. **A hard60s
completion guarantee is unavailable:** advertised MCP schemas expose no
operation timeout/cancellation parameter, and the unchanged controller does
not timestamp the first autonomous cleanup event. Both limitations remain
explicit; a late/unknown result never becomes a measured60s success.

### Immutable actual inputs and historical timing limits

Read the complete exact14902-byte retained aca controller, SHA
f18fab70ddf21d151c6db745fc3263c43c680a3f77bccd3d58b0ed10731f9473;
193-line body SHA
7bc7270c4230b7dc760b319d43b9b14a40005aad030e54fe0a3f926bf903ebb5.
The full aca6aab606e7cabb8875f7d038c6aef119a46945 report is the immutable
phase base,270696bytes/4398lines, blob
c0ef18980df310adb12ca9c42fb5fd9c4f9d1d95, SHA
f6c998fddc37408378ac8c291bc2505a00c03321c0a0cb5f12c9617c2e9882b2.

The prior actual outcome remains browser_prepare_deadline, helper NOT RUN,
no navigation, marker liveness check ProcessLookupError before writing,
outer elapsed104.14s. Own preparation-through-binding wall interval1.780s
is the observed interval from that attempt; it is not a guaranteed next-call
duration. The marker was not delivered before the gate deadline.
No gate-entry/marker-call timestamp was retained, so no exact orchestration
gap, older GUI duration or sender attribution is inferred.

The fresh final-absence readback was about69.766s after controller completion;
a full60s cleanup interval from that timestamp remains not demonstrated.
All nine private metadata files remain dated, byte/hash/mode0600 unchanged,
including the2077-byte prepared result SHA
062f8ea18abe84db1dd8f2e1235a286b586ad2ab473e8570523e790cc76ccc1b and359-byte
prepared provider result SHA
6fad035f1897b5287b2ebd812405fcabf6a33b72d2d4f498f3bca7081eb719ab.
Earlier Authorization senders remain UNKNOWN. This phase supplies no new
current timing, permissions, resource-state or journey observation.

### Exact one-literal prospective controller

Complete candidate command14903 UTF-8 bytes, SHA
3d6b4124bbc77cb82166729c3e529e08d5cc7cab28b5561c973390386cb896b7.
Body14880bytes/193lines, SHA
e251f4e45f80c4edffd94d94a8a8ff13bb30eb8ed6113bdc96e249caed307881.
The only edit from the retained aca command is this complete zero-context diff:

```diff
--- retained-aca-controller
+++ PROSPECTIVE-180s-preparation-controller
@@ -134 +134 @@
-    prepare_deadline=time.monotonic()+30
+    prepare_deadline=time.monotonic()+180
```

This candidate deliberately inherits the historical START and already-used
output names to isolate the one-literal source review. It is not executable
as a fresh fixture. A later root reservation must separately specify fresh
START and unused exclusive outer/provider paths; reverse those substitutions
as well when checking actual future bytes against this archive.

The deadline is assigned once. Both existing time.monotonic() checks compare
against that same value; neither resets it. Existing tick() still stops at
the original wall-clock START+840 and disk threshold8.5GiB on one-second
samples. Thus preparation receives at most180s subject to the remaining
original active budget, not an additional180s beyond it. The sleep0.2,
two fixed prepare failure labels, private zero0600 marker predicates, live
owned server/failure/stop guards and single helper launch stay exact.
Listener30/page30, helper600/pending180, nativeHTTP5 andCLI60 stay exact.
No reprepare/rebind/retry or alternate port/profile/provider is introduced.

Complete exact prospective source archive; never executed/imported here:

```sh
python3 -u - <<'PY'
import datetime,fcntl,hashlib,json,os,pathlib,pty,secrets,select,shutil,signal,stat,subprocess,sys,tempfile,termios,time,urllib.request
ROOT=pathlib.Path.cwd()
START=1790966050.847
ACTIVE=START+840
BIN=pathlib.Path('/Users/dominik/orca/projects/riAuth-public-preview-local-module-boundaries-wave27/target/d01-essentials-c01c39a/aarch64-apple-darwin/debug')
PRIVATE=ROOT/'deployment-private'
OUT=PRIVATE/'d01-confidential-browser-prepared.redacted.json'
PROVIDER_META=PRIVATE/'d01-confidential-browser-prepared-provider-20261002.json'
record={'schema':'riauth.d01-confidential-browser-outer/v1','result':'failed','failure_stage':None,'failure_tag':None,'request_invalid_reason':None,'preflow_authorization_refusals':None,'commands':[],'artifacts':{},'cleanup':{},'owned_child_exits':[],'started_epoch':START,'active_seconds':840,'cleanup_seconds':60,'minimum_free_bytes':None,'disk_samples':0}
lab=None;children=[];names={};server=None;helper=None;password=None;stage='preflight';resultfd=None;lastsample=0
def require(ok,tag):
    if not ok:raise RuntimeError(tag)
def tick():
    global lastsample
    now=time.monotonic()
    if now-lastsample>=1:
        free=shutil.disk_usage(ROOT).free
        record['disk_samples']+=1
        record['minimum_free_bytes']=free if record['minimum_free_bytes'] is None else min(record['minimum_free_bytes'],free)
        lastsample=now
        require(free>=8.5*1024**3,'disk_margin')
    require(time.time()<ACTIVE,'active_deadline')
def write_exclusive(path,value):
    fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    raw=(json.dumps(value,sort_keys=True,indent=2)+'\n').encode('ascii')
    with os.fdopen(fd,'wb') as f:
        f.write(raw);f.flush();os.fsync(f.fileno())
    return hashlib.sha256(raw).hexdigest()
def listeners(port):
    p=subprocess.run(['/usr/sbin/lsof','-nP','-t','-iTCP:'+str(port),'-sTCP:LISTEN'],capture_output=True,timeout=3)
    require(p.returncode in (0,1),'socket_observation_failed')
    return set(int(v) for v in p.stdout.split())
def own(p,name):
    children.append(p);names[p.pid]=name;return p
def stop_child(p):
    if p.poll() is None:
        for sig,wait in [(signal.SIGINT,8),(signal.SIGTERM,5),(signal.SIGKILL,2)]:
            if p.poll() is not None:break
            p.send_signal(sig)
            try:p.wait(timeout=wait)
            except subprocess.TimeoutExpired:pass
    require(p.poll() is not None,'owned_child_reap_failed')
def controlling_tty():
    os.setsid();fcntl.ioctl(0,termios.TIOCSCTTY,0)
def cli(name,args,prompts=0):
    tick()
    master,slave=pty.openpty()
    p=own(subprocess.Popen(args,cwd=lab,env=environment,stdin=slave,stdout=slave,stderr=slave,preexec_fn=controlling_tty),name)
    os.close(slave);seen=0;raw=b'';started=time.monotonic()
    try:
        while p.poll() is None:
            tick();require(time.monotonic()-started<60,'cli_deadline')
            if select.select([master],[],[],0.2)[0]:
                try:chunk=os.read(master,4096)
                except OSError:chunk=b''
                raw+=chunk;require(len(raw)<=131072,'cli_output_limit')
                for prompt in ([b'Password: ',b'Confirm password: '] if prompts==2 else [b'Password: ']):
                    if seen<prompts and prompt in raw:
                        require(prompt==([b'Password: ',b'Confirm password: '][seen] if prompts==2 else b'Password: '),'cli_prompt_order')
                        os.write(master,password.encode()+b'\n');seen+=1;raw=b''
        while select.select([master],[],[],0)[0]:
            try:
                chunk=os.read(master,4096)
                if not chunk:break
                raw+=chunk
            except OSError:break
        code=p.wait()
        record['commands'].append({'name':name,'exit':code,'password_prompts':seen})
        require(code==0,'cli_nonzero');require(seen==prompts,'cli_prompt_missing')
    finally:
        raw=b'';os.close(master)
        if p.poll() is None:stop_child(p)
def wait_listener(p,port):
    deadline=time.monotonic()+30
    while time.monotonic()<deadline:
        tick();require(p.poll() is None,'owned_service_early_exit')
        owners=listeners(port)
        if owners:
            require(owners=={p.pid},'listener_owner_mismatch');return
        time.sleep(0.2)
    raise RuntimeError('listener_deadline')
try:
    tick()
    require(PRIVATE.is_dir() and not PRIVATE.is_symlink() and stat.S_IMODE(PRIVATE.stat().st_mode)==0o700,'private_directory_invalid')
    require(not OUT.exists() and not PROVIDER_META.exists(),'evidence_already_exists')
    require(not listeners(9000) and not listeners(3000),'port_occupied');record['ports_preflight_empty']=True
    pins={'riauth':'7abf745c10691a012a1918c83089168d0e0d88b42764dbf8ed538b90c828b606','riauth-maintenance':'86490c7f71de6b7ae9d4dabdf9060a8757100f3aedbdaa670284d9e1206a2a95','riauthctl':'bfbbb322f1a66d0ac9998beb9fb5838097cea0442cea3fd3a1d057fcb3f600cf'}
    for name,pin in pins.items():
        tick()
        with (BIN/name).open('rb') as f:digest=hashlib.file_digest(f,'sha256').hexdigest()
        record['artifacts'][name]=digest;require(digest==pin,'artifact_hash_mismatch')
    helper_path=ROOT/'scripts/d01-confidential-browser-demo.py'
    require(hashlib.sha256(helper_path.read_bytes()).hexdigest()=='7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0','helper_hash_mismatch')
    verifier=subprocess.check_output(['git','show','9cefe7a56425bb73c17753e8766d92320b77da3b:scripts/recovery-drill-oidc.py'],timeout=5)
    require(hashlib.sha256(verifier).hexdigest()=='f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d','verifier_hash_mismatch')
    provider=pathlib.Path('/opt/homebrew/bin/openssl').resolve(strict=True)
    with provider.open('rb') as f:provider_hash=hashlib.file_digest(f,'sha256').hexdigest()
    require(provider_hash=='67a83dd6d6d747d50c5d296dffb23e32bae9a2c588c93ae2d77e4c607b455c72','provider_hash_mismatch')
    provider_env={k:v for k,v in os.environ.items() if k in {'PATH','HOME','TMPDIR','LANG','LC_ALL'}}
    p=subprocess.Popen([str(provider),'version'],stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=provider_env)
    try:stdout,stderr=p.communicate(timeout=5);timeout=False
    except subprocess.TimeoutExpired:
        p.kill();stdout,stderr=p.communicate(timeout=2);timeout=True
    provider_record={'sha256':provider_hash,'exit':p.returncode,'timeout':timeout,'stdout_ascii':stdout[:4096].decode('ascii',errors='backslashreplace'),'stderr_ascii':stderr[:4096].decode('ascii',errors='backslashreplace'),'stdout_bytes':len(stdout),'stderr_bytes':len(stderr),'environment_keys':sorted(provider_env)}
    record['provider_metadata_sha256']=write_exclusive(PROVIDER_META,provider_record)
    require(not timeout and p.returncode==0 and len(stdout)<=256 and len(stderr)<=4096 and stdout.decode('ascii').strip()=='OpenSSL 3.6.4 25 Aug 2026 (Library: OpenSSL 3.6.4 25 Aug 2026)','provider_version_failed')
    tick()
    resultfd=os.open(OUT,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    lab=pathlib.Path(tempfile.mkdtemp(prefix='d01-confidential-browser-diagnostic.',dir=PRIVATE));os.chmod(lab,0o700)
    for name in ('xdg','deployment-private','rp'):(lab/name).mkdir(mode=0o700)
    (lab/'recovery-drill-oidc.py').write_bytes(verifier);os.chmod(lab/'recovery-drill-oidc.py',0o600);verifier=None
    password=secrets.token_urlsafe(30)
    fd=os.open(lab/'browser-password',os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
    with os.fdopen(fd,'w') as f:f.write(password)
    environment=dict(provider_env);environment['XDG_CONFIG_HOME']=str(lab/'xdg')
    stage='init';cli('maintenance_init',[str(BIN/'riauth-maintenance'),'--config',str(lab/'riauth.toml'),'init','--issuer','http://localhost:9000','--listen','127.0.0.1:9000','--data-dir','data','--admin','admin'],2)
    stage='serve'
    server=own(subprocess.Popen([str(BIN/'riauth'),'--config',str(lab/'riauth.toml'),'serve'],cwd=lab,env=environment,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL),'server')
    record['server_pid']=server.pid;wait_listener(server,9000);record['server_listener_owned']=True
    stage='readyz'
    with urllib.request.urlopen('http://127.0.0.1:9000/readyz',timeout=5) as response:
        record['readyz_status']=response.status;response.read(4096)
    require(record['readyz_status']==200,'readyz_failed')
    stage='cli_login'
    base=[str(BIN/'riauthctl'),'--server','http://localhost:9000']
    cli('operator_login',base+['login','admin'],1)
    stage='client_create';secretpath=lab/'deployment-private/local-demo-secret.json'
    cli('confidential_client_create',base+['client','create','local-demo','--name','Local demo','--confidential','--redirect-uri','http://localhost:3000/callback','--scope','openid,profile','--secret-file',str(secretpath)])
    require(stat.S_IMODE(secretpath.stat().st_mode)==0o600,'cli_secret_mode_invalid')
    stage='discovery';cli('discovery',base+['discovery'])
    stage='whoami';cli('whoami',base+['whoami'])
    stage='browser_prepare'
    prepare_deadline=time.monotonic()+180
    prepare_marker=lab/'browser-prepared'
    print(json.dumps({'browser_prepare_required':True,'guard_pid':os.getpid(),'server_pid':server.pid,'lab':str(lab)}),flush=True)
    while not prepare_marker.exists():
        tick();require(server.poll() is None,'idp_early_exit')
        require(not (lab/'ui-failure').exists() and not (lab/'stop').exists(),'browser_checkpoint_failed')
        require(time.monotonic()<prepare_deadline,'browser_prepare_deadline')
        time.sleep(0.2)
    tick();require(server.poll() is None,'idp_early_exit')
    require(not (lab/'ui-failure').exists() and not (lab/'stop').exists(),'browser_checkpoint_failed')
    require(time.monotonic()<prepare_deadline,'browser_prepare_deadline')
    prepare_info=prepare_marker.lstat()
    require(stat.S_ISREG(prepare_info.st_mode) and stat.S_IMODE(prepare_info.st_mode)==0o600 and prepare_info.st_uid==os.getuid() and prepare_info.st_nlink==1 and prepare_info.st_size==0,'browser_prepare_marker_invalid')
    stage='rp_start'
    helper=own(subprocess.Popen([sys.executable,'-B',str(helper_path),'--workspace',str(lab/'rp'),'--secret-file',str(secretpath),'--verifier-helper',str(lab/'recovery-drill-oidc.py'),'--openssl','/opt/homebrew/bin/openssl','--deadline-seconds','600','--evidence',str(lab/'rp-result.json')],cwd=ROOT,env=environment,stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,text=True),'helper')
    record['helper_pid']=helper.pid;record['helper_invocations']=1
    wait_listener(helper,3000);record['helper_listener_owned']=True
    print(json.dumps({'fixture_ready':True,'guard_pid':os.getpid(),'server_pid':server.pid,'helper_pid':helper.pid,'lab':str(lab),'commands':record['commands'],'provider_metadata_sha256':record['provider_metadata_sha256'],'minimum_free_bytes':record['minimum_free_bytes']}),flush=True)
    stage='browser_checkpoint';announced=False
    while not (lab/'stop').exists():
        tick();require(server.poll() is None,'idp_early_exit')
        if helper.poll() is not None:
            code=helper.wait()
            if not announced:
                record['helper_exit']=code;record['helper_result']=json.loads((lab/'rp-result.json').read_bytes())
                record['request_invalid_reason']=record['helper_result']['request_invalid_reason']
                record['preflow_authorization_refusals']=record['helper_result']['preflow_authorization_refusals']
                print(json.dumps({'helper_completed':True,'exit':code,'result':record['helper_result']['result'],'failure_tag':record['helper_result']['failure_tag'],'request_invalid_reason':record['request_invalid_reason'],'preflow_authorization_refusals':record['preflow_authorization_refusals']}),flush=True);announced=True
            require(code==0,'rp_nonzero')
        time.sleep(0.2)
    if (lab/'ui-failure').exists():raise RuntimeError('browser_checkpoint_failed')
    require(helper.poll()==0,'rp_checkpoint_incomplete');record['result']='passed'
except Exception as error:
    record['failure_stage']=stage
    tags={'disk_margin','active_deadline','private_directory_invalid','evidence_already_exists','port_occupied','socket_observation_failed','artifact_hash_mismatch','helper_hash_mismatch','verifier_hash_mismatch','provider_hash_mismatch','provider_version_failed','cli_deadline','cli_output_limit','cli_prompt_order','cli_prompt_missing','cli_nonzero','owned_service_early_exit','listener_owner_mismatch','listener_deadline','readyz_failed','cli_secret_mode_invalid','idp_early_exit','rp_nonzero','browser_checkpoint_failed','rp_checkpoint_incomplete','browser_prepare_deadline','browser_prepare_marker_invalid'}
    record['failure_tag']=str(error) if str(error) in tags else 'outer_unexpected_failure'
finally:
    for p in reversed(children):
        try:stop_child(p)
        except Exception:record['cleanup']['child_reap_failure']=True
        record['owned_child_exits'].append({'name':names[p.pid],'pid':p.pid,'exit':p.poll()})
    record['cleanup']['owned_children_reaped']=all(p.poll() is not None for p in children)
    if lab is not None:
        if helper is not None:
            record['helper_exit']=helper.poll()
            if (lab/'rp-result.json').exists():
                record['helper_result']=json.loads((lab/'rp-result.json').read_bytes())
                record['request_invalid_reason']=record['helper_result']['request_invalid_reason']
                record['preflow_authorization_refusals']=record['helper_result']['preflow_authorization_refusals']
        password=None;shutil.rmtree(lab);record['cleanup']['lab_removed']=not lab.exists()
    else:record['cleanup']['lab_removed']=True
    try:
        record['cleanup']['port9000_absent']=not listeners(9000);record['cleanup']['port3000_absent']=not listeners(3000)
    except Exception:record['cleanup']['socket_observation_failure']=True
    record['completed_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();record['elapsed_seconds']=round(time.time()-START,3)
    if resultfd is None and not OUT.exists():resultfd=os.open(OUT,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    if resultfd is not None:
        with os.fdopen(resultfd,'wb') as f:
            f.write((json.dumps(record,sort_keys=True,indent=2)+'\n').encode('ascii'));f.flush();os.fsync(f.fileno())
    print(json.dumps({'fixture_finished':True,'result':record['result'],'failure_stage':record['failure_stage'],'failure_tag':record['failure_tag'],'request_invalid_reason':record['request_invalid_reason'],'preflow_authorization_refusals':record['preflow_authorization_refusals'],'cleanup':record['cleanup'],'owned_child_exits':record['owned_child_exits'],'minimum_free_bytes':record['minimum_free_bytes'],'disk_samples':record['disk_samples'],'elapsed_seconds':record['elapsed_seconds']}),flush=True)
sys.exit(0 if record['result']=='passed' else 1)
PY
```

Full-byte reversal of only180 to30 reconstructs the entire aca controller.
AST normalization changes precisely the integer Constant belonging to
prepare_deadline,180 to30, and reconstructs the complete old AST exactly.
All nine functions, request builders, provider hash/path/ASCII/full-version
comparison and retention order, printed localhost9000 issuer/confidential
Local demo/localhost3000 callback, helper Popen arguments, readiness request,
result handling and finally remain unchanged. Helper470 remains byte-exact
SHA7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0.
Its strict Host/Authorization/parser/body/cookie/crypto/native/accepted routes,
first-three preflow403/fourth terminal403 and post-flow400 contracts are not
modified. No source/controller/helper file is materialized.

### Preflight and cached ownership before future cleanup

In a separately released future runtime, read installed skills, advertised
schemas and read-only Driver permissions/health/session state before inclusive
START. Retain when/version/provider those observations refer to; they are not
a perpetual capability claim. Do not invoke metadata or skill/description
reads during the timed gate/cleanup interval. Refresh only if an actual
prerequisite becomes invalid, then stop; no repair/retry is authorized.

Set inclusive START once immediately before actual fixture preparation;
all artifact/provider checks, private lab/XDG/CLI setup, browser preparation/
binding and the marker writer count against840 active/900 inclusive.
Fresh capacity before any setup and one-second monitoring remain mandatory.
No time spent in the180s gate resets that global clock.

Cache only this fixture's returned ownership/handles: browser PID from
spawned_by_driver endpoint ownership; exact window/target/tab/session from
the successful exact blank binding; guard/server/helper PIDs and lab from
the same own controller events; numeric child inventory from its own result.
Cache browser ownership as soon as returned and complete the bound-handle
cache when binding succeeds. No URL/query/header/cookie/password/secret is
stored in the cleanup cache. A session label or a title match alone is not
browser process ownership.

If no exact owned browser PID was returned, do not guess a PID or kill an app.
End only the known own session and report browser cleanup identity unknown.
If identity becomes ambiguous/reused, retain the fixed ownership failure;
no broad process search, fallback or unrelated kill is allowed.

### Exact prospective cleanup order and observational allocation

At the first worker-initiated stop/cleanup trigger, latch a shared-host
monotonic clock plus wall epoch **before** the stop signal/Driver action.
For a normal controlled stop, use the existing private stop/UI-failure protocol
and unchanged controller-owned child cleanup. No new manual process signal,
shell browser kill or changed controller finally is introduced.

The clock/source reads below are proposed, not executed now. Capture clocks
and immediately preserve the numeric records in the outer observation cache
before any assertion. Use the same host clock source before/after every action:

```python
# Prospective fixed local clock read only; not executed in this phase.
import json
import time
print(json.dumps({"monotonic_ns":time.monotonic_ns(),
                  "wall_epoch_ns":time.time_ns()},sort_keys=True))
```

Use monotonic elapsed time for budget arithmetic; wall epoch timestamps are
retained for comparison to existing completed_utc and for readable chronology.
A clock-read failure/backward/invalid value makes timing unavailable and
prevents a60s-success claim; required owned cleanup still proceeds once.
No new timing field is added to the unchanged controller/helper.

Let T0 be the recorded cleanup trigger clock, D=T0+60000000000ns.
Each stage checks remaining budget before dispatch and records both returned
clock samples, observed duration and whether its allocation was exceeded.
All Driver calls use only the actual advertised arguments below; no timeout,
deadline, abort signal, delivery mode or undocumented field is injected.

| Order | Exact proposed operation | Observational allocation |
| --- | --- | --- |
| 0 | Latch clock before own cleanup trigger; request existing owned controller stop if still required | No reset; record source/proof status of T0 |
| 1 | mcp__cua_driver__kill_app({pid:OWNED_BROWSER_PID}) once, immediately | max(0, min(30000ms, remaining budget minus10s readback reserve)) |
| 2 | mcp__cua_driver__end_session({session:OWNED_SESSION}) once | max(0, min(15000ms, remaining budget minus10s readback reserve)) |
| 3 | mcp__cua_driver__list_windows({pid:OWNED_BROWSER_PID}) once; retain count/absence only | max(0, min(5000ms, remaining budget minus10s readback reserve)) |
| 4 | Join/read exact owned controller result; collect numeric child exits, own-PID/ports/lab absence plus final clock | Remaining budget; nominal10s reserve |

Exact nonnegative accounting: remaining_ms=max(0, (D-now_monotonic_ns)//1000000);
allowed_ms=max(0, min(stage_cap_ms, remaining_ms-10000)) for the three Driver
steps. Zero allocation marks the budget failed; it never becomes an invented
tool timeout or permission to abandon required owned cleanup.

Missing browser ownership skips operations1/3 and records their absence
proof unknown, rather than declaring no browser exists. Known session cleanup
is still performed once. There is no cooperative cmd+q, native menu,
foreground escalation, logout requiring another authentication, browser
rebind/reprepare, extra HTTP probe or repeated kill.

Owned server/helper cleanup runs in the existing controller concurrently with
Driver cleanup. Its existing stop_child may spend8+5+2s per still-live child;
normally only server/helper remain live after the CLI prerequisites. Neither
these waits nor filesystem removal/output writing are rewritten by this
proposal. A join that remains incomplete is recorded incomplete, never
substituted with a new signal or success.

Fresh PID readback may query only the cached own inventory with numeric-only
ps selection (pid column, no args/environment or broader process listing).
It does not discover a new target or send a process signal. A still-live PID
is not assumed safe to kill again. Fresh lsof checks only9000/3000 with the
same bounded read-only listener semantics; lab absence checks only the exact
own lab. The output retains fixed booleans/counts/exits rather than window
titles, raw process output or path contents. No HTTP/network request is used.

Nominal conditional arithmetic: returned Driver calls within30+15+5s plus
a10s final reserve fit60s; controller child cleanup can proceed concurrently.
This is a scheduling allocation, not a proven worst-case tool bound.
Three Driver calls each taking30s would already exceed60s, so a generic
30s-per-call assumption is insufficient. The smaller later allocations and
remaining-budget accounting are required for a measured success.

### What can be measured and what cannot be forced

Advertised kill_app accepts only pid. end_session accepts session; list_windows
accepts pid/on_screen_only. None advertises timeout_ms, a cancellable handle,
abort signal or an operation deadline. RUNTIME.md describes persistent MCP
ownership/lifecycle and end_session hooks, without a hard call-duration bound.
The user-requested direct kill of this explicitly owned disposable browser
overrides the usual cooperative-close preference; it does not broaden process
ownership or remove runtime authorization checks.

The outer recorder can observe a call taking more than its allocation and
mark driver_operation_over_budget after it returns. It cannot force an
unavailable MCP call to return at30s. Do not use Promise.race, isolate
termination or a discarded promise as a claim that a daemon operation was
cancelled. Do not run a second kill/end call, raw process signal, new daemon,
alternate provider or parallel uncertain GUI action to mask a stalled call.

On first unexpected result/budget exhaustion: stop all journey/ordinary GUI
actions, retain the first fixed failure, mark the60s gate failed or unknown.
Complete the remaining essential cleanup/readbacks once when the pending call
returns; do not abandon owned resources just because the observational budget
expired. That completion may be late and must be recorded as such. If the call
does not return, cleanup completion is unresolved; no immediate-success or
resource-absence receipt may be invented.

There is a second, separate limit to the phrase “first termination/cleanup
event”. The unchanged controller/helper can autonomously enter finally before
the outer worker observes an exit or issues its own cleanup trigger.
Neither source records that first actual cleanup-start clock.
completed_utc is a completion timestamp, not the start of cleanup.

Therefore keep first_event_proven false unless an actual source/evidence
record proves T0 precedes/equal the first owned termination/cleanup event.
A worker's first observed failure/exit clock is explicitly labelled observed,
not retrospectively relabelled the earliest actual event. For an autonomous
failure like aca, a60s interval from the true first cleanup event remains
unproven under this literal-only controller reservation. A later timestamp
seam would require separate root ownership; none is smuggled into this payload.

A60s-success claim requires all of: first-event anchor proven, all action
records present and monotonic, all owned child exits numeric/joined, exact
owned browser process absent and window count0, own controller/server/helper
absent as applicable, own ports unused, own lab absent, own session ended,
and final readback <=D with no allocation failure. If the first anchor is
unknown, the report may give an observed-trigger-to-absence interval but
must keep whole_cleanup_within60_proven false. This proposal does not promise
a universally enforceable60s cleanup.

### Fixed redacted prospective observation contract

The complete outer observation contract is separate from the unchanged
controller JSON/body. Any future durable file/path requires root reservation;
this phase writes none. Record fields are restricted to:

| Field | Fixed type/scope |
| --- | --- |
| schema | riauth.d01-cleanup-observation/v1 |
| cleanup_budget_ms / driver_max_observation_ms | 60000 / 30000 |
| first_event_kind | worker_cleanup_trigger, first_observed_controller_exit, first_observed_helper_exit, or unknown |
| first_event_proven | boolean; unknown/observed never becomes proven automatically |
| first_event_monotonic_ns / first_event_wall_epoch_ns | nonnegative signed64-bit integers or null |
| actions | At most4 fixed labels: kill_app, end_session, list_windows, owned_readback |
| per-action start/end clocks | Same numeric clock types; null if unavailable |
| per-action allowed_ms / elapsed_ms / over_budget | Nonnegative finite integers or null, plus boolean; never a tool timeout parameter |
| driver request/result state | returned/refused/exception/unknown finite labels; no raw message |
| owned_child_exits | Existing fixed names/PIDs/numeric exits; null means not joined, not success |
| final_absence | Fixed own PID/window/ports/lab/session booleans/counts, null when unknown |
| first_failure | null or one fixed label from the whitelist below; preserved once |
| whole_cleanup_within60_proven | boolean subject to every gate above |

Fixed failure whitelist: ownership_unknown, clock_unavailable,
clock_invalid, driver_kill_refused, driver_end_refused, driver_window_unknown,
driver_operation_over_budget, cleanup_budget_exceeded, child_reap_incomplete,
owned_resource_present, first_event_unproven. No raw headers/host/paths/query/
method/cookies/Origin/errors/sender/private values are recorded. Complete
numeric action clocks and child exits are retained before comparisons so
another lost-first-result reporting error is not repeated.

### Static proof, remaining reservation and preservation

Actually performed now: complete retained-source read; advertised schema and
installed RUNTIME/SKILL/MACOS source reads; AST parse/in-memory code-object
compile of candidate only; integer-only AST delta; full byte and normalized
AST reversal; nine function AST identity; helper470 entire byte identity.
The prospective clock-source code was parsed/compiled only, never run.
The180s assignment remains unique, tick/START+840/inclusive900/cleanup60 and
the180s no-reset checks remain source-identifiable. No lifecycle memory
execution, fixture/helper import, network/native/tool invocation or build ran.

Before any future source/runtime release, root can review this full candidate
and cleanup allocation/clock contract, decide the unsupported hard-timeout/
first-event limitations, and reserve exact fresh START/output/observation
paths. No runtime recommendation claims that180s alone cures unknown requests
or proves a confidential journey. Actual failure cleanup still takes priority
over evidence formatting; immediate truthful resource release precedes the
future report append only after joined/verified owned cleanup.

The whole270696-byte aca report remains a byte-exact prefix. All9 private
metadata records and helper470/7fbc remain unchanged, preserving aca's
deadline/helperNOTRUN/noNav/markerProcessLookup and69.766s readback limit.
All older request refusals/senderUNKNOWN, provider lost output/reporting
correction,61/76 failed memory run, dated78-case memory pass, capacity refusal
and4b/aca copy-mode handoff refusals remain retained. No input-mode workaround
or automatic handoff retry occurs in this source-only phase.

Only this existing report is appended, with static scope/prefix/archive/hash/
metadata/fences/whitespace/finalnewline/diff checks and a report-only commit.
No controllerfile/helper/product/guide/D05/config/test edit or newworker/task/
worktree/shell/main/push/status/other-worker contact. Desktop and Cargo remain
unacquired; acceptance/integration/release decisions belong to root.

## Dated actual phase: separately released 180s preparation gate, 2026-10-02

Reservation: wave30_D01_prepared_browser_single_checkpoint, latest root release
of the exact3822 archived180s controller. This was ONE fresh attempt, not a
continuation or retry of aca. Result: FAILED at the application root page after
a proven initial protected403. Desktop/operator resources were released in
commentary after joined owned cleanup and before this evidence append. No
Cargo slot was acquired or released. D01/D05 acceptance remains root-owned.

### Actual observations and stopping point

The exact Driver-bound browser first navigated to
http://localhost:3000/protected. Fresh semantic snapshot p858 showed title and
heading “Local demo” and “Sign in required.” The helper retained
protected_before403 and protected_without_cookie_denied true. Browser transport
success alone was not treated as proof of the HTTP status; that403 comes from
the helper's separate retained status/check.

The next normal application navigation to http://localhost:3000/ returned a
fresh semantic snapshot p859 with “Local demo could not complete this request.”
There was no start-form action to continue. This was the first unexpected
journey outcome; ordinary GUI actions stopped. No password was read/typed,
no sign-in, consent, callback, token exchange, userinfo or fresh protected200
was reached. No retry, replay, fallback, reprepare or rebind occurred.

Joined helper exit1 retained result failed, stage request, tag
unexpected_failure, request_invalid_reason null and
preflow_authorization_refusals0. Joined outer controller exit1 retained stage
browser_checkpoint and tag rp_nonzero. The generic failure does not identify
a cause, a failed predicate, a request sender or a product defect. No raw
exception/request/header/private value was retained, printed or investigated
in this evidence phase. Earlier Authorization senders remain UNKNOWN.
This attempt's observed0 refusals is not a counter reset or extra allowance.

The helper's credential_private_validated, discovery_verified,
provider_identity_verified and protected_without_cookie_denied checks are true.
All six subsequent journey/crypto acceptance checks are false; subsequent
authorization/callback/exchange/userinfo/protected-after statuses are null.
There is no new journey/crypto/passkey/tenant/full-D01/full-D05 success credit.

### Pre-start reads, artifact pins and actual bounded setup

Before inclusiveSTART, the cua-driver SKILL.md v0.29.1, MACOS.md and BROWSER.md
were read in full, advertised MCP descriptions were cached, and health,
permissions and read-only session state were read. Health was cua-driver0.30.4,
macOS26.2 arm64, overall ok; Accessibility and Screen Recording were granted,
direct capture was not checked and no permission/settings change occurred.
No existing Driver session was listed. No Driver history tool was advertised
or called. MCP schema supplies no hard per-call timeout/cancel.

Fresh pre-setup free capacity was22599233536 bytes, above8.5GiB.
Before executing the retained controller it was22594727936 bytes.
Both9000 and3000 listener preflights returned absent, with empty lsof outputs.
The three current evidence paths below were declared absent before any setup,
within the existing private0700 deployment-private directory. Files were
created exclusively0600; no historical metadata was overwritten.

The controller freshly rehashed the c01 Essentials artifacts from the approved
wave27 target, the helper, git-object verifier and resolved native provider.
These are identity checks, not additional source body reviews or builds:

| Input | Actual matching pin |
| --- | --- |
| c01 source provenance | c01c39ab4e092423d5522bedc50fff87656d8c0a |
| riauth | 7abf745c10691a012a1918c83089168d0e0d88b42764dbf8ed538b90c828b606 |
| riauth-maintenance | 86490c7f71de6b7ae9d4dabdf9060a8757100f3aedbdaa670284d9e1206a2a95 |
| riauthctl | bfbbb322f1a66d0ac9998beb9fb5838097cea0442cea3fd3a1d057fcb3f600cf |
| unchanged470 helper | 7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0 |
| verifier commit | 9cefe7a56425bb73c17753e8766d92320b77da3b |
| verifier blob | 3be747d03146f1bcaa3ec012ee8d173b61fa737d |
| verifier SHA256 | f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d |
| resolved /opt/homebrew/bin/openssl | 67a83dd6d6d747d50c5d296dffb23e32bae9a2c588c93ae2d77e4c607b455c72 |

The outer native version invocation exited0 within its5s bound, with63ASCII
stdout bytes and0stderr bytes. Complete numeric exit, full stdout/stderr,
lengths/hash/environment-key names were retained in the exclusive provider
metadata BEFORE exact equality:
OpenSSL 3.6.4 25 Aug 2026 (Library: OpenSSL 3.6.4 25 Aug 2026), followed by newline.
No extra diagnostic/provider invocation was made. The helper performed its
unchanged native identity/verification prerequisite; no custom crypto was added.

The exact printed CLI setup used a fresh own0700 lab and private XDG, synthetic
password through prompt stdin and the normal confidential local-demo creation
with localhost3000 callback/openid,profile scope and private0600 client secret.
No session/review/header/receipt bypass occurred. Actual fixed command records:

| Command | Exit | Password prompt writes |
| --- | --- | --- |
| maintenance_init | 0 | 2 |
| operator_login | 0 | 1 |
| confidential_client_create | 0 | 0 |
| discovery | 0 | 0 |
| whoami | 0 | 0 |

The public commands/arguments are completely archived below; generated secrets,
session values and runtime private paths are not copied into evidence.
The owned IdP listener was exactPID7637 on9000, with readyz200. No new RP
readiness HTTP request/probe was added. Initialization is operator setup,
not an application browser journey.

InclusiveSTART was1790968887.308 (2026-10-02T19:21:27.308Z). The reviewed180s
preparation gate ran once within START+840 and inclusive900, with cleanup60.
Helper deadline600, pending180, native/HTTP5, CLI60 and listener/page30 bounds
were unchanged. Disk checks retained108 outer samples and42 helper samples;
outer minimum22573965312 bytes, helper minimum22573801472 bytes (~21.023GiB).
No8.5GiB stop/floor event occurred. Final cleanup capacity22598422528 bytes.

### Exact Driver ownership/order and gate protocol

One Driver MCP session d01-budget180-checkpoint was created. ONE
browser_prepare isolated_new call returned driver-owned disposable PID9690,
window103554, new isolated profile and blank page. No profile copying,
personal-process modification, screenshot or security/settings change occurred.

The exact returned blank target was bound once:
target bt-30a830b4-18ec-48d8-aa10-1c62b2bb7dd4,
tab tab-a95fdd8c-894a-4ca0-8e12-61f505c07486.
Binding quality exact and mutation_allowed true; internal Driver transport
was provider-owned. No raw CDP or alternate provider was used by this worker.

At browser_prepare_required, controllerPID7480/serverPID7637 were live and
helper3000 had not been created. Preparation, exact window read/binding,
exact guard/server numeric PID liveness check and exclusive zero0600 own-lab
browser-prepared marker creation were followed by waiting for fixture_ready.
No description/skill load or rebind intervened. The marker writer checked only
those exact two PIDs; no browser navigation/RP probe preceded the marker.
After fixture_ready proved exact helperPID9879 listener3000, the same bound
target promptly navigated to protected and obtained fresh p858. Application
root navigation then obtained fresh p859 and stopped the journey on its
generic failure. Exactly one helper main invocation occurred.

Helper first3 stateless Authorization403/fourth terminal403, post-flow400,
Host/parser/body-target/cookie/credential/crypto/single-use guards were unchanged.
No rejected traffic is credited as a journey. No extra accepted request,
counter reset, allow-list change, alternate port, HTTP sign-in/approval,
CLI service-session substitute, physical passkey or terminal approval was used.

### Retained actual controller: pre-execution pin and full archive

Actual command14905 bytes/SHA256
6662bdbb168bf0ba6ec5ff24b6f3b88dc15b54b4dfc3b800fdaca1cbee403bbb;
body14882 bytes/193 lines/SHA256
271b4a00711a084e21792b0310839e69d4fb8779fa5e6d003bde04caf0c82c72.
Complete payload/hash were retained before execution, followed by static
AST parse/in-memory code-object compile only and full byte/AST reversal.

Reversing ONLY START=1790968887.308 to START=1790966050.847 and the two fresh
budget180 result/provider filenames to the reviewed prepared filenames
reconstructs the exact3822 command14903 bytes/SHA
3d6b4124bbc77cb82166729c3e529e08d5cc7cab28b5561c973390386cb896b7,
body14880 bytes/193 lines/SHA
e251f4e45f80c4edffd94d94a8a8ff13bb30eb8ed6113bdc96e249caed307881.
The180s assignment is the only reviewed behavior delta from the older30s
candidate. No controller file was materialized or helper/source changed.

```sh
python3 -u - <<'PY'
import datetime,fcntl,hashlib,json,os,pathlib,pty,secrets,select,shutil,signal,stat,subprocess,sys,tempfile,termios,time,urllib.request
ROOT=pathlib.Path.cwd()
START=1790968887.308
ACTIVE=START+840
BIN=pathlib.Path('/Users/dominik/orca/projects/riAuth-public-preview-local-module-boundaries-wave27/target/d01-essentials-c01c39a/aarch64-apple-darwin/debug')
PRIVATE=ROOT/'deployment-private'
OUT=PRIVATE/'d01-confidential-browser-budget180.redacted.json'
PROVIDER_META=PRIVATE/'d01-confidential-browser-budget180-provider-20261002.json'
record={'schema':'riauth.d01-confidential-browser-outer/v1','result':'failed','failure_stage':None,'failure_tag':None,'request_invalid_reason':None,'preflow_authorization_refusals':None,'commands':[],'artifacts':{},'cleanup':{},'owned_child_exits':[],'started_epoch':START,'active_seconds':840,'cleanup_seconds':60,'minimum_free_bytes':None,'disk_samples':0}
lab=None;children=[];names={};server=None;helper=None;password=None;stage='preflight';resultfd=None;lastsample=0
def require(ok,tag):
    if not ok:raise RuntimeError(tag)
def tick():
    global lastsample
    now=time.monotonic()
    if now-lastsample>=1:
        free=shutil.disk_usage(ROOT).free
        record['disk_samples']+=1
        record['minimum_free_bytes']=free if record['minimum_free_bytes'] is None else min(record['minimum_free_bytes'],free)
        lastsample=now
        require(free>=8.5*1024**3,'disk_margin')
    require(time.time()<ACTIVE,'active_deadline')
def write_exclusive(path,value):
    fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    raw=(json.dumps(value,sort_keys=True,indent=2)+'\n').encode('ascii')
    with os.fdopen(fd,'wb') as f:
        f.write(raw);f.flush();os.fsync(f.fileno())
    return hashlib.sha256(raw).hexdigest()
def listeners(port):
    p=subprocess.run(['/usr/sbin/lsof','-nP','-t','-iTCP:'+str(port),'-sTCP:LISTEN'],capture_output=True,timeout=3)
    require(p.returncode in (0,1),'socket_observation_failed')
    return set(int(v) for v in p.stdout.split())
def own(p,name):
    children.append(p);names[p.pid]=name;return p
def stop_child(p):
    if p.poll() is None:
        for sig,wait in [(signal.SIGINT,8),(signal.SIGTERM,5),(signal.SIGKILL,2)]:
            if p.poll() is not None:break
            p.send_signal(sig)
            try:p.wait(timeout=wait)
            except subprocess.TimeoutExpired:pass
    require(p.poll() is not None,'owned_child_reap_failed')
def controlling_tty():
    os.setsid();fcntl.ioctl(0,termios.TIOCSCTTY,0)
def cli(name,args,prompts=0):
    tick()
    master,slave=pty.openpty()
    p=own(subprocess.Popen(args,cwd=lab,env=environment,stdin=slave,stdout=slave,stderr=slave,preexec_fn=controlling_tty),name)
    os.close(slave);seen=0;raw=b'';started=time.monotonic()
    try:
        while p.poll() is None:
            tick();require(time.monotonic()-started<60,'cli_deadline')
            if select.select([master],[],[],0.2)[0]:
                try:chunk=os.read(master,4096)
                except OSError:chunk=b''
                raw+=chunk;require(len(raw)<=131072,'cli_output_limit')
                for prompt in ([b'Password: ',b'Confirm password: '] if prompts==2 else [b'Password: ']):
                    if seen<prompts and prompt in raw:
                        require(prompt==([b'Password: ',b'Confirm password: '][seen] if prompts==2 else b'Password: '),'cli_prompt_order')
                        os.write(master,password.encode()+b'\n');seen+=1;raw=b''
        while select.select([master],[],[],0)[0]:
            try:
                chunk=os.read(master,4096)
                if not chunk:break
                raw+=chunk
            except OSError:break
        code=p.wait()
        record['commands'].append({'name':name,'exit':code,'password_prompts':seen})
        require(code==0,'cli_nonzero');require(seen==prompts,'cli_prompt_missing')
    finally:
        raw=b'';os.close(master)
        if p.poll() is None:stop_child(p)
def wait_listener(p,port):
    deadline=time.monotonic()+30
    while time.monotonic()<deadline:
        tick();require(p.poll() is None,'owned_service_early_exit')
        owners=listeners(port)
        if owners:
            require(owners=={p.pid},'listener_owner_mismatch');return
        time.sleep(0.2)
    raise RuntimeError('listener_deadline')
try:
    tick()
    require(PRIVATE.is_dir() and not PRIVATE.is_symlink() and stat.S_IMODE(PRIVATE.stat().st_mode)==0o700,'private_directory_invalid')
    require(not OUT.exists() and not PROVIDER_META.exists(),'evidence_already_exists')
    require(not listeners(9000) and not listeners(3000),'port_occupied');record['ports_preflight_empty']=True
    pins={'riauth':'7abf745c10691a012a1918c83089168d0e0d88b42764dbf8ed538b90c828b606','riauth-maintenance':'86490c7f71de6b7ae9d4dabdf9060a8757100f3aedbdaa670284d9e1206a2a95','riauthctl':'bfbbb322f1a66d0ac9998beb9fb5838097cea0442cea3fd3a1d057fcb3f600cf'}
    for name,pin in pins.items():
        tick()
        with (BIN/name).open('rb') as f:digest=hashlib.file_digest(f,'sha256').hexdigest()
        record['artifacts'][name]=digest;require(digest==pin,'artifact_hash_mismatch')
    helper_path=ROOT/'scripts/d01-confidential-browser-demo.py'
    require(hashlib.sha256(helper_path.read_bytes()).hexdigest()=='7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0','helper_hash_mismatch')
    verifier=subprocess.check_output(['git','show','9cefe7a56425bb73c17753e8766d92320b77da3b:scripts/recovery-drill-oidc.py'],timeout=5)
    require(hashlib.sha256(verifier).hexdigest()=='f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d','verifier_hash_mismatch')
    provider=pathlib.Path('/opt/homebrew/bin/openssl').resolve(strict=True)
    with provider.open('rb') as f:provider_hash=hashlib.file_digest(f,'sha256').hexdigest()
    require(provider_hash=='67a83dd6d6d747d50c5d296dffb23e32bae9a2c588c93ae2d77e4c607b455c72','provider_hash_mismatch')
    provider_env={k:v for k,v in os.environ.items() if k in {'PATH','HOME','TMPDIR','LANG','LC_ALL'}}
    p=subprocess.Popen([str(provider),'version'],stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=provider_env)
    try:stdout,stderr=p.communicate(timeout=5);timeout=False
    except subprocess.TimeoutExpired:
        p.kill();stdout,stderr=p.communicate(timeout=2);timeout=True
    provider_record={'sha256':provider_hash,'exit':p.returncode,'timeout':timeout,'stdout_ascii':stdout[:4096].decode('ascii',errors='backslashreplace'),'stderr_ascii':stderr[:4096].decode('ascii',errors='backslashreplace'),'stdout_bytes':len(stdout),'stderr_bytes':len(stderr),'environment_keys':sorted(provider_env)}
    record['provider_metadata_sha256']=write_exclusive(PROVIDER_META,provider_record)
    require(not timeout and p.returncode==0 and len(stdout)<=256 and len(stderr)<=4096 and stdout.decode('ascii').strip()=='OpenSSL 3.6.4 25 Aug 2026 (Library: OpenSSL 3.6.4 25 Aug 2026)','provider_version_failed')
    tick()
    resultfd=os.open(OUT,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    lab=pathlib.Path(tempfile.mkdtemp(prefix='d01-confidential-browser-diagnostic.',dir=PRIVATE));os.chmod(lab,0o700)
    for name in ('xdg','deployment-private','rp'):(lab/name).mkdir(mode=0o700)
    (lab/'recovery-drill-oidc.py').write_bytes(verifier);os.chmod(lab/'recovery-drill-oidc.py',0o600);verifier=None
    password=secrets.token_urlsafe(30)
    fd=os.open(lab/'browser-password',os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
    with os.fdopen(fd,'w') as f:f.write(password)
    environment=dict(provider_env);environment['XDG_CONFIG_HOME']=str(lab/'xdg')
    stage='init';cli('maintenance_init',[str(BIN/'riauth-maintenance'),'--config',str(lab/'riauth.toml'),'init','--issuer','http://localhost:9000','--listen','127.0.0.1:9000','--data-dir','data','--admin','admin'],2)
    stage='serve'
    server=own(subprocess.Popen([str(BIN/'riauth'),'--config',str(lab/'riauth.toml'),'serve'],cwd=lab,env=environment,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL),'server')
    record['server_pid']=server.pid;wait_listener(server,9000);record['server_listener_owned']=True
    stage='readyz'
    with urllib.request.urlopen('http://127.0.0.1:9000/readyz',timeout=5) as response:
        record['readyz_status']=response.status;response.read(4096)
    require(record['readyz_status']==200,'readyz_failed')
    stage='cli_login'
    base=[str(BIN/'riauthctl'),'--server','http://localhost:9000']
    cli('operator_login',base+['login','admin'],1)
    stage='client_create';secretpath=lab/'deployment-private/local-demo-secret.json'
    cli('confidential_client_create',base+['client','create','local-demo','--name','Local demo','--confidential','--redirect-uri','http://localhost:3000/callback','--scope','openid,profile','--secret-file',str(secretpath)])
    require(stat.S_IMODE(secretpath.stat().st_mode)==0o600,'cli_secret_mode_invalid')
    stage='discovery';cli('discovery',base+['discovery'])
    stage='whoami';cli('whoami',base+['whoami'])
    stage='browser_prepare'
    prepare_deadline=time.monotonic()+180
    prepare_marker=lab/'browser-prepared'
    print(json.dumps({'browser_prepare_required':True,'guard_pid':os.getpid(),'server_pid':server.pid,'lab':str(lab)}),flush=True)
    while not prepare_marker.exists():
        tick();require(server.poll() is None,'idp_early_exit')
        require(not (lab/'ui-failure').exists() and not (lab/'stop').exists(),'browser_checkpoint_failed')
        require(time.monotonic()<prepare_deadline,'browser_prepare_deadline')
        time.sleep(0.2)
    tick();require(server.poll() is None,'idp_early_exit')
    require(not (lab/'ui-failure').exists() and not (lab/'stop').exists(),'browser_checkpoint_failed')
    require(time.monotonic()<prepare_deadline,'browser_prepare_deadline')
    prepare_info=prepare_marker.lstat()
    require(stat.S_ISREG(prepare_info.st_mode) and stat.S_IMODE(prepare_info.st_mode)==0o600 and prepare_info.st_uid==os.getuid() and prepare_info.st_nlink==1 and prepare_info.st_size==0,'browser_prepare_marker_invalid')
    stage='rp_start'
    helper=own(subprocess.Popen([sys.executable,'-B',str(helper_path),'--workspace',str(lab/'rp'),'--secret-file',str(secretpath),'--verifier-helper',str(lab/'recovery-drill-oidc.py'),'--openssl','/opt/homebrew/bin/openssl','--deadline-seconds','600','--evidence',str(lab/'rp-result.json')],cwd=ROOT,env=environment,stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,text=True),'helper')
    record['helper_pid']=helper.pid;record['helper_invocations']=1
    wait_listener(helper,3000);record['helper_listener_owned']=True
    print(json.dumps({'fixture_ready':True,'guard_pid':os.getpid(),'server_pid':server.pid,'helper_pid':helper.pid,'lab':str(lab),'commands':record['commands'],'provider_metadata_sha256':record['provider_metadata_sha256'],'minimum_free_bytes':record['minimum_free_bytes']}),flush=True)
    stage='browser_checkpoint';announced=False
    while not (lab/'stop').exists():
        tick();require(server.poll() is None,'idp_early_exit')
        if helper.poll() is not None:
            code=helper.wait()
            if not announced:
                record['helper_exit']=code;record['helper_result']=json.loads((lab/'rp-result.json').read_bytes())
                record['request_invalid_reason']=record['helper_result']['request_invalid_reason']
                record['preflow_authorization_refusals']=record['helper_result']['preflow_authorization_refusals']
                print(json.dumps({'helper_completed':True,'exit':code,'result':record['helper_result']['result'],'failure_tag':record['helper_result']['failure_tag'],'request_invalid_reason':record['request_invalid_reason'],'preflow_authorization_refusals':record['preflow_authorization_refusals']}),flush=True);announced=True
            require(code==0,'rp_nonzero')
        time.sleep(0.2)
    if (lab/'ui-failure').exists():raise RuntimeError('browser_checkpoint_failed')
    require(helper.poll()==0,'rp_checkpoint_incomplete');record['result']='passed'
except Exception as error:
    record['failure_stage']=stage
    tags={'disk_margin','active_deadline','private_directory_invalid','evidence_already_exists','port_occupied','socket_observation_failed','artifact_hash_mismatch','helper_hash_mismatch','verifier_hash_mismatch','provider_hash_mismatch','provider_version_failed','cli_deadline','cli_output_limit','cli_prompt_order','cli_prompt_missing','cli_nonzero','owned_service_early_exit','listener_owner_mismatch','listener_deadline','readyz_failed','cli_secret_mode_invalid','idp_early_exit','rp_nonzero','browser_checkpoint_failed','rp_checkpoint_incomplete','browser_prepare_deadline','browser_prepare_marker_invalid'}
    record['failure_tag']=str(error) if str(error) in tags else 'outer_unexpected_failure'
finally:
    for p in reversed(children):
        try:stop_child(p)
        except Exception:record['cleanup']['child_reap_failure']=True
        record['owned_child_exits'].append({'name':names[p.pid],'pid':p.pid,'exit':p.poll()})
    record['cleanup']['owned_children_reaped']=all(p.poll() is not None for p in children)
    if lab is not None:
        if helper is not None:
            record['helper_exit']=helper.poll()
            if (lab/'rp-result.json').exists():
                record['helper_result']=json.loads((lab/'rp-result.json').read_bytes())
                record['request_invalid_reason']=record['helper_result']['request_invalid_reason']
                record['preflow_authorization_refusals']=record['helper_result']['preflow_authorization_refusals']
        password=None;shutil.rmtree(lab);record['cleanup']['lab_removed']=not lab.exists()
    else:record['cleanup']['lab_removed']=True
    try:
        record['cleanup']['port9000_absent']=not listeners(9000);record['cleanup']['port3000_absent']=not listeners(3000)
    except Exception:record['cleanup']['socket_observation_failure']=True
    record['completed_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();record['elapsed_seconds']=round(time.time()-START,3)
    if resultfd is None and not OUT.exists():resultfd=os.open(OUT,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    if resultfd is not None:
        with os.fdopen(resultfd,'wb') as f:
            f.write((json.dumps(record,sort_keys=True,indent=2)+'\n').encode('ascii'));f.flush();os.fsync(f.fileno())
    print(json.dumps({'fixture_finished':True,'result':record['result'],'failure_stage':record['failure_stage'],'failure_tag':record['failure_tag'],'request_invalid_reason':record['request_invalid_reason'],'preflow_authorization_refusals':record['preflow_authorization_refusals'],'cleanup':record['cleanup'],'owned_child_exits':record['owned_child_exits'],'minimum_free_bytes':record['minimum_free_bytes'],'disk_samples':record['disk_samples'],'elapsed_seconds':record['elapsed_seconds']}),flush=True)
sys.exit(0 if record['result']=='passed' else 1)
PY```

### Actual cleanup, precise evidence and timing limits

Cleanup ran as one contiguous orchestration call: record local clock latch;
use original own-lab ui-failure/stop marker protocol if the lab still exists;
immediate Driver kill_app exact proven-owned PID9690 ONCE; end_session exact
session ONCE; list_windows exactPID9690 ONCE; join original controller;
numeric-only exact own PID/9000/3000/lab absence readback; exclusive cleanup
metadata write/fsync. No cooperative Cmd+Q, manual browser signal, unrelated
kill, extra cleanup attempt or replacement provider was used.

The original controller autonomously joined helper9879 exit1 and IdP7637 exit0
and its other five children, removing its own lab/listeners. The worker joined
controller exit1 and independently read back own guard7480/server7637/
helper9879/browser9690 absent, exact ports9000/3000 absent, lab absent,
zero Driver browser windows and session ended. All seven child exit values
are retained below before comparisons. Immediate DESKTOP/OPERATOR RELEASE
was sent after those checks and before this append.

| Observed action | Allocation ms | Retained elapsed ms | Result |
| --- | --- | --- | --- |
| kill_app | 30000 | 600 | returned |
| end_session | 15000 | 638 | returned |
| list_windows | 5000 | 591 | returned |
| owned_readback | 55820 remaining | 986 | returned |

No call exceeded its observational allocation. MCP has no advertised hard
timeout/cancel, so these actual timings are not a future enforcement guarantee.
The observed worker-trigger-to-final-absence interval was5165ms by the retained
shared monotonic clocks. first_event_proven is false; first_failure is
first_event_unproven and whole_cleanup_within60_proven is false.

The outer controller completed at19:23:18.594564Z, elapsed111.287s.
The later worker cleanup-trigger wall clock was about19:25:12.386914Z; final
absence readback about19:25:17.551883Z, roughly118.957s after outer completion
and230.244s after inclusiveSTART. This gap is explicitly retained. The5.165s
contiguous cleanup interval cannot be used to erase that delay or prove the
requested whole60s bound. No exact first unexpected UI observation clock or
first actual autonomous cleanup-start clock was recorded; no such old clock
is invented. Essential owned cleanup finished even though whole60s success
cannot be claimed.

Clock sources were Python time.monotonic_ns/time.time_ns. Clock records were
retained before comparisons. Monotonic values here are below2^53 and survive
JavaScript JSON-number transport exactly. Wall epoch nanoseconds exceed2^53;
JavaScript Number transport rounds their low-order nanoseconds. The durable
wall integers are retained unchanged, but do not carry exact nanosecond
precision. Timing arithmetic for action/remaining budgets uses monotonic
values. No correction/rewrite of existing metadata is made.

### Exclusive current redacted metadata and full fixed results

All three new files remain deployment-private0600. These were absent before
setup, written exclusively and read back after cleanup. They are not git
tracked; no raw secret/request/header/query/token/cookie/password was stored.
The first lost provider invocation and incorrect historical report are still
retained in the prior dated phases rather than overwritten by these results.

| New file | Bytes | SHA256 |
| --- | --- | --- |
| d01-confidential-browser-budget180-cleanup.redacted.json | 2612 | 2ac06f21da0aff2874094788cd227397f3a67b64dafa2a434e5d5a6e8ba47dda |
| d01-confidential-browser-budget180-provider-20261002.json | 359 | 6fad035f1897b5287b2ebd812405fcabf6a33b72d2d4f498f3bca7081eb719ab |
| d01-confidential-browser-budget180.redacted.json | 4099 | 83549d45aec2900fae09b1ec2ea4bd7792945a5508bde085568b59506a08cd9f |

Complete outer/helper fixed result:

```json
{
  "active_seconds": 840,
  "artifacts": {
    "riauth": "7abf745c10691a012a1918c83089168d0e0d88b42764dbf8ed538b90c828b606",
    "riauth-maintenance": "86490c7f71de6b7ae9d4dabdf9060a8757100f3aedbdaa670284d9e1206a2a95",
    "riauthctl": "bfbbb322f1a66d0ac9998beb9fb5838097cea0442cea3fd3a1d057fcb3f600cf"
  },
  "cleanup": {
    "lab_removed": true,
    "owned_children_reaped": true,
    "port3000_absent": true,
    "port9000_absent": true
  },
  "cleanup_seconds": 60,
  "commands": [
    {
      "exit": 0,
      "name": "maintenance_init",
      "password_prompts": 2
    },
    {
      "exit": 0,
      "name": "operator_login",
      "password_prompts": 1
    },
    {
      "exit": 0,
      "name": "confidential_client_create",
      "password_prompts": 0
    },
    {
      "exit": 0,
      "name": "discovery",
      "password_prompts": 0
    },
    {
      "exit": 0,
      "name": "whoami",
      "password_prompts": 0
    }
  ],
  "completed_utc": "2026-10-02T19:23:18.594564+00:00",
  "disk_samples": 108,
  "elapsed_seconds": 111.287,
  "failure_stage": "browser_checkpoint",
  "failure_tag": "rp_nonzero",
  "helper_exit": 1,
  "helper_invocations": 1,
  "helper_listener_owned": true,
  "helper_pid": 9879,
  "helper_result": {
    "checks": {
      "authorization_redirect_issued": false,
      "confidential_s256_exchange_verified": false,
      "credential_private_validated": true,
      "discovery_verified": true,
      "protected_with_fresh_cookie_accepted": false,
      "protected_without_cookie_denied": true,
      "provider_identity_verified": true,
      "rs256_jwks_issuer_audience_nonce_time_access_hash_verified": false,
      "state_issuer_flow_cookie_verified": false,
      "userinfo_subject_verified": false
    },
    "cleanup": {
      "connection_closed": true,
      "listener_closed": true,
      "private_references_cleared": true,
      "verifier_temporaries_removed": true
    },
    "disk_samples": 42,
    "elapsed_seconds": 10.096,
    "failure_stage": "request",
    "failure_tag": "unexpected_failure",
    "http_statuses": {
      "authorization_redirect": null,
      "callback": null,
      "protected_after": null,
      "protected_before": 403,
      "token_exchange": null,
      "userinfo": null
    },
    "minimum_free_bytes": 22573801472,
    "preflow_authorization_refusals": 0,
    "provider": {
      "sha256": "67a83dd6d6d747d50c5d296dffb23e32bae9a2c588c93ae2d77e4c607b455c72",
      "version": "OpenSSL 3.6.4 25 Aug 2026 (Library: OpenSSL 3.6.4 25 Aug 2026)"
    },
    "request_invalid_reason": null,
    "result": "failed",
    "schema": "riauth.d01-confidential-browser/v1",
    "source": {
      "helper_sha256": "7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0",
      "verifier_blob": "3be747d03146f1bcaa3ec012ee8d173b61fa737d",
      "verifier_commit": "9cefe7a56425bb73c17753e8766d92320b77da3b",
      "verifier_expected_sha256": "f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d",
      "verifier_sha256": "f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d"
    }
  },
  "minimum_free_bytes": 22573965312,
  "owned_child_exits": [
    {
      "exit": 1,
      "name": "helper",
      "pid": 9879
    },
    {
      "exit": 0,
      "name": "whoami",
      "pid": 7715
    },
    {
      "exit": 0,
      "name": "discovery",
      "pid": 7712
    },
    {
      "exit": 0,
      "name": "confidential_client_create",
      "pid": 7707
    },
    {
      "exit": 0,
      "name": "operator_login",
      "pid": 7683
    },
    {
      "exit": 0,
      "name": "server",
      "pid": 7637
    },
    {
      "exit": 0,
      "name": "maintenance_init",
      "pid": 7567
    }
  ],
  "ports_preflight_empty": true,
  "preflow_authorization_refusals": 0,
  "provider_metadata_sha256": "6fad035f1897b5287b2ebd812405fcabf6a33b72d2d4f498f3bca7081eb719ab",
  "readyz_status": 200,
  "request_invalid_reason": null,
  "result": "failed",
  "schema": "riauth.d01-confidential-browser-outer/v1",
  "server_listener_owned": true,
  "server_pid": 7637,
  "started_epoch": 1790968887.308
}
```

Complete native provider metadata, retained before equality:

```json
{
  "environment_keys": [
    "HOME",
    "LANG",
    "LC_ALL",
    "PATH",
    "TMPDIR"
  ],
  "exit": 0,
  "sha256": "67a83dd6d6d747d50c5d296dffb23e32bae9a2c588c93ae2d77e4c607b455c72",
  "stderr_ascii": "",
  "stderr_bytes": 0,
  "stdout_ascii": "OpenSSL 3.6.4 25 Aug 2026 (Library: OpenSSL 3.6.4 25 Aug 2026)\n",
  "stdout_bytes": 63,
  "timeout": false
}
```

Complete cleanup observation, including numeric clocks/exits before comparisons:

```json
{
  "actions": [
    {
      "allowed_ms": 30000,
      "elapsed_ms": 600,
      "end_monotonic_ns": 2466445222762625,
      "end_wall_epoch_ns": 1790969113925337000,
      "label": "kill_app",
      "over_budget": false,
      "result_state": "returned",
      "start_monotonic_ns": 2466444622279541,
      "start_wall_epoch_ns": 1790969113324955000
    },
    {
      "allowed_ms": 15000,
      "elapsed_ms": 638,
      "end_monotonic_ns": 2466446328554375,
      "end_wall_epoch_ns": 1790969115030950000,
      "label": "end_session",
      "over_budget": false,
      "result_state": "returned",
      "start_monotonic_ns": 2466445690294791,
      "start_wall_epoch_ns": 1790969114392790000
    },
    {
      "allowed_ms": 5000,
      "elapsed_ms": 591,
      "end_monotonic_ns": 2466447394085083,
      "end_wall_epoch_ns": 1790969116096320000,
      "label": "list_windows",
      "over_budget": false,
      "result_state": "returned",
      "start_monotonic_ns": 2466446802951000,
      "start_wall_epoch_ns": 1790969115505273000
    },
    {
      "allowed_ms": 55820,
      "elapsed_ms": 986,
      "end_monotonic_ns": 2466448849850000,
      "end_wall_epoch_ns": 1790969117551883000,
      "label": "owned_readback",
      "over_budget": false,
      "result_state": "returned",
      "start_monotonic_ns": 2466447863624416,
      "start_wall_epoch_ns": 1790969116565792000
    }
  ],
  "cleanup_budget_ms": 60000,
  "driver_max_observation_ms": 30000,
  "final_absence": {
    "lab": true,
    "owned_pids": {
      "7480": true,
      "7637": true,
      "9690": true,
      "9879": true
    },
    "ports": {
      "3000": true,
      "9000": true
    },
    "session_ended": true,
    "window_count": 0
  },
  "first_event_kind": "worker_cleanup_trigger",
  "first_event_monotonic_ns": 2466443684070333,
  "first_event_proven": false,
  "first_event_wall_epoch_ns": 1790969112386914000,
  "first_failure": "first_event_unproven",
  "owned_child_exits": [
    {
      "exit": 1,
      "name": "helper",
      "pid": 9879
    },
    {
      "exit": 0,
      "name": "whoami",
      "pid": 7715
    },
    {
      "exit": 0,
      "name": "discovery",
      "pid": 7712
    },
    {
      "exit": 0,
      "name": "confidential_client_create",
      "pid": 7707
    },
    {
      "exit": 0,
      "name": "operator_login",
      "pid": 7683
    },
    {
      "exit": 0,
      "name": "server",
      "pid": 7637
    },
    {
      "exit": 0,
      "name": "maintenance_init",
      "pid": 7567
    }
  ],
  "schema": "riauth.d01-cleanup-observation/v1",
  "whole_cleanup_within60_proven": false
}
```

### Prefix/scope checks and candid handoff

The entire3822 report304690-byte prefix/SHA
e2744d8d26a6d885dbdfbc43264a64d8886d365c11f63802b24c192cd7c5d617 is
preserved byte-exact. All nine older0600 metadata files and all three new0600
files retain their exact recorded byte counts/hashes. Helper470 retains its
entire immutable byte identity and SHA7fbc. Post-append archive equality,
fresh-literal-only byte/AST reversal, metadata mode/hash/count, Markdown fence,
final-newline, whitespace and report-only diff checks are performed; no new
test, build, runtime, browser action or source correction accompanies them.

Preserved historical phases include the lost provider output/report correction,
all older request refusals/senderUNKNOWN, failed61/76 memory run, dated78/78
memory pass with44baseline/80candidate calls, capacity refusal/notstarted,
500b no journey, aca helperNOTRUN/preparation deadline/marker ProcessLookup
and69.766s absence-readback limit. Their dates/outcomes are not refreshed.
Memory sinks and this initial403 are not confidential browser journey proof.

The concrete remaining checkpoint is still the real confidential application's
printed password sign-in/Local demo consent/callback/validated token-userinfo/
fresh protected200. This attempt cannot proceed past its generic application
failure. No speculative cause/fix, source investigation or retry is authorized
or performed here. Root decides any separately reserved diagnosis/correction
and original D01/D05 disposition. No universal install/HA/release/tenant/
physical-passkey or ordinary-user/invitation/application coverage is claimed.

Only this existing report is appended and committed. No helper/controller
file/product/guide/D05/config/test edit, main/merge/push/status, new worker/task/
WT/shell or other-worker contact occurred. No Cargo lane was touched.
Desktop/operator cleanup is complete and released; all future runtime remains
HELD pending a separate root decision. Historical copy-mode handoff refusals
are retained; no input-mode workaround or automatic handoff retry is made.

## Dated design phase: immediate observation and owned cleanup, 2026-10-02

Reservation wave30_D01_immediate_observation_cleanup_plan owns ONLY this
append. This is a prospective orchestration design, not a source implementation,
fixture release or run. No desktop/operator/Cargo slot is acquired or released.
The complete c88 report and all twelve retained private records remain dated
and unchanged. In particular, protected403/generic unexpected_failure/zeroAuth,
no journey and final absence about119s after controller completion remain the
actual c88 outcome. The earlier60s cleanup limits are not repaired retroactively.

The ONE proposed seam moves the application-root navigation, fresh semantic
observation and conditional cleanup into a single composed functions.exec cell.
The failed observation is projected to fixed booleans/labels, its return wall
clock is latched synchronously, and the first failure is stored before any new
await, text, notification, yield_control, report formatting or model turn.
That same cell immediately requests the original own-lab stop protocol and
finishes essential owned cleanup before returning its redacted failure result.
There is no model decision between the known failed observation and cleanup.

### Exact future sequence and deliberate limits

Pre-start descriptions/skills/schema/health and the original fixture preparation
remain outside this observation cell. Future root release must still declare
fresh absent0600 observation/output paths, freshly match artifact/provider
pins, measure capacity, prepare/bind one isolated_new Driver target at the180s
gate, create its exact private marker after guard/server liveness, and prove
fixture_ready/exact own helper3000 plus the initial browser protected403.
The archived cell starts at the next normal application-root navigation.
It does not launch a controller/helper/browser or repeat setup.

1. Read ONLY the original controller's exact exec session once. Retain received
   wall time and numeric exit projection before comparisons; parse its bounded
   fixed helper_completed/fixture_finished events. A helper failure or controller
   completion before this app checkpoint latches the first fixed failure.
2. If still live, navigate the same exact Driver target/tab to the existing
   public application root; a returned refusal/exception latches a fixed failure.
3. If still live, obtain one fresh semantic_v2 snapshot with no screenshot.
   Retain only fixed flags. The known generic failure phrase triggers immediate
   application_page_failed. Unknown/incomplete status, missing Local demo
   heading or missing interactive click ref fails closed as unconfirmed; it is
   not product blame or a success inference. A passing projection proves only
   this observation, not sign-in/consent/callback/crypto/fresh protected200.
4. If still live, read the original controller session once more. A helper
   failure/completion or controller completion also enters the same path.
5. On the first failed observation, persist the observation clock in one new
   exclusive0600 file and write only the original own-lab ui-failure/stop zero
   markers if that exact owned lab still exists. Do not recreate a removed lab.
   Metadata-write failure is separate and does not skip the original stop.
6. Immediately kill only the proven Driver-owned browser PID once, end its
   exact session once, then list only that PID's windows once. The unchanged
   controller concurrently runs its original owned-child finally after stop.
7. Join ONLY its original exec session, observe exact cached own PIDs and
   9000/3000/lab absence, read its original redacted numeric child exits, retain
   clocks/exits before comparisons, and write one new exclusive cleanup record.
   No unrelated PID/signal, direct browser signal, replacement service or extra
   cleanup attempt is introduced. No file deletion occurs in the new collectors.
8. Return only the fixed failure and cleanup projection. A truthful resource
   release requires the numeric original controller join, seven child records,
   own PID/listener/lab absence, zero own windows and ended session. A missing
   proof is returned as unresolved; no success/release receipt is invented.

Controller polling is output observation, not another RP HTTP request/probe.
It uses the advertised empty write_stdin polling yield5000; that is not a
process deadline. A failed poll is not automatically retried: further join
polls are disabled and the result remains unproven. Normal running-session
polls continue essential joining, even after the nominal60s cleanup allocation,
up to the existing inclusive900 wall deadline as observed between calls.
No journey continues after a failure. No Promise.race/discarded promise,
isolate termination or unavailable cancellation is used as proof of stopping
an in-flight operation.

All Driver calls are awaited and serialized. Consequently this design can
react to a helper/controller event when the exact poll returns, and to a UI
failure when the exact navigation/snapshot returns. It cannot observe the
controller while blocked in an unbounded Driver call, or run a watchdog after
the composed cell has returned to the model. Before/after polling bounds
ordinary gaps by actual tool returns, not a new universal detection guarantee.
An unreturned MCP operation remains unresolved; no overlapping Driver kill,
second controller or force interruption is invented.

The archived cell's stage predicate is application-root only. The same
latch-before-yield rule must wrap each later, separately reviewed printed
journey step; this archive does not invent those stage predicates or a whole
scripted browser journey. Root can reserve such a concrete composition with
its existing journey steps after source review. This seam does not diagnose
the helper failure or substitute a service/API session for browser behavior.

### Owned context and exact advertised calls

The future single cell reads d01_immediate_owned_handles from the same isolate
session store, created ONLY from that future attempt's returned preparation/
binding/fixture-ready receipts. Required values are:

| Field | Exact meaning |
| --- | --- |
| workspace | This existing WT's absolute path |
| runtime_release / driver_owned / exact_bound / single_controller | All true only under a fresh root release and returned ownership/binding proof |
| fixture_ready / listener_pid_proof / fresh_paths_preflight | Fresh true receipts, exact own9000/3000 PIDs and absent output paths |
| guard_pid / server_pid / helper_pid / browser_pid | Four distinct positive exact returned own PIDs |
| exec_session | Original controller's exact exec_command session ID; never another task |
| session / target_id / tab_id | Exact returned Driver-owned lifecycle/target/tab identifiers |
| start_epoch_ms | Once-started original inclusiveSTART in exact safe integer milliseconds |
| lab / outer_out | Exact fresh own lab and unchanged controller's fixed result path |
| event_out / cleanup_out | Two NEW absent deployment-private paths reserved before a future run |

Prospective path names for root reservation are
deployment-private/d01-confidential-browser-immediate-observation.redacted.json
and deployment-private/d01-confidential-browser-immediate-cleanup.redacted.json.
Neither path is created or checked by a runtime invocation in this phase.
Fresh controller/provider outputs are also a future root decision, not reuse
of any historical output. Exclusive O_EXCL/O_NOFOLLOW protects existing records.

The c88 historical context was guard7480/server7637/helper9879/browser9690,
window103554, original exec session16673 and Driver session
d01-budget180-checkpoint with target
bt-30a830b4-18ec-48d8-aa10-1c62b2bb7dd4 and tab
tab-a95fdd8c-894a-4ca0-8e12-61f505c07486.
Those are past identifiers, all already absent/ended. The source intentionally
does not reuse them: future context must be fresh exact returned identifiers.
A missing ownership context refuses the proposal; it grants no right to kill
an arbitrary/personal process. The exact fixture lifetime is the ownership
scope, not a broad process-name/argument/environment search.

Actually reread advertised descriptions in this design phase:
browser_navigate(session,target_id,tab_id,url);
get_browser_state(session,target_id,tab_id,snapshot_format,include_screenshot);
kill_app(pid); end_session(session); list_windows(pid).
No timeout/cancel parameter is advertised. No Driver command/session/state/
health/browser call ran. No alternate provider/CLI Driver/CDP/Playwright/
AppleScript/screenshot/input-mode workaround is introduced.

The direct proven-owned disposable kill is the user's explicit cleanup choice;
it does not widen the normal skill's cooperative-close or process ownership
rules. The exact local collectors use only stdlib, original marker writes,
numeric-only ps/lsof observations, the original fixed redacted result and
exclusive evidence writes. They neither import/execute the helper nor invoke
a provider, CLI, API, browser/sign-in substitute or native crypto.

### Clock and failure contract: no retrospective first-event claim

The first tool-return wall clock is Date.now, a safe integer in milliseconds,
latched synchronously in the same composed call. Its received-time flag/exit
projection is retained before any outcome comparison. The first failure label
is write-once; subsequent controller events/cleanup errors cannot overwrite it.

Before any Driver cleanup, the local collector captures shared-host
time.monotonic_ns/time.time_ns, attempts exclusive first-observation persistence
BEFORE marker/state/budget comparisons, and returns the clock plus finite
marker/evidence state. The synchronous observed wall clock precedes this local
monotonic latch; tool invocation/bookkeeping latency is not silently discarded.
No model/report delay is inserted between the two.

Clocks cross JavaScript as bounded decimal strings; final persistence converts
them directly to Python signed64-bit integers. This prospective encoding avoids
new JS Number rounding of wall epoch nanoseconds. It does not rewrite c88's
already rounded wall integers or any old record. Per-action start is retained
before its Driver call; returned end/result is retained before elapsed/budget
comparison. Exact shared monotonic durations cover collector latch through
final readback. Received-observation-to-final wall milliseconds are recorded
separately and remain subject to wall-clock adjustment.

The observed first event is still an OBSERVATION, not necessarily the earliest
actual helper/controller cleanup event. No controller timing insertion is
proposed. first_event_proven and whole_cleanup_within60_proven remain false
in this archive, even if the observed interval is under60s. A future true
first-event proof would need separately reserved source evidence. Historic
unknown/null anchors and c88's119s delayed readback remain unchanged.

The fixed observational allocations are Driver kill30s, end15s, windows5s,
then nonnegative remaining-to60 for original join/readback. No allocation is
passed as a fictitious MCP timeout. Actual over-allocation is recorded only
after an awaited call returns. Essential cleanup is continued if late.
Local exec yield10000 is also a tool return/yield observation, not a hard
command deadline; a still-running collector yields local_pending/unconfirmed
rather than being treated as completed. Numeric collector exit/session
receipts are retained before comparison; an unjoined collector suppresses the
resource-release projection even if the cached primary PIDs are absent. Its
handle is retained as unresolved, with no retry/cancellation claim. The
cleanup-file writer's own returned numeric receipt stays in the session store
and final projection; its already-written file is not overwritten to record
its own post-write exit. The bounded ps/lsof subprocess calls
use their real3s stdlib timeouts; they are readbacks, not process signaling.

First journey/observation failure whitelist:
controller_observation_invalid, helper_failed,
helper_completed_before_app_checkpoint,
controller_completed_before_app_checkpoint,
controller_observation_unavailable, driver_navigation_refused,
driver_navigation_exception, application_page_failed,
application_snapshot_unconfirmed, driver_snapshot_exception.

Cleanup errors are a separate finite list:
clock_unavailable, clock_invalid, driver_operation_over_budget,
driver_operation_unconfirmed, ownership_unknown, stop_unconfirmed,
observation_metadata_write_unconfirmed, stop_or_clock_record_unavailable,
controller_join_unconfirmed, cleanup_budget_exceeded, child_reap_incomplete,
owned_resource_absence_unproven, owned_readback_unavailable,
first_event_unproven, cleanup_metadata_write_unconfirmed,
owned_local_command_unjoined.
The local exceptions are suppressed to fixed labels; no raw message is stored.
No request/header/Host/Origin/path/query/method/cookie/token/credential/subject/
sender is written. Browser snapshot evidence is fixed booleans only, not raw
outline/names/values/URLs. Controller observations are fixed flags/numeric exits
from its bounded stdout; canonical outer/helper/provider records stay intact.

The new record has fixed schema, first-failure/received clock, false anchor
proof, clocks/actions/error list, numeric controller/child exits, exact owned
resource booleans/counts and separate observed durations. A resource-release
projection never grants journey/crypto credit. Complete existing failure
metadata is preserved rather than replaced by this smaller projection.

### Source pins and full unchanged-controller proof

No prospective controller insertion is needed. The approved command remains
14905 bytes/SHA256
6662bdbb168bf0ba6ec5ff24b6f3b88dc15b54b4dfc3b800fdaca1cbee403bbb,
193-line body14882 bytes/SHA256
271b4a00711a084e21792b0310839e69d4fb8779fa5e6d003bde04caf0c82c72.
The complete actual command is already archived in c88 above. Entire byte
identity and entire AST identity are zero-delta reversal, including all nine
functions. No line is added to that controller. Future fresh START/two result
names require the existing literal-only reversal before any invocation.

Unchanged controller guards: START+840 active/inclusive900/cleanup60;
prepare180 once/no reset; helper600/pending180; native/HTTP5/CLI60;
owned listener/page30;1s disk8.5stop before8floor. The original helper470 SHA
7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0,
refusal count4/first3 stateless403/fourthterminal403/postflow400,
Host/parser/body/cookie/secret/state/nonce/S256/crypto/accepted request behavior
remain byte-identical. No helper body diagnosis or ownership is taken here.

The original c01 three binary hashes, verifier commit/blob/SHAf6dd and resolved
providerSHA67a/full-version literal remain the c88 pins. No new runtime rehash,
provider invocation, source/build/release claim is made from this design.
The helper pin check is file identity only, not another helper body review.
The distinct Sol3 diagnostic report/work is neither inspected nor contacted.

### Exact composed-call source and static manifest

The complete prospective executable cell follows, with no materialized
controller/helper/harness file. It is NOT executed. Embedded Python constants
are JSON string encodings; the readable sources below reconstruct each
constant byte-for-byte.

| Source | Bytes | Lines | SHA256 |
| --- | --- | --- | --- |
| clock | 114 | 2 | cfb3f3154cb778388724728b6f7cb804bd7e142844ded7c044afc8dd3135ab4d |
| stop | 1549 | 29 | ccc2702c4e10209952d8e5489c2974a09a7f4d6c47a3705dc89d5bb028ec3ffb |
| readback | 1662 | 20 | 95c6fe05f0016ef302dd9c63533b95141b09ea71a661ab390c112c9e6c13ccb3 |
| persist | 805 | 18 | 819a7785625ce2f78a9cb03b5f242dbfa6728f9f507b242e3c048887ad01ea30 |
| composed_js | 18091 | 284 | 0a7cea070bdab32b419a82eb26a7a991b71a2b14a55553f890b53788cda04a5e |

```js
// Prospective ONE functions.exec cell; NOT executed in this design phase.
const CLOCK="import json,time\nprint(json.dumps({'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())}))\n";
const STOP="import json,os,pathlib,stat,sys,time\nc=json.loads(sys.argv[1])\nclock={'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())}\nrecord={'schema':'riauth.d01-first-observation/v1','first_failure':c['first_failure'],'first_observation_wall_ms':c['first_observation_wall_ms'],'first_event_proven':False,'clock':clock}\nevent_state='write_unconfirmed'\ntry:\n    fd=os.open(pathlib.Path(c['event_out']),os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\n    with os.fdopen(fd,'w',encoding='ascii') as f:\n        f.write(json.dumps(record,sort_keys=True)+'\\n');f.flush();os.fsync(f.fileno())\n    event_state='written'\nexcept OSError:pass\n# Attempt clock persistence BEFORE marker/state/budget comparisons.\n# A metadata error does not prevent the original essential stop protocol.\nlab=pathlib.Path(c['lab']);marker_state='lab_absent'\ntry:\n    if lab.exists():\n        info=lab.lstat()\n        if not stat.S_ISDIR(info.st_mode) or stat.S_IMODE(info.st_mode)!=0o700 or info.st_uid!=os.getuid():\n            marker_state='ownership_unknown'\n        else:\n            marker_state='stop_requested'\n            for name in ('ui-failure','stop'):\n                try:\n                    fd=os.open(lab/name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\n                    os.close(fd)\n                except FileNotFoundError:marker_state='lab_absent'\n                except FileExistsError:pass\nexcept OSError:marker_state='stop_unconfirmed'\nprint(json.dumps({'clock':clock,'event_state':event_state,'marker_state':marker_state}))\n";
const READBACK="import json,pathlib,subprocess,sys,time\nc=json.loads(sys.argv[1])\npids=c['owned_pids']\np=subprocess.run(['/bin/ps','-p',','.join(str(v) for v in pids),'-o','pid='],capture_output=True,timeout=3)\nps_known=p.returncode in (0,1) and all(v.isdigit() for v in p.stdout.split())\npresent=set(int(v) for v in p.stdout.split()) if ps_known else set()\nports={}\nfor port in (9000,3000):\n    p=subprocess.run(['/usr/sbin/lsof','-nP','-t','-iTCP:'+str(port),'-sTCP:LISTEN'],capture_output=True,timeout=3)\n    ports[str(port)]=not bool(p.stdout.strip()) if p.returncode in (0,1) else None\nchildren=None\nouter=pathlib.Path(c['outer_out'])\nif outer.is_file():\n    # This is the unchanged controller's fixed redacted evidence, not app data.\n    data=json.loads(outer.read_bytes())\n    allowed={'helper','whoami','discovery','confidential_client_create','operator_login','server','maintenance_init'}\n    raw=data.get('owned_child_exits')\n    if isinstance(raw,list) and len(raw)==7 and all(isinstance(v,dict) and v.get('name') in allowed and type(v.get('pid')) is int and type(v.get('exit')) is int for v in raw) and {v['name'] for v in raw}==allowed and len({v['pid'] for v in raw})==7 and next(v['pid'] for v in raw if v['name']=='helper')==c['helper_pid'] and next(v['pid'] for v in raw if v['name']=='server')==c['server_pid']:\n        children=[{k:v[k] for k in ('name','pid','exit')} for v in raw]\nprint(json.dumps({'clock':{'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())},'owned_pids':{str(v):(v not in present if ps_known else None) for v in pids},'ports':ports,'lab_absent':not pathlib.Path(c['lab']).exists(),'owned_child_exits':children}))\n";
const PERSIST="import json,os,pathlib,sys\npath=pathlib.Path(sys.argv[1])\nrecord=json.loads(sys.argv[2])\n# Convert decimal strings directly to Python integers, avoiding JS Number rounding.\ndef clocks(value):\n    if isinstance(value,dict):\n        for k,v in list(value.items()):\n            if k in ('monotonic_ns','wall_epoch_ns') and isinstance(v,str):\n                assert v.isdecimal() and len(v)<=19 and 0<=int(v)<2**63\n                value[k]=int(v)\n            else:clocks(v)\n    elif isinstance(value,list):\n        for v in value:clocks(v)\nclocks(record)\nfd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\nwith os.fdopen(fd,'w',encoding='ascii') as f:\n    f.write(json.dumps(record,sort_keys=True,indent=2)+'\\n');f.flush();os.fsync(f.fileno())\nprint(json.dumps({'written_exclusive':True}))\n";
const own=load("d01_immediate_owned_handles");
const OBSKEY="d01_immediate_observation_record";
if (!own || own.runtime_release!==true || own.driver_owned!==true ||
    own.exact_bound!==true || own.single_controller!==true ||
    own.fixture_ready!==true || own.listener_pid_proof!==true ||
    own.fresh_paths_preflight!==true ||
    !Number.isSafeInteger(own.start_epoch_ms) || typeof own.workspace!=="string" ||
    new Set([own.guard_pid,own.server_pid,own.helper_pid,own.browser_pid]).size!==4 ||
    ![own.guard_pid,own.server_pid,own.helper_pid,own.browser_pid,own.exec_session]
       .every(v=>Number.isSafeInteger(v)&&v>0) ||
    ![own.session,own.target_id,own.tab_id,own.lab,own.outer_out,
       own.event_out,own.cleanup_out].every(v=>typeof v==="string"&&v.length>0)) {
  text({proposal_refused:"fresh_owned_context_required"});exit();
}
const record={
  schema:"riauth.d01-immediate-observation-cleanup/v1",
  first_failure:null,first_observation_wall_ms:null,
  first_event_proven:false,whole_cleanup_within60_proven:false,
  first_clock:null,observation_receipts:[],controller_observations:[],local_tool_receipts:[],actions:[],cleanup_errors:[],
  final_absence:null,owned_child_exits:null,controller_exit:null,
  first_observation_to_final_wall_ms:null,latch_to_absence_ms:null
};
const sq=s=>"'"+String(s).replace(/'/g,"'\\''")+"'";
const retain=()=>store(OBSKEY,record);
const cleanupError=label=>{
  if(!record.cleanup_errors.includes(label))record.cleanup_errors.push(label);
  retain();
};
const local=async(source,arg)=>{
  const cmd="python3 -c "+sq(source)+(arg===undefined?"":" "+sq(JSON.stringify(arg)));
  const r=await tools.exec_command({cmd,workdir:own.workspace,
    yield_time_ms:10000,max_output_tokens:2000});
  record.local_tool_receipts.push({
    label:source===CLOCK?"clock":source===STOP?"stop":source===READBACK?"readback":"unknown",
    exit:typeof r.exit_code==="number"?r.exit_code:null,
    session_id:typeof r.session_id==="number"?r.session_id:null
  });retain(); // Numeric collector receipt BEFORE comparisons.
  if(r.session_id!==undefined) {
    cleanupError("owned_local_command_unjoined");throw new Error("local_pending");
  }
  if(r.exit_code!==0)throw new Error("local_failed");
  return JSON.parse(r.output);
};
const ns=c=>BigInt(c.monotonic_ns);
const getClock=()=>local(CLOCK);
let controllerJoined=false;
let controllerPollAvailable=true;
let controllerBuffer="";
let cleanupStarted=false;
let lastSnapshotFlags=null;
function latch(label,receivedWall) {
  if(record.first_failure===null) {
    record.first_failure=label;
    record.first_observation_wall_ms=receivedWall;
    retain(); // Synchronous first-failure/wall latch BEFORE any new await/output.
  }
}
function observeController(r,receivedWall) {
  const observation={received_wall_ms:receivedWall,
    exit:typeof r.exit_code==="number"?r.exit_code:null,
    helper_completed:false,helper_exit:null,fixture_finished:false};
  // Complete numeric result projection is retained BEFORE comparisons.
  record.controller_observations.push(observation);retain();
  if(typeof r.exit_code==="number") {
    controllerJoined=true;record.controller_exit=r.exit_code;retain();
  }
  controllerBuffer+=typeof r.output==="string"?r.output:"";
  if(controllerBuffer.length>16384) {
    latch("controller_observation_invalid",receivedWall);controllerBuffer="";return;
  }
  let cut;
  while((cut=controllerBuffer.indexOf("\n"))>=0) {
    const line=controllerBuffer.slice(0,cut);controllerBuffer=controllerBuffer.slice(cut+1);
    if(!line.trim())continue;
    let event;
    try{event=JSON.parse(line);}catch{
      latch("controller_observation_invalid",receivedWall);continue;
    }
    if(event.helper_completed===true) {
      observation.helper_completed=true;
      observation.helper_exit=typeof event.exit==="number"?event.exit:null;
      retain();
      if(event.exit!==0)latch("helper_failed",receivedWall);
      else if(!cleanupStarted)latch("helper_completed_before_app_checkpoint",receivedWall);
    }
    if(event.fixture_finished===true) {
      observation.fixture_finished=true;retain();
      if(!cleanupStarted)latch("controller_completed_before_app_checkpoint",receivedWall);
    }
  }
  if(controllerJoined&&!cleanupStarted)
    latch("controller_completed_before_app_checkpoint",receivedWall);
}
async function pollController() {
  if(controllerJoined||!controllerPollAvailable)return;
  try {
    const r=await tools.write_stdin({session_id:own.exec_session,
      chars:"",yield_time_ms:5000,max_output_tokens:2000});
    const receivedWall=Date.now();
    observeController(r,receivedWall);
  } catch {
    controllerPollAvailable=false;
    latch("controller_observation_unavailable",Date.now());
    if(cleanupStarted)cleanupError("controller_join_unconfirmed");
  }
}
async function timedDriver(label,allocation,operation,project) {
  let start=null,end=null,result=null,state="unknown";
  try{start=await getClock();}catch{cleanupError("clock_unavailable");}
  const action={label,start_clock:start,end_clock:null,allowed_ms:allocation,
    result_state:"unknown",elapsed_ms:null,over_budget:null};
  record.actions.push(action);retain(); // Start retained BEFORE entering Driver call.
  try {
    result=await operation();
    state=result?.isError===true?"refused":"returned";
  } catch {state="exception";}
  try{end=await getClock();}catch{cleanupError("clock_unavailable");}
  action.end_clock=end;action.result_state=state;
  retain(); // End/result retained BEFORE budget comparison.
  if(start&&end) {
    const d=ns(end)-ns(start);
    if(d>=0n) {
      action.elapsed_ms=Number(d/1000000n);
      action.over_budget=action.elapsed_ms>allocation;
    } else cleanupError("clock_invalid");
  }
  if(action.over_budget)cleanupError("driver_operation_over_budget");
  if(state!=="returned")cleanupError("driver_operation_unconfirmed");
  if(project)project(result,state);
  retain();
}
async function cleanup() {
  if(cleanupStarted)return;
  cleanupStarted=true;
  try {
    const r=await local(STOP,{
      lab:own.lab,event_out:own.event_out,
      first_failure:record.first_failure,
      first_observation_wall_ms:record.first_observation_wall_ms
    });
    record.first_clock=r.clock;record.stop_state=r.marker_state;retain();
    if(r.event_state!=="written")cleanupError("observation_metadata_write_unconfirmed");
    if(r.marker_state==="stop_unconfirmed")cleanupError("stop_unconfirmed");
    if(r.marker_state==="ownership_unknown")cleanupError("ownership_unknown");
  } catch {cleanupError("stop_or_clock_record_unavailable");}
  // No outstanding Driver call exists here: all page calls above were awaited.
  // Original stop protocol already causes the controller's owned-child finally.
  await timedDriver("kill_app",30000,
    ()=>tools.mcp__cua_driver__kill_app({pid:own.browser_pid}));
  await timedDriver("end_session",15000,
    ()=>tools.mcp__cua_driver__end_session({session:own.session}),(r,state)=>{
      record.session_ended=state==="returned"&&
        r?.structuredContent?.active===false&&r.structuredContent.session===own.session;
      retain();
    });
  await timedDriver("list_windows",5000,
    ()=>tools.mcp__cua_driver__list_windows({pid:own.browser_pid}),(r,state)=>{
      const windows=r?.structuredContent?.windows;
      record.window_count=state==="returned"&&Array.isArray(windows)?windows.length:null;
      retain();
    });
  let readStart=null;
  try{readStart=await getClock();}catch{cleanupError("clock_unavailable");}
  const action={label:"owned_join_readback",start_clock:readStart,end_clock:null,
    allowed_ms:null,elapsed_ms:null,over_budget:null,result_state:"unknown"};
  record.actions.push(action);retain();
  if(readStart&&record.first_clock) {
    action.allowed_ms=Math.max(0,60000-Number((ns(readStart)-ns(record.first_clock))/1000000n));
    retain();
  }
  // Continue essential join after60s if late; never claim that deadline enforced.
  // Do not poll another exec session or send a manual process signal.
  while(!controllerJoined&&controllerPollAvailable&&Date.now()<own.start_epoch_ms+900000) {
    await pollController();
    if(record.first_clock) {
      try {
        const c=await getClock();record.last_join_clock=c;retain();
        if(ns(c)-ns(record.first_clock)>60000000000n)
          cleanupError("cleanup_budget_exceeded");
      } catch{cleanupError("clock_unavailable");}
    }
  }
  if(!controllerJoined)cleanupError("child_reap_incomplete");
  try {
    const r=await local(READBACK,{
      owned_pids:[own.guard_pid,own.server_pid,own.helper_pid,own.browser_pid],
      lab:own.lab,outer_out:own.outer_out,helper_pid:own.helper_pid,server_pid:own.server_pid
    });
    action.end_clock=r.clock;action.result_state="returned";
    record.owned_child_exits=r.owned_child_exits;
    record.final_absence={owned_pids:r.owned_pids,ports:r.ports,
      lab:r.lab_absent,window_count:record.window_count??null,
      session_ended:record.session_ended===true};
    record.first_observation_to_final_wall_ms=Date.now()-record.first_observation_wall_ms;
    retain(); // Full fixed readback/exits/clock BEFORE comparisons.
    if(readStart) {
      action.elapsed_ms=Number((ns(r.clock)-ns(readStart))/1000000n);
      action.over_budget=action.allowed_ms===null?null:action.elapsed_ms>action.allowed_ms;
    }
    if(record.first_clock)
      record.latch_to_absence_ms=Number((ns(r.clock)-ns(record.first_clock))/1000000n);
    if(action.over_budget)cleanupError("cleanup_budget_exceeded");
    const a=record.final_absence;
    if(!controllerJoined||!Array.isArray(record.owned_child_exits)||
       record.owned_child_exits.length!==7)cleanupError("child_reap_incomplete");
    if(!Object.values(a.owned_pids).every(v=>v===true)||
       !Object.values(a.ports).every(v=>v===true)||a.lab!==true||
       a.window_count!==0||a.session_ended!==true)
      cleanupError("owned_resource_absence_unproven");
  } catch {action.result_state="exception";cleanupError("owned_readback_unavailable");}
  cleanupError("first_event_unproven");
  record.whole_cleanup_within60_proven=false;retain();
  // Exclusive new file only; existing outer/helper/provider evidence untouched.
  try {
    const cmd="python3 -c "+sq(PERSIST)+" "+sq(own.cleanup_out)+" "+sq(JSON.stringify(record));
    const r=await tools.exec_command({cmd,workdir:own.workspace,
      yield_time_ms:10000,max_output_tokens:500});
    record.local_tool_receipts.push({label:"persist",
      exit:typeof r.exit_code==="number"?r.exit_code:null,
      session_id:typeof r.session_id==="number"?r.session_id:null});retain();
    if(r.session_id!==undefined)cleanupError("owned_local_command_unjoined");
    if(r.session_id!==undefined||r.exit_code!==0)
      cleanupError("cleanup_metadata_write_unconfirmed");
  } catch {cleanupError("cleanup_metadata_write_unconfirmed");}
  controllerBuffer="";
}
await pollController();
if(record.first_failure===null) {
  try {
    const r=await tools.mcp__cua_driver__browser_navigate({
      session:own.session,target_id:own.target_id,tab_id:own.tab_id,
      url:"http://localhost:3000/"});
    const receivedWall=Date.now();
    record.observation_receipts.push({kind:"browser_navigation",received_wall_ms:receivedWall});
    retain(); // Received clock BEFORE navigation outcome comparison.
    if(r?.isError===true)latch("driver_navigation_refused",receivedWall);
  } catch {latch("driver_navigation_exception",Date.now());}
}
if(record.first_failure===null) {
  try {
    const r=await tools.mcp__cua_driver__get_browser_state({
      session:own.session,target_id:own.target_id,tab_id:own.tab_id,
      snapshot_format:"semantic_v2",include_screenshot:false});
    const receivedWall=Date.now();
    record.observation_receipts.push({kind:"browser_snapshot",received_wall_ms:receivedWall});
    retain(); // Received clock BEFORE any page predicate comparison.
    const s=r?.structuredContent;
    const nodes=Array.isArray(s?.content_refs)?s.content_refs:[];
    lastSnapshotFlags={
      received_wall_ms:receivedWall,
      status_ok:r?.isError!==true&&s?.status==="ok",
      complete:s?.snapshot?.complete===true,
      heading_match:nodes.some(n=>n.role==="heading"&&n.name==="Local demo"),
      error_match:nodes.some(n=>n.name==="Local demo could not complete this request."),
      interactive_ref_present:Array.isArray(s?.refs)&&
        s.refs.some(n=>Array.isArray(n.actions)&&n.actions.includes("click"))
    };
    store("d01_immediate_snapshot_flags",lastSnapshotFlags);
    // Finite booleans only; no outline/URL/name/value/subject/token is persisted.
    if(lastSnapshotFlags.error_match)latch("application_page_failed",receivedWall);
    else if(!lastSnapshotFlags.status_ok||!lastSnapshotFlags.complete||
            !lastSnapshotFlags.heading_match||!lastSnapshotFlags.interactive_ref_present)
      latch("application_snapshot_unconfirmed",receivedWall);
  } catch {latch("driver_snapshot_exception",Date.now());}
}
if(record.first_failure===null)await pollController();
if(record.first_failure!==null) {
  await cleanup(); // NO text/notify/yield_control/model turn before essential cleanup.
  text({result:"failed",first_failure:record.first_failure,
    cleanup_errors:record.cleanup_errors,final_absence:record.final_absence,
    controller_exit:record.controller_exit,whole_cleanup_within60_proven:false,
    resource_release_proven:record.final_absence!==null&&
      !record.cleanup_errors.some(v=>[
        "child_reap_incomplete","owned_resource_absence_unproven",
        "owned_readback_unavailable","owned_local_command_unjoined"].includes(v))});
} else {
  text({result:"application_checkpoint_observed_only",
    snapshot_flags:lastSnapshotFlags,journey_credit:false});
}
```

Readable exact clock source:

```python
import json,time
print(json.dumps({'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())}))
```

Readable exact first-clock/original-stop source:

```python
import json,os,pathlib,stat,sys,time
c=json.loads(sys.argv[1])
clock={'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())}
record={'schema':'riauth.d01-first-observation/v1','first_failure':c['first_failure'],'first_observation_wall_ms':c['first_observation_wall_ms'],'first_event_proven':False,'clock':clock}
event_state='write_unconfirmed'
try:
    fd=os.open(pathlib.Path(c['event_out']),os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    with os.fdopen(fd,'w',encoding='ascii') as f:
        f.write(json.dumps(record,sort_keys=True)+'\n');f.flush();os.fsync(f.fileno())
    event_state='written'
except OSError:pass
# Attempt clock persistence BEFORE marker/state/budget comparisons.
# A metadata error does not prevent the original essential stop protocol.
lab=pathlib.Path(c['lab']);marker_state='lab_absent'
try:
    if lab.exists():
        info=lab.lstat()
        if not stat.S_ISDIR(info.st_mode) or stat.S_IMODE(info.st_mode)!=0o700 or info.st_uid!=os.getuid():
            marker_state='ownership_unknown'
        else:
            marker_state='stop_requested'
            for name in ('ui-failure','stop'):
                try:
                    fd=os.open(lab/name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
                    os.close(fd)
                except FileNotFoundError:marker_state='lab_absent'
                except FileExistsError:pass
except OSError:marker_state='stop_unconfirmed'
print(json.dumps({'clock':clock,'event_state':event_state,'marker_state':marker_state}))
```

Readable exact own-resource readback source:

```python
import json,pathlib,subprocess,sys,time
c=json.loads(sys.argv[1])
pids=c['owned_pids']
p=subprocess.run(['/bin/ps','-p',','.join(str(v) for v in pids),'-o','pid='],capture_output=True,timeout=3)
ps_known=p.returncode in (0,1) and all(v.isdigit() for v in p.stdout.split())
present=set(int(v) for v in p.stdout.split()) if ps_known else set()
ports={}
for port in (9000,3000):
    p=subprocess.run(['/usr/sbin/lsof','-nP','-t','-iTCP:'+str(port),'-sTCP:LISTEN'],capture_output=True,timeout=3)
    ports[str(port)]=not bool(p.stdout.strip()) if p.returncode in (0,1) else None
children=None
outer=pathlib.Path(c['outer_out'])
if outer.is_file():
    # This is the unchanged controller's fixed redacted evidence, not app data.
    data=json.loads(outer.read_bytes())
    allowed={'helper','whoami','discovery','confidential_client_create','operator_login','server','maintenance_init'}
    raw=data.get('owned_child_exits')
    if isinstance(raw,list) and len(raw)==7 and all(isinstance(v,dict) and v.get('name') in allowed and type(v.get('pid')) is int and type(v.get('exit')) is int for v in raw) and {v['name'] for v in raw}==allowed and len({v['pid'] for v in raw})==7 and next(v['pid'] for v in raw if v['name']=='helper')==c['helper_pid'] and next(v['pid'] for v in raw if v['name']=='server')==c['server_pid']:
        children=[{k:v[k] for k in ('name','pid','exit')} for v in raw]
print(json.dumps({'clock':{'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())},'owned_pids':{str(v):(v not in present if ps_known else None) for v in pids},'ports':ports,'lab_absent':not pathlib.Path(c['lab']).exists(),'owned_child_exits':children}))
```

Readable exact exclusive cleanup-record writer:

```python
import json,os,pathlib,sys
path=pathlib.Path(sys.argv[1])
record=json.loads(sys.argv[2])
# Convert decimal strings directly to Python integers, avoiding JS Number rounding.
def clocks(value):
    if isinstance(value,dict):
        for k,v in list(value.items()):
            if k in ('monotonic_ns','wall_epoch_ns') and isinstance(v,str):
                assert v.isdecimal() and len(v)<=19 and 0<=int(v)<2**63
                value[k]=int(v)
            else:clocks(v)
    elif isinstance(value,list):
        for v in value:clocks(v)
clocks(record)
fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
with os.fdopen(fd,'w',encoding='ascii') as f:
    f.write(json.dumps(record,sort_keys=True,indent=2)+'\n');f.flush();os.fsync(f.fileno())
print(json.dumps({'written_exclusive':True}))
```

### Actual static checks, future validation and held scope

Actually performed: final four Python AST parses/in-memory code-object compiles,
final composed module node --input-type=module --check exit0 (syntax only),
full actual controller AST/code-object parse with nine functions and zero
byte/AST delta, archive/embedded-source identity, helper pin and all twelve
metadata byte/hash/mode checks. Earlier in-memory source drafts were checked
while preparing the final archive; no proposed cell or collector ran, no
helper import/main, module payload execution or runtime case was attempted.

An initial source-only rg used nonexistent skill references/MACOS.md and
references/BROWSER.md paths and exited2. The installed sibling paths were then
read directly; this was a documentation-path correction, not a provider switch
or runtime retry. The repository AGENTS.md file search exited1 with no match.
No actual fixture failure/result is revised by either read.

Before any future runtime, root must review this immutable source and decide
its new observation-file reservation and unsupported-call/true-first-event
limits. Any later finite memory verification needs separate authorization;
none is inferred from static syntax checks. Necessary future controlled
validation covers: snapshot error and incomplete/unknown projection; navigation
refusal; helper_failed/controller completion/partial JSON/exit projection;
first-failure preservation across cleanup errors; exclusive evidence failure
still requesting stop; each Driver refusal/exception/over-allocation; missing
child/PID/window/session/port/lab proof preventing release; late/unreturned
calls and collector yield remaining unresolved; full preserved-controller and
security-pin proofs. Stub cases would not be browser/crypto journey evidence.
No new tests, dependencies, targets or broad campaign are proposed or run.

The complete341644-byte c88 report prefix/SHA
2493b5ebf2405d624cf1c610cfd4cace0513c96708d72871c9df2ad2d004f721
remains byte-exact. All nine historical0600 records plus the three c88 records
remain exact, retaining failed61/76/dated memory78 limits, old senderUNKNOWN,
lost provider values/reporting correction, all fixture/capacity/preparation/
copy-mode failures and c88's generic failure/cleanup timing. No raw private
value or broader process args/env is inspected. Only this report is appended
with archive/prefix/hash/fence/final-newline/whitespace/scope checks and a
separate immutable report-only commit. No source/helper/controller/product/
guide/D05/config/test edit, filesystem cleanup, Driver/native/CLI/HTTP/listener/
Cargo runtime, worker/task/WT/shell/main/push/status or contact occurs.
D01/D05 and future runtime remain root-owned and HELD; closed rows stay closed.

## Dated design phase: finite memory envelope for immediate cleanup, 2026-10-02

Reservation wave30_D01_immediate_cleanup_finite_envelope_design owns ONLY this
append. No cell/harness/stub/collector is executed and no runtime slot is
acquired or released. This is a complete prospective18-case memory design
around the actual delayed-observation/cleanup risk, not a browser pass or an
every-combination campaign. Real fixture/runtime remains HELD.

The exact reviewed284-line composed cell from3f957001 is embedded unchanged.
One source-only discrepancy requires root gate review before a future execution:
the current resource_release_proven predicate excludes missing-resource/join/
readback/unjoined-command errors, but does NOT exclude
driver_operation_unconfirmed or driver_operation_over_budget. A kill refusal,
exception or late return can therefore still emit resource_release_proven true
if every separate absence readback is true. whole_cleanup_within60_proven
still stays false. The prior c88 resource release and119s readback history are
not reclassified by this prospective observation.

The requested strict envelope deliberately expects false for those three
Driver cases even with complete final absence readbacks. It does not weaken
that expectation or alter the immutable cell to manufacture a green run.
Under that strict interpretation, case5 driver_exception is statically
expected to expose resource_release_failclosed; cases6/7 exercise the same
boundary separately if ever reached. These are source-derived predictions,
not observed failures or executed cases. Root must decide the intended
resource-release versus cleanup-allocation gate before separately releasing
memory verification. An actual absence-only interpretation would differ from
these strict expectations and needs an explicit revised reservation; this
phase makes neither a source correction nor a silent gate reinterpretation.

### Minimal cases, fixed prospective counts and stopping rule

There are18 distinct invocations, one per needed boundary, with no Cartesian
combinations. The six missing-proof cases each remove ONE proof rather than
adding unrelated failures. Partial JSON, failed join and malformed readback
are distinct because they exercise different first-failure/cleanup paths.

| Order/case | Expected first latch | Expected release projection | Planned stub tool calls |
| --- | --- | --- | --- |
| 1. ui_error | application_page_failed | true | 18 |
| 2. helper_failure | helper_failed | true | 16 |
| 3. controller_completion | controller_completed_before_app_checkpoint | true | 14 |
| 4. metadata_write_failure | application_page_failed | true | 18 |
| 5. driver_exception | application_page_failed | false | 18 |
| 6. driver_refusal | application_page_failed | false | 18 |
| 7. driver_late_return | application_page_failed | false | 18 |
| 8. missing_child_proof | application_page_failed | false | 18 |
| 9. missing_pid_proof | application_page_failed | false | 18 |
| 10. missing_window_proof | application_page_failed | false | 18 |
| 11. missing_session_proof | application_page_failed | false | 18 |
| 12. missing_port_proof | application_page_failed | false | 18 |
| 13. missing_lab_proof | application_page_failed | false | 18 |
| 14. partial_controller_json | application_page_failed | true | 18 |
| 15. join_failure | application_page_failed | false | 18 |
| 16. readback_projection_failure | application_page_failed | false | 18 |
| 17. missing_ownership | none | not emitted | 0 |
| 18. application_confirmed | none | not emitted | 4 |

Complete-run planned counts, derived statically from the unchanged source:
18 attempted/completed case groups;286 total stub tool calls, comprising
78 Driver calls,33 original-controller output polls and175 collector calls.
Collectors:127 clock,16 original-stop,16 readback,16 exclusive-persist.
There are16 cleanup sequences and18 final output projections. Store/date/
assertion bookkeeping is not counted as a tool call. These are planned counts,
not actual measurements, and a failing run must not report them as completed.

The first16 cases stop/cleanup; ownership refusal touches no stub tool;
application-confirmed uses only two controller polls plus navigation/snapshot,
has no cleanup yet and leaves journey_credit false. The normal UI-error case
proves the proposed ordering even though it has no journey credit. Successful
resource absence is independent from the always-false actual-first-event/
whole60 flags.

The harness stops at the FIRST failed assertion/cell outcome. It retains the
attempted case's full fixed counts/result BEFORE verification assertions,
includes that failed case in cumulative counts, keeps completed distinct from
attempted, and lists every unreached case. No case/expectation/source correction
or automatic rerun follows a failure. With the static gate discrepancy above,
a case5 failure would mean attempted5/completed4/unreached13 and84 stub calls
(21 Driver,9 controller,54 collector;39 clock and5 each stop/readback/persist),
if earlier cases run as modelled. This forecast is not an execution receipt.

The future fixed result schema is
riauth.d01-cleanup-memory-envelope/v1: source cell/logic hashes, planned counts,
attempted/completed, per-case fixed counts/first-failure/resource-release/
assertion projection, cumulative actual counts, first assertion failure and
unreached names. The output is retained/printed before setting the numeric
process exit code. No raw exception/stack/request/page/private value is output.
A memory assertion result is not browser/crypto/physical resource evidence.

### Isolation, finite scheduling and clock model

The prospective payload imports only node:vm and node:crypto for memory
evaluation and source SHA256. It imports no filesystem, child-process,
network/HTTP/provider/Driver module. There is no child process or actual tool
reference in the evaluated context. Names such as exec_command, write_stdin
and Driver methods are newly defined deterministic stubs, not forwarded tools.
No Python collector, helper, module main, binary, native verifier, CLI,
listener, browser or file cleanup is executed by this envelope.

Every stub async method settles immediately with a finite public value or
fixed synthetic exception. Logical time advances10ms per stub call; the ONE
late-return case adds31000 virtual ms. Clock values are decimal strings with
a fixed public origin. They do not sample OS time or prove actual elapsed
cleanup, first-event identity, deadline enforcement or the old119s gap.

Limits per case:32 stub tool calls,512 store calls,128 Date.now reads and at
most2 controller polls. Source/command parsing is bounded to65536 characters.
There are exactly18 case entries. VM synchronous evaluation has a real
supported timeout1000ms, which does not become an asynchronous/MCP timeout.
No pending real tool promise, Promise.race, cancellation, real timer or
unawaited operation is introduced. The immutable async awaits call only
immediately settled, bounded stubs; its loop cannot acquire external work.
These limits bound the controlled memory model, not Driver call duration.

A later release must still reserve ONE bounded memory invocation, proposed
outer budget <=30s, and retain exact stdin payload/hash/stdout/stderr/numeric
exit/elapsed before comparing it. This design does not invent a timeout field
for tools.exec or Driver. The exact future stdin below targets
node --input-type=module; any enforceable host process deadline/outer command
is root's separately reviewed reservation. No invocation is made here.

The embedded cell is wrapped only as an async function for its existing
top-level awaits. Its complete bytes are preserved between the wrapper's two
lines. No predicate, ordering, source constant, guard or output is rewritten.
The synthetic context uses fresh public101/102/103/104 identities and901 exec
session with /synthetic paths. They are never passed to real tools or treated
as ownership grants. The missing-ownership case flips exactly exact_bound to
false and must refuse before any tool/kill. No real/runtime-release authority
is inferred from synthetic true flags.

The local shell-word decoder parses only the existing quoted Python command
construction as DATA. It matches each entire embedded Python source byte-for-
byte, decodes fixed public JSON arguments, and returns modelled results.
It never interprets/evaluates a shell or Python source. Full-record persistence
is an in-memory clone. No path is opened, written, removed, signalled or probed.

### Exact assertions and advertised receipt shape

On every cleanup stub entry, the stored first failure must already equal the
case expectation and no text/output may have occurred. The trace requires
first latch before original stop, then kill_app/end_session/list_windows/
readback/persist, no later journey operation, exactly four cleanup action
projections, and only one final output after cleanup returns.
The initial received clock must remain unchanged after later failures.
The record and final result must keep first_event_proven/whole60 false.

Helper failure and early controller completion trigger cleanup before browser
navigation/snapshot. The partial JSON case splits one finite helper_completed
line across initial and join polls: it must reconstruct the later failure and
preserve the earlier UI latch. Failed join has no automatic poll retry.
Malformed readback is only an ASCII invalid JSON projection; it must record a
separate cleanup error and preserve the original failure.

Metadata-write failure returns event_state write_unconfirmed, but still emits
the modelled original stop marker and follows the unchanged JS cleanup path.
This checks the orchestration handling of the collector's fixed result.
It does NOT execute/validate an OS file-write exception. The already reviewed
STOP Python source is additionally checked statically: its OSError handler
continues to the independent original-marker block. The source stays exact;
no actual marker/metadata write or deletion occurs.

Missing child uses null; missing PID uses the full four-key null projection
that an unknown ps observation can produce; window/session use generic
CallToolResult without their structured proof; port uses one null observation;
lab uses null. Each must prevent release under the existing missing-proof
predicate. The Driver exception/refusal/late cases deliberately keep all
independent absence proofs true to expose, rather than mask, the strict gate
discrepancy described above.

No Driver invocation was made. Shape provenance is limited to:
c88 retained semantic p859/p858 structuredContent status ok, snapshot.complete
boolean, content_refs role/name/actions and refs arrays; retained end_session
active false plus exact session; retained list_windows structured windows[];
and retained kill_app generic text result with no structured proof.
Advertised get_browser_state/kill_app/end_session/list_windows descriptions
were read from tool metadata only. No timeout/cancel capability is advertised.

The snapshot stub uses exactly the fields inspected by the immutable cell.
The confirmed case's synthetic click ref follows the documented actions-array
contract; it is a modelled interactive ref, not an observed actual application
success or newly invented real ref. Browser text/name values are fixed public
schema examples. The public root URL only validates stub arguments and sends
no request. The generic kill result is not treated as absence proof; the
separate readback must establish it. No header/request/error/credential/
token/cookie/subject/query URL/sender is logged or inferred.

### Immutable pins and complete archives

Source identity is separate from body review and execution. Exact pins:

| Source | Bytes | Lines | SHA256 |
| --- | --- | --- | --- |
| cell | 18091 | 284 | 0a7cea070bdab32b419a82eb26a7a991b71a2b14a55553f890b53788cda04a5e |
| logic | 17528 | 322 | e3d39dc4a38a48ef9a1ace4d232544bcb0c70172b9822a5531072bed9387bb9d |
| payload | 37730 | 16 | 21ff02d7affa22017651be01fbc4072acc70265c6bf8191e55e2d40a1dab7719 |

Original four Python constants remain exact:
CLOCK cfb3f3154cb778388724728b6f7cb804bd7e142844ded7c044afc8dd3135ab4d;
STOP ccc2702c4e10209952d8e5489c2974a09a7f4d6c47a3705dc89d5bb028ec3ffb;
READBACK95c6fe05f0016ef302dd9c63533b95141b09ea71a661ab390c112c9e6c13ccb3;
PERSIST819a7785625ce2f78a9cb03b5f242dbfa6728f9f507b242e3c048887ad01ea30.
The complete readable284-line cell and four collectors are already in the
byte-preserved3f957001 prefix; the exact payload embeds them again as source.

The original193-line controller command SHA6662bdbb168bf0ba6ec5ff24b6f3b88dc15b54b4dfc3b800fdaca1cbee403bbb
and helper470 SHA7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0
remain unchanged, as do START+840/inclusive900/prepare180/helper600/
pending180/refusal4/8.5stop8floor and every accepted route/crypto/security gate.
No controller/helper timing insertion or source correction is proposed here.
Root staging44fbc3f is a provenance reference supplied by root, not a merge,
new body review or runtime observation. Sol3's observer/helper diagnostics are
not inspected, owned or contacted.

Complete readable memory logic, NOT executed:

```js
const CELL_PIN="0a7cea070bdab32b419a82eb26a7a991b71a2b14a55553f890b53788cda04a5e";
const PYTHON_PINS={
  CLOCK:"cfb3f3154cb778388724728b6f7cb804bd7e142844ded7c044afc8dd3135ab4d",
  STOP:"ccc2702c4e10209952d8e5489c2974a09a7f4d6c47a3705dc89d5bb028ec3ffb",
  READBACK:"95c6fe05f0016ef302dd9c63533b95141b09ea71a661ab390c112c9e6c13ccb3",
  PERSIST:"819a7785625ce2f78a9cb03b5f242dbfa6728f9f507b242e3c048887ad01ea30"
};
const CASES=[
  {name:"ui_error",first:"application_page_failed",release:true,tools:18},
  {name:"helper_failure",first:"helper_failed",release:true,tools:16},
  {name:"controller_completion",first:"controller_completed_before_app_checkpoint",release:true,tools:14},
  {name:"metadata_write_failure",first:"application_page_failed",release:true,tools:18},
  {name:"driver_exception",first:"application_page_failed",release:false,tools:18},
  {name:"driver_refusal",first:"application_page_failed",release:false,tools:18},
  {name:"driver_late_return",first:"application_page_failed",release:false,tools:18},
  {name:"missing_child_proof",first:"application_page_failed",release:false,tools:18},
  {name:"missing_pid_proof",first:"application_page_failed",release:false,tools:18},
  {name:"missing_window_proof",first:"application_page_failed",release:false,tools:18},
  {name:"missing_session_proof",first:"application_page_failed",release:false,tools:18},
  {name:"missing_port_proof",first:"application_page_failed",release:false,tools:18},
  {name:"missing_lab_proof",first:"application_page_failed",release:false,tools:18},
  {name:"partial_controller_json",first:"application_page_failed",release:true,tools:18},
  {name:"join_failure",first:"application_page_failed",release:false,tools:18},
  {name:"readback_projection_failure",first:"application_page_failed",release:false,tools:18},
  {name:"missing_ownership",first:null,release:null,tools:0},
  {name:"application_confirmed",first:null,release:null,tools:4}
];
const PLAN={cases:18,tool_calls:286,driver_calls:78,controller_calls:33,
  collector_calls:175,clock:127,stop:16,readback:16,persist:16,
  cleanup_sequences:16,outputs:18};
const clone=v=>JSON.parse(JSON.stringify(v));
function ensure(ok,label){if(!ok)throw {envelope_assertion:label};}
function shellWords(command){
  ensure(typeof command==="string"&&command.length<65536,"command_bounds");
  const words=[];let word="",quoted=false,started=false;
  for(let i=0;i<command.length;i++){
    const c=command[i];
    if(quoted){if(c==="'")quoted=false;else word+=c;continue;}
    if(c==="'"){quoted=true;started=true;continue;}
    if(c==="\\"){ensure(i+1<command.length,"command_escape");word+=command[++i];started=true;continue;}
    if(/\s/.test(c)){if(started){words.push(word);word="";started=false;}continue;}
    word+=c;started=true;
  }
  ensure(!quoted,"command_quote");
  if(started)words.push(word);
  return words;
}
function sources(cell){
  const result={};
  for(const name of Object.keys(PYTHON_PINS)){
    const line=cell.split("\n").find(v=>v.startsWith("const "+name+"="));
    ensure(typeof line==="string","embedded_source_missing");
    result[name]=JSON.parse(line.slice(("const "+name+"=").length,-1));
    ensure(sha(result[name])===PYTHON_PINS[name],"embedded_source_hash");
  }
  return result;
}
function ownContext(){
  return {runtime_release:true,driver_owned:true,exact_bound:true,single_controller:true,
    fixture_ready:true,listener_pid_proof:true,fresh_paths_preflight:true,
    workspace:"/synthetic/d01",guard_pid:101,server_pid:102,helper_pid:103,browser_pid:104,
    exec_session:901,session:"synthetic-session",target_id:"synthetic-target",tab_id:"synthetic-tab",
    lab:"/synthetic/lab",outer_out:"/synthetic/outer",event_out:"/synthetic/event",
    cleanup_out:"/synthetic/cleanup",start_epoch_ms:1700000000000};
}
async function oneCase(spec,cell,py){
  const own=ownContext();
  if(spec.name==="missing_ownership")own.exact_bound=false;
  const memory=new Map(),trace=[],outputs=[],counts={
    tool_calls:0,driver_calls:0,controller_calls:0,collector_calls:0,
    clock:0,stop:0,readback:0,persist:0
  };
  let tick=1000,stores=0,dateCalls=0,polls=0,firstLatch=null,retainedFirst=null;
  let cleanupRecord=null,stopRequested=false,exitSeen=false;
  const EXIT={synthetic_exit:true};
  const push=(kind,label)=>trace.push({kind,label,tick});
  const now=()=>{ensure(++dateCalls<=128,"date_bound");return own.start_epoch_ms+tick;};
  const clock=()=>({monotonic_ns:String(1000000000000n+BigInt(tick)*1000000n),
    wall_epoch_ns:String(BigInt(own.start_epoch_ms+tick)*1000000n)});
  function call(label,domain,cleanup){
    ensure(++counts.tool_calls<=32,"tool_bound");
    counts[domain]++;
    push("tool",label);tick+=10;
    if(cleanup){
      const r=memory.get("d01_immediate_observation_record");
      ensure(firstLatch!==null&&r?.first_failure===spec.first,"latch_before_cleanup");
      ensure(outputs.length===0,"no_output_before_cleanup");
    }
  }
  const toolResult=v=>({exit_code:0,output:JSON.stringify(v)+"\n"});
  const children=()=>["helper","whoami","discovery","confidential_client_create",
    "operator_login","server","maintenance_init"].map((name,i)=>({
      name,pid:name==="helper"?103:name==="server"?102:200+i,exit:name==="helper"?1:0}));
  const tools={
    exec_command:async args=>{
      ensure(args.workdir===own.workspace&&args.yield_time_ms===10000,"collector_shape");
      const words=shellWords(args.cmd);
      ensure(words[0]==="python3"&&words[1]==="-c","collector_command");
      const name=Object.keys(py).find(n=>py[n]===words[2]);
      ensure(name!==undefined,"collector_source_identity");
      call(name.toLowerCase(),"collector_calls",true);counts[name.toLowerCase()]++;
      if(name==="CLOCK"){
        ensure(words.length===3,"clock_arguments");
        return toolResult(clock());
      }
      if(name==="STOP"){
        ensure(words.length===4,"stop_arguments");
        const c=JSON.parse(words[3]);
        ensure(c.lab===own.lab&&c.event_out===own.event_out&&c.first_failure===spec.first,
          "stop_owned_arguments");
        ensure(c.first_observation_wall_ms===retainedFirst,"stop_received_clock");
        push("marker","original_stop_requested");stopRequested=true;
        return toolResult({clock:clock(),event_state:spec.name==="metadata_write_failure"?
          "write_unconfirmed":"written",marker_state:"stop_requested"});
      }
      if(name==="READBACK"){
        ensure(words.length===4,"readback_arguments");
        const c=JSON.parse(words[3]);
        ensure(JSON.stringify(c.owned_pids)===JSON.stringify([101,102,103,104])&&
          c.lab===own.lab&&c.outer_out===own.outer_out&&c.helper_pid===103&&c.server_pid===102,
          "readback_owned_arguments");
        if(spec.name==="readback_projection_failure")return {exit_code:0,output:"{"};
        const unknown=spec.name==="missing_pid_proof";
        return toolResult({clock:clock(),owned_pids:Object.fromEntries(
          [101,102,103,104].map(p=>[String(p),unknown?null:true])),
          ports:{"9000":true,"3000":spec.name==="missing_port_proof"?null:true},
          lab_absent:spec.name==="missing_lab_proof"?null:true,
          owned_child_exits:spec.name==="missing_child_proof"?null:children()});
      }
      ensure(name==="PERSIST"&&words.length===5&&words[3]===own.cleanup_out,"persist_arguments");
      cleanupRecord=JSON.parse(words[4]);
      push("persist","exclusive_cleanup_projection");
      return {exit_code:0,output:'{"written_exclusive":true}\n'};
    },
    write_stdin:async args=>{
      ensure(args.session_id===901&&args.chars===""&&args.yield_time_ms===5000,
        "controller_owned_arguments");
      call("controller_poll","controller_calls",polls>0&&spec.name!=="application_confirmed");
      polls++;
      ensure(polls<=2,"controller_poll_bound");
      if(polls===1){
        if(spec.name==="helper_failure")return {session_id:901,
          output:'{"helper_completed":true,"exit":1}\n'};
        if(spec.name==="controller_completion")return {exit_code:1,
          output:'{"fixture_finished":true}\n'};
        if(spec.name==="partial_controller_json")return {session_id:901,
          output:'{"helper_completed":'};
        return {session_id:901,output:""};
      }
      if(spec.name==="application_confirmed")return {session_id:901,output:""};
      if(spec.name==="join_failure")throw {synthetic_failure:true};
      if(spec.name==="partial_controller_json")return {exit_code:1,
        output:'true,"exit":1}\n'};
      return {exit_code:1,output:'{"fixture_finished":true}\n'};
    },
    mcp__cua_driver__browser_navigate:async args=>{
      ensure(args.session===own.session&&args.target_id===own.target_id&&
        args.tab_id===own.tab_id&&args.url==="http://localhost:3000/","navigation_binding");
      call("browser_navigate","driver_calls",false);
      return {content:[{type:"text",text:"synthetic navigation returned"}]};
    },
    mcp__cua_driver__get_browser_state:async args=>{
      ensure(args.session===own.session&&args.target_id===own.target_id&&
        args.tab_id===own.tab_id&&args.snapshot_format==="semantic_v2"&&
        args.include_screenshot===false,"snapshot_binding");
      call("browser_snapshot","driver_calls",false);
      const good=spec.name==="application_confirmed";
      const content_refs=[{role:"heading",name:"Local demo",actions:[],frame:"main"}];
      if(!good)content_refs.push({role:"statictext",
        name:"Local demo could not complete this request.",actions:[],frame:"main"});
      return {structuredContent:{status:"ok",mode:"snapshot",
        snapshot:{complete:true,format:"semantic_v2",id:"synthetic-snapshot"},
        content_refs,refs:good?[{role:"button",name:"synthetic action",
          ref:"synthetic-snapshot:1",actions:["click"],frame:"main"}]:[]}};
    },
    mcp__cua_driver__kill_app:async args=>{
      ensure(args.pid===104&&Object.keys(args).length===1,"kill_owned_arguments");
      call("kill_app","driver_calls",true);
      if(spec.name==="driver_exception")throw {synthetic_failure:true};
      if(spec.name==="driver_refusal")return {isError:true,
        content:[{type:"text",text:"synthetic refusal"}]};
      if(spec.name==="driver_late_return")tick+=31000;
      return {content:[{type:"text",text:"synthetic kill returned"}]};
    },
    mcp__cua_driver__end_session:async args=>{
      ensure(args.session===own.session,"end_owned_arguments");
      call("end_session","driver_calls",true);
      if(spec.name==="missing_session_proof")return {content:[]};
      return {structuredContent:{active:false,session:own.session}};
    },
    mcp__cua_driver__list_windows:async args=>{
      ensure(args.pid===104,"windows_owned_arguments");
      call("list_windows","driver_calls",true);
      return spec.name==="missing_window_proof"?{content:[]}:
        {structuredContent:{current_space_id:1,windows:[]}};
    }
  };
  const sandbox={
    tools,Date:class {static now(){return now();}},
    load:key=>{ensure(key==="d01_immediate_owned_handles","load_key");return clone(own);},
    store:(key,value)=>{
      ensure(++stores<=512,"store_bound");
      ensure(["d01_immediate_observation_record","d01_immediate_snapshot_flags"].includes(key),
        "store_key");
      const copy=clone(value);memory.set(key,copy);
      if(key==="d01_immediate_observation_record"&&copy.first_failure!==null){
        if(firstLatch===null){
          firstLatch=trace.length;retainedFirst=copy.first_observation_wall_ms;
          push("latch",copy.first_failure);
        }
        ensure(copy.first_failure===spec.first,"first_failure_immutable");
      }
    },
    text:value=>{
      push("output","fixed_result");outputs.push(clone(value));
      ensure(outputs.length===1,"output_bound");
    },
    notify:()=>{throw {envelope_assertion:"notification_forbidden"};},
    yield_control:()=>{throw {envelope_assertion:"model_yield_forbidden"};},
    exit:()=>{exitSeen=true;throw EXIT;}
  };
  let cellError=null;
  try{
    const script=new vm.Script("(async()=>{\n"+cell+"\n})()",{filename:"immutable-cell"});
    const promise=script.runInNewContext(sandbox,{timeout:1000});
    await promise;
  }catch(error){if(error!==EXIT)cellError=error;}
  // Retain complete fixed outcome/counts BEFORE any verification assertion.
  const item={case:spec.name,counts:clone(counts),first_failure:outputs[0]?.first_failure??null,
    resource_release_proven:outputs[0]?.resource_release_proven??null,
    assertions_completed:false,assertion:null};
  try {
  ensure(cellError===null,"immutable_cell_exception");
  ensure(counts.tool_calls===spec.tools,"planned_tool_count");
  ensure(outputs.length===1,"exact_output_count");
  const result=outputs[0];
  if(spec.name==="missing_ownership"){
    ensure(exitSeen&&counts.tool_calls===0&&firstLatch===null&&
      result.proposal_refused==="fresh_owned_context_required","ownership_refuses_before_tools");
  }else if(spec.name==="application_confirmed"){
    ensure(result.result==="application_checkpoint_observed_only"&&result.journey_credit===false&&
      firstLatch===null&&counts.collector_calls===0&&counts.driver_calls===2,
      "confirmed_not_journey_or_cleanup");
  }else{
    const record=memory.get("d01_immediate_observation_record");
    ensure(result.result==="failed"&&result.first_failure===spec.first&&
      record.first_failure===spec.first,"failure_preserved");
    ensure(record.first_observation_wall_ms===retainedFirst,"first_clock_preserved");
    ensure(record.first_event_proven===false&&record.whole_cleanup_within60_proven===false&&
      result.whole_cleanup_within60_proven===false,"no_actual_anchor_or_timing_credit");
    ensure(stopRequested&&counts.stop===1&&counts.persist===1&&cleanupRecord!==null,
      "original_stop_and_exclusive_projection");
    const order=["stop","kill_app","end_session","list_windows","readback","persist"];
    const positions=order.map(label=>trace.findIndex(v=>v.kind==="tool"&&v.label===label));
    ensure(positions.every(p=>p>firstLatch)&&positions.every((p,i)=>i===0||p>positions[i-1]),
      "latch_then_cleanup_order");
    ensure(trace.at(-1).kind==="output","no_output_until_cleanup_returned");
    ensure(!trace.some(v=>v.kind==="tool"&&["browser_navigate","browser_snapshot"].includes(v.label)&&
      trace.indexOf(v)>firstLatch),"no_journey_after_failure");
    ensure(record.actions.length===4&&record.actions[0].label==="kill_app"&&
      record.actions[1].label==="end_session"&&record.actions[2].label==="list_windows"&&
      record.actions[3].label==="owned_join_readback","exact_action_projection");
    if(spec.name==="metadata_write_failure")
      ensure(record.cleanup_errors.includes("observation_metadata_write_unconfirmed"),
        "metadata_failure_separate");
    if(["driver_exception","driver_refusal"].includes(spec.name))
      ensure(record.cleanup_errors.includes("driver_operation_unconfirmed"),"driver_failure_recorded");
    if(spec.name==="driver_late_return")
      ensure(record.cleanup_errors.includes("driver_operation_over_budget")&&
        record.actions[0].elapsed_ms>30000,"late_return_observed_only");
    if(spec.name==="partial_controller_json")
      ensure(record.controller_observations.at(-1).helper_completed===true&&
        record.controller_observations.at(-1).helper_exit===1,"partial_json_preserves_failure");
    if(spec.name==="join_failure")
      ensure(record.cleanup_errors.includes("controller_join_unconfirmed")&&polls===2,
        "join_failure_no_retry");
    if(spec.name==="readback_projection_failure")
      ensure(record.cleanup_errors.includes("owned_readback_unavailable"),
        "projection_failure_separate");
    // Strong requested gate: Driver exception/refusal/late return also withhold release.
    // No source correction or expectation relaxation is applied to the immutable cell.
    ensure(result.resource_release_proven===spec.release,"resource_release_failclosed");
  }
  item.assertions_completed=true;
  }catch(error){
    item.assertion=typeof error?.envelope_assertion==="string"?error.envelope_assertion:
      "envelope_unexpected_failure";
  }
  return item;
}
async function runEnvelope(cell,source){
  const result={schema:"riauth.d01-cleanup-memory-envelope/v1",result:"failed",
    source,planned:PLAN,attempted:0,completed:0,cases:[],first_failure:null,
    unreached:CASES.map(v=>v.name),counts:{tool_calls:0,driver_calls:0,controller_calls:0,
      collector_calls:0,clock:0,stop:0,readback:0,persist:0}};
  ensure(sha(cell)===CELL_PIN,"immutable_cell_hash");
  const py=sources(cell);
  for(const spec of CASES){
    result.attempted++;
    try{
      const item=await oneCase(spec,cell,py);
      result.cases.push(item);
      for(const key of Object.keys(result.counts))result.counts[key]+=item.counts[key];
      result.unreached=CASES.slice(result.attempted).map(v=>v.name);
      if(!item.assertions_completed){
        result.first_failure={case:spec.name,assertion:item.assertion};
        return result;
      }
      result.completed++;
    }catch(error){
      result.first_failure={case:spec.name,
        assertion:typeof error?.envelope_assertion==="string"?error.envelope_assertion:
          "envelope_unexpected_failure"};
      result.unreached=CASES.slice(result.attempted).map(v=>v.name);
      return result;
    }
  }
  ensure(result.completed===PLAN.cases,"planned_cases");
  for(const key of Object.keys(result.counts))ensure(result.counts[key]===PLAN[key],"planned_total");
  result.result="passed";
  return result;
}
```

Complete exact standalone future stdin payload, NOT executed:

```js
import vm from "node:vm";
import {createHash} from "node:crypto";
const CELL="// Prospective ONE functions.exec cell; NOT executed in this design phase.\nconst CLOCK=\"import json,time\\nprint(json.dumps({'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())}))\\n\";\nconst STOP=\"import json,os,pathlib,stat,sys,time\\nc=json.loads(sys.argv[1])\\nclock={'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())}\\nrecord={'schema':'riauth.d01-first-observation/v1','first_failure':c['first_failure'],'first_observation_wall_ms':c['first_observation_wall_ms'],'first_event_proven':False,'clock':clock}\\nevent_state='write_unconfirmed'\\ntry:\\n    fd=os.open(pathlib.Path(c['event_out']),os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\\n    with os.fdopen(fd,'w',encoding='ascii') as f:\\n        f.write(json.dumps(record,sort_keys=True)+'\\\\n');f.flush();os.fsync(f.fileno())\\n    event_state='written'\\nexcept OSError:pass\\n# Attempt clock persistence BEFORE marker/state/budget comparisons.\\n# A metadata error does not prevent the original essential stop protocol.\\nlab=pathlib.Path(c['lab']);marker_state='lab_absent'\\ntry:\\n    if lab.exists():\\n        info=lab.lstat()\\n        if not stat.S_ISDIR(info.st_mode) or stat.S_IMODE(info.st_mode)!=0o700 or info.st_uid!=os.getuid():\\n            marker_state='ownership_unknown'\\n        else:\\n            marker_state='stop_requested'\\n            for name in ('ui-failure','stop'):\\n                try:\\n                    fd=os.open(lab/name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\\n                    os.close(fd)\\n                except FileNotFoundError:marker_state='lab_absent'\\n                except FileExistsError:pass\\nexcept OSError:marker_state='stop_unconfirmed'\\nprint(json.dumps({'clock':clock,'event_state':event_state,'marker_state':marker_state}))\\n\";\nconst READBACK=\"import json,pathlib,subprocess,sys,time\\nc=json.loads(sys.argv[1])\\npids=c['owned_pids']\\np=subprocess.run(['/bin/ps','-p',','.join(str(v) for v in pids),'-o','pid='],capture_output=True,timeout=3)\\nps_known=p.returncode in (0,1) and all(v.isdigit() for v in p.stdout.split())\\npresent=set(int(v) for v in p.stdout.split()) if ps_known else set()\\nports={}\\nfor port in (9000,3000):\\n    p=subprocess.run(['/usr/sbin/lsof','-nP','-t','-iTCP:'+str(port),'-sTCP:LISTEN'],capture_output=True,timeout=3)\\n    ports[str(port)]=not bool(p.stdout.strip()) if p.returncode in (0,1) else None\\nchildren=None\\nouter=pathlib.Path(c['outer_out'])\\nif outer.is_file():\\n    # This is the unchanged controller's fixed redacted evidence, not app data.\\n    data=json.loads(outer.read_bytes())\\n    allowed={'helper','whoami','discovery','confidential_client_create','operator_login','server','maintenance_init'}\\n    raw=data.get('owned_child_exits')\\n    if isinstance(raw,list) and len(raw)==7 and all(isinstance(v,dict) and v.get('name') in allowed and type(v.get('pid')) is int and type(v.get('exit')) is int for v in raw) and {v['name'] for v in raw}==allowed and len({v['pid'] for v in raw})==7 and next(v['pid'] for v in raw if v['name']=='helper')==c['helper_pid'] and next(v['pid'] for v in raw if v['name']=='server')==c['server_pid']:\\n        children=[{k:v[k] for k in ('name','pid','exit')} for v in raw]\\nprint(json.dumps({'clock':{'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())},'owned_pids':{str(v):(v not in present if ps_known else None) for v in pids},'ports':ports,'lab_absent':not pathlib.Path(c['lab']).exists(),'owned_child_exits':children}))\\n\";\nconst PERSIST=\"import json,os,pathlib,sys\\npath=pathlib.Path(sys.argv[1])\\nrecord=json.loads(sys.argv[2])\\n# Convert decimal strings directly to Python integers, avoiding JS Number rounding.\\ndef clocks(value):\\n    if isinstance(value,dict):\\n        for k,v in list(value.items()):\\n            if k in ('monotonic_ns','wall_epoch_ns') and isinstance(v,str):\\n                assert v.isdecimal() and len(v)<=19 and 0<=int(v)<2**63\\n                value[k]=int(v)\\n            else:clocks(v)\\n    elif isinstance(value,list):\\n        for v in value:clocks(v)\\nclocks(record)\\nfd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\\nwith os.fdopen(fd,'w',encoding='ascii') as f:\\n    f.write(json.dumps(record,sort_keys=True,indent=2)+'\\\\n');f.flush();os.fsync(f.fileno())\\nprint(json.dumps({'written_exclusive':True}))\\n\";\nconst own=load(\"d01_immediate_owned_handles\");\nconst OBSKEY=\"d01_immediate_observation_record\";\nif (!own || own.runtime_release!==true || own.driver_owned!==true ||\n    own.exact_bound!==true || own.single_controller!==true ||\n    own.fixture_ready!==true || own.listener_pid_proof!==true ||\n    own.fresh_paths_preflight!==true ||\n    !Number.isSafeInteger(own.start_epoch_ms) || typeof own.workspace!==\"string\" ||\n    new Set([own.guard_pid,own.server_pid,own.helper_pid,own.browser_pid]).size!==4 ||\n    ![own.guard_pid,own.server_pid,own.helper_pid,own.browser_pid,own.exec_session]\n       .every(v=>Number.isSafeInteger(v)&&v>0) ||\n    ![own.session,own.target_id,own.tab_id,own.lab,own.outer_out,\n       own.event_out,own.cleanup_out].every(v=>typeof v===\"string\"&&v.length>0)) {\n  text({proposal_refused:\"fresh_owned_context_required\"});exit();\n}\nconst record={\n  schema:\"riauth.d01-immediate-observation-cleanup/v1\",\n  first_failure:null,first_observation_wall_ms:null,\n  first_event_proven:false,whole_cleanup_within60_proven:false,\n  first_clock:null,observation_receipts:[],controller_observations:[],local_tool_receipts:[],actions:[],cleanup_errors:[],\n  final_absence:null,owned_child_exits:null,controller_exit:null,\n  first_observation_to_final_wall_ms:null,latch_to_absence_ms:null\n};\nconst sq=s=>\"'\"+String(s).replace(/'/g,\"'\\\\''\")+\"'\";\nconst retain=()=>store(OBSKEY,record);\nconst cleanupError=label=>{\n  if(!record.cleanup_errors.includes(label))record.cleanup_errors.push(label);\n  retain();\n};\nconst local=async(source,arg)=>{\n  const cmd=\"python3 -c \"+sq(source)+(arg===undefined?\"\":\" \"+sq(JSON.stringify(arg)));\n  const r=await tools.exec_command({cmd,workdir:own.workspace,\n    yield_time_ms:10000,max_output_tokens:2000});\n  record.local_tool_receipts.push({\n    label:source===CLOCK?\"clock\":source===STOP?\"stop\":source===READBACK?\"readback\":\"unknown\",\n    exit:typeof r.exit_code===\"number\"?r.exit_code:null,\n    session_id:typeof r.session_id===\"number\"?r.session_id:null\n  });retain(); // Numeric collector receipt BEFORE comparisons.\n  if(r.session_id!==undefined) {\n    cleanupError(\"owned_local_command_unjoined\");throw new Error(\"local_pending\");\n  }\n  if(r.exit_code!==0)throw new Error(\"local_failed\");\n  return JSON.parse(r.output);\n};\nconst ns=c=>BigInt(c.monotonic_ns);\nconst getClock=()=>local(CLOCK);\nlet controllerJoined=false;\nlet controllerPollAvailable=true;\nlet controllerBuffer=\"\";\nlet cleanupStarted=false;\nlet lastSnapshotFlags=null;\nfunction latch(label,receivedWall) {\n  if(record.first_failure===null) {\n    record.first_failure=label;\n    record.first_observation_wall_ms=receivedWall;\n    retain(); // Synchronous first-failure/wall latch BEFORE any new await/output.\n  }\n}\nfunction observeController(r,receivedWall) {\n  const observation={received_wall_ms:receivedWall,\n    exit:typeof r.exit_code===\"number\"?r.exit_code:null,\n    helper_completed:false,helper_exit:null,fixture_finished:false};\n  // Complete numeric result projection is retained BEFORE comparisons.\n  record.controller_observations.push(observation);retain();\n  if(typeof r.exit_code===\"number\") {\n    controllerJoined=true;record.controller_exit=r.exit_code;retain();\n  }\n  controllerBuffer+=typeof r.output===\"string\"?r.output:\"\";\n  if(controllerBuffer.length>16384) {\n    latch(\"controller_observation_invalid\",receivedWall);controllerBuffer=\"\";return;\n  }\n  let cut;\n  while((cut=controllerBuffer.indexOf(\"\\n\"))>=0) {\n    const line=controllerBuffer.slice(0,cut);controllerBuffer=controllerBuffer.slice(cut+1);\n    if(!line.trim())continue;\n    let event;\n    try{event=JSON.parse(line);}catch{\n      latch(\"controller_observation_invalid\",receivedWall);continue;\n    }\n    if(event.helper_completed===true) {\n      observation.helper_completed=true;\n      observation.helper_exit=typeof event.exit===\"number\"?event.exit:null;\n      retain();\n      if(event.exit!==0)latch(\"helper_failed\",receivedWall);\n      else if(!cleanupStarted)latch(\"helper_completed_before_app_checkpoint\",receivedWall);\n    }\n    if(event.fixture_finished===true) {\n      observation.fixture_finished=true;retain();\n      if(!cleanupStarted)latch(\"controller_completed_before_app_checkpoint\",receivedWall);\n    }\n  }\n  if(controllerJoined&&!cleanupStarted)\n    latch(\"controller_completed_before_app_checkpoint\",receivedWall);\n}\nasync function pollController() {\n  if(controllerJoined||!controllerPollAvailable)return;\n  try {\n    const r=await tools.write_stdin({session_id:own.exec_session,\n      chars:\"\",yield_time_ms:5000,max_output_tokens:2000});\n    const receivedWall=Date.now();\n    observeController(r,receivedWall);\n  } catch {\n    controllerPollAvailable=false;\n    latch(\"controller_observation_unavailable\",Date.now());\n    if(cleanupStarted)cleanupError(\"controller_join_unconfirmed\");\n  }\n}\nasync function timedDriver(label,allocation,operation,project) {\n  let start=null,end=null,result=null,state=\"unknown\";\n  try{start=await getClock();}catch{cleanupError(\"clock_unavailable\");}\n  const action={label,start_clock:start,end_clock:null,allowed_ms:allocation,\n    result_state:\"unknown\",elapsed_ms:null,over_budget:null};\n  record.actions.push(action);retain(); // Start retained BEFORE entering Driver call.\n  try {\n    result=await operation();\n    state=result?.isError===true?\"refused\":\"returned\";\n  } catch {state=\"exception\";}\n  try{end=await getClock();}catch{cleanupError(\"clock_unavailable\");}\n  action.end_clock=end;action.result_state=state;\n  retain(); // End/result retained BEFORE budget comparison.\n  if(start&&end) {\n    const d=ns(end)-ns(start);\n    if(d>=0n) {\n      action.elapsed_ms=Number(d/1000000n);\n      action.over_budget=action.elapsed_ms>allocation;\n    } else cleanupError(\"clock_invalid\");\n  }\n  if(action.over_budget)cleanupError(\"driver_operation_over_budget\");\n  if(state!==\"returned\")cleanupError(\"driver_operation_unconfirmed\");\n  if(project)project(result,state);\n  retain();\n}\nasync function cleanup() {\n  if(cleanupStarted)return;\n  cleanupStarted=true;\n  try {\n    const r=await local(STOP,{\n      lab:own.lab,event_out:own.event_out,\n      first_failure:record.first_failure,\n      first_observation_wall_ms:record.first_observation_wall_ms\n    });\n    record.first_clock=r.clock;record.stop_state=r.marker_state;retain();\n    if(r.event_state!==\"written\")cleanupError(\"observation_metadata_write_unconfirmed\");\n    if(r.marker_state===\"stop_unconfirmed\")cleanupError(\"stop_unconfirmed\");\n    if(r.marker_state===\"ownership_unknown\")cleanupError(\"ownership_unknown\");\n  } catch {cleanupError(\"stop_or_clock_record_unavailable\");}\n  // No outstanding Driver call exists here: all page calls above were awaited.\n  // Original stop protocol already causes the controller's owned-child finally.\n  await timedDriver(\"kill_app\",30000,\n    ()=>tools.mcp__cua_driver__kill_app({pid:own.browser_pid}));\n  await timedDriver(\"end_session\",15000,\n    ()=>tools.mcp__cua_driver__end_session({session:own.session}),(r,state)=>{\n      record.session_ended=state===\"returned\"&&\n        r?.structuredContent?.active===false&&r.structuredContent.session===own.session;\n      retain();\n    });\n  await timedDriver(\"list_windows\",5000,\n    ()=>tools.mcp__cua_driver__list_windows({pid:own.browser_pid}),(r,state)=>{\n      const windows=r?.structuredContent?.windows;\n      record.window_count=state===\"returned\"&&Array.isArray(windows)?windows.length:null;\n      retain();\n    });\n  let readStart=null;\n  try{readStart=await getClock();}catch{cleanupError(\"clock_unavailable\");}\n  const action={label:\"owned_join_readback\",start_clock:readStart,end_clock:null,\n    allowed_ms:null,elapsed_ms:null,over_budget:null,result_state:\"unknown\"};\n  record.actions.push(action);retain();\n  if(readStart&&record.first_clock) {\n    action.allowed_ms=Math.max(0,60000-Number((ns(readStart)-ns(record.first_clock))/1000000n));\n    retain();\n  }\n  // Continue essential join after60s if late; never claim that deadline enforced.\n  // Do not poll another exec session or send a manual process signal.\n  while(!controllerJoined&&controllerPollAvailable&&Date.now()<own.start_epoch_ms+900000) {\n    await pollController();\n    if(record.first_clock) {\n      try {\n        const c=await getClock();record.last_join_clock=c;retain();\n        if(ns(c)-ns(record.first_clock)>60000000000n)\n          cleanupError(\"cleanup_budget_exceeded\");\n      } catch{cleanupError(\"clock_unavailable\");}\n    }\n  }\n  if(!controllerJoined)cleanupError(\"child_reap_incomplete\");\n  try {\n    const r=await local(READBACK,{\n      owned_pids:[own.guard_pid,own.server_pid,own.helper_pid,own.browser_pid],\n      lab:own.lab,outer_out:own.outer_out,helper_pid:own.helper_pid,server_pid:own.server_pid\n    });\n    action.end_clock=r.clock;action.result_state=\"returned\";\n    record.owned_child_exits=r.owned_child_exits;\n    record.final_absence={owned_pids:r.owned_pids,ports:r.ports,\n      lab:r.lab_absent,window_count:record.window_count??null,\n      session_ended:record.session_ended===true};\n    record.first_observation_to_final_wall_ms=Date.now()-record.first_observation_wall_ms;\n    retain(); // Full fixed readback/exits/clock BEFORE comparisons.\n    if(readStart) {\n      action.elapsed_ms=Number((ns(r.clock)-ns(readStart))/1000000n);\n      action.over_budget=action.allowed_ms===null?null:action.elapsed_ms>action.allowed_ms;\n    }\n    if(record.first_clock)\n      record.latch_to_absence_ms=Number((ns(r.clock)-ns(record.first_clock))/1000000n);\n    if(action.over_budget)cleanupError(\"cleanup_budget_exceeded\");\n    const a=record.final_absence;\n    if(!controllerJoined||!Array.isArray(record.owned_child_exits)||\n       record.owned_child_exits.length!==7)cleanupError(\"child_reap_incomplete\");\n    if(!Object.values(a.owned_pids).every(v=>v===true)||\n       !Object.values(a.ports).every(v=>v===true)||a.lab!==true||\n       a.window_count!==0||a.session_ended!==true)\n      cleanupError(\"owned_resource_absence_unproven\");\n  } catch {action.result_state=\"exception\";cleanupError(\"owned_readback_unavailable\");}\n  cleanupError(\"first_event_unproven\");\n  record.whole_cleanup_within60_proven=false;retain();\n  // Exclusive new file only; existing outer/helper/provider evidence untouched.\n  try {\n    const cmd=\"python3 -c \"+sq(PERSIST)+\" \"+sq(own.cleanup_out)+\" \"+sq(JSON.stringify(record));\n    const r=await tools.exec_command({cmd,workdir:own.workspace,\n      yield_time_ms:10000,max_output_tokens:500});\n    record.local_tool_receipts.push({label:\"persist\",\n      exit:typeof r.exit_code===\"number\"?r.exit_code:null,\n      session_id:typeof r.session_id===\"number\"?r.session_id:null});retain();\n    if(r.session_id!==undefined)cleanupError(\"owned_local_command_unjoined\");\n    if(r.session_id!==undefined||r.exit_code!==0)\n      cleanupError(\"cleanup_metadata_write_unconfirmed\");\n  } catch {cleanupError(\"cleanup_metadata_write_unconfirmed\");}\n  controllerBuffer=\"\";\n}\nawait pollController();\nif(record.first_failure===null) {\n  try {\n    const r=await tools.mcp__cua_driver__browser_navigate({\n      session:own.session,target_id:own.target_id,tab_id:own.tab_id,\n      url:\"http://localhost:3000/\"});\n    const receivedWall=Date.now();\n    record.observation_receipts.push({kind:\"browser_navigation\",received_wall_ms:receivedWall});\n    retain(); // Received clock BEFORE navigation outcome comparison.\n    if(r?.isError===true)latch(\"driver_navigation_refused\",receivedWall);\n  } catch {latch(\"driver_navigation_exception\",Date.now());}\n}\nif(record.first_failure===null) {\n  try {\n    const r=await tools.mcp__cua_driver__get_browser_state({\n      session:own.session,target_id:own.target_id,tab_id:own.tab_id,\n      snapshot_format:\"semantic_v2\",include_screenshot:false});\n    const receivedWall=Date.now();\n    record.observation_receipts.push({kind:\"browser_snapshot\",received_wall_ms:receivedWall});\n    retain(); // Received clock BEFORE any page predicate comparison.\n    const s=r?.structuredContent;\n    const nodes=Array.isArray(s?.content_refs)?s.content_refs:[];\n    lastSnapshotFlags={\n      received_wall_ms:receivedWall,\n      status_ok:r?.isError!==true&&s?.status===\"ok\",\n      complete:s?.snapshot?.complete===true,\n      heading_match:nodes.some(n=>n.role===\"heading\"&&n.name===\"Local demo\"),\n      error_match:nodes.some(n=>n.name===\"Local demo could not complete this request.\"),\n      interactive_ref_present:Array.isArray(s?.refs)&&\n        s.refs.some(n=>Array.isArray(n.actions)&&n.actions.includes(\"click\"))\n    };\n    store(\"d01_immediate_snapshot_flags\",lastSnapshotFlags);\n    // Finite booleans only; no outline/URL/name/value/subject/token is persisted.\n    if(lastSnapshotFlags.error_match)latch(\"application_page_failed\",receivedWall);\n    else if(!lastSnapshotFlags.status_ok||!lastSnapshotFlags.complete||\n            !lastSnapshotFlags.heading_match||!lastSnapshotFlags.interactive_ref_present)\n      latch(\"application_snapshot_unconfirmed\",receivedWall);\n  } catch {latch(\"driver_snapshot_exception\",Date.now());}\n}\nif(record.first_failure===null)await pollController();\nif(record.first_failure!==null) {\n  await cleanup(); // NO text/notify/yield_control/model turn before essential cleanup.\n  text({result:\"failed\",first_failure:record.first_failure,\n    cleanup_errors:record.cleanup_errors,final_absence:record.final_absence,\n    controller_exit:record.controller_exit,whole_cleanup_within60_proven:false,\n    resource_release_proven:record.final_absence!==null&&\n      !record.cleanup_errors.some(v=>[\n        \"child_reap_incomplete\",\"owned_resource_absence_unproven\",\n        \"owned_readback_unavailable\",\"owned_local_command_unjoined\"].includes(v))});\n} else {\n  text({result:\"application_checkpoint_observed_only\",\n    snapshot_flags:lastSnapshotFlags,journey_credit:false});\n}\n";
const LOGIC="const CELL_PIN=\"0a7cea070bdab32b419a82eb26a7a991b71a2b14a55553f890b53788cda04a5e\";\nconst PYTHON_PINS={\n  CLOCK:\"cfb3f3154cb778388724728b6f7cb804bd7e142844ded7c044afc8dd3135ab4d\",\n  STOP:\"ccc2702c4e10209952d8e5489c2974a09a7f4d6c47a3705dc89d5bb028ec3ffb\",\n  READBACK:\"95c6fe05f0016ef302dd9c63533b95141b09ea71a661ab390c112c9e6c13ccb3\",\n  PERSIST:\"819a7785625ce2f78a9cb03b5f242dbfa6728f9f507b242e3c048887ad01ea30\"\n};\nconst CASES=[\n  {name:\"ui_error\",first:\"application_page_failed\",release:true,tools:18},\n  {name:\"helper_failure\",first:\"helper_failed\",release:true,tools:16},\n  {name:\"controller_completion\",first:\"controller_completed_before_app_checkpoint\",release:true,tools:14},\n  {name:\"metadata_write_failure\",first:\"application_page_failed\",release:true,tools:18},\n  {name:\"driver_exception\",first:\"application_page_failed\",release:false,tools:18},\n  {name:\"driver_refusal\",first:\"application_page_failed\",release:false,tools:18},\n  {name:\"driver_late_return\",first:\"application_page_failed\",release:false,tools:18},\n  {name:\"missing_child_proof\",first:\"application_page_failed\",release:false,tools:18},\n  {name:\"missing_pid_proof\",first:\"application_page_failed\",release:false,tools:18},\n  {name:\"missing_window_proof\",first:\"application_page_failed\",release:false,tools:18},\n  {name:\"missing_session_proof\",first:\"application_page_failed\",release:false,tools:18},\n  {name:\"missing_port_proof\",first:\"application_page_failed\",release:false,tools:18},\n  {name:\"missing_lab_proof\",first:\"application_page_failed\",release:false,tools:18},\n  {name:\"partial_controller_json\",first:\"application_page_failed\",release:true,tools:18},\n  {name:\"join_failure\",first:\"application_page_failed\",release:false,tools:18},\n  {name:\"readback_projection_failure\",first:\"application_page_failed\",release:false,tools:18},\n  {name:\"missing_ownership\",first:null,release:null,tools:0},\n  {name:\"application_confirmed\",first:null,release:null,tools:4}\n];\nconst PLAN={cases:18,tool_calls:286,driver_calls:78,controller_calls:33,\n  collector_calls:175,clock:127,stop:16,readback:16,persist:16,\n  cleanup_sequences:16,outputs:18};\nconst clone=v=>JSON.parse(JSON.stringify(v));\nfunction ensure(ok,label){if(!ok)throw {envelope_assertion:label};}\nfunction shellWords(command){\n  ensure(typeof command===\"string\"&&command.length<65536,\"command_bounds\");\n  const words=[];let word=\"\",quoted=false,started=false;\n  for(let i=0;i<command.length;i++){\n    const c=command[i];\n    if(quoted){if(c===\"'\")quoted=false;else word+=c;continue;}\n    if(c===\"'\"){quoted=true;started=true;continue;}\n    if(c===\"\\\\\"){ensure(i+1<command.length,\"command_escape\");word+=command[++i];started=true;continue;}\n    if(/\\s/.test(c)){if(started){words.push(word);word=\"\";started=false;}continue;}\n    word+=c;started=true;\n  }\n  ensure(!quoted,\"command_quote\");\n  if(started)words.push(word);\n  return words;\n}\nfunction sources(cell){\n  const result={};\n  for(const name of Object.keys(PYTHON_PINS)){\n    const line=cell.split(\"\\n\").find(v=>v.startsWith(\"const \"+name+\"=\"));\n    ensure(typeof line===\"string\",\"embedded_source_missing\");\n    result[name]=JSON.parse(line.slice((\"const \"+name+\"=\").length,-1));\n    ensure(sha(result[name])===PYTHON_PINS[name],\"embedded_source_hash\");\n  }\n  return result;\n}\nfunction ownContext(){\n  return {runtime_release:true,driver_owned:true,exact_bound:true,single_controller:true,\n    fixture_ready:true,listener_pid_proof:true,fresh_paths_preflight:true,\n    workspace:\"/synthetic/d01\",guard_pid:101,server_pid:102,helper_pid:103,browser_pid:104,\n    exec_session:901,session:\"synthetic-session\",target_id:\"synthetic-target\",tab_id:\"synthetic-tab\",\n    lab:\"/synthetic/lab\",outer_out:\"/synthetic/outer\",event_out:\"/synthetic/event\",\n    cleanup_out:\"/synthetic/cleanup\",start_epoch_ms:1700000000000};\n}\nasync function oneCase(spec,cell,py){\n  const own=ownContext();\n  if(spec.name===\"missing_ownership\")own.exact_bound=false;\n  const memory=new Map(),trace=[],outputs=[],counts={\n    tool_calls:0,driver_calls:0,controller_calls:0,collector_calls:0,\n    clock:0,stop:0,readback:0,persist:0\n  };\n  let tick=1000,stores=0,dateCalls=0,polls=0,firstLatch=null,retainedFirst=null;\n  let cleanupRecord=null,stopRequested=false,exitSeen=false;\n  const EXIT={synthetic_exit:true};\n  const push=(kind,label)=>trace.push({kind,label,tick});\n  const now=()=>{ensure(++dateCalls<=128,\"date_bound\");return own.start_epoch_ms+tick;};\n  const clock=()=>({monotonic_ns:String(1000000000000n+BigInt(tick)*1000000n),\n    wall_epoch_ns:String(BigInt(own.start_epoch_ms+tick)*1000000n)});\n  function call(label,domain,cleanup){\n    ensure(++counts.tool_calls<=32,\"tool_bound\");\n    counts[domain]++;\n    push(\"tool\",label);tick+=10;\n    if(cleanup){\n      const r=memory.get(\"d01_immediate_observation_record\");\n      ensure(firstLatch!==null&&r?.first_failure===spec.first,\"latch_before_cleanup\");\n      ensure(outputs.length===0,\"no_output_before_cleanup\");\n    }\n  }\n  const toolResult=v=>({exit_code:0,output:JSON.stringify(v)+\"\\n\"});\n  const children=()=>[\"helper\",\"whoami\",\"discovery\",\"confidential_client_create\",\n    \"operator_login\",\"server\",\"maintenance_init\"].map((name,i)=>({\n      name,pid:name===\"helper\"?103:name===\"server\"?102:200+i,exit:name===\"helper\"?1:0}));\n  const tools={\n    exec_command:async args=>{\n      ensure(args.workdir===own.workspace&&args.yield_time_ms===10000,\"collector_shape\");\n      const words=shellWords(args.cmd);\n      ensure(words[0]===\"python3\"&&words[1]===\"-c\",\"collector_command\");\n      const name=Object.keys(py).find(n=>py[n]===words[2]);\n      ensure(name!==undefined,\"collector_source_identity\");\n      call(name.toLowerCase(),\"collector_calls\",true);counts[name.toLowerCase()]++;\n      if(name===\"CLOCK\"){\n        ensure(words.length===3,\"clock_arguments\");\n        return toolResult(clock());\n      }\n      if(name===\"STOP\"){\n        ensure(words.length===4,\"stop_arguments\");\n        const c=JSON.parse(words[3]);\n        ensure(c.lab===own.lab&&c.event_out===own.event_out&&c.first_failure===spec.first,\n          \"stop_owned_arguments\");\n        ensure(c.first_observation_wall_ms===retainedFirst,\"stop_received_clock\");\n        push(\"marker\",\"original_stop_requested\");stopRequested=true;\n        return toolResult({clock:clock(),event_state:spec.name===\"metadata_write_failure\"?\n          \"write_unconfirmed\":\"written\",marker_state:\"stop_requested\"});\n      }\n      if(name===\"READBACK\"){\n        ensure(words.length===4,\"readback_arguments\");\n        const c=JSON.parse(words[3]);\n        ensure(JSON.stringify(c.owned_pids)===JSON.stringify([101,102,103,104])&&\n          c.lab===own.lab&&c.outer_out===own.outer_out&&c.helper_pid===103&&c.server_pid===102,\n          \"readback_owned_arguments\");\n        if(spec.name===\"readback_projection_failure\")return {exit_code:0,output:\"{\"};\n        const unknown=spec.name===\"missing_pid_proof\";\n        return toolResult({clock:clock(),owned_pids:Object.fromEntries(\n          [101,102,103,104].map(p=>[String(p),unknown?null:true])),\n          ports:{\"9000\":true,\"3000\":spec.name===\"missing_port_proof\"?null:true},\n          lab_absent:spec.name===\"missing_lab_proof\"?null:true,\n          owned_child_exits:spec.name===\"missing_child_proof\"?null:children()});\n      }\n      ensure(name===\"PERSIST\"&&words.length===5&&words[3]===own.cleanup_out,\"persist_arguments\");\n      cleanupRecord=JSON.parse(words[4]);\n      push(\"persist\",\"exclusive_cleanup_projection\");\n      return {exit_code:0,output:'{\"written_exclusive\":true}\\n'};\n    },\n    write_stdin:async args=>{\n      ensure(args.session_id===901&&args.chars===\"\"&&args.yield_time_ms===5000,\n        \"controller_owned_arguments\");\n      call(\"controller_poll\",\"controller_calls\",polls>0&&spec.name!==\"application_confirmed\");\n      polls++;\n      ensure(polls<=2,\"controller_poll_bound\");\n      if(polls===1){\n        if(spec.name===\"helper_failure\")return {session_id:901,\n          output:'{\"helper_completed\":true,\"exit\":1}\\n'};\n        if(spec.name===\"controller_completion\")return {exit_code:1,\n          output:'{\"fixture_finished\":true}\\n'};\n        if(spec.name===\"partial_controller_json\")return {session_id:901,\n          output:'{\"helper_completed\":'};\n        return {session_id:901,output:\"\"};\n      }\n      if(spec.name===\"application_confirmed\")return {session_id:901,output:\"\"};\n      if(spec.name===\"join_failure\")throw {synthetic_failure:true};\n      if(spec.name===\"partial_controller_json\")return {exit_code:1,\n        output:'true,\"exit\":1}\\n'};\n      return {exit_code:1,output:'{\"fixture_finished\":true}\\n'};\n    },\n    mcp__cua_driver__browser_navigate:async args=>{\n      ensure(args.session===own.session&&args.target_id===own.target_id&&\n        args.tab_id===own.tab_id&&args.url===\"http://localhost:3000/\",\"navigation_binding\");\n      call(\"browser_navigate\",\"driver_calls\",false);\n      return {content:[{type:\"text\",text:\"synthetic navigation returned\"}]};\n    },\n    mcp__cua_driver__get_browser_state:async args=>{\n      ensure(args.session===own.session&&args.target_id===own.target_id&&\n        args.tab_id===own.tab_id&&args.snapshot_format===\"semantic_v2\"&&\n        args.include_screenshot===false,\"snapshot_binding\");\n      call(\"browser_snapshot\",\"driver_calls\",false);\n      const good=spec.name===\"application_confirmed\";\n      const content_refs=[{role:\"heading\",name:\"Local demo\",actions:[],frame:\"main\"}];\n      if(!good)content_refs.push({role:\"statictext\",\n        name:\"Local demo could not complete this request.\",actions:[],frame:\"main\"});\n      return {structuredContent:{status:\"ok\",mode:\"snapshot\",\n        snapshot:{complete:true,format:\"semantic_v2\",id:\"synthetic-snapshot\"},\n        content_refs,refs:good?[{role:\"button\",name:\"synthetic action\",\n          ref:\"synthetic-snapshot:1\",actions:[\"click\"],frame:\"main\"}]:[]}};\n    },\n    mcp__cua_driver__kill_app:async args=>{\n      ensure(args.pid===104&&Object.keys(args).length===1,\"kill_owned_arguments\");\n      call(\"kill_app\",\"driver_calls\",true);\n      if(spec.name===\"driver_exception\")throw {synthetic_failure:true};\n      if(spec.name===\"driver_refusal\")return {isError:true,\n        content:[{type:\"text\",text:\"synthetic refusal\"}]};\n      if(spec.name===\"driver_late_return\")tick+=31000;\n      return {content:[{type:\"text\",text:\"synthetic kill returned\"}]};\n    },\n    mcp__cua_driver__end_session:async args=>{\n      ensure(args.session===own.session,\"end_owned_arguments\");\n      call(\"end_session\",\"driver_calls\",true);\n      if(spec.name===\"missing_session_proof\")return {content:[]};\n      return {structuredContent:{active:false,session:own.session}};\n    },\n    mcp__cua_driver__list_windows:async args=>{\n      ensure(args.pid===104,\"windows_owned_arguments\");\n      call(\"list_windows\",\"driver_calls\",true);\n      return spec.name===\"missing_window_proof\"?{content:[]}:\n        {structuredContent:{current_space_id:1,windows:[]}};\n    }\n  };\n  const sandbox={\n    tools,Date:class {static now(){return now();}},\n    load:key=>{ensure(key===\"d01_immediate_owned_handles\",\"load_key\");return clone(own);},\n    store:(key,value)=>{\n      ensure(++stores<=512,\"store_bound\");\n      ensure([\"d01_immediate_observation_record\",\"d01_immediate_snapshot_flags\"].includes(key),\n        \"store_key\");\n      const copy=clone(value);memory.set(key,copy);\n      if(key===\"d01_immediate_observation_record\"&&copy.first_failure!==null){\n        if(firstLatch===null){\n          firstLatch=trace.length;retainedFirst=copy.first_observation_wall_ms;\n          push(\"latch\",copy.first_failure);\n        }\n        ensure(copy.first_failure===spec.first,\"first_failure_immutable\");\n      }\n    },\n    text:value=>{\n      push(\"output\",\"fixed_result\");outputs.push(clone(value));\n      ensure(outputs.length===1,\"output_bound\");\n    },\n    notify:()=>{throw {envelope_assertion:\"notification_forbidden\"};},\n    yield_control:()=>{throw {envelope_assertion:\"model_yield_forbidden\"};},\n    exit:()=>{exitSeen=true;throw EXIT;}\n  };\n  let cellError=null;\n  try{\n    const script=new vm.Script(\"(async()=>{\\n\"+cell+\"\\n})()\",{filename:\"immutable-cell\"});\n    const promise=script.runInNewContext(sandbox,{timeout:1000});\n    await promise;\n  }catch(error){if(error!==EXIT)cellError=error;}\n  // Retain complete fixed outcome/counts BEFORE any verification assertion.\n  const item={case:spec.name,counts:clone(counts),first_failure:outputs[0]?.first_failure??null,\n    resource_release_proven:outputs[0]?.resource_release_proven??null,\n    assertions_completed:false,assertion:null};\n  try {\n  ensure(cellError===null,\"immutable_cell_exception\");\n  ensure(counts.tool_calls===spec.tools,\"planned_tool_count\");\n  ensure(outputs.length===1,\"exact_output_count\");\n  const result=outputs[0];\n  if(spec.name===\"missing_ownership\"){\n    ensure(exitSeen&&counts.tool_calls===0&&firstLatch===null&&\n      result.proposal_refused===\"fresh_owned_context_required\",\"ownership_refuses_before_tools\");\n  }else if(spec.name===\"application_confirmed\"){\n    ensure(result.result===\"application_checkpoint_observed_only\"&&result.journey_credit===false&&\n      firstLatch===null&&counts.collector_calls===0&&counts.driver_calls===2,\n      \"confirmed_not_journey_or_cleanup\");\n  }else{\n    const record=memory.get(\"d01_immediate_observation_record\");\n    ensure(result.result===\"failed\"&&result.first_failure===spec.first&&\n      record.first_failure===spec.first,\"failure_preserved\");\n    ensure(record.first_observation_wall_ms===retainedFirst,\"first_clock_preserved\");\n    ensure(record.first_event_proven===false&&record.whole_cleanup_within60_proven===false&&\n      result.whole_cleanup_within60_proven===false,\"no_actual_anchor_or_timing_credit\");\n    ensure(stopRequested&&counts.stop===1&&counts.persist===1&&cleanupRecord!==null,\n      \"original_stop_and_exclusive_projection\");\n    const order=[\"stop\",\"kill_app\",\"end_session\",\"list_windows\",\"readback\",\"persist\"];\n    const positions=order.map(label=>trace.findIndex(v=>v.kind===\"tool\"&&v.label===label));\n    ensure(positions.every(p=>p>firstLatch)&&positions.every((p,i)=>i===0||p>positions[i-1]),\n      \"latch_then_cleanup_order\");\n    ensure(trace.at(-1).kind===\"output\",\"no_output_until_cleanup_returned\");\n    ensure(!trace.some(v=>v.kind===\"tool\"&&[\"browser_navigate\",\"browser_snapshot\"].includes(v.label)&&\n      trace.indexOf(v)>firstLatch),\"no_journey_after_failure\");\n    ensure(record.actions.length===4&&record.actions[0].label===\"kill_app\"&&\n      record.actions[1].label===\"end_session\"&&record.actions[2].label===\"list_windows\"&&\n      record.actions[3].label===\"owned_join_readback\",\"exact_action_projection\");\n    if(spec.name===\"metadata_write_failure\")\n      ensure(record.cleanup_errors.includes(\"observation_metadata_write_unconfirmed\"),\n        \"metadata_failure_separate\");\n    if([\"driver_exception\",\"driver_refusal\"].includes(spec.name))\n      ensure(record.cleanup_errors.includes(\"driver_operation_unconfirmed\"),\"driver_failure_recorded\");\n    if(spec.name===\"driver_late_return\")\n      ensure(record.cleanup_errors.includes(\"driver_operation_over_budget\")&&\n        record.actions[0].elapsed_ms>30000,\"late_return_observed_only\");\n    if(spec.name===\"partial_controller_json\")\n      ensure(record.controller_observations.at(-1).helper_completed===true&&\n        record.controller_observations.at(-1).helper_exit===1,\"partial_json_preserves_failure\");\n    if(spec.name===\"join_failure\")\n      ensure(record.cleanup_errors.includes(\"controller_join_unconfirmed\")&&polls===2,\n        \"join_failure_no_retry\");\n    if(spec.name===\"readback_projection_failure\")\n      ensure(record.cleanup_errors.includes(\"owned_readback_unavailable\"),\n        \"projection_failure_separate\");\n    // Strong requested gate: Driver exception/refusal/late return also withhold release.\n    // No source correction or expectation relaxation is applied to the immutable cell.\n    ensure(result.resource_release_proven===spec.release,\"resource_release_failclosed\");\n  }\n  item.assertions_completed=true;\n  }catch(error){\n    item.assertion=typeof error?.envelope_assertion===\"string\"?error.envelope_assertion:\n      \"envelope_unexpected_failure\";\n  }\n  return item;\n}\nasync function runEnvelope(cell,source){\n  const result={schema:\"riauth.d01-cleanup-memory-envelope/v1\",result:\"failed\",\n    source,planned:PLAN,attempted:0,completed:0,cases:[],first_failure:null,\n    unreached:CASES.map(v=>v.name),counts:{tool_calls:0,driver_calls:0,controller_calls:0,\n      collector_calls:0,clock:0,stop:0,readback:0,persist:0}};\n  ensure(sha(cell)===CELL_PIN,\"immutable_cell_hash\");\n  const py=sources(cell);\n  for(const spec of CASES){\n    result.attempted++;\n    try{\n      const item=await oneCase(spec,cell,py);\n      result.cases.push(item);\n      for(const key of Object.keys(result.counts))result.counts[key]+=item.counts[key];\n      result.unreached=CASES.slice(result.attempted).map(v=>v.name);\n      if(!item.assertions_completed){\n        result.first_failure={case:spec.name,assertion:item.assertion};\n        return result;\n      }\n      result.completed++;\n    }catch(error){\n      result.first_failure={case:spec.name,\n        assertion:typeof error?.envelope_assertion===\"string\"?error.envelope_assertion:\n          \"envelope_unexpected_failure\"};\n      result.unreached=CASES.slice(result.attempted).map(v=>v.name);\n      return result;\n    }\n  }\n  ensure(result.completed===PLAN.cases,\"planned_cases\");\n  for(const key of Object.keys(result.counts))ensure(result.counts[key]===PLAN[key],\"planned_total\");\n  result.result=\"passed\";\n  return result;\n}\n";
const sha=s=>createHash("sha256").update(s).digest("hex");
const source={cell_sha256:sha(CELL),logic_sha256:sha(LOGIC)};
let result;
try {
  const script=new vm.Script(LOGIC+"\nrunEnvelope(CELL,source);",{filename:"finite-envelope"});
  result=await script.runInNewContext({vm,sha,CELL,source},{timeout:1000});
} catch {
  result={schema:"riauth.d01-cleanup-memory-envelope/v1",result:"failed",source,
    first_failure:{case:null,assertion:"envelope_preflight_or_totals_failed"}};
}
process.stdout.write(JSON.stringify(result)+"\n");
process.exitCode=result.result==="passed"?0:1;
```

Payload reconstruction is mechanical: JSON-decode ONLY the CELL and LOGIC
constant lines; CELL must equal the entire prior284-line archive and LOGIC
must equal the readable322-line logic above. All four embedded Python constants
must remain identical to the preceding source pins. No filesystem harness/
controller/helper file is materialized.

### Actual static checks and remaining gate

Actually performed: node --input-type=module --check on the complete readable
logic and complete standalone payload, each exit0; four immutable collector
AST parses/in-memory code-object compiles; exact CELL/LOGIC JSON reconstruction;
full original-cell/prefix pin checks; static18-entry/286-call sum and unique
case-name checks; no forbidden filesystem/process/network/timer import or
Promise.race in the harness; helper/controller identity and twelve metadata
mode/byte/hash preservation; fence/final-newline/whitespace/report-only checks.
No VM evaluation, mocked cell, stub, async harness, imported module body,
Python collector or test is executed. Actual memory case count is ZERO.
Syntax/AST acceptance is not a proof that cases pass or that strict release
gating is implemented.

Root must first resolve the explicitly exposed Driver-error/all-absence gate,
then review this exact immutable envelope before separately reserving any
memory execution. No source or expectation is changed under this reservation.
A future run must stop at its first unexpected assertion and preserve complete
fixed counts/output, including attempted-but-failed and unreached cases.
No actual/full D01/D05/GUI/crypto/tenant/physical/hardware/release gate claim is
made; the real fixture remains HELD pending its distinct observer-aware memory
result and cleanup composition review. Historic first-event/whole60 unknown
and every prior failure remain intact.

The entire382782-byte3f957001 prefix/SHA
da1e543c3cd4229e7307921e5c4109d9ad30c6e7b5c9d13a8a6e1d798aa60286
is byte-preserved. All nine older0600 metadata files plus three c88 records
remain byte/hash/mode exact. Lost provider values/reporting correction, failed
61/76 and dated memory78 limits, unknown request senders, all refused/failed
fixtures and c88's actual119s delayed readback are not refreshed or discarded.
Only this existing report is appended and committed. No source/helper/
observer/controller/product/guide/D05/test/config edit, extra file, actual
provider/collector/CLI/HTTP/listener/browser/Driver/Cargo/filecleanup invocation,
worker/task/WT/shell/main/push/status or contact occurs. No runtime slot is
acquired or released; original D01/D05 disposition remains root-owned.

## Dated design correction: resource absence versus cleanup budget, 2026-10-02

Reservation wave30_D01_cleanup_absence_vs_budget_envelope_correction owns ONLY
this append. Root explicitly corrected the overstrict assignment wording:
resource RELEASE means complete independent ownership/absence/join/session/
window/port/lab proofs, even when a Driver operation failed or returned late.
The Driver error and budget result remain separate and failed; whole60 is
never proven by this cell. Missing any actual ownership/absence/join proof
still prevents a release claim.

This is a requirement/expectation correction, not a product bug, source repair
or executed failure. The prior strict01f249c design, its five-line-different
logic/full payload and source-derived case5 failure prediction remain dated
and byte-preserved. That prediction was NEVER an actual memory result and is
not silently relabelled as one. c88's real generic failure, zeroAuth, no journey
and119s delayed absence readback remain unchanged. The old first-event/whole60
unknown limits are retained; no retrospective cleanup pass is claimed.

### Exactly five harness-only edits

Only these release expectations change false to true:
driver_exception, driver_refusal, driver_late_return.
Their stubs still model complete independent resource absence. Existing
driver_operation_unconfirmed assertions for exception/refusal and the
driver_operation_over_budget/elapsed>30000 assertion for late return stay
byte-exact and occur BEFORE the release comparison. First-failure/clock/error
retention and always-false first-event/whole60 assertions are unchanged.

Only the two misleading strict-gate comments are also corrected. No other
logic/payload header/cell/collector/controller/helper source or assertion is
changed. In particular, the six missing-proof cases, failed join, invalid
readback, partial controller JSON, missing ownership and application-confirmed
journey_credit false behavior remain unchanged.

Exact zero-context, two-hunk/five-line readable-logic diff:

```diff
--- original readable logic e3d39dc4
+++ corrected readable logic b68ee1a0
@@ -13,3 +13,3 @@
-  {name:"driver_exception",first:"application_page_failed",release:false,tools:18},
-  {name:"driver_refusal",first:"application_page_failed",release:false,tools:18},
-  {name:"driver_late_return",first:"application_page_failed",release:false,tools:18},
+  {name:"driver_exception",first:"application_page_failed",release:true,tools:18},
+  {name:"driver_refusal",first:"application_page_failed",release:true,tools:18},
+  {name:"driver_late_return",first:"application_page_failed",release:true,tools:18},
@@ -280,2 +280,2 @@
-    // Strong requested gate: Driver exception/refusal/late return also withhold release.
-    // No source correction or expectation relaxation is applied to the immutable cell.
+    // Resource release uses independent absence proofs; Driver errors remain recorded.
+    // The immutable cell and all error, elapsed and always false whole60 assertions stay unchanged.
```

The corrected readable logic remains exactly322 lines. Both old and corrected whole logic have the same line count;
changed line numbers are13,14,15,280,281. Reversing those five literal edits
reconstructs the ENTIRE old17528-byte logic/SHA
e3d39dc4a38a48ef9a1ace4d232544bcb0c70172b9822a5531072bed9387bb9d.
There are no intervening whitespace/comment/source changes outside those five
lines.

The standalone payload retains all imports/wrapper/output/exit code lines
and exact CELL string. Its ONLY changed line is the JSON-encoded LOGIC string.
Replace that corrected encoded string with the reconstructed original LOGIC
string to recover the ENTIRE old37730-byte payload/SHA
21ff02d7affa22017651be01fbc4072acc70265c6bf8191e55e2d40a1dab7719.
This is full-byte surgical reversal, not a matching-prefix or function-only
claim. No candidate is evaluated while proving that identity.

### Unchanged counts, bounds and held execution

All18 cases and every planned count remain unchanged:
286 stub tool calls;78 Driver,33 controller,175 collector;
127 clock,16 stop,16 readback,16 persist;16 cleanup sequences and18 outputs.
Store/date/assertion bookkeeping remains outside tool-call counts.
These are prospective counts, not results. Zero memory cases have executed.
No green run, full-gate result or exact elapsed time is inferred from parsing.

The first-failure/failed-case-count retention and fail-fast stopping rule stay
unchanged. Any future unexpected assertion still records attempted versus
completed, actual counts, first assertion and unreached cases, then stops
without correction/retry. The three corrected cases now permit resource
release only because every independent proof is present, while still requiring
their separate Driver error/over-allocation record. The always-false whole60
flag cannot turn true through this correction.

Existing finite bounds remain:18 invocations,32 stub tools/512 stores/128
Date.now reads per case, at most2 controller polls, bounded65536-character
synthetic command parsing, immediately settled stub promises, fixed virtual
time and the same synchronous VM evaluation timeout1000ms. No real timer,
MCP timeout, cancellation or asynchronous time guarantee is added. Actual
Driver call duration is not measured by the memory model.

The prospective standalone payload remains intended for ONE separately
released bounded memory invocation, proposed outer<=30s as already recorded.
Root must review the immutable corrected archive and exact host invocation
before release. No VM/mock/harness/module body, Python collector, helper,
native/provider, application/process fixture, network/HTTP, Driver/browser,
Cargo or filesystem cleanup is executed here. Only static parsing/identity
checks run. No runtime slot is acquired or released.

The real browser remains HELD while the distinct observer-aware memory design
and cleanup composition validation complete under root ownership. No mutable
observer/helper partial work is inspected or contacted. Original D01/D05
disposition and all closed/accepted rows stay root-owned and unchanged.

### Corrected pins and unchanged embedded source

| Source | Bytes | Lines | SHA256 |
| --- | --- | --- | --- |
| cell | 18091 | 284 | 0a7cea070bdab32b419a82eb26a7a991b71a2b14a55553f890b53788cda04a5e |
| logic | 17536 | 322 | b68ee1a0f88203ac57e9a41b957ef16ad2f15f11c0979c0d436c5e14779e9b50 |
| payload | 37738 | 16 | a06ab7b0783569f8f3778fffedda55dae0b0a824a8174533e77bb273ba702230 |

The exact284-line cell18091 bytes/SHA0a7cea070bdab32b419a82eb26a7a991b71a2b14a55553f890b53788cda04a5e
is embedded unchanged. All four entire encoded Python constants are unchanged,
not recompiled/executed as collectors or altered by this correction:
CLOCK cfb3f3154cb778388724728b6f7cb804bd7e142844ded7c044afc8dd3135ab4d;
STOP ccc2702c4e10209952d8e5489c2974a09a7f4d6c47a3705dc89d5bb028ec3ffb;
READBACK95c6fe05f0016ef302dd9c63533b95141b09ea71a661ab390c112c9e6c13ccb3;
PERSIST819a7785625ce2f78a9cb03b5f242dbfa6728f9f507b242e3c048887ad01ea30.

The193-line controller remains exact command SHA
6662bdbb168bf0ba6ec5ff24b6f3b88dc15b54b4dfc3b800fdaca1cbee403bbb,
and helper470 remains SHA
7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0.
START+840/inclusive900/preparation180/helper600/pending180/refusal4/
8.5stop8floor and every route/crypto/security behavior stay unchanged.
No source/controller/helper file is materialized or edited.

Complete corrected readable logic, NOT executed:

```js
const CELL_PIN="0a7cea070bdab32b419a82eb26a7a991b71a2b14a55553f890b53788cda04a5e";
const PYTHON_PINS={
  CLOCK:"cfb3f3154cb778388724728b6f7cb804bd7e142844ded7c044afc8dd3135ab4d",
  STOP:"ccc2702c4e10209952d8e5489c2974a09a7f4d6c47a3705dc89d5bb028ec3ffb",
  READBACK:"95c6fe05f0016ef302dd9c63533b95141b09ea71a661ab390c112c9e6c13ccb3",
  PERSIST:"819a7785625ce2f78a9cb03b5f242dbfa6728f9f507b242e3c048887ad01ea30"
};
const CASES=[
  {name:"ui_error",first:"application_page_failed",release:true,tools:18},
  {name:"helper_failure",first:"helper_failed",release:true,tools:16},
  {name:"controller_completion",first:"controller_completed_before_app_checkpoint",release:true,tools:14},
  {name:"metadata_write_failure",first:"application_page_failed",release:true,tools:18},
  {name:"driver_exception",first:"application_page_failed",release:true,tools:18},
  {name:"driver_refusal",first:"application_page_failed",release:true,tools:18},
  {name:"driver_late_return",first:"application_page_failed",release:true,tools:18},
  {name:"missing_child_proof",first:"application_page_failed",release:false,tools:18},
  {name:"missing_pid_proof",first:"application_page_failed",release:false,tools:18},
  {name:"missing_window_proof",first:"application_page_failed",release:false,tools:18},
  {name:"missing_session_proof",first:"application_page_failed",release:false,tools:18},
  {name:"missing_port_proof",first:"application_page_failed",release:false,tools:18},
  {name:"missing_lab_proof",first:"application_page_failed",release:false,tools:18},
  {name:"partial_controller_json",first:"application_page_failed",release:true,tools:18},
  {name:"join_failure",first:"application_page_failed",release:false,tools:18},
  {name:"readback_projection_failure",first:"application_page_failed",release:false,tools:18},
  {name:"missing_ownership",first:null,release:null,tools:0},
  {name:"application_confirmed",first:null,release:null,tools:4}
];
const PLAN={cases:18,tool_calls:286,driver_calls:78,controller_calls:33,
  collector_calls:175,clock:127,stop:16,readback:16,persist:16,
  cleanup_sequences:16,outputs:18};
const clone=v=>JSON.parse(JSON.stringify(v));
function ensure(ok,label){if(!ok)throw {envelope_assertion:label};}
function shellWords(command){
  ensure(typeof command==="string"&&command.length<65536,"command_bounds");
  const words=[];let word="",quoted=false,started=false;
  for(let i=0;i<command.length;i++){
    const c=command[i];
    if(quoted){if(c==="'")quoted=false;else word+=c;continue;}
    if(c==="'"){quoted=true;started=true;continue;}
    if(c==="\\"){ensure(i+1<command.length,"command_escape");word+=command[++i];started=true;continue;}
    if(/\s/.test(c)){if(started){words.push(word);word="";started=false;}continue;}
    word+=c;started=true;
  }
  ensure(!quoted,"command_quote");
  if(started)words.push(word);
  return words;
}
function sources(cell){
  const result={};
  for(const name of Object.keys(PYTHON_PINS)){
    const line=cell.split("\n").find(v=>v.startsWith("const "+name+"="));
    ensure(typeof line==="string","embedded_source_missing");
    result[name]=JSON.parse(line.slice(("const "+name+"=").length,-1));
    ensure(sha(result[name])===PYTHON_PINS[name],"embedded_source_hash");
  }
  return result;
}
function ownContext(){
  return {runtime_release:true,driver_owned:true,exact_bound:true,single_controller:true,
    fixture_ready:true,listener_pid_proof:true,fresh_paths_preflight:true,
    workspace:"/synthetic/d01",guard_pid:101,server_pid:102,helper_pid:103,browser_pid:104,
    exec_session:901,session:"synthetic-session",target_id:"synthetic-target",tab_id:"synthetic-tab",
    lab:"/synthetic/lab",outer_out:"/synthetic/outer",event_out:"/synthetic/event",
    cleanup_out:"/synthetic/cleanup",start_epoch_ms:1700000000000};
}
async function oneCase(spec,cell,py){
  const own=ownContext();
  if(spec.name==="missing_ownership")own.exact_bound=false;
  const memory=new Map(),trace=[],outputs=[],counts={
    tool_calls:0,driver_calls:0,controller_calls:0,collector_calls:0,
    clock:0,stop:0,readback:0,persist:0
  };
  let tick=1000,stores=0,dateCalls=0,polls=0,firstLatch=null,retainedFirst=null;
  let cleanupRecord=null,stopRequested=false,exitSeen=false;
  const EXIT={synthetic_exit:true};
  const push=(kind,label)=>trace.push({kind,label,tick});
  const now=()=>{ensure(++dateCalls<=128,"date_bound");return own.start_epoch_ms+tick;};
  const clock=()=>({monotonic_ns:String(1000000000000n+BigInt(tick)*1000000n),
    wall_epoch_ns:String(BigInt(own.start_epoch_ms+tick)*1000000n)});
  function call(label,domain,cleanup){
    ensure(++counts.tool_calls<=32,"tool_bound");
    counts[domain]++;
    push("tool",label);tick+=10;
    if(cleanup){
      const r=memory.get("d01_immediate_observation_record");
      ensure(firstLatch!==null&&r?.first_failure===spec.first,"latch_before_cleanup");
      ensure(outputs.length===0,"no_output_before_cleanup");
    }
  }
  const toolResult=v=>({exit_code:0,output:JSON.stringify(v)+"\n"});
  const children=()=>["helper","whoami","discovery","confidential_client_create",
    "operator_login","server","maintenance_init"].map((name,i)=>({
      name,pid:name==="helper"?103:name==="server"?102:200+i,exit:name==="helper"?1:0}));
  const tools={
    exec_command:async args=>{
      ensure(args.workdir===own.workspace&&args.yield_time_ms===10000,"collector_shape");
      const words=shellWords(args.cmd);
      ensure(words[0]==="python3"&&words[1]==="-c","collector_command");
      const name=Object.keys(py).find(n=>py[n]===words[2]);
      ensure(name!==undefined,"collector_source_identity");
      call(name.toLowerCase(),"collector_calls",true);counts[name.toLowerCase()]++;
      if(name==="CLOCK"){
        ensure(words.length===3,"clock_arguments");
        return toolResult(clock());
      }
      if(name==="STOP"){
        ensure(words.length===4,"stop_arguments");
        const c=JSON.parse(words[3]);
        ensure(c.lab===own.lab&&c.event_out===own.event_out&&c.first_failure===spec.first,
          "stop_owned_arguments");
        ensure(c.first_observation_wall_ms===retainedFirst,"stop_received_clock");
        push("marker","original_stop_requested");stopRequested=true;
        return toolResult({clock:clock(),event_state:spec.name==="metadata_write_failure"?
          "write_unconfirmed":"written",marker_state:"stop_requested"});
      }
      if(name==="READBACK"){
        ensure(words.length===4,"readback_arguments");
        const c=JSON.parse(words[3]);
        ensure(JSON.stringify(c.owned_pids)===JSON.stringify([101,102,103,104])&&
          c.lab===own.lab&&c.outer_out===own.outer_out&&c.helper_pid===103&&c.server_pid===102,
          "readback_owned_arguments");
        if(spec.name==="readback_projection_failure")return {exit_code:0,output:"{"};
        const unknown=spec.name==="missing_pid_proof";
        return toolResult({clock:clock(),owned_pids:Object.fromEntries(
          [101,102,103,104].map(p=>[String(p),unknown?null:true])),
          ports:{"9000":true,"3000":spec.name==="missing_port_proof"?null:true},
          lab_absent:spec.name==="missing_lab_proof"?null:true,
          owned_child_exits:spec.name==="missing_child_proof"?null:children()});
      }
      ensure(name==="PERSIST"&&words.length===5&&words[3]===own.cleanup_out,"persist_arguments");
      cleanupRecord=JSON.parse(words[4]);
      push("persist","exclusive_cleanup_projection");
      return {exit_code:0,output:'{"written_exclusive":true}\n'};
    },
    write_stdin:async args=>{
      ensure(args.session_id===901&&args.chars===""&&args.yield_time_ms===5000,
        "controller_owned_arguments");
      call("controller_poll","controller_calls",polls>0&&spec.name!=="application_confirmed");
      polls++;
      ensure(polls<=2,"controller_poll_bound");
      if(polls===1){
        if(spec.name==="helper_failure")return {session_id:901,
          output:'{"helper_completed":true,"exit":1}\n'};
        if(spec.name==="controller_completion")return {exit_code:1,
          output:'{"fixture_finished":true}\n'};
        if(spec.name==="partial_controller_json")return {session_id:901,
          output:'{"helper_completed":'};
        return {session_id:901,output:""};
      }
      if(spec.name==="application_confirmed")return {session_id:901,output:""};
      if(spec.name==="join_failure")throw {synthetic_failure:true};
      if(spec.name==="partial_controller_json")return {exit_code:1,
        output:'true,"exit":1}\n'};
      return {exit_code:1,output:'{"fixture_finished":true}\n'};
    },
    mcp__cua_driver__browser_navigate:async args=>{
      ensure(args.session===own.session&&args.target_id===own.target_id&&
        args.tab_id===own.tab_id&&args.url==="http://localhost:3000/","navigation_binding");
      call("browser_navigate","driver_calls",false);
      return {content:[{type:"text",text:"synthetic navigation returned"}]};
    },
    mcp__cua_driver__get_browser_state:async args=>{
      ensure(args.session===own.session&&args.target_id===own.target_id&&
        args.tab_id===own.tab_id&&args.snapshot_format==="semantic_v2"&&
        args.include_screenshot===false,"snapshot_binding");
      call("browser_snapshot","driver_calls",false);
      const good=spec.name==="application_confirmed";
      const content_refs=[{role:"heading",name:"Local demo",actions:[],frame:"main"}];
      if(!good)content_refs.push({role:"statictext",
        name:"Local demo could not complete this request.",actions:[],frame:"main"});
      return {structuredContent:{status:"ok",mode:"snapshot",
        snapshot:{complete:true,format:"semantic_v2",id:"synthetic-snapshot"},
        content_refs,refs:good?[{role:"button",name:"synthetic action",
          ref:"synthetic-snapshot:1",actions:["click"],frame:"main"}]:[]}};
    },
    mcp__cua_driver__kill_app:async args=>{
      ensure(args.pid===104&&Object.keys(args).length===1,"kill_owned_arguments");
      call("kill_app","driver_calls",true);
      if(spec.name==="driver_exception")throw {synthetic_failure:true};
      if(spec.name==="driver_refusal")return {isError:true,
        content:[{type:"text",text:"synthetic refusal"}]};
      if(spec.name==="driver_late_return")tick+=31000;
      return {content:[{type:"text",text:"synthetic kill returned"}]};
    },
    mcp__cua_driver__end_session:async args=>{
      ensure(args.session===own.session,"end_owned_arguments");
      call("end_session","driver_calls",true);
      if(spec.name==="missing_session_proof")return {content:[]};
      return {structuredContent:{active:false,session:own.session}};
    },
    mcp__cua_driver__list_windows:async args=>{
      ensure(args.pid===104,"windows_owned_arguments");
      call("list_windows","driver_calls",true);
      return spec.name==="missing_window_proof"?{content:[]}:
        {structuredContent:{current_space_id:1,windows:[]}};
    }
  };
  const sandbox={
    tools,Date:class {static now(){return now();}},
    load:key=>{ensure(key==="d01_immediate_owned_handles","load_key");return clone(own);},
    store:(key,value)=>{
      ensure(++stores<=512,"store_bound");
      ensure(["d01_immediate_observation_record","d01_immediate_snapshot_flags"].includes(key),
        "store_key");
      const copy=clone(value);memory.set(key,copy);
      if(key==="d01_immediate_observation_record"&&copy.first_failure!==null){
        if(firstLatch===null){
          firstLatch=trace.length;retainedFirst=copy.first_observation_wall_ms;
          push("latch",copy.first_failure);
        }
        ensure(copy.first_failure===spec.first,"first_failure_immutable");
      }
    },
    text:value=>{
      push("output","fixed_result");outputs.push(clone(value));
      ensure(outputs.length===1,"output_bound");
    },
    notify:()=>{throw {envelope_assertion:"notification_forbidden"};},
    yield_control:()=>{throw {envelope_assertion:"model_yield_forbidden"};},
    exit:()=>{exitSeen=true;throw EXIT;}
  };
  let cellError=null;
  try{
    const script=new vm.Script("(async()=>{\n"+cell+"\n})()",{filename:"immutable-cell"});
    const promise=script.runInNewContext(sandbox,{timeout:1000});
    await promise;
  }catch(error){if(error!==EXIT)cellError=error;}
  // Retain complete fixed outcome/counts BEFORE any verification assertion.
  const item={case:spec.name,counts:clone(counts),first_failure:outputs[0]?.first_failure??null,
    resource_release_proven:outputs[0]?.resource_release_proven??null,
    assertions_completed:false,assertion:null};
  try {
  ensure(cellError===null,"immutable_cell_exception");
  ensure(counts.tool_calls===spec.tools,"planned_tool_count");
  ensure(outputs.length===1,"exact_output_count");
  const result=outputs[0];
  if(spec.name==="missing_ownership"){
    ensure(exitSeen&&counts.tool_calls===0&&firstLatch===null&&
      result.proposal_refused==="fresh_owned_context_required","ownership_refuses_before_tools");
  }else if(spec.name==="application_confirmed"){
    ensure(result.result==="application_checkpoint_observed_only"&&result.journey_credit===false&&
      firstLatch===null&&counts.collector_calls===0&&counts.driver_calls===2,
      "confirmed_not_journey_or_cleanup");
  }else{
    const record=memory.get("d01_immediate_observation_record");
    ensure(result.result==="failed"&&result.first_failure===spec.first&&
      record.first_failure===spec.first,"failure_preserved");
    ensure(record.first_observation_wall_ms===retainedFirst,"first_clock_preserved");
    ensure(record.first_event_proven===false&&record.whole_cleanup_within60_proven===false&&
      result.whole_cleanup_within60_proven===false,"no_actual_anchor_or_timing_credit");
    ensure(stopRequested&&counts.stop===1&&counts.persist===1&&cleanupRecord!==null,
      "original_stop_and_exclusive_projection");
    const order=["stop","kill_app","end_session","list_windows","readback","persist"];
    const positions=order.map(label=>trace.findIndex(v=>v.kind==="tool"&&v.label===label));
    ensure(positions.every(p=>p>firstLatch)&&positions.every((p,i)=>i===0||p>positions[i-1]),
      "latch_then_cleanup_order");
    ensure(trace.at(-1).kind==="output","no_output_until_cleanup_returned");
    ensure(!trace.some(v=>v.kind==="tool"&&["browser_navigate","browser_snapshot"].includes(v.label)&&
      trace.indexOf(v)>firstLatch),"no_journey_after_failure");
    ensure(record.actions.length===4&&record.actions[0].label==="kill_app"&&
      record.actions[1].label==="end_session"&&record.actions[2].label==="list_windows"&&
      record.actions[3].label==="owned_join_readback","exact_action_projection");
    if(spec.name==="metadata_write_failure")
      ensure(record.cleanup_errors.includes("observation_metadata_write_unconfirmed"),
        "metadata_failure_separate");
    if(["driver_exception","driver_refusal"].includes(spec.name))
      ensure(record.cleanup_errors.includes("driver_operation_unconfirmed"),"driver_failure_recorded");
    if(spec.name==="driver_late_return")
      ensure(record.cleanup_errors.includes("driver_operation_over_budget")&&
        record.actions[0].elapsed_ms>30000,"late_return_observed_only");
    if(spec.name==="partial_controller_json")
      ensure(record.controller_observations.at(-1).helper_completed===true&&
        record.controller_observations.at(-1).helper_exit===1,"partial_json_preserves_failure");
    if(spec.name==="join_failure")
      ensure(record.cleanup_errors.includes("controller_join_unconfirmed")&&polls===2,
        "join_failure_no_retry");
    if(spec.name==="readback_projection_failure")
      ensure(record.cleanup_errors.includes("owned_readback_unavailable"),
        "projection_failure_separate");
    // Resource release uses independent absence proofs; Driver errors remain recorded.
    // The immutable cell and all error, elapsed and always false whole60 assertions stay unchanged.
    ensure(result.resource_release_proven===spec.release,"resource_release_failclosed");
  }
  item.assertions_completed=true;
  }catch(error){
    item.assertion=typeof error?.envelope_assertion==="string"?error.envelope_assertion:
      "envelope_unexpected_failure";
  }
  return item;
}
async function runEnvelope(cell,source){
  const result={schema:"riauth.d01-cleanup-memory-envelope/v1",result:"failed",
    source,planned:PLAN,attempted:0,completed:0,cases:[],first_failure:null,
    unreached:CASES.map(v=>v.name),counts:{tool_calls:0,driver_calls:0,controller_calls:0,
      collector_calls:0,clock:0,stop:0,readback:0,persist:0}};
  ensure(sha(cell)===CELL_PIN,"immutable_cell_hash");
  const py=sources(cell);
  for(const spec of CASES){
    result.attempted++;
    try{
      const item=await oneCase(spec,cell,py);
      result.cases.push(item);
      for(const key of Object.keys(result.counts))result.counts[key]+=item.counts[key];
      result.unreached=CASES.slice(result.attempted).map(v=>v.name);
      if(!item.assertions_completed){
        result.first_failure={case:spec.name,assertion:item.assertion};
        return result;
      }
      result.completed++;
    }catch(error){
      result.first_failure={case:spec.name,
        assertion:typeof error?.envelope_assertion==="string"?error.envelope_assertion:
          "envelope_unexpected_failure"};
      result.unreached=CASES.slice(result.attempted).map(v=>v.name);
      return result;
    }
  }
  ensure(result.completed===PLAN.cases,"planned_cases");
  for(const key of Object.keys(result.counts))ensure(result.counts[key]===PLAN[key],"planned_total");
  result.result="passed";
  return result;
}
```

Complete corrected exact standalone payload, NOT executed:

```js
import vm from "node:vm";
import {createHash} from "node:crypto";
const CELL="// Prospective ONE functions.exec cell; NOT executed in this design phase.\nconst CLOCK=\"import json,time\\nprint(json.dumps({'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())}))\\n\";\nconst STOP=\"import json,os,pathlib,stat,sys,time\\nc=json.loads(sys.argv[1])\\nclock={'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())}\\nrecord={'schema':'riauth.d01-first-observation/v1','first_failure':c['first_failure'],'first_observation_wall_ms':c['first_observation_wall_ms'],'first_event_proven':False,'clock':clock}\\nevent_state='write_unconfirmed'\\ntry:\\n    fd=os.open(pathlib.Path(c['event_out']),os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\\n    with os.fdopen(fd,'w',encoding='ascii') as f:\\n        f.write(json.dumps(record,sort_keys=True)+'\\\\n');f.flush();os.fsync(f.fileno())\\n    event_state='written'\\nexcept OSError:pass\\n# Attempt clock persistence BEFORE marker/state/budget comparisons.\\n# A metadata error does not prevent the original essential stop protocol.\\nlab=pathlib.Path(c['lab']);marker_state='lab_absent'\\ntry:\\n    if lab.exists():\\n        info=lab.lstat()\\n        if not stat.S_ISDIR(info.st_mode) or stat.S_IMODE(info.st_mode)!=0o700 or info.st_uid!=os.getuid():\\n            marker_state='ownership_unknown'\\n        else:\\n            marker_state='stop_requested'\\n            for name in ('ui-failure','stop'):\\n                try:\\n                    fd=os.open(lab/name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\\n                    os.close(fd)\\n                except FileNotFoundError:marker_state='lab_absent'\\n                except FileExistsError:pass\\nexcept OSError:marker_state='stop_unconfirmed'\\nprint(json.dumps({'clock':clock,'event_state':event_state,'marker_state':marker_state}))\\n\";\nconst READBACK=\"import json,pathlib,subprocess,sys,time\\nc=json.loads(sys.argv[1])\\npids=c['owned_pids']\\np=subprocess.run(['/bin/ps','-p',','.join(str(v) for v in pids),'-o','pid='],capture_output=True,timeout=3)\\nps_known=p.returncode in (0,1) and all(v.isdigit() for v in p.stdout.split())\\npresent=set(int(v) for v in p.stdout.split()) if ps_known else set()\\nports={}\\nfor port in (9000,3000):\\n    p=subprocess.run(['/usr/sbin/lsof','-nP','-t','-iTCP:'+str(port),'-sTCP:LISTEN'],capture_output=True,timeout=3)\\n    ports[str(port)]=not bool(p.stdout.strip()) if p.returncode in (0,1) else None\\nchildren=None\\nouter=pathlib.Path(c['outer_out'])\\nif outer.is_file():\\n    # This is the unchanged controller's fixed redacted evidence, not app data.\\n    data=json.loads(outer.read_bytes())\\n    allowed={'helper','whoami','discovery','confidential_client_create','operator_login','server','maintenance_init'}\\n    raw=data.get('owned_child_exits')\\n    if isinstance(raw,list) and len(raw)==7 and all(isinstance(v,dict) and v.get('name') in allowed and type(v.get('pid')) is int and type(v.get('exit')) is int for v in raw) and {v['name'] for v in raw}==allowed and len({v['pid'] for v in raw})==7 and next(v['pid'] for v in raw if v['name']=='helper')==c['helper_pid'] and next(v['pid'] for v in raw if v['name']=='server')==c['server_pid']:\\n        children=[{k:v[k] for k in ('name','pid','exit')} for v in raw]\\nprint(json.dumps({'clock':{'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())},'owned_pids':{str(v):(v not in present if ps_known else None) for v in pids},'ports':ports,'lab_absent':not pathlib.Path(c['lab']).exists(),'owned_child_exits':children}))\\n\";\nconst PERSIST=\"import json,os,pathlib,sys\\npath=pathlib.Path(sys.argv[1])\\nrecord=json.loads(sys.argv[2])\\n# Convert decimal strings directly to Python integers, avoiding JS Number rounding.\\ndef clocks(value):\\n    if isinstance(value,dict):\\n        for k,v in list(value.items()):\\n            if k in ('monotonic_ns','wall_epoch_ns') and isinstance(v,str):\\n                assert v.isdecimal() and len(v)<=19 and 0<=int(v)<2**63\\n                value[k]=int(v)\\n            else:clocks(v)\\n    elif isinstance(value,list):\\n        for v in value:clocks(v)\\nclocks(record)\\nfd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\\nwith os.fdopen(fd,'w',encoding='ascii') as f:\\n    f.write(json.dumps(record,sort_keys=True,indent=2)+'\\\\n');f.flush();os.fsync(f.fileno())\\nprint(json.dumps({'written_exclusive':True}))\\n\";\nconst own=load(\"d01_immediate_owned_handles\");\nconst OBSKEY=\"d01_immediate_observation_record\";\nif (!own || own.runtime_release!==true || own.driver_owned!==true ||\n    own.exact_bound!==true || own.single_controller!==true ||\n    own.fixture_ready!==true || own.listener_pid_proof!==true ||\n    own.fresh_paths_preflight!==true ||\n    !Number.isSafeInteger(own.start_epoch_ms) || typeof own.workspace!==\"string\" ||\n    new Set([own.guard_pid,own.server_pid,own.helper_pid,own.browser_pid]).size!==4 ||\n    ![own.guard_pid,own.server_pid,own.helper_pid,own.browser_pid,own.exec_session]\n       .every(v=>Number.isSafeInteger(v)&&v>0) ||\n    ![own.session,own.target_id,own.tab_id,own.lab,own.outer_out,\n       own.event_out,own.cleanup_out].every(v=>typeof v===\"string\"&&v.length>0)) {\n  text({proposal_refused:\"fresh_owned_context_required\"});exit();\n}\nconst record={\n  schema:\"riauth.d01-immediate-observation-cleanup/v1\",\n  first_failure:null,first_observation_wall_ms:null,\n  first_event_proven:false,whole_cleanup_within60_proven:false,\n  first_clock:null,observation_receipts:[],controller_observations:[],local_tool_receipts:[],actions:[],cleanup_errors:[],\n  final_absence:null,owned_child_exits:null,controller_exit:null,\n  first_observation_to_final_wall_ms:null,latch_to_absence_ms:null\n};\nconst sq=s=>\"'\"+String(s).replace(/'/g,\"'\\\\''\")+\"'\";\nconst retain=()=>store(OBSKEY,record);\nconst cleanupError=label=>{\n  if(!record.cleanup_errors.includes(label))record.cleanup_errors.push(label);\n  retain();\n};\nconst local=async(source,arg)=>{\n  const cmd=\"python3 -c \"+sq(source)+(arg===undefined?\"\":\" \"+sq(JSON.stringify(arg)));\n  const r=await tools.exec_command({cmd,workdir:own.workspace,\n    yield_time_ms:10000,max_output_tokens:2000});\n  record.local_tool_receipts.push({\n    label:source===CLOCK?\"clock\":source===STOP?\"stop\":source===READBACK?\"readback\":\"unknown\",\n    exit:typeof r.exit_code===\"number\"?r.exit_code:null,\n    session_id:typeof r.session_id===\"number\"?r.session_id:null\n  });retain(); // Numeric collector receipt BEFORE comparisons.\n  if(r.session_id!==undefined) {\n    cleanupError(\"owned_local_command_unjoined\");throw new Error(\"local_pending\");\n  }\n  if(r.exit_code!==0)throw new Error(\"local_failed\");\n  return JSON.parse(r.output);\n};\nconst ns=c=>BigInt(c.monotonic_ns);\nconst getClock=()=>local(CLOCK);\nlet controllerJoined=false;\nlet controllerPollAvailable=true;\nlet controllerBuffer=\"\";\nlet cleanupStarted=false;\nlet lastSnapshotFlags=null;\nfunction latch(label,receivedWall) {\n  if(record.first_failure===null) {\n    record.first_failure=label;\n    record.first_observation_wall_ms=receivedWall;\n    retain(); // Synchronous first-failure/wall latch BEFORE any new await/output.\n  }\n}\nfunction observeController(r,receivedWall) {\n  const observation={received_wall_ms:receivedWall,\n    exit:typeof r.exit_code===\"number\"?r.exit_code:null,\n    helper_completed:false,helper_exit:null,fixture_finished:false};\n  // Complete numeric result projection is retained BEFORE comparisons.\n  record.controller_observations.push(observation);retain();\n  if(typeof r.exit_code===\"number\") {\n    controllerJoined=true;record.controller_exit=r.exit_code;retain();\n  }\n  controllerBuffer+=typeof r.output===\"string\"?r.output:\"\";\n  if(controllerBuffer.length>16384) {\n    latch(\"controller_observation_invalid\",receivedWall);controllerBuffer=\"\";return;\n  }\n  let cut;\n  while((cut=controllerBuffer.indexOf(\"\\n\"))>=0) {\n    const line=controllerBuffer.slice(0,cut);controllerBuffer=controllerBuffer.slice(cut+1);\n    if(!line.trim())continue;\n    let event;\n    try{event=JSON.parse(line);}catch{\n      latch(\"controller_observation_invalid\",receivedWall);continue;\n    }\n    if(event.helper_completed===true) {\n      observation.helper_completed=true;\n      observation.helper_exit=typeof event.exit===\"number\"?event.exit:null;\n      retain();\n      if(event.exit!==0)latch(\"helper_failed\",receivedWall);\n      else if(!cleanupStarted)latch(\"helper_completed_before_app_checkpoint\",receivedWall);\n    }\n    if(event.fixture_finished===true) {\n      observation.fixture_finished=true;retain();\n      if(!cleanupStarted)latch(\"controller_completed_before_app_checkpoint\",receivedWall);\n    }\n  }\n  if(controllerJoined&&!cleanupStarted)\n    latch(\"controller_completed_before_app_checkpoint\",receivedWall);\n}\nasync function pollController() {\n  if(controllerJoined||!controllerPollAvailable)return;\n  try {\n    const r=await tools.write_stdin({session_id:own.exec_session,\n      chars:\"\",yield_time_ms:5000,max_output_tokens:2000});\n    const receivedWall=Date.now();\n    observeController(r,receivedWall);\n  } catch {\n    controllerPollAvailable=false;\n    latch(\"controller_observation_unavailable\",Date.now());\n    if(cleanupStarted)cleanupError(\"controller_join_unconfirmed\");\n  }\n}\nasync function timedDriver(label,allocation,operation,project) {\n  let start=null,end=null,result=null,state=\"unknown\";\n  try{start=await getClock();}catch{cleanupError(\"clock_unavailable\");}\n  const action={label,start_clock:start,end_clock:null,allowed_ms:allocation,\n    result_state:\"unknown\",elapsed_ms:null,over_budget:null};\n  record.actions.push(action);retain(); // Start retained BEFORE entering Driver call.\n  try {\n    result=await operation();\n    state=result?.isError===true?\"refused\":\"returned\";\n  } catch {state=\"exception\";}\n  try{end=await getClock();}catch{cleanupError(\"clock_unavailable\");}\n  action.end_clock=end;action.result_state=state;\n  retain(); // End/result retained BEFORE budget comparison.\n  if(start&&end) {\n    const d=ns(end)-ns(start);\n    if(d>=0n) {\n      action.elapsed_ms=Number(d/1000000n);\n      action.over_budget=action.elapsed_ms>allocation;\n    } else cleanupError(\"clock_invalid\");\n  }\n  if(action.over_budget)cleanupError(\"driver_operation_over_budget\");\n  if(state!==\"returned\")cleanupError(\"driver_operation_unconfirmed\");\n  if(project)project(result,state);\n  retain();\n}\nasync function cleanup() {\n  if(cleanupStarted)return;\n  cleanupStarted=true;\n  try {\n    const r=await local(STOP,{\n      lab:own.lab,event_out:own.event_out,\n      first_failure:record.first_failure,\n      first_observation_wall_ms:record.first_observation_wall_ms\n    });\n    record.first_clock=r.clock;record.stop_state=r.marker_state;retain();\n    if(r.event_state!==\"written\")cleanupError(\"observation_metadata_write_unconfirmed\");\n    if(r.marker_state===\"stop_unconfirmed\")cleanupError(\"stop_unconfirmed\");\n    if(r.marker_state===\"ownership_unknown\")cleanupError(\"ownership_unknown\");\n  } catch {cleanupError(\"stop_or_clock_record_unavailable\");}\n  // No outstanding Driver call exists here: all page calls above were awaited.\n  // Original stop protocol already causes the controller's owned-child finally.\n  await timedDriver(\"kill_app\",30000,\n    ()=>tools.mcp__cua_driver__kill_app({pid:own.browser_pid}));\n  await timedDriver(\"end_session\",15000,\n    ()=>tools.mcp__cua_driver__end_session({session:own.session}),(r,state)=>{\n      record.session_ended=state===\"returned\"&&\n        r?.structuredContent?.active===false&&r.structuredContent.session===own.session;\n      retain();\n    });\n  await timedDriver(\"list_windows\",5000,\n    ()=>tools.mcp__cua_driver__list_windows({pid:own.browser_pid}),(r,state)=>{\n      const windows=r?.structuredContent?.windows;\n      record.window_count=state===\"returned\"&&Array.isArray(windows)?windows.length:null;\n      retain();\n    });\n  let readStart=null;\n  try{readStart=await getClock();}catch{cleanupError(\"clock_unavailable\");}\n  const action={label:\"owned_join_readback\",start_clock:readStart,end_clock:null,\n    allowed_ms:null,elapsed_ms:null,over_budget:null,result_state:\"unknown\"};\n  record.actions.push(action);retain();\n  if(readStart&&record.first_clock) {\n    action.allowed_ms=Math.max(0,60000-Number((ns(readStart)-ns(record.first_clock))/1000000n));\n    retain();\n  }\n  // Continue essential join after60s if late; never claim that deadline enforced.\n  // Do not poll another exec session or send a manual process signal.\n  while(!controllerJoined&&controllerPollAvailable&&Date.now()<own.start_epoch_ms+900000) {\n    await pollController();\n    if(record.first_clock) {\n      try {\n        const c=await getClock();record.last_join_clock=c;retain();\n        if(ns(c)-ns(record.first_clock)>60000000000n)\n          cleanupError(\"cleanup_budget_exceeded\");\n      } catch{cleanupError(\"clock_unavailable\");}\n    }\n  }\n  if(!controllerJoined)cleanupError(\"child_reap_incomplete\");\n  try {\n    const r=await local(READBACK,{\n      owned_pids:[own.guard_pid,own.server_pid,own.helper_pid,own.browser_pid],\n      lab:own.lab,outer_out:own.outer_out,helper_pid:own.helper_pid,server_pid:own.server_pid\n    });\n    action.end_clock=r.clock;action.result_state=\"returned\";\n    record.owned_child_exits=r.owned_child_exits;\n    record.final_absence={owned_pids:r.owned_pids,ports:r.ports,\n      lab:r.lab_absent,window_count:record.window_count??null,\n      session_ended:record.session_ended===true};\n    record.first_observation_to_final_wall_ms=Date.now()-record.first_observation_wall_ms;\n    retain(); // Full fixed readback/exits/clock BEFORE comparisons.\n    if(readStart) {\n      action.elapsed_ms=Number((ns(r.clock)-ns(readStart))/1000000n);\n      action.over_budget=action.allowed_ms===null?null:action.elapsed_ms>action.allowed_ms;\n    }\n    if(record.first_clock)\n      record.latch_to_absence_ms=Number((ns(r.clock)-ns(record.first_clock))/1000000n);\n    if(action.over_budget)cleanupError(\"cleanup_budget_exceeded\");\n    const a=record.final_absence;\n    if(!controllerJoined||!Array.isArray(record.owned_child_exits)||\n       record.owned_child_exits.length!==7)cleanupError(\"child_reap_incomplete\");\n    if(!Object.values(a.owned_pids).every(v=>v===true)||\n       !Object.values(a.ports).every(v=>v===true)||a.lab!==true||\n       a.window_count!==0||a.session_ended!==true)\n      cleanupError(\"owned_resource_absence_unproven\");\n  } catch {action.result_state=\"exception\";cleanupError(\"owned_readback_unavailable\");}\n  cleanupError(\"first_event_unproven\");\n  record.whole_cleanup_within60_proven=false;retain();\n  // Exclusive new file only; existing outer/helper/provider evidence untouched.\n  try {\n    const cmd=\"python3 -c \"+sq(PERSIST)+\" \"+sq(own.cleanup_out)+\" \"+sq(JSON.stringify(record));\n    const r=await tools.exec_command({cmd,workdir:own.workspace,\n      yield_time_ms:10000,max_output_tokens:500});\n    record.local_tool_receipts.push({label:\"persist\",\n      exit:typeof r.exit_code===\"number\"?r.exit_code:null,\n      session_id:typeof r.session_id===\"number\"?r.session_id:null});retain();\n    if(r.session_id!==undefined)cleanupError(\"owned_local_command_unjoined\");\n    if(r.session_id!==undefined||r.exit_code!==0)\n      cleanupError(\"cleanup_metadata_write_unconfirmed\");\n  } catch {cleanupError(\"cleanup_metadata_write_unconfirmed\");}\n  controllerBuffer=\"\";\n}\nawait pollController();\nif(record.first_failure===null) {\n  try {\n    const r=await tools.mcp__cua_driver__browser_navigate({\n      session:own.session,target_id:own.target_id,tab_id:own.tab_id,\n      url:\"http://localhost:3000/\"});\n    const receivedWall=Date.now();\n    record.observation_receipts.push({kind:\"browser_navigation\",received_wall_ms:receivedWall});\n    retain(); // Received clock BEFORE navigation outcome comparison.\n    if(r?.isError===true)latch(\"driver_navigation_refused\",receivedWall);\n  } catch {latch(\"driver_navigation_exception\",Date.now());}\n}\nif(record.first_failure===null) {\n  try {\n    const r=await tools.mcp__cua_driver__get_browser_state({\n      session:own.session,target_id:own.target_id,tab_id:own.tab_id,\n      snapshot_format:\"semantic_v2\",include_screenshot:false});\n    const receivedWall=Date.now();\n    record.observation_receipts.push({kind:\"browser_snapshot\",received_wall_ms:receivedWall});\n    retain(); // Received clock BEFORE any page predicate comparison.\n    const s=r?.structuredContent;\n    const nodes=Array.isArray(s?.content_refs)?s.content_refs:[];\n    lastSnapshotFlags={\n      received_wall_ms:receivedWall,\n      status_ok:r?.isError!==true&&s?.status===\"ok\",\n      complete:s?.snapshot?.complete===true,\n      heading_match:nodes.some(n=>n.role===\"heading\"&&n.name===\"Local demo\"),\n      error_match:nodes.some(n=>n.name===\"Local demo could not complete this request.\"),\n      interactive_ref_present:Array.isArray(s?.refs)&&\n        s.refs.some(n=>Array.isArray(n.actions)&&n.actions.includes(\"click\"))\n    };\n    store(\"d01_immediate_snapshot_flags\",lastSnapshotFlags);\n    // Finite booleans only; no outline/URL/name/value/subject/token is persisted.\n    if(lastSnapshotFlags.error_match)latch(\"application_page_failed\",receivedWall);\n    else if(!lastSnapshotFlags.status_ok||!lastSnapshotFlags.complete||\n            !lastSnapshotFlags.heading_match||!lastSnapshotFlags.interactive_ref_present)\n      latch(\"application_snapshot_unconfirmed\",receivedWall);\n  } catch {latch(\"driver_snapshot_exception\",Date.now());}\n}\nif(record.first_failure===null)await pollController();\nif(record.first_failure!==null) {\n  await cleanup(); // NO text/notify/yield_control/model turn before essential cleanup.\n  text({result:\"failed\",first_failure:record.first_failure,\n    cleanup_errors:record.cleanup_errors,final_absence:record.final_absence,\n    controller_exit:record.controller_exit,whole_cleanup_within60_proven:false,\n    resource_release_proven:record.final_absence!==null&&\n      !record.cleanup_errors.some(v=>[\n        \"child_reap_incomplete\",\"owned_resource_absence_unproven\",\n        \"owned_readback_unavailable\",\"owned_local_command_unjoined\"].includes(v))});\n} else {\n  text({result:\"application_checkpoint_observed_only\",\n    snapshot_flags:lastSnapshotFlags,journey_credit:false});\n}\n";
const LOGIC="const CELL_PIN=\"0a7cea070bdab32b419a82eb26a7a991b71a2b14a55553f890b53788cda04a5e\";\nconst PYTHON_PINS={\n  CLOCK:\"cfb3f3154cb778388724728b6f7cb804bd7e142844ded7c044afc8dd3135ab4d\",\n  STOP:\"ccc2702c4e10209952d8e5489c2974a09a7f4d6c47a3705dc89d5bb028ec3ffb\",\n  READBACK:\"95c6fe05f0016ef302dd9c63533b95141b09ea71a661ab390c112c9e6c13ccb3\",\n  PERSIST:\"819a7785625ce2f78a9cb03b5f242dbfa6728f9f507b242e3c048887ad01ea30\"\n};\nconst CASES=[\n  {name:\"ui_error\",first:\"application_page_failed\",release:true,tools:18},\n  {name:\"helper_failure\",first:\"helper_failed\",release:true,tools:16},\n  {name:\"controller_completion\",first:\"controller_completed_before_app_checkpoint\",release:true,tools:14},\n  {name:\"metadata_write_failure\",first:\"application_page_failed\",release:true,tools:18},\n  {name:\"driver_exception\",first:\"application_page_failed\",release:true,tools:18},\n  {name:\"driver_refusal\",first:\"application_page_failed\",release:true,tools:18},\n  {name:\"driver_late_return\",first:\"application_page_failed\",release:true,tools:18},\n  {name:\"missing_child_proof\",first:\"application_page_failed\",release:false,tools:18},\n  {name:\"missing_pid_proof\",first:\"application_page_failed\",release:false,tools:18},\n  {name:\"missing_window_proof\",first:\"application_page_failed\",release:false,tools:18},\n  {name:\"missing_session_proof\",first:\"application_page_failed\",release:false,tools:18},\n  {name:\"missing_port_proof\",first:\"application_page_failed\",release:false,tools:18},\n  {name:\"missing_lab_proof\",first:\"application_page_failed\",release:false,tools:18},\n  {name:\"partial_controller_json\",first:\"application_page_failed\",release:true,tools:18},\n  {name:\"join_failure\",first:\"application_page_failed\",release:false,tools:18},\n  {name:\"readback_projection_failure\",first:\"application_page_failed\",release:false,tools:18},\n  {name:\"missing_ownership\",first:null,release:null,tools:0},\n  {name:\"application_confirmed\",first:null,release:null,tools:4}\n];\nconst PLAN={cases:18,tool_calls:286,driver_calls:78,controller_calls:33,\n  collector_calls:175,clock:127,stop:16,readback:16,persist:16,\n  cleanup_sequences:16,outputs:18};\nconst clone=v=>JSON.parse(JSON.stringify(v));\nfunction ensure(ok,label){if(!ok)throw {envelope_assertion:label};}\nfunction shellWords(command){\n  ensure(typeof command===\"string\"&&command.length<65536,\"command_bounds\");\n  const words=[];let word=\"\",quoted=false,started=false;\n  for(let i=0;i<command.length;i++){\n    const c=command[i];\n    if(quoted){if(c===\"'\")quoted=false;else word+=c;continue;}\n    if(c===\"'\"){quoted=true;started=true;continue;}\n    if(c===\"\\\\\"){ensure(i+1<command.length,\"command_escape\");word+=command[++i];started=true;continue;}\n    if(/\\s/.test(c)){if(started){words.push(word);word=\"\";started=false;}continue;}\n    word+=c;started=true;\n  }\n  ensure(!quoted,\"command_quote\");\n  if(started)words.push(word);\n  return words;\n}\nfunction sources(cell){\n  const result={};\n  for(const name of Object.keys(PYTHON_PINS)){\n    const line=cell.split(\"\\n\").find(v=>v.startsWith(\"const \"+name+\"=\"));\n    ensure(typeof line===\"string\",\"embedded_source_missing\");\n    result[name]=JSON.parse(line.slice((\"const \"+name+\"=\").length,-1));\n    ensure(sha(result[name])===PYTHON_PINS[name],\"embedded_source_hash\");\n  }\n  return result;\n}\nfunction ownContext(){\n  return {runtime_release:true,driver_owned:true,exact_bound:true,single_controller:true,\n    fixture_ready:true,listener_pid_proof:true,fresh_paths_preflight:true,\n    workspace:\"/synthetic/d01\",guard_pid:101,server_pid:102,helper_pid:103,browser_pid:104,\n    exec_session:901,session:\"synthetic-session\",target_id:\"synthetic-target\",tab_id:\"synthetic-tab\",\n    lab:\"/synthetic/lab\",outer_out:\"/synthetic/outer\",event_out:\"/synthetic/event\",\n    cleanup_out:\"/synthetic/cleanup\",start_epoch_ms:1700000000000};\n}\nasync function oneCase(spec,cell,py){\n  const own=ownContext();\n  if(spec.name===\"missing_ownership\")own.exact_bound=false;\n  const memory=new Map(),trace=[],outputs=[],counts={\n    tool_calls:0,driver_calls:0,controller_calls:0,collector_calls:0,\n    clock:0,stop:0,readback:0,persist:0\n  };\n  let tick=1000,stores=0,dateCalls=0,polls=0,firstLatch=null,retainedFirst=null;\n  let cleanupRecord=null,stopRequested=false,exitSeen=false;\n  const EXIT={synthetic_exit:true};\n  const push=(kind,label)=>trace.push({kind,label,tick});\n  const now=()=>{ensure(++dateCalls<=128,\"date_bound\");return own.start_epoch_ms+tick;};\n  const clock=()=>({monotonic_ns:String(1000000000000n+BigInt(tick)*1000000n),\n    wall_epoch_ns:String(BigInt(own.start_epoch_ms+tick)*1000000n)});\n  function call(label,domain,cleanup){\n    ensure(++counts.tool_calls<=32,\"tool_bound\");\n    counts[domain]++;\n    push(\"tool\",label);tick+=10;\n    if(cleanup){\n      const r=memory.get(\"d01_immediate_observation_record\");\n      ensure(firstLatch!==null&&r?.first_failure===spec.first,\"latch_before_cleanup\");\n      ensure(outputs.length===0,\"no_output_before_cleanup\");\n    }\n  }\n  const toolResult=v=>({exit_code:0,output:JSON.stringify(v)+\"\\n\"});\n  const children=()=>[\"helper\",\"whoami\",\"discovery\",\"confidential_client_create\",\n    \"operator_login\",\"server\",\"maintenance_init\"].map((name,i)=>({\n      name,pid:name===\"helper\"?103:name===\"server\"?102:200+i,exit:name===\"helper\"?1:0}));\n  const tools={\n    exec_command:async args=>{\n      ensure(args.workdir===own.workspace&&args.yield_time_ms===10000,\"collector_shape\");\n      const words=shellWords(args.cmd);\n      ensure(words[0]===\"python3\"&&words[1]===\"-c\",\"collector_command\");\n      const name=Object.keys(py).find(n=>py[n]===words[2]);\n      ensure(name!==undefined,\"collector_source_identity\");\n      call(name.toLowerCase(),\"collector_calls\",true);counts[name.toLowerCase()]++;\n      if(name===\"CLOCK\"){\n        ensure(words.length===3,\"clock_arguments\");\n        return toolResult(clock());\n      }\n      if(name===\"STOP\"){\n        ensure(words.length===4,\"stop_arguments\");\n        const c=JSON.parse(words[3]);\n        ensure(c.lab===own.lab&&c.event_out===own.event_out&&c.first_failure===spec.first,\n          \"stop_owned_arguments\");\n        ensure(c.first_observation_wall_ms===retainedFirst,\"stop_received_clock\");\n        push(\"marker\",\"original_stop_requested\");stopRequested=true;\n        return toolResult({clock:clock(),event_state:spec.name===\"metadata_write_failure\"?\n          \"write_unconfirmed\":\"written\",marker_state:\"stop_requested\"});\n      }\n      if(name===\"READBACK\"){\n        ensure(words.length===4,\"readback_arguments\");\n        const c=JSON.parse(words[3]);\n        ensure(JSON.stringify(c.owned_pids)===JSON.stringify([101,102,103,104])&&\n          c.lab===own.lab&&c.outer_out===own.outer_out&&c.helper_pid===103&&c.server_pid===102,\n          \"readback_owned_arguments\");\n        if(spec.name===\"readback_projection_failure\")return {exit_code:0,output:\"{\"};\n        const unknown=spec.name===\"missing_pid_proof\";\n        return toolResult({clock:clock(),owned_pids:Object.fromEntries(\n          [101,102,103,104].map(p=>[String(p),unknown?null:true])),\n          ports:{\"9000\":true,\"3000\":spec.name===\"missing_port_proof\"?null:true},\n          lab_absent:spec.name===\"missing_lab_proof\"?null:true,\n          owned_child_exits:spec.name===\"missing_child_proof\"?null:children()});\n      }\n      ensure(name===\"PERSIST\"&&words.length===5&&words[3]===own.cleanup_out,\"persist_arguments\");\n      cleanupRecord=JSON.parse(words[4]);\n      push(\"persist\",\"exclusive_cleanup_projection\");\n      return {exit_code:0,output:'{\"written_exclusive\":true}\\n'};\n    },\n    write_stdin:async args=>{\n      ensure(args.session_id===901&&args.chars===\"\"&&args.yield_time_ms===5000,\n        \"controller_owned_arguments\");\n      call(\"controller_poll\",\"controller_calls\",polls>0&&spec.name!==\"application_confirmed\");\n      polls++;\n      ensure(polls<=2,\"controller_poll_bound\");\n      if(polls===1){\n        if(spec.name===\"helper_failure\")return {session_id:901,\n          output:'{\"helper_completed\":true,\"exit\":1}\\n'};\n        if(spec.name===\"controller_completion\")return {exit_code:1,\n          output:'{\"fixture_finished\":true}\\n'};\n        if(spec.name===\"partial_controller_json\")return {session_id:901,\n          output:'{\"helper_completed\":'};\n        return {session_id:901,output:\"\"};\n      }\n      if(spec.name===\"application_confirmed\")return {session_id:901,output:\"\"};\n      if(spec.name===\"join_failure\")throw {synthetic_failure:true};\n      if(spec.name===\"partial_controller_json\")return {exit_code:1,\n        output:'true,\"exit\":1}\\n'};\n      return {exit_code:1,output:'{\"fixture_finished\":true}\\n'};\n    },\n    mcp__cua_driver__browser_navigate:async args=>{\n      ensure(args.session===own.session&&args.target_id===own.target_id&&\n        args.tab_id===own.tab_id&&args.url===\"http://localhost:3000/\",\"navigation_binding\");\n      call(\"browser_navigate\",\"driver_calls\",false);\n      return {content:[{type:\"text\",text:\"synthetic navigation returned\"}]};\n    },\n    mcp__cua_driver__get_browser_state:async args=>{\n      ensure(args.session===own.session&&args.target_id===own.target_id&&\n        args.tab_id===own.tab_id&&args.snapshot_format===\"semantic_v2\"&&\n        args.include_screenshot===false,\"snapshot_binding\");\n      call(\"browser_snapshot\",\"driver_calls\",false);\n      const good=spec.name===\"application_confirmed\";\n      const content_refs=[{role:\"heading\",name:\"Local demo\",actions:[],frame:\"main\"}];\n      if(!good)content_refs.push({role:\"statictext\",\n        name:\"Local demo could not complete this request.\",actions:[],frame:\"main\"});\n      return {structuredContent:{status:\"ok\",mode:\"snapshot\",\n        snapshot:{complete:true,format:\"semantic_v2\",id:\"synthetic-snapshot\"},\n        content_refs,refs:good?[{role:\"button\",name:\"synthetic action\",\n          ref:\"synthetic-snapshot:1\",actions:[\"click\"],frame:\"main\"}]:[]}};\n    },\n    mcp__cua_driver__kill_app:async args=>{\n      ensure(args.pid===104&&Object.keys(args).length===1,\"kill_owned_arguments\");\n      call(\"kill_app\",\"driver_calls\",true);\n      if(spec.name===\"driver_exception\")throw {synthetic_failure:true};\n      if(spec.name===\"driver_refusal\")return {isError:true,\n        content:[{type:\"text\",text:\"synthetic refusal\"}]};\n      if(spec.name===\"driver_late_return\")tick+=31000;\n      return {content:[{type:\"text\",text:\"synthetic kill returned\"}]};\n    },\n    mcp__cua_driver__end_session:async args=>{\n      ensure(args.session===own.session,\"end_owned_arguments\");\n      call(\"end_session\",\"driver_calls\",true);\n      if(spec.name===\"missing_session_proof\")return {content:[]};\n      return {structuredContent:{active:false,session:own.session}};\n    },\n    mcp__cua_driver__list_windows:async args=>{\n      ensure(args.pid===104,\"windows_owned_arguments\");\n      call(\"list_windows\",\"driver_calls\",true);\n      return spec.name===\"missing_window_proof\"?{content:[]}:\n        {structuredContent:{current_space_id:1,windows:[]}};\n    }\n  };\n  const sandbox={\n    tools,Date:class {static now(){return now();}},\n    load:key=>{ensure(key===\"d01_immediate_owned_handles\",\"load_key\");return clone(own);},\n    store:(key,value)=>{\n      ensure(++stores<=512,\"store_bound\");\n      ensure([\"d01_immediate_observation_record\",\"d01_immediate_snapshot_flags\"].includes(key),\n        \"store_key\");\n      const copy=clone(value);memory.set(key,copy);\n      if(key===\"d01_immediate_observation_record\"&&copy.first_failure!==null){\n        if(firstLatch===null){\n          firstLatch=trace.length;retainedFirst=copy.first_observation_wall_ms;\n          push(\"latch\",copy.first_failure);\n        }\n        ensure(copy.first_failure===spec.first,\"first_failure_immutable\");\n      }\n    },\n    text:value=>{\n      push(\"output\",\"fixed_result\");outputs.push(clone(value));\n      ensure(outputs.length===1,\"output_bound\");\n    },\n    notify:()=>{throw {envelope_assertion:\"notification_forbidden\"};},\n    yield_control:()=>{throw {envelope_assertion:\"model_yield_forbidden\"};},\n    exit:()=>{exitSeen=true;throw EXIT;}\n  };\n  let cellError=null;\n  try{\n    const script=new vm.Script(\"(async()=>{\\n\"+cell+\"\\n})()\",{filename:\"immutable-cell\"});\n    const promise=script.runInNewContext(sandbox,{timeout:1000});\n    await promise;\n  }catch(error){if(error!==EXIT)cellError=error;}\n  // Retain complete fixed outcome/counts BEFORE any verification assertion.\n  const item={case:spec.name,counts:clone(counts),first_failure:outputs[0]?.first_failure??null,\n    resource_release_proven:outputs[0]?.resource_release_proven??null,\n    assertions_completed:false,assertion:null};\n  try {\n  ensure(cellError===null,\"immutable_cell_exception\");\n  ensure(counts.tool_calls===spec.tools,\"planned_tool_count\");\n  ensure(outputs.length===1,\"exact_output_count\");\n  const result=outputs[0];\n  if(spec.name===\"missing_ownership\"){\n    ensure(exitSeen&&counts.tool_calls===0&&firstLatch===null&&\n      result.proposal_refused===\"fresh_owned_context_required\",\"ownership_refuses_before_tools\");\n  }else if(spec.name===\"application_confirmed\"){\n    ensure(result.result===\"application_checkpoint_observed_only\"&&result.journey_credit===false&&\n      firstLatch===null&&counts.collector_calls===0&&counts.driver_calls===2,\n      \"confirmed_not_journey_or_cleanup\");\n  }else{\n    const record=memory.get(\"d01_immediate_observation_record\");\n    ensure(result.result===\"failed\"&&result.first_failure===spec.first&&\n      record.first_failure===spec.first,\"failure_preserved\");\n    ensure(record.first_observation_wall_ms===retainedFirst,\"first_clock_preserved\");\n    ensure(record.first_event_proven===false&&record.whole_cleanup_within60_proven===false&&\n      result.whole_cleanup_within60_proven===false,\"no_actual_anchor_or_timing_credit\");\n    ensure(stopRequested&&counts.stop===1&&counts.persist===1&&cleanupRecord!==null,\n      \"original_stop_and_exclusive_projection\");\n    const order=[\"stop\",\"kill_app\",\"end_session\",\"list_windows\",\"readback\",\"persist\"];\n    const positions=order.map(label=>trace.findIndex(v=>v.kind===\"tool\"&&v.label===label));\n    ensure(positions.every(p=>p>firstLatch)&&positions.every((p,i)=>i===0||p>positions[i-1]),\n      \"latch_then_cleanup_order\");\n    ensure(trace.at(-1).kind===\"output\",\"no_output_until_cleanup_returned\");\n    ensure(!trace.some(v=>v.kind===\"tool\"&&[\"browser_navigate\",\"browser_snapshot\"].includes(v.label)&&\n      trace.indexOf(v)>firstLatch),\"no_journey_after_failure\");\n    ensure(record.actions.length===4&&record.actions[0].label===\"kill_app\"&&\n      record.actions[1].label===\"end_session\"&&record.actions[2].label===\"list_windows\"&&\n      record.actions[3].label===\"owned_join_readback\",\"exact_action_projection\");\n    if(spec.name===\"metadata_write_failure\")\n      ensure(record.cleanup_errors.includes(\"observation_metadata_write_unconfirmed\"),\n        \"metadata_failure_separate\");\n    if([\"driver_exception\",\"driver_refusal\"].includes(spec.name))\n      ensure(record.cleanup_errors.includes(\"driver_operation_unconfirmed\"),\"driver_failure_recorded\");\n    if(spec.name===\"driver_late_return\")\n      ensure(record.cleanup_errors.includes(\"driver_operation_over_budget\")&&\n        record.actions[0].elapsed_ms>30000,\"late_return_observed_only\");\n    if(spec.name===\"partial_controller_json\")\n      ensure(record.controller_observations.at(-1).helper_completed===true&&\n        record.controller_observations.at(-1).helper_exit===1,\"partial_json_preserves_failure\");\n    if(spec.name===\"join_failure\")\n      ensure(record.cleanup_errors.includes(\"controller_join_unconfirmed\")&&polls===2,\n        \"join_failure_no_retry\");\n    if(spec.name===\"readback_projection_failure\")\n      ensure(record.cleanup_errors.includes(\"owned_readback_unavailable\"),\n        \"projection_failure_separate\");\n    // Resource release uses independent absence proofs; Driver errors remain recorded.\n    // The immutable cell and all error, elapsed and always false whole60 assertions stay unchanged.\n    ensure(result.resource_release_proven===spec.release,\"resource_release_failclosed\");\n  }\n  item.assertions_completed=true;\n  }catch(error){\n    item.assertion=typeof error?.envelope_assertion===\"string\"?error.envelope_assertion:\n      \"envelope_unexpected_failure\";\n  }\n  return item;\n}\nasync function runEnvelope(cell,source){\n  const result={schema:\"riauth.d01-cleanup-memory-envelope/v1\",result:\"failed\",\n    source,planned:PLAN,attempted:0,completed:0,cases:[],first_failure:null,\n    unreached:CASES.map(v=>v.name),counts:{tool_calls:0,driver_calls:0,controller_calls:0,\n      collector_calls:0,clock:0,stop:0,readback:0,persist:0}};\n  ensure(sha(cell)===CELL_PIN,\"immutable_cell_hash\");\n  const py=sources(cell);\n  for(const spec of CASES){\n    result.attempted++;\n    try{\n      const item=await oneCase(spec,cell,py);\n      result.cases.push(item);\n      for(const key of Object.keys(result.counts))result.counts[key]+=item.counts[key];\n      result.unreached=CASES.slice(result.attempted).map(v=>v.name);\n      if(!item.assertions_completed){\n        result.first_failure={case:spec.name,assertion:item.assertion};\n        return result;\n      }\n      result.completed++;\n    }catch(error){\n      result.first_failure={case:spec.name,\n        assertion:typeof error?.envelope_assertion===\"string\"?error.envelope_assertion:\n          \"envelope_unexpected_failure\"};\n      result.unreached=CASES.slice(result.attempted).map(v=>v.name);\n      return result;\n    }\n  }\n  ensure(result.completed===PLAN.cases,\"planned_cases\");\n  for(const key of Object.keys(result.counts))ensure(result.counts[key]===PLAN[key],\"planned_total\");\n  result.result=\"passed\";\n  return result;\n}\n";
const sha=s=>createHash("sha256").update(s).digest("hex");
const source={cell_sha256:sha(CELL),logic_sha256:sha(LOGIC)};
let result;
try {
  const script=new vm.Script(LOGIC+"\nrunEnvelope(CELL,source);",{filename:"finite-envelope"});
  result=await script.runInNewContext({vm,sha,CELL,source},{timeout:1000});
} catch {
  result={schema:"riauth.d01-cleanup-memory-envelope/v1",result:"failed",source,
    first_failure:{case:null,assertion:"envelope_preflight_or_totals_failed"}};
}
process.stdout.write(JSON.stringify(result)+"\n");
process.exitCode=result.result==="passed"?0:1;
```

### Actual static checks and prefix preservation

Actually performed: node --input-type=module --check on corrected readable
logic and complete payload, each exit0; exact original284-line cell/four
collector string identity; full five-edit logic reversal; complete payload
reversal; exactly five changed readable-logic lines; exact CELL/LOGIC JSON
reconstruction;18-case/286-call static count; retained Driver-error/elapsed/
whole60 assertion identity; helper/controller identity; all twelve metadata
mode/byte/hash checks; report-only prefix/archive/fence/final-newline/
whitespace/diff checks. No mocked cell or harness evaluates any assertion.
Parser acceptance is only syntax evidence.

The entire454006-byte01f249c report prefix/SHA
7d505fed30c2fc21576f8938dacf96e9a11b195d4cdc91dd27c0cad4e0914a20
is byte-exact. Nine historical0600 metadata files and three c88 files retain
their full bytes/hashes/modes. All original failures, lost values/reporting
correction, senderUNKNOWN, failed61/76 and dated78 memory limits, and the old
strict unexecuted design/prediction remain dated. No prior phase is overwritten
or presented as a current test.

Only this existing report is appended in one immutable report-only commit.
No other file, product/guide/D05/config/test/workflow/approval/state edit,
main/push/status, worker/task/WT/shell/contact or runtime slot operation occurs.
A future memory run requires a separate explicit root release after review;
no pass, resource-release receipt or browser closure is claimed by this phase.

## Root reviewed corrected cleanup envelope: one actual memory pass

Root fully read the original322-line logic and604-line design, the503-line corrected appendix, and decoded both complete payloads. The corrected three Driver-case expectations and two comments exactly reverse to the old logic; the entire284-line cell and four collector sources are unchanged. The earlier strict expectation was a root requirement error, never an executed failure.

Root released and executed exactly one reviewed37,738-byte payload SHAa06ab7b0783569f8f3778fffedda55dae0b0a824a8174533e77bb273ba702230 with node --input-type=module under a30-second child bound. Exit0, elapsed0.154203s, no timeout, child94785 reaped, stderr0. Complete5,621-byte stdout SHA91f8865ac7715c9a776c91b5a6645ac8bec7e58fc970067629a3f66d430a9776 and numeric outcome were saved/fsynced before JSON/expectation comparisons. The [fixed root receipt](evidence/wave30-d01-cleanup-memory-2bf9ebc.json) retains all18 completed cases,286 stub tool calls/78 Driver/33 controller/175 collector, no first failure or unreached case.

Controlled assertions cover first latch before cleanup, stop/join/absence ordering and retained failure, metadata/Driver/refusal/late errors, each missing proof, partial JSON, failed join, malformed readback, ownership refusal and no journey credit from the application-only branch. Complete independent absence can prove resource release after Driver error or lateness while that error remains recorded and whole60 remains false. The VM used immediately settled stubs; no actual collector, Driver, browser, native helper, product, socket, signal, filesystem cleanup or Cargo operation occurred. No fixture runtime slot was acquired/released. Real first-event/whole60 timing and confidential journey remain unproven; the real fixture remains held pending the distinct observer-aware envelope. All historical failures/private evidence stay unchanged.

## Dated source-only phase: observer-aware controller and decision boundary, 2026-10-03 (Europe/Vaduz)

Reservation: wave30_D01_observer_controller_source_plan. Only this existing
report is appended. No controller/helper/source was materialized or evaluated,
no main alignment occurred, and no runtime/desktop/Cargo slot was acquired or
released. Real confidential fixture remains HELD. Root owns all original gates.

The finite propagation candidate below is complete source for the outer
controller and for a prepared-handle entry/continuation cell. It is NOT a
complete executable preparation pipeline: the exact original Driver-return
adapter and cleanup branch for a prepare failure without an owned returned
browser PID are absent from the retained controller/cell archives. That is one
concrete integration prerequisite, not an invented product defect. The small
preparation boundary is archived separately and names those missing bodies.
Root should not release this proposal as an all-phase fixture until that source
input is reserved and fully reviewed. No guessed Driver JSON mapping, unknown
PID kill, callback or private credential reader is supplied as if verified.

### Immutable inputs and actual versus proposed evidence

Read the complete retained193-line prepared180 controller,14905B/SHA256
6662bdbb168bf0ba6ec5ff24b6f3b88dc15b54b4dfc3b800fdaca1cbee403bbb,
body14882B/SHA256
271b4a00711a084e21792b0310839e69d4fb8779fa5e6d003bde04caf0c82c72;
complete284-line composed cell,18091B/SHA256
0a7cea070bdab32b419a82eb26a7a991b71a2b14a55553f890b53788cda04a5e;
and all four literal collector sources embedded in that cell. These are public
retained source, not raw private request or credential contents.

Read the complete published cleanup-memory JSON from immutable
b71b7b0041a549793233e8c7a81bbb61797e20f3:
docs/roadmap/evidence/wave30-d01-cleanup-memory-2bf9ebc.json,
10004B/SHA256 f2832e4397e69e59bb86680d975f46a06ca1dbdb7c830d4f949823be11bc989a.
Root's actual ONE a06ab7 payload invocation exited0:18/18 cases,286 stub tool
calls,0.15420270897448063s, owned Node child94785 reaped,5621 stdout bytes/SHA256
91f8865ac7715c9a776c91b5a6645ac8bec7e58fc970067629a3f66d430a9776,
zero stderr. I read that receipt; I did not independently execute its payload.
Its78 Driver/33 controller/175 collector stub calls and corrected absence
versus budget separation apply to the immutable old cell. They do not verify
this changed candidate, a real collector/tool/GUI/native path or browser
journey. Earlier strict predicted design remains dated and unexecuted; its
three false release expectations were a root requirement correction.

Read immutable main74e106b819e7186d0ac41964cedbe8217fa7e881 helper observer,
capture sites and main evidence/cleanup excerpts, and Essentials section2/3.
No complete-body review of the entire helper or large independent observer
report is claimed. Source identity:
f4ef05d8428b235511e81277ba6b4b72d5ec08ba, published helper blob
a01b1f3f81f0978eb0a02ed339c7248cae485350,35749B/757lines/SHA256
75bfb8a63d68aee6ee6b2e500959c0e7d80a6331f1c690022c4300134c7d34f1.
The descriptor-memory0ec128 design remains independent-review pending and
UNEXECUTED; no descriptor/native observer pass is assumed.

Two static-source preparation errors are disclosed separately from historical
fixture failures: a mistyped long f4ef source token ending08ba3 caused git
revision lookup exit128; immutable main74 resolved the correct40-character
token ending08ba. First in-memory candidate assembly refused because the two
identical helper-result read sites violated its single-occurrence assertion.
Assembly then explicitly counted both sites; no source file or runtime was
changed. Some broad read output was truncated, so only the complete retained
sources and selected excerpts above are counted as body reads.

The unchanged c01 artifacts are future mandatory fresh identity checks:
riauth7abf745c10691a012a1918c83089168d0e0d88b42764dbf8ed538b90c828b606;
maintenance86490c7f71de6b7ae9d4dabdf9060a8757100f3aedbdaa670284d9e1206a2a95;
riauthctlbfbbb322f1a66d0ac9998beb9fb5838097cea0442cea3fd3a1d057fcb3f600cf.
Verifier9cefe7a56425bb73c17753e8766d92320b77da3b/blob
3be747d03146f1bcaa3ec012ee8d173b61fa737d/SHA256
f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d.
Resolved native provider67a83dd6d6d747d50c5d296dffb23e32bae9a2c588c93ae2d77e4c607b455c72.
Complete63ASCII version stdout plus numeric exit/stderr must be retained
before the unchanged full app+Library equality. No artifact/provider was
invoked or freshly rehashed in this design; helper source and receipt hashes
are distinct from binary identity or body review.

### Proposed finite propagation and decision ordering

Controller changes are restricted to explicit non-executable START/output
placeholders, the helper75 source pin, and diagnostic propagation. The two
helper-result load sites project and sanitize the new nested field before
retaining it or announcing completion. Existing result/failure/stage/request
reason/counter/check/status/CLI/listener/guard/native decisions remain intact.
All nine original controller function ASTs are identical. The180s assignment,
START+840 active/inclusive900, cleanup60, helper600/pending180, native+HTTP5,
CLI60/listener30, issuer9000/callback3000, private0700/0600 setup and1s8.5GiB
stop remain exact; no reset, rebind, replay or extra RP readiness HTTP occurs.

Whitelist shape is exactly:
observed:boolean; diagnostic:null OR a dictionary with exactly
site,exception_class,own_function,own_line. observed:false requires null.
observed:true/diagnostic:null is valid when the helper's first-only projection
failed. Sites are handler/server/main; classes are the eight exact builtin
labels plus other; functions are the ten own-code labels from helper75.
Function/line are either both null or a whitelisted function plus a true
integer1..1024; Python bool is refused as an integer. No exception message,
filename, traceback/frame/locals, request/header/value/URL/query/subject or
sender is retained. First finite observation is preserved, including a null
diagnostic; malformed input gets a fixed observer_projection_invalid error
and no raw fallback. Diagnostic errors never replace first_failure, change
authentication acceptance, give journey credit or withhold independently
proven resource absence. This is transport/source validation, not cause proof.

One prepared-handle cell starts while the existing180s gate still waits and
helper3000 has NOT started. Exact returned Driver-owned PID/window/session/
target/tab must already be cached under the reviewed preparation boundary.
The new marker collector checks only owned guard/server PID liveness and the
private own lab, then writes the existing zero0600 marker exclusively.
In that same composed call: wait the owned controller fixture_ready event
within30s; one immediate controller poll; navigate/snapshot protected-before;
one poll; navigate/snapshot application entry; one poll. There is no text,
notify, yield_control, instruction/model wait, description/skill load, reprepare
or rebind between marker/listener/entry. Protected403 is ultimately the helper
status/check, not merely a browser page text or transport success.

Every tool action in the archived cell uses checked/pollController or the
existing finite cleanup runners. A returned receipt clock is stored before
predicate comparison. A failure latches synchronously once, then requests the
original stop and completes essential owned cleanup before text is possible.
The outer catch preserves that latch on projection/decision exceptions. Later
browser actions must use this same cell's browser_decision mode, with a
single-use latest-snapshot ref, exactly cached target/tab/session and a fixed
public expected role/label. There is no accepted arbitrary URL, script, HTTP
login, terminal approval, service session, cookie/header injection or physical
passkey path. Synthetic DOM click dispatch is not treated as control effect;
its fresh snapshot predicate is required and journey_credit stays false.
No action may be appended as an unguarded tool call after this source.

The continuation skeleton permits username/password/click/snapshot plus a
read-only protected-after snapshot. It never navigates a private callback URL
or sends an extra protected request after callback completion; helper source
enforces the exact callback and records native/JWKS/issuer/RS256/nonce/audience/
time/access-hash/userinfo/cookie checks. All candidate outputs retain
journey_credit:false: label/ref presence alone is not proof of password
insertion, empty optional factor, consent, callback or crypto completion.
Root must review the exact future step predicates and private-password reader/
transfer before actual journey use. d01_fresh_password_input is a required
ephemeral input, never a literal credential or durable evidence value.
Unknown/unmatched shape stops, without selector fallback or retry.

The preparation boundary source below is deliberately not an executable
receipt adapter. browser_prepare advertises a generic CallToolResult; retained
prose proves exact owned identities and binding quality but does not provide
the complete prepare/bind JSON-key adapter or the no-returned-PID cleanup body.
The old public controller and284-line cell do not contain those bodies.
The required future source input is the exact ONE prepare/list_windows/bind
caller with its finite returned-identity projection and partial-ownership
cleanup. On a preparation refusal/exception, latch first, join the original
controller using own lab stop, end the exact known session, and kill only a
proven returned driver-owned browser PID; unknown PID means no kill and no
absence/release claim. Every preparation decision must use that boundary
before any output. Root needs to reserve and review those exact bodies before
executing even this prospective all-phase sequence. No guessed response keys
or replacement provider is introduced here.

### Cleanup contract and source placement

Failure cleanup preserves STOP then one exact Driver kill_app, end_session,
list_windows, original controller join, fresh owned PID/port/lab readback and
exclusive metadata persistence. Unknown ownership suppresses kill/release.
The adapted readback covers six fixed children if helper was never created,
or the original seven if helper ran; it learns a helper PID only from the fresh
canonical owned-controller record and matching named child exits, then checks
its absence too. It reads only bounded262144B owned0600 canonical evidence,
fixed PID ps output and9000/3000 listener ownership, never broader args/env.
No separate peer, pressure or unrelated process observation occurs.

STOP's sole extra branch writes stop without ui-failure on normal final-page
cleanup; all failed paths still write both original markers. This is necessary
to avoid converting observed success to browser_checkpoint_failed. It changes
no public/authentication/helper behavior. CLOCK and PERSIST are byte-identical.
READBACK's diagnostic projection and pre-helper resource proof are declared
integration changes, not hidden observer/helper changes.

The prior actual18-case memory pass permits resource release only after
complete independent join/session/window/PID/port/lab proofs. Failed or late
Driver operations and metadata errors remain separate cleanup errors even
when those absence proofs pass. Missing any proof or any unjoined collector
means resource_release_proven:false.30+15+5+remaining60 are observational
allocations; MCP advertises no hard per-call timeout/cancel. Whole60 and true
first-event proof remain false; no new latency guarantee or retrospective
cleanup anchor is claimed. Essential cleanup still proceeds when late.

WT7 currently contains unchanged helper470:
32723B/SHA2567fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0.
The new75 source pin would correctly refuse this tree. AFTER descriptor-memory
PASS, independent/full root review and an explicit history/source reservation,
propose this single history-preserving alignment, not a reset:
git merge --no-ff --no-edit 74e106b819e7186d0ac41964cedbe8217fa7e881
in WT7. Root decides/owns conflict resolution; pause on conflicts, preserve
both histories and every report prefix, and rehash757-line helper75. No merge,
cherry-pick, helper replacement or alignment was performed here.

Fresh START is intentionally float('__ROOT_RELEASED_START_ONCE__'), which
refuses as written. OUT/PROVIDER_META remain literal root-placeholder
basenames. Root must separately declare one numeric START and distinct absent
exclusive0600 output/event/cleanup paths, check >=8.5GiB/unused9000+3000 before
any setup and archive/hash actual substituted payload before invoking it.
No current resource reservation, capacity claim, permission check or future
release is inferred from this report.

### Complete archived candidate sources — NOT EXECUTED
Static source receipt; node --check and Python parse/code-object compile only:

```json
{
  "cell": {
    "bytes": 31461,
    "changed_occurrences": 22,
    "edit_units": 22,
    "lines": 463,
    "node_syntax_exit": 0,
    "sha256": "eff0cdad3981ce0ad57d355203b082e21371553408444bf46a3b4ecfcf5fea0a"
  },
  "collectors": {
    "clock": {
      "bytes": 114,
      "lines": 2,
      "sha256": "cfb3f3154cb778388724728b6f7cb804bd7e142844ded7c044afc8dd3135ab4d"
    },
    "marker": {
      "bytes": 626,
      "lines": 10,
      "sha256": "13c6549e0328ad695769938cbc0be7eb47d7a10ea9f00c4d897655603437a949"
    },
    "persist": {
      "bytes": 805,
      "lines": 18,
      "sha256": "819a7785625ce2f78a9cb03b5f242dbfa6728f9f507b242e3c048887ad01ea30"
    },
    "readback": {
      "bytes": 4424,
      "lines": 52,
      "sha256": "b7d3504245ae94a584b455fa65826fbaa7759ec7a50a7b81c2cf8284c9f9051f"
    },
    "stop": {
      "bytes": 1600,
      "lines": 29,
      "sha256": "5de5244440b134689bedfb1235e805c65a8b726e753cbdac1d0831ef1c83d071"
    }
  },
  "controller": {
    "body_bytes": 17303,
    "body_lines": 223,
    "body_sha256": "ce8d3f7b5cc08ce114e7f60a69a498305dadcc51956c8824d362213b58fc0efa",
    "bytes": 17326,
    "changed_occurrences": 10,
    "edit_units": 9,
    "lines": 225,
    "nine_existing_function_asts_unchanged": true,
    "sha256": "5eb6ab2d3d23b49fd431c41f1932082fe0c6779ad724ac77b7fded10926b26b0"
  },
  "helper470_unchanged": true,
  "not_executed": true,
  "prefix_bytes": 518404,
  "prefix_sha256": "2ae110bff13b1943ee8edf0e8b2fb72f38ce417052032cb653fc945d62c4a982",
  "preparation_boundary": {
    "bytes": 2217,
    "lines": 37,
    "node_syntax_exit": 0,
    "sha256": "1fb412fbf551a3f76595fca551adb2828775cdd3dfa1801e798426557a6e1355"
  },
  "schema": "riauth.d01-observer-controller-design-static/v1",
  "twelve_old_private_metadata_unchanged": true,
  "whole_reversal_controller_and_cell": true
}
```

Complete outer controller command:

```sh
python3 -u - <<'PY'
import datetime,fcntl,hashlib,json,os,pathlib,pty,secrets,select,shutil,signal,stat,subprocess,sys,tempfile,termios,time,urllib.request
ROOT=pathlib.Path.cwd()
START=float('__ROOT_RELEASED_START_ONCE__')
ACTIVE=START+840
BIN=pathlib.Path('/Users/dominik/orca/projects/riAuth-public-preview-local-module-boundaries-wave27/target/d01-essentials-c01c39a/aarch64-apple-darwin/debug')
PRIVATE=ROOT/'deployment-private'
OUT=PRIVATE/'__ROOT_FRESH_OBSERVER_OUT_BASENAME__'
PROVIDER_META=PRIVATE/'__ROOT_FRESH_OBSERVER_PROVIDER_BASENAME__'
record={'schema':'riauth.d01-confidential-browser-outer/v1','result':'failed','failure_stage':None,'failure_tag':None,'request_invalid_reason':None,'preflow_authorization_refusals':None,'unexpected_failure_observation':None,'unexpected_observation_projection':'unavailable','commands':[],'artifacts':{},'cleanup':{},'owned_child_exits':[],'started_epoch':START,'active_seconds':840,'cleanup_seconds':60,'minimum_free_bytes':None,'disk_samples':0}
lab=None;children=[];names={};server=None;helper=None;password=None;stage='preflight';resultfd=None;lastsample=0
def require(ok,tag):
    if not ok:raise RuntimeError(tag)
def tick():
    global lastsample
    now=time.monotonic()
    if now-lastsample>=1:
        free=shutil.disk_usage(ROOT).free
        record['disk_samples']+=1
        record['minimum_free_bytes']=free if record['minimum_free_bytes'] is None else min(record['minimum_free_bytes'],free)
        lastsample=now
        require(free>=8.5*1024**3,'disk_margin')
    require(time.time()<ACTIVE,'active_deadline')
def write_exclusive(path,value):
    fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    raw=(json.dumps(value,sort_keys=True,indent=2)+'\n').encode('ascii')
    with os.fdopen(fd,'wb') as f:
        f.write(raw);f.flush();os.fsync(f.fileno())
    return hashlib.sha256(raw).hexdigest()
def finite_observation(value):
    if type(value) is not dict or set(value)!= {'observed','diagnostic'} or type(value['observed']) is not bool:
        return None
    diagnostic=value['diagnostic']
    if diagnostic is None:
        return {'observed':value['observed'],'diagnostic':None}
    if not value['observed'] or type(diagnostic) is not dict or set(diagnostic)!= {'site','exception_class','own_function','own_line'}:
        return None
    sites=('handler','server','main')
    kinds=('AttributeError','TypeError','ValueError','KeyError','OSError','BrokenPipeError','ConnectionResetError','TimeoutError','other')
    functions=('HeaderReader.readline','DemoServer.process_request','Demo.begin','Demo.callback','Demo.invoke','Handler.handle_one_request','Handler.send_error','Handler.get','Handler.reply','main')
    site,kind,function,line=(diagnostic[k] for k in ('site','exception_class','own_function','own_line'))
    if type(site) is not str or site not in sites or type(kind) is not str or kind not in kinds:
        return None
    if not ((function is None and line is None) or (type(function) is str and function in functions and type(line) is int and 1<=line<=1024)):
        return None
    return {'observed':True,'diagnostic':{'site':site,'exception_class':kind,'own_function':function,'own_line':line}}
def retain_observation():
    # Sanitize even the nested helper_result field; never copy malformed details.
    value=record['helper_result'].get('unexpected_failure_observation')
    projected=finite_observation(value)
    record['helper_result']['unexpected_failure_observation']=projected
    if projected is None:
        record['unexpected_observation_projection']='invalid'
    elif record['unexpected_failure_observation'] is None:
        record['unexpected_failure_observation']=projected
        if record['unexpected_observation_projection']!='invalid':
            record['unexpected_observation_projection']='valid'
def listeners(port):
    p=subprocess.run(['/usr/sbin/lsof','-nP','-t','-iTCP:'+str(port),'-sTCP:LISTEN'],capture_output=True,timeout=3)
    require(p.returncode in (0,1),'socket_observation_failed')
    return set(int(v) for v in p.stdout.split())
def own(p,name):
    children.append(p);names[p.pid]=name;return p
def stop_child(p):
    if p.poll() is None:
        for sig,wait in [(signal.SIGINT,8),(signal.SIGTERM,5),(signal.SIGKILL,2)]:
            if p.poll() is not None:break
            p.send_signal(sig)
            try:p.wait(timeout=wait)
            except subprocess.TimeoutExpired:pass
    require(p.poll() is not None,'owned_child_reap_failed')
def controlling_tty():
    os.setsid();fcntl.ioctl(0,termios.TIOCSCTTY,0)
def cli(name,args,prompts=0):
    tick()
    master,slave=pty.openpty()
    p=own(subprocess.Popen(args,cwd=lab,env=environment,stdin=slave,stdout=slave,stderr=slave,preexec_fn=controlling_tty),name)
    os.close(slave);seen=0;raw=b'';started=time.monotonic()
    try:
        while p.poll() is None:
            tick();require(time.monotonic()-started<60,'cli_deadline')
            if select.select([master],[],[],0.2)[0]:
                try:chunk=os.read(master,4096)
                except OSError:chunk=b''
                raw+=chunk;require(len(raw)<=131072,'cli_output_limit')
                for prompt in ([b'Password: ',b'Confirm password: '] if prompts==2 else [b'Password: ']):
                    if seen<prompts and prompt in raw:
                        require(prompt==([b'Password: ',b'Confirm password: '][seen] if prompts==2 else b'Password: '),'cli_prompt_order')
                        os.write(master,password.encode()+b'\n');seen+=1;raw=b''
        while select.select([master],[],[],0)[0]:
            try:
                chunk=os.read(master,4096)
                if not chunk:break
                raw+=chunk
            except OSError:break
        code=p.wait()
        record['commands'].append({'name':name,'exit':code,'password_prompts':seen})
        require(code==0,'cli_nonzero');require(seen==prompts,'cli_prompt_missing')
    finally:
        raw=b'';os.close(master)
        if p.poll() is None:stop_child(p)
def wait_listener(p,port):
    deadline=time.monotonic()+30
    while time.monotonic()<deadline:
        tick();require(p.poll() is None,'owned_service_early_exit')
        owners=listeners(port)
        if owners:
            require(owners=={p.pid},'listener_owner_mismatch');return
        time.sleep(0.2)
    raise RuntimeError('listener_deadline')
try:
    tick()
    require(PRIVATE.is_dir() and not PRIVATE.is_symlink() and stat.S_IMODE(PRIVATE.stat().st_mode)==0o700,'private_directory_invalid')
    require(not OUT.exists() and not PROVIDER_META.exists(),'evidence_already_exists')
    require(not listeners(9000) and not listeners(3000),'port_occupied');record['ports_preflight_empty']=True
    pins={'riauth':'7abf745c10691a012a1918c83089168d0e0d88b42764dbf8ed538b90c828b606','riauth-maintenance':'86490c7f71de6b7ae9d4dabdf9060a8757100f3aedbdaa670284d9e1206a2a95','riauthctl':'bfbbb322f1a66d0ac9998beb9fb5838097cea0442cea3fd3a1d057fcb3f600cf'}
    for name,pin in pins.items():
        tick()
        with (BIN/name).open('rb') as f:digest=hashlib.file_digest(f,'sha256').hexdigest()
        record['artifacts'][name]=digest;require(digest==pin,'artifact_hash_mismatch')
    helper_path=ROOT/'scripts/d01-confidential-browser-demo.py'
    require(hashlib.sha256(helper_path.read_bytes()).hexdigest()=='75bfb8a63d68aee6ee6b2e500959c0e7d80a6331f1c690022c4300134c7d34f1','helper_hash_mismatch')
    verifier=subprocess.check_output(['git','show','9cefe7a56425bb73c17753e8766d92320b77da3b:scripts/recovery-drill-oidc.py'],timeout=5)
    require(hashlib.sha256(verifier).hexdigest()=='f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d','verifier_hash_mismatch')
    provider=pathlib.Path('/opt/homebrew/bin/openssl').resolve(strict=True)
    with provider.open('rb') as f:provider_hash=hashlib.file_digest(f,'sha256').hexdigest()
    require(provider_hash=='67a83dd6d6d747d50c5d296dffb23e32bae9a2c588c93ae2d77e4c607b455c72','provider_hash_mismatch')
    provider_env={k:v for k,v in os.environ.items() if k in {'PATH','HOME','TMPDIR','LANG','LC_ALL'}}
    p=subprocess.Popen([str(provider),'version'],stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=provider_env)
    try:stdout,stderr=p.communicate(timeout=5);timeout=False
    except subprocess.TimeoutExpired:
        p.kill();stdout,stderr=p.communicate(timeout=2);timeout=True
    provider_record={'sha256':provider_hash,'exit':p.returncode,'timeout':timeout,'stdout_ascii':stdout[:4096].decode('ascii',errors='backslashreplace'),'stderr_ascii':stderr[:4096].decode('ascii',errors='backslashreplace'),'stdout_bytes':len(stdout),'stderr_bytes':len(stderr),'environment_keys':sorted(provider_env)}
    record['provider_metadata_sha256']=write_exclusive(PROVIDER_META,provider_record)
    require(not timeout and p.returncode==0 and len(stdout)<=256 and len(stderr)<=4096 and stdout.decode('ascii').strip()=='OpenSSL 3.6.4 25 Aug 2026 (Library: OpenSSL 3.6.4 25 Aug 2026)','provider_version_failed')
    tick()
    resultfd=os.open(OUT,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    lab=pathlib.Path(tempfile.mkdtemp(prefix='d01-confidential-browser-diagnostic.',dir=PRIVATE));os.chmod(lab,0o700)
    for name in ('xdg','deployment-private','rp'):(lab/name).mkdir(mode=0o700)
    (lab/'recovery-drill-oidc.py').write_bytes(verifier);os.chmod(lab/'recovery-drill-oidc.py',0o600);verifier=None
    password=secrets.token_urlsafe(30)
    fd=os.open(lab/'browser-password',os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
    with os.fdopen(fd,'w') as f:f.write(password)
    environment=dict(provider_env);environment['XDG_CONFIG_HOME']=str(lab/'xdg')
    stage='init';cli('maintenance_init',[str(BIN/'riauth-maintenance'),'--config',str(lab/'riauth.toml'),'init','--issuer','http://localhost:9000','--listen','127.0.0.1:9000','--data-dir','data','--admin','admin'],2)
    stage='serve'
    server=own(subprocess.Popen([str(BIN/'riauth'),'--config',str(lab/'riauth.toml'),'serve'],cwd=lab,env=environment,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL),'server')
    record['server_pid']=server.pid;wait_listener(server,9000);record['server_listener_owned']=True
    stage='readyz'
    with urllib.request.urlopen('http://127.0.0.1:9000/readyz',timeout=5) as response:
        record['readyz_status']=response.status;response.read(4096)
    require(record['readyz_status']==200,'readyz_failed')
    stage='cli_login'
    base=[str(BIN/'riauthctl'),'--server','http://localhost:9000']
    cli('operator_login',base+['login','admin'],1)
    stage='client_create';secretpath=lab/'deployment-private/local-demo-secret.json'
    cli('confidential_client_create',base+['client','create','local-demo','--name','Local demo','--confidential','--redirect-uri','http://localhost:3000/callback','--scope','openid,profile','--secret-file',str(secretpath)])
    require(stat.S_IMODE(secretpath.stat().st_mode)==0o600,'cli_secret_mode_invalid')
    stage='discovery';cli('discovery',base+['discovery'])
    stage='whoami';cli('whoami',base+['whoami'])
    stage='browser_prepare'
    prepare_deadline=time.monotonic()+180
    prepare_marker=lab/'browser-prepared'
    print(json.dumps({'browser_prepare_required':True,'guard_pid':os.getpid(),'server_pid':server.pid,'lab':str(lab)}),flush=True)
    while not prepare_marker.exists():
        tick();require(server.poll() is None,'idp_early_exit')
        require(not (lab/'ui-failure').exists() and not (lab/'stop').exists(),'browser_checkpoint_failed')
        require(time.monotonic()<prepare_deadline,'browser_prepare_deadline')
        time.sleep(0.2)
    tick();require(server.poll() is None,'idp_early_exit')
    require(not (lab/'ui-failure').exists() and not (lab/'stop').exists(),'browser_checkpoint_failed')
    require(time.monotonic()<prepare_deadline,'browser_prepare_deadline')
    prepare_info=prepare_marker.lstat()
    require(stat.S_ISREG(prepare_info.st_mode) and stat.S_IMODE(prepare_info.st_mode)==0o600 and prepare_info.st_uid==os.getuid() and prepare_info.st_nlink==1 and prepare_info.st_size==0,'browser_prepare_marker_invalid')
    stage='rp_start'
    helper=own(subprocess.Popen([sys.executable,'-B',str(helper_path),'--workspace',str(lab/'rp'),'--secret-file',str(secretpath),'--verifier-helper',str(lab/'recovery-drill-oidc.py'),'--openssl','/opt/homebrew/bin/openssl','--deadline-seconds','600','--evidence',str(lab/'rp-result.json')],cwd=ROOT,env=environment,stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,text=True),'helper')
    record['helper_pid']=helper.pid;record['helper_invocations']=1
    wait_listener(helper,3000);record['helper_listener_owned']=True
    print(json.dumps({'fixture_ready':True,'guard_pid':os.getpid(),'server_pid':server.pid,'helper_pid':helper.pid,'lab':str(lab),'commands':record['commands'],'provider_metadata_sha256':record['provider_metadata_sha256'],'minimum_free_bytes':record['minimum_free_bytes']}),flush=True)
    stage='browser_checkpoint';announced=False
    while not (lab/'stop').exists():
        tick();require(server.poll() is None,'idp_early_exit')
        if helper.poll() is not None:
            code=helper.wait()
            if not announced:
                record['helper_exit']=code;record['helper_result']=json.loads((lab/'rp-result.json').read_bytes())
                retain_observation()
                record['request_invalid_reason']=record['helper_result']['request_invalid_reason']
                record['preflow_authorization_refusals']=record['helper_result']['preflow_authorization_refusals']
                print(json.dumps({'helper_completed':True,'exit':code,'result':record['helper_result']['result'],'failure_tag':record['helper_result']['failure_tag'],'request_invalid_reason':record['request_invalid_reason'],'preflow_authorization_refusals':record['preflow_authorization_refusals'],'unexpected_failure_observation':record['unexpected_failure_observation'],'unexpected_observation_projection':record['unexpected_observation_projection']}),flush=True);announced=True
            require(code==0,'rp_nonzero')
        time.sleep(0.2)
    if (lab/'ui-failure').exists():raise RuntimeError('browser_checkpoint_failed')
    require(helper.poll()==0,'rp_checkpoint_incomplete');record['result']='passed'
except Exception as error:
    record['failure_stage']=stage
    tags={'disk_margin','active_deadline','private_directory_invalid','evidence_already_exists','port_occupied','socket_observation_failed','artifact_hash_mismatch','helper_hash_mismatch','verifier_hash_mismatch','provider_hash_mismatch','provider_version_failed','cli_deadline','cli_output_limit','cli_prompt_order','cli_prompt_missing','cli_nonzero','owned_service_early_exit','listener_owner_mismatch','listener_deadline','readyz_failed','cli_secret_mode_invalid','idp_early_exit','rp_nonzero','browser_checkpoint_failed','rp_checkpoint_incomplete','browser_prepare_deadline','browser_prepare_marker_invalid'}
    record['failure_tag']=str(error) if str(error) in tags else 'outer_unexpected_failure'
finally:
    for p in reversed(children):
        try:stop_child(p)
        except Exception:record['cleanup']['child_reap_failure']=True
        record['owned_child_exits'].append({'name':names[p.pid],'pid':p.pid,'exit':p.poll()})
    record['cleanup']['owned_children_reaped']=all(p.poll() is not None for p in children)
    if lab is not None:
        if helper is not None:
            record['helper_exit']=helper.poll()
            if (lab/'rp-result.json').exists():
                record['helper_result']=json.loads((lab/'rp-result.json').read_bytes())
                retain_observation()
                record['request_invalid_reason']=record['helper_result']['request_invalid_reason']
                record['preflow_authorization_refusals']=record['helper_result']['preflow_authorization_refusals']
        password=None;shutil.rmtree(lab);record['cleanup']['lab_removed']=not lab.exists()
    else:record['cleanup']['lab_removed']=True
    try:
        record['cleanup']['port9000_absent']=not listeners(9000);record['cleanup']['port3000_absent']=not listeners(3000)
    except Exception:record['cleanup']['socket_observation_failure']=True
    record['completed_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();record['elapsed_seconds']=round(time.time()-START,3)
    if resultfd is None and not OUT.exists():resultfd=os.open(OUT,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    if resultfd is not None:
        with os.fdopen(resultfd,'wb') as f:
            f.write((json.dumps(record,sort_keys=True,indent=2)+'\n').encode('ascii'));f.flush();os.fsync(f.fileno())
    print(json.dumps({'fixture_finished':True,'result':record['result'],'failure_stage':record['failure_stage'],'failure_tag':record['failure_tag'],'request_invalid_reason':record['request_invalid_reason'],'preflow_authorization_refusals':record['preflow_authorization_refusals'],'unexpected_failure_observation':record['unexpected_failure_observation'],'unexpected_observation_projection':record['unexpected_observation_projection'],'cleanup':record['cleanup'],'owned_child_exits':record['owned_child_exits'],'minimum_free_bytes':record['minimum_free_bytes'],'disk_samples':record['disk_samples'],'elapsed_seconds':record['elapsed_seconds']}),flush=True)
sys.exit(0 if record['result']=='passed' else 1)
PY
```

Exact zero-context controller delta from6662:

```diff
--- retained-controller-6662
+++ DESIGN-only-observer-controller
@@ -4 +4 @@
-START=1790968887.308
+START=float('__ROOT_RELEASED_START_ONCE__')
@@ -8,3 +8,3 @@
-OUT=PRIVATE/'d01-confidential-browser-budget180.redacted.json'
-PROVIDER_META=PRIVATE/'d01-confidential-browser-budget180-provider-20261002.json'
-record={'schema':'riauth.d01-confidential-browser-outer/v1','result':'failed','failure_stage':None,'failure_tag':None,'request_invalid_reason':None,'preflow_authorization_refusals':None,'commands':[],'artifacts':{},'cleanup':{},'owned_child_exits':[],'started_epoch':START,'active_seconds':840,'cleanup_seconds':60,'minimum_free_bytes':None,'disk_samples':0}
+OUT=PRIVATE/'__ROOT_FRESH_OBSERVER_OUT_BASENAME__'
+PROVIDER_META=PRIVATE/'__ROOT_FRESH_OBSERVER_PROVIDER_BASENAME__'
+record={'schema':'riauth.d01-confidential-browser-outer/v1','result':'failed','failure_stage':None,'failure_tag':None,'request_invalid_reason':None,'preflow_authorization_refusals':None,'unexpected_failure_observation':None,'unexpected_observation_projection':'unavailable','commands':[],'artifacts':{},'cleanup':{},'owned_child_exits':[],'started_epoch':START,'active_seconds':840,'cleanup_seconds':60,'minimum_free_bytes':None,'disk_samples':0}
@@ -29,0 +30,28 @@
+def finite_observation(value):
+    if type(value) is not dict or set(value)!= {'observed','diagnostic'} or type(value['observed']) is not bool:
+        return None
+    diagnostic=value['diagnostic']
+    if diagnostic is None:
+        return {'observed':value['observed'],'diagnostic':None}
+    if not value['observed'] or type(diagnostic) is not dict or set(diagnostic)!= {'site','exception_class','own_function','own_line'}:
+        return None
+    sites=('handler','server','main')
+    kinds=('AttributeError','TypeError','ValueError','KeyError','OSError','BrokenPipeError','ConnectionResetError','TimeoutError','other')
+    functions=('HeaderReader.readline','DemoServer.process_request','Demo.begin','Demo.callback','Demo.invoke','Handler.handle_one_request','Handler.send_error','Handler.get','Handler.reply','main')
+    site,kind,function,line=(diagnostic[k] for k in ('site','exception_class','own_function','own_line'))
+    if type(site) is not str or site not in sites or type(kind) is not str or kind not in kinds:
+        return None
+    if not ((function is None and line is None) or (type(function) is str and function in functions and type(line) is int and 1<=line<=1024)):
+        return None
+    return {'observed':True,'diagnostic':{'site':site,'exception_class':kind,'own_function':function,'own_line':line}}
+def retain_observation():
+    # Sanitize even the nested helper_result field; never copy malformed details.
+    value=record['helper_result'].get('unexpected_failure_observation')
+    projected=finite_observation(value)
+    record['helper_result']['unexpected_failure_observation']=projected
+    if projected is None:
+        record['unexpected_observation_projection']='invalid'
+    elif record['unexpected_failure_observation'] is None:
+        record['unexpected_failure_observation']=projected
+        if record['unexpected_observation_projection']!='invalid':
+            record['unexpected_observation_projection']='valid'
@@ -94 +122 @@
-    require(hashlib.sha256(helper_path.read_bytes()).hexdigest()=='7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0','helper_hash_mismatch')
+    require(hashlib.sha256(helper_path.read_bytes()).hexdigest()=='75bfb8a63d68aee6ee6b2e500959c0e7d80a6331f1c690022c4300134c7d34f1','helper_hash_mismatch')
@@ -158,0 +187 @@
+                retain_observation()
@@ -161 +190 @@
-                print(json.dumps({'helper_completed':True,'exit':code,'result':record['helper_result']['result'],'failure_tag':record['helper_result']['failure_tag'],'request_invalid_reason':record['request_invalid_reason'],'preflow_authorization_refusals':record['preflow_authorization_refusals']}),flush=True);announced=True
+                print(json.dumps({'helper_completed':True,'exit':code,'result':record['helper_result']['result'],'failure_tag':record['helper_result']['failure_tag'],'request_invalid_reason':record['request_invalid_reason'],'preflow_authorization_refusals':record['preflow_authorization_refusals'],'unexpected_failure_observation':record['unexpected_failure_observation'],'unexpected_observation_projection':record['unexpected_observation_projection']}),flush=True);announced=True
@@ -180,0 +210 @@
+                retain_observation()
@@ -193 +223 @@
-    print(json.dumps({'fixture_finished':True,'result':record['result'],'failure_stage':record['failure_stage'],'failure_tag':record['failure_tag'],'request_invalid_reason':record['request_invalid_reason'],'preflow_authorization_refusals':record['preflow_authorization_refusals'],'cleanup':record['cleanup'],'owned_child_exits':record['owned_child_exits'],'minimum_free_bytes':record['minimum_free_bytes'],'disk_samples':record['disk_samples'],'elapsed_seconds':record['elapsed_seconds']}),flush=True)
+    print(json.dumps({'fixture_finished':True,'result':record['result'],'failure_stage':record['failure_stage'],'failure_tag':record['failure_tag'],'request_invalid_reason':record['request_invalid_reason'],'preflow_authorization_refusals':record['preflow_authorization_refusals'],'unexpected_failure_observation':record['unexpected_failure_observation'],'unexpected_observation_projection':record['unexpected_observation_projection'],'cleanup':record['cleanup'],'owned_child_exits':record['owned_child_exits'],'minimum_free_bytes':record['minimum_free_bytes'],'disk_samples':record['disk_samples'],'elapsed_seconds':record['elapsed_seconds']}),flush=True)
```

Complete prepared-handle entry/continuation cell:

```javascript
// Prospective ONE functions.exec cell; NOT executed in this design phase.
const CLOCK="import json,time\nprint(json.dumps({'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())}))\n";
const STOP="import json,os,pathlib,stat,sys,time\nc=json.loads(sys.argv[1])\nclock={'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())}\nrecord={'schema':'riauth.d01-first-observation/v1','first_failure':c['first_failure'],'first_observation_wall_ms':c['first_observation_wall_ms'],'first_event_proven':False,'clock':clock}\nevent_state='write_unconfirmed'\ntry:\n    fd=os.open(pathlib.Path(c['event_out']),os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\n    with os.fdopen(fd,'w',encoding='ascii') as f:\n        f.write(json.dumps(record,sort_keys=True)+'\\n');f.flush();os.fsync(f.fileno())\n    event_state='written'\nexcept OSError:pass\n# Attempt clock persistence BEFORE marker/state/budget comparisons.\n# A metadata error does not prevent the original essential stop protocol.\nlab=pathlib.Path(c['lab']);marker_state='lab_absent'\ntry:\n    if lab.exists():\n        info=lab.lstat()\n        if not stat.S_ISDIR(info.st_mode) or stat.S_IMODE(info.st_mode)!=0o700 or info.st_uid!=os.getuid():\n            marker_state='ownership_unknown'\n        else:\n            marker_state='stop_requested'\n            for name in (('ui-failure','stop') if c['first_failure'] is not None else ('stop',)):\n                try:\n                    fd=os.open(lab/name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\n                    os.close(fd)\n                except FileNotFoundError:marker_state='lab_absent'\n                except FileExistsError:pass\nexcept OSError:marker_state='stop_unconfirmed'\nprint(json.dumps({'clock':clock,'event_state':event_state,'marker_state':marker_state}))\n";
const READBACK="import json,os,pathlib,stat,subprocess,sys,time\nc=json.loads(sys.argv[1])\nchildren=None;helper_pid=c['helper_pid'];outer_observation=None;projection='unavailable'\ndef finite_observation(value):\n    if type(value) is not dict or set(value)!= {'observed','diagnostic'} or type(value['observed']) is not bool:\n        return None\n    diagnostic=value['diagnostic']\n    if diagnostic is None:\n        return {'observed':value['observed'],'diagnostic':None}\n    if not value['observed'] or type(diagnostic) is not dict or set(diagnostic)!= {'site','exception_class','own_function','own_line'}:\n        return None\n    sites=('handler','server','main')\n    kinds=('AttributeError','TypeError','ValueError','KeyError','OSError','BrokenPipeError','ConnectionResetError','TimeoutError','other')\n    functions=('HeaderReader.readline','DemoServer.process_request','Demo.begin','Demo.callback','Demo.invoke','Handler.handle_one_request','Handler.send_error','Handler.get','Handler.reply','main')\n    site,kind,function,line=(diagnostic[k] for k in ('site','exception_class','own_function','own_line'))\n    if type(site) is not str or site not in sites or type(kind) is not str or kind not in kinds:\n        return None\n    if not ((function is None and line is None) or (type(function) is str and function in functions and type(line) is int and 1<=line<=1024)):\n        return None\n    return {'observed':True,'diagnostic':{'site':site,'exception_class':kind,'own_function':function,'own_line':line}}\nouter=pathlib.Path(c['outer_out'])\nif outer.is_file():\n    fd=os.open(outer,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)\n    try:\n        info=os.fstat(fd)\n        if not (stat.S_ISREG(info.st_mode) and info.st_uid==os.getuid() and stat.S_IMODE(info.st_mode)==0o600 and 0<info.st_size<=262144):\n            raise ValueError('fixed_outer_evidence_invalid')\n        with os.fdopen(fd,'rb',closefd=False) as source:raw_bytes=source.read(262145)\n        if not 0<len(raw_bytes)<=262144:raise ValueError('fixed_outer_evidence_invalid')\n        data=json.loads(raw_bytes)\n    finally:os.close(fd)\n    outer_observation=finite_observation(data.get('unexpected_failure_observation'))\n    projection=data.get('unexpected_observation_projection')\n    if projection not in ('unavailable','valid','invalid'):projection='invalid'\n    if projection=='valid' and outer_observation is None:projection='invalid'\n    allowed={'whoami','discovery','confidential_client_create','operator_login','server','maintenance_init'}\n    started=data.get('helper_invocations')==1 and type(data.get('helper_pid')) is int and data['helper_pid']>0\n    if started:allowed.add('helper')\n    raw=data.get('owned_child_exits')\n    if isinstance(raw,list) and len(raw)==len(allowed) and all(isinstance(v,dict) and v.get('name') in allowed and type(v.get('pid')) is int and v['pid']>0 and type(v.get('exit')) is int for v in raw) and {v['name'] for v in raw}==allowed and len({v['pid'] for v in raw})==len(allowed) and next(v['pid'] for v in raw if v['name']=='server')==c['server_pid'] and ((started and next(v['pid'] for v in raw if v['name']=='helper')==data['helper_pid'] and (helper_pid is None or helper_pid==data['helper_pid'])) or (not started and helper_pid is None and data.get('helper_invocations') is None and data.get('helper_pid') is None)):\n        children=[{k:v[k] for k in ('name','pid','exit')} for v in raw]\n        helper_pid=data['helper_pid'] if started else None\npids=list(c['owned_pids'])\nif helper_pid is not None and helper_pid not in pids:pids.append(helper_pid)\np=subprocess.run(['/bin/ps','-p',','.join(str(v) for v in pids),'-o','pid='],capture_output=True,timeout=3)\nps_known=p.returncode in (0,1) and all(v.isdigit() for v in p.stdout.split())\npresent=set(int(v) for v in p.stdout.split()) if ps_known else set()\nports={}\nfor port in (9000,3000):\n    p=subprocess.run(['/usr/sbin/lsof','-nP','-t','-iTCP:'+str(port),'-sTCP:LISTEN'],capture_output=True,timeout=3)\n    ports[str(port)]=not bool(p.stdout.strip()) if p.returncode in (0,1) else None\nprint(json.dumps({'clock':{'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())},'owned_pids':{str(v):(v not in present if ps_known else None) for v in pids},'ports':ports,'lab_absent':not pathlib.Path(c['lab']).exists(),'owned_child_exits':children,'helper_pid':helper_pid,'unexpected_failure_observation':outer_observation,'unexpected_observation_projection':projection}))\n";
const MARKER="import os,pathlib,stat,sys\nlab=pathlib.Path(sys.argv[1]);guard=int(sys.argv[2]);server=int(sys.argv[3])\nif guard<=0 or server<=0 or guard==server:raise ValueError('owned_context_invalid')\ninfo=lab.lstat()\nif not (stat.S_ISDIR(info.st_mode) and stat.S_IMODE(info.st_mode)==0o700 and info.st_uid==os.getuid()):raise ValueError('owned_context_invalid')\nos.kill(guard,0);os.kill(server,0)\nif (lab/'stop').exists() or (lab/'ui-failure').exists():raise ValueError('owned_context_invalid')\nfd=os.open(lab/'browser-prepared',os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\nos.close(fd)\nprint('{\"prepared_marker_written\":true}')\n";
const PERSIST="import json,os,pathlib,sys\npath=pathlib.Path(sys.argv[1])\nrecord=json.loads(sys.argv[2])\n# Convert decimal strings directly to Python integers, avoiding JS Number rounding.\ndef clocks(value):\n    if isinstance(value,dict):\n        for k,v in list(value.items()):\n            if k in ('monotonic_ns','wall_epoch_ns') and isinstance(v,str):\n                assert v.isdecimal() and len(v)<=19 and 0<=int(v)<2**63\n                value[k]=int(v)\n            else:clocks(v)\n    elif isinstance(value,list):\n        for v in value:clocks(v)\nclocks(record)\nfd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\nwith os.fdopen(fd,'w',encoding='ascii') as f:\n    f.write(json.dumps(record,sort_keys=True,indent=2)+'\\n');f.flush();os.fsync(f.fileno())\nprint(json.dumps({'written_exclusive':True}))\n";
const own=load("d01_immediate_owned_handles");
const OBSKEY="d01_immediate_observation_record";
if (!own || own.runtime_release!==true || own.driver_owned!==true ||
    own.exact_bound!==true || own.single_controller!==true ||
    !["prepared_entry","browser_decision"].includes(own.phase) ||
    (own.phase==="prepared_entry"&&(own.prepared_gate!==true||own.helper_pid!==null||
      own.fixture_ready===true||own.listener_pid_proof===true)) ||
    (own.phase==="browser_decision"&&(own.fixture_ready!==true||own.listener_pid_proof!==true)) ||
    own.fresh_paths_preflight!==true ||
    !Number.isSafeInteger(own.start_epoch_ms) || typeof own.workspace!=="string" ||
    new Set([own.guard_pid,own.server_pid,own.browser_pid,
       ...(own.helper_pid===null?[]:[own.helper_pid])]).size!==(own.helper_pid===null?3:4) ||
    ![own.guard_pid,own.server_pid,own.browser_pid,own.exec_session,
       ...(own.helper_pid===null?[]:[own.helper_pid])]
       .every(v=>Number.isSafeInteger(v)&&v>0) ||
    ![own.session,own.target_id,own.tab_id,own.lab,own.outer_out,
       own.event_out,own.cleanup_out].every(v=>typeof v==="string"&&v.length>0)) {
  text({proposal_refused:"fresh_owned_context_required"});exit();
}
const prior=load(OBSKEY);
if(own.closed===true||prior?.cleanup_started===true){
  text({proposal_refused:"fixture_already_stopped",resource_release_proven:false});exit();
}
const record=prior??{
  schema:"riauth.d01-immediate-observation-cleanup/v1",
  first_failure:null,first_observation_wall_ms:null,
  first_event_proven:false,whole_cleanup_within60_proven:false,
  unexpected_failure_observation:null,diagnostic_errors:[],cleanup_started:false,
  first_clock:null,observation_receipts:[],controller_observations:[],local_tool_receipts:[],actions:[],cleanup_errors:[],
  final_absence:null,owned_child_exits:null,controller_exit:null,
  first_observation_to_final_wall_ms:null,latch_to_absence_ms:null
};
const sq=s=>"'"+String(s).replace(/'/g,"'\\''")+"'";
const retain=()=>store(OBSKEY,record);
const cleanupError=label=>{
  if(!record.cleanup_errors.includes(label))record.cleanup_errors.push(label);
  retain();
};
const local=async(source,arg)=>{
  const cmd="python3 -c "+sq(source)+(arg===undefined?"":" "+sq(JSON.stringify(arg)));
  const r=await tools.exec_command({cmd,workdir:own.workspace,
    yield_time_ms:10000,max_output_tokens:2000});
  record.local_tool_receipts.push({
    label:source===CLOCK?"clock":source===STOP?"stop":source===READBACK?"readback":"unknown",
    exit:typeof r.exit_code==="number"?r.exit_code:null,
    session_id:typeof r.session_id==="number"?r.session_id:null
  });retain(); // Numeric collector receipt BEFORE comparisons.
  if(r.session_id!==undefined) {
    cleanupError("owned_local_command_unjoined");throw new Error("local_pending");
  }
  if(r.exit_code!==0)throw new Error("local_failed");
  return JSON.parse(r.output);
};
function finiteObservation(value) {
  const exact=(v,keys)=>v!==null&&typeof v==="object"&&!Array.isArray(v)&&
    Object.keys(v).length===keys.length&&keys.every(k=>Object.hasOwn(v,k));
  if(!exact(value,["observed","diagnostic"])||typeof value.observed!=="boolean")return null;
  const d=value.diagnostic;
  if(d===null)return {observed:value.observed,diagnostic:null};
  if(!value.observed||!exact(d,["site","exception_class","own_function","own_line"])||
     !["handler","server","main"].includes(d.site)||
     !["AttributeError","TypeError","ValueError","KeyError","OSError","BrokenPipeError",
       "ConnectionResetError","TimeoutError","other"].includes(d.exception_class))return null;
  const functions=["HeaderReader.readline","DemoServer.process_request","Demo.begin","Demo.callback",
    "Demo.invoke","Handler.handle_one_request","Handler.send_error","Handler.get","Handler.reply","main"];
  if(!((d.own_function===null&&d.own_line===null)||
       (functions.includes(d.own_function)&&Number.isSafeInteger(d.own_line)&&
        d.own_line>=1&&d.own_line<=1024)))return null;
  return {observed:true,diagnostic:{site:d.site,exception_class:d.exception_class,
    own_function:d.own_function,own_line:d.own_line}};
}
function diagnostic(value,projection) {
  const projected=finiteObservation(value);
  if(projection==="invalid"||!["valid","unavailable"].includes(projection)||
     (projection==="valid"&&projected===null)) {
    if(!record.diagnostic_errors.includes("observer_projection_invalid"))
      record.diagnostic_errors.push("observer_projection_invalid");
  }
  if(projected!==null&&record.unexpected_failure_observation===null)
    record.unexpected_failure_observation=projected;
  retain(); // No raw diagnostic fallback and no first-failure or outcome mutation.
}
const ns=c=>BigInt(c.monotonic_ns);
const getClock=()=>local(CLOCK);
let controllerJoined=false;
let controllerPollAvailable=true;
let controllerBuffer="";
let cleanupStarted=false;
let lastSnapshotFlags=null;
function latch(label,receivedWall) {
  if(record.first_failure===null) {
    record.first_failure=label;
    record.first_observation_wall_ms=receivedWall;
    retain(); // Synchronous first-failure/wall latch BEFORE any new await/output.
  }
}
function observeController(r,receivedWall) {
  const observation={received_wall_ms:receivedWall,
    exit:typeof r.exit_code==="number"?r.exit_code:null,
    helper_completed:false,helper_exit:null,fixture_finished:false};
  // Complete numeric result projection is retained BEFORE comparisons.
  record.controller_observations.push(observation);retain();
  if(typeof r.exit_code==="number") {
    controllerJoined=true;record.controller_exit=r.exit_code;retain();
  }
  controllerBuffer+=typeof r.output==="string"?r.output:"";
  if(controllerBuffer.length>16384) {
    latch("controller_observation_invalid",receivedWall);controllerBuffer="";return;
  }
  let cut;
  while((cut=controllerBuffer.indexOf("\n"))>=0) {
    const line=controllerBuffer.slice(0,cut);controllerBuffer=controllerBuffer.slice(cut+1);
    if(!line.trim())continue;
    let event;
    try{event=JSON.parse(line);}catch{
      latch("controller_observation_invalid",receivedWall);continue;
    }
    if(Object.hasOwn(event,"unexpected_failure_observation"))
      diagnostic(event.unexpected_failure_observation,event.unexpected_observation_projection);
    if(event.fixture_ready===true) {
      if(event.guard_pid!==own.guard_pid||event.server_pid!==own.server_pid||
         event.lab!==own.lab||!Number.isSafeInteger(event.helper_pid)||event.helper_pid<=0||
         [own.guard_pid,own.server_pid,own.browser_pid].includes(event.helper_pid)||
         (own.helper_pid!==null&&own.helper_pid!==event.helper_pid))
        latch("fixture_ready_unconfirmed",receivedWall);
      else {
        own.helper_pid=event.helper_pid;own.fixture_ready=true;own.listener_pid_proof=true;
        store("d01_immediate_owned_handles",own);
      }
    }
    if(event.helper_completed===true) {
      observation.helper_completed=true;
      observation.helper_exit=typeof event.exit==="number"?event.exit:null;
      retain();
      if(event.exit!==0)latch("helper_failed",receivedWall);
      else if(!cleanupStarted&&record.protected_after_page!==true)
        latch("helper_completed_before_app_checkpoint",receivedWall);
    }
    if(event.fixture_finished===true) {
      observation.fixture_finished=true;retain();
      if(!cleanupStarted)latch("controller_completed_before_app_checkpoint",receivedWall);
    }
  }
  if(controllerJoined&&!cleanupStarted)
    latch("controller_completed_before_app_checkpoint",receivedWall);
}
async function pollController() {
  if(controllerJoined||!controllerPollAvailable)return;
  try {
    const r=await tools.write_stdin({session_id:own.exec_session,
      chars:"",yield_time_ms:5000,max_output_tokens:2000});
    const receivedWall=Date.now();
    observeController(r,receivedWall);
  } catch {
    controllerPollAvailable=false;
    latch("controller_observation_unavailable",Date.now());
    if(cleanupStarted)cleanupError("controller_join_unconfirmed");
  }
}
async function timedDriver(label,allocation,operation,project) {
  let start=null,end=null,result=null,state="unknown";
  try{start=await getClock();}catch{cleanupError("clock_unavailable");}
  const action={label,start_clock:start,end_clock:null,allowed_ms:allocation,
    result_state:"unknown",elapsed_ms:null,over_budget:null};
  record.actions.push(action);retain(); // Start retained BEFORE entering Driver call.
  try {
    result=await operation();
    state=result?.isError===true?"refused":"returned";
  } catch {state="exception";}
  try{end=await getClock();}catch{cleanupError("clock_unavailable");}
  action.end_clock=end;action.result_state=state;
  retain(); // End/result retained BEFORE budget comparison.
  if(start&&end) {
    const d=ns(end)-ns(start);
    if(d>=0n) {
      action.elapsed_ms=Number(d/1000000n);
      action.over_budget=action.elapsed_ms>allocation;
    } else cleanupError("clock_invalid");
  }
  if(action.over_budget)cleanupError("driver_operation_over_budget");
  if(state!=="returned")cleanupError("driver_operation_unconfirmed");
  if(project)project(result,state);
  retain();
}
async function cleanup() {
  if(cleanupStarted)return;
  cleanupStarted=true;record.cleanup_started=true;retain();
  try {
    const r=await local(STOP,{
      lab:own.lab,event_out:own.event_out,
      first_failure:record.first_failure,
      first_observation_wall_ms:record.first_observation_wall_ms
    });
    record.first_clock=r.clock;record.stop_state=r.marker_state;retain();
    if(r.event_state!=="written")cleanupError("observation_metadata_write_unconfirmed");
    if(r.marker_state==="stop_unconfirmed")cleanupError("stop_unconfirmed");
    if(r.marker_state==="ownership_unknown")cleanupError("ownership_unknown");
  } catch {cleanupError("stop_or_clock_record_unavailable");}
  // No outstanding Driver call exists here: all page calls above were awaited.
  // Original stop protocol already causes the controller's owned-child finally.
  await timedDriver("kill_app",30000,
    ()=>tools.mcp__cua_driver__kill_app({pid:own.browser_pid}));
  await timedDriver("end_session",15000,
    ()=>tools.mcp__cua_driver__end_session({session:own.session}),(r,state)=>{
      record.session_ended=state==="returned"&&
        r?.structuredContent?.active===false&&r.structuredContent.session===own.session;
      retain();
    });
  await timedDriver("list_windows",5000,
    ()=>tools.mcp__cua_driver__list_windows({pid:own.browser_pid}),(r,state)=>{
      const windows=r?.structuredContent?.windows;
      record.window_count=state==="returned"&&Array.isArray(windows)?windows.length:null;
      retain();
    });
  let readStart=null;
  try{readStart=await getClock();}catch{cleanupError("clock_unavailable");}
  const action={label:"owned_join_readback",start_clock:readStart,end_clock:null,
    allowed_ms:null,elapsed_ms:null,over_budget:null,result_state:"unknown"};
  record.actions.push(action);retain();
  if(readStart&&record.first_clock) {
    action.allowed_ms=Math.max(0,60000-Number((ns(readStart)-ns(record.first_clock))/1000000n));
    retain();
  }
  // Continue essential join after60s if late; never claim that deadline enforced.
  // Do not poll another exec session or send a manual process signal.
  while(!controllerJoined&&controllerPollAvailable&&Date.now()<own.start_epoch_ms+900000) {
    await pollController();
    if(record.first_clock) {
      try {
        const c=await getClock();record.last_join_clock=c;retain();
        if(ns(c)-ns(record.first_clock)>60000000000n)
          cleanupError("cleanup_budget_exceeded");
      } catch{cleanupError("clock_unavailable");}
    }
  }
  if(!controllerJoined)cleanupError("child_reap_incomplete");
  try {
    const r=await local(READBACK,{
      owned_pids:[own.guard_pid,own.server_pid,own.browser_pid,
        ...(own.helper_pid===null?[]:[own.helper_pid])],
      lab:own.lab,outer_out:own.outer_out,helper_pid:own.helper_pid,server_pid:own.server_pid
    });
    action.end_clock=r.clock;action.result_state="returned";
    record.owned_child_exits=r.owned_child_exits;
    diagnostic(r.unexpected_failure_observation,r.unexpected_observation_projection);
    if(Number.isSafeInteger(r.helper_pid)&&r.helper_pid>0)own.helper_pid=r.helper_pid;
    record.final_absence={owned_pids:r.owned_pids,ports:r.ports,
      lab:r.lab_absent,window_count:record.window_count??null,
      session_ended:record.session_ended===true};
    record.first_observation_to_final_wall_ms=record.first_observation_wall_ms===null?null:
      Date.now()-record.first_observation_wall_ms;
    retain(); // Full fixed readback/exits/clock BEFORE comparisons.
    if(readStart) {
      action.elapsed_ms=Number((ns(r.clock)-ns(readStart))/1000000n);
      action.over_budget=action.allowed_ms===null?null:action.elapsed_ms>action.allowed_ms;
    }
    if(record.first_clock)
      record.latch_to_absence_ms=Number((ns(r.clock)-ns(record.first_clock))/1000000n);
    if(action.over_budget)cleanupError("cleanup_budget_exceeded");
    const a=record.final_absence;
    if(!controllerJoined||!Array.isArray(record.owned_child_exits)||
       record.owned_child_exits.length!==(own.helper_pid===null?6:7))
      cleanupError("child_reap_incomplete");
    if(!Object.values(a.owned_pids).every(v=>v===true)||
       !Object.values(a.ports).every(v=>v===true)||a.lab!==true||
       a.window_count!==0||a.session_ended!==true)
      cleanupError("owned_resource_absence_unproven");
  } catch {action.result_state="exception";cleanupError("owned_readback_unavailable");}
  cleanupError("first_event_unproven");
  record.whole_cleanup_within60_proven=false;retain();
  // Exclusive new file only; existing outer/helper/provider evidence untouched.
  try {
    const cmd="python3 -c "+sq(PERSIST)+" "+sq(own.cleanup_out)+" "+sq(JSON.stringify(record));
    const r=await tools.exec_command({cmd,workdir:own.workspace,
      yield_time_ms:10000,max_output_tokens:500});
    record.local_tool_receipts.push({label:"persist",
      exit:typeof r.exit_code==="number"?r.exit_code:null,
      session_id:typeof r.session_id==="number"?r.session_id:null});retain();
    if(r.session_id!==undefined)cleanupError("owned_local_command_unjoined");
    if(r.session_id!==undefined||r.exit_code!==0)
      cleanupError("cleanup_metadata_write_unconfirmed");
  } catch {cleanupError("cleanup_metadata_write_unconfirmed");}
  controllerBuffer="";own.closed=true;store("d01_immediate_owned_handles",own);
}
async function checked(label,operation,predicate) {
  if(record.first_failure!==null)return null;
  if(Date.now()>=own.start_epoch_ms+840000) {
    latch("browser_active_deadline",Date.now());await cleanup();return null;
  }
  let result,receivedWall;
  try {
    result=await operation();receivedWall=Date.now();
    record.observation_receipts.push({kind:label,received_wall_ms:receivedWall});retain();
    if(result?.isError===true)latch("browser_tool_refused",receivedWall);
    else if(!predicate(result))latch("browser_decision_unconfirmed",receivedWall);
  } catch {latch("browser_tool_or_decision_exception",Date.now());}
  if(record.first_failure!==null)await cleanup();
  return record.first_failure===null?result:null;
}
const snapshotArgs={session:own.session,target_id:own.target_id,tab_id:own.tab_id,
  snapshot_format:"semantic_v2",include_screenshot:false};
function page(result,kind) {
  const s=result?.structuredContent,nodes=Array.isArray(s?.content_refs)?s.content_refs:[];
  const flags={
    status_ok:result?.isError!==true&&s?.status==="ok",
    complete:s?.snapshot?.complete===true,
    heading_match:nodes.some(n=>n.role==="heading"&&n.name===
      (kind==="protected_after"?"Protected application access":"Local demo")),
    error_match:nodes.some(n=>n.name==="Local demo could not complete this request."),
    required_text:nodes.some(n=>n.name===(kind==="protected_before"?"Sign in required.":
      kind==="protected_after"?"Signed in. Protected application access is available.":
      "Use Sign in to open this local application.")),
    interactive_ref_present:Array.isArray(s?.refs)&&
      s.refs.some(n=>Array.isArray(n.actions)&&n.actions.includes("click"))
  };
  record.last_page_flags={kind,...flags};retain();
  // Only provider refs and fixed public-label matches survive this raw snapshot.
  own.fresh_refs=Array.isArray(s?.refs)?s.refs.filter(n=>typeof n.ref==="string"&&
    /^p[0-9]+:[0-9]+$/.test(n.ref)).map(n=>n.ref):[];
  store("d01_immediate_owned_handles",own);
  if(flags.error_match)return false;
  if(!flags.status_ok||!flags.complete)return false;
  if(kind==="generic_checked") {
    if(nodes.some(n=>n.role==="heading"&&n.name==="Protected application access")&&
       nodes.some(n=>n.name==="Signed in. Protected application access is available."))
      record.protected_after_page=true;
    return true;
  }
  const ok=flags.heading_match&&flags.required_text&&
    (kind!=="application"||flags.interactive_ref_present);
  if(ok&&kind==="protected_after")record.protected_after_page=true;
  return ok;
}
async function navigateAndSnapshot(url,kind) {
  if(await checked("browser_navigation",
      ()=>tools.mcp__cua_driver__browser_navigate({session:own.session,
        target_id:own.target_id,tab_id:own.tab_id,url}),
      r=>r?.isError!==true)) {
    await checked("browser_snapshot",
      ()=>tools.mcp__cua_driver__get_browser_state(snapshotArgs),r=>page(r,kind));
  }
}
async function entry() {
  // The exact Driver-owned blank handles already exist while the180s gate waits.
  // There is no model yield, tool-description load or rebind after this marker.
  const markerCommand="python3 -c "+sq(MARKER)+" "+sq(own.lab)+" "+
    sq(own.guard_pid)+" "+sq(own.server_pid);
  await checked("prepared_marker",
    ()=>tools.exec_command({cmd:markerCommand,workdir:own.workspace,
      yield_time_ms:10000,max_output_tokens:500}),r=>{
      record.local_tool_receipts.push({label:"marker",
        exit:typeof r.exit_code==="number"?r.exit_code:null,
        session_id:typeof r.session_id==="number"?r.session_id:null});retain();
      if(r.session_id!==undefined)cleanupError("owned_local_command_unjoined");
      return r.exit_code===0&&r.session_id===undefined&&
        JSON.parse(r.output).prepared_marker_written===true;
    });
  const readyDeadline=Date.now()+30000;
  while(record.first_failure===null&&own.fixture_ready!==true&&Date.now()<readyDeadline)
    await pollController();
  if(record.first_failure===null&&own.fixture_ready!==true)
    latch("fixture_ready_unconfirmed",Date.now());
  if(record.first_failure!==null){await cleanup();return;}
  await pollController(); // ONE immediate pre-navigation poll, no RP HTTP probe.
  if(record.first_failure!==null){await cleanup();return;}
  await navigateAndSnapshot("http://localhost:3000/protected","protected_before");
  if(record.first_failure===null)await pollController();
  if(record.first_failure!==null){await cleanup();return;}
  await navigateAndSnapshot("http://localhost:3000/","application");
  if(record.first_failure===null)await pollController(); // ONE post-entry poll.
  if(record.first_failure!==null)await cleanup();
}
async function continuation() {
  const spec=own.next_decision;
  const allowedKinds=["click","username","password","snapshot","protected_after"];
  if(!spec||!allowedKinds.includes(spec.kind)) {
    latch("browser_decision_unconfirmed",Date.now());await cleanup();return;
  }
  await pollController();
  if(record.first_failure!==null){await cleanup();return;}
  if(spec.kind==="protected_after") {
    // Read the fresh callback-following protected page; do not send another RP request.
    await checked("browser_snapshot",
      ()=>tools.mcp__cua_driver__get_browser_state(snapshotArgs),r=>page(r,"protected_after"));
  } else if(spec.kind==="snapshot") {
    await checked("browser_snapshot",
      ()=>tools.mcp__cua_driver__get_browser_state(snapshotArgs),
      r=>page(r,"generic_checked")&&publicPredicate(r,spec));
  } else {
    if(typeof spec.ref!=="string"||!Array.isArray(own.fresh_refs)||
       !own.fresh_refs.includes(spec.ref)) {
      latch("browser_decision_unconfirmed",Date.now());await cleanup();return;
    }
    own.fresh_refs=[];store("d01_immediate_owned_handles",own); // Single-use snapshot ref.
    const args={session:own.session,target_id:own.target_id,tab_id:own.tab_id,ref:spec.ref};
    const operation=spec.kind==="click"?
      ()=>tools.mcp__cua_driver__browser_click({...args,input_route:"dom_event"}):
      ()=>tools.mcp__cua_driver__browser_type({...args,replace:true,
        text:spec.kind==="username"?"admin":load("d01_fresh_password_input")});
    if(spec.kind==="password"&&(typeof load("d01_fresh_password_input")!=="string"||
       !/^[A-Za-z0-9_-]{30,128}$/.test(load("d01_fresh_password_input")))) {
      latch("browser_decision_unconfirmed",Date.now());await cleanup();return;
    }
    await checked("browser_input",operation,r=>{
      const s=r?.structuredContent;
      return r?.isError!==true&&["confirmed","unverifiable"].includes(s?.effect);
    }); // Dispatch alone never earns an application outcome or journey credit.
    store("d01_fresh_password_input",null);
    if(record.first_failure===null)
      await checked("browser_snapshot",
        ()=>tools.mcp__cua_driver__get_browser_state(snapshotArgs),
        r=>page(r,"generic_checked")&&publicPredicate(r,spec));
  }
  if(record.first_failure===null)await pollController();
  if(record.first_failure!==null)await cleanup();
  else if(spec.kind==="protected_after") {
    record.cleanup_requested_wall_ms=Date.now();retain();
    await cleanup();
  }
}
function publicPredicate(result,spec) {
  const nodes=result?.structuredContent?.content_refs;
  // Root must pin one printed public expected label/role before that action.
  // No raw field value, URL, subject, query or arbitrary regex predicate is allowed.
  const roles=["heading","button","statictext","textbox"];
  const labels=["Username","Password","Authenticator or recovery code","Sign in",
    "Local demo","Protected application access",
    "Signed in. Protected application access is available."];
  return Array.isArray(nodes)&&roles.includes(spec.expected_role)&&
    labels.includes(spec.expected_label)&&
    nodes.some(n=>n.role===spec.expected_role&&n.name===spec.expected_label);
}
try {
  if(own.phase==="prepared_entry")await entry();else await continuation();
} catch {
  latch("composed_decision_exception",Date.now());
  await cleanup();
}
if(record.first_failure!==null) {
  text({result:"failed",first_failure:record.first_failure,
    unexpected_failure_observation:record.unexpected_failure_observation,
    diagnostic_errors:record.diagnostic_errors,cleanup_errors:record.cleanup_errors,
    final_absence:record.final_absence,controller_exit:record.controller_exit,
    whole_cleanup_within60_proven:false,
    resource_release_proven:record.final_absence!==null&&
      !record.cleanup_errors.some(v=>[
        "child_reap_incomplete","owned_resource_absence_unproven",
        "owned_readback_unavailable","owned_local_command_unjoined"].includes(v))});
} else {
  text({result:"browser_decision_observed_only",page_flags:record.last_page_flags??null,
    journey_credit:false,cleanup_errors:record.cleanup_errors,
    resource_release_proven:record.cleanup_started===true&&record.final_absence!==null&&
      !record.cleanup_errors.some(v=>[
        "child_reap_incomplete","owned_resource_absence_unproven",
        "owned_readback_unavailable","owned_local_command_unjoined"].includes(v)),
    whole_cleanup_within60_proven:false});
}
```

Exact zero-context cell delta from0a7cea:

```diff
--- retained-composed-cell-0a7cea
+++ DESIGN-only-observer-entry-cell
@@ -3,2 +3,3 @@
-const STOP="import json,os,pathlib,stat,sys,time\nc=json.loads(sys.argv[1])\nclock={'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())}\nrecord={'schema':'riauth.d01-first-observation/v1','first_failure':c['first_failure'],'first_observation_wall_ms':c['first_observation_wall_ms'],'first_event_proven':False,'clock':clock}\nevent_state='write_unconfirmed'\ntry:\n    fd=os.open(pathlib.Path(c['event_out']),os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\n    with os.fdopen(fd,'w',encoding='ascii') as f:\n        f.write(json.dumps(record,sort_keys=True)+'\\n');f.flush();os.fsync(f.fileno())\n    event_state='written'\nexcept OSError:pass\n# Attempt clock persistence BEFORE marker/state/budget comparisons.\n# A metadata error does not prevent the original essential stop protocol.\nlab=pathlib.Path(c['lab']);marker_state='lab_absent'\ntry:\n    if lab.exists():\n        info=lab.lstat()\n        if not stat.S_ISDIR(info.st_mode) or stat.S_IMODE(info.st_mode)!=0o700 or info.st_uid!=os.getuid():\n            marker_state='ownership_unknown'\n        else:\n            marker_state='stop_requested'\n            for name in ('ui-failure','stop'):\n                try:\n                    fd=os.open(lab/name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\n                    os.close(fd)\n                except FileNotFoundError:marker_state='lab_absent'\n                except FileExistsError:pass\nexcept OSError:marker_state='stop_unconfirmed'\nprint(json.dumps({'clock':clock,'event_state':event_state,'marker_state':marker_state}))\n";
-const READBACK="import json,pathlib,subprocess,sys,time\nc=json.loads(sys.argv[1])\npids=c['owned_pids']\np=subprocess.run(['/bin/ps','-p',','.join(str(v) for v in pids),'-o','pid='],capture_output=True,timeout=3)\nps_known=p.returncode in (0,1) and all(v.isdigit() for v in p.stdout.split())\npresent=set(int(v) for v in p.stdout.split()) if ps_known else set()\nports={}\nfor port in (9000,3000):\n    p=subprocess.run(['/usr/sbin/lsof','-nP','-t','-iTCP:'+str(port),'-sTCP:LISTEN'],capture_output=True,timeout=3)\n    ports[str(port)]=not bool(p.stdout.strip()) if p.returncode in (0,1) else None\nchildren=None\nouter=pathlib.Path(c['outer_out'])\nif outer.is_file():\n    # This is the unchanged controller's fixed redacted evidence, not app data.\n    data=json.loads(outer.read_bytes())\n    allowed={'helper','whoami','discovery','confidential_client_create','operator_login','server','maintenance_init'}\n    raw=data.get('owned_child_exits')\n    if isinstance(raw,list) and len(raw)==7 and all(isinstance(v,dict) and v.get('name') in allowed and type(v.get('pid')) is int and type(v.get('exit')) is int for v in raw) and {v['name'] for v in raw}==allowed and len({v['pid'] for v in raw})==7 and next(v['pid'] for v in raw if v['name']=='helper')==c['helper_pid'] and next(v['pid'] for v in raw if v['name']=='server')==c['server_pid']:\n        children=[{k:v[k] for k in ('name','pid','exit')} for v in raw]\nprint(json.dumps({'clock':{'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())},'owned_pids':{str(v):(v not in present if ps_known else None) for v in pids},'ports':ports,'lab_absent':not pathlib.Path(c['lab']).exists(),'owned_child_exits':children}))\n";
+const STOP="import json,os,pathlib,stat,sys,time\nc=json.loads(sys.argv[1])\nclock={'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())}\nrecord={'schema':'riauth.d01-first-observation/v1','first_failure':c['first_failure'],'first_observation_wall_ms':c['first_observation_wall_ms'],'first_event_proven':False,'clock':clock}\nevent_state='write_unconfirmed'\ntry:\n    fd=os.open(pathlib.Path(c['event_out']),os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\n    with os.fdopen(fd,'w',encoding='ascii') as f:\n        f.write(json.dumps(record,sort_keys=True)+'\\n');f.flush();os.fsync(f.fileno())\n    event_state='written'\nexcept OSError:pass\n# Attempt clock persistence BEFORE marker/state/budget comparisons.\n# A metadata error does not prevent the original essential stop protocol.\nlab=pathlib.Path(c['lab']);marker_state='lab_absent'\ntry:\n    if lab.exists():\n        info=lab.lstat()\n        if not stat.S_ISDIR(info.st_mode) or stat.S_IMODE(info.st_mode)!=0o700 or info.st_uid!=os.getuid():\n            marker_state='ownership_unknown'\n        else:\n            marker_state='stop_requested'\n            for name in (('ui-failure','stop') if c['first_failure'] is not None else ('stop',)):\n                try:\n                    fd=os.open(lab/name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\n                    os.close(fd)\n                except FileNotFoundError:marker_state='lab_absent'\n                except FileExistsError:pass\nexcept OSError:marker_state='stop_unconfirmed'\nprint(json.dumps({'clock':clock,'event_state':event_state,'marker_state':marker_state}))\n";
+const READBACK="import json,os,pathlib,stat,subprocess,sys,time\nc=json.loads(sys.argv[1])\nchildren=None;helper_pid=c['helper_pid'];outer_observation=None;projection='unavailable'\ndef finite_observation(value):\n    if type(value) is not dict or set(value)!= {'observed','diagnostic'} or type(value['observed']) is not bool:\n        return None\n    diagnostic=value['diagnostic']\n    if diagnostic is None:\n        return {'observed':value['observed'],'diagnostic':None}\n    if not value['observed'] or type(diagnostic) is not dict or set(diagnostic)!= {'site','exception_class','own_function','own_line'}:\n        return None\n    sites=('handler','server','main')\n    kinds=('AttributeError','TypeError','ValueError','KeyError','OSError','BrokenPipeError','ConnectionResetError','TimeoutError','other')\n    functions=('HeaderReader.readline','DemoServer.process_request','Demo.begin','Demo.callback','Demo.invoke','Handler.handle_one_request','Handler.send_error','Handler.get','Handler.reply','main')\n    site,kind,function,line=(diagnostic[k] for k in ('site','exception_class','own_function','own_line'))\n    if type(site) is not str or site not in sites or type(kind) is not str or kind not in kinds:\n        return None\n    if not ((function is None and line is None) or (type(function) is str and function in functions and type(line) is int and 1<=line<=1024)):\n        return None\n    return {'observed':True,'diagnostic':{'site':site,'exception_class':kind,'own_function':function,'own_line':line}}\nouter=pathlib.Path(c['outer_out'])\nif outer.is_file():\n    fd=os.open(outer,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)\n    try:\n        info=os.fstat(fd)\n        if not (stat.S_ISREG(info.st_mode) and info.st_uid==os.getuid() and stat.S_IMODE(info.st_mode)==0o600 and 0<info.st_size<=262144):\n            raise ValueError('fixed_outer_evidence_invalid')\n        with os.fdopen(fd,'rb',closefd=False) as source:raw_bytes=source.read(262145)\n        if not 0<len(raw_bytes)<=262144:raise ValueError('fixed_outer_evidence_invalid')\n        data=json.loads(raw_bytes)\n    finally:os.close(fd)\n    outer_observation=finite_observation(data.get('unexpected_failure_observation'))\n    projection=data.get('unexpected_observation_projection')\n    if projection not in ('unavailable','valid','invalid'):projection='invalid'\n    if projection=='valid' and outer_observation is None:projection='invalid'\n    allowed={'whoami','discovery','confidential_client_create','operator_login','server','maintenance_init'}\n    started=data.get('helper_invocations')==1 and type(data.get('helper_pid')) is int and data['helper_pid']>0\n    if started:allowed.add('helper')\n    raw=data.get('owned_child_exits')\n    if isinstance(raw,list) and len(raw)==len(allowed) and all(isinstance(v,dict) and v.get('name') in allowed and type(v.get('pid')) is int and v['pid']>0 and type(v.get('exit')) is int for v in raw) and {v['name'] for v in raw}==allowed and len({v['pid'] for v in raw})==len(allowed) and next(v['pid'] for v in raw if v['name']=='server')==c['server_pid'] and ((started and next(v['pid'] for v in raw if v['name']=='helper')==data['helper_pid'] and (helper_pid is None or helper_pid==data['helper_pid'])) or (not started and helper_pid is None and data.get('helper_invocations') is None and data.get('helper_pid') is None)):\n        children=[{k:v[k] for k in ('name','pid','exit')} for v in raw]\n        helper_pid=data['helper_pid'] if started else None\npids=list(c['owned_pids'])\nif helper_pid is not None and helper_pid not in pids:pids.append(helper_pid)\np=subprocess.run(['/bin/ps','-p',','.join(str(v) for v in pids),'-o','pid='],capture_output=True,timeout=3)\nps_known=p.returncode in (0,1) and all(v.isdigit() for v in p.stdout.split())\npresent=set(int(v) for v in p.stdout.split()) if ps_known else set()\nports={}\nfor port in (9000,3000):\n    p=subprocess.run(['/usr/sbin/lsof','-nP','-t','-iTCP:'+str(port),'-sTCP:LISTEN'],capture_output=True,timeout=3)\n    ports[str(port)]=not bool(p.stdout.strip()) if p.returncode in (0,1) else None\nprint(json.dumps({'clock':{'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())},'owned_pids':{str(v):(v not in present if ps_known else None) for v in pids},'ports':ports,'lab_absent':not pathlib.Path(c['lab']).exists(),'owned_child_exits':children,'helper_pid':helper_pid,'unexpected_failure_observation':outer_observation,'unexpected_observation_projection':projection}))\n";
+const MARKER="import os,pathlib,stat,sys\nlab=pathlib.Path(sys.argv[1]);guard=int(sys.argv[2]);server=int(sys.argv[3])\nif guard<=0 or server<=0 or guard==server:raise ValueError('owned_context_invalid')\ninfo=lab.lstat()\nif not (stat.S_ISDIR(info.st_mode) and stat.S_IMODE(info.st_mode)==0o700 and info.st_uid==os.getuid()):raise ValueError('owned_context_invalid')\nos.kill(guard,0);os.kill(server,0)\nif (lab/'stop').exists() or (lab/'ui-failure').exists():raise ValueError('owned_context_invalid')\nfd=os.open(lab/'browser-prepared',os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\nos.close(fd)\nprint('{\"prepared_marker_written\":true}')\n";
@@ -10 +11,4 @@
-    own.fixture_ready!==true || own.listener_pid_proof!==true ||
+    !["prepared_entry","browser_decision"].includes(own.phase) ||
+    (own.phase==="prepared_entry"&&(own.prepared_gate!==true||own.helper_pid!==null||
+      own.fixture_ready===true||own.listener_pid_proof===true)) ||
+    (own.phase==="browser_decision"&&(own.fixture_ready!==true||own.listener_pid_proof!==true)) ||
@@ -13,2 +17,4 @@
-    new Set([own.guard_pid,own.server_pid,own.helper_pid,own.browser_pid]).size!==4 ||
-    ![own.guard_pid,own.server_pid,own.helper_pid,own.browser_pid,own.exec_session]
+    new Set([own.guard_pid,own.server_pid,own.browser_pid,
+       ...(own.helper_pid===null?[]:[own.helper_pid])]).size!==(own.helper_pid===null?3:4) ||
+    ![own.guard_pid,own.server_pid,own.browser_pid,own.exec_session,
+       ...(own.helper_pid===null?[]:[own.helper_pid])]
@@ -20 +26,5 @@
-const record={
+const prior=load(OBSKEY);
+if(own.closed===true||prior?.cleanup_started===true){
+  text({proposal_refused:"fixture_already_stopped",resource_release_proven:false});exit();
+}
+const record=prior??{
@@ -23,0 +34 @@
+  unexpected_failure_observation:null,diagnostic_errors:[],cleanup_started:false,
@@ -48,0 +60,29 @@
+function finiteObservation(value) {
+  const exact=(v,keys)=>v!==null&&typeof v==="object"&&!Array.isArray(v)&&
+    Object.keys(v).length===keys.length&&keys.every(k=>Object.hasOwn(v,k));
+  if(!exact(value,["observed","diagnostic"])||typeof value.observed!=="boolean")return null;
+  const d=value.diagnostic;
+  if(d===null)return {observed:value.observed,diagnostic:null};
+  if(!value.observed||!exact(d,["site","exception_class","own_function","own_line"])||
+     !["handler","server","main"].includes(d.site)||
+     !["AttributeError","TypeError","ValueError","KeyError","OSError","BrokenPipeError",
+       "ConnectionResetError","TimeoutError","other"].includes(d.exception_class))return null;
+  const functions=["HeaderReader.readline","DemoServer.process_request","Demo.begin","Demo.callback",
+    "Demo.invoke","Handler.handle_one_request","Handler.send_error","Handler.get","Handler.reply","main"];
+  if(!((d.own_function===null&&d.own_line===null)||
+       (functions.includes(d.own_function)&&Number.isSafeInteger(d.own_line)&&
+        d.own_line>=1&&d.own_line<=1024)))return null;
+  return {observed:true,diagnostic:{site:d.site,exception_class:d.exception_class,
+    own_function:d.own_function,own_line:d.own_line}};
+}
+function diagnostic(value,projection) {
+  const projected=finiteObservation(value);
+  if(projection==="invalid"||!["valid","unavailable"].includes(projection)||
+     (projection==="valid"&&projected===null)) {
+    if(!record.diagnostic_errors.includes("observer_projection_invalid"))
+      record.diagnostic_errors.push("observer_projection_invalid");
+  }
+  if(projected!==null&&record.unexpected_failure_observation===null)
+    record.unexpected_failure_observation=projected;
+  retain(); // No raw diagnostic fallback and no first-failure or outcome mutation.
+}
@@ -83,0 +124,13 @@
+    if(Object.hasOwn(event,"unexpected_failure_observation"))
+      diagnostic(event.unexpected_failure_observation,event.unexpected_observation_projection);
+    if(event.fixture_ready===true) {
+      if(event.guard_pid!==own.guard_pid||event.server_pid!==own.server_pid||
+         event.lab!==own.lab||!Number.isSafeInteger(event.helper_pid)||event.helper_pid<=0||
+         [own.guard_pid,own.server_pid,own.browser_pid].includes(event.helper_pid)||
+         (own.helper_pid!==null&&own.helper_pid!==event.helper_pid))
+        latch("fixture_ready_unconfirmed",receivedWall);
+      else {
+        own.helper_pid=event.helper_pid;own.fixture_ready=true;own.listener_pid_proof=true;
+        store("d01_immediate_owned_handles",own);
+      }
+    }
@@ -89 +142,2 @@
-      else if(!cleanupStarted)latch("helper_completed_before_app_checkpoint",receivedWall);
+      else if(!cleanupStarted&&record.protected_after_page!==true)
+        latch("helper_completed_before_app_checkpoint",receivedWall);
@@ -139 +193 @@
-  cleanupStarted=true;
+  cleanupStarted=true;record.cleanup_started=true;retain();
@@ -191 +245,2 @@
-      owned_pids:[own.guard_pid,own.server_pid,own.helper_pid,own.browser_pid],
+      owned_pids:[own.guard_pid,own.server_pid,own.browser_pid,
+        ...(own.helper_pid===null?[]:[own.helper_pid])],
@@ -195,0 +251,2 @@
+    diagnostic(r.unexpected_failure_observation,r.unexpected_observation_projection);
+    if(Number.isSafeInteger(r.helper_pid)&&r.helper_pid>0)own.helper_pid=r.helper_pid;
@@ -199 +256,2 @@
-    record.first_observation_to_final_wall_ms=Date.now()-record.first_observation_wall_ms;
+    record.first_observation_to_final_wall_ms=record.first_observation_wall_ms===null?null:
+      Date.now()-record.first_observation_wall_ms;
@@ -210 +268,2 @@
-       record.owned_child_exits.length!==7)cleanupError("child_reap_incomplete");
+       record.owned_child_exits.length!==(own.helper_pid===null?6:7))
+      cleanupError("child_reap_incomplete");
@@ -230,42 +289,156 @@
-  controllerBuffer="";
-}
-await pollController();
-if(record.first_failure===null) {
-  try {
-    const r=await tools.mcp__cua_driver__browser_navigate({
-      session:own.session,target_id:own.target_id,tab_id:own.tab_id,
-      url:"http://localhost:3000/"});
-    const receivedWall=Date.now();
-    record.observation_receipts.push({kind:"browser_navigation",received_wall_ms:receivedWall});
-    retain(); // Received clock BEFORE navigation outcome comparison.
-    if(r?.isError===true)latch("driver_navigation_refused",receivedWall);
-  } catch {latch("driver_navigation_exception",Date.now());}
-}
-if(record.first_failure===null) {
-  try {
-    const r=await tools.mcp__cua_driver__get_browser_state({
-      session:own.session,target_id:own.target_id,tab_id:own.tab_id,
-      snapshot_format:"semantic_v2",include_screenshot:false});
-    const receivedWall=Date.now();
-    record.observation_receipts.push({kind:"browser_snapshot",received_wall_ms:receivedWall});
-    retain(); // Received clock BEFORE any page predicate comparison.
-    const s=r?.structuredContent;
-    const nodes=Array.isArray(s?.content_refs)?s.content_refs:[];
-    lastSnapshotFlags={
-      received_wall_ms:receivedWall,
-      status_ok:r?.isError!==true&&s?.status==="ok",
-      complete:s?.snapshot?.complete===true,
-      heading_match:nodes.some(n=>n.role==="heading"&&n.name==="Local demo"),
-      error_match:nodes.some(n=>n.name==="Local demo could not complete this request."),
-      interactive_ref_present:Array.isArray(s?.refs)&&
-        s.refs.some(n=>Array.isArray(n.actions)&&n.actions.includes("click"))
-    };
-    store("d01_immediate_snapshot_flags",lastSnapshotFlags);
-    // Finite booleans only; no outline/URL/name/value/subject/token is persisted.
-    if(lastSnapshotFlags.error_match)latch("application_page_failed",receivedWall);
-    else if(!lastSnapshotFlags.status_ok||!lastSnapshotFlags.complete||
-            !lastSnapshotFlags.heading_match||!lastSnapshotFlags.interactive_ref_present)
-      latch("application_snapshot_unconfirmed",receivedWall);
-  } catch {latch("driver_snapshot_exception",Date.now());}
-}
-if(record.first_failure===null)await pollController();
+  controllerBuffer="";own.closed=true;store("d01_immediate_owned_handles",own);
+}
+async function checked(label,operation,predicate) {
+  if(record.first_failure!==null)return null;
+  if(Date.now()>=own.start_epoch_ms+840000) {
+    latch("browser_active_deadline",Date.now());await cleanup();return null;
+  }
+  let result,receivedWall;
+  try {
+    result=await operation();receivedWall=Date.now();
+    record.observation_receipts.push({kind:label,received_wall_ms:receivedWall});retain();
+    if(result?.isError===true)latch("browser_tool_refused",receivedWall);
+    else if(!predicate(result))latch("browser_decision_unconfirmed",receivedWall);
+  } catch {latch("browser_tool_or_decision_exception",Date.now());}
+  if(record.first_failure!==null)await cleanup();
+  return record.first_failure===null?result:null;
+}
+const snapshotArgs={session:own.session,target_id:own.target_id,tab_id:own.tab_id,
+  snapshot_format:"semantic_v2",include_screenshot:false};
+function page(result,kind) {
+  const s=result?.structuredContent,nodes=Array.isArray(s?.content_refs)?s.content_refs:[];
+  const flags={
+    status_ok:result?.isError!==true&&s?.status==="ok",
+    complete:s?.snapshot?.complete===true,
+    heading_match:nodes.some(n=>n.role==="heading"&&n.name===
+      (kind==="protected_after"?"Protected application access":"Local demo")),
+    error_match:nodes.some(n=>n.name==="Local demo could not complete this request."),
+    required_text:nodes.some(n=>n.name===(kind==="protected_before"?"Sign in required.":
+      kind==="protected_after"?"Signed in. Protected application access is available.":
+      "Use Sign in to open this local application.")),
+    interactive_ref_present:Array.isArray(s?.refs)&&
+      s.refs.some(n=>Array.isArray(n.actions)&&n.actions.includes("click"))
+  };
+  record.last_page_flags={kind,...flags};retain();
+  // Only provider refs and fixed public-label matches survive this raw snapshot.
+  own.fresh_refs=Array.isArray(s?.refs)?s.refs.filter(n=>typeof n.ref==="string"&&
+    /^p[0-9]+:[0-9]+$/.test(n.ref)).map(n=>n.ref):[];
+  store("d01_immediate_owned_handles",own);
+  if(flags.error_match)return false;
+  if(!flags.status_ok||!flags.complete)return false;
+  if(kind==="generic_checked") {
+    if(nodes.some(n=>n.role==="heading"&&n.name==="Protected application access")&&
+       nodes.some(n=>n.name==="Signed in. Protected application access is available."))
+      record.protected_after_page=true;
+    return true;
+  }
+  const ok=flags.heading_match&&flags.required_text&&
+    (kind!=="application"||flags.interactive_ref_present);
+  if(ok&&kind==="protected_after")record.protected_after_page=true;
+  return ok;
+}
+async function navigateAndSnapshot(url,kind) {
+  if(await checked("browser_navigation",
+      ()=>tools.mcp__cua_driver__browser_navigate({session:own.session,
+        target_id:own.target_id,tab_id:own.tab_id,url}),
+      r=>r?.isError!==true)) {
+    await checked("browser_snapshot",
+      ()=>tools.mcp__cua_driver__get_browser_state(snapshotArgs),r=>page(r,kind));
+  }
+}
+async function entry() {
+  // The exact Driver-owned blank handles already exist while the180s gate waits.
+  // There is no model yield, tool-description load or rebind after this marker.
+  const markerCommand="python3 -c "+sq(MARKER)+" "+sq(own.lab)+" "+
+    sq(own.guard_pid)+" "+sq(own.server_pid);
+  await checked("prepared_marker",
+    ()=>tools.exec_command({cmd:markerCommand,workdir:own.workspace,
+      yield_time_ms:10000,max_output_tokens:500}),r=>{
+      record.local_tool_receipts.push({label:"marker",
+        exit:typeof r.exit_code==="number"?r.exit_code:null,
+        session_id:typeof r.session_id==="number"?r.session_id:null});retain();
+      if(r.session_id!==undefined)cleanupError("owned_local_command_unjoined");
+      return r.exit_code===0&&r.session_id===undefined&&
+        JSON.parse(r.output).prepared_marker_written===true;
+    });
+  const readyDeadline=Date.now()+30000;
+  while(record.first_failure===null&&own.fixture_ready!==true&&Date.now()<readyDeadline)
+    await pollController();
+  if(record.first_failure===null&&own.fixture_ready!==true)
+    latch("fixture_ready_unconfirmed",Date.now());
+  if(record.first_failure!==null){await cleanup();return;}
+  await pollController(); // ONE immediate pre-navigation poll, no RP HTTP probe.
+  if(record.first_failure!==null){await cleanup();return;}
+  await navigateAndSnapshot("http://localhost:3000/protected","protected_before");
+  if(record.first_failure===null)await pollController();
+  if(record.first_failure!==null){await cleanup();return;}
+  await navigateAndSnapshot("http://localhost:3000/","application");
+  if(record.first_failure===null)await pollController(); // ONE post-entry poll.
+  if(record.first_failure!==null)await cleanup();
+}
+async function continuation() {
+  const spec=own.next_decision;
+  const allowedKinds=["click","username","password","snapshot","protected_after"];
+  if(!spec||!allowedKinds.includes(spec.kind)) {
+    latch("browser_decision_unconfirmed",Date.now());await cleanup();return;
+  }
+  await pollController();
+  if(record.first_failure!==null){await cleanup();return;}
+  if(spec.kind==="protected_after") {
+    // Read the fresh callback-following protected page; do not send another RP request.
+    await checked("browser_snapshot",
+      ()=>tools.mcp__cua_driver__get_browser_state(snapshotArgs),r=>page(r,"protected_after"));
+  } else if(spec.kind==="snapshot") {
+    await checked("browser_snapshot",
+      ()=>tools.mcp__cua_driver__get_browser_state(snapshotArgs),
+      r=>page(r,"generic_checked")&&publicPredicate(r,spec));
+  } else {
+    if(typeof spec.ref!=="string"||!Array.isArray(own.fresh_refs)||
+       !own.fresh_refs.includes(spec.ref)) {
+      latch("browser_decision_unconfirmed",Date.now());await cleanup();return;
+    }
+    own.fresh_refs=[];store("d01_immediate_owned_handles",own); // Single-use snapshot ref.
+    const args={session:own.session,target_id:own.target_id,tab_id:own.tab_id,ref:spec.ref};
+    const operation=spec.kind==="click"?
+      ()=>tools.mcp__cua_driver__browser_click({...args,input_route:"dom_event"}):
+      ()=>tools.mcp__cua_driver__browser_type({...args,replace:true,
+        text:spec.kind==="username"?"admin":load("d01_fresh_password_input")});
+    if(spec.kind==="password"&&(typeof load("d01_fresh_password_input")!=="string"||
+       !/^[A-Za-z0-9_-]{30,128}$/.test(load("d01_fresh_password_input")))) {
+      latch("browser_decision_unconfirmed",Date.now());await cleanup();return;
+    }
+    await checked("browser_input",operation,r=>{
+      const s=r?.structuredContent;
+      return r?.isError!==true&&["confirmed","unverifiable"].includes(s?.effect);
+    }); // Dispatch alone never earns an application outcome or journey credit.
+    store("d01_fresh_password_input",null);
+    if(record.first_failure===null)
+      await checked("browser_snapshot",
+        ()=>tools.mcp__cua_driver__get_browser_state(snapshotArgs),
+        r=>page(r,"generic_checked")&&publicPredicate(r,spec));
+  }
+  if(record.first_failure===null)await pollController();
+  if(record.first_failure!==null)await cleanup();
+  else if(spec.kind==="protected_after") {
+    record.cleanup_requested_wall_ms=Date.now();retain();
+    await cleanup();
+  }
+}
+function publicPredicate(result,spec) {
+  const nodes=result?.structuredContent?.content_refs;
+  // Root must pin one printed public expected label/role before that action.
+  // No raw field value, URL, subject, query or arbitrary regex predicate is allowed.
+  const roles=["heading","button","statictext","textbox"];
+  const labels=["Username","Password","Authenticator or recovery code","Sign in",
+    "Local demo","Protected application access",
+    "Signed in. Protected application access is available."];
+  return Array.isArray(nodes)&&roles.includes(spec.expected_role)&&
+    labels.includes(spec.expected_label)&&
+    nodes.some(n=>n.role===spec.expected_role&&n.name===spec.expected_label);
+}
+try {
+  if(own.phase==="prepared_entry")await entry();else await continuation();
+} catch {
+  latch("composed_decision_exception",Date.now());
+  await cleanup();
+}
@@ -273 +445,0 @@
-  await cleanup(); // NO text/notify/yield_control/model turn before essential cleanup.
@@ -275,2 +447,4 @@
-    cleanup_errors:record.cleanup_errors,final_absence:record.final_absence,
-    controller_exit:record.controller_exit,whole_cleanup_within60_proven:false,
+    unexpected_failure_observation:record.unexpected_failure_observation,
+    diagnostic_errors:record.diagnostic_errors,cleanup_errors:record.cleanup_errors,
+    final_absence:record.final_absence,controller_exit:record.controller_exit,
+    whole_cleanup_within60_proven:false,
@@ -282,3 +456,8 @@
-  text({result:"application_checkpoint_observed_only",
-    snapshot_flags:lastSnapshotFlags,journey_credit:false});
-}
+  text({result:"browser_decision_observed_only",page_flags:record.last_page_flags??null,
+    journey_credit:false,cleanup_errors:record.cleanup_errors,
+    resource_release_proven:record.cleanup_started===true&&record.final_absence!==null&&
+      !record.cleanup_errors.some(v=>[
+        "child_reap_incomplete","owned_resource_absence_unproven",
+        "owned_readback_unavailable","owned_local_command_unjoined"].includes(v)),
+    whole_cleanup_within60_proven:false});
+}
```

Complete preparation failure-order boundary with explicit unavailable source inputs:

```javascript
// Preparation boundary contract; source-only, never called in this phase.
// Record/own/latch/cleanup are the same lexical objects as the composed cell.
// The exact returned-identity projection MUST be supplied from a reviewed
// retained Driver receipt adapter; this design does not guess its JSON keys.
async function preparationDecision(label,operation,projectOwnedIdentity) {
  let result,receivedWall;
  try {
    result=await operation();receivedWall=Date.now();
    record.observation_receipts.push({kind:label,received_wall_ms:receivedWall});
    retain();
    // Cache an attested PID/session/window even on a refusal if the reviewed
    // adapter proves ownership. Never derive ownership from a PID alone.
    const outcome=projectOwnedIdentity(result,own);
    if(result?.isError===true||outcome.confirmed!==true)
      latch("driver_preparation_unconfirmed",receivedWall);
  } catch {latch("driver_preparation_exception",Date.now());}
  if(record.first_failure!==null) {
    // If no exact owned browser PID was returned, do not call kill_app.
    // Join original controller and preserve absence/session uncertainty.
    await cleanupKnownPreparationResources();
    return null;
  }
  return result;
}
// REQUIRED call order, once each, while the unchanged180s controller gate waits:
// preparationDecision("prepare", ()=>tools.mcp__cua_driver__browser_prepare({
//   session:own.session,allow_launch:true,profile:{mode:"isolated_new"}}), adapter.prepare);
// preparationDecision("windows", ()=>tools.mcp__cua_driver__list_windows({
//   pid:own.browser_pid}), adapter.windows);
// preparationDecision("bind", ()=>tools.mcp__cua_driver__get_browser_state({
//   session:own.session,pid:own.browser_pid,window_id:own.window_id,
//   snapshot_format:"semantic_v2",include_screenshot:false}), adapter.bind);
// Then run entry() in the SAME functions.exec cell without text/yield/model wait.
// This boundary is a complete failure-order function, not an executable adapter:
// adapter.* and cleanupKnownPreparationResources remain source inputs unavailable
// in the retained public controller/284-line cell, so root must reserve/review
// their exact bodies before any fixture release on this proposal.
```

Complete CLOCK source (also embedded literally in the candidate cell):

```python
import json,time
print(json.dumps({'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())}))
```

Complete STOP source (also embedded literally in the candidate cell):

```python
import json,os,pathlib,stat,sys,time
c=json.loads(sys.argv[1])
clock={'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())}
record={'schema':'riauth.d01-first-observation/v1','first_failure':c['first_failure'],'first_observation_wall_ms':c['first_observation_wall_ms'],'first_event_proven':False,'clock':clock}
event_state='write_unconfirmed'
try:
    fd=os.open(pathlib.Path(c['event_out']),os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    with os.fdopen(fd,'w',encoding='ascii') as f:
        f.write(json.dumps(record,sort_keys=True)+'\n');f.flush();os.fsync(f.fileno())
    event_state='written'
except OSError:pass
# Attempt clock persistence BEFORE marker/state/budget comparisons.
# A metadata error does not prevent the original essential stop protocol.
lab=pathlib.Path(c['lab']);marker_state='lab_absent'
try:
    if lab.exists():
        info=lab.lstat()
        if not stat.S_ISDIR(info.st_mode) or stat.S_IMODE(info.st_mode)!=0o700 or info.st_uid!=os.getuid():
            marker_state='ownership_unknown'
        else:
            marker_state='stop_requested'
            for name in (('ui-failure','stop') if c['first_failure'] is not None else ('stop',)):
                try:
                    fd=os.open(lab/name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
                    os.close(fd)
                except FileNotFoundError:marker_state='lab_absent'
                except FileExistsError:pass
except OSError:marker_state='stop_unconfirmed'
print(json.dumps({'clock':clock,'event_state':event_state,'marker_state':marker_state}))
```

Complete READBACK source (also embedded literally in the candidate cell):

```python
import json,os,pathlib,stat,subprocess,sys,time
c=json.loads(sys.argv[1])
children=None;helper_pid=c['helper_pid'];outer_observation=None;projection='unavailable'
def finite_observation(value):
    if type(value) is not dict or set(value)!= {'observed','diagnostic'} or type(value['observed']) is not bool:
        return None
    diagnostic=value['diagnostic']
    if diagnostic is None:
        return {'observed':value['observed'],'diagnostic':None}
    if not value['observed'] or type(diagnostic) is not dict or set(diagnostic)!= {'site','exception_class','own_function','own_line'}:
        return None
    sites=('handler','server','main')
    kinds=('AttributeError','TypeError','ValueError','KeyError','OSError','BrokenPipeError','ConnectionResetError','TimeoutError','other')
    functions=('HeaderReader.readline','DemoServer.process_request','Demo.begin','Demo.callback','Demo.invoke','Handler.handle_one_request','Handler.send_error','Handler.get','Handler.reply','main')
    site,kind,function,line=(diagnostic[k] for k in ('site','exception_class','own_function','own_line'))
    if type(site) is not str or site not in sites or type(kind) is not str or kind not in kinds:
        return None
    if not ((function is None and line is None) or (type(function) is str and function in functions and type(line) is int and 1<=line<=1024)):
        return None
    return {'observed':True,'diagnostic':{'site':site,'exception_class':kind,'own_function':function,'own_line':line}}
outer=pathlib.Path(c['outer_out'])
if outer.is_file():
    fd=os.open(outer,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
    try:
        info=os.fstat(fd)
        if not (stat.S_ISREG(info.st_mode) and info.st_uid==os.getuid() and stat.S_IMODE(info.st_mode)==0o600 and 0<info.st_size<=262144):
            raise ValueError('fixed_outer_evidence_invalid')
        with os.fdopen(fd,'rb',closefd=False) as source:raw_bytes=source.read(262145)
        if not 0<len(raw_bytes)<=262144:raise ValueError('fixed_outer_evidence_invalid')
        data=json.loads(raw_bytes)
    finally:os.close(fd)
    outer_observation=finite_observation(data.get('unexpected_failure_observation'))
    projection=data.get('unexpected_observation_projection')
    if projection not in ('unavailable','valid','invalid'):projection='invalid'
    if projection=='valid' and outer_observation is None:projection='invalid'
    allowed={'whoami','discovery','confidential_client_create','operator_login','server','maintenance_init'}
    started=data.get('helper_invocations')==1 and type(data.get('helper_pid')) is int and data['helper_pid']>0
    if started:allowed.add('helper')
    raw=data.get('owned_child_exits')
    if isinstance(raw,list) and len(raw)==len(allowed) and all(isinstance(v,dict) and v.get('name') in allowed and type(v.get('pid')) is int and v['pid']>0 and type(v.get('exit')) is int for v in raw) and {v['name'] for v in raw}==allowed and len({v['pid'] for v in raw})==len(allowed) and next(v['pid'] for v in raw if v['name']=='server')==c['server_pid'] and ((started and next(v['pid'] for v in raw if v['name']=='helper')==data['helper_pid'] and (helper_pid is None or helper_pid==data['helper_pid'])) or (not started and helper_pid is None and data.get('helper_invocations') is None and data.get('helper_pid') is None)):
        children=[{k:v[k] for k in ('name','pid','exit')} for v in raw]
        helper_pid=data['helper_pid'] if started else None
pids=list(c['owned_pids'])
if helper_pid is not None and helper_pid not in pids:pids.append(helper_pid)
p=subprocess.run(['/bin/ps','-p',','.join(str(v) for v in pids),'-o','pid='],capture_output=True,timeout=3)
ps_known=p.returncode in (0,1) and all(v.isdigit() for v in p.stdout.split())
present=set(int(v) for v in p.stdout.split()) if ps_known else set()
ports={}
for port in (9000,3000):
    p=subprocess.run(['/usr/sbin/lsof','-nP','-t','-iTCP:'+str(port),'-sTCP:LISTEN'],capture_output=True,timeout=3)
    ports[str(port)]=not bool(p.stdout.strip()) if p.returncode in (0,1) else None
print(json.dumps({'clock':{'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())},'owned_pids':{str(v):(v not in present if ps_known else None) for v in pids},'ports':ports,'lab_absent':not pathlib.Path(c['lab']).exists(),'owned_child_exits':children,'helper_pid':helper_pid,'unexpected_failure_observation':outer_observation,'unexpected_observation_projection':projection}))
```

Complete PERSIST source (also embedded literally in the candidate cell):

```python
import json,os,pathlib,sys
path=pathlib.Path(sys.argv[1])
record=json.loads(sys.argv[2])
# Convert decimal strings directly to Python integers, avoiding JS Number rounding.
def clocks(value):
    if isinstance(value,dict):
        for k,v in list(value.items()):
            if k in ('monotonic_ns','wall_epoch_ns') and isinstance(v,str):
                assert v.isdecimal() and len(v)<=19 and 0<=int(v)<2**63
                value[k]=int(v)
            else:clocks(v)
    elif isinstance(value,list):
        for v in value:clocks(v)
clocks(record)
fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
with os.fdopen(fd,'w',encoding='ascii') as f:
    f.write(json.dumps(record,sort_keys=True,indent=2)+'\n');f.flush();os.fsync(f.fileno())
print(json.dumps({'written_exclusive':True}))
```

Complete MARKER source (also embedded literally in the candidate cell):

```python
import os,pathlib,stat,sys
lab=pathlib.Path(sys.argv[1]);guard=int(sys.argv[2]);server=int(sys.argv[3])
if guard<=0 or server<=0 or guard==server:raise ValueError('owned_context_invalid')
info=lab.lstat()
if not (stat.S_ISDIR(info.st_mode) and stat.S_IMODE(info.st_mode)==0o700 and info.st_uid==os.getuid()):raise ValueError('owned_context_invalid')
os.kill(guard,0);os.kill(server,0)
if (lab/'stop').exists() or (lab/'ui-failure').exists():raise ValueError('owned_context_invalid')
fd=os.open(lab/'browser-prepared',os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
os.close(fd)
print('{"prepared_marker_written":true}')
```

Exact STOP and READBACK deltas; MARKER is a new integration source only:

```diff
--- retained-stop
+++ DESIGN-only-stop
@@ -22 +22 @@
-            for name in ('ui-failure','stop'):
+            for name in (('ui-failure','stop') if c['first_failure'] is not None else ('stop',)):
```

```diff
--- retained-readback
+++ DESIGN-only-readback
@@ -1 +1 @@
-import json,pathlib,subprocess,sys,time
+import json,os,pathlib,stat,subprocess,sys,time
@@ -3 +3,42 @@
-pids=c['owned_pids']
+children=None;helper_pid=c['helper_pid'];outer_observation=None;projection='unavailable'
+def finite_observation(value):
+    if type(value) is not dict or set(value)!= {'observed','diagnostic'} or type(value['observed']) is not bool:
+        return None
+    diagnostic=value['diagnostic']
+    if diagnostic is None:
+        return {'observed':value['observed'],'diagnostic':None}
+    if not value['observed'] or type(diagnostic) is not dict or set(diagnostic)!= {'site','exception_class','own_function','own_line'}:
+        return None
+    sites=('handler','server','main')
+    kinds=('AttributeError','TypeError','ValueError','KeyError','OSError','BrokenPipeError','ConnectionResetError','TimeoutError','other')
+    functions=('HeaderReader.readline','DemoServer.process_request','Demo.begin','Demo.callback','Demo.invoke','Handler.handle_one_request','Handler.send_error','Handler.get','Handler.reply','main')
+    site,kind,function,line=(diagnostic[k] for k in ('site','exception_class','own_function','own_line'))
+    if type(site) is not str or site not in sites or type(kind) is not str or kind not in kinds:
+        return None
+    if not ((function is None and line is None) or (type(function) is str and function in functions and type(line) is int and 1<=line<=1024)):
+        return None
+    return {'observed':True,'diagnostic':{'site':site,'exception_class':kind,'own_function':function,'own_line':line}}
+outer=pathlib.Path(c['outer_out'])
+if outer.is_file():
+    fd=os.open(outer,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
+    try:
+        info=os.fstat(fd)
+        if not (stat.S_ISREG(info.st_mode) and info.st_uid==os.getuid() and stat.S_IMODE(info.st_mode)==0o600 and 0<info.st_size<=262144):
+            raise ValueError('fixed_outer_evidence_invalid')
+        with os.fdopen(fd,'rb',closefd=False) as source:raw_bytes=source.read(262145)
+        if not 0<len(raw_bytes)<=262144:raise ValueError('fixed_outer_evidence_invalid')
+        data=json.loads(raw_bytes)
+    finally:os.close(fd)
+    outer_observation=finite_observation(data.get('unexpected_failure_observation'))
+    projection=data.get('unexpected_observation_projection')
+    if projection not in ('unavailable','valid','invalid'):projection='invalid'
+    if projection=='valid' and outer_observation is None:projection='invalid'
+    allowed={'whoami','discovery','confidential_client_create','operator_login','server','maintenance_init'}
+    started=data.get('helper_invocations')==1 and type(data.get('helper_pid')) is int and data['helper_pid']>0
+    if started:allowed.add('helper')
+    raw=data.get('owned_child_exits')
+    if isinstance(raw,list) and len(raw)==len(allowed) and all(isinstance(v,dict) and v.get('name') in allowed and type(v.get('pid')) is int and v['pid']>0 and type(v.get('exit')) is int for v in raw) and {v['name'] for v in raw}==allowed and len({v['pid'] for v in raw})==len(allowed) and next(v['pid'] for v in raw if v['name']=='server')==c['server_pid'] and ((started and next(v['pid'] for v in raw if v['name']=='helper')==data['helper_pid'] and (helper_pid is None or helper_pid==data['helper_pid'])) or (not started and helper_pid is None and data.get('helper_invocations') is None and data.get('helper_pid') is None)):
+        children=[{k:v[k] for k in ('name','pid','exit')} for v in raw]
+        helper_pid=data['helper_pid'] if started else None
+pids=list(c['owned_pids'])
+if helper_pid is not None and helper_pid not in pids:pids.append(helper_pid)
@@ -11,10 +52 @@
-children=None
-outer=pathlib.Path(c['outer_out'])
-if outer.is_file():
-    # This is the unchanged controller's fixed redacted evidence, not app data.
-    data=json.loads(outer.read_bytes())
-    allowed={'helper','whoami','discovery','confidential_client_create','operator_login','server','maintenance_init'}
-    raw=data.get('owned_child_exits')
-    if isinstance(raw,list) and len(raw)==7 and all(isinstance(v,dict) and v.get('name') in allowed and type(v.get('pid')) is int and type(v.get('exit')) is int for v in raw) and {v['name'] for v in raw}==allowed and len({v['pid'] for v in raw})==7 and next(v['pid'] for v in raw if v['name']=='helper')==c['helper_pid'] and next(v['pid'] for v in raw if v['name']=='server')==c['server_pid']:
-        children=[{k:v[k] for k in ('name','pid','exit')} for v in raw]
-print(json.dumps({'clock':{'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())},'owned_pids':{str(v):(v not in present if ps_known else None) for v in pids},'ports':ports,'lab_absent':not pathlib.Path(c['lab']).exists(),'owned_child_exits':children}))
+print(json.dumps({'clock':{'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())},'owned_pids':{str(v):(v not in present if ps_known else None) for v in pids},'ports':ports,'lab_absent':not pathlib.Path(c['lab']).exists(),'owned_child_exits':children,'helper_pid':helper_pid,'unexpected_failure_observation':outer_observation,'unexpected_observation_projection':projection}))
```

### Actual static proofs, limits and immutable preservation

Actually performed: full controller/cell edit-unit reversal byte-for-byte to
6662/0a7cea, Python AST equality after controller reversal, all nine original
function ASTs identical, parse/in-memory code-object compile of proposed
controller and all five collector/marker literals, node --check exit0 for
the complete463-line cell and37-line preparation boundary. No code object,
VM/stub/harness/controller/helper/collector/module main was evaluated.
Exact JS byte reversal reconstructs identical parser input; no separate JS
AST tree was generated or evaluated. New full-controller/cell and all literal
sources/hashes/diffs above make reconstruction reviewable.

Controller has nine declared edit units/ten source occurrences, including
two helper-result projection sites. Cell has22 sequential edit units; reverse
their recorded replacements or apply the full zero-context diff backwards to
recover all284 original lines. Readback/stop integration deltas and marker are
fully declared, so no claimed unchanged collector secretly changed. These
are static source/diff counts, not tests or browser observations. New memory
validation would need root's separately reviewed exact payload/release; the
old18-case pass alone does not transfer to this source.

Verified entire2bf report518404B/SHA256
2ae110bff13b1943ee8edf0e8b2fb72f38ce417052032cb653fc945d62c4a982 as unchanged
prefix; unchanged actual helper470; all12 older private metadata exact bytes/
hashes/mode0600. No old metadata content was printed or reinterpreted.
All historic capacity/prepare/Authorization/request failures,500b senderUNKNOWN,
c88 genericunexpected/zeroAuth/protected403/no journey, lost provider values/
wrong-report correction,61/76 failed memory and dated78-case pass remain.
c88 first-event and whole60 stay UNKNOWN, including its119s final absence
after controller end; no retrospective timing/sender/cause claim occurs.

Proposed next action is root source review, not execution approval: obtain/
reserve the missing exact preparation adapter/partial cleanup and private-input
source; wait for actual descriptor-memory PASS plus independent review; review
the changed diagnostic/partial-readback/decision/success-stop behavior in one
bounded memory envelope before a separate ONE fixture release and fresh
capacity. No broader browser/operator/category/full-D01/D05/tenant/passkey/
install/HA/release/CI claim follows from this report. Original acceptance stays
root-owned, D01/D05 remain open, and closed rows remain closed.

This report alone is committed after prefix/archive/hash/scope/docs/whitespace
checks. No helper/product/guide/D05/test/config/controllerfile/main/history/
push/status edit, runtime/resource release, filesystem cleanup or worker
contact occurs. RiWork Cua.ai Driver MCP remains the only future GUI provider;
there was no Driver health/session/browser/tool invocation in this phase.

Final actual documentation check: python3 scripts/check-docs.py exited1. Its
sole missing-target message comes from protected prior-prefix line2813, where
an archived indexing call is scanned as a Markdown link to
SimpleNamespace(demo=d. This is a static documentation-check failure, not a
helper/fixture execution or a new product failure. A separate read-only scan
using the same link expression found exactly that one old-prefix match and
zero missing targets in this new appendix. The entire2bf prefix is mandatory
and was not edited to make the global check pass. Root's previously published
archive encoding is distinct from this retained local prefix; a future report
encoding/alignment change needs its own explicit reservation. git diff --check
exited0. Final archive/fence/hash/prefix/scope checks and the report-only local
commit follow; no source, runtime or gate inference is made from link checking.

Final archive readback reconstructs all8 complete sources against their
stated hashes,13 closed fence pairs, the exact518404-byte protected prefix,
all12 metadata hashes/modes and only the reserved report diff. The retained
shell command has NO final newline after PY. Its Markdown closing fence needs
one structural newline; remove exactly that delimiter newline when reconstructing
the17326-byte command. All other source fences are literal byte-exact content.
A static fence assertion exited1 after mistakenly removing that required
Markdown separator; restoring only the separator corrected the archive, without
changing the command/source hashes or executing anything. This static formatting
failure remains distinct from all historical fixture failures.


### Root review of the observer-aware controller design

Root read the complete immutable 2e29c30 controller, 463-line browser cell,
collector bodies, preparation boundary, prose and declared diffs. Reversing the
complete zero-context cell diff reproduced all 18,091 original bytes and SHA256
0a7cea070bdab32b419a82eb26a7a991b71a2b14a55553f890b53788cda04a5e.
The complete previously published report, archive encoding and root appendices
are retained, followed by the exact 116,987-byte author append. An initial root
prefix check omitted the already documented inserted encoding note and refused
before writing; including that note restored the intended preservation check.
A root-note append first used the orchestrator directory and failed before any
write. The author-only append was committed separately; this note follows it.

One source-derived lifecycle blocker remains: successful entry does not change
stored `own.phase` from `prepared_entry` to `browser_decision`, while populating
helper/readiness fields that the prepared-entry gate refuses on the next call.
Root reserved a report-only proposal for the successful-entry phase transition
and an independent review. This is not an observed fixture failure or a claim
about the earlier unknown sender or unexpected exception.

The distinct [descriptor memory receipt](evidence/wave30-d01-descriptor-memory-0ec651d.json)
now records one actual 128/128 memory pass. That result does not validate this
changed controller or prove a browser journey. Exact preparation/private-input
adapters, partial-ownership cleanup, changed-cell verification and a separately
released fixture remain prerequisites. No proposed controller, collector,
marker, Driver call or helper was executed by this root review. Runtime remains
held and original D01/D05 disposition remains open.

## Dated source-only phase: successful-entry phase transition, 2026-10-03 (Europe/Vaduz)

Reservation: wave30_D01_controller_entry_phase_design. Append ONLY this report,
preserving all635391 bytes of immutable2e29c30d01ccd41a2aac3914e3ac20afa38d324f,
SHA256c3d7d338676cc1156a9958253c2b51eab823eb25b7478c9ec1b0d4fba37c91a8.
No controller/helper/source file, evidence record, history alignment or runtime
was changed. No desktop/operator/Cargo slot was acquired or released.

Root's source review found a lifecycle blocker in the463-line candidate:
entry() populates helper_pid, fixture_ready and listener_pid_proof, but leaves
own.phase as prepared_entry. A next invocation selects the prepared-entry gate
and correctly refuses that now-populated context. This is a source-derived
candidate defect, not an observed GUI failure, helper outcome or retrospective
cause for any historic fixture.

The smallest correction inserts ONE four-line guarded block immediately after
entry's final poll and before the unchanged failure cleanup line. The test is
exactly record.first_failure===null. Only own.phase becomes browser_decision
and the same own object is stored under the existing key. Failed entry paths,
including failures identified by the final poll, never enter that block.
No await, tool call, output, deadline reset, marker, PID/ownership assertion,
metadata field, ref consumption, failure latch or cleanup operation is added.
The surrounding try/catch and all context gates remain exact.

### Exact hunk and complete corrected candidate — NOT EXECUTED

```diff
--- immutable-2e29c30-cell-eff0cdad
+++ DESIGN-only-successful-entry-phase
@@ -374,6 +374,10 @@
   if(record.first_failure!==null){await cleanup();return;}
   await navigateAndSnapshot("http://localhost:3000/","application");
   if(record.first_failure===null)await pollController(); // ONE post-entry poll.
+  if(record.first_failure===null) {
+    own.phase="browser_decision";
+    store("d01_immediate_owned_handles",own);
+  }
   if(record.first_failure!==null)await cleanup();
 }
 async function continuation() {
```

Complete467-line candidate31581B/SHA256
d727878009b955e6c6e309ffa90fe30c3644089b4a0fb7ebbe4ebd7264fd3d9d.
Its final newline is part of the exact fenced source.

```javascript
// Prospective ONE functions.exec cell; NOT executed in this design phase.
const CLOCK="import json,time\nprint(json.dumps({'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())}))\n";
const STOP="import json,os,pathlib,stat,sys,time\nc=json.loads(sys.argv[1])\nclock={'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())}\nrecord={'schema':'riauth.d01-first-observation/v1','first_failure':c['first_failure'],'first_observation_wall_ms':c['first_observation_wall_ms'],'first_event_proven':False,'clock':clock}\nevent_state='write_unconfirmed'\ntry:\n    fd=os.open(pathlib.Path(c['event_out']),os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\n    with os.fdopen(fd,'w',encoding='ascii') as f:\n        f.write(json.dumps(record,sort_keys=True)+'\\n');f.flush();os.fsync(f.fileno())\n    event_state='written'\nexcept OSError:pass\n# Attempt clock persistence BEFORE marker/state/budget comparisons.\n# A metadata error does not prevent the original essential stop protocol.\nlab=pathlib.Path(c['lab']);marker_state='lab_absent'\ntry:\n    if lab.exists():\n        info=lab.lstat()\n        if not stat.S_ISDIR(info.st_mode) or stat.S_IMODE(info.st_mode)!=0o700 or info.st_uid!=os.getuid():\n            marker_state='ownership_unknown'\n        else:\n            marker_state='stop_requested'\n            for name in (('ui-failure','stop') if c['first_failure'] is not None else ('stop',)):\n                try:\n                    fd=os.open(lab/name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\n                    os.close(fd)\n                except FileNotFoundError:marker_state='lab_absent'\n                except FileExistsError:pass\nexcept OSError:marker_state='stop_unconfirmed'\nprint(json.dumps({'clock':clock,'event_state':event_state,'marker_state':marker_state}))\n";
const READBACK="import json,os,pathlib,stat,subprocess,sys,time\nc=json.loads(sys.argv[1])\nchildren=None;helper_pid=c['helper_pid'];outer_observation=None;projection='unavailable'\ndef finite_observation(value):\n    if type(value) is not dict or set(value)!= {'observed','diagnostic'} or type(value['observed']) is not bool:\n        return None\n    diagnostic=value['diagnostic']\n    if diagnostic is None:\n        return {'observed':value['observed'],'diagnostic':None}\n    if not value['observed'] or type(diagnostic) is not dict or set(diagnostic)!= {'site','exception_class','own_function','own_line'}:\n        return None\n    sites=('handler','server','main')\n    kinds=('AttributeError','TypeError','ValueError','KeyError','OSError','BrokenPipeError','ConnectionResetError','TimeoutError','other')\n    functions=('HeaderReader.readline','DemoServer.process_request','Demo.begin','Demo.callback','Demo.invoke','Handler.handle_one_request','Handler.send_error','Handler.get','Handler.reply','main')\n    site,kind,function,line=(diagnostic[k] for k in ('site','exception_class','own_function','own_line'))\n    if type(site) is not str or site not in sites or type(kind) is not str or kind not in kinds:\n        return None\n    if not ((function is None and line is None) or (type(function) is str and function in functions and type(line) is int and 1<=line<=1024)):\n        return None\n    return {'observed':True,'diagnostic':{'site':site,'exception_class':kind,'own_function':function,'own_line':line}}\nouter=pathlib.Path(c['outer_out'])\nif outer.is_file():\n    fd=os.open(outer,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)\n    try:\n        info=os.fstat(fd)\n        if not (stat.S_ISREG(info.st_mode) and info.st_uid==os.getuid() and stat.S_IMODE(info.st_mode)==0o600 and 0<info.st_size<=262144):\n            raise ValueError('fixed_outer_evidence_invalid')\n        with os.fdopen(fd,'rb',closefd=False) as source:raw_bytes=source.read(262145)\n        if not 0<len(raw_bytes)<=262144:raise ValueError('fixed_outer_evidence_invalid')\n        data=json.loads(raw_bytes)\n    finally:os.close(fd)\n    outer_observation=finite_observation(data.get('unexpected_failure_observation'))\n    projection=data.get('unexpected_observation_projection')\n    if projection not in ('unavailable','valid','invalid'):projection='invalid'\n    if projection=='valid' and outer_observation is None:projection='invalid'\n    allowed={'whoami','discovery','confidential_client_create','operator_login','server','maintenance_init'}\n    started=data.get('helper_invocations')==1 and type(data.get('helper_pid')) is int and data['helper_pid']>0\n    if started:allowed.add('helper')\n    raw=data.get('owned_child_exits')\n    if isinstance(raw,list) and len(raw)==len(allowed) and all(isinstance(v,dict) and v.get('name') in allowed and type(v.get('pid')) is int and v['pid']>0 and type(v.get('exit')) is int for v in raw) and {v['name'] for v in raw}==allowed and len({v['pid'] for v in raw})==len(allowed) and next(v['pid'] for v in raw if v['name']=='server')==c['server_pid'] and ((started and next(v['pid'] for v in raw if v['name']=='helper')==data['helper_pid'] and (helper_pid is None or helper_pid==data['helper_pid'])) or (not started and helper_pid is None and data.get('helper_invocations') is None and data.get('helper_pid') is None)):\n        children=[{k:v[k] for k in ('name','pid','exit')} for v in raw]\n        helper_pid=data['helper_pid'] if started else None\npids=list(c['owned_pids'])\nif helper_pid is not None and helper_pid not in pids:pids.append(helper_pid)\np=subprocess.run(['/bin/ps','-p',','.join(str(v) for v in pids),'-o','pid='],capture_output=True,timeout=3)\nps_known=p.returncode in (0,1) and all(v.isdigit() for v in p.stdout.split())\npresent=set(int(v) for v in p.stdout.split()) if ps_known else set()\nports={}\nfor port in (9000,3000):\n    p=subprocess.run(['/usr/sbin/lsof','-nP','-t','-iTCP:'+str(port),'-sTCP:LISTEN'],capture_output=True,timeout=3)\n    ports[str(port)]=not bool(p.stdout.strip()) if p.returncode in (0,1) else None\nprint(json.dumps({'clock':{'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())},'owned_pids':{str(v):(v not in present if ps_known else None) for v in pids},'ports':ports,'lab_absent':not pathlib.Path(c['lab']).exists(),'owned_child_exits':children,'helper_pid':helper_pid,'unexpected_failure_observation':outer_observation,'unexpected_observation_projection':projection}))\n";
const MARKER="import os,pathlib,stat,sys\nlab=pathlib.Path(sys.argv[1]);guard=int(sys.argv[2]);server=int(sys.argv[3])\nif guard<=0 or server<=0 or guard==server:raise ValueError('owned_context_invalid')\ninfo=lab.lstat()\nif not (stat.S_ISDIR(info.st_mode) and stat.S_IMODE(info.st_mode)==0o700 and info.st_uid==os.getuid()):raise ValueError('owned_context_invalid')\nos.kill(guard,0);os.kill(server,0)\nif (lab/'stop').exists() or (lab/'ui-failure').exists():raise ValueError('owned_context_invalid')\nfd=os.open(lab/'browser-prepared',os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\nos.close(fd)\nprint('{\"prepared_marker_written\":true}')\n";
const PERSIST="import json,os,pathlib,sys\npath=pathlib.Path(sys.argv[1])\nrecord=json.loads(sys.argv[2])\n# Convert decimal strings directly to Python integers, avoiding JS Number rounding.\ndef clocks(value):\n    if isinstance(value,dict):\n        for k,v in list(value.items()):\n            if k in ('monotonic_ns','wall_epoch_ns') and isinstance(v,str):\n                assert v.isdecimal() and len(v)<=19 and 0<=int(v)<2**63\n                value[k]=int(v)\n            else:clocks(v)\n    elif isinstance(value,list):\n        for v in value:clocks(v)\nclocks(record)\nfd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\nwith os.fdopen(fd,'w',encoding='ascii') as f:\n    f.write(json.dumps(record,sort_keys=True,indent=2)+'\\n');f.flush();os.fsync(f.fileno())\nprint(json.dumps({'written_exclusive':True}))\n";
const own=load("d01_immediate_owned_handles");
const OBSKEY="d01_immediate_observation_record";
if (!own || own.runtime_release!==true || own.driver_owned!==true ||
    own.exact_bound!==true || own.single_controller!==true ||
    !["prepared_entry","browser_decision"].includes(own.phase) ||
    (own.phase==="prepared_entry"&&(own.prepared_gate!==true||own.helper_pid!==null||
      own.fixture_ready===true||own.listener_pid_proof===true)) ||
    (own.phase==="browser_decision"&&(own.fixture_ready!==true||own.listener_pid_proof!==true)) ||
    own.fresh_paths_preflight!==true ||
    !Number.isSafeInteger(own.start_epoch_ms) || typeof own.workspace!=="string" ||
    new Set([own.guard_pid,own.server_pid,own.browser_pid,
       ...(own.helper_pid===null?[]:[own.helper_pid])]).size!==(own.helper_pid===null?3:4) ||
    ![own.guard_pid,own.server_pid,own.browser_pid,own.exec_session,
       ...(own.helper_pid===null?[]:[own.helper_pid])]
       .every(v=>Number.isSafeInteger(v)&&v>0) ||
    ![own.session,own.target_id,own.tab_id,own.lab,own.outer_out,
       own.event_out,own.cleanup_out].every(v=>typeof v==="string"&&v.length>0)) {
  text({proposal_refused:"fresh_owned_context_required"});exit();
}
const prior=load(OBSKEY);
if(own.closed===true||prior?.cleanup_started===true){
  text({proposal_refused:"fixture_already_stopped",resource_release_proven:false});exit();
}
const record=prior??{
  schema:"riauth.d01-immediate-observation-cleanup/v1",
  first_failure:null,first_observation_wall_ms:null,
  first_event_proven:false,whole_cleanup_within60_proven:false,
  unexpected_failure_observation:null,diagnostic_errors:[],cleanup_started:false,
  first_clock:null,observation_receipts:[],controller_observations:[],local_tool_receipts:[],actions:[],cleanup_errors:[],
  final_absence:null,owned_child_exits:null,controller_exit:null,
  first_observation_to_final_wall_ms:null,latch_to_absence_ms:null
};
const sq=s=>"'"+String(s).replace(/'/g,"'\\''")+"'";
const retain=()=>store(OBSKEY,record);
const cleanupError=label=>{
  if(!record.cleanup_errors.includes(label))record.cleanup_errors.push(label);
  retain();
};
const local=async(source,arg)=>{
  const cmd="python3 -c "+sq(source)+(arg===undefined?"":" "+sq(JSON.stringify(arg)));
  const r=await tools.exec_command({cmd,workdir:own.workspace,
    yield_time_ms:10000,max_output_tokens:2000});
  record.local_tool_receipts.push({
    label:source===CLOCK?"clock":source===STOP?"stop":source===READBACK?"readback":"unknown",
    exit:typeof r.exit_code==="number"?r.exit_code:null,
    session_id:typeof r.session_id==="number"?r.session_id:null
  });retain(); // Numeric collector receipt BEFORE comparisons.
  if(r.session_id!==undefined) {
    cleanupError("owned_local_command_unjoined");throw new Error("local_pending");
  }
  if(r.exit_code!==0)throw new Error("local_failed");
  return JSON.parse(r.output);
};
function finiteObservation(value) {
  const exact=(v,keys)=>v!==null&&typeof v==="object"&&!Array.isArray(v)&&
    Object.keys(v).length===keys.length&&keys.every(k=>Object.hasOwn(v,k));
  if(!exact(value,["observed","diagnostic"])||typeof value.observed!=="boolean")return null;
  const d=value.diagnostic;
  if(d===null)return {observed:value.observed,diagnostic:null};
  if(!value.observed||!exact(d,["site","exception_class","own_function","own_line"])||
     !["handler","server","main"].includes(d.site)||
     !["AttributeError","TypeError","ValueError","KeyError","OSError","BrokenPipeError",
       "ConnectionResetError","TimeoutError","other"].includes(d.exception_class))return null;
  const functions=["HeaderReader.readline","DemoServer.process_request","Demo.begin","Demo.callback",
    "Demo.invoke","Handler.handle_one_request","Handler.send_error","Handler.get","Handler.reply","main"];
  if(!((d.own_function===null&&d.own_line===null)||
       (functions.includes(d.own_function)&&Number.isSafeInteger(d.own_line)&&
        d.own_line>=1&&d.own_line<=1024)))return null;
  return {observed:true,diagnostic:{site:d.site,exception_class:d.exception_class,
    own_function:d.own_function,own_line:d.own_line}};
}
function diagnostic(value,projection) {
  const projected=finiteObservation(value);
  if(projection==="invalid"||!["valid","unavailable"].includes(projection)||
     (projection==="valid"&&projected===null)) {
    if(!record.diagnostic_errors.includes("observer_projection_invalid"))
      record.diagnostic_errors.push("observer_projection_invalid");
  }
  if(projected!==null&&record.unexpected_failure_observation===null)
    record.unexpected_failure_observation=projected;
  retain(); // No raw diagnostic fallback and no first-failure or outcome mutation.
}
const ns=c=>BigInt(c.monotonic_ns);
const getClock=()=>local(CLOCK);
let controllerJoined=false;
let controllerPollAvailable=true;
let controllerBuffer="";
let cleanupStarted=false;
let lastSnapshotFlags=null;
function latch(label,receivedWall) {
  if(record.first_failure===null) {
    record.first_failure=label;
    record.first_observation_wall_ms=receivedWall;
    retain(); // Synchronous first-failure/wall latch BEFORE any new await/output.
  }
}
function observeController(r,receivedWall) {
  const observation={received_wall_ms:receivedWall,
    exit:typeof r.exit_code==="number"?r.exit_code:null,
    helper_completed:false,helper_exit:null,fixture_finished:false};
  // Complete numeric result projection is retained BEFORE comparisons.
  record.controller_observations.push(observation);retain();
  if(typeof r.exit_code==="number") {
    controllerJoined=true;record.controller_exit=r.exit_code;retain();
  }
  controllerBuffer+=typeof r.output==="string"?r.output:"";
  if(controllerBuffer.length>16384) {
    latch("controller_observation_invalid",receivedWall);controllerBuffer="";return;
  }
  let cut;
  while((cut=controllerBuffer.indexOf("\n"))>=0) {
    const line=controllerBuffer.slice(0,cut);controllerBuffer=controllerBuffer.slice(cut+1);
    if(!line.trim())continue;
    let event;
    try{event=JSON.parse(line);}catch{
      latch("controller_observation_invalid",receivedWall);continue;
    }
    if(Object.hasOwn(event,"unexpected_failure_observation"))
      diagnostic(event.unexpected_failure_observation,event.unexpected_observation_projection);
    if(event.fixture_ready===true) {
      if(event.guard_pid!==own.guard_pid||event.server_pid!==own.server_pid||
         event.lab!==own.lab||!Number.isSafeInteger(event.helper_pid)||event.helper_pid<=0||
         [own.guard_pid,own.server_pid,own.browser_pid].includes(event.helper_pid)||
         (own.helper_pid!==null&&own.helper_pid!==event.helper_pid))
        latch("fixture_ready_unconfirmed",receivedWall);
      else {
        own.helper_pid=event.helper_pid;own.fixture_ready=true;own.listener_pid_proof=true;
        store("d01_immediate_owned_handles",own);
      }
    }
    if(event.helper_completed===true) {
      observation.helper_completed=true;
      observation.helper_exit=typeof event.exit==="number"?event.exit:null;
      retain();
      if(event.exit!==0)latch("helper_failed",receivedWall);
      else if(!cleanupStarted&&record.protected_after_page!==true)
        latch("helper_completed_before_app_checkpoint",receivedWall);
    }
    if(event.fixture_finished===true) {
      observation.fixture_finished=true;retain();
      if(!cleanupStarted)latch("controller_completed_before_app_checkpoint",receivedWall);
    }
  }
  if(controllerJoined&&!cleanupStarted)
    latch("controller_completed_before_app_checkpoint",receivedWall);
}
async function pollController() {
  if(controllerJoined||!controllerPollAvailable)return;
  try {
    const r=await tools.write_stdin({session_id:own.exec_session,
      chars:"",yield_time_ms:5000,max_output_tokens:2000});
    const receivedWall=Date.now();
    observeController(r,receivedWall);
  } catch {
    controllerPollAvailable=false;
    latch("controller_observation_unavailable",Date.now());
    if(cleanupStarted)cleanupError("controller_join_unconfirmed");
  }
}
async function timedDriver(label,allocation,operation,project) {
  let start=null,end=null,result=null,state="unknown";
  try{start=await getClock();}catch{cleanupError("clock_unavailable");}
  const action={label,start_clock:start,end_clock:null,allowed_ms:allocation,
    result_state:"unknown",elapsed_ms:null,over_budget:null};
  record.actions.push(action);retain(); // Start retained BEFORE entering Driver call.
  try {
    result=await operation();
    state=result?.isError===true?"refused":"returned";
  } catch {state="exception";}
  try{end=await getClock();}catch{cleanupError("clock_unavailable");}
  action.end_clock=end;action.result_state=state;
  retain(); // End/result retained BEFORE budget comparison.
  if(start&&end) {
    const d=ns(end)-ns(start);
    if(d>=0n) {
      action.elapsed_ms=Number(d/1000000n);
      action.over_budget=action.elapsed_ms>allocation;
    } else cleanupError("clock_invalid");
  }
  if(action.over_budget)cleanupError("driver_operation_over_budget");
  if(state!=="returned")cleanupError("driver_operation_unconfirmed");
  if(project)project(result,state);
  retain();
}
async function cleanup() {
  if(cleanupStarted)return;
  cleanupStarted=true;record.cleanup_started=true;retain();
  try {
    const r=await local(STOP,{
      lab:own.lab,event_out:own.event_out,
      first_failure:record.first_failure,
      first_observation_wall_ms:record.first_observation_wall_ms
    });
    record.first_clock=r.clock;record.stop_state=r.marker_state;retain();
    if(r.event_state!=="written")cleanupError("observation_metadata_write_unconfirmed");
    if(r.marker_state==="stop_unconfirmed")cleanupError("stop_unconfirmed");
    if(r.marker_state==="ownership_unknown")cleanupError("ownership_unknown");
  } catch {cleanupError("stop_or_clock_record_unavailable");}
  // No outstanding Driver call exists here: all page calls above were awaited.
  // Original stop protocol already causes the controller's owned-child finally.
  await timedDriver("kill_app",30000,
    ()=>tools.mcp__cua_driver__kill_app({pid:own.browser_pid}));
  await timedDriver("end_session",15000,
    ()=>tools.mcp__cua_driver__end_session({session:own.session}),(r,state)=>{
      record.session_ended=state==="returned"&&
        r?.structuredContent?.active===false&&r.structuredContent.session===own.session;
      retain();
    });
  await timedDriver("list_windows",5000,
    ()=>tools.mcp__cua_driver__list_windows({pid:own.browser_pid}),(r,state)=>{
      const windows=r?.structuredContent?.windows;
      record.window_count=state==="returned"&&Array.isArray(windows)?windows.length:null;
      retain();
    });
  let readStart=null;
  try{readStart=await getClock();}catch{cleanupError("clock_unavailable");}
  const action={label:"owned_join_readback",start_clock:readStart,end_clock:null,
    allowed_ms:null,elapsed_ms:null,over_budget:null,result_state:"unknown"};
  record.actions.push(action);retain();
  if(readStart&&record.first_clock) {
    action.allowed_ms=Math.max(0,60000-Number((ns(readStart)-ns(record.first_clock))/1000000n));
    retain();
  }
  // Continue essential join after60s if late; never claim that deadline enforced.
  // Do not poll another exec session or send a manual process signal.
  while(!controllerJoined&&controllerPollAvailable&&Date.now()<own.start_epoch_ms+900000) {
    await pollController();
    if(record.first_clock) {
      try {
        const c=await getClock();record.last_join_clock=c;retain();
        if(ns(c)-ns(record.first_clock)>60000000000n)
          cleanupError("cleanup_budget_exceeded");
      } catch{cleanupError("clock_unavailable");}
    }
  }
  if(!controllerJoined)cleanupError("child_reap_incomplete");
  try {
    const r=await local(READBACK,{
      owned_pids:[own.guard_pid,own.server_pid,own.browser_pid,
        ...(own.helper_pid===null?[]:[own.helper_pid])],
      lab:own.lab,outer_out:own.outer_out,helper_pid:own.helper_pid,server_pid:own.server_pid
    });
    action.end_clock=r.clock;action.result_state="returned";
    record.owned_child_exits=r.owned_child_exits;
    diagnostic(r.unexpected_failure_observation,r.unexpected_observation_projection);
    if(Number.isSafeInteger(r.helper_pid)&&r.helper_pid>0)own.helper_pid=r.helper_pid;
    record.final_absence={owned_pids:r.owned_pids,ports:r.ports,
      lab:r.lab_absent,window_count:record.window_count??null,
      session_ended:record.session_ended===true};
    record.first_observation_to_final_wall_ms=record.first_observation_wall_ms===null?null:
      Date.now()-record.first_observation_wall_ms;
    retain(); // Full fixed readback/exits/clock BEFORE comparisons.
    if(readStart) {
      action.elapsed_ms=Number((ns(r.clock)-ns(readStart))/1000000n);
      action.over_budget=action.allowed_ms===null?null:action.elapsed_ms>action.allowed_ms;
    }
    if(record.first_clock)
      record.latch_to_absence_ms=Number((ns(r.clock)-ns(record.first_clock))/1000000n);
    if(action.over_budget)cleanupError("cleanup_budget_exceeded");
    const a=record.final_absence;
    if(!controllerJoined||!Array.isArray(record.owned_child_exits)||
       record.owned_child_exits.length!==(own.helper_pid===null?6:7))
      cleanupError("child_reap_incomplete");
    if(!Object.values(a.owned_pids).every(v=>v===true)||
       !Object.values(a.ports).every(v=>v===true)||a.lab!==true||
       a.window_count!==0||a.session_ended!==true)
      cleanupError("owned_resource_absence_unproven");
  } catch {action.result_state="exception";cleanupError("owned_readback_unavailable");}
  cleanupError("first_event_unproven");
  record.whole_cleanup_within60_proven=false;retain();
  // Exclusive new file only; existing outer/helper/provider evidence untouched.
  try {
    const cmd="python3 -c "+sq(PERSIST)+" "+sq(own.cleanup_out)+" "+sq(JSON.stringify(record));
    const r=await tools.exec_command({cmd,workdir:own.workspace,
      yield_time_ms:10000,max_output_tokens:500});
    record.local_tool_receipts.push({label:"persist",
      exit:typeof r.exit_code==="number"?r.exit_code:null,
      session_id:typeof r.session_id==="number"?r.session_id:null});retain();
    if(r.session_id!==undefined)cleanupError("owned_local_command_unjoined");
    if(r.session_id!==undefined||r.exit_code!==0)
      cleanupError("cleanup_metadata_write_unconfirmed");
  } catch {cleanupError("cleanup_metadata_write_unconfirmed");}
  controllerBuffer="";own.closed=true;store("d01_immediate_owned_handles",own);
}
async function checked(label,operation,predicate) {
  if(record.first_failure!==null)return null;
  if(Date.now()>=own.start_epoch_ms+840000) {
    latch("browser_active_deadline",Date.now());await cleanup();return null;
  }
  let result,receivedWall;
  try {
    result=await operation();receivedWall=Date.now();
    record.observation_receipts.push({kind:label,received_wall_ms:receivedWall});retain();
    if(result?.isError===true)latch("browser_tool_refused",receivedWall);
    else if(!predicate(result))latch("browser_decision_unconfirmed",receivedWall);
  } catch {latch("browser_tool_or_decision_exception",Date.now());}
  if(record.first_failure!==null)await cleanup();
  return record.first_failure===null?result:null;
}
const snapshotArgs={session:own.session,target_id:own.target_id,tab_id:own.tab_id,
  snapshot_format:"semantic_v2",include_screenshot:false};
function page(result,kind) {
  const s=result?.structuredContent,nodes=Array.isArray(s?.content_refs)?s.content_refs:[];
  const flags={
    status_ok:result?.isError!==true&&s?.status==="ok",
    complete:s?.snapshot?.complete===true,
    heading_match:nodes.some(n=>n.role==="heading"&&n.name===
      (kind==="protected_after"?"Protected application access":"Local demo")),
    error_match:nodes.some(n=>n.name==="Local demo could not complete this request."),
    required_text:nodes.some(n=>n.name===(kind==="protected_before"?"Sign in required.":
      kind==="protected_after"?"Signed in. Protected application access is available.":
      "Use Sign in to open this local application.")),
    interactive_ref_present:Array.isArray(s?.refs)&&
      s.refs.some(n=>Array.isArray(n.actions)&&n.actions.includes("click"))
  };
  record.last_page_flags={kind,...flags};retain();
  // Only provider refs and fixed public-label matches survive this raw snapshot.
  own.fresh_refs=Array.isArray(s?.refs)?s.refs.filter(n=>typeof n.ref==="string"&&
    /^p[0-9]+:[0-9]+$/.test(n.ref)).map(n=>n.ref):[];
  store("d01_immediate_owned_handles",own);
  if(flags.error_match)return false;
  if(!flags.status_ok||!flags.complete)return false;
  if(kind==="generic_checked") {
    if(nodes.some(n=>n.role==="heading"&&n.name==="Protected application access")&&
       nodes.some(n=>n.name==="Signed in. Protected application access is available."))
      record.protected_after_page=true;
    return true;
  }
  const ok=flags.heading_match&&flags.required_text&&
    (kind!=="application"||flags.interactive_ref_present);
  if(ok&&kind==="protected_after")record.protected_after_page=true;
  return ok;
}
async function navigateAndSnapshot(url,kind) {
  if(await checked("browser_navigation",
      ()=>tools.mcp__cua_driver__browser_navigate({session:own.session,
        target_id:own.target_id,tab_id:own.tab_id,url}),
      r=>r?.isError!==true)) {
    await checked("browser_snapshot",
      ()=>tools.mcp__cua_driver__get_browser_state(snapshotArgs),r=>page(r,kind));
  }
}
async function entry() {
  // The exact Driver-owned blank handles already exist while the180s gate waits.
  // There is no model yield, tool-description load or rebind after this marker.
  const markerCommand="python3 -c "+sq(MARKER)+" "+sq(own.lab)+" "+
    sq(own.guard_pid)+" "+sq(own.server_pid);
  await checked("prepared_marker",
    ()=>tools.exec_command({cmd:markerCommand,workdir:own.workspace,
      yield_time_ms:10000,max_output_tokens:500}),r=>{
      record.local_tool_receipts.push({label:"marker",
        exit:typeof r.exit_code==="number"?r.exit_code:null,
        session_id:typeof r.session_id==="number"?r.session_id:null});retain();
      if(r.session_id!==undefined)cleanupError("owned_local_command_unjoined");
      return r.exit_code===0&&r.session_id===undefined&&
        JSON.parse(r.output).prepared_marker_written===true;
    });
  const readyDeadline=Date.now()+30000;
  while(record.first_failure===null&&own.fixture_ready!==true&&Date.now()<readyDeadline)
    await pollController();
  if(record.first_failure===null&&own.fixture_ready!==true)
    latch("fixture_ready_unconfirmed",Date.now());
  if(record.first_failure!==null){await cleanup();return;}
  await pollController(); // ONE immediate pre-navigation poll, no RP HTTP probe.
  if(record.first_failure!==null){await cleanup();return;}
  await navigateAndSnapshot("http://localhost:3000/protected","protected_before");
  if(record.first_failure===null)await pollController();
  if(record.first_failure!==null){await cleanup();return;}
  await navigateAndSnapshot("http://localhost:3000/","application");
  if(record.first_failure===null)await pollController(); // ONE post-entry poll.
  if(record.first_failure===null) {
    own.phase="browser_decision";
    store("d01_immediate_owned_handles",own);
  }
  if(record.first_failure!==null)await cleanup();
}
async function continuation() {
  const spec=own.next_decision;
  const allowedKinds=["click","username","password","snapshot","protected_after"];
  if(!spec||!allowedKinds.includes(spec.kind)) {
    latch("browser_decision_unconfirmed",Date.now());await cleanup();return;
  }
  await pollController();
  if(record.first_failure!==null){await cleanup();return;}
  if(spec.kind==="protected_after") {
    // Read the fresh callback-following protected page; do not send another RP request.
    await checked("browser_snapshot",
      ()=>tools.mcp__cua_driver__get_browser_state(snapshotArgs),r=>page(r,"protected_after"));
  } else if(spec.kind==="snapshot") {
    await checked("browser_snapshot",
      ()=>tools.mcp__cua_driver__get_browser_state(snapshotArgs),
      r=>page(r,"generic_checked")&&publicPredicate(r,spec));
  } else {
    if(typeof spec.ref!=="string"||!Array.isArray(own.fresh_refs)||
       !own.fresh_refs.includes(spec.ref)) {
      latch("browser_decision_unconfirmed",Date.now());await cleanup();return;
    }
    own.fresh_refs=[];store("d01_immediate_owned_handles",own); // Single-use snapshot ref.
    const args={session:own.session,target_id:own.target_id,tab_id:own.tab_id,ref:spec.ref};
    const operation=spec.kind==="click"?
      ()=>tools.mcp__cua_driver__browser_click({...args,input_route:"dom_event"}):
      ()=>tools.mcp__cua_driver__browser_type({...args,replace:true,
        text:spec.kind==="username"?"admin":load("d01_fresh_password_input")});
    if(spec.kind==="password"&&(typeof load("d01_fresh_password_input")!=="string"||
       !/^[A-Za-z0-9_-]{30,128}$/.test(load("d01_fresh_password_input")))) {
      latch("browser_decision_unconfirmed",Date.now());await cleanup();return;
    }
    await checked("browser_input",operation,r=>{
      const s=r?.structuredContent;
      return r?.isError!==true&&["confirmed","unverifiable"].includes(s?.effect);
    }); // Dispatch alone never earns an application outcome or journey credit.
    store("d01_fresh_password_input",null);
    if(record.first_failure===null)
      await checked("browser_snapshot",
        ()=>tools.mcp__cua_driver__get_browser_state(snapshotArgs),
        r=>page(r,"generic_checked")&&publicPredicate(r,spec));
  }
  if(record.first_failure===null)await pollController();
  if(record.first_failure!==null)await cleanup();
  else if(spec.kind==="protected_after") {
    record.cleanup_requested_wall_ms=Date.now();retain();
    await cleanup();
  }
}
function publicPredicate(result,spec) {
  const nodes=result?.structuredContent?.content_refs;
  // Root must pin one printed public expected label/role before that action.
  // No raw field value, URL, subject, query or arbitrary regex predicate is allowed.
  const roles=["heading","button","statictext","textbox"];
  const labels=["Username","Password","Authenticator or recovery code","Sign in",
    "Local demo","Protected application access",
    "Signed in. Protected application access is available."];
  return Array.isArray(nodes)&&roles.includes(spec.expected_role)&&
    labels.includes(spec.expected_label)&&
    nodes.some(n=>n.role===spec.expected_role&&n.name===spec.expected_label);
}
try {
  if(own.phase==="prepared_entry")await entry();else await continuation();
} catch {
  latch("composed_decision_exception",Date.now());
  await cleanup();
}
if(record.first_failure!==null) {
  text({result:"failed",first_failure:record.first_failure,
    unexpected_failure_observation:record.unexpected_failure_observation,
    diagnostic_errors:record.diagnostic_errors,cleanup_errors:record.cleanup_errors,
    final_absence:record.final_absence,controller_exit:record.controller_exit,
    whole_cleanup_within60_proven:false,
    resource_release_proven:record.final_absence!==null&&
      !record.cleanup_errors.some(v=>[
        "child_reap_incomplete","owned_resource_absence_unproven",
        "owned_readback_unavailable","owned_local_command_unjoined"].includes(v))});
} else {
  text({result:"browser_decision_observed_only",page_flags:record.last_page_flags??null,
    journey_credit:false,cleanup_errors:record.cleanup_errors,
    resource_release_proven:record.cleanup_started===true&&record.final_absence!==null&&
      !record.cleanup_errors.some(v=>[
        "child_reap_incomplete","owned_resource_absence_unproven",
        "owned_readback_unavailable","owned_local_command_unjoined"].includes(v)),
    whole_cleanup_within60_proven:false});
}
```

### Actual static byte and AST proofs

Baseline is the complete literal candidate archived inside2e29c30:
31461B/463lines/SHA256
eff0cdad3981ce0ad57d355203b082e21371553408444bf46a3b4ecfcf5fea0a.
It was extracted/matched against that immutable report, not a mutable worker
partial file. Removing precisely the inserted120-byte block reconstructs
the complete old candidate byte-for-byte. The outer controller5eb6ab2d,
all collector literals, preparation boundary and every other source in the
prior appendix remain unchanged.

A static checker used the already installed Node bundled Acorn parser:
parse old, corrected and byte-restored source without evaluation; normalize
only AST location offsets and serialize source BigInt literals as labelled
decimal strings; verify the added node exactly equals the expected guarded
assignment/store node; verify it is immediately after the final awaited
pollController and before the old cleanup line; remove only that node and
compare the entire AST to baseline. All13 top-level function declarations
and all remaining expressions/statements are therefore unchanged after that
one-node reversal. No VM, mocked cell, helper/module main, collector, native
provider, HTTP/socket/listener, CLI, Driver/browser or harness was executed.

First static AST-checker invocation exited1 because plain JSON serialization
cannot encode the existing source BigInt literals. No candidate code ran and
no candidate bytes changed. The checker alone added a labelled decimal
normalization for those literal AST values. Its corrected invocation exited0;
node --check --input-type=module also exited0 with empty stdout/stderr.
This static-check failure is retained separately from historical product/
fixture failures. Neither syntax nor AST equivalence is a lifecycle memory
execution or a journey pass.

Complete static checker source2160B/SHA256
c7d3d95eef92dee74d3cc3fb189d002e7d616c42c0af8836d0baaeaafc90bfff;
it reads only public source JSON from stdin and parses it. Archive below is
for reproducibility, not permission to execute a candidate or runtime fixture.

```javascript
const fs=require("node:fs"),crypto=require("node:crypto");
const acorn=require("internal/deps/acorn/acorn/dist/acorn");
const input=JSON.parse(fs.readFileSync(0,"utf8"));
const parse=s=>acorn.parse(s,{ecmaVersion:"latest",sourceType:"module"});
const normalize=value=>{
  if(typeof value==="bigint")return {ast_bigint_decimal:value.toString(10)};
  if(Array.isArray(value))return value.map(normalize);
  if(value&&typeof value==="object")return Object.fromEntries(Object.entries(value)
    .filter(([k])=>!["start","end","loc","range"].includes(k))
    .map(([k,v])=>[k,normalize(v)]));
  return value;
};
const baseline=normalize(parse(input.old)),candidate=normalize(parse(input.candidate));
const restored=normalize(parse(input.candidate.replace(input.anchor+input.insert,input.anchor)));
if(JSON.stringify(restored)!==JSON.stringify(baseline))throw Error("ast_reversal");
const entry=candidate.body.find(n=>n.type==="FunctionDeclaration"&&n.id.name==="entry");
const expected=normalize(parse(input.insert)).body[0];
const matches=entry.body.body.map((n,i)=>JSON.stringify(n)===JSON.stringify(expected)?i:-1).filter(i=>i>=0);
if(matches.length!==1||matches[0]!==entry.body.body.length-2)throw Error("entry_position");
const preceding=entry.body.body[matches[0]-1];
if(preceding.type!=="IfStatement"||preceding.consequent.type!=="ExpressionStatement"||
   preceding.consequent.expression.type!=="AwaitExpression"||
   preceding.consequent.expression.argument.callee.name!=="pollController")
  throw Error("final_poll_precedes_phase");
entry.body.body.splice(matches[0],1);
if(JSON.stringify(candidate)!==JSON.stringify(baseline))throw Error("outside_branch_changed");
const hash=v=>crypto.createHash("sha256").update(JSON.stringify(v)).digest("hex");
console.log(JSON.stringify({byte_and_ast_reversal:true,whole_ast_equal_outside_insert:true,
  branch_after_final_poll:true,added_if_statements:1,added_assignment_statements:1,
  added_store_calls:1,baseline_normalized_ast_sha256:hash(baseline),
  restored_normalized_ast_sha256:hash(restored),
  top_level_functions:baseline.body.filter(n=>n.type==="FunctionDeclaration").length,
  cell_evaluated:false}));
```

Actual fixed static result:

```json
{
  "ast": {
    "added_assignment_statements": 1,
    "added_if_statements": 1,
    "added_store_calls": 1,
    "baseline_normalized_ast_sha256": "103b85f3ea8e2b805af3afb031b56ac5378595130f9f8bac38d43df7695d472a",
    "branch_after_final_poll": true,
    "byte_and_ast_reversal": true,
    "cell_evaluated": false,
    "restored_normalized_ast_sha256": "103b85f3ea8e2b805af3afb031b56ac5378595130f9f8bac38d43df7695d472a",
    "top_level_functions": 13,
    "whole_ast_equal_outside_insert": true
  },
  "ast_parser_exit": 0,
  "candidate_bytes": 31581,
  "candidate_evaluated": false,
  "candidate_lines": 467,
  "candidate_sha256": "d727878009b955e6c6e309ffa90fe30c3644089b4a0fb7ebbe4ebd7264fd3d9d",
  "node_syntax_exit": 0,
  "old_bytes": 31461,
  "old_lines": 463,
  "old_sha256": "eff0cdad3981ce0ad57d355203b082e21371553408444bf46a3b4ecfcf5fea0a",
  "parser_driver_bytes": 2160,
  "parser_driver_sha256": "c7d3d95eef92dee74d3cc3fb189d002e7d616c42c0af8836d0baaeaafc90bfff",
  "prefix_bytes": 635391,
  "prefix_sha256": "c3d7d338676cc1156a9958253c2b51eab823eb25b7478c9ec1b0d4fba37c91a8"
}
```

### Dated published descriptor-memory receipt and remaining prerequisites

Read the complete5638-byte immutable root receipt:
35c3fd3007c52d8142c7bee1d69aee127cc42a95:
docs/roadmap/evidence/wave30-d01-descriptor-memory-0ec651d.json,
SHA2565df541995dd1f74bff9c91104400140040d986a98a9f13f8688fbcb82cc4782a.
This updates the prior phase's dated pending/UNEXECUTED observation without
rewriting it. Root executed ONE exact payload
b10cf8376edbb80b00c051305f31643b04714494260c4a49576a354361048779:
128/128 PASS (50 observer plus78 legacy), child exit0/reaped, spawn_count1,
first_failure null, cleanup_failures empty, child elapsed0.101654s and
controller0.201888s/within35 true. The actual1398-byte stdout SHA256 is
006d33cc7314d7c483f99dcb8ed59d3b6b537571b5908ee4178d14c21094024f;
stderr0. I read that published receipt; I did not independently run the payload.

That result covers selected helper definitions and controlled Python memory
sinks/native Python frame/traceback objects. It does not cover this corrected
cell, actual browser/preparation/cleanup, OIDC crypto or a confidential
application journey. Helper/module main and server constructor were not
invoked. No provider/network/listener/CLI/Driver/browser/Cargo execution or
full D01/D05 gate credit follows. All older failed/unexecuted outcomes remain
dated and intact; historic cause/sender/first cleanup and whole60 remain unknown.

Exact preparation adapter, private-input transfer and unknown/partial-ownership
cleanup are still separate missing source prerequisites. No MCP response field,
returned ownership, session-end guarantee or no-PID cleanup is assumed or
implemented here. The four-line phase transition does not supply any of those
bodies. The published descriptor-memory PASS does not authorize alignment,
source materialization or runtime. Root must review this immutable correction
and the missing source before any separate fixture release; entire fixture
remains HELD. Root retains all original gate/status/integration decisions.

Before this append, source/hash/mode checks verified local helper470 remains
32723B/7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0
and all12 older deployment-private metadata records retain exact bytes/hash/
mode0600. The complete2e29 prefix, older500b/c88 failures, unknown sender,
provider lost values/wrong report,61/76 failed and78-case memory histories,
cleanup18-case receipt and all prior source/design outcomes are preserved.
No guide/product/D05 artifact, helper/controller file, board/main/push/status,
new worker/task/worktree or other-worker contact occurs. RiWork Cua.ai Driver
MCP remains the sole future GUI provider; no GUI tool was called in this phase.

Final append/archive/hash/AST/prefix/scope/whitespace/new-link checks and one
local report-only commit are the handoff. The global documentation checker has
a known false link in protected-prefix line2813, documented in2e29; no permission
is inferred to rewrite that immutable prefix to make the global check pass.

Final actual checks: archived corrected cell and static checker are byte-exact;
fixed static JSON matches the retained result; four fence pairs are closed;
only this report differs; new appendix link scan and git diff --check passed.
python3 scripts/check-docs.py exited1 with the same known missing-target match
from immutable prior-prefix line2813 (SimpleNamespace(demo=d). No new appendix
link error exists and that protected source archive was not altered. The
expected global docs failure and initial BigInt serializer failure remain
reported, without being relabelled as candidate/runtime failures.


### Root review of the successful-entry phase correction

Root read the complete c5d4d17 appendix and verified the sole 120-byte phase
block reverses the full 31,581-byte candidate to the prior 31,461-byte cell.
It runs only after successful final entry polling and adds no tool call,
deadline reset or failure-path change. All published report bytes are retained,
followed by the exact author append. This is accepted as design evidence only.
Independent review of the whole candidate and the missing preparation/input
adapters remain pending; no changed cell or browser fixture was executed.


## Root independent controller-boundary disposition (2026-10-03)

Root read the complete [independent report](local-wave30-d01-controller-boundary-independent-review.md) at author309ee22. It finds four source paths in the archived2e29 cell: stored entry phase, premature private-password clearing, successful helper completion before the protected-page snapshot, and unretained partial controller events across invocations. The separately reviewed c5d4 phase-only design addresses only the first. Root reserves review/correction of the remaining three plus the missing exact preparation/private-input/partial-ownership body before any real fixture release. These are source findings, not retrospective attribution for c88 or the earlier Authorization failures. Actual128-case helper memory evidence supplies no changed controller/cell/GUI journey proof. Runtime remains HELD; no status changes.
