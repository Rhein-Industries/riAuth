# RADIUS and RadSec

riAuth serves PAP and EAP-TLS RADIUS from the same Rust service as OIDC. Policy, group membership, password verification, TOTP/recovery codes and identity revocation use the identity store. Configure listeners in the private server TOML, and manage their policy clients through the normal agent client settings or manifests.

```toml
[radius_listeners.lan]
listen = "192.0.2.10:1812"
transport = "udp"

[radius_listeners.lan.nas.switch]
peer = "192.0.2.20"
client_id = "network"
shared_secret_file = "secrets/switch-radius"
```

The service address 192.0.2.10 and NAS address 192.0.2.20 are documentation placeholders; replace them with addresses in your deployment.

The shared secret must contain at least 32 non-whitespace ASCII bytes in an owner-only regular file. Use a distinct randomly generated secret for each NAS. riAuth requires Message-Authenticator on **every** Access-Request, verifies it before password processing, and puts it first in every response. An unregistered source, malformed packet or invalid authenticator receives no response. No legacy mode disables this check.

A policy client needs `openid` and `radius` scopes, `service: false`, and `settings.radius`. It can have an empty redirect list. Normal `allowed_groups`, `require_mfa`, default ACR, device-trust requirements and `policy.access`/`policy.scopes.radius` restrictions apply. RADIUS creates a new session and currently has no exchange for the session-bound device-trust challenge; enabling `require_device_trust` therefore rejects authentication, including when another browser or CLI session is trusted. Active [temporary group grants](enterprise/ENT-01.md) can satisfy group policy until expiry/revocation; an established NAS session is still controlled by the NAS. Example reply settings:

```json
{
  "radius": {
    "reply": [
      {"kind":"standard","code":6,"value":{"type":"integer","value":2}},
      {"kind":"standard","code":64,"value":{"type":"integer","value":13}},
      {"kind":"standard","code":65,"value":{"type":"integer","value":6}},
      {"kind":"standard","code":81,"value":{"type":"text","value":"100"}},
      {"kind":"vendor","vendor":12345,"code":1,"value":{"type":"attribute","value":"network_role"}}
    ]
  }
}
```

Reply values are typed text, unsigned integers, IPv4 addresses or explicitly selected user attributes. Standard wire types are checked. Tunnel attributes use tag zero; VLAN IDs are strings. Credential, EAP and authenticator attributes are reserved. Vendor attributes use the standard four-byte vendor ID and one-byte type/length encoding.

For a TOTP-enrolled user, send `password;OTP` (or `password;recovery-code`). Passwords for users without TOTP are not split. An exact authenticated packet retry reuses its response for 90 seconds without consuming a factor again; a new packet must use a fresh OTP. Successful retries still check current client configuration, policy, session, user and reply values. Shared PostgreSQL carries this duplicate state across instances. A crash between factor consumption and reply persistence can require a new authentication attempt; in-flight authentication is not a distributed exactly-once transaction.

Prefer mutual-TLS RadSec where the NAS supports it:

```toml
[radius_listeners.radsec]
listen = "192.0.2.10:2083"
transport = "tls"
tls_cert_file = "tls/radius.pem"
tls_key_file = "tls/radius.key"
client_ca_file = "tls/private-nas-ca.pem"

[radius_listeners.radsec.nas.switch]
peer = "192.0.2.20"
client_id = "network"
certificate_sha256 = "BASE64URL_SHA256_OF_NAS_LEAF_CERT_DER_WITHOUT_PADDING"
```

Both the private CA chain and exact leaf certificate pin must match. The NAS must verify the server certificate and hostname. RFC 6614 uses the fixed internal RADIUS secret `radsec`; do not configure a UDP shared secret for this transport. Certificates are reloaded every 60 seconds; an invalid replacement retains the active configuration. TLS is mandatory on this listener, with no plaintext downgrade.

The listener limits packet size, connection count, concurrent authentication, peer request rates and timeouts. It sends Access-Accept/Reject for PAP and EAP-TLS, and Access-Challenge for EAP. Accounting, CHAP/MS-CHAP, PEAP/TTLS, dynamic authorization/CoA and active termination of established NAS sessions are not implemented by this profile. A NAS enforces reply attributes and its own session lifetime.

The network regression test uses an independent OpenSSL packet authenticator and TLS client against actual UDP/RadSec listeners. It covers MFA, duplicate/replay handling, missing/bad authenticators, group revocation, VLAN/vendor encoding and certificate pinning. Actual switch/AP/VPN interoperability remains a deployment check.

