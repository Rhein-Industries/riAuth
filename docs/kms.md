# External signing with Vault Transit

A configured Vault Transit key can sign ID tokens, access tokens, UserInfo, JARM and back-channel logout. Its private key is never imported into riAuth. The database stores a signer name, exact key version and pinned public JWK. Each returned signature is independently verified before use; an unavailable service, unexpected version or wrong signature fails token issuance.

Configure a key version in `riauth.toml`:

```toml
[signers.production-v7]
address = "https://vault.example.com"
mount = "transit"
key_name = "oidc"
key_version = 7
token_file = "vault-token"
ca_file = "vault-ca.pem"

[signers.production-v7.public_jwk]
kty = "RSA"
alg = "RS256"
kid = "oidc-v7"
use = "sig"
n = "BASE64URL_RSA_MODULUS_FROM_YOUR_PUBLIC_KEY"
e = "AQAB"
```

Provision the non-exportable key and narrowly scoped Vault token in Vault first. The example modulus is a placeholder and fails validation. An optional `namespace` selects the Vault namespace. Paths resolve beside the configuration; owner-only credential files (at most 4096 bytes) are read for signing and can be rotated without putting the credential into a manifest or database. All service nodes need the same signer definition and access.

```sh
riauth --if-revision 42 --idempotency-key bind-signing-v7 keys bind signing --signer production-v7 --algorithm RS256
# Or bind a provider-specific signing domain:
riauth --if-revision 43 --idempotency-key bind-payroll-v7 keys bind payroll --signer production-v7 --algorithm RS256
```

Use the current values from `riauth revision` in place of 42 and 43. Each logical write needs its own stable key; retry the same command with its original revision and key. The existing `key.write` permission controls binding. Agents can select only signer names already configured by the server operator, not supply network endpoints or arbitrary credential paths. A new binding signs and verifies a challenge after authorization, revision and idempotency checks. An authenticated retry with a matching saved receipt returns that result without contacting Vault, even during an outage. New key versions should be configured under distinct signer names and kids; bind the new version after configuration is present on every node. Rotation retains old public verification keys through the token/session retention window. The serving `rotate-key` retention window and the effects it leaves in place are in [credential compromise](credential-compromise.md).

Supported algorithms are RS256 with PKCS#1 v1.5/SHA-256, ES256/P-256 with the JWS signature representation, and Ed25519/EdDSA. These parameters follow the [Vault Transit signing API](https://developer.hashicorp.com/vault/api-docs/secret/transit). HTTP loopback is accepted for local fixtures; other addresses require HTTPS, certificate verification and no redirects. Each signing request has a three-second timeout and a 64 KiB response limit. Key-service response bodies and credentials are not logged.

This is a Vault Transit integration, not an assertion that the configured Vault uses hardware custody or that an HSM has been tested. Native PKCS#11 and AWS/GCP/Azure-specific KMS adapters are not implemented. Tests use an independent OpenSSL signer behind a local Transit API fixture and cover all three algorithms, version mismatches, invalid signatures, unavailable service and retry after transaction rollback.

Encrypted database backups preserve the remote signer name, version and public pin,
plus configuration file references. They cannot restore the Vault key or bearer
credential. Restore never contacts Vault, and a server without the matching signer
starts but answers token requests with `signer_unavailable`. Re-provision the exact
signer and test issuance during recovery; a successful local restore/JWKS check
alone does not prove that Vault can sign. See [disaster recovery](disaster-recovery.md).
`riauth_signing_errors_total` and the optional signing-failure alert signal report
process-local signing failures; see [operations](operations.md).

## Failure reasons

