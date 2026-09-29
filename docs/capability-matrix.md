# Capability and compatibility matrix

This page reads source revision `fdcbfee0950112986e5955f39bd28103d8b0319c`.
It records compiled inclusion, runtime prerequisites, protocol profile and
direction, peers named by tests, and known limits for Essentials, Platform,
and `riauthctl`. The assembly counts below are 87 names: 60 shared and 27
compiled only in Platform. `workflow.controlled_extensions` is the additional
Platform-only name.

The [A02 product contract](roadmap/product-contracts.md) and its
[capability matrix JSON](roadmap/capability-matrix.json) are the target
contract prepared against `96e23e2`. They are not this page. Where they
disagree with the source below, this page follows the source. The
[Q08 bundle note](roadmap/q08-exact-edition-bundles.md) records a different
commit, `4ca7558`, and its binary hashes do not apply here.

Labels used below:

| Label | Meaning |
| --- | --- |
| **Implemented** | The behavior is in this revision's source. |
| **In-tree test** | A test file asserts it. Writing this page did not run the test, Cargo, or CI. |
| **CI step** | [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) is written to run that test. No workflow result was collected for this commit. |
| **Recorded drill** | A JSON file under [roadmap/evidence](roadmap/evidence/) records a local run. Neither file stores a git revision. |
| **Roadmap** | Only the A02 contract asks for it. |
| **Untested peer** | No test in this repository connects to that product, tenant, or device. |

`riauth capabilities` (and `riauth --json capabilities`) calls
[`capability::artifact`](../src/capability.rs). Scope is `artifact`, schema
`riauth.capabilities/v2`, and `enabled`, `configured`, `runtime_ready`, and
`usable` are null. It does not open a database. A running server's
`GET /api/capabilities` is scope `instance`. Its `usable` flag is local
readiness. External peer health is outside that snapshot.

## Assemblies

The server crate refuses to compile until feature `essentials` is selected
([`src/lib.rs`](../src/lib.rs)). The Cargo default is `platform`, which
includes `essentials` ([`Cargo.toml`](../Cargo.toml)).

| Piece | How it is selected | What that build contains |
| --- | --- | --- |
| Essentials `riauth` and `riauth-maintenance` | `--no-default-features --features essentials` | 60 capability names. Same identity, authorization, revocation, and credential code as Platform. |
| Platform `riauth` and `riauth-maintenance` | default features, or `--features platform` | All 87 names: 60 shared and 27 Platform-only. Adds the listeners, routes, and agent actions below. |
| `riauthctl` | separate crate, default features empty ([`crates/riauthctl`](../crates/riauthctl/Cargo.toml)) | Remote HTTP administration. No server crate, redb, or PostgreSQL dependency. |
| `riauthctl` with `terminal-usb` | `--features terminal-usb` | Adds CTAP2 USB passkey login and enrollment. |

`riauth-maintenance` is the offline binary for `init`, `prepare-setup`,
`restore`, `recover-admin`, `migrate-postgres`, `keygen`, `import-authentik`,
and `transition-preflight` / `transition-plan` / `transition-activate`
([`src/cli/local.rs`](../src/cli/local.rs)). It uses the same feature set as
the server build that produced it. `riauthctl` has none of those commands.

The check job's main `cargo test` line uses the default Platform feature set
plus `test-support` and `fuzzing`. `test-support` is what lets cloud tests
point at a loopback fake. Shipped builds leave it off. The essentials binary
in that job is the USB-boundary build, which runs
`legacy_usb_commands_fail_locally_with_client_guidance`. Integration steps
also use the default Platform build. An in-tree test on that job is coverage
of the Platform build unless the row says otherwise.

### Names compiled in both editions

`portal.user_applications`, `portal.terminal_sign_in`,
`directory.ldap_sync`, `identity.ldap_authentication`, `identity.passkeys`,
`identity.email_verification`, `identity.invitations`,
`identity.email_password_reset`, `operations.postgresql`,
`operations.shared_rate_limits`, `operations.native_tls`, `oidc.par`,
`oidc.jar`, `oidc.jarm`, `oidc.claims_requests`, `oidc.dpop`,
`oidc.bound_key`, `oidc.pairwise_subjects`, `oidc.resource_indicators`,
`oidc.key_domains`, `oidc.jwe`, `oidc.provider_issuers`,
`oidc.frontchannel_logout`, `oidc.session_management`,
`identity.oidc_sources`, `identity.oauth_sources`, `identity.source_linking`,
`identity.totp_import`, `agents.source_manifests`,
`directory.scim_outbound`, `operations.prometheus`,
`operations.schema_migrations`, `oidc.private_key_jwt`,
`oidc.federated_machine_grants`, `oidc.token_exchange`,
`oidc.dynamic_registration`, `oidc.code.pkce_s256`, `oidc.device`,
`oidc.refresh_rotation`, `oidc.native_redirects`,
`oidc.request_bound_reauthentication`, `agents.scoped_credentials`,
`agents.plan_apply`, `agents.atomic_idempotency`,
`agents.conditional_mutations`, `agents.audit_run_id`, `agents.schema`,
`oidc.browser_terminal_handoff`, `oidc.rp_logout`, `oidc.backchannel_logout`,
`oidc.claim_mappings`, `oidc.scope_policies`, `oidc.provider_settings`,
`oidc.cors`, `operations.encrypted_backup_restore`,
`operations.encrypted_storage`, `identity.recovery_codes`,
`identity.self_password_change`, `operations.audit_review`,
`operations.csv_export`.