Protocol references: [RADIUS](https://www.rfc-editor.org/rfc/rfc2865.html), [Message-Authenticator](https://www.rfc-editor.org/rfc/rfc3579.html), [tunnel attributes](https://www.rfc-editor.org/rfc/rfc2868.html), [RadSec](https://www.rfc-editor.org/rfc/rfc6614.html), [Authentik RADIUS](https://docs.goauthentik.io/add-secure-apps/providers/radius/).


## Certificate authentication with EAP-TLS

Enable `settings.radius.eap_tls: true` on the policy client and configure a private client trust profile on either UDP or RadSec. These certificates authenticate the end user/device; RadSec's NAS certificate is a separate outer transport identity. [HTTPS client-certificate login](enterprise/ENT-05.md) has its own trust configuration and enrollment API; enrolling there does not enroll this EAP-TLS profile.

```toml
[radius_listeners.lan.eap_tls]
certificate_file = "tls/eap-server.pem"
key_file = "tls/eap-server.key"
client_ca_file = "tls/device-ca.pem"
client_crl_file = "tls/device-ca-crls.pem"
# Optional DER OCSP response to staple for the server certificate:
# ocsp_response_file = "tls/eap-server.ocsp"
tls12 = false
fragment_size = 1024
```

TLS 1.3 is the default. Explicit `tls12: true` also permits TLS 1.2 with Extended Master Secret. TLS 1.0/1.1, early data, tickets and resumption are disabled. Clients must verify the configured server CA, server SAN and revocation status. riAuth requires current PEM CRLs covering the entire client chain except its trust anchor; unknown, revoked, expired or not-yet-valid status fails closed. Update these files atomically before `nextUpdate`. The private server key requires owner-only permissions. File changes are loaded for each new handshake; changed trust material invalidates existing pending handshakes and cached accepts.

Bind each CA-validated leaf certificate to an explicit existing user:

```sh
riauth radius bind-certificate alice --listener lan --file alice-chain.pem
riauth radius certificates
riauth radius revoke-certificate CERTIFICATE_ID
riauth --json schema radius-certificate
```

The fingerprint is SHA-256 of leaf DER. A certificate cannot be adopted by another identity. CN, email, outer EAP identity and RADIUS User-Name do not select the local user. Anonymous outer identities are supported; successful replies carry the authenticated local username. Agents need `certificate.write` on `user/alice` and `radius.enroll` on `radius/lan`; inventory requires `certificate.read` on the user and the listener permission. Agents cannot enroll administrator identities. Normal revision conditions, idempotency receipts and audit attribution apply to these management operations.

Certificate authentication has AMR `x509` and ACR `urn:riauth:acr:certificate`. It does not satisfy a password or MFA policy. This ACR may be selected as the RADIUS client's default. The resulting short-lived private identity is never issued as an HTTP bearer credential. Authorization checks live user state, group/scope policy, client settings, certificate binding, expiry and trust-file fingerprint again before Access-Accept and on its cached retries.

The service handles EAP-Start, anonymous Identity, EAP-TLS fragmentation and acknowledgements, TLS 1.3 protected success, EAP Success/Failure and RFC 2548 MS-MPPE receive/send keys. The TLS exporter produces the full 128-byte EAP key material before selecting the MSK; encrypted RADIUS keys use distinct salts. Handshakes have a 120-second lifetime, a 64 KiB flight limit and a 256-round bound. Pending TLS state lives in the serving process, so configure NAS affinity; restart incomplete authentication after process failover. Completed duplicate responses use shared storage. Certificate revocation affects subsequent authentication and retries; the NAS controls existing network sessions.

The automated independent OpenSSL peer verifies both TLS versions, bidirectional fragmentation, every duplicate flight, protected success and independently decrypted/exported MS-MPPE keys. Negative cases cover missing/unregistered certificates, TLS downgrade, certificate/CRL revocation, group changes and MFA enforcement. Compatibility with switches, access points and operating-system supplicants requires a device matrix for the intended deployment (see [testing](testing.md)). The independent OpenSSL fixture is not evidence that those hardware/OS combinations were run.

References: [EAP-TLS 1.2](https://www.rfc-editor.org/rfc/rfc5216.html), [EAP-TLS 1.3](https://www.rfc-editor.org/rfc/rfc9190.html), [MS-MPPE keys](https://www.rfc-editor.org/rfc/rfc2548.html).
