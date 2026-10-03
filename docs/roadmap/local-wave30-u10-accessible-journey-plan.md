# U10 original accessible journey scope: source audit and proposed local seam

2026-10-03. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original U10
`934531ec-3a80-497c-b931-0d86b298ead2`. Reservation
`wave30_U10_accessible_journey_original_scope_source_audit` owns only this new
report. Existing supporting WT `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`, shell
`2173637e-bdb3-4eba-a128-178f57b41a34`, entry
`472a2eb481fc710de0702c7169e7f93665af3484`. The supporting branch is deliberately
unaligned. Current product/fixture reads below use immutable published
`544d1340b80cd3e040dc13142cdcbc1d75fea4cb`, not this branch's HEAD.

One reachable local feedback omission remains in the interaction sign-in form:
submitting missing username/password focuses its alert but does not mark the
missing fields invalid or associate those fields with the error. The workspace
and administrator confirmation paths already implement that feedback. The next
proposed slice adds it to the shared interaction form and checks one local
rejection path. This is source-derived DOM behavior, not an observed failure of
a screen reader, a diagnosis of a past browser failure, or U10 completion.

No product, UI, test, helper, existing guide, workflow, task, assignment, or status
is changed. No runtime or slot was acquired/released. S02's root-disposed DONE
scope stays closed. A09 ARM `37101183416` owns validation/Cargo according to the
assignment; this report did not query that run. D01 confidential-demo execution
and I02 accepted source-stage transport remain separate evidence and ownership.

## Original row and dated state provenance

The complete JSON export was parsed and the complete exact-UUID row read from
the project's `planning/current-tasks.json` under
`/Users/dominik/.local/share/riwork/orchestrators/projects/891e7443-8dac-4c1b-897f-9e53cb59c7ee/`.
Export: 244354 bytes, SHA-256
`0b38ca3eb8810a4628436eb5083f6329de848f3f4ddaa972c850601ad45c0835`,
mtime `2026-10-03T05:51:05.061711341Z`. This is the local dated export, not a
fresh project query. The row remains `todo`, primary WT
`cef37394-7d96-42ef-93d3-003ef3a4420f`, created `1790534560`, updated `1790893653`.
The older scheduling paragraph authorizes no launch; the current user message
authorizes this supporting static audit only, with the original assignment intact.

Exact acceptance: **Test complete journeys with keyboard navigation, assistive
technology, small screens, and supported authenticators.** Prerequisites U01–U09;
workstream Complete browser account experience. Exact goal: **Ordinary users
never need the terminal to manage their accounts.** Exact completion gate:
**A nontechnical user can accept an invitation, enroll a passkey, access an
application, manage factors and sessions, and recover access entirely in the
browser.** Both editions retain shared identity, authorization, revocation and
credential-protection semantics. The row expressly disallows declaring
implementation complete from documentation or a worker report alone.

The historical closure audit describes U10 as then `in_progress` and recommends
`todo` after the user cancelled accessibility testing. That is a dated state,
consistent with the later export, not a new status decision. This reservation
does not restart desktop/accessibility execution. Any later desktop work must
use RiWork's Cua.ai Driver, read its descriptions/current state first, and be
separately authorized by root. No desktop was needed here.

## Current supported journey bodies

The pinned source supplies browser account journeys. An older coverage inventory
that says later account pages do not exist cannot replace these actual bodies.
Presence, role names and control flow are source findings; successful rendering,
spoken announcements and independent human completion need execution evidence.

