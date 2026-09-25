# OIDC profiles and federation

Configuration and recovery use CLI/API operations. Browsers sign in and consent on riAuth's interaction page, or approve from the terminal; an explicitly configured source stage can complete upstream login and resume its original authorization in the browser. Form-post responses use a hidden auto-submit form; front-channel logout and session checks use protocol-only frames. There is no administration GUI. Cross-site session checks depend on browser cookie policy.

Command examples using `deployment-private/` assume the repository root as the working directory, where that path is ignored by Git. Outside a checkout, use a private operator directory.

## Connect your first OIDC application

Start with an application that supports the authorization code flow and S256 PKCE. Set up an HTTPS issuer and its TLS/proxy configuration first ([operations](operations.md#tls-and-process-service)); the default HTTP issuer is for local loopback evaluation.

For a remote deployment, select its issuer and sign in in the shell where you will run the commands:

```sh
export RIAUTH_SERVER=https://id.example.com
riauth login admin
```

A local checkout with `riauth.toml` uses its configured issuer. Register the application's **exact** callback URL. Run this example from the repository root, where `deployment-private/` is ignored by Git:

```sh
mkdir -p deployment-private
riauth client create reports --name 'Reports' --confidential \
  --redirect-uri https://reports.example.com/oauth/callback \
  --scope openid,profile,email \
  --output-file deployment-private/reports-credential.json
```

The new private output file contains `client.client_id` and the one-time `client_secret`. Outside the repository, replace the output path with an equally private directory. Store the secret in the application's secret store. A browser-only application that cannot keep a secret should be a public client (omit `--confidential` and `--output-file`); it still needs S256 PKCE. Use one riAuth client per application and request only the scopes it needs. Add `offline_access` only when the application needs refresh tokens. For group-restricted access, register the group with `--group` and check the result with `riauth explain reports USERNAME --scope 'openid profile email'`.

Configure the application with the **exact issuer** from riAuth and its discovery document at the issuer path plus `/.well-known/openid-configuration`. Discovery supplies the authorization, token, UserInfo and JWKS URLs; preserve its returned issuer and endpoint URLs, including any application path. Set the application's client ID, private secret (for a confidential client), registered callback and scopes. Configure code flow, `state`, `nonce` and S256 PKCE; the application must validate the callback state and the ID token's signature, issuer, audience and nonce. riAuth's browser sign-in and consent pages handle the interactive part.

Verify by starting login **from the application**. After sign-in and consent, its exact callback should receive a code, and the application should establish its own session after redeeming it. Check a second login, denial, and sign-out in that application before rollout. If authorization stops before sign-in, compare the callback byte-for-byte with the registration and confirm the app sent `response_type=code` and a valid S256 challenge. If token exchange returns `invalid_client`, check the client type, authentication method and stored secret. If login succeeds but access is denied, inspect registered scopes, group policy, MFA requirements and `riauth explain`; a valid riAuth session alone does not bypass application policy. The [API contract](api.md#protocol) lists the wire endpoints and errors.

Keep the client secret and authorization codes out of logs. An application session remains the application's responsibility: configure and test its logout behavior rather than assuming a token or riAuth session revocation closes every existing application session. See [logout delivery](operations.md#diagnostics-and-recovery) and [testing](testing.md) for deployment checks.

## Browser sign-in, consent and proxy clients

An interactive authorization from a browser leads to the sign-in page (see the [API contract](api.md#browser-sign-in-and-interaction-pages)). How request parameters behave there:

| Request | Browser page | Terminal alternative |
| --- | --- | --- |
| Valid session, consent satisfied, policy passes | No page: the browser goes straight to the callback | – |
| `prompt=none` without that | Error `login_required` ("Interaction required") at the callback | – |
| `prompt=login` | "Confirm it's you"; the signed-in account is pinned and must sign in again for **this** request. The session id is kept, so RP `sid` and refresh tokens stay valid | `riauth request approve CODE` re-authenticates first |
| `max_age` exceeded (including `max_age=0`) | "Confirm it's you" / "Your sign-in is older than {App} allows."; same rules as `prompt=login` | Same |
| `prompt=select_account` | "Choose an account for {App}"; the username is editable. Signing in as another user ends the previous browser session and queues its logout | `--username` |
| `prompt=consent` | The consent screen, even with remembered consent or `implicit_consent` | `request approve` shows the scopes |
| `require_mfa`, a default ACR or `acr_values` needing MFA | "Extra verification needed"; a passkey, or password plus TOTP or recovery code. A password alone is refused (403) and creates no session | Same factors |
| An ACR the browser cannot provide (for example certificate login) | `unavailable` with "{App} requires a sign-in method that's only available from your terminal." | Terminal only |

A proof written by a browser sign-in is bound to the exact request (including its browser binding) and to the resulting session, is single use, and expires with the request. Another tab with an identical request cannot use it.

**Consent.** The page lists the requested scopes and offers "Remember this decision for 30 days", as `--remember` does. `settings.implicit_consent: true` skips the screen for first-party applications: only holders of `client.write` (administrators, agents with that permission, and state apply) can set it, dynamic-registration templates that set it are rejected, dynamically registered clients always get `false`, and it is refused for service and native clients and for public clients that are not proxy clients (it needs a confidential, proxy or SAML client). `prompt=consent` still asks, and every access policy still applies. Changes are audited as `client.update`. The consent and sign-out buttons ignore clicks for half a second after their screen appears or the window regains focus, so a click aimed at a closing popup cannot grant consent ([portal](PORTAL.md#session-kinds-and-sign-out)).

**One-shot delivery.** The resume path delivers the authorization response once and then deletes the pending request, its code index and any leftover proof. Reloading the resume URL afterwards shows "This sign-in has expired or was already completed".

**Proxy clients** (those with `settings.proxy`) are public, and their codes are redeemed only by riAuth's own outpost. `/oauth/token` refuses every grant for them with 400 `unauthorized_client`, after client identification. See [proxy SSO](proxy.md).

**Redirect URIs.** A client can register at most 32 exact redirect URIs (previously 20), enough for applications with one callback per interface language.

## Client authentication and workload trust

Set `settings.token_endpoint_auth_method` to `none`, `client_secret_basic`, `client_secret_post`, or `private_key_jwt`. The legacy unset value accepts Basic or post for a secret client. Private-key clients are created with `confidential: true` and pinned public `settings.jwks`, without a shared secret. Key IDs and algorithms must match: RS256/RSA, ES256/P-256, or EdDSA/Ed25519. Token-controlled key URLs are never fetched.

A private client assertion uses `iss = sub = client_id`, the **shared canonical token endpoint URL** as `aud`, a unique `jti`, and `iat`/`exp` covering at most five minutes. Use that audience at PAR, introspection, revocation and device authorization too. Assertions are consumed in the same transaction as their successful result.

```sh
mkdir -p deployment-private
riauth token service --client-id worker --assertion-file deployment-private/assertion.jwt --scope jobs
riauth schema provider --json
```

JWT bearer workload grants require a service client with explicit `machine_trust` entries mapping a pinned issuer, exact subject, public JWKS and allowed scopes. Removing that trust invalidates its active tokens. `riauth token request --file deployment-private/request.json` accepts the full JWT grant or token-exchange request. Secrets and assertions belong in private files.

RFC 8693 exchange requires an authenticated confidential requester, an `exchange` policy on it, and `exchange_from` on the target client. Scope expansion is forbidden. Delegation additionally requires the requester's own service access token. Parent/actor revocation propagates online; chains are limited to four exchanges. Only access-token inputs/outputs are supported, with no new login or offline credentials. `audience` selects a client; `resource` selects a unique registered resource within permitted target clients.

## Authorization and assurance

Supported response modes are `query`, `fragment`, `form_post`, `jwt`, `query.jwt`, `fragment.jwt`, and `form_post.jwt`, using authorization code with mandatory S256 PKCE. The `jwt` modes deliver a signed JARM response. Implicit and hybrid grants are not advertised.

`claims` requests can make supported claims essential and constrain their values. Mapped claims require their registered scopes and consent. The base assurance values are `urn:riauth:acr:password`, `urn:riauth:acr:mfa`, and `urn:riauth:acr:federated`. When HTTPS client-certificate authentication is configured, discovery also advertises `urn:riauth:acr:certificate`; an enrolled certificate produces AMR `cert` and does not satisfy MFA. See [client-certificate login](enterprise/ENT-05.md). A source satisfies MFA only when its verified ACR is explicitly listed in the source's `trusted_mfa_acr`, or when local TOTP is completed. Authentication-method claims retain their provenance.

`settings.default_acr_values` supplies client defaults. `require_pushed_authorization_requests` and `require_signed_request` can require PAR and pinned signed request objects. JAR requires `iss = client_id`, the provider's exact issuer as `aud`, `iat`, `exp` within five minutes and a unique `jti`. Conflicting outer parameters are rejected. PAR requires the client's registered authentication method and returns a server-issued request URI with a 90-second start window; starting it reserves a ten-minute terminal completion window. Requests cannot be overridden or reused after successful authorization.

```sh
mkdir -p deployment-private
riauth par --client-id app --assertion-file deployment-private/assertion.jwt --file deployment-private/authorization-parameters.json
riauth request inspect USER-CODE
riauth request approve USER-CODE --yes
riauth logout-request inspect USER-CODE
riauth logout-request approve USER-CODE --yes
```

Temporary approved group grants participate in authorization and group claims while active, without modifying durable membership. Expiry/revocation is enforced on the next online check; an already issued JWT retains its signed claims until expiration when validated offline. See [temporary access](enterprise/ENT-01.md).

`settings.require_device_trust` additionally requires fresh verification bound to the originating session, user epoch and device id. Challenges cannot cross sessions, and freshness ends at the earliest configured TTL, verifier JWT expiry or session expiry. The current verifier accepts a nonce-bound JWT under a configured local key; it is a stand-in, not a tested Chrome Enterprise/Verified Access integration. Sessions remain bearer credentials; hardware proof on each request is a separate integration. See [device trust](enterprise/ENT-06.md).

## Keys, encryption and issuer continuity

```sh
riauth keys generate application-key --algorithm ES256
riauth keys import legacy-key --algorithm RS256 --file legacy-private.pem --kid original-kid
riauth keys list
```

Select a domain with `settings.signing_key`; `signing` is the instance domain. Rotation retains old public verification keys for the configured retention window. JWKS publishes public keys across domains. `settings.issuer` sets an exact provider issuer, including an Authentik application path. Discovery is served for that host/path and points to the shared protocol endpoints. Tokens, response `iss`, JARM and logout retain the provider issuer. Issuer/subject-mode changes revoke affected grants.

Encryption settings (`id_token_encryption`, `access_token_encryption`, `userinfo_encryption`, `authorization_encryption`) contain `kid`, an X.509 `PUBLIC KEY` PEM, and `content_encryption` of `A256GCM` or `A256CBC-HS512`. Both use RSA-OAEP-256 and nested signed JWTs. `userinfo_signed_response` enables signed UserInfo without encryption. The CLI preserves a JWT UserInfo response in its `jwt` field; applications must validate/decrypt it.

`pairwise_sector` is an explicitly approved canonical host. Subjects are stable within a sector and distinct across sectors. Explicit imported subject overrides take precedence. Automatic `sector_identifier_uri` retrieval is not implemented. `token_managers` permits named confidential clients to introspect/revoke this provider's tokens; trust is directional.

## DPoP and resource audiences

Send a fresh RFC 9449 proof through `DPoP`. CLI commands accept `--dpop-proof-file`. Proofs bind method, exact endpoint URL, key, time and unique `jti`; UserInfo/proxy proofs also require `ath`. DPoP access tokens require the `DPoP` authorization scheme. Refresh and code replay cannot revoke another key's family.

The `bound_key` scope follows Authentik's ID-token profile: it produces `dpop+id_token` and `cnf.jkt`; access remains Bearer unless independently bound. `dpop_bound_access_tokens` or authorization `dpop_jkt` requires access-token binding.

`settings.resources` maps an absolute resource URI to its allowed scopes. One resource audience is supported per grant. Include `resource` in authorization/device start, or service/workload requests. Code redemption and refresh cannot change that audience. Resource-targeted access tokens are rejected at UserInfo unless its exact endpoint is the authorized resource. ID tokens retain the client audience. Removing a resource or its permissions invalidates affected tokens online. Remembered consent includes the resource; consenting to one API does not silently authorize another.

## Upstream OIDC sources

Inspect `riauth schema source-input` for direct configuration or `source` for manifest entries. A source contains exact issuer/authorization/token endpoints, upstream client ID/auth method, pinned public JWKS, scopes, approved provisioning groups and trusted MFA ACRs. Supply its client secret through a private input file or versioned manifest reference.

```sh
mkdir -p deployment-private
riauth source put --file deployment-private/private-source-input.json
riauth source list
riauth source start corporate --out deployment-private/source-transaction.json
# Visit the returned upstream URL, then review the verified identity:
riauth source finish --file deployment-private/source-transaction.json
riauth source finish --file deployment-private/source-transaction.json --yes
```

The final command saves a private CLI session. The browser callback never receives that session credential. Local TOTP, if required, comes from `RIAUTH_OTP` or `--otp-stdin`. To link an existing account, authenticate locally and start with `--link`; fresh local authentication is required. `source links` lists associations; `source unlink ID` removes one and revokes sessions created through it. Email equality never links accounts. Auto-provisioning cannot create administrators. Administrative source login requires an explicit human-admin setting, and agents cannot modify such a source.

Source profiles and `source_links` are supported in plan/apply. An imported link names the source, local username and exact upstream subject; agent permissions must cover both source and user. Source secrets, polling credentials and private state do not appear in exports/audit. Configuration changes revoke source sessions. Source key rotation is explicit; failed or ambiguous upstream code exchanges require a new login request.

This profile supports signed OIDC code responses and Basic/post/public upstream clients. The OAuth-only JSON identity profile is described below. Encrypted upstream ID tokens and upstream `private_key_jwt` remain unsupported.

## Embedded source stages

Set `settings.source_stage` to one enabled source ID to route an interactive authorization through that source when authentication is needed. The original request remains suspended for at most ten minutes and is bound to the source result; local password/passkey login and standalone source completion cannot satisfy that stage's transaction. An already identified account must match an existing explicit source link. `prompt=none` cannot start a stage.

The upstream callback redirects to `/oauth/source-stages/{stage_id}/resume?authorization_id=…`. Successful resume approves the original authorization; it does not return a terminal session credential. If a local factor is required, resume returns `local_factor_required`; submit the same `authorization_id` and `otp` in a JSON POST to that resume route. The CLI exposes `source stage-cancel STAGE AUTHORIZATION` for cancellation, but has no dedicated stage-resume/OTP command or browser OTP form.

OAuth-only sources cannot prove fresh authentication time for request-bound reauthentication. Source MFA trust, account provisioning restrictions and live application policy still apply. See [embedded source stages](enterprise/ENT-11.md) for the state machine and tested boundaries.

## Restricted dynamic registration

From the repository root, create `deployment-private/` with `mkdir -p deployment-private` before saving registration credentials. `registration create --file template.json --out deployment-private/registration-credential.json` creates an initial access credential with expiry and a maximum use count. Templates constrain exact redirects, grants, scopes, authentication methods and provider settings. `registration register --credential-file deployment-private/registration-credential.json --file metadata.json --output-file deployment-private/client.json` registers an eligible client. `registration revoke ID` stops further registrations. The creator's current permissions/expiry are checked. This is RFC 7591 registration; RFC 7592 management is not advertised.

Protocol references: [OIDC Core](https://openid.net/specs/openid-connect-core-1_0.html), [JWT assertions](https://www.rfc-editor.org/rfc/rfc7523), [token exchange](https://www.rfc-editor.org/rfc/rfc8693), [PAR](https://www.rfc-editor.org/rfc/rfc9126), [JAR](https://www.rfc-editor.org/rfc/rfc9101), [JARM](https://openid.net/specs/oauth-v2-jarm-final.html), [DPoP](https://www.rfc-editor.org/rfc/rfc9449), [resource indicators](https://www.rfc-editor.org/rfc/rfc8707), and [Authentik OIDC profiles](https://docs.goauthentik.io/add-secure-apps/providers/oauth2/). Implementation tests are local evidence, not independent certification. Independent OIDC conformance has not been established; see [testing](testing.md) and [limitations](limitations.md).

## OAuth-only upstream identity providers

A source can set `oauth_profile` to use an authenticated JSON identity endpoint in place of an OIDC ID token. It keeps the same client authentication, mandatory S256 PKCE, browser state, explicit account links, optional provisioning, source revocation and CLI completion:

```json
{
  "userinfo_endpoint":"https://accounts.example.com/api/user",
  "subject_pointer":"/id",
  "name_pointer":"/name",
  "email_pointer":"/email",
  "email_verified_pointer":null
}
```

The surrounding source must use its provider's OAuth scopes without `openid`, empty `jwks`, and empty `trusted_mfa_acr`. Subject IDs can be strings or unsigned integers; email equality never links accounts. Verified email is false unless an explicitly configured JSON pointer returns boolean true. The identity endpoint is pinned in source configuration, uses the newly exchanged bearer token over verified HTTPS, and cannot redirect.

OAuth does not provide OIDC authentication time or assurance. This profile does not assert upstream MFA, omits unknown `auth_time` from issued claims, and cannot complete request-bound reauthentication. Use a local authenticator or an OIDC source for `prompt=login`, `max_age` or essential authentication-time requirements. A source's issuer/client/identity mapping cannot change while accounts are linked. Provider-specific multi-endpoint claims adapters still require explicit implementations.
