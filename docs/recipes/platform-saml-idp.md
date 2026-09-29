# Platform SAML IdP recipe

This page is the service-provider fixture inside `saml_independent_xmlsec_sign_verify_and_decrypt` in [tests/identity/saml.rs](../../tests/identity/saml.rs). That test is `#[ignore = "requires independent xmlsec1 binary in RIAUTH_TEST_XMLSEC"]`. It reads `RIAUTH_TEST_XMLSEC` and calls `exercise(Some(path))`. The client id is `saml-app`. Its display name, `SAML fixture`, is that fixture's label. The service provider is XML and keys the test builds for `https://sp.example.test`. xmlsec1 signs, verifies, and decrypts those messages. The test was not executed while this page was written.

`saml.idp_signed_browser_sso`, `saml.sp_initiated_logout`, `saml.logout_fanout`, `saml.upstream_logout`, `saml.assertion_encryption`, and `identity.saml_sources` are `PLATFORM_FEATURES` ([src/agent.rs](../../src/agent.rs)). `compiled_for` includes a `PLATFORM_FEATURES` name when the target is Platform ([src/capability.rs](../../src/capability.rs)). `validate_client_settings_for` rejects `settings.saml` on Essentials with `Client setting saml requires the Platform build` ([src/edition.rs](../../src/edition.rs)). The SAML HTTP handlers are behind `feature = "platform"` ([src/api.rs](../../src/api.rs)). Cargo's default features select `platform` ([Cargo.toml](../../Cargo.toml)). The integration job's `cargo test` line leaves those defaults on. This page does not claim an Essentials run.

The wider IdP and source profile stays in [saml.md](../saml.md). The operator example there uses `https://sp.example.com`. This fixture uses `https://sp.example.test`. `saml_source_independent_xmlsec` and `saml_logout_independent_xmlsec` are separate ignored tests. `trailing_slash_issuers_advertise_reachable_saml_endpoints` is a different test in the same file.

## Three different claims

| Claim | What it means here |
| --- | --- |
| Integration-job script | [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) step "Independent SAML XML signature and encryption" runs `RIAUTH_TEST_XMLSEC="$(command -v xmlsec1)" cargo test --test identity --locked saml_independent_xmlsec -- --ignored` on `ubuntu-24.04` with Rust 1.98.1. The same job's "Install integration dependencies" step installs distro `xmlsec1` with no version pin. The filter selects `saml_independent_xmlsec_sign_verify_and_decrypt`. "Independent upstream SAML signatures" and "Independent SAML logout" are the other two xmlsec1 steps. The workflow text is not a result for this revision. |
| This revision | No Cargo command was run while this page was written. The ignored test was not started. The non-ignored sibling was not started either. |
| Deployment peer | xmlsec1 is the signature and decryption program named by `RIAUTH_TEST_XMLSEC`. The fixture service provider is the test's own XML, certificate, and ACS. No named service provider is connected. |

[saml.md](../saml.md) shows the operator form `RIAUTH_TEST_XMLSEC=/path/to/xmlsec1 cargo test --locked --test identity saml_independent_xmlsec -- --ignored --nocapture`. That form adds `--nocapture` and leaves the binary path to the operator. It was not run for this page.

`saml_signed_requests_terminal_delivery_claims_encryption_logout` calls `exercise(None)`. The check job's `cargo test --all-targets --features test-support,fuzzing --locked` does not pass `--ignored`, so that sibling is inside the check script and the xmlsec1 test is outside it. This page did not run the check script.

## Records the fixture creates

`exercise` uses `Fixture::new()` from [tests/common/mod.rs](../../tests/common/mod.rs). That copies an initialized `riauth.redb` into a temporary directory and opens it with `Config::default()` except for `data_dir`. The default issuer is `http://localhost:9000` ([src/config.rs](../../src/config.rs)). No browser, nginx, PostgreSQL, or HTTP listener is started.

