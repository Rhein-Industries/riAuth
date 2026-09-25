# ENT-09 Audit review

[Implementation](../../src/reports.rs) and [tests](../../tests/reports.rs).

This is an operational review control for riAuth administrators and agents. It is **not** a SOC, ISO, or other compliance certification, and it does not by itself make a deployment certifiable.

## Contract

Every audit row stores `id`, `at` (Unix seconds), `actor`, `action`, `target`, optional `run_id`, and `details.request_id` when the call ran inside a request context.

Rows written for `users`, `groups`, `clients`, `agents`, `sources`, and `windows_devices` also store redacted `details.changes[]` with `resource`, `before`, and `after`. Credential writes add `changed_credentials` and replace the secret with `[redacted]` or `[changed]`. `source_secrets` records only those markers; the secret bytes are not decoded into the change log.

`GET /api/audit/review` requires the same check as `GET /api/audit`: `audit.read` on `audit/events` (`management`). Human administrators pass. A normal user, and an agent without that permission, receive `access_denied`.

| Query | Meaning |
| --- | --- |
| `action` | Exact action, or that action plus dot-separated children (`user` matches `user.create`). A trailing `.` or `*` is a raw prefix (`user.` / `user*`). |
| `actor`, `target`, `run_id` | Exact match. |
| `from`, `to` | Inclusive Unix-second bounds on `at`. |
| `limit` | Clamped to 1..=500. Default 100. |
| `cursor` | Opaque, encrypted with the inventory cursor key and context `riauth.audit.page/v1`. Bound to the actor and the filter. Expires after one hour. Not bound to the configuration revision: new rows sort at the newest end of a reverse scan, so they fall outside a cursor that has already moved backward. |

The response is `{events, next_cursor, limit, retention_seconds}`. Each event is `{id, at, actor, action, target, run_id, request_id, changes}`. The raw `details` object is not returned. One request examines at most 10,000 stored rows; if the page is not full it still returns `next_cursor` so the caller can continue. Order is reverse audit storage-key order, using timestamp-prefixed keys for application-written events. It is not a separate sort of arbitrary imported `at` values.

`GET /api/reports/audit.csv` uses this same page, permission, filter, and cursor. See [ENT-15.md](ENT-15.md).

## Actor attribution

The actor is the principal id: the user id for an administrator, `agent:{id}` for an agent, or the explicit name used by bootstrap and recovery (`bootstrap`, `local-recovery`, `anonymous`, `upstream`). `run_id` comes from `X-riAuth-Run-ID` / `RIAUTH_RUN_ID`. `request_id` is the server-generated request id when the mutation runs under HTTP context.

## Redaction

Before an audit row is stored, and again when review projects it, any JSON field whose name contains `secret`, `password`, `token`, `hash`, `totp`, `recovery`, `seed`, `private_key`, or `key_material` is replaced with `[redacted]`. The same applies to `authorization`, `proxy-authorization`, `cookie`, and header-like `authorization_header` / `auth_header` names. Markers `[redacted]` and `[changed]` are kept so a reviewer can see that a credential changed without the value.

That rule is intentionally broad. Configuration names such as `token_endpoint` are redacted because they contain `token`. Public views already omit `password_hash`, `totp_secret`, `recovery_codes`, `secret_hash`, and `token_hash`. Those values are compared only in memory.

The review response and the audit CSV do not include a details blob. CSV columns are listed in ENT-15.

## Retention

Audit rows are kept for 90 days (`AUDIT_RETENTION_SECONDS` = 7,776,000). `cleanup` deletes a row only when `at + 90 days < now`, so a row exactly 90 days old is kept. The server calls `cleanup` about once a minute, and each pass visits at most 128 audit rows (`maintenance_page`). A large backlog therefore takes more than one pass. Retention was already enforced; this work did not change the period.

## Coverage and gaps

Before/after is recorded for every write in the same transaction to `users`, `groups`, `clients`, `agents`, `sources`, or `windows_devices`, plus the redacted `source_secrets` marker. That includes the core management mutations (user create, update, and password change; group create and membership; client create, update, and secret rotation; agent create, rotate, and revoke; `source.configure`) without rewriting every historical call site. Other writers of those collections pick up the same record: SCIM, directory user/group sync, state reconcile, invitations, MFA, passkeys, login's user-row update, and dynamic client registration.

These families still have actor, action, target, time, and optional run/request ids, but no object before/after, because they do not write those collections:

- Login failure and upstream authentication state that only touch login or attempt rows
- Session revoke and OIDC, SAML, and proxy logout/session rows
- Token issue, revoke, exchange, and replay
- Signing-key rotate and configure (private and retired key bytes are not copied into the audit)
- Consent decisions
- Provisioner plan, apply, and complete bookkeeping
- Directory plan documents (apply is covered only for the user and group rows it writes)
- Certificate bind and revoke
- Registration-template create and revoke (the issued client is covered by `client.register`)
- Account verification requests that only enqueue mail

Existing application logs are not a substitute for this review record.
