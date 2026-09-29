# Re-enrollment and user communication

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task G04
`e68a97cb-99d9-41e7-93f2-f209fcb55612`.

This page is the operator decision table and the copy-ready notices for an
Authentik migration. It covers passkeys, other factors, recovery, session
invalidation, and rollback expectations. The converter and the account pages
it names already exist. This page adds no mail sender, no cutover tool, and
no deployment.

A credential moves only when an operator puts a private reference in the
import bundle and a later plan apply accepts that reference. The Authentik
user API does not return password hashes, WebAuthn credentials, sessions, or
opaque tokens. The report below is how the converter says so. Read it with
[Authentik migration](migration.md), [passkeys](passkeys.md), and
[account email and recovery](lifecycle.md).

Writing this page did not run Cargo, did not convert a real Authentik export,
and did not send these notices. The tests named under
[Evidence and remaining gates](#evidence-and-remaining-gates) are existing
in-tree fixtures. They are not a cutover.

## Report fields that are always present

A successful `import-authentik` conversion, including `--preflight`, always
sets these fields. They stay set when `ready_for_plan` is true.

```json
{
  "reauthentication_required": true,
  "old_tokens_and_sessions_imported": false,
  "signing_keys_imported": false,
  "administrators_imported": false
}
```

Every export also gets the findings in the next table. `blocking` is false:
the missing credential does not stop a plan, and it does not mean the
credential moved. `source_kind` is omitted. `blocker` is null because these
rows do not block.

| `kind` | `id` | Classification | Reason | Action |
| --- | --- | --- | --- | --- |
| `passkey` | `*` | unsupported | WebAuthn credentials are bound to Authentik's hostname and are not exported | Users add passkeys in the riAuth portal after sign-in; plan this before moving require_mfa applications |
| `session` | `*` | unsupported | Live sessions, cookies and opaque access or refresh tokens are not imported | Users sign in again; plan relying-party sessions and offline token validators |
| `totp` | `*` | manual | TOTP devices are not in the API export; only users listed in totp keep their factor | Supply TOTP references from an authorized offline export, or have users enroll again |
| `credential` | `static_tokens` | unsupported | Authentik static recovery tokens are not in the API export, and riAuth accepts only recovery codes it issued | Users generate new recovery codes in riAuth after signing in |
| `credential` | `authenticators` | unsupported | Duo, SMS, email and other Authentik authenticator devices are not exported and have no riAuth equivalent | Users enroll a passkey or TOTP factor in riAuth; plan this before moving require_mfa applications |
| `credential` | `tokens` | unsupported | Authentik app passwords, API tokens and upstream tokens stored with source connections are not imported, and riAuth accepts only credentials it issued | Issue riAuth agent credentials for automation and replace app-password sign-ins before cutover |

One passkey finding, in the shape `report.json` uses:

```json
{
  "kind": "passkey",
  "id": "*",
  "classification": "unsupported",
  "blocking": false,
  "reason": "WebAuthn credentials are bound to Authentik's hostname and are not exported",
  "action": "Users add passkeys in the riAuth portal after sign-in; plan this before moving require_mfa applications",
  "blocker": null
}
```

Per-user `password` and `totp` rows depend on the bundle. They are in the
decision table. A per-user TOTP row does not remove the `totp` `*` row.

## Operator decision table

Choose the notice from the person's rows. Send a password notice only after
that account's apply has succeeded. Keep Authentik available until the
rollback owner has recorded the route back. Rehearsal with each real
application remains [cutover work](migration.md#rehearse-and-cut-over), not a
result of this page.

| Situation | Verified product behavior | Decision before any notice |
| --- | --- | --- |
| Active local account, `passwords` entry with `hashed: true` | Preflight classifies `password` as convertible and does not read the hash. Apply accepts Django `pbkdf2_sha256` (1,000–2,000,000 iterations) and Argon2id/i v19 within the documented bounds. The first successful sign-in can rehash to riAuth's Argon2id, including a password shorter than 12 bytes. The fixture password `legacy` is six bytes and verifies. `password_matches` rejects a password longer than 1024 bytes. The account id stays. An invalid hash fails the whole apply. | Send [Imported password](#imported-password) only after apply succeeds for that username. Tell the person the password itself. An imported password can be shorter than 12 characters. Describe the hash as a private reference the operator supplied. |
| Active local account, password reference with `hashed` absent or false | Preflight classifies `password` as manual and non-blocking. Apply stores a new password from that private reference through `password_hash`, which requires 12–1024 bytes and otherwise returns "Passwords must be between 12 and 1024 bytes". The portal new-password fields use `minlength="12"` and `maxlength="1024"`. The Authentik password is not the value that was stored. | Deliver a 12–1024 byte password on the managed channel you already use for secrets. Send [New password](#new-password) without putting the password in the notice. |
| Active local account, no `passwords` entry | Preflight classifies `password` as manual and blocking: "No password or password-hash reference was supplied." The account is left out of an applicable manifest until the bundle has a hash reference or a newly established password reference. This includes an Authentik passkey-only user, because that passkey has no import field. | Do not tell them they can sign in. Add the reference and convert again, or leave the account out. Invitation is a separate account flow; it is not the converter's remediation. |
| External account whose carried `source_links` entry matches `user_source_connections` | The local password is disabled. Preflight does not block for the missing password. The user signs in through the reviewed source. | Send [Upstream sign-in](#upstream-sign-in). Rehearse that source. A passkey, session, or Authentik token still does not move with the link. |
| Inactive Authentik account | The account is kept disabled, with the local password disabled, and the missing password does not block. | Send no sign-in notice. Reactivation needs a managed password reference or a verified source link, then another conversion. |
| Username listed in `totp` | Preflight classifies that `totp` row as convertible and does not read the secret. Apply imports base32 or hex (16–64 bytes), SHA1/SHA256/SHA512, 6 or 8 digits, and a 15–120 second period. The current time step is marked spent. Recovery codes on the riAuth account are cleared. | Send [Imported authenticator app](#imported-authenticator-app) only after apply succeeds. The code visible at import time will fail until the next period. |
| Account with no `totp` entry | The `totp` `*` finding stays manual and non-blocking. No authenticator secret is created. | Send [Factors that stay in Authentik](#factors-that-stay-in-authentik). |
| Passkeys, on every export | The `passkey` `*` finding is unsupported and non-blocking. Nothing in the bundle selects a passkey to move. | Send the passkey notice that matches the hostname decision below. |
| Static recovery tokens, Duo, SMS, email codes, other authenticator devices, app passwords, API tokens, upstream tokens | The three `credential` findings are unsupported and non-blocking. riAuth recovery codes exist only after this service has an authenticator app, and only as codes it issues. | Send [Recovery](#recovery) and, for automation, replace tokens with riAuth agent credentials before cutover. |
| Live Authentik sessions, cookies, opaque access and refresh tokens | The `session` `*` finding is unsupported. `old_tokens_and_sessions_imported` is false. riAuth does not read or end those Authentik sessions. | Send [Sessions](#sessions). Separately, plan each relying party's own session and any offline validator. That plan is not executed here. |
| `require_mfa` client | Browser attach refuses the sign-in when `identity.mfa` is false and creates no session. An account that already has a passkey or authenticator app receives 403 `unmet_authentication_requirements` ("{client name} requires a passkey or an authenticator code."). An account with neither receives 403 `mfa_setup_required` ("{client name} requires a passkey or an authenticator code. Add a passkey in your applications portal first."). A passkey sign-in sets `mfa`. A password sign-in sets `mfa` only when an authenticator app is enrolled, after that sign-in consumes a current code or a riAuth recovery code. `authorize_identity` also forbids a later use of an identity whose `mfa` flag is false. | Move the application after each affected person has a working passkey or an imported or newly enrolled authenticator app, and has signed in with that factor. |
| riAuth issuer hostname differs from Authentik | The WebAuthn RP ID is the issuer hostname. A passkey registered for the other hostname is not a credential for this issuer. | Send [Passkeys on a new address](#passkeys-on-a-new-address). |
| The same hostname will be served by riAuth | A browser may still offer the Authentik passkey. riAuth answers `unknown_passkey`: "This passkey isn't registered with riAuth. Use another passkey or sign in with your password." The sign-in script does not call `PublicKeyCredential.signalUnknownCredential`, so the browser keeps the credential. | Send [Passkeys on the same address](#passkeys-on-the-same-address). Whether Authentik accepts that credential again after routing returns was not run. |
| Forgotten-password mail | Imported accounts copy the address and set `email_verified` false. **Forgot your password?** returns the same accepted response for every name and sends mail only for an enabled local-password account whose address is already verified, and only when mail is configured. Reset keeps passkeys and the authenticator app unless the account is marked credential-exposed; an exposed reset removes those factors. Either reset signs the account out and does not sign it back in. | Do not promise mail recovery on cutover day. Have the person verify the address after signing in. Send [Recovery](#recovery). A human administrator's plan apply does not by itself mark exposure. An agent apply of a password or TOTP does. Check that before promising that factors remain. |
| Lost authenticator app or passkey | A riAuth recovery code works only when this service issued it and an authenticator app is still enrolled. `riauth user reset-mfa NAME` removes passkeys, the authenticator app, and recovery codes, then advances the epoch. It refuses an account with no local password. `riauth user passwd NAME` sets a new password, keeps factors, and also advances the epoch. Both require `--idempotency-key` and `--if-revision`. | Use reset only when the person must lose every factor. Send [Factors reset by an administrator](#factors-reset-by-an-administrator) after that command. Otherwise send [Recovery](#recovery). |
| Adding or removing a passkey, or enabling, replacing, or removing the authenticator app | The change needs a sign-in from the last 300 seconds. When the account already has TOTP or a passkey, that sign-in must be an MFA session; a password-only session gets 403 `mfa_required` ("Sign in with your passkey or authenticator code first"). An imported authenticator app counts, so the first portal passkey needs that MFA sign-in. The first factor on an account with neither needs only the recent sign-in. The account epoch advances, existing riAuth sessions fail the epoch check, and logout is queued for RP sessions this service recorded. Back-channel logout is queued only where the client has a back-channel URI. The portal says adding or removing a passkey signs you out everywhere. | Include [Sessions](#sessions) with the enrollment notice. Renaming a passkey does not do this. Creating a new set of recovery codes does not do this. |
| Browser signed in by approving a terminal code | For a local password, the dialog says "This browser uses your terminal's sign-in. Sign in here to change your password or passkeys." A passkey, authenticator-app, or recovery-code attempt uses the same opening with "your passkeys", "your authenticator app", or "your recovery codes". The API behind those buttons returns 403 `reauthentication_required`: "This browser uses your terminal's sign-in. Sign in here before changing your password, passkeys or authenticator app." | Put the dialog sentence in the enrollment notice when people use terminal approval. |
| Route an application back to Authentik | riAuth does not write accounts, passwords, passkeys, TOTP, recovery codes, or sessions back to Authentik. Authentik still has the credentials that were never exported. A password changed only in riAuth is not the Authentik password. | Send [Rollback](#rollback) before cutover, and again if you actually roll back. Record the route and the owner in your inventory. |
| Restore a riAuth backup | [Restored-state recovery](recovery.md) invalidates restored riAuth sessions and grants and waits for reconciliation of persistent credentials. That procedure does not restore Authentik. | Use it for the riAuth instance. Do not describe it as the Authentik rollback. |
| Signing keys and administrators | `signing_keys_imported` and `administrators_imported` are false. Superuser status is not promoted. | Import a reviewed key with `riauth keys import` only when that key custody decision is already made. Grant riAuth administration explicitly. Neither action is a user credential transfer. |

Managed password and factor-reset commands, after `riauth revision` returns
the number you substitute. The password is read from stdin and is not part of
this page. Repeat a write only with the same key and the same
`--if-revision`.

```sh
riauth revision
riauth \
  --idempotency-key migration-alice-password \
  --if-revision '<revision>' \
  user passwd alice --password-stdin
riauth revision
riauth \
  --idempotency-key migration-alice-reset-mfa \
  --if-revision '<revision>' \
  user reset-mfa alice
```

`user reset-mfa` is the destructive row. Run it only for the person who must
enroll every factor again.

## Copy-ready notices

These are drafts for a later operator send. This repository does not mail
them, post them, or schedule them. Replace every bracket. Delete a paragraph
whose decision-table row does not match that person. Do not add a sentence
that says a passkey, session, recovery token, app password, or API token will
arrive on its own.

The product's own words are in quotation marks. The surrounding sentences are
the migration notice.

### Passkeys on a new address

Send when the riAuth issuer hostname is not Authentik's hostname.

```text
Your Authentik passkey stays with Authentik. It will not sign you in at
[sign-in address].

After you can sign in at that address, open your applications and choose
Sign-in and security. Add a passkey there. The page says: "Sign in without
a password. Adding or removing a passkey signs you out everywhere."

If an application tells you "Some applications need extra verification. Add
a passkey or an authenticator app under Sign-in and security.", add the
passkey before [application names] move on [date].

[help contact]
```

### Passkeys on the same address

Send when riAuth will answer Authentik's current hostname.

```text
Your existing passkey stays registered with Authentik. It is not copied.

If your browser offers that passkey at [sign-in address], riAuth will say:
"This passkey isn't registered with riAuth. Use another passkey or sign in
with your password." Keep the old passkey. It is still the one to use if we
switch this address back to Authentik.

Sign in with your password, then open Sign-in and security and add a passkey
for this service. Adding or removing a passkey signs you out everywhere.

[help contact]
```

### Imported password

Send only after apply succeeded for this account's hash reference.

```text
You will sign in at [sign-in address] with the same password you use at
Authentik. An administrator supplied that password hash through a private
import. Your Authentik session and passkey were not part of that import.

The first sign-in may store the password in this service's own hash. You
type the same password.

[help contact]
```

### New password

Send when the bundle's password reference is a newly established password.
The password must be 12–1024 bytes. Apply rejects any other length with
"Passwords must be between 12 and 1024 bytes". Deliver the password itself
on the secret channel, not in this text.

```text
Your Authentik password was not copied. You will receive a new password
separately from [help contact]. Use that password at [sign-in address].
This message does not contain it.

[help contact]
```

### Upstream sign-in

Send for an external account with a carried source link and no local password.

```text
You will sign in at [sign-in address] through [source name], the same kind
of upstream sign-in you use today. Your Authentik passkey, session, and
tokens were not copied. If [application names] require a passkey or an
authenticator code, add one under Sign-in and security after that upstream
sign-in works.

[help contact]
```

### Imported authenticator app

Send only after that username's TOTP apply succeeded.

```text
Your authenticator app can work at [sign-in address] because an
administrator imported its secret from a private export. The code showing
at the time of the import will be rejected. Wait for the next code, then
sign in with your password and that new code.

Your Authentik recovery tokens were not imported. After you are signed in,
open Sign-in and security and choose "Create new recovery codes". Save the
new codes. They are shown once. Creating them does not sign you out. The
old Authentik codes will not work here.

[help contact]
```

### Factors that stay in Authentik

Send when this person has no imported TOTP, and for Duo, SMS, email codes,
and other Authentik authenticator devices.

```text
Your authenticator app, Duo, SMS, email code, or other Authentik device was
not copied. Those methods will not work at [sign-in address].

Sign in with your password. Open Sign-in and security and choose "Set up an
authenticator app", or add a passkey. Do this before [application names]
move on [date] if those applications require a passkey or an authenticator
code.

App passwords and API tokens were not copied. Automation needs a new
credential from [help contact].

[help contact]
```

### Recovery

Send with whichever password and factor notices you send.

```text
Forgot your password? on the sign-in page can email a reset link only after
you have verified your email address at [sign-in address], and only if mail
is configured. On cutover day your imported address is unverified, so use
the password instructions above. The reset page keeps your passkeys and
authenticator app unless [help contact] tells you that factors were reset.
After an ordinary reset the page says: "Every session on your account was
signed out. Your passkeys and authenticator app are unchanged; if you use
an authenticator app, signing in still asks for its code." After a factor
reset it says: "Every session on your account was signed out. Your previous
passkeys and authenticator app were removed. Enroll new factors after
signing in." A reset signs you out everywhere and does not sign you back in.

Authentik static recovery tokens were not copied. Recovery codes here exist
only after an authenticator app is set up, and only as the ten codes this
service shows you. Each code works once. "Create new recovery codes"
replaces the previous set and leaves your session signed in.

If you lose both the authenticator app and the passkey, contact
[help contact]. A recovery code from this service works only while the
authenticator app is still enrolled.

[help contact]
```

### Factors reset by an administrator

Send only after `user reset-mfa` succeeded for this account.

```text
An administrator reset your sign-in methods at [sign-in address]. Your
passkeys, authenticator app, and recovery codes on this service were
removed. You were signed out everywhere. Sign in with your password, then
enroll a passkey or an authenticator app under Sign-in and security. Your
Authentik passkey was not part of this reset and was not copied.

[help contact]
```

### Sessions

Send to everyone who will be asked to use riAuth.

```text
You will sign in again at [sign-in address]. Your current Authentik sign-in
is not copied, and this service does not end it. Applications that already
have a session from Authentik keep that session until that application ends
it. Tell [help contact] which applications you use so those sessions can be
planned.

After you are on this service, adding or removing a passkey, setting up,
replacing, or removing an authenticator app, or changing your password
signs you out everywhere on this service. Applications that registered
logout with this service are included. Creating new recovery codes, or
renaming a passkey, leaves you signed in.

If a page says "This browser uses your terminal's sign-in. Sign in here
to change your password or passkeys.", sign in on that browser first.

[help contact]
```

### Rollback

Send before cutover, and again if routing returns to Authentik.

```text
If we switch [application names] back to Authentik, you use Authentik as it
was before the switch. Accounts, passwords, passkeys, authenticator apps,
and recovery codes created only at [sign-in address] do not move back.

A password you changed only at the new service is not your Authentik
password. A passkey you added only at the new service is not an Authentik
passkey. If the address is the same, the browser may offer both; use the
passkey that belongs to the service you are signing in to. The Authentik
passkey you already had remains the Authentik one.

[help contact] is the rollback owner for [application names].

[help contact]
```

## Evidence and remaining gates

The behavior above is what this tree's source does. The integrated preflight
and identity-continuity code, browser passkey enrollment, browser password
change and reset, and browser authenticator-app and recovery-code pages are
in this tree. Older diverged branch tips were checked for the same sentences
and session rules; where this tree has moved on, this page follows this tree.

### Verified in this tree

These tests were not re-run while writing this page.

| Claim | Source | Existing test |
| --- | --- | --- |
| Passkey, session, and `totp` `*` findings; per-user password and TOTP classifications | `src/migration.rs` | `authentik_preflight_classifies_every_exported_item` in `tests/identity/operations.rs` |
| `static_tokens`, `authenticators`, and `tokens` findings | `src/migration.rs` | `authentik_preflight_never_renames_merges_or_invents_identities` in `tests/identity/operations.rs` |
| Report booleans `reauthentication_required`, `old_tokens_and_sessions_imported`, `signing_keys_imported`, `administrators_imported` | `src/migration.rs` `convert` | The booleans are set on every successful convert. The classification tests above do not assert them. |
| A supported imported password hash verifies and rehashes to Argon2id, including the six-byte fixture password `legacy`. The plan does not contain the hash. A newly established password goes through `password_hash` and must be 12–1024 bytes. | `src/crypto.rs` `password_matches`, `upgrade_password_hash`, `password_hash`; `src/management.rs` apply | `imported_django_password_hash_is_verified_and_upgraded_without_identity_change` in `tests/identity/factors.rs` |
| Imported TOTP keeps its settings, rejects the current code, accepts the next period once, and omits the secret from audit and export. The prior session's `me` fails. | `src/authenticator.rs` `import`, `src/management.rs` | `imported_totp_factors_preserve_settings_reject_replay_and_reconcile_without_secret_leaks` in `tests/identity/factors.rs` |
| Import clears recovery codes. New codes require an authenticator app. Ten codes, prefix `ri_recovery_`. Rotation keeps the session. | `src/authenticator.rs` | `rotation_replaces_every_code_once_and_keeps_sessions` in `tests/totp_management.rs` |
| Confirming an authenticator app revokes sessions, including the confirming browser | `src/authenticator.rs`, `src/portal/mfa.rs` | `enrollment_is_session_bound_and_grants_nothing_until_confirmed` in `tests/totp_management.rs` |
| Passkey registration revokes the previous session. Unknown passkeys use the quoted message. The sign-in script has no `signalUnknownCredential`. | `src/assembly/passkey.rs`, `src/passkey.rs`, `src/portal/signin.js` | `passkeys_require_user_verification_origin_nonce_and_live_counter_and_revoke_cleanly` in `tests/identity/factors.rs`; `browser_signin.rs` asserts the script does not contain `signalUnknownCredential` |
| Password change returns `sessions_revoked: true` and advances the epoch. Browser reset keeps factors unless the reset is the exposed-factor path. | `src/password.rs`, `src/lifecycle.rs`, `src/portal/account.js` | `recovery_codes_are_single_use_and_password_change_revokes_old_sessions` in `tests/identity/factors.rs`; `change_with_enrolled_factors_needs_fresh_mfa_and_keeps_factors` and `browser_reset_is_scanner_safe_single_use_and_keeps_factors_required` in `tests/password_browser.rs` |
| A `require_mfa` client refuses a browser sign-in whose `identity.mfa` is false and creates no session. Password plus a current authenticator code is accepted with `mfa` true. Passkey login sets `mfa: true`. Password login sets `mfa` from `totp_secret.is_some()` after the code or recovery code is consumed. `authorize_identity` forbids a later use when the flag is false. | `src/assurance.rs` `needs_step_up`, `src/signin.rs` `insufficient_error`, `src/browser.rs` `authorize_attach`, `src/core.rs` password login and `authorize_identity`, `src/assembly/passkey.rs` `passkey_login_finish`, `src/authenticator.rs` `consume_password_factor` | `require_mfa_client_rejects_password_only_login_without_creating_a_session` and `require_mfa_client_accepts_totp_login` in `tests/browser_signin.rs`. The imported-TOTP test above asserts `me()["mfa"] == true`. Those fixtures use a local client named `secure`. No migration fixture drives a `require_mfa` client, and no test signs a passkey in to one. |
| Epoch change on an existing account queues RP logout. Back-channel delivery exists only when the client has a back-channel URI. | `src/management.rs` `write_user_record`, `src/identity/logout_queue.rs` | Session-failure assertions in the factor tests above. Delivery to a real relying party was not run. |
| Freshness is 300 seconds. An existing TOTP or passkey requires an MFA session for factor and password changes. A password-only session then gets 403 `mfa_required`. | `src/signin.rs` `FRESH_SECONDS`, `src/passkey.rs` `require_fresh_factor`, `src/identity.rs` `require_factor_session` | `stale_password_only_and_terminal_sessions_cannot_change_factors` in `tests/totp_management.rs`; `mfa_enrollment_requires_mfa_session_for_passkey_users` in `tests/signin_core.rs` |
| A terminal-backed browser is refused factor and password changes with 403 `reauthentication_required`. The API description is the `portal_factor_session` string. The dialog text is `hint` in `app.js`. | `src/portal.rs` `portal_factor_session`, `src/portal/app.js` | `stale_password_only_and_terminal_sessions_cannot_change_factors` in `tests/totp_management.rs` asserts the code for TOTP start, remove, and recovery codes. `browser_change_verifies_current_password_in_own_session_and_signs_out` in `tests/password_browser.rs` asserts the same code for a password change. Those tests do not assert the sentence. |

Portal labels used in the notices are the controls in
`src/portal/index.html`: **Sign-in and security**, **Change password**,
**Add a passkey**, **Set up an authenticator app**, and **Create new recovery
codes**. The sign-in pages use **Forgot your password?** and **Sign in with a
passkey**.

### Unrun gates

These remain open after this page:

- A real Authentik export. Every classification test uses a synthetic bundle.
  Identity continuity against a production export is still the G02 gate.
- A passkey sign-in to a `require_mfa` client. Password-only refusal and
  password-plus-TOTP acceptance are fixture-client tests. The passkey path
  is source: passkey login sets `mfa: true`, and `needs_step_up` allows that
  identity. Those two pieces were not run together.
- Staged cutover per application, including logout and rollback with the real
  peer. That is G05. This page does not record a rollback owner, a route, or
  a peer result.
- Delivery of these notices. No mailbox, ticket, or status page was contacted.
- Whether browsers offer an Authentik passkey after a hostname takeover, and
  whether Authentik accepts it again on rollback. The verified half is
  riAuth's unknown-passkey error and the absence of
  `signalUnknownCredential`.
- Physical security keys, synced platform passkeys, and phone hybrid
  transports. Existing passkey tests use a software authenticator.
- Relying-party sessions and offline tokens Authentik already issued. riAuth
  has no record of them, so it cannot end them.
- Mail delivery to imported users. `email_verified` starts false, and no SMTP
  conversation was run.
- `riauth user passwd` and `riauth user reset-mfa` against a migrated
  account. The commands are the CLI in `src/cli.rs`. They were not executed.
- Cargo. No build and no test run was started for this change, and the
  accepted worktree's Cargo target was not used.