| Record | Fixture value |
| --- | --- |
| Administrator | `NewUser` username `admin`, display name `Administrator`, email absent, `admin` true. The password is `tests/common/mod.rs` `PASSWORD`, the literal `test-password-for-fixtures-only`. It exists only for that temporary store. |
| Users | `saml-user` and `saml-other`, each with email `{username}@example.test`, display name `Test User`, `admin` false. `saml-user`'s display name is then patched to `Name <&" escape`. |
| Group | `saml-group`, with `saml-user` added. |
| IdP key | Active PEM at store path `meta` / `keys`. `Core::initialize` generates it with `SigningKey::generate`, which is RS256 and RSA 3072 ([src/crypto.rs](../../src/crypto.rs)). The test does not call `riauth keys import`. |
| IdP certificate | Minted in the test: X.509 version 3 (`set_version(2)`), serial 1, CN `riAuth fixture`, subject equal to issuer, self-signed SHA-256, not-before now, not-after one day. Written to `idp.pem`. |
| SP key and certificate | `Rsa::generate(2048)`, CN `SP fixture`, same certificate helper. PKCS#8 PEM is stored with `write_private` (on Unix the file mode is `0o600`). |
| Client | id `saml-app`, name `SAML fixture`, `confidential` false, `service` false, empty `redirect_uris`, scopes `openid`, `saml`, `profile`, `groups`, `allowed_groups` `saml-group`, `require_mfa` false. |
| SAML settings | `sp_entity_id` `https://sp.example.test/metadata`. One ACS, `https://sp.example.test/acs`, also stored at index `7`. `idp_entity_id` unset. `name_id_format` persistent. `assertion_ttl` 120. `idp_initiated` false and `default_relay_state` unset until the encryption tail. `encryption_certificate_pem` unset until that tail. SLO redirect and POST are both `https://sp.example.test/logout`. |
| Attributes | `urn:test:name` from claim `name`, friendly name `Display name`, required. `urn:test:groups` from claim `groups`, required. |
| SP metadata file | EntityDescriptor `entityID` `https://sp.example.test/metadata`, `AuthnRequestsSigned="true"`, one signing certificate (base64 DER of the SP cert), HTTP-POST ACS at that URL, index `7`, `isDefault="true"`. The test writes the file. It does not fetch metadata. |

The SSO destination the requests use is `http://localhost:9000/saml/saml-app/sso`.

With `idp_entity_id` unset, `Settings::issuer` returns `http://localhost:9000/saml/saml-app/metadata` ([src/assembly/saml.rs](../../src/assembly/saml.rs)). `saml_metadata` puts that value in `entityID`, sets `ID` to `_metadata_saml-app`, sets `WantAuthnRequestsSigned="true"`, advertises the persistent NameID format, and publishes Redirect and POST single-sign-on and single-logout services at `http://localhost:9000/saml/saml-app/sso`. The test asserts the `WantAuthnRequestsSigned` attribute. It does not assert the entityID or the service URLs.

## Profile and xmlsec1 flags

Redirect requests are built in the test and signed with the openssl `Signer` over SHA-256. The query pairs are `SAMLRequest` (raw deflate, then standard base64), `RelayState` `state & exact +`, and `SigAlg` `http://www.w3.org/2001/04/xmldsig-more#rsa-sha256`, then `Signature`. xmlsec1 does not sign those redirect requests.

The AuthnRequest the test builds uses Version `2.0`, Destination `http://localhost:9000/saml/saml-app/sso`, `AssertionConsumerServiceURL` `https://sp.example.test/acs`, `ProtocolBinding` `urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST`, Issuer `https://sp.example.test/metadata`, and `NameIDPolicy` Format `urn:oasis:names:tc:SAML:2.0:nameid-format:persistent` with `AllowCreate="true"`. The first request id is `_one` and it sets `ForceAuthn="true"`.

`xmlsec` runs `{RIAUTH_TEST_XMLSEC} --help-all` and keeps `--lax-key-search` only when that help text contains the flag. The source comment says XMLSec 1.3 selects keys strictly and older releases omit the flag. This page did not observe a version. `verify_xml` does not pass `--lax-key-search`. Sign and decrypt do, subject to that help check. Each call must exit 0. A failed process includes stderr in the assertion message. The helper does not parse stdout.

`verify_xml` writes the document to `verify.xml` in the fixture directory and runs:

```sh
"$RIAUTH_TEST_XMLSEC" --verify \
  --pubkey-cert-pem idp.pem \
  --trusted-pem idp.pem \
  --enabled-reference-uris same-doc \
  --id-attr:ID Response \
  --id-attr:ID Assertion \
  --id-attr:ID EntityDescriptor \
  --node-xpath XPATH \
  verify.xml
```

`XPATH` is `/*/*[local-name()='Signature']` when the call passes `assertion=false`, and `//*[local-name()='Assertion']/*[local-name()='Signature']` when it passes `assertion=true`.

