# Passkeys

riAuth supports FIDO2/WebAuthn passkeys with required user verification, in the browser and from the terminal. For signed-in accounts, enrollment, renaming, and credential deletion require a session authenticated within five minutes. A pending invitation can instead authorize a first passkey through the API described below. A user can register up to sixteen credentials. Registering or deleting one advances the account's credential version and revokes existing sessions and grants. Renaming changes only the label and keeps the credential and sessions intact. The WebAuthn RP ID is the issuer's hostname, and the origin must be the issuer's exact origin.

## Changing factors needs an MFA session

When an account already has TOTP or a passkey, **adding, renaming, or removing a passkey, starting, confirming, replacing or removing TOTP, creating recovery codes and changing the password** need a session that was signed in with a passkey or an authenticator or recovery code. A password-only session gets 403 `mfa_required` ("Sign in with your passkey or authenticator code first"). A session older than five minutes gets 403 `reauthentication_required`. Both rules apply to the portal and to the CLI/API. The first factor of an account that has none needs only a recent sign-in.

In the portal, the session must also belong to the browser. A browser that signed in by terminal approval shares that terminal session, and because an approval can be phished (someone talks the user into approving their code), such a browser gets 403 `reauthentication_required` for every passkey, authenticator-app, recovery-code or password change until it signs in itself. Signing in there gives it a session of its own; the terminal session is unchanged. Without this rule a phished approval would let the other browser plant a passkey that outlives session revocation and a password change. A passkey-only account cannot remove its last passkey.

First setup offers **Passkeys only** after the operator supplies the ownership proof; see [browser setup](browser-bootstrap.md). It uses the same two-credential policy described below.