| Journey | Pinned body read and concrete UI contract | Boundary preserved |
| --- | --- | --- |
| Workspace and application launch | Full `index.html` and `app.js`: skip link, labelled search/category, navigation `aria-current`, pressed view/favourite buttons, named new-tab launch links, polite status, focus restoration after list replacement; responsive single-column/sidebar rules read in `app.css`. | Only the exact local policy-checked `apps/launch?client_id=…` route is actionable. Metadata uses text nodes. Password/passkey browser sign-in exists; terminal approval is an optional alternative, not proof of the ordinary-user gate. |
| Initial setup | Full `setup.html`/`setup.js`: labelled proof/username/display/password/confirmation, password/passkey radio choice, live/focused progress and explicit cancellation; passkey-only setup verifies primary then distinct backup credentials. | Ownership proof is memory/form data, not URL/storage; completion requires separate sign-in. Distinct credential IDs do not prove independent synced-device storage. Operator setup is distinct from an ordinary user's invitation. |
| Administrator and invitation sender | Full `admin.html`; complete JS bodies for shared form/error lifecycle, navigation rendering, confirmation/secret dialogs, invitation reissue and new-person setup, including the two-credential completion path. Labels, legends, hash navigation, focused headings/errors, explicit confirmation and return-to-opener focus are present. | Capability presentation is not authority. Ordinary invitations omit administrator roles. Mail-unconfigured installations hide/refuse invitations. Revision/idempotency/receipt and one-time-secret clearing behavior remain untouched. This is selected body coverage, not a claim to have reviewed every 2223-line admin feature. |
| Invitation recipient, verification and reset | Full `account.html`/`account.js`: labelled fields, invitation passkey choice when usable, focused completion/error/fallback, reset availability and nonenumerating request feedback. Proof moves from fragment into memory and is removed from location before actions. | Password/passkey acceptance requires a later sign-in; GET does not consume proof. Expired/revoked/used links stop the form. Reset retains factors; unavailable email, directory/passwordless accounts and loss of all factors do not silently obtain recovery authority. |
| Application interaction | Full `signin.html`/`signin.js`: labelled username/password/OTP, focused screen headings and alerts, account/reason/scope text, Allow/Continue/Deny, remembered-consent checkbox, explicit switch/cancel/logout; poll/generation state avoids stale actions. | Resume navigation is restricted to this interaction's own same-origin path. Server session reference and browser binding remain authoritative. Consent/logout click arming and request-in-flight suppression remain. OIDC is shared; SAML routes compile only for Platform. |
| Native passkeys, passwords, TOTP and recovery codes | Full `auth.js` and `app.js`: actual `navigator.credentials.get/create` delivery with AbortSignal, named passkey rows/rename/removal confirmation, password-change form, QR with text-key alternative, focused enrollment error and code-saving dialog. | Credential finish POSTs are not retried. Cancelled prompts retain options for a fresh gesture; explicit cancel consumes pending ceremony. Account/dialog generation and expected user bind changes; finishing mutations cannot be cancelled as though unsubmitted. Server enforces fresh own-browser/MFA authority and epoch revocation. Recovery codes are displayed once and cleared on close. |
| Sessions and consent | Full `self_service/security.html`/`security.js` and entire 104-line Rust HTTP module: labelled session/consent actions, recent-browser reauthentication, focused invalid/password error, bound expected user/session and guarded explicit confirmations. | A terminal-derived browser must establish its own authority. Selected/revoke-all/withdraw write guards and server binding are unchanged. Read/reauth definitions are not proof that all destructive keyboard/assistive actions have run. |
| Device approval | Full `device.html`/`device.js`: named code/sign-in/review/finished screens, focused heading/error, registered app/account/scopes, approval/rejection controls and polite announcement. | A code lookup is not a decision. Approval requires validated review/session reference and current permission; guard/in-flight/idempotency remain. It is a supported auxiliary UI, not evidence about who sent a historical D01 request. |

The actual Rust delivery/guard bodies were read, rather than inferred from a UI:
`src/portal/http.rs` password/reset handlers, `portal_html`, `browser_write_guard_for`
and passkey start/finish/cancel/register/rename/remove handlers; and
`src/api/interaction.rs:1–231` including `interaction_page`, state, password,
passkey start/finish and exact binding-cookie selection. `portal_html` sends
self-only script/style/connect, no inline execution, frame denial, no-referrer,
self-only public-key credential permissions, and COOP for portal pages.
Interaction pages deliberately omit COOP for RP popups. The write guard requires
exact issuer Origin, one `X-Riauth-Portal: 1`, and same-origin Fetch Metadata when
present. Native credential JSON is decoded into WebAuthn types; core verification
is not replaced by UI assertions. The proposal changes none of these Rust bodies.

The read CSS has visible-focus/invalid-field/error outlines, 44px field minimums,
420/760px responsive changes, wrapping passkey/dialog controls, reduced-motion and
forced-colour rules. These source rules and automated bounds do not establish
contrast in every OS, real browser zoom, a mobile virtual keyboard or spoken focus.

## Exact existing fixture scope and evidence distinctions

Full fixture bodies read: `accessibility-journeys.spec.js` (297 lines),
`signin.spec.js` (593), `portal.spec.js` (51), `setup.spec.js` (181),
`admin.spec.js` (206), `invitation-password.spec.js` (312),
`invitation-passkey.spec.js` (355), `invitation-passkey-shim.spec.js` (330),
`authenticator-recovery.spec.js` (312), `password-reset-replay.spec.js` (347),
`multi-authenticator.spec.js` (569), `passkey-rename.spec.js` (394),
`passkey-revocation.spec.js` (169), plus full `fixture.js`/Playwright config.
Bodies were read in bounded slices; truncated initial displays were followed by
the omitted slices. None of their imports, tests, helper functions or browser
callbacks ran. The 23-file Acorn pass below is a syntax check, not a browser result.