The ignored test calls that verifier on IdP metadata (`assertion=false`), on the first successful response (both xpaths), on the encrypted response (`assertion=false`), and on the decrypted assertion (`assertion=true`).

The POST AuthnRequest is first signed by `risaml::crypto::construct_saml_signature` with the SP key, the SP certificate, and the RSA-SHA256 URI. The ignored test then writes that XML to `sp-template.xml` and runs:

```sh
"$RIAUTH_TEST_XMLSEC" --sign --lax-key-search \
  --privkey-pem sp.key \
  --id-attr:ID AuthnRequest \
  --output sp-signed.xml \
  sp-template.xml
```

`sp-signed.xml` replaces the request XML. The form body is `SAMLRequest` (standard base64 of that XML) and `RelayState` `state & exact +`. `exercise(None)` keeps the risaml signature and skips this command.

On the encryption tail, after risaml has verified the outer signature and decrypted an assertion that contains `AuthnStatement`, the ignored test writes the response to `encrypted.xml` and runs:

```sh
"$RIAUTH_TEST_XMLSEC" --decrypt --lax-key-search \
  --privkey-pem sp.key \
  --output decrypted.xml \
  encrypted.xml
```

`saml_issue` encrypts with `http://www.w3.org/2009/xmlenc11#aes256-gcm` and `http://www.w3.org/2001/04/xmlenc#rsa-oaep-mgf1p` ([src/assembly/saml.rs](../../src/assembly/saml.rs)). The test does not assert those URI strings.

## Assertions the ignored test makes

The ignored test runs the whole `exercise` function, so the rows below are what that function requires. xmlsec1's own check, on the calls above, is a zero exit status. Redirect signatures, the logout signature, and the denial results are Rust assertions. xmlsec1 is not invoked on those steps.

| Step | What `exercise` requires |
| --- | --- |
| Metadata | The string `WantAuthnRequestsSigned="true"`. With xmlsec1, `verify_xml` on that document exits 0. |
| Import | In-process `import_sp_metadata` maps ACS index `7` to `https://sp.example.test/acs` and returns one SP certificate. Entity id `https://wrong.example/sp` returns an error. |
| Import CLI | `CARGO_BIN_EXE_riauth` exits 0. The arguments are `--json --non-interactive saml import-sp --file sp-metadata.xml --entity-id https://sp.example.test/metadata --idp-certificate idp.pem --out sp-import.json`, with those paths under the temp directory. The JSON file equals the in-process value. Stdout is not compared. |
| IdP-initiated while disabled | `saml_initiate("saml-app", None)` returns an error. `saml_initiate` returns `Error::forbidden()` when `idp_initiated` is false. The test does not match that error value. |
| ForceAuthn handoff | `saml_start` of the signed redirect, `is_post` false, no cookie, is `Reply::Waiting`. The test reads `user_code`, the resume id from the last segment of `refresh`, and the `riauth_saml` cookie value. |
| Request details | `browser_details` has `protocol` `saml` and `reauthentication_required` true. |
| Fresh login | `login_for("saml-user", PASSWORD, None, Some(transaction))` yields `session_token`. |
| First response | `saml_resume` with the binding cookie is `Reply::Post`. `target` is `https://sp.example.test/acs`. `RelayState` is `state & exact +`. `SAMLResponse` is standard base64 and parses as XML. A second resume of the same id returns an error. |
| Response XML | `InResponseTo` `_one`. `Destination` is the ACS. Audience text is the SP entity id. `SubjectConfirmationData` `Recipient` is the ACS and `InResponseTo` is `_one`. `AuthnContextClassRef` text is `urn:oasis:names:tc:SAML:2.0:ac:classes:Password`. `AuthnStatement` has `SessionIndex` and `AuthnInstant`. An `AttributeValue` in the assertion namespace has text `Name <&" escape`. Two elements named `Signature` in `http://www.w3.org/2000/09/xmldsig#` are present. `risaml::crypto::verify_signature` with the IdP certificate returns true. With xmlsec1, both verify xpaths exit 0. |
| SSO cookie | A `riauth_sso` cookie is present on that POST. Later signed starts pass its value. |
| Rejected requests | Tampered RelayState, ACS `https://attacker.example/acs`, `IssueInstant` of now minus 400 seconds, and an `Assertion` nested in `Extensions` each make `saml_start` return an error. The store snapshot is unchanged. The assertion message is `rejected signed SAML requests must not change identity or pending state`. `me` on the original `saml-user` session still succeeds. |
| Passive | `IsPassive="true"`, no SSO cookie. The reply is a POST to the ACS with the same RelayState. The XML contains `:No passive` and does not contain `<saml:Assertion`. |
| POST | After the signature step above, `saml_start` with `is_post` true and the SSO cookie returns a POST whose XML contains `InResponseTo="_post"`. xmlsec1 is not asked to verify this response. |
| Indexed ACS | The URL attribute is replaced with `AssertionConsumerServiceIndex="7"`. Redirect binding, SSO cookie. `body` requires a POST to the ACS and the same RelayState. `InResponseTo` is not read. |
| Logout | NameID `wrong-user` makes `saml_start` return an error. `consents` on the fresh session succeeds. The matching LogoutRequest uses the assertion's NameID, its `NameQualifier`, `SPNameQualifier` equal to the SP entity id, the captured `SessionIndex`, and the persistent NameID URI. The reply is `Reply::Redirect` and the URL starts with `https://sp.example.test/logout?`. |
| Logout signature | The test splits the query on `&Signature=`, standard-base64-decodes `SAMLResponse`, raw-inflates it, and parses a `LogoutResponse` whose `InResponseTo` is `_logout`. An openssl `Verifier` over SHA-256, using the IdP private key, accepts the query prefix. `consents` on that session then returns an error. The test does not read a Success or PartialLogout status. xmlsec1 is not called. |
| Encrypted IdP-initiated response | After the settings update below, `saml_initiate` waits, approval uses `remember` false, and resume is a POST to the ACS. The XML contains `EncryptedAssertion`, does not contain `Name &lt;`, and does not contain `InResponseTo=`. risaml verification succeeds. risaml decryption contains `AuthnStatement`. With xmlsec1, outer verify, decrypt, and assertion verify exit 0, and a `NameID` in `urn:oasis:names:tc:SAML:2.0:assertion` has the same text as the first response. |

