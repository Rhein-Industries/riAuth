# ENT-06 Device trust

[Implementation](../../src/device_trust.rs) and [tests](../../tests/device_trust.rs).

Local stand-in for a Chrome Enterprise device-trust signal. A managed Chrome enrollment was **not** tested, and this build does **not** call Google's Verified Access API (`verifiedaccess.googleapis.com`).

The implemented protocol accepts a locally defined signed JWT with the claims below. Compatibility with a managed Chrome challenge or a vendor verification response has not been established. A production integration needs an implemented and reviewed adapter for the actual vendor protocol; replacing the local public key alone is not acceptance evidence.

## Configuration

```toml
[device_trust]
pem_file = "device-trust-public.pem"   # SPKI or PKCS#1; mutually exclusive with jwks_file
algorithm = "ES256"                    # RS256, ES256, or EdDSA; required with pem_file
kid = "local-device"                   # optional; JWT kid must match when set
# jwks_file = "device-trust.jwks"
freshness_ttl = 300                    # seconds, 1..=3600, default 300
```

If `device_trust` is absent, or the key file cannot be loaded, a client with `require_device_trust` fails closed. The check is not skipped.

Set it on the provider:

```json
{ "settings": { "require_device_trust": true } }
```

The default is `false`. Token issuance for clients that leave it unset is unchanged.

## Protocol

1. `POST /api/device-trust/challenge` with the user's bearer session. The response contains a base64url challenge of 32 random bytes, valid for at most 120 seconds and capped by session expiry. Only the hash is stored. The challenge is bound to that exact session and user epoch.
2. The device returns a compact JWT signed by the pinned key:
   - `aud` is the issuer URL
   - `nonce` is the challenge string
   - `exp` is required
   - a device identifier is `device_permanent_id`, `devicePermanentId`, `device_id`, `device_enrollment_id`, or `sub`
   - `kid` must select the pinned JWKS key, or match `kid` when a PEM key sets one
3. `POST /api/device-trust/verify` with the **same bearer session** and `{ "token": "<jwt>" }`. Another session for the same user cannot consume the challenge. Expired tokens, wrong audience, wrong nonce, unknown keys, and a second use of the same challenge are rejected. The challenge is consumed only after all checks succeed, including a second expiry check in the write transaction.
4. A success stores one verification keyed by session id, including user id, epoch, device id, `verified_at`, and `expires_at`. Expiry is the earliest of `verified_at + freshness_ttl`, the accepted JWT's `exp`, and the session's expiry. The device id is fixed for the session's lifetime: renewal requires the same device id; another device requires a new session. Legacy records without a session binding grant no trust and are removed by cleanup.
5. While `require_device_trust` is true, authorization, token issuance (including refresh), and online grant checks require the originating session's verification, a matching user epoch, an active session, and a loadable verifier key. A different session has to complete its own challenge. Token issuance fails with `unmet_authentication_requirements` when trust is absent or stale.

`GET /api/policy/explain` is a username-based simulation without a session proof. For a protected client it reports `device_trust_session_required`, or `device_trust_verifier_unconfigured` if the verifier cannot be loaded; it never borrows another session's trust.

This binds the accepted device signal to a session, not every HTTP request to hardware. The session remains a bearer credential; a copied session can be used until revoked or expired. Hardware proof of possession and a vendor adapter require separate integration and acceptance.

The same originating-session gate applies to portal launch, SAML and proxy access. LDAP password binds and RADIUS authentication create their own sessions and currently have no device-challenge exchange for those sessions. A client requiring device trust therefore rejects those user authentications even when another browser or CLI session is trusted. Scoped LDAP service searches remain governed by their agent permissions.

## What a real Chrome test still requires

- A managed Chrome browser enrolled through Google Admin / Chrome Enterprise.
- An implemented vendor protocol adapter, including Verified Access integration when required; the local compact-JWT contract is not a verified vendor contract.
- An end-to-end challenge exchange with that browser and verified binding of the resulting signal to the requesting device/session.
- Evidence that a forged, replayed, or stale signal is rejected against that production key.

Local tests in `tests/device_trust.rs` use an openssl P-256 key and an in-process JWKS. They do not establish managed Chrome compatibility.
