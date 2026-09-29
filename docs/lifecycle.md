# Account email and recovery

Invitation acceptance, email verification, password reset and password change have browser pages. The SMTP service sends fixed-purpose messages with a one-use code, a browser link and a terminal command.

## What still needs the terminal

Browsers can sign in (passkey, or password with an optional TOTP or recovery code), sign out, add and remove passkeys, set up, replace and remove an authenticator app, create recovery codes, give consent, confirm RP sign-out, accept an invitation, verify an email address, change a password and reset a forgotten one. The CLI remains available for these journeys (`riauth account accept|verify|reset --token-stdin`, `riauth account reset-request NAME`, `riauth passwd`, `riauth mfa enroll|replace|confirm|remove`, `riauth mfa recovery-codes --out FILE`). These tasks still need the `riauth` CLI:

| Task | Command |
| --- | --- |
| List or revoke your sessions and remembered consent | `riauth session list`, `riauth session revoke ID`, `riauth consents` |
| Approve a device-flow login | `riauth device approve CODE` |
| Sign in with or link a SAML upstream source | The platform portal can start it. The ACS POST does not finish that login; a same-site return must present both the start cookie and the one-time cookie set on the ACS response. The CLI remains `riauth source start ID --out FILE`, `riauth source finish --file FILE`. OIDC and OAuth sources also work in the browser: see [the portal](PORTAL.md#upstream-sign-in-providers) |
| Administration | every `user`, `group`, `client` and other management command |

Browser users recover a forgotten local password with **Forgot your password?** on the sign-in pages (below). An administrator can still set one with `riauth user passwd NAME`, which signs the user out everywhere, clears a lockout and keeps their factors. A lost authenticator app or passkey is factor recovery, which is separate: sign in with a recovery code, or ask an administrator, who can run `riauth user reset-mfa NAME`. Authenticator-app enrollment, replacement, and removal, and recovery-code creation, are in **Sign-in and security**. The CLI commands above remain available. Recovery codes require an authenticator app already enrolled on this service, and creating a new set does not sign the account out. Before an Authentik cutover, follow [re-enrollment and user communication](reenrollment.md).

## Change or reset a password in the browser

Only an enabled account with a local password can change or reset it here. An account whose password an imported [LDAP directory](ldap.md) verifies changes it in the directory; passkey-only, upstream-only and pending accounts have no local password, and neither journey creates one. Directory accounts are refused before any directory bind.

**Change.** Signed-in users open **Sign-in and security** in the applications portal and choose **Change password**. The form asks for the current password again. A browser that signed in by terminal approval must sign in here first. When the account has TOTP or a passkey, the browser session must also be an MFA sign-in (passkey, or password plus code) from the last five minutes; the dialog offers that confirmation. A wrong current password counts toward the same lockout as sign-in: five failures in fifteen minutes pause password checks for fifteen minutes. `riauth passwd` applies the same account and MFA rules; there a TOTP or recovery code comes from `RIAUTH_OTP`. Passkey-only accounts have no local password to change.

For the bearer password-change API used by `riauth passwd`, password verification
is bound to the original account, session, credential hash and epoch. The writer
rechecks live authority and consumes a submitted TOTP or recovery code in the
same transaction as password history, replacement, epoch advancement and
revocation. A history rejection leaves the factor unspent for a valid retry;
successful consumption is shared with ordinary sign-in's replay protection.
No intermediate login or session is created. Essentials and Platform use the
same verifier and mutation path.

**Reset.** **Forgot your password?** on the portal and on application sign-in pages opens `/account/reset`. Enter a username and the server answers the same way for every name; an eligible account (enabled, verified email, local password) receives a link that expires after 30 minutes. The code is in the link's fragment, so opening or scanning the link spends nothing. The page removes it from the address bar and sends it only with the new password. A newer request replaces the previous link; a password change, email change or another reset invalidates it. Each link works once.

Both journeys keep the account's passkeys, authenticator app and recovery codes. They sign the account out everywhere, including this browser, and end its application grants; a reset clears a password lockout. Neither signs the browser in. The next password sign-in still needs the authenticator or recovery code when TOTP is enrolled, and applications and changes that require MFA still need a passkey or code. Resetting a password never removes or bypasses a factor.

Configure delivery in `riauth.toml`:

```toml
[mail]
host = "smtp.example.com"
port = 465
from = "Identity <identity@example.com>"
security = "tls"
username = "identity"
password_file = "smtp-password"
```

`security = "starttls"` requires a successful TLS upgrade and certificate verification; it never falls back to plaintext. `loopback` permits plaintext only to a literal loopback IP for a local relay or tests. Relative credential paths resolve beside the server configuration. Keep credential files private. No email is sent by the development tests outside their loopback SMTP fixture.

```sh
mkdir -p deployment-private
riauth account verify-request
riauth account verify --token-stdin
riauth account reset-request alice
riauth account reset --token-stdin
riauth account invite --file deployment-private/invitation.json
riauth account revoke-invitation alice
riauth account deliveries
```

An invitation file follows `riauth schema invitation`. From the repository root, keep it under the ignored `deployment-private/` directory; outside the checkout, use a private operator directory:

```json
{"username":"alice","email":"alice@example.com","display_name":"Alice","groups":["engineering"]}
```

Invitations create disabled, non-administrator accounts. Acceptance through the emailed browser link or `riauth account accept --token-stdin` sets a password, verifies the delivered email address, enables the account and adds the approved groups. Existing unrelated or accepted accounts cannot be replaced. An authorized administrator or agent can issue a new invitation for the same still-pending username, including after expiry or cancellation; this preserves the reserved user ID and invalidates any earlier code. The invitation creator must still be an enabled administrator or an active agent with the required `user.write` and `group.members` permissions when acceptance occurs. An agent attached to a parent user also requires that parent to remain enabled and non-administrative. Cancellation invalidates the invitation; the reserved account stays disabled.

The browser links put the code in a URL fragment, which is not sent with the GET request. Opening or scanning a link only renders the page. The page removes the fragment from the address bar and sends the code to a same-origin POST endpoint only when the person explicitly accepts the invitation or verifies the address. The server performs the same one-time validation as the CLI. Expired, revoked, replaced and already-used links have distinct responses; request a new link or contact the invitation administrator when needed. Verification does not create a session or change MFA factors; invitation acceptance requires a fresh sign-in afterward.

Signed-in users with an unverified address can request or renew the verification email in the applications portal. The request requires authentication within the last five minutes; the portal offers a fresh sign-in when needed. It reports whether delivery was queued, a recent request is still cooling down, or the address is already verified. Queued means the request entered the delivery outbox; it does not confirm arrival. Opening the resulting link never consumes the proof; only the explicit verification POST does.

Verification codes last 24 hours, reset codes 30 minutes and invitations seven days. They are hashed in the proof store and bound to purpose, user ID, current email address and credential version. A reset is available only to enabled accounts with a verified email and an existing local password that no imported directory manages. Upstream-only, passkey-only and directory accounts are not silently converted into local password accounts, including when an account changed type after its link was sent. Reset requests return the same accepted response for unknown or ineligible accounts, and are throttled by account and network origin. A password reset revokes sessions and grants, clears password lockout, and preserves every enrolled MFA factor.

Password change, password reset and invitation acceptance participate in the configured [password history](enterprise/ENT-08.md): `password_history = 5` by default, `0` disables checking, and at most 24 hashes are retained per user. A reused password fails without consuming a successful account transition. Imported hashes are retained for later plaintext comparisons; an imported hash alone cannot be checked for reuse without its plaintext.

Interactive CLI completion prompts for the new password. Automation supplies `RIAUTH_EMAIL_TOKEN` and `RIAUTH_PASSWORD` through its secret mechanism; codes and passwords are not command-line arguments. Tokens are never returned by management or delivery-status endpoints. Pending email bodies necessarily contain the code until delivery; configure database encryption to protect the durable outbox at rest. Delivery removes its body; expired or superseded messages are redacted during maintenance.

For later departures, [scheduled offboarding](enterprise/ENT-10.md) persists a job and revokes access locally when maintenance reaches its absolute execution time. Its timezone field is an audit label, not a conversion rule. Outbound SCIM deactivation requires a separate reviewed provisioning plan.

Delivery workers use transactionally claimed leases, bounded retries and fixed expiry. SMTP acknowledgement can be lost, so delivery is at least once; repeated email delivery does not make the proof reusable. SMTP credentials, recipient addresses and message bodies are excluded from delivery status and error logs. `operations.read` on `operations/mail` permits delivery-status inspection. Administrative invitation writes require `If-Match` and `Idempotency-Key` for browser, bearer API and CLI callers.

Mail polls first read at most one due outbox entry. When none is due, they return without acquiring the Store writer; an enqueue committed after that snapshot is picked up on a later poll. A positive probe always rereads the queue and rechecks the proof, expiry, attempt limit and lease under the writer before claiming. Due messages that have expired, exhausted their retries or lost their proof still enter the writer to stop delivery and redact their bodies. This is the same contract for redb and PostgreSQL.
