# Account email and recovery

The account workflow stays in the CLI. The SMTP service sends fixed-purpose messages containing a one-use code and a terminal command. It does not host verification or password-reset forms.

## What still needs the terminal

Browsers can sign in (passkey, or password with an optional TOTP or recovery code), sign out, add and remove passkeys, give consent and confirm RP sign-out. Everything below still needs the `riauth` CLI:

| Task | Command |
| --- | --- |
| Request and complete a password reset | `riauth account reset-request NAME`, `riauth account reset --token-stdin` |
| Accept an invitation, verify an email address | `riauth account accept --token-stdin`, `riauth account verify --token-stdin` |
| Change your own password | `riauth passwd` |
| Enroll TOTP, rotate recovery codes | `riauth mfa enroll`, `riauth mfa confirm`, `riauth mfa recovery-codes --out FILE` |
| List or revoke your sessions and remembered consent | `riauth session list`, `riauth session revoke ID`, `riauth consents` |
| Approve a device-flow login | `riauth device approve CODE` |
| Sign in with or link an upstream source | `riauth source start ID --out FILE`, `riauth source finish --file FILE` |
| Administration | every `user`, `group`, `client` and other management command |

A user without a terminal therefore cannot recover a forgotten password alone; an administrator can set a new one with `riauth user passwd NAME`, which signs the user out everywhere, clears a lockout and keeps their factors. Such users should enroll passkeys in the portal rather than TOTP. Emails still point to the CLI. Decide whether terminal-only recovery meets your users' needs before deployment; see [current limitations](limitations.md).

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

Invitations create disabled, non-administrator accounts. Acceptance through `riauth account accept --token-stdin` sets a password, verifies the delivered email address, enables the account and adds the approved groups. Existing accounts cannot be replaced. The invitation creator must still be an enabled administrator or an active agent with the required `user.write` and `group.members` permissions when acceptance occurs. An agent attached to a parent user also requires that parent to remain enabled and non-administrative. Cancellation invalidates the invitation; the reserved account stays disabled.

Verification codes last 24 hours, reset codes 30 minutes and invitations seven days. They are hashed in the proof store and bound to purpose, user ID, current email address and credential version. A reset is available only to enabled accounts with a verified email and an existing local password. Upstream-only accounts are not silently converted into password accounts. Reset requests return the same accepted response for unknown or ineligible accounts, and are throttled by account and network origin. A password reset revokes sessions and grants, clears password lockout, and preserves every enrolled MFA factor.

Password reset and invitation acceptance participate in the configured [password history](enterprise/ENT-08.md): `password_history = 5` by default, `0` disables checking, and at most 24 hashes are retained per user. A reused password fails without consuming a successful account transition. Imported hashes are retained for later plaintext comparisons; an imported hash alone cannot be checked for reuse without its plaintext.

Interactive completion prompts for the new password. Automation supplies `RIAUTH_EMAIL_TOKEN` and `RIAUTH_PASSWORD` through its secret mechanism; codes and passwords are not command-line arguments. Tokens are never returned by management or delivery-status endpoints. Pending email bodies necessarily contain the code until delivery; configure database encryption to protect the durable outbox at rest. Delivery removes its body; expired or superseded messages are redacted during maintenance.

For later departures, [scheduled offboarding](enterprise/ENT-10.md) persists a job and revokes access locally when maintenance reaches its absolute execution time. Its timezone field is an audit label, not a conversion rule. Outbound SCIM deactivation requires a separate reviewed provisioning plan.

Delivery workers use transactionally claimed leases, bounded retries and fixed expiry. SMTP acknowledgement can be lost, so delivery is at least once; repeated email delivery does not make the proof reusable. SMTP credentials, recipient addresses and message bodies are excluded from delivery status and error logs. `operations.read` on `operations/mail` permits delivery-status inspection. Administrative invitation writes use the same If-Match and idempotency rules as other agent mutations.
