# I04 supported-profile evidence review

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Original task: `dae9c528-9e32-462c-947f-661a571f136b` (`I04`).
Supporting worktree: `e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`, branch
`roadmap/local-extension-isolation-wave27`. Original primary worktree remains
`a2dff16a-c4b0-47fc-96de-ac85b1fb6d9e`.
Published source reviewed: `9a819317efb3a13fa27cd86f884be2be00898fc0`.
Supporting branch parent: `dc4754475342fed022b85329c50430e37a7f6845`.

Recommendation: credit the exact executed local profiles below; the inspected
records do not establish the whole original I04 outcome. Propose **one local GNU
Lasso SP logout receiver slice**. There is a concrete missing peer-lifecycle
check, rather than an established product defect. This report neither changes
status nor authorizes implementation or runtime. The proposal was successfully
sent to the explicit project orchestrator before this report was written.

## Original row and review method

The live RiWork row read for this review is `in_progress`, assigned to its
original primary worktree. Its exact requested outcome is:

> Test exact supported roles, bindings, methods, mappings, certificates, and revocation behavior against real peers.

The workstream gate is:

> Every advertised integration has a working setup, lifecycle, and failure-handling path—not just an endpoint or protocol module.

Prerequisites are A02, A06 and Q02. Completion evidence calls for relevant
implementation, tests, documentation and released artifacts as applicable,
actual verification, remaining gaps and external prerequisites; it expressly
rejects completion from documentation or a worker report alone. The row retains
an older scheduling paragraph; this support followed the current explicit user
assignment, without starting a worker or changing the row.

All product, guide, script and assertion-body reads used immutable `git show`,
`git grep`, tree/blob lookup or comparison at the published pin. No source
worktree was read, merged, reset or contacted. No protocol binary, Cargo,
service, provider, browser or desktop was started. Existing accepted raw logs
and redacted orchestrator evidence were read only. Definitions and historical
execution are identified separately; equal fixture blobs do not prove the
entire current server was executed. This is not an I02, D01 or I10 re-audit.

## Exact advertised profiles and observed evidence

The operator contracts are [SAML](../saml.md), [LDAP import](../ldap.md),
[LDAP provider](../ldap-provider.md) and [RADIUS](../radius.md). The current
[limitations](../limitations.md) explicitly bound the advertised profiles.
Essentials includes LDAP import/password authentication; SAML, LDAP provider,
RADIUS/RadSec/EAP-TLS are Platform capabilities.

