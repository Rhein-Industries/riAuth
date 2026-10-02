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


## Approved source phase: bounded Lasso SLO receiver definition

Reservation: `wave30_I04_lasso_slo_receiver`, explicit user/root approval.
Source commit: `e028106c3d7c9a236056730ad500aad876616540`; only the two approved
helper/test files changed. Original report `1e2fa64` is retained byte-exact as
this file's first 23966 bytes / 211 lines, SHA-256
`6d9b322ed9d54a99c50a5395b1f1934d201bb95d9eb9c0d56746ea3fbd13120c`.
This section records **source/static work**, not new interoperability execution.
Compiler, helper and Cargo runtime remain held pending root's independent source
review and exact serialized release; no runtime slot was taken or released here.

### Selected local primary API and implementation

The selected installed prefix resolves to `/opt/homebrew/Cellar/lasso/2.9.0_4`.
Its `lib/pkgconfig/lasso.pc` declares `Version: 2.9.0`, SHA-256
`b33a6d16197865beda424287def683acd372026239b1725f08cbc06f3b6a452e`.
The install receipt declares stable 2.9.0, built/poured bottle, SHA-256
`36aa759a7677d6a571c99f979b89fce206f6e5fcf831c63af2f576e68d66d939`.
These files were read, not executed through `pkg-config` or a version helper.
The first attempted `include/lasso/version.h` read found no such file; version
confirmation instead used the actual package metadata and receipt above.

The already present primary archive
`/tmp/riauth-i04-saml-sp/lasso-2.9.0.tar.gz` is 4053813 bytes, SHA-256
`63816c8219df48cdefeccb1acb35e04014ca6395b5263c70aacd5470ea95c351`;
its `.tarball-version` is `2.9.0`. Installed logout/session/profile and SAML2
LogoutRequest headers are byte-identical to their archive counterparts. No
archive extraction, download, install or mutation was performed. Relevant pins:

| Local primary file | SHA-256 |
| --- | --- |
| Installed `lasso/id-ff/logout.h` | `f4f383a7278f32b3677915c077252db287a1a0771c297daed5164c43f95ecaa0` |
| Installed `lasso/id-ff/session.h` | `f6a09505f6acd5ac427e763794169439e8074b65ceec8c77601351ef20b1b64a` |
| Installed `lasso/id-ff/profile.h` | `18807d0d1a3dc8f83cd387d35080054a0ae90251f0ee5150913ac78a2087e5b5` |
| Installed SAML2 LogoutRequest header | `7b578542521d68b4f5be496adbe9d26f822dff71a924692e5c8b2c9fb66b8f66` |
| Archive `lasso/saml-2.0/logout.c` | `7f0a7d07c369563a5722d0672c27a64d17b9032ffd6678a3769cd1bc515c64a7` |
| Archive `lasso/id-ff/session.c` | `48b97bee44b9437f338c85aaffa7cef608149a244c0bc0b169abe6121d955003` |
| Archive `lasso/id-ff/profile.c` | `7a758a27705f54f968c3a2f4020aa0cc4f65e75e81eff80ddc598130620d0363` |
| Installed arm64 `lib/liblasso.3.dylib` | `0af7c7ccfda4fe8d20c6ccdf5974a006b2b59a2d50244be95ea197c2d1f72cde` |

A read-only Python Mach-O symbol-table inspection found defined external symbols
for the new logout creation/process/validate/response/destruction calls, profile
restore/signature-status calls, session assertion/name/index/emptiness calls,
identity/session dumps and `lasso_node_dump`. This establishes symbol presence,
not a compiled/linked/executed result or compatibility across other hosts.

