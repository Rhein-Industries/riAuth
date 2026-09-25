# ENT-11 Embedded source stages

[Implementation](../../src/source.rs) and [tests](../../tests/source_stage.rs).

An interactive authorization can name one upstream source as a stage. The stage suspends that authorization, sends the browser through the existing source login, and resumes the same request. It does not start a second login stack and it does not treat an upstream OAuth login as a phishing-resistant or MFA factor.

## Configuration

Set `settings.source_stage` on the relying party to one enabled source id. The field is optional and defaults to absent. For example, a client create/update body can include `{"settings":{"source_stage":"corp-oidc"}}` after that source has been configured. Omit it, or leave it empty, and authorization keeps the existing local session, password, TOTP, and passkey behavior.

OIDC and OAuth sources both use `POST /api/sources/{id}/start`. SAML sources use that same start and their existing ACS. The stage stores the source login state; it does not build a parallel redirect.

## State machine

Records live at most 10 minutes.

1. **Pending.** Authorization needs a login: there is no session, or the request has `prompt=login`, `prompt=select_account`, or a `max_age` that the current session does not satisfy. `prompt=none` does not start a stage. riAuth keeps the original authentication transaction pending (`authenticated_session` empty) and binds it to the stage id so password, passkey, directory, and standalone source completion cannot finish it. The stage record stores the authorization id, source id, request hash, account id when one is already identified, the upstream nonce, and the expiry. The browser or CLI is sent to the source authorization URL.
2. **Upstream verified.** The existing OIDC callback or SAML ACS checks the source login once, then records the upstream identity. A second callback is rejected. The callback response does not contain a session or an authorization code. When a stage is attached, the HTTP callback redirects to the stage resume URL.
3. **Resume.** Resume succeeds only when the stage id matches, the authorization id matches, the record is unused, unexpired, and still bound to that same authorization request. The stage is marked used in the same transaction as success or terminal failure, so replay cannot issue a code.
4. **Local factor.** If the linked user has TOTP and the upstream result did not satisfy the source's `trusted_mfa_acr` list, resume returns `local_factor_required` and does not issue a code. A later resume with that TOTP or recovery code continues the same stage. This is the existing source-login rule.
5. **Complete.** The original transaction is bound to the new source session and the original authorization is approved. The code is delivered to the client's redirect URI. The source session is browser-owned: its bearer token is discarded and no terminal credential exists for it. On the browser sign-in page, a request waiting on a stage shows "Finish signing in with your organization's provider", and its password, passkey and decision endpoints return 409 `request_decided`.
6. **Rejected.** The stage is used and cancelled. The redirect is `access_denied` (or `login_required` / `unmet_authentication_requirements` when freshness or MFA cannot be met) and contains no code. A later local session cannot complete that same request while the stage record is unexpired.
7. **Cancelled.** `POST /oauth/source-stages/{stage_id}/cancel` with `authorization_id`, or `riauth source stage-cancel STAGE AUTHORIZATION`, deletes the ability to resume and fails the authorization with `access_denied`. No code is issued.
8. **Expired.** An expired stage cannot be resumed. Cleanup drops expired stage records.

Use `POST /oauth/source-stages/{stage_id}/resume` with JSON `authorization_id` and optional `otp` for a local factor. The GET resume route accepts only `authorization_id`; factor parameters, including empty or URL-encoded `otp`, are rejected before consuming the stage. Send TOTP and recovery codes only in the JSON POST body. Cancellation accepts only `authorization_id`.

While a stage is pending, `authorize` and browser approval of that same request fail closed. Cancelling or rejecting it keeps failing closed until the stage expires.

## Account binding

If the authorization already identified a user, the upstream subject must already point at that same user in `source_links`. A missing link or a link to someone else is `access_denied`, does not issue a code, and does not create or reattach an account. Auto-provisioning does not override a bound user.

If no user was identified, the existing link table selects the account. Email address never matches an account. A new user is created only when that source already has `auto_provision` enabled. The new link is the one source login would have written. The upstream login is never attached to a different local account than the link table says.

## Assurance limits

`amr` stays on the existing source-login path:

- `federated` for every source session
- `mfa` only when the upstream OIDC or SAML assertion's ACR is listed in that source's `trusted_mfa_acr`
- `otp` only when local TOTP or a recovery code was completed during resume

`acr` is unchanged from `assurance::actual`: `urn:riauth:acr:mfa` when the resulting session has `mfa`, otherwise `urn:riauth:acr:federated` for a non-LDAP source. An OAuth-only profile has no ID token, so its session keeps `auth_time = 0`, `mfa = false`, and `amr = ["federated"]` even if the userinfo document contains an `acr` claim. OAuth-only sources cannot start a stage for `prompt=login`, `prompt=select_account`, or `max_age`; those still require a fresh request-bound transaction (OIDC-02). Completing one stage does not satisfy a later prompt or `max_age`.

No stage sets `hwk`, `swk`, `pop`, `phrh`, or any other phishing-resistant method. Upstream OAuth is not an authentication assurance level.

## SAML

SAML sources are not a second implementation. Start goes through the existing signed redirect, and the ACS stores the same login result the OIDC callback stores. Resume then applies the account, factor, and authorization rules above. Regressions drive the OIDC and OAuth-only return paths and a signed, encrypted SAML ACS stage through code issuance, account/request binding and replay rejection. The opt-in XMLsec fixture runs that same SAML stage with an independent signer. The ACS only adds the stage resume hint when login was started as a stage.
