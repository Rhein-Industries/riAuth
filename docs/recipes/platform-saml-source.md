# Platform SAML source recipe

This page is `exercise` in [tests/identity/saml_source.rs](../../tests/identity/saml_source.rs). [tests/identity.rs](../../tests/identity.rs) includes that file as module `saml_source_tests`. `saml_source_signed_encrypted_terminal_linking_and_live_trust` calls `exercise(None)`. `saml_source_independent_xmlsec_responses` is `#[ignore = "requires RIAUTH_TEST_XMLSEC"]` and calls `exercise(Some(path))` with `RIAUTH_TEST_XMLSEC`. Neither caller was executed while this page was written.

`identity.saml_sources` is listed in `PLATFORM_FEATURES` ([src/agent.rs](../../src/agent.rs)). `compiled_for` is false for a Platform-only name when the target is Essentials ([src/capability.rs](../../src/capability.rs)). `validate_source_for` rejects `source.saml` on Essentials with `SAML source requires the Platform build` ([src/edition.rs](../../src/edition.rs)). `validate_client_settings_for` rejects `settings.source_stage` on Essentials with `Client setting source_stage requires the Platform build`. Cargo's default features select `platform` ([Cargo.toml](../../Cargo.toml)). The check job's `cargo test --all-targets` line does not pass `--no-default-features`. A later step in that job builds Essentials and runs one CLI USB test. It does not name this function. This page does not claim an Essentials run.

`saml_browser_acs_handoff_checks_success_foreign_browser_missing_cookie_replay_and_cli` is a separate `#[tokio::test]` in the same file. It has no ignore attribute. It calls `install_saml`, which sets `auto_provision` true and `xmlsec: None`, and it posts through `router(...).oneshot`. `exercise` sets `auto_provision` false. This page does not recount the browser test.

The operator profile stays in [saml.md](../saml.md). The CLI block there uses source id `corporate-saml`, signing key `source-sp`, issuer `urn:company:idp`, and client entity `urn:company:riauth-sp`. This fixture uses `enterprise`, `signing`, `urn:example:enterprise-idp`, and `urn:example:riauth-sp`. Those CLI commands were not run. The [Platform SAML IdP recipe](platform-saml-idp.md) is the other direction: riAuth signs assertions for a fixture service provider. `saml_logout_multiple` and `saml_logout_independent_xmlsec` are separate tests. This fixture sets both SLO URLs to `None`.

## Three different claims

| Claim | What it means here |
| --- | --- |
| Commands written in CI | The check job in [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) runs `cargo test --all-targets --features test-support,fuzzing --locked` on `ubuntu-24.04` with Rust 1.98.1. It does not pass `--ignored`. That command's text includes `saml_source_signed_encrypted_terminal_linking_and_live_trust` and the browser test. It excludes `saml_source_independent_xmlsec_responses`. The integration job's "Install integration dependencies" step installs distro `xmlsec1` with no version pin. Its "Independent upstream SAML signatures" step runs `RIAUTH_TEST_XMLSEC="$(command -v xmlsec1)" cargo test --test identity --locked saml_source_independent_xmlsec -- --ignored`. The filter is a substring of `saml_source_independent_xmlsec_responses`. `--ignored` selects that ignored function. The same job's "Independent SAML XML signature and encryption" step filters `saml_independent_xmlsec`. "Independent SAML logout" filters `saml_logout_independent_xmlsec`. The workflow text is not a result for this revision. |
| This revision | No Cargo command was run while this page was written. `exercise(None)` was not started. The ignored xmlsec1 caller was not started. The browser test was not started. |
| Named external IdP | The identity provider is the in-process `Upstream` helper. Its issuer is `urn:example:enterprise-idp` and its SSO URL is `https://enterprise.example.test/sso`. Those strings are fixture values. xmlsec1 is the local signature program named by `RIAUTH_TEST_XMLSEC`. Okta, Entra, ADFS, and Authentik are not connected. [saml.md](../saml.md) links an Authentik document. The matrix peer row for Authentik is a different offline export fixture. |

[saml.md](../saml.md) describes an additional `saml_source_independent_xmlsec` test and does not paste the integration-job argv. The shell block in that guide filters `saml_independent_xmlsec` for the IdP test and `saml_logout_independent_xmlsec` for logout. Neither form is the upstream-source step.

## Records the fixture creates

