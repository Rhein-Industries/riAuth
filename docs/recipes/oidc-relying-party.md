# OIDC relying-party recipe

This page is the public client and the loopback relying party inside `browser_terminal_login_callback_and_signed_backchannel_logout` in [tests/browser.rs](../../tests/browser.rs). The client id is `rp`. Its display name, `Real browser RP`, is that fixture's label. The relying party is an axum router in the same process. The file header says that router serves no HTML. Headless Chrome is the user agent that opens the router's `/start` URL. The test was not executed while this page was written.

`oidc.backchannel_logout` is in the shared capability list and is absent from `PLATFORM_FEATURES` ([src/agent.rs](../../src/agent.rs)). Essentials therefore compiles it ([src/capability.rs](../../src/capability.rs) `compiled_for`). `validate_client_settings_for` does not reject `backchannel_logout_uri` on Essentials ([src/edition.rs](../../src/edition.rs)). The integration job builds this test with Cargo's default features. Those features select `platform` ([Cargo.toml](../../Cargo.toml)). This page does not claim an Essentials run of the ignored test.

The wider provider profile, including the confidential `reports` example, stays in [oidc-profiles.md](../oidc-profiles.md). In-process checks in `tests/identity/oidc.rs` are a different suite. They are not the assertions below.

## Three different claims

| Claim | What it means here |
| --- | --- |
| Integration-job script | [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) step "Real browser and relying party" runs `RIAUTH_TEST_BROWSER="$(command -v riauth-chrome)" cargo test --test browser --locked -- --ignored` on `ubuntu-24.04` with Rust 1.98.1. The test is `#[ignore]`, so the check job's `cargo test --all-targets` does not include it. The workflow text is not a result for this revision. |
| This revision | No Cargo command was run while this page was written. The ignored test was not started. |
| Deployment peer | The relying party is the test's axum router, and Chrome is only the browser. The fixture does not speak to a named external application. |

Chrome in that job is `google-chrome-stable_current_amd64.deb` when `google-chrome` is missing. The workflow does not pin a checksum. It writes a `riauth-chrome` wrapper that executes `/usr/bin/google-chrome --no-sandbox --disable-dev-shm-usage` and then the test's own arguments. A local `RIAUTH_TEST_BROWSER` binary receives only the arguments in the test.

## Records the fixture creates

The test calls `Core::initialize`, `Core::login`, and `Core::create_client` in process. It does not run `riauth login` or `riauth client create`.

| Record | Fixture value |
| --- | --- |
| Issuer | `http://127.0.0.1:{port}` from `TcpListener::bind("127.0.0.1:0")`. The rest of `Config` is `Default`, with `data_dir` under a temporary directory. |
| Administrator | `NewUser` username `admin`, display name `Admin`, email absent, `admin` true. The password literal in the test is `browser-fixture-password`. It exists only for that temporary process. |
| Session file | After `Core::login`, `{temp}/session.json` holds `issuer`, `token`, and `expires_at` for that login. |
| Client | id `rp`, name `Real browser RP`, `confidential` false, `service` false, one redirect URI `{rp}/callback`, scopes `openid`, `profile`, `offline_access`, `allowed_groups` empty, `require_mfa` false. |
| Settings | `backchannel_logout_uri` is `{rp}/logout`. Other `ProviderSettings` fields stay at `Default`. `implicit_consent` stays false. |
| Relying party | A second `127.0.0.1:0` listener. `GET /start` redirects to the authorize URL. `GET /callback` redeems the code. `POST /logout` accepts the logout token. |

`{rp}` is `http://` plus that second listener's bound address. Both listeners are loopback HTTP. A remote deployment uses an HTTPS issuer, as [oidc-profiles.md](../oidc-profiles.md) describes. This fixture does not.

## Protocol profile this test exercises

The authorize URL is `{issuer}/oauth/authorize`. The test sets these query pairs, in order, through `url::Url::query_pairs_mut`:

