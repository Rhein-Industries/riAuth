# Upstream OIDC recipe

This page is the issuer inside `upstream_oidc_pkce_pinned_keys_claim_validation_and_terminal_completion` in [tests/identity/sources.rs](../../tests/identity/sources.rs). [tests/identity.rs](../../tests/identity.rs) includes that file as module `sources_tests`. The issuer is the `Upstream` helper in the same `tests/identity.rs` file: an axum router in the test process. riAuth is the client of that issuer. The [OIDC relying-party recipe](oidc-relying-party.md) is the other direction, where riAuth is the authorization server. The test was not executed while this page was written.

`identity.oidc_sources` is in `FEATURES` and is absent from `PLATFORM_FEATURES` ([src/agent.rs](../../src/agent.rs)). `compiled_for` includes such a name on Essentials ([src/capability.rs](../../src/capability.rs)). `validate_source_for` rejects a SAML source on Essentials and does not reject an OIDC `[sources]` entry ([src/edition.rs](../../src/edition.rs)). `source.write` and `user.write` are absent from `PLATFORM_ACTIONS`, so `action_available` is true for them on an Essentials build. Cargo's default features select `platform` ([Cargo.toml](../../Cargo.toml)). The check job's `cargo test --all-targets` line does not pass `--no-default-features`. A later step in that job builds Essentials and runs one CLI USB test. It does not name this function. This page does not claim an Essentials run.

The operator profile stays in [oidc-profiles.md](../oidc-profiles.md). Other functions in `tests/identity/sources.rs` are separate. This page does not use their assertions. Those functions include account linking, desired-state links, redacted manifests, the OAuth-only userinfo profile, browser completion, and `oidc_source_jwks_rotation_checks_old_and_new_keys_stale_assertions_and_rollback_replay`.

## Three different claims

| Claim | What it means here |
| --- | --- |
| Check-job script | The check job in [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) runs `cargo test --all-targets --features test-support,fuzzing --locked` on `ubuntu-24.04` with Rust 1.98.1. It does not pass `--ignored`. This function is a `#[tokio::test]` with no ignore attribute, so that command's text includes it. The same command also includes the other functions in the file. The integration job's `cargo test --test identity` steps filter on `saml_independent_xmlsec`, `saml_source_independent_xmlsec`, and `saml_logout_independent_xmlsec` with `--ignored`. They do not name this function. The workflow text is not a result for this revision. |
| This revision | No Cargo command was run while this page was written. The test was not started. |
| Named issuer | The issuer is the in-process `Upstream` router. No Okta tenant is connected. No Entra tenant is connected. No Google OIDC tenant is connected. |

## Records the fixture creates

The test calls `Fixture::new`, `Fixture::client`, `Fixture::user`, `Core::source_put`, `Core::source_start`, `Core::source_callback`, `Core::source_finish`, `Fixture::tokens`, `Core::userinfo`, and `Core::me` in process. It does not run `riauth source`.

`Fixture::new` copies a store from `Core::initialize`. The administrator username is `admin`, the display name is `Administrator`, `admin` is true, and the password is the fixture constant `test-password-for-fixtures-only`. `Config` is `Default` apart from `data_dir`. The default issuer is `http://localhost:9000` ([src/config.rs](../../src/config.rs)). `Core::initialize` stores `SigningKey::generate`, which uses RS256 ([src/crypto.rs](../../src/crypto.rs), [src/core.rs](../../src/core.rs)).

| Record | Value the helper sets |
| --- | --- |
| Source id and name | `upstream`, `Example upstream` |
| Issuer | `http://127.0.0.1:{port}` from `TcpListener::bind("127.0.0.1:0")` |
| Authorization URL | `{issuer}/authorize`. The router has no `/authorize` route. |
| Token URL | `{issuer}/token`, the only route, `POST` |
| Upstream client | id `upstream-client`, method `ClientSecretBasic` |
| First secret | `source_put` sends `client_secret` `source-client-secret`. That literal exists only in this fixture. |
| Pinned JWKS | `core.jwks()` |
| Signing key the mock uses | The active key from `meta/keys`, the same RS256 key riAuth generated |
| Scopes | `openid`, `profile`, `email` |
| Flags | `enabled` true, `auto_provision` true, `allow_admin_login` false, `groups` left at `Default`, `saml` absent, `oauth_profile` absent |
| Trusted ACR | `urn:upstream:mfa` |
| Local Alice | `Fixture::user("alice")`: email `alice@example.test`, display name `Test User`, `admin` false, the same fixture password. The return value is Alice's session token. |
| Local client `app` | `Fixture::client("app", false)`: public, redirect `http://localhost:7777/callback?existing=1`, scopes `openid`, `profile`, `email`, `groups`, `offline_access`, `require_mfa` false |