The registry is [`FEATURES`](../src/agent.rs) minus
[`PLATFORM_FEATURES`](../src/agent.rs).

### Names compiled only in Platform

`audit.self_hosted_event_map`, `saml.idp_signed_browser_sso`,
`saml.sp_initiated_logout`, `saml.logout_fanout`, `saml.upstream_logout`,
`saml.assertion_encryption`, `radius.pap`, `radius.radsec`, `radius.eap_tls`,
`agents.certificate_bindings`, `directory.ldap_provider`,
`proxy.forward_auth_sso`, `proxy.shared_domain_sso`, `proxy.reverse_proxy`,
`operations.vault_transit_signing`, `identity.saml_sources`,
`directory.scim_inbound`, `access.temporary_entitlements`,
`identity.scheduled_offboarding`, `identity.windows_device_login`,
`agents.parent_ownership`, `directory.workspace_sync`,
`directory.entra_sync`, `identity.https_client_certificates`,
`identity.device_trust`, `ssf.push`, `workflow.controlled_extensions`.

Essentials also omits agent actions `ldap.search`, `certificate.read`,
`certificate.write`, `mtls.read`, `mtls.bind`, `radius.enroll`,
`user.offboard`, `access.read`, `device.enroll`, `ssf.manage`,
`ssf.configure`, `workflow.read`, and `workflow.write`
([`PLATFORM_ACTIONS`](../src/edition.rs)). Directory wildcards and
`workspace/…` or `entra/…` resources are rejected on Essentials. Parent-owned
agents are rejected. The artifact schema list still includes `workflow`;
`radius-certificate`, `windows-device`, `windows-login`,
`client-certificate`, and `cloud-directory-plan` are omitted
([`src/schema.rs`](../src/schema.rs)).

Essentials rejects configuration that carries `proxy_listeners`,
`radius_listeners`, `ldap_listeners`, `workspace_directories`,
`entra_directories`, `signers`, `workflows`, `workflow_extensions`,
`pam_approvers`, `client_certificates`, or `device_trust`, plus Workspace or
Entra reconciliation controllers, `rate_limits` keys `saml`, `forward_auth`, and
`outpost_start`, and client settings `saml`, `radius`, `ldap`, `proxy`,
`source_stage`, `require_device_trust`, and certificate ACR. A SAML source
is rejected. An initialized store without edition provenance, a store last
activated as Platform, or a store that still holds a Platform record family
refuses Essentials open ([`src/edition.rs`](../src/edition.rs),
[`src/edition/transition.rs`](../src/edition/transition.rs)). Platform HTTP
routes, including inbound SCIM, SAML, proxy and outpost, certificate login,
Windows, cloud directories, SSF, workflows, the event map, and temporary
access, are registered only with the `platform` feature
([`src/api.rs`](../src/api.rs)).

The only `capabilities.disabled` entry either build accepts is
`identity.device_trust`, and only when that name is compiled. Every other
known name fails with `Capability {name} cannot be disabled by this build`.
Putting a Platform-only name in that list on Essentials fails earlier,
because the name is not compiled ([`src/capability.rs`](../src/capability.rs)).
On an initialized store, the compiled names that remain enabled are stored
with the issuer ([`src/node_security.rs`](../src/node_security.rs)). A process
whose set differs is refused before it binds. `configured` and
`runtime_ready` stay local. A back-channel logout claim stores a 60-second
lease and the worker pins it before the POST. See [node security](roadmap/o03-node-security.md).

## Runtime prerequisites

On an instance document, `enabled` is true unless the name is in
`capabilities.disabled`. `configured` is true for every name that the table
below does not list. `runtime_ready` is null except for the embedded reverse
proxy, RADIUS, and the LDAP provider, which need a live bound listener.
`usable` is compiled, enabled, configured, and ready (null ready counts as
ready).

