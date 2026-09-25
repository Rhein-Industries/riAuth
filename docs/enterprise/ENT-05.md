# ENT-05 HTTPS client-certificate authentication

[Implementation](../../src/mtls.rs) and [tests](../../tests/mtls.rs).

riAuth can authenticate an enrolled user from a client certificate on `POST /api/login/certificate` and open a normal session. The profile is optional. When `client_certificates` is absent, the HTTPS listener stays on `with_no_client_auth` and the login route reports that the feature is not configured.

This is not EAP-TLS enrollment. RADIUS keeps `certificate.read`, `certificate.write`, and `radius.enroll`. HTTPS bindings use `mtls.read` and `mtls.bind` on `user/{username}`.

## Configuration

```toml
tls_cert_file = "tls.pem"
tls_key_file = "tls.key"
trusted_proxies = ["192.0.2.10"]

[client_certificates]
trust_anchors_file = "client-ca.pem"
crl_file = "client-ca.crl"          # optional
forwarded_header = "X-Client-Cert"  # optional; requires trusted_proxies
mode = "optional"                    # or "required"
```

`trust_anchors_file` is a PEM bundle of CA certificates. Chain building uses only those anchors. The system trust store and `webpki-roots` are not consulted.

`mode` is `optional` (default) or `required`. Both modes build the rustls 0.23 verifier with `WebPkiClientVerifier::allow_unauthenticated`. The listener requests a client certificate and still completes anonymous handshakes, so browser OpenID Connect login, password login, health checks, and every other route keep working. `required` does **not** mean "reject TCP connections that do not present a certificate." It only records that `POST /api/login/certificate` is expected to present one. A missing certificate fails that route alone, in either mode, with `certificate_required`.

Relative paths in the profile are resolved from the configuration file's directory, like the server TLS files.

When the profile is enabled, TLS session resumption is disabled on that listener so a resumed handshake cannot drop the peer certificate. Deployments that leave the profile unset keep the default session cache.

## Termination topologies

1. **Native TLS.** riAuth terminates HTTPS. After the handshake, the peer certificate chain is copied from the rustls connection into the request extensions. `X-Client-Cert`, `Forwarded`, and any other certificate header are ignored unless `forwarded_header` is set **and** the immediate TCP peer is one of `trusted_proxies`.

2. **TLS terminated at a trusted proxy.** Set `forwarded_header` to the header that carries the client certificate PEM (leaf first, then intermediates). Percent-encoded PEM is decoded once. The header is accepted only from an immediate peer listed in `trusted_proxies`. riAuth verifies that PEM against `trust_anchors_file` and the optional CRL. A header is not proof by itself. If native TLS is not configured, `forwarded_header` is required; the issuer may be HTTP only on loopback, under the existing issuer rules.

3. **Direct client plus a forged header.** A peer that is not in `trusted_proxies` cannot authenticate with the header. The header is not parsed for that peer. If that peer also presents a TLS client certificate, only the TLS certificate is verified.

When the peer is trusted and `forwarded_header` is set, certificate login uses the header and does not fall back to the proxy's own TLS client certificate. A missing or duplicate header fails the certificate login. Duplicate values are `invalid_request` and do not authenticate.

## Enrollment

A certificate from a trusted CA is not a user. An administrator or an agent with `mtls.bind` on `user/{username}` enrolls one binding per user:

- `POST /api/certificates` with `username` and at least one selector: `certificate_pem` (leaf first), `san_uri`, or `san_email`
- `GET /api/certificates` requires `mtls.read`
- `DELETE /api/certificates/{id}` requires `mtls.bind`

CLI: `riauth certificate bind|list|revoke`.

The fingerprint is SHA-256 of the leaf DER, base64url without padding. Re-binding the same user replaces the binding and revokes sessions that were opened with the previous binding. Password sessions are left in place. An unchanged binding is idempotent.

If a PEM is supplied, it is verified with the same trust anchors and CRL as login, and every SAN in the request must appear on that leaf. SAN-only enrollment does not require a certificate at bind time. At login, every constraint that is set must match: fingerprint, exact email SAN, and exact URI SAN. Email and URI comparison is case-sensitive. Subject CN, DNS SAN, and a username-shaped CN do not select a user. Agents cannot bind or revoke an administrator.

## Login and assurance

`POST /api/login/certificate` accepts an optional JSON body `{ "transaction_id": "..." }` for the same browser authentication-transaction handoff as password login. An empty body is allowed.

Success returns the same session shape as password login: `session_token`, `expires_at`, and `user`. The session is stored with `amr: ["cert"]` and `mfa: false`. `amr` value `cert` selects the existing ACR `urn:riauth:acr:certificate` (the same value EAP-TLS selects via `amr` `x509`). It is not a higher ACR. Discovery advertises that ACR only when this profile is configured. Clients may set it in `default_acr_values`.

Certificate login does not satisfy `require_mfa` or `urn:riauth:acr:mfa`. It does not prove a password or a one-time code. It does not reset password attempt counters. The route shares the login rate limit. Disabled users and epoch changes fail. The session expires at the earlier of `session_ttl` and the leaf `notAfter`.

While the binding remains unchanged, later use of the session checks that the binding id and selector still match. Revoke and rotation invalidate those sessions. RADIUS `x509` session checks are not applied to `amr` `cert`.

## Revocation and reload

`crl_file` is optional. If it is omitted, revocation is **not** checked. If it is set, the file must contain 1..=16 PEM CRLs with `thisUpdate` not in the future and `nextUpdate` strictly in the future. rustls checks revocation for the built chain, with CRL expiry enforced. A CRL that does not establish status fails closed.

The HTTPS server reloads its rustls configuration, including the handshake verifier and its CRL, about every 60 seconds. Certificate login reads the trust anchors and CRL from disk on every attempt, so header-mode certificates and logins after a handshake are not delayed by that reload. A certificate the current handshake verifier rejects never reaches the login handler; presenting it fails the connection for every route. A certificate that was acceptable at handshake and is later revoked is rejected at the next certificate login even if the process has not reloaded the listener yet.

OCSP is not fetched or honored.

## What is not verified

- Membership in the trust anchor alone
- Subject CN, DNS SAN, or "CN equals username"
- System or public roots
- Forwarded headers from any peer outside `trusted_proxies`
- The proxy's own TLS certificate when header mode applies to that peer
- Revocation, when `crl_file` is unset
- OCSP, delta CRLs, and indirect CRLs beyond what rustls rejects
- That an existing session's certificate is still unrevoked, until the binding is revoked or rotated, the user is disabled, or `notAfter` / session expiry passes
- MFA, password, or a second factor
- A higher ACR than `urn:riauth:acr:certificate`
- Declarative desired-state enrollment (bindings are store records and are included in encrypted database backups)

A trusted proxy can present any enrolled certificate. That is the same trust boundary as `trusted_proxies` for `X-Forwarded-For`.