| Pair | Value |
| --- | --- |
| `client_id` | `rp` |
| `response_type` | `code` |
| `redirect_uri` | `{rp}/callback` |
| `scope` | `openid profile offline_access` |
| `state` | `crypto::random_token("")` |
| `nonce` | a second fresh token |
| `code_challenge` | `crypto::digest` of the verifier: URL-safe, unpadded SHA-256 ([src/crypto.rs](../../src/crypto.rs)) |
| `code_challenge_method` | `S256` |

The test does not set `response_mode`. The callback handler reads a GET query. The verifier, state, and nonce stay in the relying-party process. The browser receives the challenge, not the verifier.

The token request is `POST {issuer}/oauth/token` with form fields, in order: `grant_type=authorization_code`, `client_id=rp`, `code` from the callback, `redirect_uri` equal to the registered callback, and `code_verifier`. There is no `client_secret` and no HTTP Basic credential. The handler requires an HTTP success status and then reads JSON.

ID tokens and logout tokens are checked by the same `verify` function in the test. It selects the JWKS key whose `kid` matches the token header, builds a `DecodingKey` from that JWK, and decodes with `Validation::new(RS256)`, audience `rp`, and the loopback issuer. A new server's generated signing key uses RS256 ([src/crypto.rs](../../src/crypto.rs) `SigningKey::generate`). The fixture does not accept ES256 or EdDSA in this verifier.

The test never requests `/.well-known/openid-configuration`. Discovery's `backchannel_logout_supported` and `backchannel_logout_session_supported` flags are a different path ([src/assembly/oidc.rs](../../src/assembly/oidc.rs)).

## Assertions the test makes

| Step | What the test requires |
| --- | --- |
| Browser startup | `RIAUTH_TEST_BROWSER` is set. Chrome is started with `--headless`, `--no-first-run`, `--no-default-browser-check`, `--disable-background-networking`, a temporary `--user-data-dir`, and `{rp}/start`. The process is still running while the test looks for a code. |
| User code | Within 20 seconds, the first `browser_authorizations` row returned by `list` has a string `code`. The test sleeps 50 milliseconds between reads. |
| Terminal approval | `CARGO_BIN_EXE_riauth` exits 0. The arguments are `--server {issuer} --session-file {temp}/session.json --non-interactive --json request approve CODE --yes --remember`. The environment removes `RIAUTH_AGENT_FILE` and `RIAUTH_RUN_ID`. |
| Callback | Query `state` equals the value the relying party generated. Query `iss` equals the loopback issuer. |
| Token response | HTTP success, then JSON. The test reads `id_token` and, later, `access_token`. |
| ID token | RS256 verification as above. `nonce` equals the authorize nonce. `preferred_username` is `admin`. With the `profile` scope, that claim is the username ([src/claims.rs](../../src/claims.rs)). |
| Callback wait | A login message arrives within 30 seconds. The sign-in page uses a 15000 ms poll wait when its terminal element is hidden or closed, and 2000 ms when that element is visible and open. When more than one second remains before `expires_at`, the delay is the minimum of that wait and the time left ([src/portal/signin.js](../../src/portal/signin.js)). |
| Logout token | `POST /logout` form field `logout_token` verifies with the same RS256 rules. `events["http://schemas.openid.net/event/backchannel-logout"]` is a JSON object. `nonce` is absent. `jti` is a string. `sid` equals the ID token `sid`. The handler returns 204. The test waits up to 5 seconds for that message. |
| UserInfo | `Core::userinfo` on the access token returns an error. The test does not match a status or an error code, and it does not call UserInfo before logout. |

The login message also carries the token JSON. The test does not assert `refresh_token`, `token_type`, `expires_in`, `scope`, `name`, `amr`, `acr`, `auth_time`, or `at_hash`. Requesting `offline_access` does not, by itself, make a refresh token one of these assertions.

## How the approved session reaches logout