| Direction/profile at the published pin | Security, mapping and lifecycle contract | Real evidence inspected; exact limit |
| --- | --- | --- |
| riAuth SAML IdP: signed Redirect or POST AuthnRequest; POST response | Exact entity/ACS or configured ACS index; persistent/unspecified/transient/email NameID with format-specific conditions; scoped registered typed attributes, required attributes fail closed; ForceAuthn/IsPassive/context/consent/browser binding; signed metadata, assertion and response | GNU Lasso 2.9.0 is an actual external **library receiver** for Redirect request → POST response, persistent NameID, empty attribute mapping. Its signature/audience/lifetime checks are library calls; Recipient is a helper comparison. XMLsec CI independently signs a POST request, verifies metadata/response/assertion and decrypts the encrypted assertion. Neither check establishes all advertised NameIDs, mappings, bindings or a production SP session. |
| riAuth SAML SP/source: signed Redirect request; signed POST response and separately signed assertion | Stable persistent/email/unspecified NameID, exact issuer/audience/Recipient/InResponseTo; explicit linking, no adoption by email; display/email/verified mappings, exact trusted MFA context; optional required AES-256-GCM assertion encryption; 1–4 pinned IdP certificates and live source fingerprint | XMLsec CI really signs the source fixture messages and verifies SP metadata. The IdP protocol behavior/XML structure is the Rust `Upstream` helper, and encryption is still riAuth's risaml path. It is an independent **signature tool**, not an external IdP. The accepted native preparation only reached `source start`, not ACS/finish. No completed real IdP source exchange is established here. |
| SAML SLO: SP/IdP initiation and upstream source logout, Redirect/POST propagation | Signed pinned peer, exact subject/qualifiers and 1–8 distinct issued indices; revoke local session before propagation, no loop back to initiating participant; current fingerprints and pending request/relay/peer; Success vs PartialLogout; replay/account isolation | Actual CI `saml_logout_independent_xmlsec` checks XMLsec signatures for POST and OpenSSL signatures for Redirect while the test constructs peer protocol messages. It asserts local revocation, unrelated-user preservation, retries, partial completion and late/changed-peer failures. No Lasso logout receiver exists in the inspected helper/test. Its accepted SP-authentication test explicitly has both SLO endpoints `None`. |
| LDAP importer/password source: service search, simple user bind; STARTTLS/LDAPS, bounded paging | Immutable text/binary stable IDs, explicit username/display/email/group-user-filter mapping; no password-hash copy/adoption/local fallback; actor/config/revision/removal-confirmed atomic apply; search errors cannot become removals; rename retains identity/factors; sync/revocation invalidates sessions | Actual private OpenLDAP `slapd` STARTTLS CI fixture: `entryUUID`, `uid`, `cn`, `mail`, group filter `description=staff`, 205 entries, actual user/password binds. Includes untrusted CA, MFA/recovery replay, stale source, collision, revoked agent and exact removal confirmation. This is real directory behavior, not AD `objectGUID`/`sAMAccountName`/`memberOf`, not a real LDAPS **import** run. Provider LDAPS evidence must not be credited to this direction. |
| LDAP provider: read-only LDAPv3 simple user/service binds, LDAPS or mandatory STARTTLS | Explicit peers/client policy/scopes; service token `ldap.search`, live agent revocation; self-only user view and scoped durable group projection; no credential/private attributes; bounded RFC2696 cursors/filters; failed rebind clears authority | Accepted OpenLDAP `ldapsearch` 2.7.1 external client on separate actual LDAPS and STARTTLS listeners: one-entry pages, two scoped users, disabled-member omission, wrong/revoked token rc49, unrelated CA rc1, crossed schemes refuse before bind. Independent `ldap3` network assertion bodies additionally define user/factor/root-DSE/WhoAmI/rebind/paging checks; their presence is not a new execution claim. No AD/POSIX schema emulation or arbitrary application compatibility is inferred. |
| RADIUS PAP server on UDP and mutually authenticated RadSec | Every request needs Message-Authenticator before password work, response places it first; exact peer and UDP secret; RadSec CA + exact NAS leaf pin, fixed internal `radsec` secret; password plus TOTP/recovery when required; scoped typed reply attributes and live authorization on cached retries | Actual FreeRADIUS `radclient` 3.2.10 **client** exercises loopback UDP PAP correct/wrong password and identical datagram replay with one `radius.accept` audit. It is not a FreeRADIUS server, hardware NAS or RadSec client. The separate network fixture uses actual OpenSSL TLS plus handwritten RADIUS frames for RadSec/pinning and independent authenticators/reply assertions; this review credits those as definitions, not newly executed checks. |
| RADIUS EAP-TLS 1.3, optionally 1.2 + EMS, on UDP/RadSec | Explicit CA-validated leaf→user binding, never CN/email/outer-identity adoption; current chain CRLs, expiry and binding revocation; distinct NAS outer certificate; anonymous identity/fragmentation/protected success/exporter/MPPE; live user/group/policy/trust checks on accepts and cached retries | `TlsPeer` is a genuine OpenSSL TLS implementation inside a test-controlled EAP/RADIUS `NasPeer`, not a complete supplicant. Its definition checks TLS1.3/1.2, fragments, byte-identical retries, protected success and independently exported/decrypted MPPE; missing/unregistered cert, MFA, group removal, binding/CRL revoke and TLS1.2 refusal on strict listener. No actual supplicant/OS/device/RadSec EAP combination execution is established by these records. |

Unsupported SOAP/artifact/ECP/encrypted NameID, transient or unsolicited upstream
source login, LDAP writes/SASL/password-modify/AD or POSIX emulation, RADIUS
accounting/CHAP/MS-CHAP/PEAP/TTLS/CoA and termination of established NAS sessions
remain outside the profile. This review does not add them to I04 acceptance.
Likewise a deployment-specific compatibility check is not a generic gate for
every vendor, every browser, certification or every release platform.

## Accepted execution provenance and limits