| Names | `configured` becomes true when |
| --- | --- |
| `operations.postgresql` | `[postgres]` is set |
| `operations.native_tls` | TLS certificate and key files are both set |
| `operations.encrypted_storage` | a database key file is set |
| `identity.email_verification`, `identity.invitations`, `identity.email_password_reset` | `[mail]` is present and its local SMTP material checks pass |
| `directory.ldap_sync`, `identity.ldap_authentication` | at least one `[directories]` entry |
| `directory.scim_outbound` | at least one `[scim_targets]` entry |
| `directory.workspace_sync` | at least one Workspace directory |
| `directory.entra_sync` | at least one Entra directory |
| `directory.ldap_provider` | at least one LDAP listener |
| `proxy.forward_auth_sso` | an enabled client with valid proxy settings |
| `proxy.shared_domain_sso` | one of those clients sets a shared domain |
| `proxy.reverse_proxy` | listeners, routes, and matching client origins; the listener must be running |
| `radius.pap` | any RADIUS listener whose material and NAS clients are ready, and that listener is running. A ready RadSec or EAP listener also marks PAP configured |
| `radius.radsec` | a ready running listener with TLS transport |
| `radius.eap_tls`, `agents.certificate_bindings` | every EAP-TLS listener is ready, at least one exists, and one is running |
| `saml.idp_signed_browser_sso` | an enabled SAML client |
| `saml.sp_initiated_logout`, `saml.logout_fanout` | that client has an SLO URL |
| `saml.assertion_encryption` | that client has an encryption certificate |
| `saml.upstream_logout` | an enabled SAML source with an SLO URL |
| `identity.oidc_sources`, `identity.oauth_sources`, `identity.saml_sources` | an enabled source of that kind |
| `identity.source_linking` | any enabled source |
| `identity.https_client_certificates` | the profile validates and its verifier material loads |
| `identity.device_trust` | `[device_trust]` validates |
| `operations.vault_transit_signing` | at least one signer |
| `access.temporary_entitlements` | at least one PAM approver |
| `ssf.push` | at least one stored SSF stream |
| `workflow.controlled_extensions` | one active workflow whose key matches its definition id, validated as the extension-then-password graph, plus an admitted `workflow_extensions` manifest that covers that stage. Default, inactive, unsupported, rejected, and non-covering configurations stay false. Essentials does not compile the name, so configured and usable stay false. Usable does not add host imports, network, or a wider graph. The timeout is fuel only: `min(manifest fuel, timeout_seconds × 1,000)`, reported as `timeout` only when that budget is strictly smaller than the manifest fuel. Wasmi 0.40.0 `Config` has no epoch or interrupt. `call_hook` and `call_resumable` pause only around host calls, and this guest has no imports, so the budget does not preempt on wall-clock time. Admission also rejects more than one function, more than 32 i32 locals, or a data segment before compilation. The value stack is reserved at 64 `UntypedVal` slots; a frame that would reach that height is `limit` before the buffer grows. The first call charges 7 fuel per function-body byte and does not translate when that charge exceeds the installed budget. `Module::new` still validates the body before fuel is installed. A translation that fits is not wall-clock preempted |

`directory.scim_inbound`, `identity.windows_device_login`,
`identity.scheduled_offboarding`, `agents.parent_ownership`, and
`audit.self_hosted_event_map` have no extra instance prerequisite. `usable:
true` means the route can serve. It does not mean a SCIM client, Windows
host, or event-map peer is connected. Backup encryption is mandatory in the
product sense ([limitations](limitations.md)); `operations.encrypted_backup_restore`
does not itself check that a backup key file exists.

## Protocol profiles

Direction is from riAuth's side. "In-tree test" paths were not executed for
this page. CI steps are written for the default Platform build.

