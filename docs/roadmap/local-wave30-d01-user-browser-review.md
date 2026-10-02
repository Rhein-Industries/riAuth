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