`validate_server_url` allows HTTP only for `localhost`, `127.0.0.1`, and `[::1]`, and rejects credentials, queries, and fragments ([src/config.rs](../../src/config.rs)). `Source::validate` maps a rejected URL to `Source URLs must be canonical HTTPS (HTTP only on loopback)` ([src/source.rs](../../src/source.rs)). The issuer and both endpoints in this fixture are loopback HTTP. The test does not try a public HTTP issuer.

## What the token handler asserts

The handler is the test. These `assert_eq` calls run only when the test runs. This revision did not run them.

| Check | Value |
| --- | --- |
| `Authorization` | `Basic ` plus standard base64 of `upstream-client:source-client-secret` |
| `grant_type` | `authorization_code` |
| `redirect_uri` | `http://localhost:9000/oauth/sources/upstream/callback` |
| PKCE | `crypto::digest` of `code_verifier` equals the `code_challenge` copied from the authorization URL. `digest` is unpadded URL-safe SHA-256 ([src/crypto.rs](../../src/crypto.rs)). |

On accept, the handler removes that code from its map and returns JSON with `id_token` and `access_token` set to `mock-access`. The object has no `token_type`, `expires_in`, or `refresh_token`. A later post of the same code is not what this test does. The successful callback in the test requires `completed` true for that body.

`redeem` parses `authorization_url` and asserts `max_age` is `0` and `code_challenge_method` is `S256` before it stores the code. It does not send an HTTP request to `/authorize`. It then calls `Core::source_callback` with query pairs `state`, `code`, and `iss` equal to the ephemeral issuer, and with browser binding `None`.

## Authorization query the server builds

`source_start_for` appends these pairs for an OIDC source ([src/source.rs](../../src/source.rs)). The test reads `max_age` and `code_challenge_method` from that URL. It does not assert the other pairs.

| Pair | Value |
| --- | --- |
| `response_type` | `code` |
| `client_id` | the source client id |
| `redirect_uri` | `{config.issuer}/oauth/sources/{id}/callback`. In this fixture that is `http://localhost:9000/oauth/sources/upstream/callback`. |
| `scope` | the source scopes joined with spaces |
| `state` | a fresh token |
| `code_challenge` | `digest` of the verifier |
| `code_challenge_method` | `S256` |
| `nonce` | a fresh token |
| `max_age` | `0` |

The pending row stores the verifier, the nonce, `started_at`, and `expires_at` of `now()` plus 600. The poll credential uses the prefix `ri_source_`. The start JSON instruction is `Authenticate at the upstream provider, then inspect and finish this request in the CLI`. The test does not assert the prefix, the lifetime, or that instruction.

This test calls `Upstream::start` with no link token. `source_start` then passes browser binding off. The callback binding is `None`. A browser login whose callback omits its binding cookie is a different path, and other functions in the file cover browser completion. This function does not.

## ID token the mock signs

On the empty overlay, the claims are `iss` equal to the ephemeral issuer, `sub` `subject-1`, `aud` the string `upstream-client`, `iat` `now()`, `exp` `now()` plus 300, `auth_time` `now()`, `nonce` copied from the authorization query, `email` `alice@example.test`, `email_verified` true, `name` `Upstream Alice`, and `acr` `urn:upstream:mfa`. `SigningKey::sign` with `access` false sets typ `JWT`. The test does not decode this token. The effects it checks are below.

The signature verifies because the pinned JWKS is riAuth's own JWKS and the signer is riAuth's own active key. The fixture does not generate a separate upstream key pair.