| Existing fixture | Assertions defined in source | What the definition cannot establish |
| --- | --- | --- |
| `accessibility-journeys` | Eight named tests: device-code rejection; mail-unavailable recovery; session reauth; security-dialog close; 320px signed-in navigation; missing workspace fields; missing administrator confirmation; Chromium virtual-passkey cancellation. Axe A/AA tags, focus, visible outline/control bounds, 320/768/1440 reflow and doubled DOM font sizes appear on selected screens. | Not eight complete keyboard journeys. Some setup steps use fill/click or programmatic focus, terminal approval uses fixture HTTP authority, and Firefox/WebKit skip the CDP case. Doubled fonts are not OS/browser zoom. |
| `signin`, `portal` | Keyboard password/TOTP; focused wrong-code errors; consent/account/logout; code/state/issuer/PKCE token exchange; zero/double request and click-arming oracles; app catalogue/search/reflow/offline/logout. Shim enrollment/passwordless and Chromium CDP verification/cancel coverage. | RP callback stub is not the D01 confidential app or a live external app. CDP has automatic presence and shim sets UV itself. Terminal-approval alternatives do not satisfy ordinary-user no-terminal completion. |
| `setup`, `admin` | Ownership proof rejection/single use/expiry and later cookie sign-in; admin application setup, secret erasure, session/account changes, stalled refresh and wizard field retention. | Definitions are not a new run, accessibility assessment of every admin form, or human completion. Setup proof is operator input, not an end-user recovery credential. |
| Invitation password/CDP/shim | Exact expired/used rejection, no session on completion, later factor sign-in/sign-out, request counts and 390×844 control bounds; password path has zero passkey POSTs. Password/shim span three configured engines; CDP is Chromium-only. | Loopback SMTP capture is not an external mailbox. CSS phone-sized viewport is not iOS/Android. Shim is not native WebAuthn or hardware; CDP virtual authenticator is not a physical/synced/phone authenticator. |
| Authenticator recovery/reset | Keyboard enrollment errors, one-use recovery-code sign-in/replay/counts, another browser's epoch revocation; email reset nonenumeration, empty/mismatch/current-password rejection, single-use link and old-password refusal. | TOTP is computed by the test from a page key, not a phone app. Reset fixture has no enrolled factor, so kept-factor page copy alone does not prove retained factors in that journey. Loss of all factors remains an operator/help-desk boundary. |
| Rename/revocation/two authenticators | Keyboard name validation/trim, same credential preserved by rename; removed credential refused, other session epoch revoked, password/remaining key still usable, TOTP remains enabled; 390×844 fit/control checks. | These are two simulated credential stores, not independent physical or synced devices. Some assertions scroll controls programmatically. Stable session cookies do not imply live authority; the HTTP status checks are material. |

Config declares Desktop Chrome/Firefox/Safari engines, one worker, no retries;
Playwright `1.63.0`, axe `4.13.0`. The Firefox test-only COOP automation preference
does not change server security headers or establish real Firefox deployment
compatibility. Fixture startup defaults to 120000ms (validated range 1..300000),
with 5000ms beforeAll allowance. Tests retain their own interaction deadlines.
The fixture builds/installs are separate prerequisites; no fixture build occurred.
`stopFixture` waits for its owned child's exit without an outer deadline in this
helper: a future run still requires a separately reviewed finite supervisor.

### Historical reported execution, kept at its actual revision

The following are historical accepted report/commit claims read at immutable
objects. Raw browser traces/logs and private credentials were not opened. The
audit does not independently rerun or re-grade those historical executions.