| Evidence | Pin, observation and provenance |
| --- | --- |
| Raw integration CI log | `/tmp/riauth-wave29-integration-110661000640.log`, 265770 bytes, SHA-256 `458a30895b5987c0bebe2fb7368a1a32969fa0d41ce90acbfa56f43299ccd186`, rehashed here. Accepted run `36950097067`, job `110661000640`, checkout `2d05c00deed3a6dafcd458a5f1a1a9beb17d0dd8`. Root's prior SUCCESS attestation is retained; this review did not query GitHub. |
| CI IdP XMLsec | Raw lines 581–582 command `RIAUTH_TEST_XMLSEC="$(command -v xmlsec1)" cargo test --test identity --locked saml_independent_xmlsec -- --ignored`; lines 1328–1330 named test passes 1/0 in 5.94s. Distro dependency install in workflow is not an exact XMLsec version pin. Do not borrow the native walkthrough's 1.3.12 version for CI. |
| CI source XMLsec | Raw lines 1332–1333 same shape/filter `saml_source_independent_xmlsec`; lines 1347–1349 `saml_source_tests::saml_source_independent_xmlsec_responses` passes 1/0 in 5.63s. `Upstream::sign` invokes external XMLsec; `Upstream::response` assembles XML and uses risaml for encryption. |
| CI logout XMLsec | Raw lines 1351–1352 filter `saml_logout_independent_xmlsec`; lines 1366–1368 named test passes 1/0 in 7.78s. The fixture constructs requests/responses; the late-response scenario advances a stored deadline directly. It is not natural network timeout or live multi-origin SLO evidence. |
| CI real OpenLDAP import | Raw lines 2884–2885 `scripts/test-ldap.sh`; lines 2900–2902 `openldap_plans_stable_ids_tls_login_mfa_and_fail_closed_sync` passes 1/0 in 8.05s. The script starts an isolated `slapd` and real STARTTLS. The accepted local import correction is source `67b3a189664e51f476e0e49850a4ade087853e85`, integration `b1e09657196e98545b189c77493776c0ffaa48a8`, with local OpenLDAP 2.7.1 attribution in the acceptance ledger. No OpenLDAP version is inferred for the CI daemon. |
| Raw FreeRADIUS log | `/tmp/riauth-i04-radius-test.log`, 12298 bytes, SHA-256 `f5fff90c6e750382a9ccbac1e44c817df36490c7fb0e96bb8befb3939a28bb86`, rehashed here. Lines 1/372 banner `radclient version 3.2.10, built on Sep 29 2026 at 07:49:30`; lines 374/376 named PAP peer test passes 1/0 in 2.47s. It does not echo every exchange; rc0 Access-Accept, rc1 Access-Reject and exact replay/no duplicate audit are assertion-body requirements. Accepted integration/run head `38f82fe7fce02c0bd70db4a4cd22bb15edcea7a6`, source `c51a0ef9bdb1dc03eddee5244871345c1f5a7456`. The raw log itself has no Git checkout marker; its linkage comes from the accepted ledger. |
| OpenLDAP provider accepted runs | Initial STARTTLS acceptance `1caa0939163fe3a67d8a435ce3733c1bc7fe0f18`; cumulative separate LDAPS+STARTTLS acceptance/run `fcba8a5e81037edb365c02c291b512576817e920`, source `d00437fc534faef0b1e2a0bb07a9891d1f018083`. Ledger commands use `/opt/homebrew/opt/openldap/bin/ldapsearch` and `scripts/test-ldap-provider.sh`, result 1/1. Published guide records 2.7.1 banner and exact rc/CA/scheme observations; current assertion body confirms them. A separate original raw terminal log was not located/read, so this attribution is accepted historical evidence, not a fresh reproduction. |
| GNU Lasso accepted peer runs | Authentication/signature `62ce3d83980c52f5242353952a3c4a0c7fdc6b7a`; lifetime `8d389d4ca919918e8a9ddf5f4531effc55fb3757`; Recipient `25cd9b517a932fa72251291433ec9ef6f2ac890d`. Each accepted ledger result is 1/1. Current definitions require tampered NameID/substituted metadata cert to fail with signature `(-111)`, re-signed expired Conditions to fail the Lasso lifetime validator, and re-signed wrong Recipient to fail the helper comparison, with no NameID on refusal. These historical negatives were not rerun here; no separate original raw terminal log was read. |
| Accepted native Lasso/XMLsec observation | Read the already accepted redacted `planning/evidence/d01-section12-2026-09-29/evidence.json`: 36288 bytes, SHA-256 `2cddcd2466674497a420a7213de70d96a80e0800710556dc7258eb9cd136a6b1`; matching cleanup file SHA-256 `0b4108a62971799398fa1103d2446f3fd0a3dd69c1f3ed4dbc6360e017f92fb2`. Lasso 2.9.0 compile/request/accept all rc0, persistent NameID present; XMLsec1 1.3.12 metadata verifies each rc0. This used a copied native Platform `58357fde77211e62dc51c14fb3fc216bdf143ceb` snapshot, binary SHA-256 `de06f9b46ce3e4a929d4d065681325d664b9aedb6485f649ec098a57c22a6069`, rather than the current published binary. HTTP SSO/resume were driven programmatically; the ACS form target was not contacted. No browser, source finish, upstream fetch, encryption or logout was run. The earlier failed attempt and output-overwrite refusal remain preserved, not converted into success. |