## Assertions the test makes

| Step | What the test requires |
| --- | --- |
| Finish before the callback | `status` is `pending` |
| Empty-overlay callback | `completed` is true. The JSON text does not contain `ri_session_` |
| Finish without approval | `status` is `review`, `mfa` is true, `local_user` is null |
| First approval | `user.id` differs from `Core::me` of Alice |
| Second approval of that same start | `is_err` |
| Local tokens for client `app` | `amr` is `["federated", "mfa"]` after `Fixture::tokens` |
| Later start and callback for `subject-1`, then approval | the same `user.id` |
| Each overlay below | `completed` is false, and approval `is_err` |
| After `enabled` is set false | approval of the already-reviewed start `is_err`; `userinfo` of the access token from the earlier `Fixture::tokens` call `is_err`; `Core::me` of Alice `is_ok` |

`Fixture::tokens` calls `exchange_request`, which also asserts that the local redirect's `state` is `state with & delimiters`, that `iss` equals `issuer::for_client` of the configured issuer and client `app`, and that `existing` is `1`. Those checks belong to the local client `app`. They are not checks of the upstream token.

The review object the server returns also has `issuer`, `subject`, `name`, `email`, `email_verified`, `linking`, `local_otp_required`, and `auto_provision` ([src/source.rs](../../src/source.rs) `complete_source_login`). The test reads `status`, `mfa`, and `local_user`. The approval object also has `status` `complete` and `expires_at`. The test reads `session_token` and `user.id`.

`mfa` true on review follows the ID-token `acr` being listed in `trusted_mfa_acr`. The test does not read the ACR string. `local_user` null, together with a new user id, is the test's check that Alice's identical email does not link her account. The test does not read the provisioned username. The source username is `oidc-` plus `digest` of `link_key`, and `link_key` is `digest` of the source id, a NUL, the issuer, a NUL, and the subject. The generated password is discarded and `password_hash` is cleared. `admin` on that new user is false. The test does not read those fields.

This start has no stage. The callback body is `completed` and the instruction `Return to the CLI to inspect and finish the request`. The success check requires the absence of `ri_session_` in the whole JSON text. It does not require the instruction text.

## Overlays, and the checks the test does not read

Each overlay is a fresh start. `redeem` still requires `max_age=0` and `code_challenge_method=S256`, then replaces claims with `extend`. The callback query `iss` stays the ephemeral issuer. `source_callback` turns a verify failure into `completed: false`. It does not put the verify message in that JSON. `source_finish` then rejects the failed row through `Error::unauthorized()` (`invalid_token`, `Authentication required or session expired`). The test stops at `completed` false and `is_err`.

| Overlay | Verify check the failed token hits |
| --- | --- |
| `nonce` `wrong` | `Upstream nonce or authentication time mismatch` |
| `iss` `https://foreign.example` | `jwks.verify` against the configured issuer returns `JWT signature or claims validation failed`. The signer is still the pinned key. This overlay does not change the callback query `iss`. |
| `azp` `other-client` | `Upstream authorized party mismatch` |
| `aud` `["upstream-client","foreign"]` | `Upstream authorized party mismatch`. `azp` is absent, and `aud` is an array longer than one. |
| `at_hash` `wrong` | `Upstream access token hash mismatch`. The access token is `mock-access`. For a non-EdDSA header the source compares unpadded URL-safe SHA-256 of the access token, first 16 bytes. The test does not compute that digest. |
| `auth_time` `now()-600` | the same nonce and authentication-time message. `auth_time` is 600 seconds before `now()`, outside the 30-second window against `started_at`. |
| `exp` `now()-1` | the same message when `exp <= iat`. The overlay replaces `exp` and leaves `iat` at the `now()` captured for the claim. |

A callback query `iss` that differs from the configured issuer is a different check, `Upstream response issuer mismatch`, and it runs before the token request. This test does not send that query value.