| Historical source/accepted pin | Read reported result and attribution | Limit |
| --- | --- | --- |
| `0d590fa316a494307650a23751317c3e6bcb16c4` / `099c19047ecde2ab990f7c7e9dada604981b9ff8` | Root closure audit ledger slice 351 reports focused 320px empty-admin-password path Chromium/Firefox/WebKit 3/3; alert focus, invalid/description and no mutation on cancelled dialog. Author commit states the exact original omission/fix. | Current full a11y file is later: the subsequent workspace test was added. No real screen reader/physical mobile credit. |
| `e3036220e37ad99140a5a681e0da4780a7308d74` / `5892563fddecaf5446c4d7219a0cacb4e236e551` | Root closure audit slice 360 reports focused 320px empty-workspace path 3/3, invalid/association and input clear. Current a11y file is byte-identical to this accepted file. | File equality does not make the old run a current fixture/binary run or a full assistive journey. |
| `94764690429918a7a8189322c78d522817f0b544` / `bd35698b08394d712a9d71e44ca084407aa476e5` | Author reports shim passkey-revocation 3/3, no skips; enrollment/removal revoke other browser and removed credential is refused. | 390×844 CSS viewport; no physical/synced/mobile authenticator. |
| `3b2c2d0a663a47f45d4e1504bd47d11d8b9fb207` / `9359b80ff2ab2c27b9c8791b4db166d491174530` | Author reports authenticator recovery Chromium 8.2s, Firefox 9.7s, WebKit 8.8s, no skips; codes drop 10→9→8 and replay refuses. | Page key plus test-computed TOTP; not a phone app. |
| `6bceffb1122d282b7ef1fba327d0f025db21b003` / `d08f8e734cc22cfb3937b38910d4336ab5327677` | Author reports reset replay Chromium 6.2s/Firefox 6.1s/WebKit 7.1s, no skips; separate mail-off alert 829ms/1.2s/808ms. | Fixture mailbox; recovery account has no factor. Unavailable-mail refusal is not successful recovery. |
| `d1f675d7410e05475f9acf2a90ac27604dfb60de` / `bb55a50c7bc4fca79b9f4e34b2deb2efbc09d22c` | Author reports rename 3.9s/4.8s/5.0s, no skips; keyboard validation, trimmed name and credential/session preservation. | Chromium uses one backwards Tab when forward traversal skips the name. This is a recorded test mechanic, not proof of spoken-reader usability. |
| `ec5f0140388b4a5a4e92d08ea2bb9c4eb2e10532` / `d039306bae4a30f763102802686a85beb8b6c54a` | Author reports two-authenticator 10.0s/11.4s/11.1s, no skips; first credential refused after removal, second and TOTP survive. | Two shims; no hardware/mobile/assistive evidence. |
| `b77e6370504f13fd5dcf87f46aad6a8121299542` / `a72a08630570f6830f4987919d5173f0ce1f40e3` | Author reports CDP invitation Chromium 1/1 in 2.4s; Firefox and WebKit explicitly skipped. | Not a three-engine native-authenticator pass. |
| `a4d0a1ff8e84e1a04322c1b5648b64998723a241` / `58e5ef3c0b15a8ab2e76e423bb6c592982041dc6` | Author reports password invitation 2.0s/3.2s/2.4s, three passes in 26.6s; keyboard acceptance, expiry/replay, later login/logout, passkey choice unused. D05 accepted evidence separately repeats 3/3. | Loopback mail; no authenticator credit. Current file differs only in introductory comments, as exact diff showed. |
| `7187286fabd5b521dbd664be1d0e2c79f9c76dc5` / `03124272a6cfe14ac48227c504cc896c2f11d9aa` | Author reports shim invitation 1.6s/2.1s/1.8s, three passes in 23.1s. D05 accepted evidence repeats 3/3 and self-set UV limitation. | Current file differs only in introductory CI/limit comments. Still no native/hardware/screen-reader credit. |

Root's entire U10 entry was read in pinned
`docs/roadmap/local-wave28-task-closure-audit.json:464–538` at published 544d.
Its referenced older `32770ab73270901ec94a2d1249cbd8bb6052a415` sign-in and portal
fixture blobs are respectively `50bf9a0623d4c72211d14ce0cccf1ac8afbd4dfa` and
`331822cca062e669e0240f4a935531c204f65d48`, equal to the current fixture bytes.
That pin's complete integration verdict was read, including explicit cancelled
accessibility testing. The later audit file did not yet exist at 32770: that
initial lookup failed and was corrected to the actual published audit, without
inventing an earlier artifact. D05 evidence at 544d, relevant source-only and
Usability sections, distinguishes ancestor-local headless evidence from missing
320/768/1440/200%-text/reader/device/user results on its older selected HEAD.
Those older missing-measurement sentences are dated, not claims that current
source lacks responsive fixtures. Historical held S02/O06 text is not reopened.

D05's Usability table also retains a different kind of historical observation:
U01's 2026-09-28 evidence, base `14de533` on `roadmap/integration-accepted`,
reports a Cua.ai Driver session that reached a backup Touch ID prompt. Backup
completion is expressly unclaimed. The table names `passkey-prompt.png` and
`backup-prompt.png`; neither those images nor the underlying private fixture was
read here. A reported native prompt is distinct from a simulated authenticator,
but cannot establish completed backup enrollment, a supported-device matrix,
screen-reader behavior, or the independent ordinary-user journey. It remains a
bounded historical report claim, not new physical-device execution or a reason
to describe the full native journey as passed.

## One smallest proposed follow-up reservation

Propose only `src/portal/signin.js` three hunks below and one new focused test in
existing `tools/browser/signin.spec.js`. These paths are **not reserved or edited
by this report**. No HTML, CSS, guide, protocol, route, core, manifest or helper
change is needed for this concrete missing-field feedback. Root must review the
exact source/test design and separately reserve any implementation/runtime.

Witness: current `signin.html:32,36–39` defines a focused alert, `novalidate` form,
required labelled username/password and an OTP hint. Current `signin.js:313`
rejects missing username or required password before `act`/POST, but only clears
and focuses an alert. `clearError:65–68` and input handler at 323 remove invalid
flags without an error association because none was added. In contrast current
`app.js:382–405` explicitly marks and associates each missing workspace field.
The proposed attributes activate existing invalid-field CSS and provide an error
description when the user returns to a missing field. Speech quality is untested.

The exact predicate, acting guard, return, error text/focus and no-POST branch
remain. Only missing fields are marked; a supplied/pinned username and an optional
password are not newly invalid. Error association clears on input and when the
error resets. OTP's existing `signin-otp-hint`, required-OTP rejection, parser,
credential clearing timing, generic server refusals and all passkey/consent/logout
branches remain byte-exact. This minimal slice does not attempt every other form
or certify WCAG compliance.