`response_xml` writes Success as `urn:oasis:names:tc:SAML:2.0:status:Success`. Any other status is a Responder code wrapping `urn:oasis:names:tc:SAML:2.0:status:{status}` ([src/saml/wire.rs](../../src/saml/wire.rs)). The passive substring `:No passive` matches that wrapped value. The test does not assert the Responder code or the Success value.

The receiver rejects an `IssueInstant` when `instant > now + 30` or `instant + 300 < now`, with `SAML request is expired or future dated`. The replay row stores `now + 630`. A later start returns `SAML request replay` while that stored expiry is still ahead of now. The digest key is `{client id}`, a NUL byte, and the request id ([src/assembly/saml.rs](../../src/assembly/saml.rs)). The test asserts `is_err` for the expired request and for the second start of `_one`. It does not match those messages.

The issuer writes `NameFormat="urn:oasis:names:tc:SAML:2.0:attrname-format:unspecified"` on each attribute. The test does not read `NameFormat`, the groups attribute, `assertion_ttl`, or a status code. Password authentication on this `http://` issuer selects the Password context ([src/assembly/saml.rs](../../src/assembly/saml.rs) `authn_context`). The test does assert that context URI.

When resume delivers an approval whose `remember` flag is true, `remember_approved_consent` stores a SAML consent with `expires_at` of now plus 2,592,000 seconds ([src/management/consents.rs](../../src/management/consents.rs)). The ForceAuthn path passes `remember: true` and then resumes successfully. The test does not read the consent row. `consents(&fresh)` loads the bearer session and lists consents. The test requires that call to succeed before the LogoutRequest and to fail after the signed redirect. It does not inspect the list.

## Permission and denial cases

Directory and client writes use the administrator session from `Fixture::new`. User, group, and client methods require `Idempotency-Key` and `If-Match` only when an HTTP context is present ([src/core.rs](../../src/core.rs)). This fixture calls `Core` with no HTTP context, so those checks are skipped. The test does not send the headers.