| Family | Edition | Direction and profile | Evidence | Limit |
| --- | --- | --- | --- | --- |
| OIDC provider | Both | Authorization server. Authorization code with mandatory S256 PKCE; refresh rotation; client credentials; device code; `private_key_jwt`; pinned JWT bearer; RFC 8693 access-token exchange (access tokens only, chains of at most four). PAR, JAR, JARM. DPoP. Response modes `query`, `fragment`, `form_post`, and the `.jwt` forms. JWE `RSA-OAEP-256` with `A256GCM` or `A256CBC-HS512`. Restricted RFC 7591 registration. RP, front-channel, and back-channel logout. Client authentication `none`, `client_secret_basic`, `client_secret_post`, `private_key_jwt`. | [OIDC relying-party recipe](recipes/oidc-relying-party.md), [oidc-profiles.md](oidc-profiles.md). In-tree tests in [`tests/identity/oidc.rs`](../tests/identity/oidc.rs). Ignored test `browser_terminal_login_callback_and_signed_backchannel_logout` in [`tests/browser.rs`](../tests/browser.rs) is the integration job's "Real browser and relying party" step. | Implicit and hybrid grants are rejected (`Only authorization code is supported`). RFC 7592 management and automatic sector-identifier retrieval are absent. [`scripts/run-conformance.py`](../scripts/run-conformance.py) pins OIDF suite `440eec8` and is not called by CI. No result file is in the tree. The browser fixture's client is the in-process router, not a named external relying party. |
| Upstream OIDC | Both | Client of an upstream issuer. Signed code plus S256. Upstream client authentication is `none`, Basic, or POST. Browser completion is cookie-bound; CLI `source start` / `source finish` keeps the session off the callback. | [Upstream OIDC recipe](recipes/upstream-oidc.md), [oidc-profiles.md](oidc-profiles.md). Function `upstream_oidc_pkce_pinned_keys_claim_validation_and_terminal_completion` in [`tests/identity/sources.rs`](../tests/identity/sources.rs). The check job's `cargo test --all-targets --features test-support,fuzzing --locked` does not pass `--ignored`, and this function is not ignored. This page did not run that command. | Encrypted upstream ID tokens and upstream `private_key_jwt` are rejected. The recipe issuer is an in-process loopback `POST /token` signed with riAuth's own active key. No Okta, Entra, or Google OIDC tenant. |
| Upstream OAuth JSON identity | Both | Client. Pinned userinfo URL, no `openid`, no ID token, no authentication time. | Same guide and `tests/identity/sources.rs`. | Cannot satisfy request-bound reauthentication. Email does not link accounts. |
| Embedded source stage | Platform client setting | Suspends an interactive authorization for one configured OIDC or OAuth source. Essentials rejects `settings.source_stage`. | [oidc-profiles.md](oidc-profiles.md), [`src/edition.rs`](../src/edition.rs). | A stage is not embedded inside a SAML AuthnRequest. Browser OTP for a required local factor is still a gap in the guide. |
| SAML IdP | Platform | Identity provider. Signed HTTP-Redirect and HTTP-POST AuthnRequest, HTTP-POST response, signed metadata, signed assertion and response. Optional assertion encryption (AES-256-GCM, RSA-OAEP). NameID persistent, transient, email, unspecified. SP-initiated SLO and IdP logout fan-out over Redirect/POST. IdP-initiated login only at `/saml/{client}/init` when enabled. | [Platform SAML IdP recipe](recipes/platform-saml-idp.md), [saml.md](saml.md). In-tree tests [`tests/identity/saml.rs`](../tests/identity/saml.rs), [`tests/identity/saml_logout.rs`](../tests/identity/saml_logout.rs). The recipe follows the integration job's "Independent SAML XML signature and encryption" step. The upstream-source and logout xmlsec1 steps are separate. | SOAP, artifact, ECP, and encrypted NameID are outside the profile. xmlsec1 checks signatures; it is not a service provider. Ignored [`scripts/test-saml-sp.sh`](../scripts/test-saml-sp.sh) ran GNU Lasso 2.9.0 (Homebrew bottle 2.9.0_4, linked to xmlsec1 1.3.12) as a loopback SP: a redirect RSA-SHA256 AuthnRequest, an HTTP-POST accept that printed a persistent NameID, and signature failure `-111` for a one-byte NameID change and for metadata whose every `X509Certificate` was replaced. `lasso_set_min_signature_method` is not exported, so the library minimum stays RSA-SHA1. After signature verification it requires Lasso audience validation for its provider ID and `lasso_saml2_assertion_validate_time_checks` at the current time. A re-signed response whose `Conditions` `NotOnOrAfter` equals that element's `NotBefore` exited 1 with `lasso lifetime: assertion lifetime is not valid (1)` and no NameID, without signature error `-111`. The unmodified response printed `conditions: valid`. Recipient is not compared. That helper is outside CI. Named production SPs remain unverified. |
| SAML source | Platform | Service provider. Signed Redirect AuthnRequest, POST ACS, optional encrypted assertions, optional Redirect/POST SLO. Stable persistent, email, and unspecified NameIDs. | [Platform SAML source recipe](recipes/platform-saml-source.md), [saml.md](saml.md). `exercise` in [`tests/identity/saml_source.rs`](../tests/identity/saml_source.rs) is called by non-ignored `saml_source_signed_encrypted_terminal_linking_and_live_trust` and ignored `saml_source_independent_xmlsec_responses`. The check job's `cargo test --all-targets --features test-support,fuzzing --locked` does not pass `--ignored`. The integration job's "Independent upstream SAML signatures" step is `RIAUTH_TEST_XMLSEC="$(command -v xmlsec1)" cargo test --test identity --locked saml_source_independent_xmlsec -- --ignored`. This page did not run either command. | Transient NameIDs, unsolicited IdP-initiated source login, artifact, SOAP, and ECP are not advertised. The recipe IdP is the in-process `Upstream` helper. xmlsec1 signs when `RIAUTH_TEST_XMLSEC` is set. No named external IdP. |
| LDAP import and password check | Both | Client of an external directory. Search and simple bind. Local accounts are created or disabled in riAuth. Directory passwords are not copied. | [LDAP import recipe](recipes/ldap-import.md), [ldap.md](ldap.md). CI step [`scripts/test-ldap.sh`](../scripts/test-ldap.sh) with ignored [`tests/ldap.rs`](../tests/ldap.rs). | The harness is loopback OpenLDAP `slapd` (distro package, version not pinned) with core, cosine, and inetOrgPerson. Active Directory attribute names in the guide have no AD server in the tests. |
| LDAP provider | Platform | Read-only LDAPv3 server. Simple bind, LDAPS, mandatory STARTTLS, root DSE, Who Am I, equality, presence, substring, and boolean filters, RFC 2696 paging. | [LDAP-provider recipe](recipes/platform-ldap-provider.md), [ldap-provider.md](ldap-provider.md). In-tree test `ldap_provider_tls_scoped_search_paging_rebind_mfa_and_revocation` in [`tests/identity/network.rs`](../tests/identity/network.rs), using `ldap3` against riAuth. The check job's `cargo test` is written to run it. Ignored [`tests/ldap_provider_peer.rs`](../tests/ldap_provider_peer.rs) runs from [`scripts/test-ldap-provider.sh`](../scripts/test-ldap-provider.sh) with an operator-supplied OpenLDAP ldapsearch. | Add, modify, delete, modifyDN, and compare return unwilling to perform. POSIX and AD schema emulation is not advertised. The client in the ldap3 test is not a third-party directory product. Loopback STARTTLS was exercised with OpenLDAP ldapsearch 2.7.1 (`scripts/test-ldap-provider.sh`): the service bind and page size 1 returned `ldap-alice` and `ldap-bob` on separate pages and exited 0, a disabled member was omitted, and a wrong token and a revoked agent each exited 49 with `ldap_bind: Invalid credentials (49)`. The confirming run also used a separate LDAPS listener at `127.0.0.1:60248` without `-ZZ`: page size 1 returned the same two users and exited 0, a disabled member was omitted, a wrong token and a revoked agent each exited 49, and an unrelated CA exited 1 with `certificate verify failed (unable to get local issuer certificate)`. `ldaps://` on the STARTTLS port `127.0.0.1:60249` and `ldap://` with `-ZZ` on the LDAPS port each exited 1 before a bind. Active Directory remains unverified. |
| SCIM inbound | Platform | Server at `/scim/v2`. Users and Groups: GET, POST, PUT, PATCH, DELETE, filtered list, POST `.search`. Sort is advertised. Bulk is advertised false. | [Inbound SCIM recipe](recipes/platform-inbound-scim.md), [scim.md](scim.md), [`src/scim.rs`](../src/scim.rs). Function `scim_http_provisioning_is_owned_atomic_retriable_and_deprovisions_sessions` in [`tests/identity/http.rs`](../tests/identity/http.rs). The check job's `cargo test --all-targets --features test-support,fuzzing --locked` is written to include that non-ignored test. This page did not run it. [`tests/scim_filters.rs`](../tests/scim_filters.rs) and [`tests/scim_pagination.rs`](../tests/scim_pagination.rs) are separate. | Nested groups and enterprise or custom schemas are rejected. No named SCIM client. The recipe's HTTP peer is in-process `oneshot`. The A02 contract still lists sort among initial exclusions; this tree implements sort with a 4,096-candidate cap. |
| SCIM outbound | Both | Client. Selected users and optional groups. A reviewed departure sets the remote user inactive and empties the selected group. The remote user remains. | [Outbound SCIM recipe](recipes/platform-outbound-scim.md), [scim.md](scim.md). Function `outbound_scim_plans_provision_groups_preserve_remote_attributes_disable_departures_and_stop_stale_jobs` in [`tests/identity/policy.rs`](../tests/identity/policy.rs). The check job's `cargo test --all-targets --features test-support,fuzzing --locked` does not pass `--ignored`, and this function is not ignored. This page did not run that command. | The recipe peer is a second riAuth router on loopback HTTP. No named SaaS directory. That function leaves `delivery_state`, the 12-attempt item stop, and reconciliation modes unread. A local disable records a separate deactivation row, which stops after five attempts. Recording that row is not delivery, and this function does not read it. |
| RADIUS | Platform | Server. PAP and EAP-TLS (method 13) on UDP, and the same on RadSec (TLS, RFC 6614). Message-Authenticator required. Access-Request only. | [radius.md](radius.md). In-tree tests in [`tests/identity/network.rs`](../tests/identity/network.rs) and [`tests/identity/radius_eap.rs`](../tests/identity/radius_eap.rs). The EAP test uses the `openssl` crate as the TLS client. Ignored [`tests/radius_peer.rs`](../tests/radius_peer.rs) runs from [`scripts/test-radius.sh`](../scripts/test-radius.sh) with an operator-supplied FreeRADIUS radclient. | Accounting, CHAP, MS-CHAP, PEAP, TTLS, CoA, and Disconnect are outside the profile. Loopback PAP was exercised with FreeRADIUS radclient 3.2.10 (`scripts/test-radius.sh`). Hardware NAS interoperability remains a deployment check. |
| Forward auth | Platform | Outpost for nginx `auth_request` and Traefik forwardAuth. Embedded login is authorization code plus S256. Proxy clients cannot redeem codes at `/oauth/token`. | [Forward-auth recipe](recipes/platform-forward-auth.md), [proxy.md](proxy.md), [`deploy/nginx-forward-auth.conf`](../deploy/nginx-forward-auth.conf), [`deploy/traefik-forward-auth.yml`](../deploy/traefik-forward-auth.yml). CI steps: ignored [`tests/outpost.rs`](../tests/outpost.rs) with distro nginx, and [`tests/outpost_traefik.rs`](../tests/outpost_traefik.rs) with Traefik **v3.7.13** (sha256 pinned in the workflow). | Confidential clients and clients that require PAR, JAR, or DPoP are rejected for this profile. riAuth does not install or upgrade the proxy. An open WebSocket is not rechecked by the nginx or Traefik path. |
| Embedded reverse proxy | Platform | Server listener. HTTP/1.1 reverse proxy with WebSocket upgrade. Open sockets recheck authorization every 30 seconds. | [proxy.md](proxy.md). In-tree test `rust_reverse_proxy_terminal_sso_headers_websocket_and_revocation` in `tests/outpost.rs`. | No managed gateway fleet and no credential injection. |
| Shared Signals | Platform | Receive and send signed push (`urn:ietf:rfc:8935`). Events for account disable, session revoke, and credential change. | [ENT-07](enterprise/ENT-07.md), [`src/ssf.rs`](../src/ssf.rs). In-tree test [`tests/ssf.rs`](../tests/ssf.rs). | Poll (`urn:ietf:rfc:8936`), stream verification, and SSF subject-management endpoints are absent. Apple Business Manager is named in the guide as untested. |
| Workspace sync | Platform | Client of Admin SDK Directory. Inbound users and selected groups. Direct service-account JWT, or a broker that speaks client-credentials. Reviewed plan before local writes. | [ENT-03](enterprise/ENT-03.md). In-tree mock [`tests/cloud_directory.rs`](../tests/cloud_directory.rs). | No Workspace tenant. Outbound directory writes are absent. `test-support` is required for the fake peer. |
| Entra sync | Platform | Client of Microsoft Graph. Inbound users and selected groups. Client secret or certificate credential. Reviewed plan before local writes. | [ENT-04](enterprise/ENT-04.md). Same mock test file. | No Entra tenant. |
| Device trust | Platform | Local nonce-bound JWT, or an explicit `google_verified_access_v2` adapter that calls only the pinned v2 generate and verify endpoints and the Google OAuth token endpoint. Fails closed when a client requires device trust and the verifier or the response identity is absent. Before verify, the response must embed the issued challenge. | [ENT-06](enterprise/ENT-06.md), [`src/device_trust.rs`](../src/device_trust.rs), [`src/verified_access.rs`](../src/verified_access.rs). In-tree test [`tests/device_trust.rs`](../tests/device_trust.rs). | No managed Chrome tenant. The adapter was not executed against `verifiedaccess.googleapis.com`. The device signature over `ChallengeResponse` stays with Google's verify endpoint. `deviceSignals` are not evaluated. Profile-only responses are rejected. |
| HTTPS client certificates | Platform | Direct TLS client-certificate authentication when trust material loads. Certificate AMR is not an MFA claim. | [ENT-05](enterprise/ENT-05.md). | Essentials uses the ordinary TLS acceptor and has no client-certificate verifier. |
| Vault Transit | Platform | External signer for ID, access, UserInfo, JARM, and back-channel logout tokens. The private key stays in Vault. | [kms.md](kms.md). In-tree stand-in in [`tests/identity/operations.rs`](../tests/identity/operations.rs). | No live Vault server. |
| Windows device login | Platform | Enrollment and ticket protocol for a separate device host. | [ENT-13](enterprise/ENT-13.md), [`src/windows_login.rs`](../src/windows_login.rs), [`tests/windows_login.rs`](../tests/windows_login.rs), [`windows/README.md`](../windows/README.md). | The guide states that no Windows interactive login has been tested. [limitations.md](limitations.md) says a packaged credential provider is not included. This page did not build or run Windows. |
| Workflows | Platform for configured definitions | Seven shipped builtin definitions are the Essentials profile (`essentials-passkey-sign-in`, `essentials-password-sign-in`, `essentials-passkey-enrollment`, `essentials-invitation`, `essentials-password-reset`, `essentials-consent`, `essentials-sensitive-action`). Platform adds `riauth.workflow/v1` authoring and the configured executor paths in [workflows.md](workflows.md). | [`src/workflow/essentials.rs`](../src/workflow/essentials.rs), [`tests/workflow_model.rs`](../tests/workflow_model.rs). | One supported guest runs on Platform: a custom `extension` step with outputs `allow` and `block` and no `network` permission, then a local `password` step, inside Wasmi 0.40. The guest has no imports and one 64 KiB page. Its timeout is fuel only. The engine installs `min(manifest fuel, timeout_seconds × 1,000)` before start, with fuel from 1 to 10,000 and 1,000 fuel per manifest second. Exhaustion is `timeout` only when that timeout budget is strictly smaller than the manifest fuel. Equal budgets report `fuel`. A timeout of 10 seconds or more cannot be tighter than the 10,000 fuel cap, so that exhaustion is `fuel`. Wasmi 0.40.0 `Config` has no epoch or interrupt, and `call_hook` / `call_resumable` pause only around host calls. This guest has no imports, so the budget does not preempt on wall-clock time. Admission rejects more than one function, more than 32 i32 locals, or a data segment before Wasmi compiles the module. The value stack is reserved at 64 `UntypedVal` slots, and a frame that would reach that height is `limit` before the buffer grows. The first call charges 7 fuel per function-body byte and does not translate when that charge exceeds the installed budget. `Module::new` still validates the body before fuel is installed. A translation that fits is not wall-clock preempted. Other custom graphs, and several verifier and browser paths, remain later work. The editor does not author new conditions, custom stages, or source references. |
| Browser administration | Both, with Platform sections | `/admin` for applications, people, and groups, calling the same management service as the API and `riauthctl`. Platform adds workflow authoring, connectors, temporary access, and the event map. | [PORTAL.md](PORTAL.md). In-tree tests [`tests/admin_ui.rs`](../tests/admin_ui.rs), [`tests/policy_simulation.rs`](../tests/policy_simulation.rs). | Help-desk page controls are not tailored to delegated roles; the server still enforces the exact permission. |
| Passkeys in the browser | Both | WebAuthn enrollment and sign-in on the portal. | [passkeys.md](passkeys.md), [PORTAL.md](PORTAL.md). Specs under [`tools/browser`](../tools/browser) use a Chromium virtual authenticator. | Those specs are not the CI Playwright command. Physical keys, synced passkeys, phone hybrid, iOS, Android, and screen readers are manual gates in the [Essentials](essentials-guide.md) and [Platform](platform-guide.md) guides. |
| Terminal USB | `riauthctl` optional feature | CTAP2 USB on the client. The server binary rejects the old commands with `Terminal USB passkeys moved to riauthctl; install/build riauthctl with --features terminal-usb`. The base client rejects them with `USB passkeys are unavailable in this build; rebuild riauthctl with --features terminal-usb`. | [`src/cli/usb.rs`](../src/cli/usb.rs), [`crates/riauthctl/src/usb.rs`](../crates/riauthctl/src/usb.rs). CI builds the Essentials server, checks the Essentials and Platform dependency trees for USB crates, tests base `riauthctl`, and `cargo check`s `terminal-usb`. | No hardware authenticator run. USB needs interactive touch or PIN. |
| redb and backup | Both | One owning process for redb. `riauth backup` writes encrypted v3 streams (default 4 GiB). Legacy JSON backup is v2 and capped at 64 MiB. Restore invalidates sessions and grants. | [limitations.md](limitations.md), [recovery.md](recovery.md). Recorded drill [r05-local-2026-09-29.json](roadmap/evidence/r05-local-2026-09-29.json) (Platform binary, localhost redb, 16 checks passed). | The JSON does not name a commit, so it is not a pass record for this revision. Keys and referenced secret files are outside the archive. The drill's own `external_gates` still include a real relying party and Vault. |
| PostgreSQL | Both | Multiple service processes may share one database. Replication, election, and fencing stay with the deployment. | [availability.md](availability.md). CI steps [`scripts/test-postgres.sh`](../scripts/test-postgres.sh) and [`scripts/test-contracts-postgres.sh`](../scripts/test-contracts-postgres.sh) use the distro `postgresql` package, version not pinned. Recorded drill [r05-postgres-local-2026-09-29.json](roadmap/evidence/r05-postgres-local-2026-09-29.json) quotes `pg_ctl (PostgreSQL) 16.14 (Homebrew)` on a loopback cluster. | That drill is not the CI replication script, does not name a commit, and leaves PITR, multi-node failover, and TLS to PostgreSQL in `external_gates`. |