Baseline 23197 bytes/373 LF, SHA-256
`3d5ee4cfc1b3a88b3d6fa46d064df36b7f1a227d03a1a557830eac4528ccff22`;
candidate 23771 bytes/384 LF, SHA-256
`d42f6ead02c1c27f55778b75b00d7ede654bbe669004aa07c964088124e0797b`.
The following reconstructs the complete candidate from the fixed baseline; LF
after each fenced line belongs to the diff, not an extra source newline.

```diff
--- a/src/portal/signin.js
+++ b/src/portal/signin.js
@@ -64,7 +64,10 @@
   }
   function clearError(id) {
     $(id).hidden = true; $(id).replaceChildren();
-    if (id === "signin-error") for (const field of FIELDS) $(field).removeAttribute("aria-invalid");
+    if (id === "signin-error") {
+      for (const field of FIELDS) $(field).removeAttribute("aria-invalid");
+      for (const field of ["signin-username", "signin-password"]) $(field).removeAttribute("aria-describedby");
+    }
   }
   function showError(id, text, portal = false) {
     $(id).replaceChildren(text);
@@ -310,7 +313,12 @@
     event.preventDefault();
     if (acting) return;
     const username = $("signin-username").value.trim(), password = $("signin-password").value, otp = code($("signin-otp").value);
-    if (!username || ($("signin-password").required && !password)) { clearError("signin-error"); showError("signin-error", "Enter your username and password."); return; }
+    if (!username || ($("signin-password").required && !password)) {
+      clearError("signin-error");
+      if (!username) { $("signin-username").setAttribute("aria-invalid", "true"); $("signin-username").setAttribute("aria-describedby", "signin-error"); }
+      if ($("signin-password").required && !password) { $("signin-password").setAttribute("aria-invalid", "true"); $("signin-password").setAttribute("aria-describedby", "signin-error"); }
+      showError("signin-error", "Enter your username and password."); return;
+    }
     if ($("signin-otp").required && !otp) { clearError("signin-error"); showError("signin-error", state?.requirements?.configured_totp ? "Enter your current authenticator code." : "Enter your authenticator or recovery code, or sign in with a passkey."); return; }
     $("signin-password").value = "";
     act($("signin-submit"), "signin-error", async () => {
@@ -320,7 +328,10 @@
       return next;
     }, "Couldn't sign in. Try again.");
   });
-  for (const field of FIELDS) $(field).addEventListener("input", () => $(field).removeAttribute("aria-invalid"));
+  for (const field of FIELDS) $(field).addEventListener("input", () => {
+    $(field).removeAttribute("aria-invalid");
+    if (field !== "signin-otp") $(field).removeAttribute("aria-describedby");
+  });
   $("signin-passkey").addEventListener("click", () => {
     if (acting) return;
     const attempt = ++passkeyAttempt;
```

### Exact focused test design, unexecuted

Insert the complete next fence plus one blank LF immediately before the unique
line `// Double-click-jacking: a cross-site page opens a popup over its own tab, moves that tab to`
in the pinned 33817-byte `signin.spec.js`. Original SHA-256
`d24ed29b9dcad2d143999c0df2fd328da8a62239bec9f3f8a0124df402518bb5`;
candidate 36605 bytes, SHA-256
`2cc417e36f76e4455fa78488e2019f19abb05999323b8e7cd8193b0d1ec49c88`.
Addition including that blank LF: 2788 bytes, SHA-256
`89304ee4f7c4825a4c47ebe3e4cb6cdcdcbd118f5852ab085585622b075e23d8`.
Existing imports, helper definitions, fixtures and every existing assertion stay
unchanged. One test contains three meaningful rejection states: both fields
missing, password alone missing, username alone missing; input correction is
checked between them. Only local synthetic unsubmitted text is typed. No native
authenticator is installed/invoked, credential accepted, application decision
made, or mutable account/factor/session request sent by this design. The explicit
zero-POST and unauthenticated status oracles must actually pass in a later run.

