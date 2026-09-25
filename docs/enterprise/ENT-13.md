# ENT-13 Windows local device login

[Implementation](../../src/windows_login.rs) and [tests](../../tests/windows_login.rs).

riAuth implements the enrollment and authentication protocol a Windows credential provider would call. It does not install a provider, and no Windows interactive login was tested. See [Windows credential provider package is not built or tested](#windows-credential-provider-package-is-not-built-or-tested).

The protocol is a machine API. It is not a browser login and it does not issue an OAuth access token.

## Roles

| Actor | What they do |
| --- | --- |
| Administrator, or an agent with `device.enroll` | Enroll, list, and revoke devices |
| Credential provider | Holds the device secret, collects the user password or a fresh session, and calls login |
| riAuth | Stores only a hash of the device secret, checks the user, and returns a short-lived sign-in ticket |

Permission `device.enroll` is scoped to `device/<id>` or `*`. An agent cannot enroll, replace, or revoke a device for an administrator, including a device that is already bound to an administrator. A non-administrator session cannot enroll devices. Human administrators can.

CLI:

```sh
riauth windows-device enroll laptop --username alice --display-name "Alice laptop" --offline-ttl 43200 --show-secrets
riauth windows-device list
riauth windows-device revoke laptop
# Device secret from RIAUTH_WINDOWS_DEVICE_SECRET, or --secret-stdin.
# Password from --password-stdin. Do not read both secrets from stdin.
RIAUTH_OTP=123456 riauth windows-device login --device-id laptop --username alice --password-stdin --show-secrets
```

`--output-file` writes the enroll or login result, including the secret or ticket, to a new private file. `--show-secrets` prints it. List and revoke do not return secrets.

HTTP, relative to the issuer path:

| Method | Route | Auth |
| --- | --- | --- |
| POST | `/api/windows-devices` | Admin session or `device.enroll` on `device/<id>` |
| GET | `/api/windows-devices` | Same permission; agents see only devices they can enroll |
| DELETE | `/api/windows-devices/{id}` | Same permission |
| POST | `/api/windows-devices/login` | None. Device secret plus user proof |
| POST | `/api/windows-devices/tickets/redeem` | None. The sign-in ticket in the body |
| POST | `/api/windows-devices/offline/verify` | None. Device secret plus offline ticket |

Agent mutations use the normal `If-Match` revision and optional `Idempotency-Key`. Login, redeem, and offline verify are not idempotent management mutations. A retried login mints another ticket; unused tickets expire after 300 seconds.

JSON Schemas: `riauth schema windows-device` and `riauth schema windows-login`.

## Enrollment and user mapping

`POST /api/windows-devices` body:

```json
{"id":"laptop","display_name":"Alice laptop","username":"alice","offline_ttl":43200}
```

`id` and `username` use the usual name rules (1–64 ASCII letters, digits, `.`, `-`, `_`, `@`). The display name is 1–200 characters without controls. `offline_ttl` is optional and, when set, is 1 to 259200 seconds (72 hours).

The server generates the device secret (`ri_windev_` plus 32 random bytes), returns it once, and stores only `SHA-256` of the secret (the same digest used for session tokens). Comparison uses `subtle` constant-time equality on those digests. List, audit, and later reads do not contain the secret.

One device maps to one user. The mapping is the username stored on the device. It does not change unless an authorized caller enrolls that same id again. Re-enroll replaces the secret, clears revocation, discards outstanding sign-in tickets for that device, and may bind a different user. The previous secret then fails. New enrollment and reassignment enforce a limit of 32 device records for the target user, including revoked ones, in the same write transaction. Re-enrolling an id already assigned to the same user remains allowed at the limit. A rejected reassignment preserves the original device mapping and secret. Disabled users cannot be enrolled.

## Login

`POST /api/windows-devices/login`:

```json
{"device_id":"laptop","device_secret":"…","username":"alice","password":"…","otp":null,"reauth_session":null}
```

Send a password **or** `reauth_session`, not both.

1. The device secret must match the stored digest. A wrong secret, unknown device, revoked device, or username other than the bound user fails. A device bound to Alice cannot log in Bob, even with Bob's password.
2. Password authentication is the normal local check: Argon2 (or a supported imported hash), the shared account lockout, and, when `totp_secret` is set, a TOTP or `ri_recovery_` code exactly as `riauth login` requires. A correct password with no code returns `mfa_required` so the provider can prompt. A wrong code is `invalid_credentials` and counts toward lockout. The missing-code prompt does not.
3. Reauthentication, instead of a password, is an unexpired riAuth session for that same user whose `auth_time` is at most 300 seconds old. If TOTP is enrolled, that session must already have `mfa: true`. This is the same fresh-authentication window used before MFA changes. The session is not consumed.

A disabled user fails. Success returns:

```json
{"signin_ticket":"ri_winticket_…","expires_in":300,"expires_at":0,"token_type":"windows-signin-ticket","username":"alice","device_id":"laptop","mfa":false}
```

The ticket is random, stored only as a digest in `windows_tickets`, lives at most 300 seconds, and is single-use. It is not written to `access`, `refresh`, or `session_tokens`. `POST /oauth/token`, userinfo, and `/api/me` reject it as a bearer token. `token_type` is not `Bearer`.

`POST /api/windows-devices/tickets/redeem` with `{"ticket":"ri_winticket_…"}` consumes it and returns a logon assertion:

```json
{"token_type":"windows-logon-assertion","username":"alice","user_id":"…","device_id":"laptop","epoch":0,"expires_at":0,"mfa":false}
```

That JSON is the decision the provider would use to unlock Windows. It is not an OAuth token and is not stored as a bearer credential. A second redeem fails. Redeem also fails when the ticket is expired, the device is revoked, the user is disabled, or the user's epoch changed. Those failures still consume the ticket.

Login shares the interactive username lockout (five failures, then 15 minutes) with `POST /api/login`.

## Revocation

`DELETE /api/windows-devices/{id}` marks the device revoked and deletes its sign-in tickets. The old secret no longer logs in. Re-enroll is how the device is restored.

Every user-disable transition revokes that user's devices and deletes outstanding sign-in tickets in the same transaction, including management, SCIM, LDAP directory removal, desired-state apply, cloud-directory synchronization, scheduled offboarding, and inbound SSF account-disable. Re-enabling the user does not resurrect those device credentials; enroll the devices again. `recover-admin` sets the administrator enabled and bumps the epoch, but a device revoked by a previous disable stays revoked until re-enroll.

Password changes and other epoch bumps do not by themselves revoke the device. Online login then requires the new password. Existing sign-in tickets fail redeem because the epoch no longer matches. Offline tickets fail the same way.

## Offline tickets and recovery

Enrollment may return an offline ticket. The server does not store the ticket or a copy of the device secret. The ticket is:

```text
base64url(payload) || "." || base64url(tag)
```

`payload` is compact JSON with keys in this order: `v` (1), `typ` (`windows-offline-logon`), `device_id`, `user_id`, `username`, `epoch`, `iat`, `exp`. No whitespace. `tag` is HMAC-SHA256 over those exact bytes. The MAC key is HMAC-SHA256 with the device secret as key and data `riauth.windows-offline/v1`, a zero byte, then the device id. Both HMACs are SHA-256. The tag check is constant-time.

A credential provider that still has the device secret can verify the MAC, device id, embedded epoch, and expiry without calling riAuth. `exp - iat` is at most 72 hours. The provider must refuse the ticket after local expiry even while offline.

Server `POST /api/windows-devices/offline/verify` repeats those checks and also requires the presented secret to match the current device hash, the device to be unrevoked, the user to be enabled, and `epoch` to equal the user's current epoch. It fails after a password or other epoch change, after expiry, after device revoke, and for a different device's secret. Rotation therefore kills offline tickets at the server: the old secret no longer matches the stored hash.

Limits, which a provider cannot paper over:

- The MAC is symmetric. Whoever can read the device secret can forge a ticket that a provider will accept offline. Protect that secret. On Windows that storage is the provider's job (DPAPI, DPAPI-NG, or an equivalent hardware-backed secret). riAuth does not store it and does not implement DPAPI.
- While the machine is offline, the provider cannot learn that an administrator revoked the device or changed the password. The stolen ticket remains usable on that provider until its expiry, at most 72 hours, unless the provider already knows a newer epoch or a revocation from an earlier online check. Do not use a long `offline_ttl` if that window is unacceptable. Online login and server verify fail closed immediately.
- Offline tickets do not survive password epoch changes. `recover-admin`, a password update, self-service password change, and any other epoch bump make server verification fail. A provider that cached the previous epoch must discard the ticket once it sees the new epoch, and must not treat the ticket as valid across that change.
- Audit records the device id and the actor. They do not include the device secret, the offline ticket, or the sign-in ticket.

Break-glass is the existing local recovery, not this protocol. With the server stopped, `riauth recover-admin <username> --password-stdin` sets a new administrator password, optionally `--reset-mfa`, enables the account, and bumps the epoch. The user's riAuth password remains the online proof a provider must send. Neither recovery path requires the device secret. Neither one logs a person into Windows by itself.

## Windows credential provider package is not built or tested

This repository does not contain a credential provider, a CP DLL, C++ sources, or a WiX/MSI package. Nothing here was compiled as a Windows binary, installed on Windows, or exercised at the secure attention sequence.

This release does not include:

- Install, upgrade, and uninstall of a credential provider on any Windows version
- Interactive sign-in, unlock, or User Account Control integration
- Disabled-user behavior at the Windows logon UI (the server rejects a disabled user; Windows itself was not tested)
- Storing the device secret or offline ticket with DPAPI or Credential Manager
- Packaging, code signing, or a supported Windows build

A Windows credential provider requires separate implementation and validation. The Rust tests cover only the server protocol above.

## What the server test covers

`cargo test --offline --test windows_login` checks enrollment (secret once, absent from list and from the stored record), login, a bad secret, a disabled user, TOTP required and a valid code, admin revoke, user disable deleting tickets, offline expiry, epoch mismatch, revoke, the wrong device, single-use redeem, rejection of the ticket as an access token and as a session, secret rotation, and absence of those secrets from audit JSON.
