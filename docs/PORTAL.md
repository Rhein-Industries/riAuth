# My applications

Open `<issuer>/apps` in a browser. The server embeds the HTML, CSS, JavaScript and icon set in the Rust binary; no Node process, frontend build, CDN, remote fonts or remote app icons are needed. The issuer root also serves the portal to browsers while retaining its JSON response for API callers.

The portal includes a responsive application grid, list view, name/description/category search, category filters and favorites. Favorites and view preferences are saved locally per account and issuer path. Only application IDs and the view choice are stored; removing access also removes an app from the active favorite list. Storage is optional, so private browsing still works.

## Sign in

An existing browser session is recognized automatically. Its HttpOnly cookie is `__Host-riauth_sso` on an https issuer and `riauth_sso` on a loopback http issuer. A browser that has no such cookie gets a placeholder value when it opens the portal or an application's sign-in page; it signs nothing in, and lets two sign-ins started at the same time in two tabs end on one session. Otherwise the portal offers three ways to sign in:

- **Sign in with a passkey.** Shown when the browser supports WebAuthn in a secure context. The browser offers the passkeys it holds for this riAuth host; no username is typed. A passkey sign-in counts as MFA.
- **Username, password and "Authenticator or recovery code (if enabled)".** One form. Leave the code empty if the account has no authenticator app; a correct TOTP or recovery code makes the session MFA. Every failure, including a locked account, shows the same text, "Check your username, password and code. After several failed attempts, sign-in pauses for 15 minutes.", no sooner than one second after submitting. **Forgot your password?** below the form opens the [password reset page](lifecycle.md#change-or-reset-a-password-in-the-browser); application sign-in pages open it in a new tab.
- **Sign in with your terminal.** Sign in to the same issuer in your terminal, then run the approval command the browser shows:

  ```sh
  riauth --server https://id.example.com login alice
  riauth --server https://id.example.com portal approve ABCD-EFGH
  ```

  Review the account, server, code and the requesting browser (`requested_from`: IP address, User-Agent and time) before approving. The code expires after ten minutes. Approval requires authentication within five minutes; the CLI signs in again automatically once its session is four minutes old, and supports `--passkey`. `portal inspect CODE` and `portal deny CODE` are also available. Agents can configure the applications and policies but cannot approve sign-in as a user.

Session credentials never enter JavaScript or local storage. Every cookie-authenticated write requires the exact issuer `Origin`, the `X-Riauth-Portal: 1` header, a `Sec-Fetch-Site` that is absent or `same-origin`, and a JSON body. Terminal requests are bound to their originating browser by an HttpOnly cookie, consumed once, rate-limited and cancellable. Signed-in data is not cached.

### Session kinds and sign-out

| How the browser signed in | Session | What **Sign out** does |
| --- | --- | --- |
| Passkey or password in the browser | A **browser** session with no terminal credential | Ends this browser's session, its SSO grants and supported logout propagation. The terminal is not affected, and `riauth logout` does not end this session. |
| Terminal approval | The approving **terminal** session, shared with this browser | Ends that login session, **including its terminal credential** and SSO grants, and starts supported logout propagation, as before. |

`riauth session list` shows the `kind` of each session. On an application's sign-in page, **Use another account** is gentler: for a terminal-approved browser it only disconnects this browser and leaves the terminal signed in; for a browser session it is a full sign-out.

Signing in again in the same browser as the same user keeps the session (and the applications' session identifiers) and extends it. Signing in as a different user ends the previous browser session and queues its logout.

Buttons that act on a decision (**Sign out**, and on an application's page **Allow**, **Deny**, **Sign out**, **Stay signed in**, **Use another account** and **Cancel**) ignore clicks for half a second after their screen appears and after the window is shown or regains focus. A click meant for another window that just closed (double-click-jacking) therefore cannot sign the browser out or grant consent.

### Applications that need a passkey or code

Some applications require MFA. After a password-only sign-in the catalogue shows a notice instead of silently hiding them:

- If the account has TOTP or a passkey: "Some applications need your passkey or authenticator code." with **Sign in with your passkey**, which re-authenticates the same account.
- Otherwise: "Some applications need extra verification. Add a passkey or an authenticator app under Sign-in and security." with a button that opens that dialog.

## Sign-in and security

Signed-in users open **Sign-in and security** from the account menu. It has a **Password** and a **Passkeys** section.

### Password

**Change password** asks for the current password, a new password and its confirmation. The change keeps passkeys, the authenticator app and recovery codes, and **signs the account out everywhere**, including this browser, terminal sessions and application grants; the page returns to sign-in with "Password changed. Sign in with your new password."

- The form proves the current password again. A wrong one shows "Your current password is incorrect…" and counts toward sign-in's lockout (five failures in fifteen minutes pause password checks for fifteen minutes). New passwords follow the 12-character minimum and the [password history](enterprise/ENT-08.md).
- If the account has TOTP or a passkey, the session must be an MFA sign-in from the last five minutes; otherwise the dialog asks to confirm with **Use your passkey** or password plus code, then reopens the form.
- A browser signed in by terminal approval signs in here first, as for passkeys.
- An account whose password an imported directory manages sees "Your organization's directory manages your password. Change it there."; a passkey-only or upstream-only account has no password to change.

Forgotten passwords are reset from the sign-in page, not from this dialog. See [account lifecycle](lifecycle.md#change-or-reset-a-password-in-the-browser).

### Passkeys

The **Passkeys** section lists the account's passkeys (name and date added), adds another authenticator with a chosen name, renames a passkey, and removes one after confirmation. Closing the dialog or selecting Cancel before verification is submitted discards a pending enrollment without changing the account.

- Up to sixteen passkeys per account. The browser is asked for a discoverable (resident) passkey with user verification.
- Adding, renaming, or removing needs a sign-in within the last five minutes. If the account already has TOTP or a passkey, it also needs a session signed in with a passkey or code; after a password-only sign-in the dialog explains "Sign in with your passkey or authenticator code to change your password or passkeys." (or "…your passkeys." when the password can already be changed) and offers **Use your passkey** or password plus code.
- The first passkey of an account with no other factor needs only a recent sign-in.
- A browser signed in by terminal approval shares the terminal's session. Because such an approval can be phished, that browser cannot change its password, passkeys, authenticator app or recovery codes: the dialog says "This browser uses your terminal's sign-in. Sign in here to change your password or passkeys." and signing in there with the account's password (plus code) or passkey gives this browser a session of its own, leaving the terminal signed in. A user whose only passkeys are on other devices signs in here with a phone or security key (cross-device sign-in), or enrolls from the terminal with `riauth passkey enroll`.
- A passkey-only account cannot remove its last passkey. It can add a backup authenticator first. Renaming leaves the credential and active sessions intact.
- **Adding or removing a passkey signs the account out everywhere**, including terminal sessions and application grants. The page returns to sign-in with "Passkey added. Sign in with it to continue." or "Passkey removed. Sign in again."

### Authenticator app and recovery codes

The same dialog sets up, replaces and removes an authenticator app (TOTP) and creates recovery codes. Every change follows the passkey rules above: a sign-in within the last five minutes, a passkey or code sign-in once the account has any factor, and this browser's own session rather than a terminal approval. When a check fails, the dialog asks the user to confirm it's them and then retries the change.

- **Set up** shows a QR code of the `otpauth://` URI, the setup key as text with a copy button for manual entry, its parameters (time-based, 6 digits, 30 seconds, SHA1) and an **Open in an authenticator app on this device** link. Nothing changes until a current code from the app is entered: an unconfirmed setup is not a factor, and **Cancel setup** or closing the dialog discards it. The pending setup belongs to the browser session that started it and expires after ten minutes.
- **Verify and turn on** enables the app, signs the account out everywhere, including terminal sessions and application grants, and shows ten new recovery codes once. The code that confirmed the app cannot be used to sign in.
- **Replace app** sets up a new secret while the current app and recovery codes keep working. Verifying the new app retires the old secret and every old recovery code, signs out everywhere and shows ten new codes.
- **Remove app**, after a confirmation step, removes the app, its recovery codes and any pending setup and signs out everywhere. Applications that require MFA then need a passkey.
- **Create new recovery codes**, after a confirmation step, replaces all ten codes; the previous codes stop working immediately. Other sessions stay signed in, because rotation adds no new way in. The section shows how many unused codes remain.

Recovery codes appear in their own dialog with **Copy codes** and **Download** (a text file generated in the page). Done needs the "I saved these codes somewhere safe" confirmation. Setup keys and codes are never written to browser storage and are cleared from the page when their step ends; the status endpoint reports only whether an app is set up and how many codes remain.

See [passkeys](passkeys.md) for how browser, USB and split ceremonies relate.

## Which applications appear

Every catalogue refresh and launch checks the current browser session and the same authorization rules used for application access:

- The user and client must be enabled, and the session must remain valid.
- Client `allowed_groups`, typed user/group allow and deny rules, MFA, default assurance and configured device-trust requirements all apply. Active temporary group grants participate until expiry or revocation. Administrator status does not bypass application policy.
- OIDC evaluates at least `openid`; proxy applications evaluate their configured profile/email/group scopes; SAML evaluates the scopes needed for its NameID and attributes. Add the scopes requested by an OIDC application's login to `settings.app.launch_scopes` so scope-specific restrictions also determine its portal visibility. Actual authorization requests always enforce their own scopes.
- Service, native, LDAP, RADIUS and explicitly hidden clients are omitted from this browser catalogue. OIDC clients with authorization-code grants disabled are omitted too.

Access is evaluated on the server. No denied application names, group policies, client secrets or administrative client settings are sent to the page. The UI refreshes every thirty seconds while signed in and on window focus; every launch rechecks immediately, so an old link cannot bypass revoked access. An expired session clears the catalogue. Applications requiring stronger authentication appear after signing in with that assurance. A device-trust gate requires the separate verification API; the portal does not collect a device assertion. Verification must belong to the portal's originating user session and remains bound to its user epoch and device id, with proof/session expiry caps. Other sessions cannot inherit trust. See [ENT-06](enterprise/ENT-06.md).

The catalogue reads riAuth's current clients and memberships. Authentik data must first be migrated through the reviewed import/plan/apply workflow; it is not fetched from a separate Authentik instance in the browser. Imported parent-group membership is expanded by the existing migration workflow.

## Configure an application

Application presentation is part of `ProviderSettings.app`, managed through the existing client CLI/API and desired-state manifests. For example:

```json
{
  "app": {
    "description": "Build, review, and ship your team's projects.",
    "category": "Engineering",
    "launch_url": "https://code.example.com/",
    "icon": "code",
    "accent": "violet",
    "launch_scopes": ["openid", "profile"],
    "hidden": false
  },
  "implicit_consent": false
}
```

`implicit_consent: true` skips the browser consent screen for a first-party application. riAuth accepts it only for confidential, proxy and SAML clients, so the public `code` client below keeps `false`; `prompt=consent` still shows the screen.

For a new client, save this as `code-settings.json` and create the provider with the application's actual callback:

```sh
riauth client create code --name 'Code workspace' \
  --redirect-uri https://code.example.com/oauth/callback \
  --scope openid,profile --settings-file code-settings.json
```

For an existing client, merge the `app` object into its **complete current settings** before using `client update code --settings-file …`: a settings update replaces the whole settings object. An agent can edit the manifest, inspect `plan` and apply it with the existing revision and permission checks. No new portal-specific access list is needed. The provider, manifest and plan schemas include application metadata.

`launch_url` is an application's home or login page. URLs must use HTTPS; HTTP is allowed for loopback development. Credentials, control characters, wildcard URLs and interpolation templates are rejected. An OIDC callback URL is never used as a guessed launch destination. Without a launch URL, an otherwise accessible app remains visible as **Setup pending**. Proxy providers inherit their `external_origin`; SAML providers can inherit the local initiation endpoint only when IdP-initiated login is explicitly enabled. An explicit launch URL takes precedence. Links open in a new tab with opener/referrer isolation, after a local access check.

Icons: `app`, `code`, `chart`, `files`, `messages`, `book`, `cloud`, `terminal`, `shield`, `globe`. Accents: `violet`, `blue`, `teal`, `amber`, `rose`, `slate`. Empty values use the default icon/accent and **Workspace** category. `hidden` controls portal visibility, while the existing client policy continues to control protocol access.

Authentik imports map application names, descriptions, category groups and explicit launch URLs onto providers. Reviewed `settings.app` takes precedence. Hidden markers are preserved. Unsafe or templated URLs require an explicit replacement, and multiple applications sharing one provider produce an import blocker. These fields follow the [Authentik application model](https://docs.goauthentik.io/add-secure-apps/applications/); arbitrary Authentik policy expressions still require typed translations.

## Administration

`<issuer>/admin` is a compact administration page for administrators who are signed in to the portal. Like the portal, it is embedded in the binary and makes no external requests. It has four sections:

- **Applications**: list, set up, and edit. **New application** is a step-by-step setup for OpenID Connect clients: type (web application with a secret, single-page app or native app with PKCE, or service), redirect URIs, post-logout redirects and allowed CORS origins with per-line feedback, allowed groups and MFA, scopes, claim delivery and custom claim mappings, and token endpoint authentication (client secret, or `private_key_jwt` with a pasted public JWKS; a pasted private key is refused in the browser). Every **Continue** sends the whole draft to `POST /api/admin/client-checks`, which runs the create path's own authorization and validation in a read transaction and writes nothing; its answer also lists what the app will receive and setup findings. **Create** is the only write. Afterwards the page shows the issuer, discovery URL, endpoints, client ID and, once, the client secret, with an environment snippet to copy. The secret is erased from the page as soon as the tab loses focus, is hidden or is left; a secret that wasn't copied is replaced by rotating. An unfinished draft belongs to the administrator session that made it: signing out, losing the session or a different account signing in discards it. A service's scopes are collected on its API access step; checks of earlier steps use a stand-in scope that is never stored. An application's page edits name, enabled state, MFA requirement, allowed groups, redirect URIs, scopes, post-logout redirects, allowed origins, claim delivery, custom claims and the portal presentation (`settings.app`). Saving sends the complete current settings, so other protocol settings are kept; signing, encryption, lifetimes and trust are shown read-only and still change through the CLI or manifests. **Rotate secret** shows a new client secret once.
- **Connection and diagnostics** on an application's page (and after setup) come from `GET /api/admin/clients/{id}/diagnostics` (`client.read`): issuer and endpoints, accepted token endpoint authentication, and findings such as a missing redirect URI or origin, allowed groups without enabled members, people without the second factor the application requires, missing signing keys, and whether any tokens have been issued. People and groups are counted only when the administrator may read them. The browser additionally fetches the issuer's discovery document when it is on the same origin. **Test a sign-in** runs the policy simulation (`POST /api/admin/clients/{id}/explain`) for one person; no token is issued. Diagnostics never contact the application.
- **People**: list, create with an initial password or enroll a passkey-only administrator, and edit display name, email, verification, enabled state and administrator role. A passkey-only administrator is created only after two distinct WebAuthn credentials are verified in the administrator's browser; the initiating administrator needs a fresh MFA sign-in in that browser. Use a different device or security key for the backup. A changed email address is always saved unverified; the admin routes refuse to mark it verified in the same change. Service applications take API scopes only, without groups or MFA. **Set a new password**, **Reset MFA** and **Sign out everywhere** each ask for confirmation and sign the person out everywhere. Password assignment and MFA reset are unavailable remotely for a passkey-only administrator; use the explicit offline recovery procedure if both passkeys are lost.
- **Groups**: list, create, and add or remove members. Each group lists the applications limited to it and any active temporary access.
- **Security**: temporary access requests waiting for review (approve or deny), active temporary access (revoke), administrators without MFA, and the 50 most recent audit events, with a link to the event map.

The page calls same-origin `/api/admin/*` routes. They authenticate the browser session and run the same management methods, agent-style permission checks, validation, idempotency receipts and audit as the bearer API. They are not a second management service. Access decisions still require a configured approver for the group. Reads require `X-Riauth-Portal: 1`. Writes also pass the portal write guard (exact `Origin`, the portal header, and same-origin Fetch Metadata). The browser credential is never accepted in an `Authorization` header.

Passkey-only creation stages two five-minute, session-bound ceremonies without creating an account; the final credential commits the user and both passkeys with `If-Match` and an `Idempotency-Key`. Ordinary edits also use the management mutation seam, with the revision and retry key sent by the browser where applicable. A changed revision rejects the final passkey creation with a prompt to reload. Signed-out browsers see a sign-in prompt, and signed-in non-administrators see "Administrator access required". Navigation uses `#/section/item` links, focuses each view's heading, supports `/` to search lists and collapses tables into labelled rows on narrow screens.

## Administrator event map

`/events` serves a separate static page whose data requires an administrator who is already signed in. It is not an application in the catalogue, and the launcher does not link to it. The page draws counts from `GET /api/audit/map` on a schematic grid embedded in the binary. It does not load map tiles, fonts or locations from the internet. The HTML shell is public. `GET /api/audit/map` returns 401 without a session and 403 to a normal user; an administrator's SSO cookie or an authorized audit bearer can read the aggregates. Coordinates must already exist in the stored event/user data: the page does not locate users from IP addresses. See [ENT-14](enterprise/ENT-14.md).

## Validation

```sh
cargo test --test portal --locked
RIAUTH_TEST_BROWSER='/path/to/chrome' \
  RIAUTH_PORTAL_SCREENSHOTS=/tmp/riauth-portal-review \
  cargo test --test portal_browser --locked -- --ignored
```

`tests/portal.rs` covers password, TOTP, recovery-code and passkey sign-in, uniform errors including lockout, the write guard, passkey registration and removal rules, sign-out scopes and the https cookie names. `tests/password_browser.rs` covers browser password change and reset: account types, fresh MFA, lockout, one-time reset links and preserved factors. The browser test signs in through the actual CLI, checks policy-filtered applications, search/categories/favorites, persisted list preferences, live group revocation, text-injection handling and logout. It checks layout from 320 to 1440 pixels and can save desktop, tablet, mobile and empty-state screenshots. The fixture applications exist only in a disposable test database.

The separate Chromium/Firefox/WebKit matrix includes keyboard interaction,
responsive layout, text scaling, connection recovery and automated accessibility:

```sh
cargo build --example portal_fixture --locked
npm ci --ignore-scripts --prefix tools/browser
npm exec --prefix tools/browser -- playwright install chromium firefox webkit
npm test --prefix tools/browser
```

Fixture startup generates signing keys and hashes a password. Its deadline is
120 seconds, configurable with `RIAUTH_BROWSER_STARTUP_TIMEOUT_MS` (1..=300000).
The browser interaction deadline remains 60 seconds. This setup allowance is not
a service-startup or authentication-latency guarantee; a contended development
host exceeded the previous 20-second fixture deadline during local verification.