The primary 2.9.0 logout implementation checks signature status after request
processing; its validation obtains the saved NameID's session indexes, checks
an issued index, and removes the provider assertion/index records. The session
implementation independently reports those records and emptiness. Its normal
`lasso_session_dump` returns an empty string for an empty session; the generic
`lasso_node_dump` serializes the actual emptied Session object, allowing a fresh
process to restore it instead of fabricating an empty-state marker. The official
[Lasso logout API documentation](https://lasso.entrouvert.org/documentation/api-reference/lasso-LassoLogout.html)
confirms the process/validate/build-response sequence and binding behavior;
exact semantics above were checked in the pinned 2.9.0 primary source.

### Implemented bounded source interface

| Path / committed blob | Additive definition |
| --- | --- |
| `scripts/lasso-saml-sp.c`, blob `ce0a928adb22a4d00a612cf3fb7a113125303e48`, SHA-256 `c3d3a8f7d1d2d472e8b877a89ea2807d534921d88638661e8b251e2365906563` | `accept-state` performs the existing forced signature, audience/lifetime/helper Recipient and `accept_sso` sequence, then saves actual accepted persistent identity/session. `session-state` restores and inspects the actual Lasso assertion, NameID/index count and emptiness. `logout` restores accepted state, forces signature verification, requires Redirect/current registered IdP/exact saved NameID/index, validates via Lasso, requires actual assertion/index absence and empty session, generates the signed response, then saves that response URL and real emptied object. |
| `tests/saml_sp_peer.rs`, blob `917364806b5be106d9e102bd5d30ba93455a8f0f`, SHA-256 `b8596b050db87d921f83e6f0b03ebf76bb6fecd74a99e191adfef6b6c7b32c86` | Exactly one appended ignored function, `lasso_idp_initiated_redirect_logout_revokes_only_bound_session_and_consumes_response_once`, with its local bounded runner. Normal Core setup/approval/resume/logout/coordinator entry points; store snapshot/audit reads are observations only. No direct record writes or clock/pin/ledger repair. |

For each mode the first five arguments are `SP_METADATA SP_KEY SP_CERT
IDP_METADATA IDP_ENTITY`; subsequent inputs/outputs are:

- `accept-state`: `LOGIN_DUMP RESPONSE_B64 RELAY IDENTITY_OUT SESSION_OUT`.
- `session-state`: `IDENTITY_IN SESSION_IN`.
- `logout`: `IDENTITY_IN SESSION_IN REQUEST_QUERY RESPONSE_URL_OUT SESSION_OUT`.

New mode file inputs are capped at 128 KiB and must be nonempty, regular,
owned by the effective UID, single-link, exactly mode0600, opened without symlink
following; embedded NUL or length change refuses. Outputs are capped and created
exclusively without symlink following, then checked/fixed mode0600. No overwrite
or empty dump is accepted as a proof of retirement. New modes have a 15-second
process alarm and 128-KiB file-size limit. Their stdout consists only of fixed
status/count fields; signed URLs and identity/session dumps go to private files.
The original `request` mode is unchanged; the new fixture bounds its generated
inputs and captures through its runner.

The fixture runner clears the child environment, closes stdin, uses the private
fixture directory and exclusive0600 regular captures, caps read/capture sizes,
checks a 20-second per-child deadline within a 60-second overall monotonic
budget, and includes synchronous capture reads before acceptance. It has no IO
reader threads or shared pipes; the owned-child guard kills/reaps on unexpected
results. New modes also have their own alarm. No sleep, domain clock override,
listener or browser is introduced. Panic diagnostics do not print private
protocol fields/captures; UTF-8 failure handling discards byte details. Private
files use the existing fixture TempDir cleanup; cleanup remains subject to normal
process/OS termination limits, not an atomic/crash-safe publication claim.

The new function **requires**, but has not executed, these checkpoints:

1. Actual Lasso SSO acceptance and private identity/session persistence, followed
   by a fresh helper's observed assertion present, one NameID/index, nonempty.
2. Local logout refusal of the bound bearer before peer delivery while unrelated
   identity remains live; identical pending request retry; exact issued NameID,
   single SessionIndex, Redirect target, RSA-SHA256 and original relay.
3. A substituted IdP metadata signing certificate refuses with signature error,
   creates neither response nor state output, and leaves saved accepted inputs
   unchanged. All older signature/lifetime/Recipient assertions remain intact.
4. Lasso's real validation removes the assertion/index and empties its session;
   reloading its own serialized empty object observes absence again. Exact
   request retry using retired state refuses and creates no new outputs.
5. Valid peer response passes riAuth's current pinned checks, confirms exactly
   one participant with no failures/remaining target, and records one confirmation
   audit. Consumed response retry refuses with unchanged complete snapshot and
   audit count. Bound identity stays revoked; unrelated identity stays live.

### Static checks actually run; execution still held

- All 14 pre-existing C function bodies, including `request_mode` and
  `accept_mode`, compare byte-exact to `1e2fa64`; no factoring was used. Removing
  only the three new dispatch blocks makes `main` byte-identical too. Old output,
  error strings, signature/lifetime/Recipient checks and dispatch remain intact.
- The entire original Rust file remains the exact first 19341 bytes, SHA-256
  `2a45eda55862c33d8784abbf17e8d4b376495f0a0f564b5b8cb3882141aa1aaa`.
  The suffix contains exactly one `#[test]` and one `#[ignore]`; imports and
  pre-existing function bodies were untouched.
- Initial changed-file `rustfmt --check` reported formatting differences in the
  appended function; formatting was applied, and the final `rustfmt --edition
  2024 --check --config skip_children=true tests/saml_sp_peer.rs` exited0. This
  parses/formats Rust, not type-checks or executes it.
- `git diff --check`, staged whitespace, exact two-file source scope and
  post-commit `git show --format= --check` passed; branch was clean after the
  source commit. No compiler, helper execution, Cargo, service or protocol test
  ran. C syntax/linking, Rust type compatibility, actual library state restoration
  and every runtime oracle remain pending the released checks.

### Precise future execution proposals — NOT RUN / NOT RELEASED

Root should first confirm the code/installed-library pins above, >=8GiB free,
the private owned target and fresh output directory absence. The proposed
`target/i04-lasso-slo-e028106` directory was absent during this source review;
create it under this existing worktree only after release. No existing helper or
accepted target is overwritten. One native compiler invocation is proposed:

```sh
/usr/bin/cc -O2 -o "$PWD/target/i04-lasso-slo-e028106/lasso-saml-sp" \
  scripts/lasso-saml-sp.c \
  $(PKG_CONFIG_PATH=/opt/homebrew/Cellar/lasso/2.9.0_4/lib/pkgconfig \
    pkg-config --cflags --libs lasso gobject-2.0)
```

The flags resolve the explicitly selected local Lasso package; `pkg-config` is a
metadata read inside that proposed compile command, not a second compilation.
One ignored filter, after successful compilation and root's exact release:

```sh
env CARGO_TARGET_DIR="$PWD/target" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  RIAUTH_TEST_LASSO_SP="$PWD/target/i04-lasso-slo-e028106/lasso-saml-sp" \
  cargo test --locked --test saml_sp_peer \
  lasso_idp_initiated_redirect_logout_revokes_only_bound_session_and_consumes_response_once \
  -- --exact --ignored --test-threads=1
```

No automatic compile/test retry, alternative peer, trust relaxation, pin repair,
clock change or broad suite is proposed. A09 currently owns the serialized slot;
root alone assigns the later execution. Record actual rc/elapsed/refusal/cleanup
and any failure separately after exit. This source phase does not establish
new Lasso interoperability, browser/remote tenant/profile completion, a release
artifact, Windows/Linux execution or whole I04 closure. Original primary
worktree, D04 DONE, O07 mapping blocker and all protected product contracts are
unchanged. The only documentation edit is this append-only reserved report.

Source-phase documentation check: `python3 scripts/check-docs.py` exited0.
Append-only original-prefix comparison, report-only diff name list and
`git diff --check` passed before the separate report commit. Final staged and
post-commit whitespace/scope checks also passed; no execution directory was
created.


## Approved preparation: exact published-source alignment, runtime still held

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original I04
`dae9c528-9e32-462c-947f-661a571f136b`; same supporting worktree/branch.
The current explicit approval permits history-preserving alignment and a
read-only disk/target/fingerprint inventory, **not** native compilation,
`pkg-config`, library/helper execution, Cargo or service runtime. Remote A09
and the prospective whole24 SCIM work retain root's serialized scheduling.
No execution slot was taken or released by this preparation.

### Exact merge, conflict and source-equivalence proof

Local merge commit: `abd01f6a51307011a7c41c711b3aeaec09341614`.
Parents, in order:
`1991672a3adf1ff9b3af5843013c009894ad3290` and the explicitly approved published
pin `ae8937800254a1ad4296ea257de1eccc4780e45b`.
Merge tree: `4989a8eb09e0ea8cccf6168d0deab32293d9481e`.
The pre-merge branch was clean; no reset, rebase, stale production-file
replacement, source-worker read or other-worker contact occurred.

There was exactly **one add/add conflict**, in this reserved report. The
published side was exactly the original `1e2fa64` report, 23966 bytes, SHA-256
`6d9b322ed9d54a99c50a5395b1f1934d201bb95d9eb9c0d56746ea3fbd13120c`.
The supporting side was the complete `1991672` report, 35724 bytes, SHA-256
`6cf0b7997eb59125804e82b0dbd7c986d289e01c11a0e16b7778c35ae6a7dcf9`;
its first 23966 bytes equal the published side. Resolution retained that
complete supporting text byte-exact. Thus all published text and the entire
original/source-phase report remain present; the published side contained no
additional root text to reconcile. This preparation section is appended only.

Before alignment, the 17 accepted production-path differences were:
`src/api.rs`, `src/api/observability.rs`, `src/api/probes.rs`,
`src/assembly/cloud_operations.rs`, `src/background.rs`, `src/config.rs`,
`src/kms.rs`, `src/kms_essentials.rs`, `src/offboarding.rs`, `src/operations.rs`,
`src/operations/key_diagnostics.rs`, `src/operations/storage_diagnostics.rs`,
`src/operations/storage_diagnostics/tests.rs`, `src/provisioning.rs`,
`src/reconciliation.rs`, `src/state.rs`, and `src/telemetry.rs`.
The merge retained the published accepted versions of every one. No new
implementation or new execution claim is attributed to these imported paths
or reports.

Index/tree comparison checked 347 tracked production/config paths, including
all `src/`, `crates/`, root Cargo manifests/lock, toolchain and any tracked
`.cargo`/build configuration: **all modes and object IDs equal the published
pin**. Comparing the entire merged tree with that pin leaves exactly three
paths: this report (174 additive source-report lines before this new append),
`scripts/lasso-saml-sp.c` (267 added lines), and `tests/saml_sp_peer.rs`
(450 added lines). No removed lines occur in those differences.

| Protected subtree/file | Merged object, equal to published pin unless explicitly reserved |
| --- | --- |
| `src/` | tree `74f4d48544a7dc6728a00bd44a908d26c39c2b2d` |
| `crates/` | tree `30affb89e98846c09b7e641ce303c2252fa118b0` |
| `Cargo.toml` | blob `5660d4bb922fcdc5bfe05d7502585980f4720d06`; SHA-256 `58e5ef824ed96290179c9f76fea208dc37173caeee21b6ce8d37f8a6dd1abcb8` |
| `Cargo.lock` | blob `f1b819d47d204d73617b095513f0c6ab6eb8aa4e`; SHA-256 `b5c9d11c001244b8017303ce8c20516e02946483845eee40720b0910758d4426` |
| `rust-toolchain.toml` | blob `c3f67b6771b777215340531caf051bc25cef066c`; SHA-256 `887f9be066a15585a2c583578e84b0fcb541126d81546276bad3d2ff00d61167`; channel1.98.1 |
| `src/saml/` | tree `72a64cc1ed0b5ae1d74bd1a8858b682b1efc4709` |
| `src/workflow/` | tree `04694671da97b9b9f6754a433b247316adec108e` |
| `src/store/maintenance.rs` | blob `6d6999eaba0eacecfc5587dcc1e94dd8a69e5f3e` |
| `src/workflow/extension_gate.rs` | blob `a6d9876e5b44b65932a9f779f5e272a2eb6e5fd4` |
| `tests/common/mod.rs` | blob `9a40464e4ca311c7c37fe7cee7d5616aeefb73c7` |
| `.github/workflows/ci.yml` | blob `7c724fd7f4dc210a2268f5a702bdc09b1a10d76b` |
| Reserved C helper | still blob `ce0a928adb22a4d00a612cf3fb7a113125303e48`, SHA-256 `c3d3a8f7d1d2d472e8b877a89ea2807d534921d88638661e8b251e2365906563`, 23856 bytes; equals `e028106` |
| Reserved Rust fixture | still blob `917364806b5be106d9e102bd5d30ba93455a8f0f`, SHA-256 `b8596b050db87d921f83e6f0b03ebf76bb6fecd74a99e191adfef6b6c7b32c86`, 37798 bytes; equals `e028106` |

The 14 original C function bodies were again compared byte-exact with the
published pin. Removing just the three reserved new dispatch blocks makes the
old `main` byte-exact too. The Rust file's original 19341-byte prefix is also
byte-exact with that pin and retains the earlier SHA-256. One initial static
proof command incorrectly counted only literal `#[ignore]` and exited1 because
the new test uses `#[ignore = "requires ..."]`; a corrected annotation-prefix
check passed. No C/Rust source changed to make that check pass.

The live original RiWork row was read again, remains `in_progress` on primary
`a2dff16a-c4b0-47fc-96de-ac85b1fb6d9e`, and retains the same requested outcome
and real-peer gate quoted above. Its old scheduling paragraph does not supersede
this explicit supporting preparation. No row/status mutation occurred. Root
must import only the reserved reviewed source/report deltas if desired; this
supporting merge is alignment evidence, **not a request to import its old branch
history into main**.

### Read-only private target, fingerprints and capacity estimate

Inventory timestamp: **2026-10-02T14:39:11.866275+00:00**.
Filesystem free space was **11949944832 bytes / 11.1293 GiB**; total filesystem
size1995218165760 bytes. These are host observations, not reserved capacity.
The existing `$PWD/target` is a real directory, UID501/effective-user-owned,
mode0755, resolves inside this exact supporting worktree, and is not a symlink.
Its descendants were inventoried by metadata only, without following symlinks
or reading private protocol/session/evidence contents. No symlink was observed;
12722 regular files and 2032 directories were counted.

| Existing private cache grouping | Logical bytes | Allocated bytes (unique inode accounting) |
| --- | --- | --- |
| Entire target | 8222222827 | 8260694016 |
| `wave29-workflow-retry/debug` | 3709693292 | 3722063872 |
| `wave27/debug` | 2162741351 | 2173845504 |
| Current `debug/deps` | 1793059924 | 1795297280 |
| Current debug root files | 328211472 | 328216576 |
| Current `debug/build` | 70342757 | 72212480 |
| Current `debug/.fingerprint` | 560939 | 7163904 |
| `wave28-m03-operations-port/debug` | 156928531 | 161132544 |

Current `debug/deps` contains 335 rlibs (874076328 bytes), 335 rmeta files
(307255308 bytes), 27 dylibs (80864304 bytes), and no `.o` files. Selected
large artifact sizes are an existing riAuth rlib407106624 bytes, server
259884832 bytes and gauge test200307328 bytes. These are size observations,
not proof that Cargo can reuse them for the aligned source or requested filter.

The current debug cache has441 fingerprint directories. Its one cached riAuth
library fingerprint records profile12672335563272108896 and features
`default, essentials, fuzzing, platform, test-support`. The proposed default
Platform filter omits the two test feature flags, so cache reuse is uncertain.
No `saml_sp_peer` fingerprint exists in the current debug cache or the two older
private `wave27`/`wave29-workflow-retry` caches. No binary/fingerprint was
executed. The `.rustc_info.json` was read only (1965 bytes, SHA-256
`27df402be20083ab5b4835c05762e2b77beed67288686dd8fda9193b24cdd7c4`);
it records rustc1.98.1 / commit48a229ceaefd4985c50990b14116b6d856af0985,
aarch64-apple-darwin, LLVM22.1.8. This is historical cache metadata, not a fresh
compiler-version execution or cache-validity claim.

**Planning estimate, not a measured peak:** budget **2–4 GiB additional disk**
for the focused filter, including a differently featured riAuth rlib/server/test,
possible dependency cache misses and compiler/linker transient files. The
existing active cache grouping is about2.06GiB; duplicating much of it plus
transients explains the upper4GiB allowance. A cache-reusing run could be smaller.
The tiny dynamically linked C helper and capped diagnostic captures are a much
smaller allowance; they do not make the Cargo estimate a guarantee.

At this observation there is only **2.1293 GiB headroom to the required9GiB
stop point**, and3.1293GiB to the8GiB hard floor. Current free space therefore
does **not** establish safe headroom for the upper estimate. Root should remeasure
after the earlier serialized jobs, ideally with at least13GiB free for the4GiB
planning allowance above the9GiB stop point. Later execution must monitor free
disk, stop at9GiB, retain the8GiB floor, and preserve all existing cache/evidence;
no automatic cleanup/deletion or retry is proposed. No target directory was
created. `target/i04-lasso-slo-e028106` remains absent.

The selected Lasso `.pc`, receipt and arm64 dylib were rehashed by file reads;
all three still match the earlier2.9.0_4 pins, including dylib SHA-256
`0af7c7ccfda4fe8d20c6ccdf5974a006b2b59a2d50244be95ea197c2d1f72cde`.
No package metadata tool, compiler, version binary or library was executed.

### Revised finite compiler proposal — NOT RUN / NOT RELEASED

After root's exact release, reverify the aligned production/helper/test/library
pins and current free disk before creating the one absent output directory.
The following is one **proposed** Python-bounded compiler invocation: one
5-second package-metadata read and one60-second native compiler, with at most
10 seconds of captured process-group cleanup on a failed/deadline/disk outcome.
It uses fixed file names under a freshly exclusive mode0700 owned directory;
capture files are exclusive0600. Compiler output is not printed. Parent checks
cap captures at128KiB; a16MiB child file-size limit also bounds between-check
overshoot and artifact writes. No helper is executed by this command.

```sh
python3 - <<'PY'
import os, resource, shlex, shutil, signal, stat, subprocess, time
from pathlib import Path

root = Path.cwd()
target = root / 'target'
s = target.lstat()
assert stat.S_ISDIR(s.st_mode) and not stat.S_ISLNK(s.st_mode)
assert s.st_uid == os.geteuid() and target.resolve() == target
stop = 9 * 2**30
assert shutil.disk_usage(root).free > stop
out = target / 'i04-lasso-slo-e028106'
out.mkdir(mode=0o700)  # exclusive; never reuse another attempt's output
env = os.environ.copy()
env['PKG_CONFIG_PATH'] = '/opt/homebrew/Cellar/lasso/2.9.0_4/lib/pkgconfig'

def child_file_cap():
    resource.setrlimit(resource.RLIMIT_FSIZE, (16 * 2**20, 16 * 2**20))

def once(argv, seconds, label):
    flags = os.O_CREAT | os.O_EXCL | os.O_WRONLY | os.O_NOFOLLOW
    with os.fdopen(os.open(out / (label + '.stdout'), flags, 0o600), 'wb') as so:
        with os.fdopen(os.open(out / (label + '.stderr'), flags, 0o600), 'wb') as se:
            p = subprocess.Popen(argv, stdin=subprocess.DEVNULL, stdout=so,
                                 stderr=se, env=env, start_new_session=True,
                                 preexec_fn=child_file_cap)
            deadline = time.monotonic() + seconds
            try:
                while True:
                    if shutil.disk_usage(root).free <= stop:
                        raise SystemExit('compiler preparation: disk stop')
                    if max(os.fstat(so.fileno()).st_size,
                           os.fstat(se.fileno()).st_size) > 128 * 1024:
                        raise SystemExit('compiler preparation: capture cap')
                    left = deadline - time.monotonic()
                    if left <= 0:
                        raise SystemExit('compiler preparation: deadline')
                    try:
                        rc = p.wait(timeout=min(left, 0.1))
                        break
                    except subprocess.TimeoutExpired:
                        pass
                assert time.monotonic() <= deadline
                assert shutil.disk_usage(root).free > stop
                assert max(os.fstat(so.fileno()).st_size,
                           os.fstat(se.fileno()).st_size) <= 128 * 1024
                print(label + ': rc=' + str(rc))
                return rc
            finally:
                if p.poll() is None:
                    os.killpg(p.pid, signal.SIGTERM)
                    try:
                        p.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        os.killpg(p.pid, signal.SIGKILL)
                        p.wait(timeout=5)

assert once(['pkg-config', '--cflags', '--libs', 'lasso', 'gobject-2.0'],
            5, 'pkg-config') == 0
flags = (out / 'pkg-config.stdout').read_bytes()
assert len(flags) <= 64 * 1024 and b'\0' not in flags
assert once(['/usr/bin/cc', '-O2', '-o', str(out / 'lasso-saml-sp'),
             'scripts/lasso-saml-sp.c'] + shlex.split(flags.decode('utf-8')),
            60, 'cc') == 0
PY
```

ONE ignored-filter proposal, after successful compiler outcome and exact root
release, still uses **only the existing own `$PWD/target`**:

```sh
env CARGO_TARGET_DIR="$PWD/target" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  RIAUTH_TEST_LASSO_SP="$PWD/target/i04-lasso-slo-e028106/lasso-saml-sp" \
  cargo test --locked --test saml_sp_peer \
  lasso_idp_initiated_redirect_logout_revokes_only_bound_session_and_consumes_response_once \
  -- --exact --ignored --test-threads=1
```

Propose a root-owned1800-second outer Cargo/build/test budget, continuous9GiB
disk stop monitoring, captured-process-group termination/reaping, and bounded
private diagnostics. This is a scheduling proposal, not a runner started here;
the test definition's existing60-second operation/20-second child/15-second C
limits remain unchanged. No automatic rerun, cache deletion, feature change,
artifact substitution, pin repair, forced clock or broad test is proposed.
C syntax/linking, Rust type compatibility and all new peer/security oracles
remain unexecuted. Production byte equality and cache inventory establish no
new interoperability result or whole-I04 completion. D04/O06/I10/R05 DONE,
O07's host-mapping blocker and the unchanged primary I04 ownership are preserved.

Preparation checks actually run: `python3 scripts/check-docs.py` exited0;
`git diff --check` and merge-commit whitespace checks passed. The proposed
inline Python command was parsed with `ast.parse` only, not executed. Exact
`1991672`/`1e2fa64` prefix checks, reserved C/Rust SHA checks, production/config
working-tree equality and absent output-directory checks passed. A fresh disk
read at2026-10-02T14:46:51.857858+00:00 reported11953029120 bytes /11.1321GiB
free. No compiler/helper/Cargo execution, listener, new target directory or
runtime result occurred. Final report-only staged/post-commit scope and whitespace
checks are part of the handoff; all checks here are static/source preparation.


## Approved own-target inventory: no eligible executable cleanup candidate

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, original I04 support in the same
existing worktree. This follow-up authorizes metadata/hash/live-use inventory
and one private ignored manifest only. It authorizes **no deletion** and no
compiler/helper/library/Cargo execution. Source HEAD before this append is
`020a32429863d2ca656b794d634efe3b8d91e70f`; accepted evidence references were
read from fixed published `9b8956f7b2a9961b14e313fa57c0f5214a136772`, not another
worker's files. Previous source, preparation, failed observations and report
prefixes remain intact. No merge/alignment or source change occurred here.

**Recommendation: empty cleanup allowlist, 0 bytes proposed relief.** There is
no defensible never-executed **and** never-cited integration-test executable in
this inventory. All six integration-test targets present in the private target
are cited in accepted actual execution reports for this same worktree/cache.
The reports do not attest the current executable SHA at each historical run;
that limitation is a reason to preserve the stored artifact, not evidence that
it was never executed. An old mtime, different feature set, unused descriptor
or absent current process does not establish historical nonexecution.

The six integration executables together occupy only **920947568 logical bytes /
920961024 allocated bytes / 0.8577 GiB**. Even including all six contrary to the
evidence exclusions would not provide the requested2GiB relief. No broader
native/server/library/cache deletion is proposed to bridge the difference.

### Exact excluded integration executable records

All six paths below were checked individually: regular, nonsymlink, no symlink
in their path to this worktree, effective-user-owned UID501, nlink1, mode0755.
Each resolved parent equals its literal parent inside this worktree's private
target. Hash reads checked dev/inode/size/mtime again afterward; no file change
was observed across hashing. No binary was executed. Literal paths:

- E1: `/Users/dominik/orca/projects/riAuth-public-preview-local-extension-isolation-wave27/target/wave29-workflow-retry/debug/deps/workflow_approval_api-be7eb35d17248f42`
- E2: `/Users/dominik/orca/projects/riAuth-public-preview-local-extension-isolation-wave27/target/wave29-workflow-retry/debug/deps/workflow_approval-f4610ae194ee42cc`
- E3: `/Users/dominik/orca/projects/riAuth-public-preview-local-extension-isolation-wave27/target/wave29-workflow-retry/debug/deps/identity-0a3dab03190e6e69`
- E4: `/Users/dominik/orca/projects/riAuth-public-preview-local-extension-isolation-wave27/target/wave29-workflow-retry/debug/deps/identity_boundary-daba31a221c5f052`
- E5: `/Users/dominik/orca/projects/riAuth-public-preview-local-extension-isolation-wave27/target/wave29-workflow-retry/debug/deps/m03_parity_workflow-797256b406b7ca3e`
- E6: `/Users/dominik/orca/projects/riAuth-public-preview-local-extension-isolation-wave27/target/debug/deps/o06_resolved_deactivation_gauge-b18eae65435f6fd2`

E1–E5 resolved parent:
`/Users/dominik/orca/projects/riAuth-public-preview-local-extension-isolation-wave27/target/wave29-workflow-retry/debug/deps`.
E6 resolved parent:
`/Users/dominik/orca/projects/riAuth-public-preview-local-extension-isolation-wave27/target/debug/deps`.

| Record | Logical / allocated bytes | mtime UTC | Executable SHA-256 | Decision/reason |
| --- | --- | --- | --- | --- |
| E1 | 200697168 / 200699904 | 2026-10-02T01:39:48.699358+00:00 | `de8f1f42fa24f9b81f84beab0ced01cd13637c6d4290d30e3ac0840df9a9d31f` | EXCLUDE: accepted executed/cited API retry target |
| E2 | 200427504 / 200429568 | 2026-10-02T01:35:57.053184+00:00 | `1babd7b6e7b21c4d2a2766b070a0a288f06f19d6db9dc0d71c5d42f17bfb30d5` | EXCLUDE: accepted executed/cited approval floor target |
| E3 | 250164960 / 250167296 | 2026-10-02T02:06:28.480019+00:00 | `0b7efe00c8df2fb70eb295d8855d03dcd4f6ad458338c77885f484b3c29c2b5b` | EXCLUDE: accepted executed/cited four operations fixtures |
| E4 | 67493040 / 67493888 | 2026-10-02T02:55:29.479895+00:00 | `f25dc79a33acb5d7c1f9997559e6d57fa5efa18e393cad3ec9e3ea1a8eb69789` | EXCLUDE: accepted executed/cited receipt baseline/final fixture |
| E5 | 1857568 / 1859584 | 2026-10-02T01:31:57.794605+00:00 | `084b3c4e4c9fef9e0c881ed1f4c04626e085379cbc6541cbae97d594ddb9dd8d` | EXCLUDE: accepted executed/cited six client mock cases |
| E6 | 200307328 / 200310784 | 2026-10-02T10:43:28.156363+00:00 | `fb93dc46d8c8945288291ae49da8ea003cd10cec56820e262661ea871ceb851a` | EXCLUDE: accepted executed/cited gauge failure/corrected result; also current cache |

Fingerprint JSON was read as build metadata only. All six record test profile
`11094973624911973823` and rustc fingerprint`17329007180185699724`; neither
number is an attestation of a current source commit. Features below are the
literal feature lists decoded from their Cargo JSON strings. Each fingerprint
path is relative to the exact own worktree root specified above; the private
manifest also records its full absolute literal path.

| Record | Fingerprint metadata path | Features | Fingerprint-file SHA-256 |
| --- | --- | --- | --- |
| E1 | `target/wave29-workflow-retry/debug/.fingerprint/riauth-be7eb35d17248f42/test-integration-test-workflow_approval_api.json` | `default, essentials, platform` | `6dd33d11ed5ff8b7f54e37422fe0ba67e96390babae0cee48d360d4a47fd6a25` |
| E2 | `target/wave29-workflow-retry/debug/.fingerprint/riauth-f4610ae194ee42cc/test-integration-test-workflow_approval.json` | `default, essentials, platform` | `5b0835d10ba0ec9b979a3eca58f78afb66838c5abc7304da07620e500538362b` |
| E3 | `target/wave29-workflow-retry/debug/.fingerprint/riauth-0a3dab03190e6e69/test-integration-test-identity.json` | `default, essentials, fuzzing, platform, test-support` | `934d5e4f272cac7babf26653b4a089c02c56456ba8a58585a25764e3d562eea4` |
| E4 | `target/wave29-workflow-retry/debug/.fingerprint/riauth-daba31a221c5f052/test-integration-test-identity_boundary.json` | `default, essentials, fuzzing, platform, test-support` | `11925a04a7cd4f3795a79a84843072dc4238848a7552f1f52307c7d4875d03b7` |
| E5 | `target/wave29-workflow-retry/debug/.fingerprint/riauthctl-797256b406b7ca3e/test-integration-test-m03_parity_workflow.json` | `default` | `c6bbae248e4ca99001241ada75d6e34dde83d925ed7f5a238ef13a378b9f6c70` |
| E6 | `target/debug/.fingerprint/riauth-b18eae65435f6fd2/test-integration-test-o06_resolved_deactivation_gauge.json` | `default, essentials, fuzzing, platform, test-support` | `4b4a727d6aa4e7c890d995806d2d70eef89ff6fb9d29470db5868048f51c83b4` |

Exact accepted source-derived disqualifiers at published9b8956f:

- E1/E2/E5: [retry implementation evidence](local-wave29-workflow-retry-parity-implementation-report.md), lines140–152, explicitly selects this worktree's `target/wave29-workflow-retry` and records actual API/floor/client runs. E5's6/6 result excludes it despite its small size.
- E3: [four operations fixture evidence](local-wave29-identity-operations-ci-report.md), lines54–63, records this same private target and four exact functions executed once each.
- E4: [receipt fixture evidence](local-wave29-identity-boundary-receipt-ci-report.md), lines63–76, records the same target, actual failed baseline and passing final run across both formats. Failed evidence is retained too.
- E6: [gauge evidence](local-wave30-o06-resolved-deactivation-gauge.md), lines88–103, records this own target, first concrete failure, corrected pass and retained result logs.

No private result log, protocol/session data, archive or evidence content was
read for this follow-up. Public committed reports were sufficient to disqualify
every integration-test artifact; no historical nonexecution was inferred from
absence of a filename in a search result.

### Live metadata checks and exact private manifest

At2026-10-02T15:03:09.814078+00:00, `/usr/sbin/lsof -nP -Fpcfn --` with the six
**literal absolute paths** returned exit1, stdout0 bytes, stderr0 bytes: no
matching open file was observed. `/bin/ps -axo pid=,ppid=,comm=` returned exit0,
1553 process-name records, and zero literal-path or exact-basename matches.
Process arguments/environment were not requested or read. The manifest stores
only matching process records (none), not the other workers' process inventory.
These are point-in-time checks; they neither promise future nonuse nor prove
that any artifact was never executed historically.

The ONE authorized manifest was created exclusively, without symlink following,
inside the existing own target; it is regular, nlink1, owned UID501, mode0600
and `git check-ignore` confirms it is ignored:
`/Users/dominik/orca/projects/riAuth-public-preview-local-extension-isolation-wave27/target/i04-slo-executable-inventory-020a324.json`.
12849 bytes, SHA-256
`5baaad8e85b1be75126f5d5b015f0b59e7f994b46fe269cb4d36314f8238e6ae`.
It contains the full literal paths, resolved parents, sizes/mtime/mode/nlink/
owner, SHA/features/
fingerprints, live checks, reasons, and **`cleanup_allowlist: []`** /
**`proposed_relief_bytes: 0`**. No new target directory was created, and no
existing manifest/output was overwritten. The dev/inode/size/mtime stability
comparison during hashing was separately checked by the inventory command;
device/inode identifiers themselves are not fields in the manifest.

The broader executable metadata walk observed15 `deps` executables: these six
integration tests plus native server/maintenance/client/unit-test artifacts,
all excluded. It also identified9 non-`deps` native/probe copies, excluded;
274 dependency build-script/tool executable records, excluded; and95 executable
noncandidate dependency/nonbinary records, excluded. The broad walk read only
regular metadata and, for executable classification in `deps`, four Mach-O magic
bytes; it did not hash native evidence copies or read private protocol contents.
Current source/filter/default-feature cache, accepted helpers/native server/
maintenance/client/test evidence binaries, **every** log/evidence/archive/rlib/
rmeta/dylib/dependency/fingerprint/build-script artifact, and other-worker
targets remain excluded. No existing target artifact was removed, pruned or
rewritten, and no inventoried executable or protocol helper was executed.

### Capacity remains insufficiently reserved; runtime remains held

Inventory manifest free space at2026-10-02T15:03:09.814226+00:00:
**11699761152 bytes /10.896 GiB**. Fresh metadata-only measurement at
2026-10-02T15:04:40.666486+00:00: **11694329856 bytes /10.8912 GiB**.
Host free capacity can change while other serialized work runs; these snapshots
are not an after-SCIM completion measurement or a reservation. This branch's
only new target write is the tiny exclusive manifest above.

The last observation leaves about1.8912GiB above the9GiB stop point, less than
the4GiB upper planning allowance. Root still needs a fresh after-SCIM capacity
measurement, ideally13GiB free for that allowance. There is no eligible own
executable cleanup proposal; no deletion permission was given or consumed.
The earlier compiler/filter proposals remain held and were not executed.
Root alone may assign exact later cleanup/compile/runtime, integration and
status changes. Original primary ownership and the D04/O06/I10/R05 completed
states, O07 blocker and protected contracts are unchanged.

Inventory static checks: `python3 scripts/check-docs.py` exited0 and
`git diff --check` passed. The complete original1e2/source199/preparation020
report prefixes remain byte-exact; both reserved C/Rust files still equal
`e028106`. Manifest hash/mode/nlink/owner and empty allowlist were checked after
its exclusive write; the proposed compiler output directory remains absent.
Fresh free capacity at2026-10-02T15:08:03.145410+00:00 was11690553344 bytes /
10.8877GiB. Staged/post-commit checks verify this one append-only report path;
no source/runtime/deletion/cleanup claim is added by those checks.


## Released compiler step: actual local compile/link only, Cargo still held

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, I04 supporting worktree
`e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`. Root explicitly released **one compiler
step only**, following its separate exhausted cleanup allowlist in another
private cache and the SCIM owner's exit/release. This worktree did not perform,
observe or claim that other-cache deletion; the preceding empty own allowlist
remains correct. Source branch pin on entry:
`b02e9dea2c819b3d8ba9a0abc2603e653b3f8d60`.

The **EXCLUSIVE PREPARATION SLOT RELEASED** result was sent to the explicit
project orchestrator immediately after exit and before writing this separate
report append. No Cargo slot was taken or released. The ignored filter remains
held until root assigns its separate exact runtime; sufficient observed disk
space is not permission to start it.

### Exact preflight and executed compiler command

Preflight at2026-10-02T15:27:39.947070+00:00 measured14547554304 bytes /
13.5485GiB free. Branch was clean; no alignment, source correction, reset,
other-worker read/contact or deletion was performed. Existing own target was
regular-directory/nonsymlink/UID501, resolved to this worktree. The one proposed
output directory was absent before setup.

- C helper matched `e028106` byte-exact: 23856 bytes, blob
  `ce0a928adb22a4d00a612cf3fb7a113125303e48`, SHA-256
  `c3d3a8f7d1d2d472e8b877a89ea2807d534921d88638661e8b251e2365906563`.
- Rust fixture matched `e028106` byte-exact: 37798 bytes, blob
  `917364806b5be106d9e102bd5d30ba93455a8f0f`, SHA-256
  `b8596b050db87d921f83e6f0b03ebf76bb6fecd74a99e191adfef6b6c7b32c86`.
- All production `src/`, `crates/`, Cargo/toolchain/build-config paths still
  equal reviewed `ae8937800254a1ad4296ea257de1eccc4780e45b`; no new source delta.
- Selected Lasso2.9.0_4 `.pc`, receipt, logout/session/profile headers and dylib
  were rehashed by file reads and matched their earlier pins. In particular
  `.pc` SHA-256 remains
  `b33a6d16197865beda424287def683acd372026239b1725f08cbc06f3b6a452e`
  and selected dylib remains
  `0af7c7ccfda4fe8d20c6ccdf5974a006b2b59a2d50244be95ea197c2d1f72cde`.

The inline compiler command was extracted **byte-exact** from immutable
`020a32429863d2ca656b794d634efe3b8d91e70f` report section
“Revised finite compiler proposal”; Python body SHA-256
`29ba586850b6d9f7004feb3c2031d622888080f6c9e77e655a52354cadb96c2f`.
An outer Python caller recorded elapsed time/endpoints and inspected artifact
metadata after the command; it did not change the reviewed wrapper. Exactly
one `pkg-config --cflags --libs lasso gobject-2.0` metadata child ran (5-second
limit), followed by exactly one `/usr/bin/cc -O2` child (60-second limit) with
those flags and the reviewed C file. There was no compiler correction/retry,
version probe, helper execution, protocol/library invocation or Cargo.

The reviewed wrapper exclusively created
`target/i04-lasso-slo-e028106`, UID501/mode0700, with exclusive0600 captures,
16MiB child file-size bound, 128KiB parent capture checks, owned child process
groups, joined-IO/deadline postchecks and9GiB disk stop. No other target or
execution directory was created; prior logs/manifests/evidence were retained.

### Actual exits, capacity, output and dependencies

| Observation | Actual result |
| --- | --- |
| Metadata child | Exit0; private stdout917 bytes, stderr0 bytes |
| Native compiler | Exit0; stdout0 bytes, stderr0 bytes; no compiler warning/error observed |
| Inline wrapper | Exit0; elapsed0.356856 seconds; one invocation only |
| Immediate wrapper disk endpoints | Before14560317440 bytes; after14560235520 bytes at2026-10-02T15:30:29.318221+00:00; both about13.56GiB |
| Fresh release-time capacity | 14557188096 bytes /13.5574GiB; measured again before orchestrator handoff |
| Deadline/capture/disk guards | Passed; no abort or threshold refusal;9GiB stop/8GiB floor retained. No independently recorded continuous peak/minimum is claimed. |
| Owned-child cleanup | Both approved children joined normally through the reviewed wait/finally. No TERM/KILL cleanup branch was needed; outputs retained. No separate live descendant-group audit was performed or claimed. |

Artifact literal path:
`/Users/dominik/orca/projects/riAuth-public-preview-local-extension-isolation-wave27/target/i04-lasso-slo-e028106/lasso-saml-sp`.
It is regular, nonsymlink, nlink1, UID501, mode0755 under the private0700
directory, **39472 bytes**, SHA-256
`18f148c0a3119d4a1268597c829c680a743c4978ccc96924dac337368d3a891c`.
Reading its Mach-O header/load-command bytes established a64-bit arm64
executable (CPU16777228, filetype2,21 load commands). It was **not executed**.
The direct dependency load names and current file-resolution observations are:

| Mach-O direct dependency | Current resolved path, observed by file metadata only |
| --- | --- |
| `/opt/homebrew/opt/lasso/lib/liblasso.3.dylib` | `/opt/homebrew/Cellar/lasso/2.9.0_4/lib/liblasso.3.dylib` |
| `/opt/homebrew/opt/glib/lib/libgobject-2.0.0.dylib` | `/opt/homebrew/Cellar/glib/2.90.0/lib/libgobject-2.0.0.dylib` |
| `/opt/homebrew/opt/glib/lib/libglib-2.0.0.dylib` | `/opt/homebrew/Cellar/glib/2.90.0/lib/libglib-2.0.0.dylib` |
| `/opt/homebrew/opt/gettext/lib/libintl.8.dylib` | `/opt/homebrew/Cellar/gettext/1.0/lib/libintl.8.dylib` |
| `/usr/lib/libSystem.B.dylib` | `/usr/lib/libSystem.B.dylib` load name; no on-disk system-library availability or dyld loading claim |

Dependency names/current symlink resolution and successful link are not a runtime
loader/ABI/session/logout interoperability result. Transitive dependencies were
not independently enumerated or exercised. Root must reverify selected dynamic
pins before any later helper execution.

All four retained captures are regular, nonsymlink, nlink1, UID501/mode0600 in
the above owned directory; no raw flags/captures or private protocol values were
printed or added to this report:

| Capture | Bytes | SHA-256 |
| --- | --- | --- |
| `pkg-config.stdout` | 917 | `8d8f51e95d756bdacbfaf1c3fb287970cb580ef1061c9f0c4207761950451a92` |
| `pkg-config.stderr` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `cc.stdout` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `cc.stderr` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

This successful step establishes the selected native C helper's compile/link
compatibility only. Rust type checking, actual accepted identity/session dump
restore, Lasso signature/index retirement observations, local-before-peer
revocation, unrelated-user preservation and response consumption/audit/retry
oracles remain **unexecuted**. The one ignored Cargo filter documented above is
still the exact pending proposal, with private target/jobs1/incremental0/dev+
testdebug0, fresh13GiB planning capacity check,9GiB stop and8GiB floor. No
automatic test, alternative feature/peer, pin repair or broader campaign is
authorized by this compiler result. No browser, tenant/profile completion,
Windows/Linux or whole-I04 closure is implied; original primary ownership,
completed rows and O07 blocker remain unchanged.

Post-compiler report checks actually run: documentation checker exited0;
`git diff --check` passed; the whole prior `b02e9de` report remains the exact
62736-byte prefix, SHA-256
`badb7b5e1070c8130b2b40906b5c9b0b82c43750abeecb42b8adc5c56c290d90`.
Protected C/Rust and reviewed production equality were rechecked; artifact and
four capture hashes/permissions/nlink/owner and exact output-file list matched.
Fresh disk at2026-10-02T15:34:09.227827+00:00 measured14562205696 bytes /
13.5621GiB. Staged/post-commit checks verify one append-only report path.
These additional checks did not execute the helper/library or Cargo.


## Released one ignored filter: actual failure at the pinned-signature oracle

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; same I04 supporting worktree.
Root released **one exact ignored filter only**, following independent review
of the compiler result and helper. Source report HEAD was
`f481938a999cf3c95e953f0c1ddd402be9706364`; C/Rust remain byte-exact to
`e028106c3d7c9a236056730ad500aad876616540`, production/config remains exactly
reviewed `ae8937800254a1ad4296ea257de1eccc4780e45b`. There was no source edit,
alignment, C rebuild, retry, alternative target/peer/feature, metadata/clock
override, deletion or configuration change.

**Result: Rust compilation succeeded; the one ignored function failed.**
Cargo exited101. The public failure is `pinned signature refusal` at
`tests/saml_sp_peer.rs:897:5`. The **SOLE CARGO SLOT RELEASED** exit/result was
sent to the explicit project orchestrator immediately after owned-group cleanup,
before this append. No subsequent Cargo/helper/protocol invocation ran. This
failure and all earlier report/failure prefixes remain preserved.

### Exact command, preflight and supervised execution

Exactly this command ran once, with default Platform and no extra features:

```sh
env CARGO_TARGET_DIR="$PWD/target" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  RIAUTH_TEST_LASSO_SP="$PWD/target/i04-lasso-slo-e028106/lasso-saml-sp" \
  cargo test --locked --test saml_sp_peer \
  lasso_idp_initiated_redirect_logout_revokes_only_bound_session_and_consumes_response_once \
  -- --exact --ignored --test-threads=1
```

Preflight freshly rechecked clean HEAD, exact C/Rust hashes, reviewed production,
source Cargo/lock/toolchain pins, private target/output ownership/readability,
helper39472-byte SHA
`18f148c0a3119d4a1268597c829c680a743c4978ccc96924dac337368d3a891c`,
Lasso `.pc`/receipt/headers/dylib pins and selected dynamic dependency paths.
Preflight free space at2026-10-02T15:38:30.154996+00:00 was14570266624 bytes /
13.5696GiB; launch remeasured14565322752 bytes /13.5650GiB, satisfying the
13GiB planning requirement. No version probe was run.

The captured Cargo child used its own session/process group, PID/PGID63914,
parent supervisor PID63886. The supervisor applied one1800-second outer
budget,2-second disk/owned-process metadata sampling,9GiB disk stop/8GiB floor,
exclusive0600 pipe-drained log capped at16MiB, null stdin, and joined-parent/
IO/deadline postchecks. It recorded only own-group executable names, PID/PPID/PGID,
not other-worker records, arguments, environment or protocol values. No fixture
source/60-second operation/20-second runner/15-second C limits changed.

| Actual observation | Result |
| --- | --- |
| Start/end UTC | 2026-10-02T15:45:35.651748+00:00 → 2026-10-02T15:46:35.338080+00:00 |
| Overall elapsed | 59.686627 seconds |
| Compile | `Finished test profile [unoptimized] target(s) in56.22s`; no Rust compiler error headers observed |
| Exact ignored function | **0 passed,1 failed,0 ignored,0 measured,1 filtered**,2.58 seconds; old authentication function was filtered, not rerun |
| Cargo/supervisor outcome | Cargo101; supervisor1; reason `cargo_or_fixture_nonzero`, stage `ignored_fixture` |
| Disk observations | 30 samples; minimum13705166848 bytes /12.7639GiB; after13705756672 bytes /12.7645GiB; no9GiB threshold/floor refusal |
| Owned cleanup | Cargo joined; final exact PG63914 had zero remaining members; no TERM/KILL signals needed; process metadata checks had no errors |
| Test fixture cleanup limit | Native child joins/TempDir cleanup are in the reviewed fixture; private fixture directories and short-lived helper PIDs were not separately enumerated. No manual deletion/cleanup command was issued. |

The metadata samples observed Cargo63914, rustc63937/64657/64735, clang64769,
ld64773 and test executable64805 in PG63914. These are sampled process names,
not a claim to observe every short-lived helper. Cargo/rustc actual executable
paths were under `/Users/dominik/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin`.
Cargo's cached compiler metadata before/after remained SHA
`27df402be20083ab5b4835c05762e2b77beed67288686dd8fda9193b24cdd7c4`,
recording rustc1.98.1, commit48a229ceaefd4985c50990b14116b6d856af0985,
hostaarch64-apple-darwin, LLVM22.1.8. No standalone version command ran. Cargo's
internal Rust linking invoked clang/ld; the already pinned C helper was not rebuilt.

### Precise failed boundary and evidence limit

At committed Rust lines891–900, the helper receives a substituted IdP metadata
certificate. The failed expression combines:
`status.code() == Some(1) && stderr.contains("(-111)")`.
Its only printed public diagnostic is `pinned signature refusal`. Therefore the
record **does not identify which conjunct failed, the actual native exit/signal,
or the actual finite Lasso operation/error code**. Private native captures were
not promoted into persistent evidence by this assertion. No raw stderr/protocol
was printed or reconstructed; no native error code is invented here.

Reaching that assertion confirms completion of preceding assertions in this
one run: normal approval/resume produced an issued NameID/SessionIndex; actual
Lasso `accept-state` succeeded and persisted accepted state; a fresh
`session-state` helper observed assertion present,1 name/index and nonempty;
local bound bearer was refused before peer delivery while unrelated bearer
remained valid; pending request retry was identical; emitted Redirect target,
RSA-SHA256/relay and exact NameID/single SessionIndex matched the issued values.
The substituted-certificate helper was invoked, but its composite result was
not accepted by the test.

Negative no-output/unchanged-saved-state postconditions at901–903 were **not
reached**. Positive pinned SLO processing, independently reloaded matched-session
absence, spent request refusal, correlated response confirmation, consumed
response retry/full snapshot/audit and final unrelated-user assertions at905+
were **not reached**. Those outcomes remain definitions, not executed proof.
Neither signature bypass nor a stale expected error-code diagnosis nor a
production security defect is established by this composite assertion alone.

The helper's immutable C lines574/576 check `process_request_msg` and signature
status before bindings/validation. The smallest further observation, if root
reserves it separately, is a diagnostic in **this one appended fixture** exposing
only numeric native exit/signal plus an allowlisted fixed helper stage and integer
Lasso code, retaining the refusal oracle and never raw stderr/private values.
No such change, classifier relaxation, native reprobe or test repeat occurred;
root alone determines the next exact source/runtime reservation.

### Private retained artifacts, hashes and residuals

The private log and redacted process/source/result evidence were created
exclusively in the existing own target, regular/nonsymlink/nlink1/UID501/mode0600:

| Artifact | Bytes | SHA-256 |
| --- | --- | --- |
| `target/i04-lasso-slo-f481938-filter.log` | 1304 | `a253f09fb45a4641d8373eb9318ca66bf11ff237e7570b490b24d02d6c2b4f03` |
| `target/i04-lasso-slo-f481938-filter-evidence.json` | 7006 | `22104b9b038bf784a28d51e5d05b360715be17a11a2f60544296225ad9eed2d6` |

The evidence stores exact command/settings/source/helper pins, timestamps,
outer budget/cap, samples, PID/group/cleanup and public result/panic locations.
It records no raw private protocol fields or native process arguments. The full
Cargo log remains private; this report reads/exposes only its fixed public
failure/result/compile labels. The new failed-run test executable is preserved
as evidence, with no cleanup/deletion proposal:
`target/debug/deps/saml_sp_peer-e2230a555b53761c`, regular/nonsymlink/nlink1/
UID501/mode0755,68254672 bytes, mtime2026-10-02T15:46:29.249722+00:00,
SHA-256 `7d22940e2ae52ac52d47ab480d577f3b569f5bf8df9823d1c768a08102ef5c7c`.
Its fingerprint
`target/debug/.fingerprint/riauth-e2230a555b53761c/test-integration-test-saml_sp_peer.json`
has SHA-256 `d6cd11f3231f7f0f97056b6fe396f6755040b37345f5e4d872a42e5c254dca45`,
features `default, essentials, platform`, profile11094973624911973823,
rustc fingerprint17329007180185699724.

Post-run file reads confirmed unchanged original C/Rust/helper bytes and the
selected Lasso2.9.0_4 dylib SHA. Fresh free space at
2026-10-02T15:52:04.765353+00:00 was13687803904 bytes /12.7478GiB: it no longer
meets13GiB for a new invocation. Any later release needs fresh capacity and a
new exact root reservation; no automatic retry/cache deletion is authorized.
This is partial local library lifecycle evidence with one actual refusal-oracle
failure, not a completed SLO receiver lifecycle, remote peer/browser/tenant,
Linux/Windows/release/profile or whole-I04 closure claim. Original primary,
completed rows/O07 blocker and protected product contracts remain untouched.

Post-run report verification: documentation checker exited0 and
`git diff --check` passed. The complete prior `f481938` report remains the exact
70552-byte prefix, SHA-256
`60b3d5eb305cfbb00f692d186b9be89ab8864f6551ef1dcd2980de2365ec6218`.
Protected C/Rust/helper and reviewed production equality, retained private
log/evidence hashes/mode/nlink/owner/ignored status and stored Cargo101/empty
group outcome were rechecked. Fresh free capacity at
2026-10-02T15:57:16.529866+00:00 was13685735424 bytes /12.7458GiB.
Staged/post-commit scope and whitespace checks cover this one append-only report.
None of those follow-up checks invokes another filter/helper or changes source.


## Approved source-only finite refusal diagnostic

Reservation `wave30_I04_native_refusal_finite_diagnostic`, explicit root/user
approval; same project/task/supporting worktree. Source commit
`bf7c394150141f8dda6b41c6c0644dd6a9a7d870`, parent
`f7bade4f969c83c8332d887d56a9dcd4d6480981`, changes **only**
`tests/saml_sp_peer.rs`:45 added lines /2 replaced lines, solely within the
appended ignored function. No C/helper/dylib, production/shared helper,
configuration, manifest, schema, deadline, cap or operation/input changed.
Runtime, Cargo, native compile/version/probe/helper execution remain **held**;
no runtime slot was taken or released during this source-only phase.

The prior failure and its unknown conjunct/native cause are not retrospectively
reclassified. The new diagnostic has **not been compiled or executed**. It is
an observation change for a separately reserved future run, not a refusal-oracle
correction or evidence that the SLO lifecycle now succeeds.

### Exact finite output and unchanged oracle

The assertion's boolean remains **byte-exact**:
`status.code() == Some(1) && stderr.contains("(-111)")`.
Only its failure message adds numeric `status.code()`, numeric Unix
`status.signal()` and `refusal_projection(&stderr)`. These formatting arguments
are evaluated by the assertion's failure branch; the passing path and every
other assertion/operation/input remain unchanged. Unix `ExitStatusExt` is
imported locally within this already Unix-specific appended function.

The output format is fixed:
`pinned signature refusal: exit={:?} signal={:?} projection={:?}`.
Exit/signal are numeric `Option<i32>`; projection is
`Option<(&'static str, i32)>`. Thus `None` means **null/unclassified**, never an
invented code0. A classified tuple contains only a selected static literal stage
and one integer. No raw stderr, strerror/message text, paths, URLs, private
protocol data, NameID or SessionIndex is copied into the diagnostic. Its fields
have a finite vocabulary and bounded numeric width, independent of the private
error-message length.

The local projector first refuses input beyond existing128KiB `CAP`. It borrows
newline segments without copying the private body. A classified C failure must
have exact framing `lasso <allowlisted function>: <nonempty text> (<i32>)\n`.
The final parenthesized token must parse as i32 **and** round-trip to canonical
decimal; plus signs, leading zeroes/negative zero, overflow, empty/invalid suffix
or unframed/missing newline refuse. Message control bytes refuse. The message
body is checked only for framing/nonemptiness/control bytes and never returned.

More than one `lasso ` failure-family line is ambiguous and returns `None`,
even if two lines name the same stage/code. An unknown or malformed `lasso `
family line also returns `None`, including when another valid line is present.
Nonmatching library-warning lines are ignored without display; absence of an
exact classified line returns `None`. The return value uses the allowlist's
static literal, rather than the raw input's borrowed stage string. The projector
does not affect the original boolean even when its result is unclassified.

Closed stage list, derived solely from unchanged selected C `open_server`,
`slo_restore`, and `slo_receive_logout` check/fail literals:

- `lasso_init`
- `lasso_server_new`
- `lasso_server_add_provider`
- `lasso_server_get_provider`
- `lasso_logout_new`
- `lasso_profile_set_identity_from_dump`
- `lasso_profile_set_session_from_dump`
- `lasso_logout_process_request_msg`
- `lasso_profile_get_signature_status`
- `lasso_logout_validate_request`
- `lasso_logout_build_response_msg`

The first seven cover explicitly selected initialization/provider/restore
failures; the last four cover the requested logout/signature/validation/response
stages. No arbitrary suffix/function-name or `lasso slo:` message is accepted as
an integer-code classifier. The unchanged C `fail` at lines34–37 is the source
of the exact `lasso %s: %s (%d)\n` framing. This is a finite diagnostic projection,
not a signature-verdict parser or authorization decision.

### Source-equivalence and protected evidence proofs

Committed test blob `5fddf7c1489faf4d4bb3f1744f2e5a57ca2c196e`,39476 bytes,
SHA-256 `862564f9154cd1de98378bc853e47948a731b994d22a1cf8d5be94083a1f52da`.
Removing precisely the local projector and reversing the local import/failure
message reconstructs the **entire** `e028106` test file,37798 bytes, SHA-256
`b8596b050db87d921f83e6f0b03ebf76bb6fecd74a99e191adfef6b6c7b32c86`.
The original19341-byte Rust prefix remains exact, SHA-256
`2a45eda55862c33d8784abbf17e8d4b376495f0a0f564b5b8cb3882141aa1aaa`.
This whole-file reversal proves preservation of all old/other assertion bodies,
runner/child limits, helper inputs/operations and pending retirement/replay cases.

The C remains blob `ce0a928adb22a4d00a612cf3fb7a113125303e48`, SHA-256
`c3d3a8f7d1d2d472e8b877a89ea2807d534921d88638661e8b251e2365906563`;
the existing helper remains39472 bytes, SHA-256
`18f148c0a3119d4a1268597c829c680a743c4978ccc96924dac337368d3a891c`;
selected Lasso dylib remains
`0af7c7ccfda4fe8d20c6ccdf5974a006b2b59a2d50244be95ea197c2d1f72cde`.
Selected `.pc`, Cargo manifests/lock/toolchain hashes and all production/config
tree objects still match their pinned reviewed versions. First failed log and
evidence remain exact SHA
`a253f09fb45a4641d8373eb9318ca66bf11ff237e7570b490b24d02d6c2b4f03` /
`22104b9b038bf784a28d51e5d05b360715be17a11a2f60544296225ad9eed2d6`.
No historical artifact/log/manifest was rewritten or deleted.

### Static checks and conditional future warm-filter proposal

`rustfmt --edition 2024 --check --config skip_children=true tests/saml_sp_peer.rs`
passed, providing syntax/format parsing only. Whole-file reversal, exact old
prefix, unchanged boolean, sole-owned-function diff and all11 allowed C literals
were checked statically. Whitespace/staged/post-commit code scope checks passed;
branch was clean after the source commit. One initial patch had a mismatched
context and was rejected without changing any bytes; the corrected narrow patch
then applied. No Rust type checking, parser runtime cases, Cargo, native version,
helper, C rebuild or protocol test ran during this source-only reservation.

Read-only cache metadata still has the prior default/essentials/platform test
fingerprint and cached library `riauth-f685f1e2409027d4`,409471184-byte rlib,
profile12672335563272108896 / rustc fingerprint17329007180185699724. The prior
test artifact is68254672 bytes (about65.09MiB), preserved from the failed run.
The actual first default-feature build took56.22 seconds and the complete-run
minimum-free observation was about0.801GiB below its launch observation; that
host delta is **not an independently attributed or guaranteed compile peak**.

For root's later planning only, unchanged libraries/features/manifests/toolchain
and this test-only diagnostic suggest a warm-cache additional allowance of
**0.5–1GiB**, covering a new test compile/link and transients. This is an estimate,
not a measured warm run; cache invalidation/rebuilds could require the previous
2–4GiB allowance. Fresh read-only capacity at
2026-10-02T16:14:55.207253+00:00 was13678907392 bytes /12.7395GiB. It does not
satisfy the prior13GiB planning check. **No threshold or release condition changes
here**: root must explicitly review/reserve any later capacity allowance and
remeasure before starting.9GiB stop/8GiB floor and all original deadlines remain.

The only proposed future command is the **same** previously released ignored
filter, now on immutable diagnostic source after root review:

```sh
env CARGO_TARGET_DIR="$PWD/target" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  RIAUTH_TEST_LASSO_SP="$PWD/target/i04-lasso-slo-e028106/lasso-saml-sp" \
  cargo test --locked --test saml_sp_peer \
  lasso_idp_initiated_redirect_logout_revokes_only_bound_session_and_consumes_response_once \
  -- --exact --ignored --test-threads=1
```

It is **NOT RELEASED / NOT RUN** in this phase. No alternate peer, feature,
unignored/broader filter, native helper probe, compiler step, source/oracle
correction, automatic retry or cache deletion is proposed. The first failure's
conjunct/native reason and all unreached positive lifecycle/consumption/audit
outcomes remain unknown/unexecuted until a separately reserved observation.
Original primary ownership, completed rows/O07 blocker and protected contracts
remain unchanged; no whole-I04, browser/tenant/profile/release/other-OS claim.

Source-phase report checks: documentation checker exited0 and
`git diff --check` passed. The whole previous `f7bade4` report remains the exact
80190-byte prefix, SHA-256
`f5d139b67f5492ccad605b15f7f5ea88aa86dc645ec03aee67e9d5edc8d73843`.
Post-source-commit diff contains only this append-only report. The selected C,
helper, first-failure log/evidence and68254672-byte first-failure test executable
hashes were rechecked unchanged. Fresh read-only free capacity at
2026-10-02T16:19:19.524096+00:00 was13686870016 bytes /12.7469GiB.
Final staged/post-commit checks cover the report-only commit and clean branch;
no new runtime or retrospective diagnostic/native-cause claim was made.