The current acceptance ledger was read only at
`planning/accepted-commits.json` under this explicit project's orchestrator
root, SHA-256 `c9751bb24286a45b2de99b3facfafbb355ffac3392c4f11a9f48d40bce5d4bc5`.
Its historical commands used jobs=2 in places; this is attribution, not an
instruction to rerun them. The current private-target/jobs1/incremental0/debug0
and disk-floor constraints remain unchanged.

Current `tests/identity/saml_source.rs`, `saml_logout.rs` and `tests/ldap.rs`
blobs equal the CI checkout's respective fixture blobs. `tests/identity/saml.rs`
differs only in the later configured-workflow fixture's known approval-ID
capture and targeted revoke; its `exercise`/ignored XMLsec caller is unchanged.
That comparison does not claim unchanged complete runtime behavior. Current
`tests/radius_peer.rs`, `tests/ldap_provider_peer.rs`, `tests/saml_sp_peer.rs`
and `scripts/lasso-saml-sp.c` equal their final accepted peer-run blobs above.
The non-ignored OpenSSL/ldap3 network/EAP definitions were inspected but no
separate named execution result for them was re-attributed in this review.

## One proposed local slice: a real Lasso SLO receiver

**Exact profile:** riAuth IdP-initiated SLO to GNU Lasso 2.9.0 SP, signed
RSA-SHA256 HTTP-Redirect LogoutRequest and correlated signed HTTP-Redirect
LogoutResponse, persistent NameID and one previously issued SessionIndex.
Reuse the accepted SP/IdP metadata, local matching keys and pinned certificates,
with an explicit Redirect SLO endpoint. These are the supported browser binding
and role; no SOAP/other protocol or remote tenant is proposed.

**Small source-derived gap:** `scripts/lasso-saml-sp.c:358` dispatches only
`request|accept`; `accept_mode` accepts SSO then destroys its login/server
without exporting the accepted identity/session. `tests/saml_sp_peer.rs:340`
sets `slo_redirect_url` and `slo_post_url` to `None`. The real receiver never
processes LogoutRequest/LogoutResponse. XMLsec's successful standalone signature
checks cannot establish the SP's logout/session-index behavior.

**Prospective reservation, not an edit made here:** narrow helper modes in
`scripts/lasso-saml-sp.c` to preserve accepted private identity/session state
and process SLO with the Lasso logout implementation; one new ignored function
in `tests/saml_sp_peer.rs`. Preserve the existing authentication and all its
negative assertions. No production writer, router, workflow, Group, API header,
CLI receipt or admission change is needed by this proposal. Before any runtime,
the owner must pin/review the exact helper/test delta and confirm the selected
Lasso build can expose the needed logout/session functions. Historical Lasso
availability is not a claim about the current machine/toolchain.

Proposed function name:
`lasso_idp_initiated_redirect_logout_revokes_only_bound_session_and_consumes_response_once`.
Its bounded checkpoints would be:

1. Accept the existing signed login using actual Lasso and save its resulting
   session/identity privately; keep a second unrelated riAuth identity live.
   The accepted state supplies NameID and SessionIndex, rather than a synthetic
   replacement. Private state/dumps are bounded and never printed in evidence.
2. Invoke normal local logout for the first identity. Assert its bearer identity
   is already refused before forwarding any request; the second remains valid.
   Obtain the issued propagation request through the normal coordinator.
3. Have Lasso process that signed Redirect request against its saved SP session,
   independently verify the pinned IdP signature and retire the matched peer
   session, then generate its correlated signed response. Require observable
   absence of the matched session in the saved Lasso session state, rather than
   treating response Success alone as proof of peer retirement.
4. Return the peer response via the supported SAML entry point. Require one
   confirmed participant, no remaining target, the unrelated identity still
   valid and the expected single confirmation audit. Exact retry cannot advance
   the coordinator or duplicate audit. Preserve the existing authenticated
   foreign-name/index/request/relay/configuration refusal definitions; do not
   mutate the ledger, force time or repair pins to manufacture a pass.

The source hooks are `src/saml/logout.rs::begin` (local revocation prerequisite),
`next` (current fingerprint, signed target, 30-second pending deadline), and
`response_in` (current peer, exact request/relay, expiry, single consumption and
confirmation audit), reached through existing assembly/Core/public SAML entry
points. Proposed execution is a focused library-peer fixture, not an assertion
of browser navigation, remote hosted SP termination or all I04 profiles.