| Boundary | What the test does |
| --- | --- |
| Create users | Administrator session. `create_user` requires `user.write` on `user/{username}`. `update_user` of the display name requires `user.write` on `user/saml-user` for a non-delegated actor ([src/management.rs](../../src/management.rs)). |
| Group | `create_group` requires `group.write` on `group/saml-group`. `group_member` requires `group.members` on that group. |
| Create and update the client | `client.write` on `client/saml-app`. The encryption update is `update_client` by the administrator. |
| CLI client write | `riauth client create` and `riauth client update` require `--idempotency-key` and `--if-revision`. The transport sends `idempotency-key` and `if-match` set to the quoted revision ([src/cli/transport.rs](../../src/cli/transport.rs)). The test does not run those commands. Omitting `--scope` on create selects `openid`, `profile`, `email`, and `offline_access`, which has no `saml` scope. |
| Import CLI | Local command, before any server call. It reads at most `48 * 1024 + 1` XML bytes and `16 * 1024 + 1` certificate bytes ([src/cli.rs](../../src/cli.rs)). The test's files fit. The test does not assert the caps. |
| Approval | `Core::browser_decide` with `approve` true. The Waiting `instruction` names `riauth request approve CODE`. The test does not spawn that command. |
| ForceAuthn binding | Approval with no `transaction_id` returns an error. `saml-other`'s session, presenting `saml-user`'s transaction, returns an error. A second ForceAuthn transaction for `saml-user` cannot approve the first code. Approval then uses the matching transaction and `remember` true. |
| Browser binding | `saml_resume` with `Some("wrong browser")` returns an error. The matching `riauth_saml` value succeeds once. |
| Signature and parser denials | The four `saml_start` failures in the assertions table. RelayState tampering replaces the encoded query text `state+%26+exact+%2B` with `changed`. |
| Logout account | NameID `wrong-user` returns an error. The assertion's NameID, qualifiers, and `SessionIndex` produce the signed redirect. |
| Consent withdrawal | IdP-initiated approval with `remember` true, then `revoke_consent` on that user session for `saml-app`, then `saml_resume` returns an error. |
| Group removal | A further IdP-initiated approval with `remember` false, then `group_member` removes `saml-user` from `saml-group`, then `saml_resume` returns an error. |
| Signing key | `validate_key` requires a local RS256 domain and a certificate that matches it: `SAML XML signing currently requires a local RS256 signing domain` ([src/saml.rs](../../src/saml.rs)). The fixture uses the generated key and a certificate it mints from that key. |
| Essentials | `settings.saml` is rejected on an Essentials target. This fixture is not built that way. |

`Settings::validate` also requires an interactive client, the `saml` scope, 1–10 ACS URLs, 1–4 SP certificates, at most 32 attributes, and `assertion_ttl` in 30..=300. ACS and SLO URLs use `endpoint_url`: canonical HTTPS, or HTTP loopback, without query or fragment. The fixture's ACS and SLO URLs are `https://sp.example.test/...`. The test reaches `create_client` success and does not assert the failure strings.

## Closest API and CLI

The test starts the `saml import-sp` process shown above and calls `Core` in process. That process was not run for this page. The other commands are the supported counterparts.

```sh
riauth --json --non-interactive saml import-sp \
  --file sp-metadata.xml \
  --entity-id https://sp.example.test/metadata \
  --idp-certificate idp.pem \
  --out sp-import.json
```

```sh
riauth revision
riauth client create saml-app \
  --name 'SAML fixture' \
  --scope openid,saml,profile,groups \
  --group saml-group \
  --settings-file deployment-private/saml-app-settings.json \
  --idempotency-key "$KEY" \
  --if-revision "$REVISION"
```

`deployment-private/saml-app-settings.json` is a `ProviderSettings` object. The fixture's initial `saml` object has this shape. Replace the PEM placeholders with the certificates the test mints. The test never wrote this file.

```json
{
  "saml": {
    "sp_entity_id": "https://sp.example.test/metadata",
    "acs_urls": ["https://sp.example.test/acs"],
    "acs_indices": {"7": "https://sp.example.test/acs"},
    "idp_entity_id": null,
    "idp_certificate_pem": "IDP CERTIFICATE PEM",
    "sp_certificates_pem": ["SP CERTIFICATE PEM"],
    "encryption_certificate_pem": null,
    "name_id_format": "persistent",
    "attributes": [
      {
        "name": "urn:test:name",
        "claim": "name",
        "friendly_name": "Display name",
        "required": true
      },
      {
        "name": "urn:test:groups",
        "claim": "groups",
        "friendly_name": null,
        "required": true
      }
    ],
    "slo_redirect_url": "https://sp.example.test/logout",
    "slo_post_url": "https://sp.example.test/logout",
    "idp_initiated": false,
    "default_relay_state": null,
    "assertion_ttl": 120
  }
}
```

