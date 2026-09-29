# ENT-06 Device trust

[Implementation](../../src/device_trust.rs), [Verified Access adapter](../../src/verified_access.rs), and [tests](../../tests/device_trust.rs).

Device trust is Platform-only. The default provider is a local compact-JWT stand-in. `provider = "google_verified_access_v2"` selects a separate adapter for the Chrome Verified Access API v2. This adapter was not executed against `verifiedaccess.googleapis.com` or a managed Chrome device, so I07 stays open.

## Local verifier

```toml
[device_trust]
provider = "local"                     # default when omitted
pem_file = "device-trust-public.pem"   # SPKI or PKCS#1; mutually exclusive with jwks_file
algorithm = "ES256"                    # RS256, ES256, or EdDSA; required with pem_file
kid = "local-device"                   # optional; JWT kid must match when set
# jwks_file = "device-trust.jwks"
freshness_ttl = 300                    # seconds, 1..=3600, default 300
```

`provider = "local"` rejects `service_account_file`, `expected_identity`, `customer_id`, and a non-empty `allowed_key_trust_levels`.

If `device_trust` is absent, or the verifier cannot be loaded, a client with `require_device_trust` fails closed. The check is not skipped.

Set it on the provider:

```json
{ "settings": { "require_device_trust": true } }
```

The default is `false`. Token issuance for clients that leave it unset is unchanged.

### Local protocol

1. `POST /api/device-trust/challenge` with the user's bearer session. The response contains a base64url challenge of 32 random bytes, `audience`, and `"provider": "local"`. It is valid for at most 120 seconds and capped by session expiry. Only the hash is stored. The challenge is bound to that exact session and user epoch. At most eight challenges are outstanding for one user.
2. The device returns a compact JWT signed by the pinned key:
   - `aud` is the issuer URL
   - `nonce` is the challenge string
   - `exp` is required
   - a device identifier is `device_permanent_id`, `devicePermanentId`, `device_id`, `device_enrollment_id`, or `sub`
   - `kid` must select the pinned JWKS key, or match `kid` when a PEM key sets one
3. `POST /api/device-trust/verify` with the **same bearer session** and `{ "token": "<jwt>" }`. A body that carries `challenge` or `challenge_response` is rejected. Another session for the same user cannot consume the challenge. Expired tokens, wrong audience, wrong nonce, unknown keys, and a second use of the same challenge are rejected. The challenge is consumed only after all checks succeed, including a second expiry check in the write transaction. A different device id does not consume the challenge.
4. A success stores one verification keyed by session id, including user id, epoch, device id, `verified_at`, and `expires_at`. Expiry is the earliest of `verified_at + freshness_ttl`, the accepted JWT's `exp`, and the session's expiry. The device id is fixed for the session's lifetime: renewal requires the same device id; another device requires a new session. Legacy records without a session binding grant no trust and are removed by cleanup.
5. While `require_device_trust` is true, authorization, token issuance (including refresh), and online grant checks require the originating session's verification, a matching user epoch, an active session, and a loadable verifier. A different session has to complete its own challenge. Token issuance fails with `unmet_authentication_requirements` when trust is absent or stale.

## Chrome Verified Access v2