`Fixture::new` copies a store from `Core::initialize` ([tests/common/mod.rs](../../tests/common/mod.rs)). `Config` is `Default` apart from `data_dir`. The default issuer is `http://localhost:9000` ([src/config.rs](../../src/config.rs)). No HTTP listener is started. `source_start`, `saml_source_callback`, and `source_finish` are in-process `Core` calls.

| Record | Value `exercise` sets |
| --- | --- |
| Administrator | Username `admin`, display name `Administrator`, email absent, `admin` true. The password is `PASSWORD`, the literal `test-password-for-fixtures-only`. It exists only for that temporary store. |
| Users | `alice` and `unrelated`. Each email is `{username}@example.test`, the display name is `Test User`, and `admin` is false. `user` returns the password-login session token. |
| IdP key and certificate | `Rsa::generate(2048)`, CN `Upstream IdP`, through `saml_tests::cert`: version 3, serial 1, subject equal to issuer, self-signed SHA-256, not-before now, not-after one day. The PKCS#8 PEM is written to `{temp}/idp.key`. |
| SP key and certificate | Active key at store path `meta` / `keys`. `SigningKey::generate` is RS256 and RSA 3072 ([src/crypto.rs](../../src/crypto.rs)). The test mints CN `riAuth source SP` from that key and sets `signing_key` to `signing`. |
| Source id and name | `enterprise`, `Enterprise SAML` |
| Issuer and SSO URL | `urn:example:enterprise-idp`, `https://enterprise.example.test/sso` |
| SP entity id | `urn:example:riauth-sp`, stored as `client_id` |
| OAuth-shaped fields | `oauth_profile` none, `token_endpoint` empty, client authentication `none`, JWKS empty, scopes empty, `client_secret` `None` on `source_put` |
| SAML settings | Both SLO URLs `None`. One pinned IdP certificate. NameID format persistent. `name_attribute` `display`, `email_attribute` `email`, `email_verified_attribute` `verified`. `require_encrypted_assertions` false until the later `source_put`. |
| Linking policy | `enabled` true, `auto_provision` false, `groups` empty, `trusted_mfa_acr` empty until the later `source_put`, `allow_admin_login` false |

`saml_source_callback_url("enterprise")` is `{endpoint_base(issuer)}/saml/sources/enterprise/acs` ([src/source/saml.rs](../../src/source/saml.rs), [src/saml.rs](../../src/saml.rs)). With the default issuer that string is `http://localhost:9000/saml/sources/enterprise/acs`.

## Signatures and encryption

`Upstream::sign` has two paths. With `xmlsec` set, it inserts a signature template after the issuer and runs `saml_tests::xmlsec` with `--sign`, `--lax-key-search`, `--privkey-pem {temp}/idp.key`, `--id-attr:ID` on `Assertion`, `Response`, `LogoutRequest`, and `LogoutResponse`, then `--output`. `xmlsec()` in [tests/identity/saml.rs](../../tests/identity/saml.rs) runs `{binary} --help-all` and drops `--lax-key-search` when that help text lacks the flag. The source comment says XMLSec 1.3 selects keys strictly and older releases omit the flag. This page did not observe a version. The process must exit 0. With `xmlsec` unset, `sign` calls `risaml::crypto::construct_saml_signature` with algorithm `http://www.w3.org/2001/04/xmldsig-more#rsa-sha256`.

`response` always signs the assertion with `whole` false, optionally strips `xmlns:saml` when the change is `$inherit-ns`, wraps a `Response`, optionally encrypts, then signs the response with `whole` true. Encryption calls `risaml::crypto::encrypt_assertion` in both modes. The data algorithm is `http://www.w3.org/2009/xmlenc11#aes256-gcm` and the key algorithm is `http://www.w3.org/2001/04/xmlenc#rsa-oaep-mgf1p`, using the source SP certificate. xmlsec1 does not encrypt the assertion in this fixture.

`verified_identity` requires the response signature to be the first signature in document order, verifies it against the pinned IdP certificates, and only then decrypts. The assertion signature is verified on its own. The `$inherit-ns` case is the fixture's view of that second check: the assertion is signed with the namespace, the namespace declaration is removed, and the callback still returns `completed` true.

## Assertions `exercise` makes

These checks run only when a caller runs. This revision did not run them.