The encryption tail updates the same client so `encryption_certificate_pem` is the SP certificate, `idp_initiated` is true, and `default_relay_state` is `state & exact +`.

| Fixture step | Supported equivalent |
| --- | --- |
| `Core::saml_metadata` | Public `GET /saml/saml-app/metadata` returns `application/samlmetadata+xml` and `cache-control: no-store`, with no authentication. `GET /api/saml/saml-app/metadata` requires `client.read` on `client/saml-app` and returns `{"metadata_xml": ...}`. `riauth saml metadata saml-app --out FILE` calls that authenticated route. The test calls `Core` and does not call either route. |
| `saml import-sp` | The local command above. `riauth keys import` is the operator way to install an IdP key. The test loads the generated key instead. |
| `Core::create_client` | The `client create` command above, or `POST /api/clients`. Both require `client.write` on `client/saml-app`. |
| `Core::saml_start` redirect | `GET /saml/saml-app/sso` with the signed query. The handler calls `saml_start` with `is_post` false. |
| `Core::saml_start` POST | `POST /saml/saml-app/sso`. The handler requires exactly one `content-type` whose media type is `application/x-www-form-urlencoded`, or it returns `SAML POST requires form encoding`. The test passes the form body and `is_post` true into `Core` and does not send that header. |
| `Core::saml_initiate` | `GET /saml/saml-app/init`. |
| `Core::browser_decide` | `riauth request approve CODE` GETs `/api/authorization/{code}`. When `reauthentication_required` is true it calls `reauthenticate` with that response's `transaction_id`, then POSTs `/api/authorization/decision` with `code`, `approve`, `remember`, and `transaction_id` ([src/cli.rs](../../src/cli.rs)). The test calls `login_for` and `browser_decide` in process. |
| `Core::saml_resume` | The resume route for that id, with the `riauth_saml` cookie. The test calls `Core`. |
| `Core::consents` and `Core::revoke_consent` | `riauth consents` is `GET /api/consents`. `riauth consents --revoke saml-app` is `DELETE /api/consents/saml-app`. The test calls the Core methods on the user session. |

## What this fixture does not prove

xmlsec1 signs and checks the documents this test hands it. A service provider is a separate peer. No named service provider was connected. The names in the migration inventory, including Keycloak and Okta, are not this fixture. `https://sp.example.com` in [saml.md](../saml.md) is the operator example. The fixture entity is `https://sp.example.test`.

The ignored test was not run for this revision. The integration-job command is workflow text until that job runs on a commit. Distro `xmlsec1` in the workflow is unpinned. The `--help-all` check can drop `--lax-key-search`. This page does not record which flag the runner kept.

`exercise(None)` is the in-process path. Its POST request keeps the risaml signature, and it never starts xmlsec1. The check job's cargo command is written to include that non-ignored test. This page did not execute it.

The upstream-source xmlsec1 step and the logout xmlsec1 step are different filters. Logout fan-out across several participants is covered by [tests/identity/saml_logout.rs](../../tests/identity/saml_logout.rs), not by the single LogoutResponse in `exercise`. Redirect AuthnRequests and that LogoutResponse use openssl. xmlsec1 is limited to the verify, sign, and decrypt commands above.

SOAP, artifact, ECP, and encrypted NameID sit outside the profile in [saml.md](../saml.md). This test does not send them. XML signing in this profile uses the local RS256 key. The fixture does not use Vault. No browser is started. Chrome, Playwright, and the OIDC relying-party recipe are separate. The OIDF runner is not this test.

## Other D03 recipes

Still without a recipe page: upstream OIDC, SAML source, LDAP import, inbound SCIM, outbound SCIM, RADIUS, Workspace, Entra, Shared Signals, device trust, HTTPS client certificates, Vault Transit, Windows device login, the embedded reverse proxy, and shared-domain SSO. The [forward-auth](platform-forward-auth.md), [LDAP provider](platform-ldap-provider.md), and [OIDC relying party](oidc-relying-party.md) recipes are separate. A named service provider is still an open peer. D04 emergency runbooks and D05 acceptance are separate work. The [capability matrix](../capability-matrix.md) records the protocol limits those recipes still have to cite.
