# SAML

riAuth supports a SAML 2.0 IdP profile with signed HTTP-Redirect/HTTP-POST AuthnRequests, HTTP-POST responses, signed metadata, signed assertions and responses, optional assertion encryption, browser and terminal sign-in, browser SSO and SP/IdP-initiated logout propagation. There is no administration GUI; the embedded portal launches permitted applications.

A browser (`Accept: text/html`) that needs interaction is sent with a 303 to the sign-in page at `/saml/resume/{id}`. The user signs in there with a passkey, or a password and an optional TOTP or recovery code, and reviews the attributes the SP receives ("Shares these attributes"). The page also offers the terminal: `riauth request approve CODE` authenticates and approves there. Either way the original browser then posts the response to the registered ACS, and only a browser whose live session made the approval receives it. JSON callers still get the terminal instructions. `settings.implicit_consent` skips the review screen for first-party SAML applications. Verify browser sign-in and logout against the intended SAML application before deployment. Password reset and other account recovery still use the terminal ([lifecycle](lifecycle.md#what-still-needs-the-terminal)).

## Connect your first SAML service provider

Start with an SP that can **sign AuthnRequests** and accept HTTP-POST responses. Collect its metadata as a local XML file, verify its entity ID and ACS URLs with the SP owner, and choose a stable riAuth client ID. Configure an HTTPS issuer first ([operations](operations.md#tls-and-process-service)); HTTP is allowed only for loopback evaluation. The SP metadata import is an offline aid, not a trust-on-first-use download.

For a remote deployment, select its issuer and sign in in the shell where you will run the commands:

```sh
export RIAUTH_SERVER=https://id.example.com
riauth login admin
```

For a local instance, use `export RIAUTH_CONFIG=/path/to/riauth.toml` from the private operator directory instead; the CLI reads that file's issuer.

Import a matching RSA IdP private key and its public X.509 certificate. Run these commands from a private operator directory outside the source checkout, with `idp-private.pem` readable only by the operator. Existing Authentik keys/certificates and user subject overrides can preserve configured identity values. The normal `key.write` agent permission controls key import; `client.write` controls provider settings and manifests.

```sh
mkdir -p deployment-private
riauth keys import saml-signing --file idp-private.pem --algorithm RS256
riauth saml import-sp --file sp-metadata.xml \
  --entity-id https://sp.example.com/metadata \
  --idp-certificate idp-certificate.pem --out deployment-private/sp-import.json
```

The import selects one exact EntityDescriptor from local, explicitly provided metadata. It preserves ACS indices, pins SP signing certificates, and reports unsupported bindings/attribute requests for review. It makes no remote metadata, KeyInfo or entity fetches. Its trust comes from your reviewed input file; it does not claim an unpinned metadata signature proves trust. Review `deployment-private/sp-import.json`, especially `unresolved`, SP certificates, ACS URLs and NameID format. In a JSON client manifest with `api_version` set to `riauth/v1`, set `client_id` to your chosen ID, `service` to `false`, `scopes` to the report's `required_scopes` plus scopes for any attribute mappings, `settings.signing_key` to `saml-signing`, and `settings.saml` to the report's `settings.saml` object. The client may have an empty OAuth redirect list. The private key must match `idp_certificate_pem`. The report may also show an SP encryption certificate; enabling assertion encryption is a separate, reviewed choice. See the [manifest contract](agent.md#desired-state) for the plan workflow.

After writing the manifest, validate it, inspect the planned changes, then apply:

```sh
riauth validate --file saml-app.json
riauth plan --file saml-app.json --out deployment-private/saml-plan.json
riauth apply --plan deployment-private/saml-plan.json
```

A SAML policy client is interactive (`service: false`) with `openid` and `saml` scopes. It can have an empty OAuth redirect list. Group/access policy, MFA/default ACR, device-trust requirements and scope policies still apply. Active temporary group grants participate in these checks; [temporary access](enterprise/ENT-01.md) does not change durable directory membership. SAML attribute settings map explicitly registered claims to SP attribute names:

```json
{
  "name": "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress",
  "claim": "email",
  "friendly_name": "Email address",
  "required": true
}
```

This mapping requires the client's `email` scope. Other built-in mappings use `profile`, `groups` or `saml` (`sub`); custom claims use their existing scoped claim mappings. Scalars and arrays of text, booleans and integers are emitted as typed XML values. Required missing attributes fail closed. Arbitrary Python expressions are not executed.

Export signed IdP metadata after provisioning:

```sh
riauth saml metadata YOUR_CLIENT_ID --out idp-metadata.xml
```

Import that metadata into the SP. It supplies riAuth's IdP entity ID, SSO URL and signing certificate; keep the SP's request-signing certificate pinned in riAuth's client settings. Start a login **from the SP** and verify that its signed request reaches riAuth, the browser signs in and reviews consent, and the SP accepts the signed response at the registered ACS. Test a denied request and logout before rollout. If the request fails before sign-in, compare the exact SP entity ID, ACS URL or index, binding and signing certificate with the reviewed import. If the SP rejects the response, compare the IdP entity ID/certificate and the SP's expected NameID, attributes and response binding. If a user is denied, inspect scopes, group policy and assurance requirements rather than loosening signature checks.

Keep the IdP private key under operator control and update both sides when rotating certificates. An existing SP session depends on that SP's lifetime and logout implementation; test its behavior explicitly. This SAML profile has local and independent XMLsec test coverage. Test the intended service provider end to end; see [testing](testing.md) and [limitations](limitations.md).

Public endpoints, relative to the configured server issuer:

| Endpoint | Behavior |
| --- | --- |
| `/saml/{client_id}/metadata` | Signed IdP metadata XML |
| `/saml/{client_id}/sso` | Signed Redirect/POST AuthnRequest, LogoutRequest and LogoutResponse |
| `/saml/{client_id}/init` | IdP-initiated login, only when explicitly enabled |
| `/saml/resume/{request_id}` | Completion bound to the initiating browser cookie |

`idp_entity_id` optionally preserves a deployed issuer independently of endpoint URLs. ACS URLs are exact canonical HTTPS endpoints (HTTP loopback allowed); this profile does not accept endpoint query strings. `acs_indices` preserves explicit metadata indices. Persistent and unspecified NameIDs use stable per-client subject mapping; transient IDs are bound to the client/session; email NameIDs require a verified address and `email` scope. `AllowCreate=false` cannot create a new persistent/transient SP association unless a subject override was already provisioned. Qualifiers, a requested subject and the configured NameID format are checked.

`ForceAuthn` always requires a fresh authentication transaction bound to this request; on the sign-in page the signed-in account is pinned and must sign in again ("Confirm it's you"). Exact/minimum requested authentication contexts support Password, PasswordProtectedTransport and riAuth's MFA/federated context URIs. Unknown assurance, unsupported comparisons or unknown contexts are rejected. The actual authentication timestamp and context are placed in AuthnStatement; the provider does not let an agent invent an authentication claim. Passive requests return a signed `NoPassive` response when interaction is necessary.

`--remember`, or "Remember this decision for 30 days" on the page, records user consent for the current client configuration. Denying on the page sends `RequestDenied` to the SP. A terminal approval requires a sign-in within the last five minutes (sessions from OAuth-only sources have no sign-in time and are refused by the SAML identity check anyway). Subsequent browser SSO checks current identity, groups, scopes, policy, session, context and consent. Inspect/revoke it with `riauth consents`. Revocation also cancels approved SAML responses not yet collected by their browser. Request IDs have durable replay records; browser completion is one-use, expires after five minutes and rechecks live authorization.

XML signatures use RSA-SHA256/SHA256 and exclusive canonicalization. Requests must have a currently valid pinned RSA certificate (2048–4096 bits). POST signatures cover the complete request with a single root reference and a restricted transform list. DTDs, entities, external references, nested IDs, extension content and excessive parser structure are rejected. Optional encryption uses AES-256-GCM and RSA-OAEP (the interoperable XML Encryption MGF1/SHA-1 transport URI); the signed assertion is encrypted before the complete response is signed.

SP-initiated SLO requires the trusted SP signature, exact NameID and one to eight session indices issued to that SP. It revokes the associated riAuth sessions and queues their OIDC back-channel logout. Other SAML participants are contacted by sequential top-level browser Redirect/POST requests; the initiating participant is excluded. The original signed LogoutResponse is returned after those attempts. Its top-level Success confirms local revocation; a nested PartialLogout reports missing, failed, timed-out or unsupported remote participants, as required by SAML Core 3.7.3.2.

CLI `logout`, session revocation, and OIDC end-session/terminal-confirmed logout also initiate SAML propagation. CLI JSON returns `saml_logout_url` when a browser must complete it. Open that URL in the affected browser. Inspect the capability ticket at the end of the URL with `riauth --json saml logout-status TICKET`. Tickets do not authenticate a user or authorize new revocations. Existing local sessions are revoked before the browser contacts any SP. OIDC front-channel iframe delivery runs before SAML navigation when available; it remains best effort.

Progress is encrypted and persistent, with at most 64 distinct participants, 30 seconds per outstanding request, and ten minutes for the flow. A repeated resume returns the same request until its deadline; after the deadline, revisiting the resume URL records a failed participant and continues. This cannot automatically recover a browser stalled on another origin. Status records remain for one day. Every response requires a pinned signature, exact issuer/destination/request ID, the current participant and original ticket. Current client/source fingerprints are rechecked before delivery; a configuration change cannot send a prior subject to a new endpoint. Pre-upgrade records missing the raw session index are reported as incomplete.

SOAP/artifact bindings, ECP, encrypted NameIDs, and arbitrary flow translations are outside this browser profile. A provider's `source_stage` implements an OIDC authorization suspension, not an embedded stage inside an incoming SAML AuthnRequest. Upstream SAML authentication and SLO are described below. Already established SP sessions depend on that SP's logout/lifetime behavior; disabling a user without a participating browser does not remotely terminate every SP session.

Rotate SAML signing certificates and keys together with an SP metadata update. A certificate/key mismatch fails closed. The current SAML signer uses the local RS256 domain; Vault-only signing and overlap publication of multiple IdP certificates are not implemented for XML signatures. SAML XML security comes from Rhein Industries' maintained forks of the upstream stack: [risaml](https://github.com/Rhein-Industries/risaml) (saml-rs), [ribergshamra](https://github.com/Rhein-Industries/ribergshamra) (bergshamra), [ritsp-ltv](https://github.com/Rhein-Industries/ritsp-ltv) (tsp-ltv) and [riptering](https://github.com/Rhein-Industries/riptering) (kryptering); this is not a certification claim.

Tests cover request-bound authentication, browser binding, trusted callbacks, typed claims, indices, passive requests, replay, tampered input, encryption, consent/policy revocation and logout account isolation. An independent `xmlsec1` test signs POST requests, verifies metadata/response/assertion signatures and decrypts assertions:

```sh
RIAUTH_TEST_XMLSEC=/path/to/xmlsec1 cargo test --locked --test identity \
  saml_independent_xmlsec -- --ignored --nocapture
```

References: [OASIS SAML bindings](https://docs.oasis-open.org/security/saml/v2.0/saml-bindings-2.0-os.pdf), [SAML profiles](https://docs.oasis-open.org/security/saml/v2.0/saml-profiles-2.0-os.pdf), [Authentik SAML](https://docs.goauthentik.io/add-secure-apps/providers/saml/), [risaml](https://github.com/Rhein-Industries/risaml) (Rhein Industries' fork of [saml-rs](https://github.com/salasebas/opensaml-rs)).


## Upstream SAML identity providers

A source makes riAuth an SP for a pinned upstream IdP. Import a local RSA key with `riauth keys import source-sp --file sp-private.pem --algorithm RS256`, then provision a source with the normal `source put` command or an agent source manifest. The source's `issuer` and `client_id` are respectively the IdP and SP entity IDs. `authorization_endpoint` is the IdP's exact Redirect SSO endpoint.

```json
{
  "source": {
    "id": "corporate-saml",
    "name": "Corporate SAML",
    "issuer": "urn:company:idp",
    "authorization_endpoint": "https://idp.example.com/sso",
    "client_id": "urn:company:riauth-sp",
    "token_endpoint_auth_method": "none",
    "scopes": [],
    "auto_provision": false,
    "saml": {
      "signing_key": "source-sp",
      "sp_certificate_pem": "PUBLIC SP CERTIFICATE PEM",
      "idp_certificates_pem": ["PINNED IDP CERTIFICATE PEM"],
      "name_id_format": "persistent",
      "name_attribute": "displayName",
      "email_attribute": "email",
      "email_verified_attribute": "emailVerified",
      "require_encrypted_assertions": true
    }
  }
}
```

Replace PEM placeholders with the actual public certificates in JSON. The SP certificate must match its local RS256 key domain. SAML sources have no OAuth profile, scopes, token endpoint, client secret or JWKS. Metadata signing and decryption currently require a local key; Vault-only XML signing is outside this profile.

```sh
mkdir -p deployment-private
riauth source put --file corporate-saml.json
riauth source metadata corporate-saml --out sp-metadata.xml
riauth source start corporate-saml --out deployment-private/source-request.json
riauth source finish --file deployment-private/source-request.json
riauth source finish --file deployment-private/source-request.json --yes
```

A SAML source can also be selected by an OIDC provider's `settings.source_stage`. Its ACS then redirects to the bound stage resume route instead of requiring standalone `source finish`; source linking, MFA and request binding remain enforced. The source-stage regression currently drives OIDC/OAuth callbacks, so an end-to-end SAML-stage browser run remains acceptance work. See [ENT-11](enterprise/ENT-11.md).

Register the exported SP metadata at the IdP. Its public metadata and ACS endpoints are `/saml/sources/ID/metadata` and `/saml/sources/ID/acs`, relative to the configured issuer. Requests are signed Redirect AuthnRequests with ForceAuthn and an exact POST ACS. The callback requires both a signed response and a separately verified signed assertion, pinned RSA-SHA256 certificates, exact issuer/audience/recipient/request binding, stable NameID, bounded conditions and fresh authentication time. Original namespace context is preserved when independently verifying the assertion. DTDs, external references, ambiguous IDs, unexpected statements and unsafe signature transforms are rejected.

When encryption is configured, the signed outer response must verify before any decryption. The profile accepts signed assertions encrypted with AES-256-GCM and XML Encryption RSA-OAEP-MGF1/SHA-1. It rejects plaintext when encryption is required. Certificate rollover accepts up to four explicitly pinned IdP certificates. Source configuration changes revoke its local sessions and cancel pending requests.

Use `source start ID --link` with a fresh local session to link an existing user. Email and NameID similarities never automatically select a local account. Imported links name the exact source subject. Agent permissions cover the source, user and any provisioned groups; agents cannot enable administrator source login. Automatic provisioning is optional and never creates administrators. The browser receives only completion status; the private CLI transaction is needed to review the identity and issue a local session. Local TOTP/recovery requirements remain enforced.

Upstream MFA is false unless the signed AuthnContextClassRef exactly matches a configured `trusted_mfa_acr`. Its AuthnInstant is preserved. SessionNotOnOrAfter bounds the local session and subsequent online identity checks. Stable persistent, email and unspecified NameIDs are supported; transient identities, unsolicited IdP-initiated source login, artifact/SOAP/ECP flows are not advertised. Optional upstream SLO is described below.

The source tests cover independently verified outgoing Redirect signatures, signed/encrypted inbound authentication, explicit linking, email-collision isolation, replay, mismatched requests, issuer/audience/recipient checks, stale authentication, signature tampering, inherited namespaces, MFA trust, source changes and unlink revocation. An additional `saml_source_independent_xmlsec` test accepts responses signed by independent `xmlsec1` and verifies exported SP metadata.

### Upstream SAML logout

Set `saml.slo_redirect_url` and/or `saml.slo_post_url` to the IdP's exact registered SLO endpoints. When configured, SP metadata advertises `/saml/sources/ID/slo`. riAuth can send signed requests to that provider during local logout and accept its signed, correlated responses. Incoming IdP requests require a pinned signature, the original NameID/qualifiers and one to eight distinct upstream session indices. All matching local sessions are revoked, downstream applications are notified, and the signed response returns to the IdP. No request is sent back to the initiating source. Absent or changed source trust produces partial completion instead of forwarding old identity data.

The `saml_logout_multiple` test covers multiple SPs and an upstream IdP, exact account/index binding, local revocation, source loop prevention, request retries, failed responses, late responses, changed configuration and partial completion. Run the same scenarios with independent XML signatures:

```sh
RIAUTH_TEST_XMLSEC=/path/to/xmlsec1 cargo test --locked --test identity \
  saml_logout_independent_xmlsec -- --ignored --nocapture
```

Status semantics follow [SAML Core 3.7](https://docs.oasis-open.org/security/saml/v2.0/saml-core-2.0-os.pdf). Browser bindings cannot guarantee delivery when a participant is unavailable or the user closes the browser.
