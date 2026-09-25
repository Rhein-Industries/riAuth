# Passkeys

riAuth supports FIDO2/WebAuthn passkeys with required user verification, in the browser and from the terminal. Enrollment and credential deletion require a session authenticated within five minutes. A user can register up to sixteen credentials. Registering or deleting one advances the account's credential version and revokes existing sessions and grants. The WebAuthn RP ID is the issuer's hostname, and the origin must be the issuer's exact origin.

## Changing factors needs an MFA session

When an account already has TOTP or a passkey, **adding a passkey, removing a passkey and starting TOTP enrollment** need a session that was signed in with a passkey or an authenticator or recovery code. A password-only session gets 403 `mfa_required` ("Sign in with your passkey or authenticator code first"). A session older than five minutes gets 403 `reauthentication_required`. Both rules apply to the portal and to the CLI/API. The first factor of an account that has none needs only a recent sign-in.

In the portal, the session must also belong to the browser. A browser that signed in by terminal approval shares that terminal session, and because an approval can be phished (someone talks the user into approving their code), such a browser gets 403 `reauthentication_required` for every passkey change until it signs in itself. Signing in there gives it a session of its own; the terminal session is unchanged. Without this rule a phished approval would let the other browser plant a passkey that outlives session revocation and a password change.

CLI consequences:

- A user whose only factor is a passkey signs in with `riauth passkey login alice` before `riauth mfa enroll` or `riauth passkey enroll`; `riauth login alice` alone is a password-only session.
- A user with TOTP uses `riauth login alice --mfa` before changing factors.
- A stale passkey enrollment or removal now fails with `reauthentication_required` (it used to be `access_denied`), and a stale `mfa enroll` with the same 403 (it used to be 400 `invalid_request`).

## Passkeys in the browser

The [portal](PORTAL.md#passkeys-and-security) and every sign-in page offer **Sign in with a passkey**.

- **Usernameless.** When no account is signed in, the browser offers the passkeys it holds for this host and sends no username. riAuth identifies the user from the credential and its user handle.
- **Pinned re-authentication.** When a page re-authenticates the signed-in account (`prompt=login`, `max_age`, step-up, or the portal's confirm panel), the challenge lists that account's credentials, so non-discoverable keys work too.
- **Enrollment** in **Passkeys and security** asks for a resident (discoverable) key with user verification. Keys enrolled from the terminal with `riauth passkey enroll` are usually not discoverable: they work for pinned re-authentication and in the terminal, but cannot start a browser sign-in on their own.
- **Ceremonies are bound.** A browser ceremony is single use, expires after five minutes and is bound to its interaction and to the browser that started it (the portal's `riauth_passkey` cookie or the interaction's binding cookie). `POST /api/passkey/authentication/finish` rejects browser ceremonies.
- **Cancelled prompts.** After a cancelled or timed-out prompt the page keeps the fetched options, so the next click opens the authenticator directly. Test this flow in every browser and device your deployment supports, including Safari.
- **Unknown passkeys.** A credential riAuth does not know fails with "This passkey isn't registered with riAuth. Use another passkey or sign in with your password." riAuth never calls `PublicKeyCredential.signalUnknownCredential`, so browsers keep passkeys that belong to another service.

**Authentik passkeys are not migrated.** They are bound to Authentik's hostname and database. Users add new passkeys in the riAuth portal after migration. If riAuth takes over Authentik's hostname, browsers may still offer the old Authentik passkeys; they fail with the message above and keep working for Authentik during a rollback. Plan re-enrollment before requiring passkeys for application access.

`POST /api/passkey/authentication/start` no longer returns `transports` hints, and an unknown username gets a decoy challenge with the same shape as a real one (it always lists one credential; real accounts may list several).

## Passkeys from the terminal

```sh
riauth login alice
riauth passkey enroll --name security-key
riauth passkey login alice
riauth request approve ABCDE-FGHIJ --passkey
riauth authorize "$AUTHORIZATION_URL" --username alice --passkey
riauth passkey list
riauth passkey remove CREDENTIAL_ID
```

The USB client requests the authenticator's PIN and touch through the terminal. It uses the issuer's hostname as the WebAuthn RP ID and its exact origin. It requires a CTAP2 USB authenticator supported by `webauthn-authenticator-rs`. Platform keychains, Bluetooth and hybrid/phone transports are not implemented by this CLI.

An external authenticator client can use split commands, including a supplied OIDC authentication transaction. From the repository root, keep the short-lived ceremony files under the ignored `deployment-private/` directory:

```sh
mkdir -p deployment-private
riauth passkey start --username alice --transaction-id "$TRANSACTION" --out deployment-private/challenge.json
# An authenticator client produces a PublicKeyCredential response from public_key.
riauth passkey finish --file deployment-private/challenge.json --response-file deployment-private/response.json
```

Use `start --name security-key --out deployment-private/challenge.json` for registration. The challenge file contains public WebAuthn options and the ceremony identifier. Verification state and credential material stay in the server database; every ceremony has one verification attempt and expires in five minutes. Authentication also checks the current account version, credential ownership and live signature counter, including concurrently issued challenges. Passkey start and finish reject authentication transactions reserved for an embedded upstream source stage; completing a different factor cannot bypass that source requirement.

The bearer HTTP counterpart is `POST /api/passkey/registration/start` with `name`, followed by `/registration/finish` with `ceremony` and `response`, both authenticated by the recent end-user bearer session. Authentication uses `/api/passkey/authentication/start` with `username` and optional `transaction_id`, then `/authentication/finish` with `ceremony` and `response`. `GET /api/passkeys` and `DELETE /api/passkeys/{id}` operate on the current user's credentials. Agent credentials cannot perform these user ceremonies.

Success saves the ordinary private CLI session file. The resulting AMR is `webauthn mfa`; riAuth does not assert hardware attestation. Soft authenticator tests cover signatures, user verification, origin, challenge, counters, replay, revocation and transaction binding. Physical hardware and platform compatibility still need testing on the intended devices.

The implementation uses the [webauthn-rs](https://github.com/kanidm/webauthn-rs) protocol library. The client is not represented as FIDO certified. Its current dependency graph includes OpenSSL and native USB bindings, although riAuth application code is Rust.