An administrator can use **People → New person → Passkey-only administrator** after signing in with a passkey or authenticator code in that browser. The page verifies a primary and a backup credential before creating the enabled account. The new administrator signs in with either credential and can administer normally without a password. A passwordless administrator must retain two passkeys; ordinary management cannot set a password or clear its MFA. The server verifies separate credential IDs but cannot prove that two synced passkeys live on independent devices. Use a separate device or security key for the backup, and rehearse sign-in with it. Lost credentials use the [offline administrator recovery procedure](operations.md#diagnostics-and-recovery).

Initial `/setup` offers a password-backed or passkey-only first administrator. Passkey-only setup verifies both a primary and a backup credential before creating the account. For an existing password-backed installation, create a passkey-only successor with the People flow, verify that both passkeys can sign in, and then disable the first administrator. The last-administrator check prevents disabling it before another enabled administrator exists.

CLI consequences:

- A user whose only factor is a passkey signs in with the optional USB-enabled `riauthctl passkey login alice`. The portal and HTTP API provide factor management beyond the standalone client's enrollment command. A password login alone remains a password-only session. With TOTP, the legacy `riauth passwd` verifies the code from `RIAUTH_OTP` itself.
- A user with TOTP uses `riauth login alice --mfa` before changing factors.
- A stale passkey enrollment or removal now fails with `reauthentication_required` (it used to be `access_denied`), and a stale `mfa enroll` with the same 403 (it used to be 400 `invalid_request`).

## Passkeys in the browser

The [portal](PORTAL.md#sign-in) and every sign-in page offer **Sign in with a passkey**.

- **Usernameless.** When no account is signed in, the browser offers the passkeys it holds for this host and sends no username. riAuth identifies the user from the credential and its user handle.
- **Pinned re-authentication.** When a page re-authenticates the signed-in account (`prompt=login`, `max_age`, step-up, or the portal's confirm panel), the challenge lists that account's credentials, so non-discoverable keys work too.
- **Enrollment** in **Sign-in and security** asks for a resident (discoverable) key with user verification. Keys enrolled from the terminal with `riauthctl passkey enroll` are usually not discoverable: they work for pinned re-authentication and in the terminal, but cannot start a browser sign-in on their own.
- **Ceremonies are bound.** A browser ceremony is single use, expires after five minutes and is bound to its interaction and to the browser that started it (the portal's `riauth_passkey` cookie or the interaction's binding cookie). `POST /api/passkey/authentication/finish` rejects browser ceremonies.
- **Cancelled prompts.** After a cancelled or timed-out prompt the page keeps the fetched options, so the next click opens the authenticator directly. Test this flow in every browser and device your deployment supports, including Safari.
- **Explicit cancellation.** Closing the passkey dialog or selecting Cancel before verification is submitted discards its pending registration. Cancelling a sign-in before submission discards its pending authentication challenge. A cancelled ceremony cannot later finish; neither action creates a credential or session.
- **Unknown passkeys.** A credential riAuth does not know fails with "This passkey isn't registered with riAuth. Use another passkey or sign in with your password." riAuth never calls `PublicKeyCredential.signalUnknownCredential`, so browsers keep passkeys that belong to another service.

**Authentik passkeys are not migrated.** They are bound to Authentik's hostname and database. Users add new passkeys in the riAuth portal after migration. If riAuth takes over Authentik's hostname, browsers may still offer the old Authentik passkeys; riAuth rejects them with the message above and does not call `PublicKeyCredential.signalUnknownCredential`. Whether Authentik accepts that credential again after routing returns was not run. Notices and the operator decision table are in [re-enrollment](reenrollment.md). Plan re-enrollment before requiring passkeys for application access.

`POST /api/passkey/authentication/start` no longer returns `transports` hints, and an unknown username gets a decoy challenge with the same shape as a real one (it always lists one credential; real accounts may list several).

## First passkey from an invitation

Both editions expose an invitation enrollment API for an account that is still
disabled and has no credentials. `POST /api/account/accept/passkey/start` takes
the invitation `token` and a passkey `name`, and returns `ceremony`, `public_key`
WebAuthn options and `expires_in`. Complete it with
`POST /api/account/accept/passkey/finish`, supplying the same token, ceremony
and authenticator `response`. `/api/account/accept/passkey/cancel` takes the
token and ceremony. These endpoints do not accept an account or session ID.

Each ceremony lasts at most two minutes and accepts one authenticator attempt.
Starting again replaces the previous ceremony. A failed authenticator attempt
or explicit cancellation leaves the invitation available for a new start.
Revoking, reissuing or accepting the invitation invalidates pending enrollment.
The server rechecks the intended account, credential epoch, invitation and
inviter's live user/group authority before attaching the first credential.
Activation, group membership, proof consumption, credential storage and old
authority revocation commit together. Platform also consumes account/request
bound invitation and enrollment receipts through its W03 completion boundary.

Success returns `{"completed":true,"login_required":true}`. The recipient then
signs in with the new passkey; enrollment creates no session. Agent-origin or
help-desk credential exposure is retained, and invitation acceptance does not
establish independent authority for human privilege elevation. The current
browser invitation page still offers password acceptance; these split passkey
endpoints are for authenticator clients.

## Passkeys from the terminal

```sh
cargo install --locked --path crates/riauthctl --features terminal-usb
riauthctl --server https://id.example.com login alice
riauthctl --server https://id.example.com passkey enroll --name security-key
riauthctl --server https://id.example.com passkey login alice
riauthctl --server https://id.example.com passkey login alice --transaction-id "$TRANSACTION"
```

The USB-enabled `riauthctl` requests the authenticator's PIN and touch through the terminal. It uses the issuer's hostname as the WebAuthn RP ID and its exact origin. It requires a CTAP2 USB authenticator supported by `webauthn-authenticator-rs`. Platform keychains, Bluetooth and hybrid/phone transports are not implemented by this CLI. Production server builds have no terminal USB feature or USB transport dependency; test builds retain a software authenticator for protocol checks. Legacy `riauth` USB enrollment, sign-in, and approval or authorization `--passkey` commands fail locally with client guidance before starting a ceremony. Its split `passkey start` and `passkey finish` commands remain available to external authenticator clients without USB support compiled in. Use the portal for approval and authorization flows that the standalone client does not yet expose.

An external authenticator client can use split commands, including a supplied OIDC authentication transaction. From the repository root, keep the short-lived ceremony files under the ignored `deployment-private/` directory:

```sh
mkdir -p deployment-private
riauth passkey start --username alice --transaction-id "$TRANSACTION" --out deployment-private/challenge.json
# An authenticator client produces a PublicKeyCredential response from public_key.
riauth passkey finish --file deployment-private/challenge.json --response-file deployment-private/response.json
```

Use `start --name security-key --out deployment-private/challenge.json` for registration. The challenge file contains public WebAuthn options and the ceremony identifier. Verification state and credential material stay in the server database; every ceremony has one verification attempt and expires in five minutes. Authentication also checks the current account version, credential ownership and live signature counter, including concurrently issued challenges. Passkey start and finish reject authentication transactions reserved for an embedded upstream source stage; completing a different factor cannot bypass that source requirement.

The bearer HTTP counterpart is `POST /api/passkey/registration/start` with `name`, followed by `/registration/finish` with `ceremony` and `response`, both authenticated by the recent end-user bearer session. `POST /api/passkey/registration/cancel` consumes a pending registration owned by the same session. Authentication uses `/api/passkey/authentication/start` with `username` and optional `transaction_id`, then `/authentication/finish` with `ceremony` and `response`. `GET /api/passkeys`, `PATCH /api/passkeys/{id}` with `name`, and `DELETE /api/passkeys/{id}` operate on the current user's credentials. Agent credentials cannot perform these user ceremonies.

Success saves the ordinary private CLI session file. The resulting AMR is `webauthn mfa`; riAuth does not assert hardware attestation. Soft authenticator tests cover signatures, user verification, origin, challenge, counters, replay, revocation and transaction binding. Physical hardware and platform compatibility still need testing on the intended devices.

The implementation uses the [webauthn-rs](https://github.com/kanidm/webauthn-rs) protocol library. The client is not represented as FIDO certified. The `terminal-usb` dependency graph includes OpenSSL and native USB bindings, although riAuth application code is Rust.