`saml_source_metadata` must contain the ACS string and `AuthnRequestsSigned="true"`. When the xmlsec path is set, the test writes the metadata and the SP certificate and runs `xmlsec1 --verify --trusted-pem {cert} --id-attr:ID EntityDescriptor {file}`. That verify call does not pass `--lax-key-search`. The non-ignored caller skips it. The test does not assert `entityID`, `WantAssertionsSigned`, the NameID format element, the ACS index, or the certificate text. The metadata builder sets those: entity id `source.client_id`, `WantAssertionsSigned="true"`, the configured NameID format, ACS binding POST at the callback URL, index `0`, `isDefault="true"`, and signing and encryption key descriptors from the SP certificate. Both SLO URLs are `None`, so the builder emits no `SingleLogoutService`.

`begin` calls `Core::source_start` and then `decode_redirect`. That helper verifies the redirect query with openssl SHA-256 against the SP certificate. The signed bytes are the query before `&Signature=`, and `Signature` is the last pair. It inflates `SAMLRequest` and asserts `ForceAuthn="true"` and `AssertionConsumerServiceURL` equal to the ACS. It returns `RelayState` and the request `ID`. It does not assert issuer, NameID policy, destination, protocol binding, or `SigAlg`. `Settings::authorization` builds those: destination is the source SSO URL, protocol binding is HTTP-POST, `ForceAuthn` is true, issuer is `source.client_id`, and `NameIDPolicy` format is the configured NameID URI with `AllowCreate="true"`. `redirect_message` deflates with fast compression, base64-encodes the XML, and orders the query as `SAMLRequest`, `RelayState`, `SigAlg` `http://www.w3.org/2001/04/xmldsig-more#rsa-sha256`, then `Signature` ([src/saml/wire.rs](../../src/saml/wire.rs)).

The assertion helper sets NameID format to the persistent URI, value `opaque-subject`, `NameQualifier` to the source issuer, and `SPNameQualifier` to the SP entity id. Subject confirmation is bearer. `Recipient` is the ACS, `InResponseTo` is the request id, and `NotOnOrAfter` is now plus 120 seconds. Conditions use `NotBefore` now minus 1 second, `NotOnOrAfter` now plus 120 seconds, and one audience equal to the SP entity id. The authentication statement uses `AuthnInstant` now, `SessionIndex` `upstream-session`, `SessionNotOnOrAfter` now plus 600 seconds, and `AuthnContextClassRef` `urn:example:password` unless a change replaces it. Attributes are `display` `Upstream &amp; Partners`, `email` `unrelated@example.test`, and `verified` `true`. The response issuer is the source issuer, destination is the ACS, `InResponseTo` is the request id, and status is Success.

| Step | Asserted result |
| --- | --- |
| First login, review | `submit` returns `completed` true. `source_finish` with `approve` false returns `local_user` null and `email` `unrelated@example.test`. User `unrelated` has that same email. The review does not select that account. |
| First login, approve | `source_finish` with `approve` true is an error. `auto_provision` is false. The test checks `is_err` and does not match an error string. |
| Link Alice | `source_start` is called with Alice's password session. Review `local_user.username` is `alice` and `mfa` is false. Approve returns `session_token`. `me` on that token has username `alice`. A second approve of the same credential is an error. |

Alice's email is `alice@example.test`. The assertion email remains `unrelated@example.test`. The link comes from the session passed to `source_start`.

The workflow block runs next, then the replay of that finished login, then the rejection loop.

## Workflow consumption

The comment in `exercise` says this block consumes the same signed verifier output without minting a session. It is inside `exercise`, so both callers run it. It is not a separate workflow product, and it does not open a workflow editor.

The test records the session count, calls `workflow_source_start` with Alice's password session and source id `enterprise`, inflates `SAMLRequest`, and asserts `ForceAuthn="true"`. It does not call `decode_redirect`, so this path does not verify the redirect signature again. The response uses `$session-expiry` of now plus 60 seconds and `submit` returns `completed` true.

The stored `source_logins` row, keyed by the digest of `RelayState`, is then replaced three times: `/result/expires_at` now, `/result/saml_session/expires_at` now, and `/result/saml_session` null. Each `workflow_source_finish` is an error, `workflow_evidence` is empty, and the login row is still present. Restoring the row finishes with `RunState::Finished` and outcome `Authenticated`. One evidence receipt has `consumed` true, `expires_at` less than or equal to that now-plus-60 value, and `source.transaction` equal to the login key. The login row is then absent. A second finish is an error. The session count equals the count taken before `workflow_source_start`.

