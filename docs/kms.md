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
riauth keys bind signing --signer production-v7 --algorithm RS256
# Or bind a provider-specific signing domain:
riauth keys bind payroll --signer production-v7 --algorithm RS256
```

The existing `key.write` permission controls binding. Agents can select only signer names already configured by the server operator, not supply network endpoints or arbitrary credential paths. A new binding signs and verifies a challenge after authorization, revision and idempotency checks. An authenticated retry with a matching saved receipt returns that result without contacting Vault, even during an outage. New key versions should be configured under distinct signer names and kids; bind the new version after configuration is present on every node. Rotation retains old public verification keys through the token/session retention window.

Supported algorithms are RS256 with PKCS#1 v1.5/SHA-256, ES256/P-256 with the JWS signature representation, and Ed25519/EdDSA. These parameters follow the [Vault Transit signing API](https://developer.hashicorp.com/vault/api-docs/secret/transit). HTTP loopback is accepted for local fixtures; other addresses require HTTPS, certificate verification and no redirects. Each signing request has a three-second timeout and a 64 KiB response limit. Key-service response bodies and credentials are not logged.

This is a Vault Transit integration, not an assertion that the configured Vault uses hardware custody or that an HSM has been tested. Native PKCS#11 and AWS/GCP/Azure-specific KMS adapters are not implemented. Tests use an independent OpenSSL signer behind a local Transit API fixture and cover all three algorithms, version mismatches, invalid signatures, unavailable service and retry after transaction rollback.

Encrypted database backups preserve the remote signer name, version and public pin,
plus configuration file references. They cannot restore the Vault key or bearer
credential. Re-provision the exact signer and test issuance during recovery; a
successful local restore/JWKS check alone does not prove that Vault can sign.
`riauth_signing_errors_total` and the optional signing-failure alert signal report
process-local signing failures; see [operations](operations.md).
