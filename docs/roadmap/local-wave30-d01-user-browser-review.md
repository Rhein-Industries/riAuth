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