The disable step is also different. The callback has already succeeded. The test then `source_put`s the same source with `enabled` false and `client_secret` `None`. `source_finish` calls `enabled`, which returns `Error::missing` (`not_found`, `Enabled source not found`) before the fingerprint compare. The test does not match that code. `write_source` leaves an existing confidential secret in place when the new secret is absent, and it revokes stored sessions whose source id is this source when the record changes. The test does not read `source_secrets` or the session row. `userinfo` loads the access-token grant and calls `identity_user`, which calls `identity_user_unbound`, which calls `validate_identity`. A missing or disabled source becomes `Error::unauthorized()` there. The test does not match that code either. It also does not call `me` on the federated session token. Alice's `me` stays successful; her session is the local login.

The test does not enable the source again, and it does not present a code while the source is disabled.

## Closest API and CLI

The test does not listen on the HTTP routes and does not run the CLI. The routes in [src/api.rs](../../src/api.rs):

| Call the test makes | Route |
| --- | --- |
| `Core::source_put` | `POST /api/sources` |
| `Core::source_start` | `POST /api/sources/{id}/start` |
| `Core::source_finish` | `POST /api/source-login/finish` |
| `Core::source_callback` | `GET /oauth/sources/{id}/callback` parses the query and also reads a browser binding cookie. This test calls `Core::source_callback` with binding `None` and does not go through that handler. |
| `Core::userinfo` | `GET /oauth/userinfo` |

The CLI counterparts in [src/cli.rs](../../src/cli.rs), also not executed by the test:

```sh
riauth source put --file deployment-private/private-source-input.json
riauth source start upstream --out deployment-private/source-transaction.json
riauth source finish --file deployment-private/source-transaction.json
riauth source finish --file deployment-private/source-transaction.json --yes
```

`source start` can take `--link` and `--authentication-transaction`. `source finish` can take `--otp-stdin`. This test passes neither a link token, nor an authentication transaction, nor an OTP. `require_write` asks `source.write` on `source/{id}` and, when `auto_provision` is true, `user.write` on `*`. The actor in this test is the administrator session from `Fixture::new`. The test does not create an agent. An agent still cannot set `allow_admin_login`; this fixture leaves that flag false, and the test does not try to change it.

## What this fixture leaves open

The issuer process is the in-process axum router. It serves only `POST /token` on loopback HTTP. No Okta, Entra, or Google OIDC tenant is connected. The ID token is signed with riAuth's own RS256 key and checked against riAuth's own JWKS.

This function uses `ClientSecretBasic` only. `Source::validate` accepts `none`, `client_secret_basic`, and `client_secret_post`, and rejects other methods with `Source client authentication supports none, client_secret_basic and client_secret_post`. The test does not store `none` or `client_secret_post`. It does not send an encrypted ID token. [oidc-profiles.md](../oidc-profiles.md) says encrypted upstream ID tokens and upstream `private_key_jwt` remain unsupported. This page does not add a result for either one.

Account linking, manifest redaction, the OAuth-only profile, browser cookie binding, and pinned-key rotation are other functions in the same file. Key rotation is already described in [oidc-profiles.md](../oidc-profiles.md). This function does not rotate keys, and this page does not use those functions' assertions.

## Other D03 recipes

Still without a recipe page: SAML source, outbound SCIM, RADIUS, Workspace, Entra, Shared Signals, device trust, HTTPS client certificates, Vault Transit, Windows device login, the embedded reverse proxy, and shared-domain SSO. The [forward-auth](platform-forward-auth.md), [LDAP provider](platform-ldap-provider.md), [OIDC relying party](oidc-relying-party.md), [SAML IdP](platform-saml-idp.md), [LDAP import](ldap-import.md), and [inbound SCIM](platform-inbound-scim.md) recipes are separate. The inbound SCIM client is in-process `oneshot`, and no named SCIM client is connected. The relying-party client is the in-tree axum fixture, and a named external relying party remains an open peer. The SAML IdP signer is xmlsec1, and a named service provider remains an open peer. The import page follows disposable loopback OpenLDAP, and a real Active Directory directory remains an open peer. Okta, Entra, and Google are not connected to this issuer fixture. D04 emergency runbooks and D05 acceptance are separate work. The [capability matrix](../capability-matrix.md) records the protocol limits those recipes still have to cite.