```js
test('empty interaction credentials identify missing fields without a request', async ({ page, browserName }) => {
  await page.setViewportSize({ width: 320, height: 640 });
  const request = authorization(fixture.clients.consent, { prompt: 'consent' });
  let posts = 0;
  page.on('request', (r) => { if (r.method() === 'POST') posts += 1; });
  await page.goto(request.url);
  await screen(page, 'authenticate');
  const username = page.locator('#signin-username');
  const password = page.locator('#signin-password');
  const otp = page.locator('#signin-otp');
  const error = page.locator('#signin-error');
  await tabTo(page, browserName, 'signin-username');
  await page.keyboard.press('Enter');
  await expect(error).toBeFocused();
  await expect(error).toHaveText('Enter your username and password.');
  for (const field of [username, password]) {
    await expect(field).toHaveAttribute('aria-invalid', 'true');
    await expect(field).toHaveAttribute('aria-describedby', 'signin-error');
  }
  await expect(otp).not.toHaveAttribute('aria-invalid');
  await expect(otp).toHaveAttribute('aria-describedby', 'signin-otp-hint');
  expect(await fitsWidth(page)).toBe(true);
  await axe(page);
  expect(posts).toBe(0);
  await tabTo(page, browserName, 'signin-username');
  await page.keyboard.type('local-only-user');
  await expect(username).not.toHaveAttribute('aria-invalid');
  await expect(username).not.toHaveAttribute('aria-describedby');
  await expect(password).toHaveAttribute('aria-invalid', 'true');
  await page.keyboard.press('Enter');
  await expect(error).toBeFocused();
  await expect(username).not.toHaveAttribute('aria-invalid');
  await expect(password).toHaveAttribute('aria-describedby', 'signin-error');
  expect(posts).toBe(0);
  await tabTo(page, browserName, 'signin-password');
  await page.keyboard.type('local-only-not-submitted');
  await expect(password).not.toHaveAttribute('aria-invalid');
  await expect(password).not.toHaveAttribute('aria-describedby');
  await tabTo(page, browserName, 'signin-username');
  await page.keyboard.press('ControlOrMeta+A');
  await page.keyboard.press('Backspace');
  await page.keyboard.press('Enter');
  await expect(error).toBeFocused();
  await expect(username).toHaveAttribute('aria-invalid', 'true');
  await expect(username).toHaveAttribute('aria-describedby', 'signin-error');
  await expect(password).not.toHaveAttribute('aria-invalid');
  await expect(password).not.toHaveAttribute('aria-describedby');
  await expect(otp).toHaveAttribute('aria-describedby', 'signin-otp-hint');
  await screen(page, 'authenticate');
  expect(await fitsWidth(page)).toBe(true);
  expect(await page.evaluate((url) => fetch(url).then((r) => r.status), `${fixture.issuer}/api/portal`)).toBe(401);
  expect(posts).toBe(0);
});
```

One prospective first filter, from `tools/browser`, after root source reservation,
reviewed finite supervision and separately verified pinned fixture/browser assets:

```sh
CARGO_TARGET_DIR="$PWD/../../target" ./node_modules/.bin/playwright test signin.spec.js --project=chromium --workers=1 --retries=0 --grep '^empty interaction credentials identify missing fields without a request$' --reporter=line
```

This is not executable readiness or an authorization. Installed metadata checks
found no own-WT `tools/browser/node_modules/@playwright/test/cli.js`; no dependency
installation/download/build was attempted. A current fixture with exact product,
toolchain/features/binary provenance and a private target is a prerequisite, not
old-cache/current-source credit. Root would separately reserve any Cargo/native
preparation, finite browser supervision, owned cleanup and resource preflight;
the 8GiB floor cannot be assumed met by this report. No broad matrix, retries,
baseline campaign, deadline expansion or D01 fixture duplication is proposed.

## Actual static proofs and original acceptance still open

Acorn from installed Node's bundled internal parser parsed 23 current JS assets/
fixtures as source DATA, exit 0. No VM/eval/import of reviewed files or their
functions/callbacks ran. Both exact source/test candidates parsed, exit 0.
Three unique production textual replacements reversed to the complete original
23197 bytes. Of 62 function nodes, the enclosing IIFE, `clearError`, submit
callback and input callback are the only byte-changed functions; the other 58
function-node source spans remain exact. Replacing only those three leaf AST
nodes with their original counterparts gives complete normalized AST equality,
including all predicate/credential/poll/passkey/consent/navigation nodes.
Removing exactly the one proposed test call gives whole original fixture AST and
whole-byte equality. This is structural preservation, not executed behavior.

Source identities and actual static documentation/scope checks are recorded below.
Pre-report `check-docs.py` exit 0, `check-repo-hygiene.py` exit 0 (1061 tracked
files), `git diff --check` exit 0; tracked/index/untracked were clean at entry.
The early wrong-pin lookup above is a disclosed source lookup error, not a test
failure or checker bug. No product/browser test ran and no universal compatibility
or compliance conclusion follows from static success.

The original acceptance is incomplete. The already recorded virtual/keyboard/
viewport slices cannot establish spoken assistive technology, actual mobile OS
behavior, physical/synced/phone authenticators, or an independent nontechnical
person completing the full invitation→passkey→application→factor/session→recovery
journey. The next local missing-field correction would close only its DOM
feedback omission. It cannot close those gates.

A later original-gate decision needs a consenting nontechnical user plus a declared
supported browser/device/assistive setup, a browser-enabled issuer whose native
credential origin/RP ID matches the device, privately provisioned invitation and
eligible recovery delivery, an actual supported authenticator with a retained
recovery method, and an application whose result can be checked. For a spoken
reader claim the actual reader/focus/error announcements must be observed; a
phone-sized CSS viewport cannot substitute for a real mobile OS claim. Loss of
every factor or unavailable delivery must retain the documented administrator/
offline recovery boundary, not gain a browser bypass. These are specific missing
inputs to the original gate, not a demand for every OS, screen reader, key vendor
or an external tenant campaign. No user/device availability was queried here.