`riauthctl` commands, all remote, are `status`, `discovery`, `login`,
`whoami`, `logout`, `revision`, `inventory`, `user`, `group`, `client`,
`session`, `request`, `device`, `authorize`, `plan`, and `apply`, plus
`passkey` when `terminal-usb` is compiled
([`crates/riauthctl/src/main.rs`](../crates/riauthctl/src/main.rs)). The
issuer must be canonical HTTPS, or HTTP on loopback. Discovery's token
endpoint must be `--server` plus `/oauth/token`. Redirects are rejected.
In-tree coverage is a local TCP fixture in
[`crates/riauthctl/tests/security.rs`](../crates/riauthctl/tests/security.rs).
The check job is written to run that suite without default features. The
client cannot activate a capability the server build omitted.

## Peers the repository actually names

Versions are pinned only where the workflow pins them. "CI step" means the
integration job on `ubuntu-24.04` is written to run the test. This page did
not run it. Package versions for `postgresql`, `slapd`, `nginx`, `xmlsec1`,
and Google Chrome are whatever that runner installs. Chrome is the unpinned
`google-chrome-stable_current_amd64.deb`. Playwright is
`@playwright/test` 1.63.0; CI installs Chromium, Firefox, and WebKit and runs
only [`tools/browser/setup.spec.js`](../tools/browser/setup.spec.js) on all
three projects. That spec is first-administrator cookie setup. It is not a
passkey journey. The WebKit project uses Playwright's Desktop Safari device
profile, which is not Apple Safari.