The REST reference for [challenge.generate](https://developers.google.com/chrome/verified-access/reference/rest/v2/challenge/generate) and [challenge.verify](https://developers.google.com/chrome/verified-access/reference/rest/v2/challenge/verify) was last updated 2025-03-20 UTC. The [developer guide](https://developers.google.com/chrome/verified-access/developer-guide) was published 2024-10-16 UTC.

- `POST https://verifiedaccess.googleapis.com/v2/challenge:generate` takes an empty body. Success is `{ "challenge": "<base64 SignedData>" }`.
- `POST https://verifiedaccess.googleapis.com/v2/challenge:verify` takes `{ "challengeResponse": "<base64 SignedData>", "expectedIdentity": "<optional>" }`.
- The OAuth scope is `https://www.googleapis.com/auth/verifiedaccess`.

The developer guide says a challenge is a Google-signed blob good for one minute. Its extension sample requests generate with an API key (`?key=`). This server calls generate itself with the service-account bearer. It does not put an API key on the URL, and configuration has no base URL. The only hosts are `verifiedaccess.googleapis.com` and `oauth2.googleapis.com`, HTTPS, with no query and no fragment. The client does not follow redirects and does not use an ambient proxy. Only HTTP 200 is success. Any other status or a transport failure is HTTP 503 `verified_access_unavailable`, message "Verified Access is unavailable". That message does not include the remote body.

Google's verify request does not include the original challenge, and the success body does not echo one. The caller still sends the issued challenge so this server can resolve the session, epoch, and replay state. The v2 `challengeResponse` is a base64 `SignedData` blob. Chromium's `device_trust_attestation_ca.proto` and AOSP `attestation_ca.proto` wrap a `ChallengeResponse` in that `SignedData`: field `data` is the serialized `ChallengeResponse`, and field `signature` is the device key over that payload (RSASSA-PKCS1-v1_5-SHA256). `ChallengeResponse` field 1 is the issued `SignedData`. Before verify, this adapter parses both blobs and requires those embedded data and signature bytes to equal the issued challenge. A response for a different challenge is rejected with "Verified Access response does not answer this challenge"; the challenge stays unused and verify is not called. Bytes that do not parse as that embedding are "Verified Access challenge response is invalid". The device public key stays inside `encrypted_key_info`, so the outer device signature is checked when Google accepts the original response bytes. Google's challenge-signing key is not pinned. The developer guide says Google checks ChromeOS origin, enterprise management, the expected identity when provided, freshness of at most one minute, administrator policy, and caller permission. Those checks were not observed against the live API.

```toml
[device_trust]
provider = "google_verified_access_v2"
service_account_file = "verified-access.json" # owner-only, at most 16 KiB
expected_identity = "devices.example.com"     # enrolled device domain
customer_id = "C01234567"
allowed_key_trust_levels = ["CHROME_OS_VERIFIED_MODE"]
freshness_ttl = 300                            # local trust lifetime, 1..=3600
```

This provider rejects `pem_file`, `jwks_file`, `algorithm`, and `kid`. `expected_identity` is the enrolled device domain: a hostname with a dot, no `@`, and no scheme. An email address is rejected, so user/EUK verification is not accepted. `customer_id` is 1..=64 characters of ASCII letters, digits, `_`, and `-`.

`allowed_key_trust_levels` lists one to three distinct values from `CHROME_OS_VERIFIED_MODE`, `CHROME_BROWSER_HW_KEY`, and `CHROME_BROWSER_OS_KEY`. `KEY_TRUST_LEVEL_UNSPECIFIED`, `CHROME_OS_DEVELOPER_MODE`, and `CHROME_BROWSER_NO_KEY` are rejected even if a configuration names them. The response field must be that string, not a protobuf enum number.

The service-account file is JSON with `type` `service_account` and `token_uri` exactly `https://oauth2.googleapis.com/token`. When `auth_uri` is present it must be `https://accounts.google.com/o/oauth2/auth`. When `universe_domain` is present it must be `googleapis.com`. `client_email` is `name@project.iam.gserviceaccount.com`. `private_key` is one PKCS#8 `PRIVATE KEY` block whose RSA modulus is 256..=1024 bytes. Extra fields such as `project_id` and `client_id` are ignored. The file is read again on each generate and verify. The token request is `grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer` to `https://oauth2.googleapis.com/token`: RS256, `kid` from `private_key_id`, `iss` the client email, `scope` the Verified Access scope, `aud` the token URL, and no domain-wide delegation `sub`. A returned `token_type` must be `Bearer`. `expires_in` must be 60..=3600 seconds. If `scope` is present it must include the Verified Access scope. The access token is cached until `expires_in - 60` seconds for that key fingerprint.

After parsing, the raw service-account JSON is wiped. The parsed private key is wiped when the account value is dropped. A Verified Access request wipes its bearer and body on drop. Copies the HTTP client holds for the call, and bytes left by an earlier reallocation, stay outside that wipe.

### Adapter protocol

1. `POST /api/device-trust/challenge` with the user's bearer session. The server checks the session and that the user has fewer than eight outstanding challenges, then calls generate. The returned challenge must be canonical standard base64 of a `SignedData` protobuf with non-empty `data` and `signature`. Anything else is discarded and reported as unavailable, and no challenge row is stored. A challenge that parses is stored by the hash of that canonical string, bound to that user, session, and epoch. `issued_at` is the local clock. `expires_at` is the earlier of one minute and the session expiry. The JSON contains `challenge`, `expires_in`, `expires_at`, and `"provider": "google_verified_access_v2"`, and it omits `audience`. If the session is gone or the cap is full after generate returns, the Google challenge is discarded. A duplicate of a challenge string already stored is rejected and not rebound.
2. `POST /api/device-trust/verify` with the same session and `{ "challenge": "<echo>", "challenge_response": "<base64>" }`. `token` is rejected, as is a `challenge_response` that contains `.`. The echo must be the canonical challenge this server issued for this session and epoch, still unused, with `issued_at` non-zero and younger than one minute. The same response bytes are denied while a prior acceptance of those bytes is retained. After that binding, the response must parse as `SignedData` whose `data` is a `ChallengeResponse`, and field 1 of that message must be a `SignedData` with the same `data` and `signature` bytes as the issued challenge. A mismatch returns "Verified Access response does not answer this challenge" and leaves the challenge unused. A parse failure returns "Verified Access challenge response is invalid". Neither case calls verify.
3. The server posts the original `challengeResponse` bytes and `expectedIdentity` as JSON. A success requires string `devicePermanentId` (the only device id stored), `customerId` equal to the configured customer, and `keyTrustLevel` in both the fixed acceptable set and the operator allowlist. `virtualDeviceId`, `deviceEnrollmentId`, `attestedDeviceId`, and `profilePermanentId` are not substitutes. The REST reference says a managed profile on an unmanaged browser omits `devicePermanentId`, `keyTrustLevel`, `virtualDeviceId`, and `customerId` and returns `profileCustomerId`, `virtualProfileId`, `profilePermanentId`, and `profileKeyTrustLevel` instead. That shape is rejected. `deviceSignals` are not evaluated. The challenge and response size limits in this adapter are local limits, not a published Google size.
4. The challenge is consumed only after those checks succeed. A negative verify result leaves the challenge unused. When Google returns a device id that differs from the verification already stored for the session, the challenge is consumed, the response hash is retained for one minute, no verification is written, and the caller receives "A different device requires a new session".
5. Success stores one verification keyed by session id. `expires_at` is the earlier of `verified_at + freshness_ttl` and the session expiry. `freshness_ttl` is the local trust lifetime. It is separate from Google's one-minute challenge lifetime.

The service-account file is an external credential. The backup archive stores its path, not its contents, the same way it stores a device-trust PEM path. `config.validate` reads the PEM, the JWKS, or this file.

## Shared behavior

`GET /api/policy/explain` is a username-based simulation without a session proof. For a protected client it reports `device_trust_session_required`, or `device_trust_verifier_unconfigured` if the configured provider cannot be loaded; it never borrows another session's trust.

This binds the accepted device signal to a session, not every HTTP request to hardware. The session remains a bearer credential; a copied session can be used until revoked or expired.

The same originating-session gate applies to portal launch, SAML and proxy access. LDAP password binds and RADIUS authentication create their own sessions and have no device-challenge exchange for those sessions. A client requiring device trust therefore rejects those user authentications even when another browser or CLI session is trusted. Scoped LDAP service searches remain governed by their agent permissions.

Local tests in `tests/device_trust.rs` use an openssl P-256 key, an in-process JWKS, and an in-process Verified Access transport. They do not establish managed Chrome compatibility, and they do not call `verifiedaccess.googleapis.com`.