Each failed remote signing attempt also counts under exactly one fixed reason, in
`riauth_remote_signing_failures_total{reason="..."}` and in
`runtime.remote_signing_failures` of `riauth metrics`. It also writes one
`remote signing failed` warning that carries only the reason. A key bind whose
test signature fails counts the same way; a bind refused before that signature,
for example for an unknown signer, is not counted. Neither the reason nor
the warning contains a signer, key or domain name, URL, path, token, claims,
response body, status code or error text. The public error is unchanged: the API
still returns the status, code and message described in
[connector incidents](connector-incidents.md#vault-transit).

In every case the request fails closed: no token is issued, already issued tokens
stay valid, sessions are not revoked, and JWKS and verification do not call Vault.
The checks below are things to look at; a reason does not prove a cause.

| Reason | When | Check |
| --- | --- | --- |
| `stored_key` | The stored remote key's metadata is inconsistent. Nothing was sent. | Do not edit the store by hand. Restore from a verified backup, or bind a new key with `riauth keys bind` after review. |
| `configuration_binding` | No configured signer has the stored signer name, `key_version` and `public_jwk`. Nothing was sent. | Restore that `[signers]` entry on this node exactly as it was bound. A different version or pin needs a reviewed new bind, not a pin edit. |
| `signer_configuration` | The configured signer fails validation when it is used. Nothing was sent. | The signer's address, mount, key name, `key_version` and namespace. |
| `credential_read` | The token file is missing, is not an owner-only regular file, is larger than 4096 bytes, or cannot be read as text. Nothing was sent. | The token file's path, owner, mode and size. |
| `credential_shape` | The token is empty after trimming, too long, or not ASCII graphic. Nothing was sent. | That the token file holds only the Vault token. |
| `ca_setup` | The configured `ca_file` could not be read. Nothing was sent. | The CA file's path and permissions. |
| `client_setup` | The HTTP client could not be built, for example because `ca_file` holds a malformed PEM block. Nothing was sent. | The CA file's contents. A file with no PEM block adds no certificate. |
| `transport` | The request was not sent, or its response or body was not received: connection, DNS, TLS, the three-second timeout, a reset, or a request that could not be built. | Reachability of the configured address from this node, and its TLS. |
| `http_3xx` | The answer was a redirect, which is never followed. | Which service answers at the configured address. |
| `http_4xx` | The answer had a 4xx status. | The token's policy, the namespace, the mount and the key name. |
| `http_5xx` | The answer had a 5xx status. | The health of the service at the configured address. |
| `http_other` | Any other non-success status. | Which service answers at the configured address. |
| `response_size` | The declared or actual body was over 64 KiB. | That the address and mount reach the Transit sign endpoint. |
| `response_shape` | The body was not JSON, had no `data.signature` string, its signature had no `vault:v<digits>:` prefix, or the signature was not Base64. | That the address and mount reach the Transit sign endpoint. |
| `response_version` | The signature was for a `vault:v<N>:` version other than the configured `key_version`. | The Vault key's versions against `key_version`. Re-pinning needs a reviewed bind. |
| `signature_verification` | The returned signature did not verify against the pinned public key, or the verified claims differed. The token was refused. | That `key_name`, `mount` and `key_version` address the pinned key. Do not re-pin to make it pass without review; see [credential compromise](credential-compromise.md). |
| `encoding` | Serializing the token header or claims failed. Not expected. | Report it. |
| `edition_unsupported` | An Essentials build found a stored remote key. | Run the Platform build, or bind a local key. |

**Limits of these counts**

- **Process-local.** The counts live in one process and reset when it starts.
- **Prometheus series.** A reason's series appears only after its first failure
  in that process, so a rate over a window that includes that moment can miss
  the first count. The runtime JSON always lists every reason.
- **Zero is not health.** A zero count means no remote signing attempt failed
  in this process since it started. It does not show that the signer works;
  test issuance for that.
- **Worker processes.** A worker serves no metrics route, so for signing done
  by a worker, such as SSF and logout delivery, the warning is the only signal.
- **SAML.** SAML XML signing is not counted.
- **No live checks.** There is no live Vault probe and no comparison between
  nodes.