Root owns source reservation, independent review, later execution and original
gate/status disposition. D01 and I02 retain their separately bounded evidence;
this report grants no new credit to them or any other closed/open row.

## Immutable read identities

All following identities are at fixed published 544d. This table identifies
bytes; the coverage descriptions above distinguish full body reads from selected
supporting Rust/admin/CSS/guide spans and syntax-only checks. It supplies no
execution credit. `B/LF` counts bytes and LF delimiters.

| Path | B/LF | SHA-256 |
| --- | --- | --- |
| `src/portal/index.html` | 21674/184 | `8659a411065f0b91197c7fa2aa911a5cb0e7e206193f5c03e8fef23f4e162c36` |
| `src/portal/app.js` | 63378/960 | `d44599e4c1c749f76ed973fe95774c6e079aa8306d9984f832878cbb5f5ec68c` |
| `src/portal/app.css` | 27971/22 | `fcc2105d9d35cdb636dda59821ea84eb94dcb3b3ca5684aec272d4b380755cf7` |
| `src/portal/auth.js` | 9235/175 | `c1a1aa440ca0144183019959b94bf950095309cbc414ec7bcffefbf48cf63ae3` |
| `src/portal/setup.html` | 4362/52 | `fb14403312f2de28cf06502c0ad1124fedbfdb881560964e64c3c04e74e75afe` |
| `src/portal/setup.js` | 7849/145 | `237877a077e6d52bd0d070ea98f38cbe3af27f7315dafeed0aa88f6006316936` |
| `src/portal/account.html` | 3355/46 | `aa80719083a36a55148c83615d3de6ac35f477e1d40f0cee7f65aa80df99d14b` |
| `src/portal/account.js` | 15804/272 | `3ca18c16e3ef6ad1ff1d0be438eadaf1071252ccb9c5bea5f0885a5603d772b3` |
| `src/portal/signin.html` | 6470/80 | `af5c163944bb397d6649c932fa91e749381772d307c685a95f53cd3f6dee5f44` |
| `src/portal/signin.js` | 23197/373 | `3d5ee4cfc1b3a88b3d6fa46d064df36b7f1a227d03a1a557830eac4528ccff22` |
| `src/portal/admin.html` | 9337/89 | `99d96476c57e74cce5395cde97618a1120db43b0af3d6ec7eab6d2952f37a1be` |
| `src/portal/admin.js` | 175308/2223 | `f56b09d1ea0fa2eaf9ce0b66387b10db2ee3b815e6ca7086b3fe09281db3a94f` |
| `src/portal/admin.css` | 17059/225 | `08ee74d5142d2f75cef261653ba51bb43a66bf804c922d24b1e114479160d412` |
| `src/portal/device.html` | 6282/87 | `d00237a12a5dd66fc4b451feb8fb87fa0aefc4ebf5b647f0ee5c6e25742d9063` |
| `src/portal/device.js` | 14557/316 | `e86c105ef05cc11e7eaa66b207a34ffd664adf4259e43522f80c00e5848b2fd8` |
| `src/portal/self_service/security.html` | 5211/67 | `7abf72c7828622ebdde48b7093638c891369ac65172c9584dd5c472e66093bd3` |
| `src/portal/self_service/security.js` | 14328/260 | `f4e9c56247a8f315ad5ec6b5c8931d61443e13f89d1e58e6ce32987b2596d38b` |
| `src/portal/self_service/security.css` | 3100/13 | `104cb9a1fe0d648c8a3150152c46d90f87d8460b88dfe802973edfccc2204e43` |
| `src/portal/http.rs` | 27412/847 | `5fc656f1655e92a61c953b479191d7177c9346266802b6601ad3b863908f8a77` |
| `src/api/interaction.rs` | 10192/311 | `d4ed262fac0bbd167c3dd999f260ce7b26e9c61bcb5072bb0b380028cc2b38b1` |
| `src/portal/self_service/http.rs` | 2945/104 | `310d820791bca3fa437b65aa4132492a74978a0f4366efaf9dc9367ba3655cab` |
| `tools/browser/accessibility-journeys.spec.js` | 14959/297 | `01b41cb562b8ca41bc16de1c59ef7d6f8f51c743e82d6b684bc5144e869c8dec` |
| `tools/browser/signin.spec.js` | 33817/593 | `d24ed29b9dcad2d143999c0df2fd328da8a62239bec9f3f8a0124df402518bb5` |
| `tools/browser/portal.spec.js` | 3124/51 | `6e2355aea435812b249197e51cdb8b94e51cbe25357927a79aa189acb993e24a` |
| `tools/browser/setup.spec.js` | 11201/181 | `dd934c78a76b2f6d1ab7803df74eda4cb485ac87a5c1fdc5ca5ec8386023169f` |
| `tools/browser/admin.spec.js` | 10603/206 | `7b5e4619c474c83afec71df3a22a08b96b79ac3f92cd6e47cf5c505510ec2952` |
| `tools/browser/invitation-password.spec.js` | 16293/312 | `b6ef8ea308494760886cc13972865de5b91ca9a5d1de8eac5c44e6e5019ba216` |
| `tools/browser/invitation-passkey.spec.js` | 19053/355 | `93b5c5c9d488858d9f76e80c3af14a9c042122c963b724dd91904d67c8721ca5` |
| `tools/browser/invitation-passkey-shim.spec.js` | 17812/330 | `212b00ea4be9e85d05dc4d6c3c80b6559a6b0904323894cab3f20734d3c4be7c` |
| `tools/browser/authenticator-recovery.spec.js` | 16772/312 | `df954147a4f4d3505ca0957fa35253f21a83bf1bfe0cc3e25607eada716ff1d9` |
| `tools/browser/password-reset-replay.spec.js` | 18482/347 | `d9fb0048c935385c226900f690717e137d397824a282209c1199e5cabe2223cd` |
| `tools/browser/multi-authenticator.spec.js` | 31081/569 | `137953b2d18440fb41d3570ff9d6bc98bad03c8af2fc8ffc408ffd95a2fe5720` |
| `tools/browser/passkey-rename.spec.js` | 20303/394 | `57fc51a84f8b0fa6a48891c064fcb4ff9dd17d50798212cff4da1af802125db6` |
| `tools/browser/passkey-revocation.spec.js` | 9049/169 | `b6061d4ac397eda9663d1dda69a3cef01f3280f9db1626da926c4e2ca0e345e6` |
| `tools/browser/fixture.js` | 4521/81 | `b886dac64616015a791f48e2784a6adf84733e7f2f5a1b2dabb2e68e3172bc8b` |
| `tools/browser/playwright.config.js` | 1330/19 | `24c63e0563b0f7e1333a7bea03445297f69c87519bc8af44c313dd84d880b5a1` |
| `tools/browser/package.json` | 210/7 | `105fbcbbde405cee41712021c586da014a41db28ba1c946820ffc6e7f4e6a78b` |
| `docs/PORTAL.md` | 43482/371 | `386fa4ca53f4607133b3f30071b7e2714ff4504cb0879b514fe2b52d86ddc015` |
| `docs/testing.md` | 20762/135 | `a925cc82b0e2595a3a79361b0645bcefc752ac8261904d291cc25e56a7d60a3e` |
| `docs/passkeys.md` | 12903/99 | `4d96608a6c1121cee131bd74f27be78dd42ee7ecfb7e7a67330513e679806b96` |
| `docs/roadmap/local-wave28-task-closure-audit.json` | 219710/3734 | `051a159610f904a6e91645bf06a059d3db8bcaec85f95e0e6c0551b233fe058b` |
| `docs/roadmap/d05-acceptance-evidence.md` | 153623/1699 | `87dc6e145f8db722f3e4c2acfa7ddc027cba4c86516acfb2252c928a92ac52f6` |
| `docs/roadmap/integration-verdict-2026-09-30.md` | 4838/64 | `dcbaf9f64712aca47ac5814f68bf849d0c398c04c4c9906097d8e65d70525932` |
| `CONTRIBUTING.md` | 3603/48 | `7e7dd7b756f734a8105cad5b96dffa8a51977c64f3ecd70b8182fa3de6ba6737` |
| `SECURITY.md` | 981/15 | `2556771d58d09a2a4754e7484645b7e948b84286ef0d21cc66169b920a31c4d8` |

The proposed product diff fence is 2388 bytes, SHA-256
`e99342043938bbcc652942d0eea17add35eeeaaf25e91c4fff7530edb7457a3b`.
Actual readback from this report independently reconstructs the full 23771-byte
candidate and 36605-byte test candidate with the hashes above. The test insertion
is exactly its readable fence plus the documented extra blank LF, not an
unreviewed test implementation. Both reverse to their original whole files.

Report-present docs checker: exit 0. The initial unstaged new-file
`git diff --no-index --check /dev/null <report>` returned 1 with no output (new
content differs from /dev/null); it reported no whitespace diagnostic. The staged
whitespace and hygiene checks below are the final checks for this actual new file.

Final report static checks: `python3 scripts/check-docs.py` exit 0;
`python3 scripts/check-repo-hygiene.py` exit 0 (1062 tracked files);
`git diff --cached --check` exit 0, no diagnostic. Staged scope is exactly one
addition, this report; no existing file or source delta. Whole report fence
readback, pinned candidate hashes, original test insertion inverse and source
AST preservation passed. No product, fixture, browser, assistive, physical-device,
helper, native, protocol, capacity or cleanup runtime was performed. No current
all-green, U10 DONE, or other-row disposition is inferred.