| Peer | What is exercised | Status |
| --- | --- | --- |
| PostgreSQL server | Loopback clusters, including a primary and standby in `scripts/test-postgres.sh` | CI step. Separate Homebrew 16.14 drill, no commit id |
| OpenLDAP `slapd` | Loopback import, STARTTLS, password login, MFA, fail-closed sync | CI step. [LDAP import recipe](recipes/ldap-import.md). No Active Directory server |
| riAuth LDAP listener | `ldap3` against this server's LDAPS and STARTTLS, and local OpenLDAP ldapsearch 2.7.1 LDAPS and STARTTLS via `scripts/test-ldap-provider.sh` | Check-job test for ldap3. The ldapsearch run is a local ignored test. No Active Directory server |
| nginx | `auth_request` template, headers, WebSocket, revocation, optional Chrome | CI step. Package version not pinned |
| Traefik v3.7.13 | forwardAuth template, headers, WebSocket origin, revocation | CI step. SHA-256 pinned in the workflow |
| xmlsec1 | Independent sign, verify, and decrypt for the [Platform SAML IdP recipe](recipes/platform-saml-idp.md). The [SAML source recipe](recipes/platform-saml-source.md) signs responses and verifies SP metadata in the upstream-source filter. Logout is a separate filter | CI step. Not an IdP or SP |
| GNU Lasso 2.9.0 | Loopback SP helper [`scripts/lasso-saml-sp.c`](../scripts/lasso-saml-sp.c) via [`scripts/test-saml-sp.sh`](../scripts/test-saml-sp.sh): signed redirect AuthnRequest, HTTP-POST response, NameID tamper, metadata-certificate rejection, and re-signed Conditions lifetime rejection | Local ignored test. Bottle 2.9.0_4, linked to xmlsec1 1.3.12. Fixture entity `https://sp.example.test/metadata`. Outside CI |
| Google Chrome | Headless user agent for the [OIDC relying-party fixture](recipes/oidc-relying-party.md), plus portal layout and nginx SSO when `RIAUTH_TEST_BROWSER` is set | CI step. Deb is not checksummed |
| Playwright Chromium, Firefox, WebKit | `setup.spec.js` only | CI step |
| Authentik | Offline API-export conversion | Fixture in [`tests/identity/operations.rs`](../tests/identity/operations.rs). No Authentik process |
| Keycloak, Okta, Active Directory | Names in migration-inventory fixtures that stay blocked | Fixture only. No server |
| Workspace, Entra, Vault | Loopback mocks | Fixture only. No tenant and no Vault server |
| RADIUS NAS | FreeRADIUS radclient 3.2.10 loopback PAP via `scripts/test-radius.sh`, plus the in-process client and OpenSSL EAP-TLS | Hardware NAS remains a deployment check |