`POST /api/authorization/decision` is handled by `browser_decide` ([src/browser.rs](../../src/browser.rs)). On approval it loads the bearer session for the posted token, stores that session id on the pending row, and issues the code through `authorize_in` with the same token ([src/assembly/oidc.rs](../../src/assembly/oidc.rs)). The ID token `sid` is the value `logout::record` returns: URL-safe, unpadded SHA-256 of `{session_id}`, a NUL byte, and the client id ([src/identity/logout_queue.rs](../../src/identity/logout_queue.rs)).

The test then calls `Core::logout` on the administrator token from `Core::login`. That revokes that one session and calls `queue_session` for its id ([src/management/sessions.rs](../../src/management/sessions.rs)). A logout delivery is stored only when the client has `backchannel_logout_uri`. The test then calls `riauth::logout::deliver`. That function POSTs form field `logout_token`, uses a 5-second HTTP timeout, and follows no redirects ([src/logout.rs](../../src/logout.rs)). One pass claims at most 16 due rows ([src/assembly/logout.rs](../../src/assembly/logout.rs)). This fixture has one client.

The minted logout claims are `iss`, `aud` equal to the client id, `sub`, `sid`, `iat`, `exp` set to now plus 120 seconds, `jti` equal to the delivery id, and an empty object at `http://schemas.openid.net/event/backchannel-logout`. `sign_jwt` uses typ `JWT`. The test asserts the event object, the missing nonce, a string `jti`, equal `sid` values, and HTTP 204. It does not assert `sub`, `iat`, or the 120-second lifetime.

`axum::serve` of `api::router` is how the test hosts the provider. The delivery worker that also calls `logout::deliver` is started from the server bootstrap ([src/api/server.rs](../../src/api/server.rs)), which this test does not call. The fixture's post is the direct `deliver` call. `riauth logout` was not executed.

When approval and `remember` are both true, `browser_decide` stores a consent whose `expires_at` is now plus 2,592,000 seconds ([src/management/consents.rs](../../src/management/consents.rs)). The test passes `--remember` and does not read the consent row.

## Permissions and boundaries

| Boundary | Fixture |
| --- | --- |
| Create the client | The administrator session from `Core::login`. `Core::create_client` checks `client.write` on `client/rp` ([src/core.rs](../../src/core.rs)). |
| CLI client write | `riauth client create` and `riauth client update` require `--idempotency-key` and `--if-revision`. The transport sends `idempotency-key` and `If-Match: "{revision}"` ([src/cli/transport.rs](../../src/cli/transport.rs)). The test does not send those flags. |
| Approval | The same administrator bearer. `request approve` refuses an agent file with `Agent credentials cannot authenticate or consent as an end user` ([src/cli.rs](../../src/cli.rs)). The test removes `RIAUTH_AGENT_FILE`. |
| Public client | `confidential` is false, so the token request carries no secret. Setting `implicit_consent` on this client fails with `implicit_consent requires a confidential, proxy or SAML client` ([src/core.rs](../../src/core.rs)). The fixture leaves it false. |
| Redirect and logout URLs | One exact redirect URI. Logout URLs must be exact HTTPS, or HTTP on `localhost`, `127.0.0.1`, or `[::1]`, without a username, password, fragment, `*`, or query keys `iss`, `sid`, or `state` ([src/provider.rs](../../src/provider.rs)). The fixture's logout URL is HTTP on `127.0.0.1`. |
| Scopes | `openid`, `profile`, and `offline_access` only. Omitting `--scope` on `riauth client create` selects `openid`, `profile`, `email`, and `offline_access`, which adds `email`. |
| Groups and MFA | `allowed_groups` is empty and `require_mfa` is false. The test enrolls no factor. |
| Response | Authorization code only. The test does not send implicit or hybrid parameters, PAR, JAR, DPoP, or an encrypted ID token. |

Changing `backchannel_logout_uri` on an existing client is the reviewed endpoint proposal in [reviewed-client-endpoints.md](../reviewed-client-endpoints.md). The fixture sets the URI at creation and does not stage that review.

