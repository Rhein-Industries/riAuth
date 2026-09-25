# ENT-12 — OAuth for outbound SCIM

[Implementation](../../src/provisioning.rs) and [tests](../../tests/scim_oauth.rs).

Outbound SCIM targets authenticate in exactly one way:

- `token_file`: a static bearer token, reread from that private file on every request
- `oauth`: an access token acquired from a token endpoint

Do not set both. The password grant is not supported.

```toml
[scim_targets.payroll]
url = "https://payroll.example.com/scim/v2"
groups = ["payroll-users"]
export_groups = true
# ca_file = "payroll-scim-ca.pem"

[scim_targets.payroll.oauth]
token_url = "https://id.example.com/oauth/token"
grant = "client_credentials"
client_id = "riauth-payroll"
client_secret_file = "payroll-client-secret"
scope = "scim:write"
audience = "https://payroll.example.com/scim"
# ca_file = "token-endpoint-ca.pem"
```

`token_url` must be canonical HTTPS, or HTTP on loopback. Relative secret and CA paths are resolved from the configuration file's directory. `client_secret_file` and `refresh_token_file` must be regular files of at most 4096 bytes with mode `0600` or `0400` (owner-read/write only). A CA file is a PEM bundle, not a secret-file mode check. When `oauth.ca_file` is omitted, the target `ca_file` is used for the token endpoint as well.

## Grants

`client_credentials` requires `client_secret_file` and must not set `refresh_token_file`. The token request is `application/x-www-form-urlencoded` and sends `grant_type`, `client_id`, `client_secret`, and the configured `scope` and `audience` when those are set.

`refresh_token` requires `refresh_token_file`. `client_secret_file` is optional. The file is read on every acquisition. If the token response includes a new `refresh_token`, riAuth discards it and does **not** rewrite `refresh_token_file`. Rotate that file outside the process; the next acquisition picks up the new contents.

Redirects from the token endpoint are not followed.

## Expiry, cache, and failure

The access token stays in process memory, keyed by target name and a generation counter. It is not written to the database or the plan. The database record `scim_oauth_cache` stores only `expires_at` and a hash of the access token.

`expires_in` is honored. A missing `expires_in` is treated as 60 seconds. The cached token is refreshed 30 seconds early, so a token whose lifetime is 30 seconds or less is not reused. Replacing the contents of `client_secret_file`, `refresh_token_file`, or `token_file` is picked up on the next acquisition without a restart. Changing a configured path or target setting requires loading the revised server configuration. Each process keeps its own cache; a mutex per target stops concurrent deliveries in that process from stampeding the token endpoint.

The token endpoint is attempted at most twice per acquisition. HTTP 5xx and connection failures are retried once; other token-endpoint errors are not. The durable provisioning job then uses its existing backoff. A SCIM `401` drops the cached access token, acquires once more, and retries that request once. A second `401` fails the attempt.

riAuth will not call SCIM when the token endpoint returns an error, a non-bearer `token_type`, or an empty token. When `scope` or `audience` is configured, a scope or audience echoed in the token response, or a `scope` / `scp` / `aud` claim in a JWT-shaped access token, must cover that configuration. The access token signature is not verified here; the SCIM server does that. An opaque token with no echoed scope or audience is accepted after the request parameters were sent.

## Redaction

Plans, audit events, job errors, and error text do not include the access token, refresh token, client secret, or token-endpoint body. Token-endpoint failures report a fixed message and, for HTTP errors, only the status code.