Other Playwright specs (`signin`, `portal`, `admin`, `accessibility-journeys`,
`workflow-editor`) are in the tree and are not in the CI command. The
coverage inventory's older note that Playwright never starts is stale against
current `ci.yml`; that command still does not run those files.

## Release, Linux, and Windows

[`.github/workflows/release.yml`](../.github/workflows/release.yml) is written
to package Essentials and Platform native archives and images, plus
`riauthctl` and edition-matched `riauth-maintenance` archives, for
`linux/amd64` and `linux/arm64`, when a `v*` tag points at current `main`.
Server and base client release builds omit terminal USB. This commit is not
that tag. No release asset was downloaded for it. The draft `v0.1.1` assets
described in the Q08 note belong to commit `5ef0261`, not to this revision.

No Windows interactive logon, no Apple Safari.app, and no iOS or Android
device were run. The Linux network-filesystem probe described in
[limitations.md](limitations.md) was not run for this page.

## Still open

The [Platform forward-auth recipe](recipes/platform-forward-auth.md), the
[Platform LDAP-provider recipe](recipes/platform-ldap-provider.md), the
[OIDC relying-party recipe](recipes/oidc-relying-party.md), the
[Platform SAML IdP recipe](recipes/platform-saml-idp.md), the
[LDAP import recipe](recipes/ldap-import.md), the
[upstream OIDC recipe](recipes/upstream-oidc.md), the
[inbound SCIM recipe](recipes/platform-inbound-scim.md), the
[Platform SAML source recipe](recipes/platform-saml-source.md), and the
[outbound SCIM recipe](recipes/platform-outbound-scim.md) are the D03
recipes in this tree. The relying-party page's client is the in-tree axum
fixture, so a named external relying party remains an open peer. The SAML
IdP page checks signatures with xmlsec1, and a separate ignored GNU Lasso 2.9.0 helper exercised one loopback profile, including a re-signed Conditions lifetime rejection. A named production service provider remains
an open peer. The import page follows disposable loopback OpenLDAP, so a
real Active Directory directory remains an open peer. The upstream OIDC
page's issuer is the in-process loopback token endpoint, so Okta, Entra,
and Google remain unconnected. The inbound SCIM page follows
`router(...).oneshot` in the test process, so no named SCIM client is
connected. The SAML source page follows the in-process `Upstream` helper,
so no named external IdP is connected. The outbound SCIM page follows
a second riAuth router on loopback HTTP, so no named SaaS directory is
connected. The other D03
integration recipes, D04 emergency runbooks, and D05 acceptance against
the category targets are still open. The
[acceptance evidence matrix](roadmap/d05-acceptance-evidence.md) classifies
the evidence that exists and leaves every category unpassed. So are a
conformance result, a named relying party or service provider, a
Workspace or Entra tenant, a live Vault, a hardware authenticator, and an
installed-release run of this commit on Linux x86-64 and ARM64.