## Closest API and CLI

These commands are the supported counterparts. The test source starts the `request approve` process shown above and calls `Core` in process. It was not run for this page.

```sh
riauth revision
riauth client create rp \
  --name 'Real browser RP' \
  --redirect-uri "$RP_URL/callback" \
  --scope openid,profile,offline_access \
  --settings-file deployment-private/rp-settings.json \
  --idempotency-key "$KEY" \
  --if-revision "$REVISION"
```

`deployment-private/rp-settings.json` is a `ProviderSettings` object. The fixture's field is one key:

```json
{"backchannel_logout_uri": "http://127.0.0.1:9/logout"}
```

Replace the URI with the relying party's exact logout URL. `http://127.0.0.1:9/logout` is a shape that passes the loopback rule. It is not an address the test binds. Omit `--confidential`. The test never wrote this file.

| Fixture step | Supported equivalent |
| --- | --- |
| `Core::initialize` administrator | The test's `NewUser`. This is not the setup wizard and not `riauth user create`. |
| `Core::login` and `session.json` | `riauth login`, which POSTs `/api/login` and writes `issuer`, `token`, and `expires_at`. |
| `Core::create_client` | The command above, or `POST /api/clients` with the same fields. Both require `client.write` on `client/rp`. |
| `riauth request approve CODE --yes --remember` | `GET /api/authorization/{code}`, then `POST /api/authorization/decision` with `code`, `approve`, `remember`, and `transaction_id` from the GET ([src/cli.rs](../../src/cli.rs)). |
| `Core::logout` of that token | `riauth logout`, which POSTs `/api/logout` and deletes the session file. `riauth session revoke ID` is `DELETE /api/sessions/{id}` for one listed session. Neither command is what the test calls, and neither command is `deliver`. |
| `riauth::logout::deliver` | The delivery pass. `GET /api/operations/logout` (`riauth deliveries`) lists queued rows for a principal allowed `operations.read` on `operations/logout`. The test does not call it. |
| `Core::userinfo` | `GET /oauth/userinfo`, or `riauth userinfo`. The test only checks that the in-process call fails after logout. |

## What this fixture does not prove

The relying party is the axum router in [tests/browser.rs](../../tests/browser.rs). Chrome drives the sign-in page. No named external relying party was connected. The client name `Real browser RP` does not name a product.

The ignored test was not run for this revision. The integration-job command is workflow text until that job runs on a commit. The Chrome package in the workflow is unpinned. The issuer is loopback HTTP, not a deployment HTTPS issuer.

The fixture does not fetch discovery. It does not assert a refresh token. It does not assert a successful UserInfo response. It does not register a front-channel logout URI, a post-logout redirect, a client secret, JAR, PAR, DPoP, JWE, or MFA. It uses one browser and one public client. The OIDF runner in `scripts/run-conformance.py` pins suite `440eec8bac7b12b7389d7ca9cbc459b53507a443` and is not this test. CI does not call that script, and the tree has no conformance result.

`tests/portal_browser.rs` and the nginx forward-auth browser path are separate ignored tests. They are not this relying party.

## Other D03 recipes

Still without a recipe page: upstream OIDC, SAML source, inbound SCIM, outbound SCIM, RADIUS, Workspace, Entra, Shared Signals, device trust, HTTPS client certificates, Vault Transit, Windows device login, the embedded reverse proxy, and shared-domain SSO. The [forward-auth](platform-forward-auth.md), [LDAP provider](platform-ldap-provider.md), [SAML IdP](platform-saml-idp.md), and [LDAP import](ldap-import.md) recipes are separate. The import page follows disposable loopback OpenLDAP, and a real Active Directory directory remains an open peer. A named external relying party is still an open peer. A named service provider is still an open peer. D04 emergency runbooks and D05 acceptance are separate work. The [capability matrix](../capability-matrix.md) records the protocol limits those recipes still have to cite.
