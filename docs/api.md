# HTTP API

Routes below are relative to the configured issuer path. For issuer `https://id.example.com/application/o/app/`, the token endpoint is `/application/o/app/oauth/token`. Discovery preserves the exact issuer, including its trailing slash. OAuth authorization-server metadata additionally supports `/.well-known/oauth-authorization-server<issuer-path>`.

Responses are JSON, protocol redirects, signed/encrypted JWTs, protocol form/iframe pages, CSV reports, the embedded application portal, or the sign-in, consent and sign-out pages. Browsers sign in on those pages or approve from the terminal; administration uses the CLI/API; a configured source stage can resume authorization through an upstream browser login. For `Accept: text/html`, interactive authorization, SAML and logout requests answer `303 See Other` to their resume page (see [browser sign-in](#browser-sign-in-and-interaction-pages)); JSON callers keep the handoff with `Refresh`, cookies and eventually a `302 Location`. Protected responses use `Cache-Control: no-store`; protected browser transitions also use a no-referrer policy. API errors use `{"error":"…","error_description":"…"}`; portal launch errors have an HTML recovery page. General request bodies are limited to 32 KiB; plan/apply allow 2 MiB and upstream SAML ACS permits 96 KiB. The SSF handler additionally limits its SET to 16 KiB. Duplicate form fields are rejected before empty values are normalized to omission.

## User application portal

| Method | Route | Contract |
| --- | --- | --- |
| GET | `/`, `/apps`, `/apps/` | Portal HTML; `/` keeps JSON for callers that do not accept HTML |
| GET | `/events`, `/events/` | Public static event-map shell; its data API requires administrator/audit access. Not an application-launcher entry |
| GET | `/api/portal` | Current browser user and permitted app summaries, plus `mfa_available` (the account has TOTP or a passkey); SSO cookie required; no administrative client settings |
| GET | `/apps/launch?client_id=…` | Recheck current session and policies, then redirect to the configured launch target |
| POST | `/api/portal/login/password` | `{"username","password","otp":string\|null,"reauthenticate":bool}`; 200 `{"status":"signed_in"}` plus the SSO cookie. `reauthenticate` pins the current SSO user (401 `invalid_token` without one). `login` bucket, credential queue, 1 s failure floor |
| POST | `/api/portal/login/passkey/start` | `{"reauthenticate":bool}`; 200 `{"ceremony","public_key":{"publicKey":…},"expires_in":300}` plus the `riauth_passkey` binding cookie. Usernameless unless pinned; pinned without a passkey is 409 `no_passkey` |
| POST | `/api/portal/login/passkey/finish` | `{"ceremony","credential"}` with `riauth_passkey`; 200 `{"status":"signed_in"}` plus the SSO cookie |
| GET | `/api/portal/passkeys` | SSO; `{"passkeys":[{"id","name","created_at","algorithm"}],"fresh","terminal","mfa","can_register","can_remove","limit":16}`. `terminal` is true when this browser shares a terminal session (it collected a terminal approval); it can then neither register nor remove |
| POST | `/api/portal/passkeys/registration/start` | SSO of a browser-owned session (a terminal-shared browser gets 403 `reauthentication_required`), signed in within five minutes, factor rule; `{"name"}`; resident key and user verification required |
| POST | `/api/portal/passkeys/registration/finish` | Same browser-owned session; `{"ceremony","credential"}`; 200 `{"status":"enrolled","passkey",…,"sessions_revoked":true}` and the SSO cookie is cleared |
| POST | `/api/portal/passkeys/{id}/remove` | SSO of a browser-owned session, signed in within five minutes, factor rule; `{}`; 200 `{"removed":true,"sessions_revoked":true}` and the SSO cookie is cleared |
| POST | `/api/portal/sign-in` | Create a ten-minute browser-bound terminal approval request; records `requested_from` |
| POST | `/api/portal/sign-in/{id}` | Poll and consume an approved request; points the browser at the approving session (revoking a displaced browser-owned session) and sets the HttpOnly SSO cookie |
| POST | `/api/portal/sign-in/{id}/cancel` | Cancel the request using its browser binding |
| POST | `/api/portal/sign-out` | No body or `{}`: revoke the cookie's session, queue SSO logout and clear the cookie; may return `saml_logout_url`. `{"scope":"browser"}`: for a terminal-approved browser, only unlink this browser |
| GET | `/api/portal/requests/{code}` | End-user CLI bearer; inspect the code, account, issuer, `requested_from`, and `reauthentication_required` (true once the session is older than 240 s) |
| POST | `/api/portal/requests/{code}` | End-user CLI bearer and `{"approve":true\|false}`; approval requires authentication within five minutes |

Every cookie-authenticated POST passes the [browser write guard](#browser-write-guard); none enables cross-origin access. Approval endpoints require an end-user session bearer, never an agent credential. The factor rule (403 `mfa_required`) applies when the account already has TOTP or a passkey and the session is not MFA. See [PORTAL.md](PORTAL.md) and [passkeys](passkeys.md).

The static assets `/portal/assets/app.css`, `/portal/assets/app.js`, `/portal/assets/auth.js`, `/portal/assets/signin.js`, `/portal/assets/map.css` and `/portal/assets/map.js` are public resources embedded in the binary.

## Protocol

| Method | Route | Contract |
| --- | --- | --- |
| GET | `/.well-known/openid-configuration` | Public OP discovery |
| GET | `/.well-known/oauth-authorization-server` | Public AS metadata |
| GET | `/oauth/jwks` | Current and retained verification keys |
| GET | `/oauth/authorize` | Query authorization request; JSON details for the CLI, or `303` to `/oauth/resume/{id}` for `Accept: text/html` (with `Vary: Accept`) |
| POST | `/oauth/authorize` | Form request plus `decision`; end-user CLI bearer and any required authentication transaction |
| GET | `/oauth/resume/{id}` | With its binding cookie: HTML gets the sign-in page while undecided; JSON gets the terminal instructions and `Refresh`. Once decided, delivers the callback **once** (302 or form post) and deletes the request, its code index and any leftover proof |
| POST | `/oauth/device/code` | Form client authentication and optional scope |
| GET | `/device` | Terminal device-approval instructions |
| POST | `/oauth/token` | Form client authentication plus grant parameters; every grant for a proxy client is refused with 400 `unauthorized_client` (its codes are redeemed only inside riAuth) |
| POST | `/oauth/par` | Authenticated pushed authorization request |
| POST | `/oauth/register` | Restricted RFC 7591 registration using an initial access credential |
| GET, POST | `/oauth/userinfo` | Access token with `openid` scope; Bearer or proof-bound DPoP authentication |
| POST | `/oauth/introspect` | Confidential client; own tokens or explicit `token_managers` trust |
| POST | `/oauth/revoke` | Client authentication/identification; own token family or explicit `token_managers` trust |
| GET, POST | `/oauth/logout` | RP-initiated logout parameters, matching browser or end-user CLI session; without a usable hint, HTML gets `303` to the sign-out confirmation page |
| GET | `/oauth/logout/resume/{id}` | Sign-out confirmation page (HTML) or terminal instructions; continues to the registered post-logout URI once decided |
| GET | `/oauth/session/iframe` | OIDC session-check iframe |
| POST | `/oauth/session/check` | JSON `client_id`, `origin`, `session_state`; checks browser SSO state |
| GET | `/saml/{id}/init` | Explicitly enabled IdP-initiated SAML login |
| GET | `/saml/resume/{id}` | Sign-in page while undecided (HTML); one-use browser-bound SAML completion once decided |
| GET | `/saml/{id}/metadata` | Signed SAML IdP metadata |
| GET, POST | `/saml/{id}/sso` | Signed AuthnRequest / LogoutRequest / LogoutResponse; an interactive AuthnRequest from a browser gets `303` to `/saml/resume/{id}` |
| GET | `/saml/sources/{id}/metadata` | Signed SAML SP metadata for an upstream source |
| POST | `/saml/sources/{id}/acs` | Signed/encrypted source assertion, bound to terminal login |
| GET, POST | `/saml/sources/{id}/slo` | Signed upstream logout request or correlated response |
| GET | `/saml/logout/{ticket}` | Continue browser SAML cleanup of already revoked sessions |
| GET | `/saml/logout/{ticket}/status` | Capability-bound propagation progress |
| GET | `/oauth/sources/{id}/callback` | State-bound upstream OIDC/OAuth callback |
| GET, POST | `/oauth/source-stages/{id}/resume` | Resume an embedded source stage with its `authorization_id`; only POST JSON accepts `otp`, GET rejects factor parameters |
| POST | `/oauth/source-stages/{id}/cancel` | Cancel that source stage using `authorization_id` |
| GET | `/.well-known/ssf-configuration` | Public Shared Signals profile metadata; see [ENT-07](enterprise/ENT-07.md) for wire limitations |
| POST | `/api/ssf/events` | Pinned signed SET in `application/secevent+jwt`; signature authenticates the sender |
| GET | `/livez` | Public process liveness; does not depend on storage |
| GET | `/readyz`, `/healthz` | Public readiness; bounded storage check and available application-worker capacity; failure is 503 |

Supported grants are `authorization_code` (`code`, `redirect_uri`, `code_verifier`), device authorization (`urn:ietf:params:oauth:grant-type:device_code`, `device_code`), `refresh_token` (`refresh_token`, optional narrowed `scope`) `client_credentials` (service clients and custom scopes only), JWT bearer (`urn:ietf:params:oauth:grant-type:jwt-bearer`) and token exchange (`urn:ietf:params:oauth:grant-type:token-exchange`). Workload trust and exchange require explicit client policy.

Clients use their registered authentication method: Basic, form secret, private_key_jwt, or public identification. Basic components are form-encoded before Base64. Mixed methods are rejected. JWT assertions, constrained token exchange and restricted RFC 7591 registration are supported; see [OIDC profiles](oidc-profiles.md).

Authorization requires code response type, registered redirect, `openid` scope and S256 PKCE. Optional fields include state, nonce, `max_age`, supported query/fragment/form-post/JARM response modes, `claims`, `acr_values`, `resource`, `dpop_jkt`, and space-separated prompts (`none`, `login`, `consent`, `select_account`). `none` is exclusive. A trusted callback receives authorization errors with state/issuer; invalid client or redirect never causes navigation.

Browser authorization creates a unique pending request, browser binding and approval code. The browser can finish it on the interaction page, or the user approves in the terminal. `GET /api/authorization/{code}` requires an end-user session, prepares request-bound reauthentication when needed, reports `requested_from` (the browser's IP address, User-Agent and time), and sets `reauthentication_required` once the session's sign-in is older than 240 s. `POST /api/authorization/decision` accepts `code`, `approve`, optional `transaction_id` and `remember`; an approval from a session signed in more than 300 s ago fails with 400 `login_required` ("Sign in again in your terminal before approving"). Sessions with `auth_time = 0` (OAuth-only upstream sources) are exempt from both rules. A successful decision is delivered to the original browser. Administrative agent credentials cannot participate. The internal `request_binding` parameter is rejected on external authorization requests.

`POST /api/login` optionally accepts `transaction_id`. Successful password/MFA authentication binds the resulting session to that exact transaction. Reusing a recently authenticated session is insufficient for `prompt=login` or `max_age=0`. Silent authorization additionally requires a valid browser session, current policy and remembered consent covering requested scopes, or a client with `implicit_consent`.

Logout accepts `id_token_hint`, optional `client_id`, registered `post_logout_redirect_uri` and `state`. Hints must be signed, correctly issued, and bound to a recent RP session. A valid hint for this browser's session signs it out directly and clears the SSO cookie. Without one, a browser gets the sign-out confirmation page and a JSON caller gets terminal instructions (`riauth logout-request approve CODE`); riAuth never silently terminates an unrelated session. On the page, **Sign out** revokes only this browser's own session: a request for another live session is refused with 403 `session_mismatch`, and a targeted session that has already ended counts as done. A logout revokes its original OP session and queues back-channel events for associated RPs. Configured SAML participants are contacted through a persistent browser continuation before the final RP redirect; see [SAML logout](saml.md). The event contains `iss`, `aud`, `iat`, `exp`, `jti`, `sub`, `sid` and the standard back-channel event, with no nonce. Delivery is retried; recipients verify and deduplicate events.

Bearer errors distinguish missing credentials, invalid tokens and insufficient scope through status and `WWW-Authenticate`. Online checks re-evaluate disabled users/clients and current policy. Signature-only JWT validation remains subject to token expiry after revocation.

## Browser sign-in and interaction pages

A browser that sends `Accept: text/html` to `GET /oauth/authorize`, `/saml/{id}/sso`, `/saml/{id}/init` or `/oauth/logout` and needs interaction gets `303 See Other` with an absolute `Location` on the issuer origin, a binding cookie, `Vary: Accept`, `Referrer-Policy: no-referrer` and an empty body. The resume path serves the sign-in, consent or sign-out page; it never sends `Refresh` and, unlike portal pages, no `Cross-Origin-Opener-Policy`, so relying-party popups keep working. A request that can complete silently (valid session, satisfied consent or `implicit_consent`, policy passes) still redirects straight to the callback. JSON callers and Core reply shapes are unchanged.

### Browser write guard

Every POST authenticated by a cookie (portal and interaction) returns 403 `access_denied` unless the request has exactly one `Origin` equal to the issuer origin, exactly one `X-Riauth-Portal: 1`, and `Sec-Fetch-Site` absent or exactly `same-origin`. Bodies are JSON (another content type is 415), at most 32 KiB, and reject unknown fields. Input limits: `username` must be a valid name, `password` at most 1024 bytes, `otp` at most 128 bytes (an empty string counts as absent), and a passkey name must be a valid display name; a violation is 400 `invalid_request`.

### Cookies

All cookies are HttpOnly, have no `Domain`, and are `Secure` on https.

| Cookie | Purpose | Path, lifetime, SameSite |
| --- | --- | --- |
| `__Host-riauth_sso` (https) or `riauth_sso` (loopback http) | Browser SSO session. A new value on every browser authentication. The portal page and the sign-in page give a browser that has none a placeholder value (one hour) that maps to no session; concurrent first sign-ins from two tabs present it and end on one session. On https, every response that sets or clears it also expires a legacy `riauth_sso` | `/` (https) or the issuer path; remaining session lifetime; `None` (https) or `Lax` |
| `__Host-riauth_return_<16 hex>` (https) or `riauth_return` | Binds an OIDC interaction to this browser | `/` (https) or `{base}oauth/resume/{id}`; 600 s; `Lax` |
| `__Host-riauth_saml_<16 hex>` or `riauth_saml` | Binds a SAML interaction | `/` or `{base}saml/resume/{id}`; 300 s; `Lax` |
| `__Host-riauth_logout_<16 hex>` or `riauth_logout` | Binds a sign-out confirmation | `/` or `{base}oauth/logout/resume/{id}`; 600 s; `Lax` |
| `riauth_passkey` | Binds a portal passkey ceremony | `{base}api/portal/login/passkey/`; 300 s; `Lax` |

The `<16 hex>` suffix is derived from the interaction id, so each interaction has its own host-only cookie. A duplicate of a binding cookie makes the request fail with 401. An approval made on the interaction page is delivered only to the browser whose live session made it.

### Interaction routes

`{id}` is the resume id from the 303. Each route needs that interaction's binding cookie.

| Method | Route | Contract |
| --- | --- | --- |
| GET | `/oauth/resume/{id}/state`, `/saml/resume/{id}/state` | The interaction state below. `browser_state` bucket |
| POST | `/oauth/resume/{id}/password`, `/saml/resume/{id}/password` | `{"username","password","otp"}`. On success, the state plus the SSO cookie. Credential queue, `login` bucket, 1 s failure floor |
| POST | `…/passkey/start` | `{}` → `{"ceremony","public_key","expires_in"}`. Pinned account: its credentials are listed; otherwise usernameless |
| POST | `…/passkey/finish` | `{"ceremony","credential"}` → the state plus the SSO cookie |
| POST | `…/decision` | `{"approve":bool,"remember":bool,"session_ref":string\|null}`. Approving needs a live SSO session whose `session_ref` matches; denying needs no session and sends `access_denied` (OIDC) or `RequestDenied` (SAML) to the application |
| GET | `/oauth/logout/resume/{id}/state` | The sign-out state |
| POST | `/oauth/logout/resume/{id}/decision` | `{"approve":bool}` → `{"status":"done","continue":…}`, clearing the SSO cookie when this browser's session was revoked |

A successful sign-in continues automatically in a separate write when consent is already satisfied, `prompt=consent` was not requested and policy passes; the state is then `complete`. A sign-in that would not satisfy the client (MFA, ACR, `prompt=login`, `max_age`) is refused with 403 and leaves no session.

### Interaction state

```json
{
  "kind": "authorize",
  "status": "authenticate",
  "reason": "sign_in",
  "expires_at": 1790000600,
  "application": {"client_id": "reports", "name": "Reports", "host": "reports.example.com"},
  "account": {"username": "alice", "display_name": "Alice Example", "mfa": true},
  "session_ref": "base64url-digest",
  "pinned": true,
  "requirements": {"mfa": true, "browser": true},
  "consent": {"required": true, "scopes": ["openid","profile","email","groups","offline_access"], "attributes": null, "resource": null, "remember_default": true},
  "logout": null,
  "terminal": {"user_code": "BCDFG-HJKLM", "issuer": "https://id.example.com"},
  "continue": null,
  "error": null,
  "message": null
}
```

- `kind` is `authorize`, `saml` or `logout`. `status` is `authenticate`, `consent`, `complete` or `unavailable` (authorize and SAML), or `confirm` or `done` (logout).
- `reason` is `sign_in`, `prompt_login`, `max_age`, `step_up`, `select_account`, `force_authn` or null. `pinned` means the page re-authenticates the signed-in account; for OIDC that is every case except `prompt=select_account`.
- `expires_at` is the effective deadline, including a pushed or signed request's own expiry. `continue` is a server-built resume path, set only for `complete` and `done`.
- `error` is `access_denied` (policy), `step_up_unavailable` (only the terminal can satisfy the request), `source_stage` (finish at the upstream provider), `invalid_request` or null.
- For logout, `logout` is `{"targeted","matches_browser","ended"}` and `consent` is null.

### Error codes

Errors keep the `{"error","error_description"}` shape.

| Code | Status | When |
| --- | ---: | --- |
| `invalid_credentials` | 401 | Any browser credential failure: unknown, disabled or invited user, wrong password or code, locked account, another user than the pinned one, or directory outage. One fixed text, answered no sooner than one second after the request started |
| `unknown_passkey` | 401 | The discoverable credential is not registered |
| `invalid_token` | 401 | Missing or wrong binding cookie, or a required SSO session is missing |
| `access_denied` | 403 | Write guard, policy, untrusted forward-auth peer |
| `account_mismatch` | 403 | The verified user differs from the pinned account |
| `unmet_authentication_requirements` | 403 | The sign-in does not meet the client's MFA or ACR requirement and the user has a factor |
| `mfa_setup_required` | 403 | Same, and the user has neither TOTP nor a passkey |
| `mfa_required` | 403 | Adding or removing a factor from a non-MFA session when the account already has one |
| `reauthentication_required` | 403 | Adding or removing a factor from a session signed in more than 300 s ago (bearer and portal), or from a browser that shares a terminal session (portal) |
| `session_mismatch` | 403 | A sign-out decision for another live session |
| `interaction_expired` | 404 | The pending request, SAML request or confirmation is missing or expired |
| `request_decided` | 409 | Already decided, or waiting on a source stage |
| `account_changed` | 409 | `session_ref` no longer matches the browser's session |
| `no_passkey` | 409 | Pinned passkey sign-in for an account without passkeys |
| `login_required` | 400 | A decision without a valid proof, or a terminal approval more than 300 s after sign-in |
| `unauthorized_client` | 400 | A token request for a proxy client from outside riAuth |
| `rate_limited` | 429 | Rate bucket exceeded; CLI lockout on `/api/login` |
| `temporarily_unavailable`, `transaction_conflict` | 503 | Admission timeout, store conflict or outage, or a staged login that expired between the two sign-in phases |

For HTML requests, a 4xx from a resume path renders a short page ("This sign-in belongs to another browser", "This sign-in has expired or was already completed…") with a link to the portal instead of JSON.

## End-user CLI sessions

`POST /api/login` accepts `username`, `password`, optional `otp` (TOTP or recovery code), and optional authentication transaction. It returns `session_token`, expiry and public user view. This is first-party CLI authentication, not an OAuth password grant. A failed login (`invalid_credentials`) is answered no sooner than one second after the request started; a locked account still gets 429 `rate_limited` here, unlike the browser endpoints.

| Method | Route | Contract |
| --- | --- | --- |
| GET | `/api/me` | Current user/session or agent/permissions |
| POST | `/api/logout` | Revoke current end-user session and associated grants; return optional `saml_logout_url` |
| GET | `/api/sessions` | Current user's unrevoked sessions, each with `kind`: `browser` (no bearer token) or `terminal` |
| DELETE | `/api/sessions/{id}` | Own session, administrator, or exact authorized agent |
| POST | `/api/password` | Current password, new `password`, optional `otp`; invalidate old sessions/grants |
| POST | `/api/mfa/enroll` | Session signed in within five minutes (else 403 `reauthentication_required`); MFA session when the account already has TOTP or a passkey (else 403 `mfa_required`); return pending enrollment secret/URI |
| POST | `/api/mfa/confirm` | `code`; enable factor and invalidate sessions |
| POST | `/api/mfa/recovery-codes` | Recent MFA session; rotate ten single-use recovery codes |
| GET | `/api/passkeys` | Current user's registered passkeys |
| DELETE | `/api/passkeys/{id}` | Recent end-user session and the factor rule; remove own passkey and invalidate credentials |
| POST | `/api/passkey/registration/start`, `/api/passkey/registration/finish` | Recent-session registration and the factor rule; server-side one-use ceremony |
| POST | `/api/passkey/authentication/start`, `/api/passkey/authentication/finish` | Username-bound authentication; optional request-bound transaction; no `transports` hints; finish rejects browser ceremonies; see [passkeys.md](passkeys.md) |
| POST | `/api/login/certificate` | Enrolled HTTPS client certificate; optional `transaction_id`; see [ENT-05](enterprise/ENT-05.md) |
| GET | `/api/authorization/{code}` | Inspect browser authorization, `requested_from` and `reauthentication_required`; prepare required authentication transaction |
| POST | `/api/authorization/decision` | `code`, `approve`, optional `transaction_id`, `remember`; approval needs a sign-in within 300 s |
| GET | `/api/logout-requests/{code}` | Inspect pending browser logout, including `requested_from` |
| POST | `/api/logout-requests/{code}/decision` | Approve or deny pending browser logout |
| POST | `/api/account/verify-request` | End-user bearer; request email verification |
| POST | `/api/account/reset-request` | Public `username`; uniform accepted response for ineligible accounts |
| POST | `/api/account/complete` | One-use `token`, `purpose`, optional new `password`; see [lifecycle.md](lifecycle.md) |
| POST | `/api/sources/{id}/start` | Start upstream login or explicit account linking |
| POST | `/api/source-login/finish` | Private source transaction, explicit approval and optional local OTP |
| GET | `/api/source-links` | Current user's linked upstream identities |
| DELETE | `/api/source-links/{id}` | Unlink own upstream identity and revoke associated sessions |
| POST | `/api/device-trust/challenge`, `/api/device-trust/verify` | End-user bearer; nonce challenge and pinned local JWT verification; see [ENT-06](enterprise/ENT-06.md) |
| GET | `/api/device/{code}` | Review device request |
| POST | `/api/device/decision` | `user_code`, `approve`; enforce identity/client policy |
| GET | `/api/consents` | Current user's remembered application consent |
| DELETE | `/api/consents/{id}` | Revoke current user's consent and grants for this client |

Temporary access routes use the same human sessions. `GET /api/access/requests` and `GET /api/access/grants` list requests/grants (agent readers need `access.read`). `POST /api/access/requests` accepts `group`, `reason`, `ttl`; `POST /api/access/requests/{id}/approve` or `POST /api/access/requests/{id}/deny` requires a configured human approver. `POST /api/access/grants/{id}/revoke` requires an approver or administrator. Self-approval and agent decisions are forbidden. See [ENT-01](enterprise/ENT-01.md).

OAuth access tokens and agent credentials cannot substitute for end-user CLI sessions.

## Management

Authenticate using a human administrator CLI session or dedicated agent bearer unless the row describes a protocol credential. Authorization is enforced by action and exact resource. See [agent.md](agent.md) for permission names, secret handling and retry guarantees. `GET /api/schema/{name}` exposes schemas also available through `riauth schema`.

| Method | Route | Contract |
| --- | --- | --- |
| GET | `/api/capabilities` | Public versioned capabilities and permission/schema discovery |
| GET, POST | `/api/users` | Visible users / create user |
| PATCH | `/api/users/{username}` | User state, credentials, attributes, verified-email state, subjects and session/MFA reset |
| GET, POST | `/api/groups` | Visible groups / create group |
| PUT, DELETE | `/api/groups/{name}/members/{username}` | Add/remove group member |
| GET, POST | `/api/clients` | Visible clients / register application |
| PATCH | `/api/clients/{id}` | Client configuration, including `settings` |
| POST | `/api/clients/{id}/rotate-secret` | Return new secret and revoke existing client grants |
| GET | `/api/resources/{kind}/{name}` | Exact user/client/group lookup |
| GET | `/api/inventory/{kind}` | Users/groups/clients/sources/audit; `after`, `limit`, optional `filter` (exact run id for audit, name substring otherwise) |
| GET | `/api/audit?limit=100` | Recent audit; maximum 1,000 events |
| GET | `/api/audit/review` | Filtered audit review (`audit.read` on `audit/events`). Query: `action`, `actor`, `target`, `run_id`, `from`, `to`, `limit` (1–500, default 100), `cursor`. See [ENT-09](enterprise/ENT-09.md). |
| GET | `/api/reports/users.csv` | UTF-8 LF user CSV. Same per-row `user.read` rule as inventory. `limit` 1–1000, `filter`, `cursor` via `x-next-cursor`. |
| GET | `/api/reports/audit.csv` | UTF-8 LF audit CSV. Same permission and filters as review. No detail blob. |
| GET | `/api/audit/map` | Aggregated stored coordinates; `audit.read` on `audit/events`; see below |
| GET | `/api/directories` | Configured LDAP directories visible to the caller |
| POST | `/api/directories/{id}/plan` | LDAP import plan; no account writes |
| GET | `/api/directory-plans/{id}` | Caller-bound LDAP plan |
| POST | `/api/directory-plans/{id}/apply` | Apply one reviewed LDAP plan |
| GET | `/api/workspace-directories` | Configured Workspace directories; no secrets |
| POST | `/api/workspace-directories/{id}/plan` | Workspace sync plan; no account writes |
| GET | `/api/workspace-directory-plans/{id}` | Caller-bound Workspace plan |
| POST | `/api/workspace-directory-plans/{id}/apply` | Apply one reviewed Workspace plan; high-impact removals need plan-ID confirmation header |
| GET | `/api/entra-directories` | Configured Entra directories; no secrets |
| POST | `/api/entra-directories/{id}/plan` | Entra sync plan; no account writes |
| GET | `/api/entra-directory-plans/{id}` | Caller-bound Entra plan |
| POST | `/api/entra-directory-plans/{id}/apply` | Apply one reviewed Entra plan; high-impact removals need plan-ID confirmation header |
| POST | `/api/policy/explain` | `client_id`, `username`, scope array, optional assumed `mfa`; read-only simulation |
| GET, POST | `/api/agents` | Human administrator: list/create scoped agents. Optional `parent` is an enabled non-administrator username |
| DELETE | `/api/agents/{id}` | Human administrator: revoke credential |
| GET, POST | `/api/windows-devices` | `device.enroll`: list devices, or enroll/rotate one. Secret and optional offline ticket are returned once. See [enterprise/ENT-13.md](enterprise/ENT-13.md) |
| DELETE | `/api/windows-devices/{id}` | Revoke that device and its sign-in tickets |
| POST | `/api/windows-devices/login` | No admin bearer. Device secret plus password or a fresh session. Returns a 300s single-use sign-in ticket, not an OAuth token |
| POST | `/api/windows-devices/tickets/redeem` | Consume a sign-in ticket once and return a Windows logon assertion |
| POST | `/api/windows-devices/offline/verify` | Re-check an offline ticket with the device secret |
| POST | `/api/agents/{id}/rotate` | Human administrator: rotate credential with requested `ttl`; parent and permissions stay unchanged |
| GET, POST | `/api/certificates` | Enrolled HTTPS certificate bindings; `mtls.read` / `mtls.bind` |
| DELETE | `/api/certificates/{id}` | Revoke an HTTPS certificate binding and its sessions |
| GET, POST | `/api/radius/certificates` | RADIUS EAP-TLS bindings; user certificate and listener enrollment permissions |
| DELETE | `/api/radius/certificates/{id}` | Revoke a RADIUS certificate binding |
| GET, POST | `/api/registration` | List/create restricted dynamic-registration templates |
| DELETE | `/api/registration/{id}` | Revoke an initial registration credential |
| GET, POST | `/api/sources` | List/upsert upstream identity-source configuration |
| GET | `/api/saml/{id}/metadata`, `/api/saml/sources/{id}/metadata` | JSON wrappers for signed XML metadata |
| POST | `/api/account/invitations` | Invite a disabled, non-administrator account with approved groups |
| DELETE | `/api/account/invitations/{username}` | Revoke its pending invitation |
| GET, POST | `/api/offboard/jobs` | List/schedule durable local offboarding jobs; `user.offboard` |
| GET | `/api/offboard/jobs/{id}` | Inspect an offboarding job |
| POST | `/api/offboard/jobs/{id}/reschedule`, `/api/offboard/jobs/{id}/cancel` | Update a scheduled job or request cancellation; see [ENT-10](enterprise/ENT-10.md) |
| GET, POST, PATCH, PUT, DELETE | `/api/ssf/streams` | SSF 1.0 receiver configuration; generated stream IDs, `stream_id` query for GET/DELETE, bearer administrator, `ssf.configure` agent, or OAuth service access token with `ssf.configure` scope |
| GET, POST | `/api/ssf/admin/streams` | List/create pinned inbound trust and subject bindings; administrator or `ssf.manage` agent |
| DELETE | `/api/ssf/admin/streams/{id}` | Delete an authorized trust registration and cancel its pending deliveries |
| PUT | `/api/ssf/admin/streams/{id}/subjects` | Replace exact approved local subject bindings for a stream |
| GET | `/api/provisioning/targets` | Configured outbound SCIM targets |
| POST | `/api/provisioning/targets/{id}/plan` | Create an immutable provisioning plan |
| GET | `/api/provisioning/plans/{id}` | Inspect caller-bound plan |
| POST | `/api/provisioning/plans/{id}/apply` | Apply reviewed plan to a durable delivery job |
| GET | `/api/provisioning/jobs` | Inspect permitted delivery jobs |
| GET, POST | `/api/keys` | List/import/generate signing-key domains |
| POST | `/api/keys/rotate` | Authorized signing-key rotation |
| POST | `/api/state/plan` | Versioned manifest; redacted immutable plan |
| GET | `/api/state/plans/{id}` | Same principal's plan and applied result |
| POST | `/api/state/apply` | Exact plan, resolved `secrets` map and optional `run_id` |
| GET | `/api/state/export` | Visible manifest, revision, no credential material |
| GET | `/api/state/revision` | Revision for conditional mutations |
| GET | `/api/operations/doctor` | Storage/key/admin diagnostics |
| GET | `/api/operations/metrics` | Process counters and capacity |
| GET | `/api/operations/prometheus` | Authenticated Prometheus text metrics |
| GET | `/api/operations/mail` | Redacted email-delivery state |
| GET | `/api/operations/logout` | Logout delivery state |
| POST | `/api/operations/backup` | Base64url `encryption_key`; encrypted complete snapshot |

Workspace and Entra plans expose `removal_impact` with `disabled_users`, `missing_users`, `removed_memberships`, and `review_required`. Any previously linked user missing from the snapshot, any mapped-group membership removal, a full linked-user disable, or a large partial disable requires confirmation. For a plan with `review_required: true`, inspect its `changes` and send `X-riAuth-Confirm-Cloud-Removals: <plan-id>` with the apply request. The exact stored plan ID is required; the server re-fetches the directory and rejects a changed snapshot or impact.

`GET /api/audit/map` returns counts of audit events by coordinates an operator already stored. Optional `since` and `until` are inclusive unix seconds. `action` is a case-sensitive action prefix, not a regular expression. A coordinate is `details.location` on the event or, only when that field is absent, the actor's user attribute `location` (`latitude` from -90 through 90, `longitude` from -180 through 180, optional `label`). Invalid or missing coordinates increment `unknown` and are not dropped. The service does not derive a location from an IP address and does not write an audit event for the read. Cells are rounded to 0.1 degree (±180 share a meridian). At most 500 cells and 10,000 scanned events are returned (`truncated`, `omitted_cells`). The JSON contains counts, rounded coordinates and an optional label — not raw events, secrets or tokens. Human administrators and agents with `audit.read` on `audit/events` use a bearer token. The `/events` shell is public; its fetch of `/api/audit/map` accepts an administrator's existing SSO cookie. A normal user or an agent without `audit.read` receives 403 from that data endpoint; a missing session receives 401. See [ENT-14](enterprise/ENT-14.md).

Provider settings include explicit grant lists, lifetimes, native profile, exact origins, post-logout/back-channel URLs, mappings, claim placement and policy. Use the generated `provider`, `client-create`, `client-update`, `user-create`, `user-update`, `manifest` and `apply` schemas rather than inferring fields from examples.

Direct administrative resource writes support `Idempotency-Key` and `If-Match: "<revision>"`; agent mutations require the latter. Successful results and their receipt commit in one transaction. Identical authenticated retries return the original result for 24 hours. Reusing a key for a different request fails. Plan/apply uses its own immutable-plan identity and preconditions. `X-riAuth-Run-ID` supplies correlation; the server creates `X-Request-ID` and records redacted mutation details.

## Inbound SCIM

| Method | Route | Contract |
| --- | --- | --- |
| GET | `/scim/v2/ServiceProviderConfig`, `/scim/v2/ResourceTypes`, `/scim/v2/Schemas`, `/scim/v2/Schemas/{id}` | Public SCIM profile metadata |
| GET, POST | `/scim/v2/{kind}` | List/create owned `Users` or `Groups` |
| POST | `/scim/v2/{kind}/.search` | Filtered resource search |
| GET, PUT, PATCH, DELETE | `/scim/v2/{kind}/{id}` | Read/replace/patch/delete an owned resource |

Resource operations use a provisioning principal with the required user/group permissions, not an end-user or OAuth access token. Agent writes require the current revision. SCIM errors, ETags, ownership, supported filters and tombstone behavior are described in [scim.md](scim.md).

## Proxy authorization and limits

`GET /api/proxy/auth?client_id=<fixed audience>` validates an OAuth access token and returns JSON plus `X-Auth-User` and `X-Auth-Sub`. The integrating proxy must fix the expected client ID and discard incoming identity headers before injecting validated ones. [Browser proxy SSO](proxy.md) additionally provides opaque application cookies and forward-auth routes under `/outpost/{client}/`.

| Method | Route | Contract |
| --- | --- | --- |
| GET | `/outpost/{id}/auth` | Trusted-peer nginx forward-auth check with fixed application origin |
| GET | `/outpost/{id}/traefik` | Trusted-peer Traefik forwardAuth check from validated `X-Forwarded-*`; read-only; see [proxy.md](proxy.md#traefik-forwardauth) |
| GET | `/outpost/{id}/start` | Browser-bound SSO start; validated `rd` return URL |
| GET | `/outpost/{id}/callback` | Redeem bound authorization code and issue application cookie |
| POST | `/outpost/{id}/logout` | Revoke application cookie; exact Origin required |

Blocking work has eight worker slots; a request waits up to two seconds for one, then gets 503 `temporarily_unavailable`. Password checks (`/api/login`, `/api/password`, `/api/portal/login/password` and the interaction `…/password` endpoints) first take one of four credential permits and keep both permits until the check finishes, even if the client disconnects; forward-auth checks use a separate queue of sixteen, each with the same two-second wait. Per effective client address and 60-second window, the limits by category are:

| Category | Limit | Routes |
| --- | ---: | --- |
| `login` | 20 | `/api/login`, `/api/login/certificate`, `/api/password`, `/api/source-login/finish`, Windows login and tickets, `/api/portal/login/password`, interaction `…/password` |
| `passkey` | 30 | `/api/passkey/*`, `/api/portal/login/passkey/*`, `/api/portal/passkeys*`, interaction `…/passkey/start` and `…/finish` |
| `browser_state` | 1200 | interaction `…/state` |
| `browser_decision` | 60 | interaction `…/decision` |
| `forward_auth` | 6000 | `/outpost/{id}/auth`, `/outpost/{id}/traefik`; always counted in memory, per node |
| `outpost_start` | 30 | `/outpost/{id}/start` |
| `portal_start` | 10 | `/api/portal/sign-in` |
| `portal_approve` | 20 | `/api/portal/requests/{code}` |
| `account` | 10 | `/api/account/*` |
| `mfa` | 10 | `/api/mfa/confirm` |
| `device_start` | 30 | `/oauth/device/code` |
| `device_verify` | 20 | `/api/device/*`, `/api/authorization/*` |
| `source_start`, `source_callback` | 30 each | source starts; upstream callbacks and stage handling |
| `saml` | 30 | `/saml/*` other than `/saml/resume/*` |
| `general` | 600 | everything else |

Interaction routes are those under `/oauth/resume/`, `/saml/resume/` and `/oauth/logout/resume/`. The optional `rate_limits` configuration overrides any category (1–100000 per minute); see [operations](operations.md#rate-limits-and-admission). IPv4 addresses count individually, IPv4-mapped IPv6 addresses count as IPv4, and other IPv6 addresses are grouped by /64. The counters hold up to 100,000 windows; a full table drops its oldest window rather than refusing a client it has not seen. Readiness/liveness probes bypass these authentication-request limits; readiness has its own bounded probe-worker pool. Account lockout additionally persists after five failures. Trusted forwarding requires explicit socket-peer IPs; other forwarding headers do not change rate-limit identity. CORS is enabled only for exact origins registered on enabled clients and supported protocol endpoints. See [operations.md](operations.md) for TLS and proxy configuration.