`saml_source_callback` then submits the first response and its original `RelayState` again. That login was already finished, so the pending row is claimed. The lookup requires an unclaimed row and returns `SAML source request expired or already used` before a new `saml_source_replays` write. The test checks `is_err`. It does not match that string, and it does not show a replay-table hit.

Each rejection below uses a new `begin`. `submit` unwraps `saml_source_callback`, so `completed` false is a returned body (`completed` is the inverse of the pending `failed` flag). The five string changes also require `source_finish` approve to be an error. The assertion label is `accepted {from}` when `completed` is not false. The test does not compare a server error string.

| Change, applied before signing | `completed` |
| --- | --- |
| NameID value `opaque-subject` replaced with an empty string | false |
| Attribute name `Recipient` replaced with `WrongRecipient` | false |
| Audience `urn:example:riauth-sp` replaced with `urn:attacker` | false |
| Attribute name `InResponseTo` replaced with `OtherRequest` | false |
| Assertion issuer `urn:example:enterprise-idp` replaced with `urn:attacker` | false |

`$inherit-ns` removes `xmlns:saml` after the assertion is signed. `completed` is true, and approve's `user.username` is `alice`. `$stale-authn` sets `AuthnInstant` to now minus 600 seconds before signing. `completed` is false. Replacing `Upstream &amp; Partners` with `attacker` after both signatures returns `completed` false. The stale-authentication call and the tamper call do not invoke `finish`.

The relay check builds one response, then calls `begin` again. Submitting that response with the second `RelayState` returns `completed` false. Submitting it with its own `RelayState` then returns `completed` true. The callback marks a pending login claimed before it parses XML. The first of those two pending rows is still unclaimed when the second state is submitted.

The test then sets `require_encrypted_assertions` true, inserts trusted ACR `urn:example:mfa`, and calls `source_put`. `me` on the earlier SAML session token is an error. The test does not read the session row. `write_source` marks sessions revoked when their `identity.source.id` equals a source whose stored record changed ([src/management.rs](../../src/management.rs)). A plaintext response then returns `completed` false. An encrypted response whose `AuthnContextClassRef` was replaced with `urn:example:mfa` returns `completed` true. Review `mfa` is true. Approve's `me` username is `alice`. `trusted_mfa_acr.contains` is how the source sets that flag. The linked review, while the trusted set was still empty and the class reference was `urn:example:password`, had `mfa` false.

## Embedded stage and unlink

`exercise` creates client `stage-app` through `Fixture::client`, then `update_client` sets `settings.source_stage` to `enterprise`. `Fixture::request` builds an authorization for that client. The test sets `prompt` to `login` and `decision` to `None`, then calls `authorization_prepare` with Alice's password session. It reads `prepared["source_stage"].authorization_url`, inflates `SAMLRequest`, and does not call `decode_redirect`. The response is encrypted and uses `urn:example:mfa`. `submit` returns `completed` true. `callback["source_stage"].stage_id` equals the prepared stage id. The callback body has no `session_token` and no `code`.

`source_stage_resume` returns `code_issued` true. The code digest in the resume redirect matches a `codes` row. That row's `identity.user_id` equals Alice's id from `me` of her password session, `identity.mfa` is true, `challenge` equals the digest of the verifier, and query `state` equals `state with & delimiters`. A second resume is an error. This tail is in-process `Core` calls. [saml.md](../saml.md) still says an end-to-end SAML-stage browser run remains acceptance work.

`source_links` of Alice's password session is followed by `source_unlink` of the first link id, using that same password session. `me` on the encrypted-MFA SAML session token is then an error. `unlink_source` deletes the link and revokes sessions whose `identity.source.link` equals that link id. The test does not match an error string and does not read the session row.

## Source limits this fixture leaves unasserted

`Settings::validate` rejects an OAuth profile, a non-empty token endpoint, client authentication other than `none`, a non-empty JWKS, or non-empty scopes, with `SAML sources use issuer/client entity IDs, an SSO authorization endpoint, none client authentication, empty scopes/token endpoint/JWKS and no OAuth profile`. It rejects a transient NameID, an empty IdP certificate list, more than four certificates, more than 64 groups, or more than 16 trusted MFA ACRs, with `SAML sources require stable NameIDs, 1..4 pinned certificates and bounded groups/ACRs`. A verified-email attribute requires an email attribute. `Settings::key` rejects a remote key and any algorithm other than RS256, and it rejects an SP certificate that does not match the signing domain. The test stores the SP certificate minted from the active local RS256 key and does not try these rejections.