After exact source/tool reservation and separate runtime release, the proposed
single Cargo filter is the new function above under `--locked --test
saml_sp_peer -- --exact --ignored --test-threads=1`, with an explicitly selected
owned helper, a private target under the owner's existing worktree,
`CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, dev/test debug=0, and at least 8 GiB
free. Use bounded helper IO/process deadlines and private owned cleanup. This
report runs neither helper compilation nor that target. No new service, image,
provider or daemon is a prerequisite for this local library slice.

## Residual original-row inputs

| Still-unproven exact profile/behavior | Required evidence/input; no substitute credit |
| --- | --- |
| SAML SP/source direction with a complete real IdP | An actual selected IdP product/version, its registered SP entity and POST ACS, pinned current/rollover certificates, signed response + assertion, configured stable NameID and display/email/verified mappings, and permitted fixture account. No such named peer/profile is supplied in the inspected records. The existing `urn:example:enterprise-idp` is a test helper, not a remotely configured IdP. |
| Additional advertised SAML behavior at a real receiver | POST AuthnRequest, configured attribute mappings/other NameID choices and optional encrypted assertions must be attributed to the peer/profile that actually accepts them; XMLsec sign/decrypt is credited independently. Upstream SLO and certificate rollover/withdrawal have detailed in-tree tests, not a completed external IdP lifecycle here. The proposed Redirect Lasso SLO check would fill only its exact row. |
| LDAP source LDAPS and advertised AD mappings | Actual LDAPS **import/password-source** peer evidence, separately from LDAPS provider evidence. For the specifically documented AD mapping, input is an authorized directory using binary `objectGUID`, `sAMAccountName`, the selected user/group filters (including actual `memberOf` behavior), configured CA/name validation and a least-privileged service bind plus disposable user. No AD endpoint/account or configuration is invented, and no AD/POSIX provider emulation is requested. |
| Complete RADIUS EAP-TLS/RadSec peer | A selected real supplicant/NAS implementation/version for the advertised EAP-TLS1.3 profile (and TLS1.2 only when explicitly enabled), trust/SAN/current CRLs, separate NAS transport pin where RadSec is selected, a leaf explicitly bound to a disposable local user, and authentication plus revocation/retry observations. OpenSSL TLS and FreeRADIUS PAP do not prove that complete supplicant exchange. No device/vendor/version is supplied or inferred. |
| Third-party enforcement of reply mappings/established sessions | Record what the chosen NAS actually enforces and its session lifetime. riAuth's revocation of subsequent authentication/cached accepts does not mean it can terminate an existing NAS session; CoA is outside the profile. Similarly SP lifetime/browser-delivery limitations stay explicit. |

No remote input is required for the **one proposed Lasso slice**. The other
rows state missing inputs/evidence rather than authorizing a second audit,
a tenant run or a broad matrix campaign. The original primary owner and root
must reconcile I04 against its actual acceptance after any approved slice;
this document is not a report-only closure candidate.

## Preserved boundaries and checks actually performed

The supported 60-second LDAP/RadSec certificate reload intervals and retention
of an active certificate after failed reload remain unchanged. Preserve LDAP
0.8-second password-source total deadline, bounded sync/Group/cursor behavior,
source-removal confirmations, fail-closed TLS/signature/trust/revocation,
SAML one-use browser/source/flow bindings and EAP bounds. No proposal relaxes
a signature method because the Lasso dylib cannot raise its global minimum;
RSA-SHA256 request selection and forced verification remain in place. Credentials,
receipt-secret behavior, route-specific headers and PAM fallback are unaffected.
M03/D04/S04/O03 and other completed rows remain completed; the O07 UID/GID mapping
blocker is untouched.

Actual checks for this report: live exact-row read; immutable source/guide/script
and assertion-body inspection; full accepted commit pinning; fixture blob/diff
comparison; raw CI/FreeRADIUS and accepted redacted Lasso/XMLsec evidence SHA-256
verification; source-scoped evidence/limits review; explicit-project proposal
send. Documentation/whitespace/scope checks are recorded below after execution.
No Rust, protocol, real-tenant, release, browser, deployed, HA, certification or
new interoperability execution is claimed.

Documentation check: `python3 scripts/check-docs.py` exited 0 (Markdown links
and build-directory layout). Initial `git diff --no-index --check /dev/null
REPORT` returned 1 with no whitespace diagnostics because the new file differs
from `/dev/null`; the wrapper incorrectly expected 0. This was a verification
wrapper assertion, not a product/test failure. The normal staged whitespace
and scope checks below are the final checks.

Final staged `git diff --cached --check` exited 0; the staged name list contains
only this new report, and the tracked product/source diff is empty. The commit
therefore preserves the supporting branch's existing history without a merge.