NameID URIs in [src/saml.rs](../../src/saml.rs) are persistent `urn:oasis:names:tc:SAML:2.0:nameid-format:persistent`, transient `urn:oasis:names:tc:SAML:2.0:nameid-format:transient`, email `urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress`, and unspecified `urn:oasis:names:tc:SAML:1.1:nameid-format:unspecified`. This fixture stores persistent. Email and unspecified are the other stable formats `validate` allows. The test does not store them. [saml.md](../saml.md) says transient NameIDs, unsolicited IdP-initiated source login, artifact, SOAP, and ECP are not advertised. This exercise does not send those flows.

The callback accepts only `SAMLResponse` and `RelayState`, each value at most 65,536 bytes, with no duplicate names. `RelayState` must be 43 characters. The pending row must match the source id, be unclaimed, be unexpired, and carry the current source fingerprint. XML limits are 48 KiB, depth 24, 2,048 nodes, 32 attributes per element, 8,192 bytes per attribute value, and 48 KiB of text. The source requires response version 2.0, the ACS as destination, `InResponseTo` equal to `_{pending.nonce}`, the configured issuer, success status with no nested status code, at most two signatures, one reference URI `#{owner ID}`, transforms enveloped-signature then exclusive canonicalization, and RSA-SHA256. NameID format must equal the configured URI. Bearer confirmation, exact recipient, and exact `InResponseTo` are required. `NotOnOrAfter` must be later than now and at most now plus 600 seconds. For a login that is not a workflow proof, `AuthnInstant` must be at least `pending.started_at` minus 5 seconds and at most now plus 30 seconds. Workflow proofs require `AuthnInstant` at or after `pending.started_at`. The stale-authentication case shows `completed` false and does not assert the 5-second bound.

`source_put` refuses a change of issuer, client id, OAuth profile, or NameID format while links exist. The later `source_put` in this test changes encryption and the trusted ACR only. Certificate rollover across the four-certificate cap is source behavior this fixture does not walk. Optional SLO is source behavior this fixture leaves unset.

## CLI counterpart

[saml.md](../saml.md) shows `riauth keys import source-sp`, `riauth source put`, `riauth source metadata`, `riauth source start`, and `riauth source finish` with and without `--yes`. The fixture calls `Core::source_put` with signing key `signing` and does not run those commands.

## What this fixture leaves open

The identity provider is the in-process `Upstream` helper. No named external IdP is connected. xmlsec1 signs and verifies metadata only in the ignored caller, and assertion encryption stays in risaml on both callers. The browser ACS handoff remains the other test in this file. Logout, email and unspecified NameIDs, certificate rollover, and a browser run of the embedded stage are outside the assertions above.

## Other D03 recipes

Still without a recipe page: RADIUS, Workspace, Entra, Shared Signals, device trust, HTTPS client certificates, Vault Transit, Windows device login, the embedded reverse proxy, and shared-domain SSO. The [forward-auth](platform-forward-auth.md), [LDAP provider](platform-ldap-provider.md), [OIDC relying party](oidc-relying-party.md), [SAML IdP](platform-saml-idp.md), [LDAP import](ldap-import.md), [upstream OIDC](upstream-oidc.md), the [inbound SCIM](platform-inbound-scim.md), and [outbound SCIM](platform-outbound-scim.md) recipes are separate. The inbound SCIM client is in-process `oneshot`, and no named SCIM client is connected. The relying-party client is the in-tree axum fixture, and a named external relying party remains an open peer. The SAML IdP signer is xmlsec1, and a named service provider remains an open peer. The import page follows disposable loopback OpenLDAP, and a real Active Directory directory remains an open peer. The upstream issuer is the in-process loopback token endpoint, and Okta, Entra, and Google are not connected. The outbound SCIM peer is a second riAuth router on loopback HTTP, and no named SaaS directory is connected. D04 emergency runbooks and D05 acceptance are separate work. The [capability matrix](../capability-matrix.md) records the protocol limits those recipes still have to cite.
