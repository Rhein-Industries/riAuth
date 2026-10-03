# Q03 original-scope conformance audit and one proposed OP pilot

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original Q03 task
`9561b552-d219-477c-bb73-80e62af45226`; supporting worktree
`a1303b57-4a34-487e-9c63-a841f05b51a0`. Reservation
`wave30_Q03_conformance_original_scope_source_audit`, 2026-10-03.

**Q03 remains open.** Select one independent OpenID Provider authorization-code
pilot for the implemented S256-PKCE profile. The next input is a reviewed pilot
package identifying an actual plan and variants in the already pinned OIDF
suite, its private configuration/authorized endpoint, and the applicable
official certification procedure. No upstream plan name, certification account,
execution or pass is invented here. This is a source audit and design handoff;
it neither implements the next step nor closes the original outcome.

## Original acceptance, source and inspection boundary

The exact project export `planning/current-tasks.json` was reread as data:
244354 bytes, SHA-256
`0b38ca3eb8810a4628436eb5083f6329de848f3f4ddaa972c850601ad45c0835`.
Q03 is `[P2] Q03 — Run relevant conformance suites`, status `todo`, original
primary worktree `b64ea3dc-db1c-4bf8-b82b-990a52a440fe`. Proposed prerequisites
are A02/I01/I04; this report does not change or independently close them.
The requested outcome is **“Publish precisely scoped results and pursue
appropriate certification for the profiles being claimed.”** The workstream
requires appropriate evidence separately for security, compatibility, speed
and recovery. Its completion instructions reject report-alone implementation
completion. The export's older wait/unassigned instruction is preserved; this
explicit supporting reservation authorizes only the present report.

All current source references below mean published
`544d1340b80cd3e040dc13142cdcbc1d75fea4cb`, read through Git objects. Entry HEAD
is `e45b92cd0280e4fd0eed36d8508ae4615e5d1645` on
`roadmap/local-workflow-safety-wave27`, initially clean. No alignment was
performed. CONTRIBUTING and SECURITY were read completely; no applicable
AGENTS.md was found. Their general build/test suggestions do not authorize
runtime under this static-only reservation.

Complete source bodies inspected for the selected seam are all ten functions
and main guard in `scripts/run-conformance.py`, all six tests and surrounding
module code in `tests/test_run_conformance.py`, and the complete existing
`q03-conformance-pilot.md` and CI workflow. The five protocol guides were read
completely. For the candidate OP scope, read complete
`src/oidc.rs::validate_authorization/get_client/authenticate_client`,
`src/assembly/oidc.rs::discovery/authorization_prepare`, the public authorize
wrappers and complete `authorize_session_proof_inner`, `token/exchange_code`,
and `userinfo/userinfo_with_proof`; inspect client-specific discovery selection
in `src/issuer.rs:115–244`. These are bounded body reads, not a claim to have
reviewed every function in the three modules.

Complete selected fixture bodies read include OIDC tests at
`tests/identity/oidc.rs:4–152` and
`metadata_authorization_code_clients_advertise_usable_requests_and_authentication`
at 1926–2040; all of `tests/browser.rs`, `tests/g05_reference_oidc.rs`,
`tests/ldap.rs`, `tests/radius_peer.rs` and `tests/ldap_provider_peer.rs`;
`tests/identity/saml.rs::xmlsec/verify_xml/exercise` and its two wrappers;
and the full `lasso_idp_redirect_post_source_lifecycle_replays_and_live_trust`
test body in `tests/saml_source_peer.rs`. Historical I04 results below come
from their accepted receipt section, not a new native run or a complete audit
of every native helper. RADIUS `decode/password` and the LDAP server's
read-only operation refusal arms were also read in full.

## Claimed profiles and the evidence they actually have

| Role / claimed boundary at fixed source | Existing evidence and attribution | Limit relevant to Q03 |
| --- | --- | --- |
| OIDC OP / OAuth: authorization **code only**, S256 PKCE for every client, exact registered redirects, one-use request/code binding, current policy/session; none/Basic/post/private-key JWT client authentication; RS256/ES256/EdDSA, selected JWE/JAR/PAR/JARM/DPoP/resource/device/refresh/service extensions | Public-JWKS ID-token verification uses independent `jsonwebtoken`, issuer/audience/kid/nonce/at_hash and matching UserInfo subject. Redirect/plain-PKCE/wrong-verifier/code-replay and client-secret/original-client refusals are explicit fixture assertions. Real Chromium completes a public-client HTTP callback, checks state/issuer/nonce/JWKS and signed backchannel logout. | Independent libraries and Chromium are compatibility evidence, not an OIDF result. Implicit/hybrid and RFC7592 registration management are not advertised. One selected pilot cannot establish all extensions or FAPI. |
| Upstream OIDC source / OAuth-only source: pinned code callback and explicit local linking; OAuth-only HTTPS JSON identity has no OIDC auth_time/MFA proof | Source guides describe exact issuer/redirect/request/session/proof binding and unsupported upstream encrypted-ID-token/private-key-JWT combinations. Existing suite runner targets an OP pilot, not all source/provider adapters. | Do not absorb upstream provider/tenant rollouts or certify them from an OP plan. Preserve explicit links, local factors and current authority. |
| Platform SAML IdP: signed Redirect/POST requests; POST response; signed metadata/response/assertion, mapped attributes, optional encrypted assertion, selected NameID and SLO profiles | CI explicitly executes three XMLsec tests. The complete IdP exercise independently verifies metadata and both signatures, signs a POST request, decrypts the assertion and verifies its signature; checks correlation, ACS/index, escaped name, one-use/browser/ForceAuthn and consent/group refusal paths. | XMLsec is an independent cryptographic tool, not a full SP or formal certification. SOAP/artifact/ECP/encrypted NameID and universal graph translation are outside the documented profile. |
| Platform SAML source/SP: signed Redirect request and pinned double-signed POST callback, stable subject, optional encryption and selected SLO | Accepted historical Lasso 2.9 IdP-source receipt records one local lifecycle pass: native response, exact request/subject/session binding, explicit links, unlink and trust-withdrawal retirement, stale/spent/bad-signature refusals. Accepted Lasso SP/SLO records are separately scoped in I04. | Real Lasso is distinct from test-authored XMLsec messages. Its selected persistent-NameID/macOS lifecycle does not establish every binding/mapping, remote IdP/SP or deployment. No recognized SAML certification procedure is supplied in this reservation. |
| RADIUS PAP over UDP / RadSec; EAP-TLS method13 with TLS1.3 or configured TLS1.2 EMS, explicit certificate binding; Message-Authenticator before password | Historical accepted records name FreeRADIUS radclient3.2.10 PAP accept/reject/exact datagram replay with one accept audit; selected OpenSSL RadSec/EAP fixtures are separately attributed by I04. The complete radclient fixture assertions were read. | radclient is an actual PAP peer; OpenSSL uses test-controlled framing. The current CI check ignores radclient. Accounting/CHAP/MS-CHAP/PEAP/TTLS/CoA/Disconnect/NAS termination are outside the profile; no hardware supplicant or network-wide certification is inferred. |
| LDAP import/password check: bounded paged simple bind/search over STARTTLS/LDAPS, stable IDs, reviewed plan/apply, no password copying | Actual CI OpenLDAP/slapd STARTTLS fixture uses ldap3: entryUUID rename preserves subject/factor, stale/collision/revoked-agent/untrusted-CA refusals, directory password+MFA, one-use recovery, and 205-entry paging with exact-plan removal confirmation. | This execution is STARTTLS import, not an LDAPS-import or real AD objectGUID/memberOf run. Partial searches never authorize removals. |
| Platform LDAP provider: read-only LDAPv3 simple bind, LDAPS/STARTTLS, RootDSE/WhoAmI, scoped filter/paging/current authority | Accepted historical OpenLDAP ldapsearch2.7.1 record covers both transports, page-size1, disabled omission, revoked/wrong-token, wrong-CA and crossed-transport refusal. Complete fixture asserts only the scoped users and no credential leakage. Production add/modify/delete/modifyDN/compare return UnwillingToPerform. | The current CI check ignores this executable peer. No writes/SASL/password-modify/AD or POSIX emulation is advertised; independent LDAP clients do not establish a certification scheme. |

This matrix separates advertised boundaries, local fixture construction and
actual peer execution. It creates no all-provider campaign or additional
universal protocol gate. Certificate, group, credential, receipt/header,
removal-review and PAM behavior stays unchanged. No Group/nonrenewed-60s/
paused-I/O contract is relaxed for the proposed pilot.

## Dated executed evidence and pins

Root-retained completed metadata for Public CI **37098106793** identifies
source **`143d99dbe43fefd6aeb3d274a0d974fd025292e0`** and success of check
`111134191840`, integration `111134191938` and audit `111134191804`.
This is a completed result at that source, not a claim about another run,
A09 ARM37101183416 or a released/deployed artifact. Byte comparisons show
every source/guide/fixture listed in the immutable table below is equal at
143 and fixed544d. The result-index first all-target command records
182 test-result blocks, 1330 passed/182 ignored; its 204 blocks across commands
are not 204 unique test programs.

The check raw log lines 2544/2551/2556 explicitly pass the selected client
metadata, public-JWKS and redirect/PKCE/replay tests. Lines 5047–5141 retain
the ordered Python commands and six conformance preflight tests **OK in0.089s**
(the preceding six-test0.028s block is the installed-release-gate command).
Those six fixture cases validate safe explicit URL, owner-only regular config,
exact suite pin/dirty refusal, exclusive private output, export symlink refusal
and missing-input refusal. The suite-pin test temporarily patches PIN to a
dummy checkout with an entry file containing `pass`; it is not an upstream
OIDF suite invocation.

Integration raw lines1328/1347/1366 pass the three named ignored XMLsec tests,
one each, with recorded test durations5.96/5.55/7.07s. Line2900 passes
`openldap_plans_stable_ids_tls_login_mfa_and_fail_closed_sync` (7.99s), and
line2920 passes
`browser_terminal_login_callback_and_signed_backchannel_logout` (3.40s).
The check alone explicitly ignores those external-tool fixtures. It also
ignores `ldapsearch_starttls_bind_scoped_paging_disable_and_revoke` and
`radclient_pap_accept_reject_and_cached_replay`; no integration CI command
executes these two tools. The complete workflow has no OIDF plan command.

Historical G05's in-process reference RP uses real `jsonwebtoken` but invokes
Core directly, restores only a saved route file and expressly has no Authentik
peer proving rollback login. Its1-pass7.33s CI result is not a production-RP
migration or conformance result. Read its complete assertions, rather than
using the word “reference” as certification evidence.

Historical real-peer attribution comes from the fixed544d
[I04 profile evidence report](https://github.com/Rhein-Industries/riAuth/blob/544d1340b80cd3e040dc13142cdcbc1d75fea4cb/docs/roadmap/local-wave30-i04-profile-evidence-review.md)
and [D05 E-PEER evidence](https://github.com/Rhein-Industries/riAuth/blob/544d1340b80cd3e040dc13142cdcbc1d75fea4cb/docs/roadmap/d05-acceptance-evidence.md).
Read I04's complete final actual receipt3086–3239: source
`4ed5b494454da33bd27bc046084522c05a55e2a5`, correction
`24c0dc921703432ff04a5cab7cc771c0c9d7df3e`, exact locked
`saml_source_peer::lasso_idp_redirect_post_source_lifecycle_replays_and_live_trust`
ignored filter, Cargo/supervisor0, **1 passed/0 failed/0 filtered,2.63s**,
five helper children, joined/empty owned group. Its test SHA is
`b196cba1eea40a67c7d70c6b463ae50ec6897d68c5d81052e33eb3bec2ae5894`,
native helper SHA
`951465d744c1bf99e7ed91fc414337d00e960c24a1715977cce0e14535794bff`,
and retained raw-log SHA
`f0807e70fc5ab811d18aa2daac6df649be29b47b5f69dc8975834c0958769602`.
The current whole test differs only by removal of the unused POST constant;
that source comparison does not remeasure its historical pass. This audit
read the accepted receipt, not the old native binary or original raw log.
Earlier I04 native/compile failures remain failures in that append-only record.
D05 E-PEER is a secondary accepted attribution of the older ldapsearch/radclient
runs; their original runtime logs were not independently rehashed here.

All following retained CI files are under root's project
`planning/evidence/`; their complete bytes were hashed, JSON parsed as data,
and relevant finite raw test/result blocks inspected, not executed:

| Retained file | Bytes | SHA-256 |
| --- | ---: | --- |
| `wave30-ci-37098106793-completed-metadata.json` | 12535 | `a2fd07518e927a2fb7ae44b9a03840114a5a4332866bbb5f27cadeab8732faa8` |
| `wave30-ci-37098106793-check-result-index.json` | 74052 | `76257552c455f5ffe96c7c38cd8438d61779f8cd94b7a75be6470c38e375a7fe` |
| `wave30-ci-37098106793-integration-root-review.json` | 3138 | `bba26efb18adab0e20d8f54ebc5937d1f9bf001eb81ad5fb759cf59e4bc00bef` |
| `wave30-ci-37098106793-check.log` | 472408 | `8e4a535d877e228c9c74e8fae5b572fb65dde96988b175bd8ee0eec0b21993e4` |
| `wave30-ci-37098106793-integration.log` | 265769 | `f1ea748ec8e652305b37e1e6dfb74ee31801f7cadbbdf3c715b8443984048afd` |

The integration-only receipt originally said the overall run was in progress;
later completed metadata establishes the dated overall success. No current
live job was queried. Raw-log hashing is an identity check, not a claim to
have reviewed every unrelated body/assertion in the whole CI run.

## Existing suite/tooling versus the missing certification input

The complete [runner](../../scripts/run-conformance.py) pins OIDF suite commit
**`440eec8bac7b12b7389d7ca9cbc459b53507a443`**. It requires the exact clean
checkout/root/entry point, owner-readable owner-only regular nonsymlink config
up to2MiB, explicit safe suite endpoint/token, exact nonempty plan string,
clean tracked riAuth source and a fresh output outside the suite checkout.
HTTPS is required except literal loopback HTTP. Token values are not logged.
It invokes only one upstream plan using `--no-parallel --export-dir`.

The retained `riauth.conformance-pilot/v2` record has
`certification_claim:false`, source/suite/plan, config/endpoint digests,
numeric exit and captured log/export sizes/hashes. Its `passed` predicate is
**upstream exit0 plus at least one export file**. It does not parse per-test
result statuses or establish that the named plan is in the upstream inventory.
The temporary managed logs/exports are removed before run.json is written.
That record cannot on its own publish precisely scoped normative results or
serve as a certification submission. Its private output directory/file modes
are0700/0600 and overwrite is refused.

The complete main child launch has no timeout, byte/total-export cap, owned
process-group/reaping supervision or finite resource guard. This is a concrete
source-derived runtime-preparation seam, not an observed failure or an
authorization to run it. Before any future invocation, root must reserve a
finite supervision and private detailed-result retention design appropriate
to the actual upstream entry point. Do not replace detailed result inspection
with exit0/digest counts, leak raw exports, or silently weaken existing checks.
Suite/configuration side effects outside the managed temporary directory also
require actual source review, as the existing pilot document already states.

The existing [pilot record](q03-conformance-pilot.md) dates its missing-input
finding to2026-09-29 at `19a69c66b473480c8b570498231fad4d4edb7a31`.
It records no independent plan execution or certification. This audit did not
search live environment/private credential contents or all local installations;
that dated absence is not recast as proof that inputs do not exist elsewhere.
No supplied official certification application/procedure or reviewed upstream
plan source was available for this audit. Protocol standards links in the guides
are not that procedure. Per instruction, no external process was browsed.

## One selected next input and bounded future plan

Choose **one code-only OpenID Provider pilot** using query code response,
mandatory S256, RS256 ID token/public JWKS, public subject, a confidential
client explicitly configured for `client_secret_basic`, exact callbacks and
JSON UserInfo. Restrict scopes to the reviewed pilot's needs; do not enable
JARM/JWE/device/token exchange/upstream federation merely to make a test pass.
This names a product scope, **not a guessed upstream plan or certification
profile**. Client-specific discovery is already a source-backed way to expose
the configured grant/authentication boundaries; its complete selected fixture
asserts code/S256/auth-method filtering. All live issuer/client/session/policy,
request-bound consent, one-use proof/code and replay protections remain active.

**The single next prerequisite is a root-supplied, reviewable pilot-input
package.** It must bind the verified upstream repository/commit provenance,
the pinned OIDF checkout's actual plan inventory and
selected plan/variants to this scope, with the complete entry-point/config
schema and current official procedure supplied as immutable source/document
input. Include authorized suite-management endpoint/account access and
owner-only configuration, a pinned pilot OP artifact/source/edition, issuer
and callback/client authentication contract, and the supported operator or
browser-consent interaction. Private credentials remain private; only their
availability/ownership and redacted input identity belong in a public report.
`CONFORMANCE_SERVER` is validated/inherited by the local runner; the absent
upstream source must establish its actual role rather than equating it with
the OP issuer by assumption.

Review applicability before reserving code/runtime: `src/oidc.rs:137–168`
rejects non-code and requires S256 even for confidential clients. A generic
plan that expects a positive non-PKCE flow cannot be made applicable by
turning off this accepted protection or falsely calling expected refusals
passes. Determine the actual supported upstream variant, or retain the exact
documented incompatibility and certification prerequisite. Do not invent
implicit/hybrid/FAPI support. Official application/submission/self-test rules,
profile/version and result-publication requirements remain explicit missing
inputs; a local pass would not automatically grant certification.

After input review, root can reserve the smallest runner supervision/result
retention change and **one** exact plan invocation on the authorized isolated
pilot. Its evidence must preserve actual exit, each official test outcome and
not-run/failure reasons, suite/config/source/artifact identity, strict refusal
assertions, finite workload/resource bounds, private caps and owned cleanup;
retain a secured original export with only reviewed redacted public results.
No executable command or guessed deadline/host budget is proposed without the
actual suite source. Existing local preflight tests may then validate only that
new seam, under a separate focused reservation. This is not a request to
repeat the Rust suite or launch a tenant/all-protocol campaign.

Appropriate certification pursuit follows the supplied official procedure for
the selected claimed profile. SAML/RADIUS/LDAP keep their precise real-peer
results and documented limits; this report neither labels them certified nor
invents universal certification programs for them. Broader appropriate suite
selection remains part of original Q03 review after this one scoped pilot.

## Immutable source identity and this report's static checks

The following full-file identities were recomputed from fixed544d objects;
all are byte-equal at completed CI source143. Full-file hash identity does not
expand the body-inspection boundary stated above.

| Path | Bytes | SHA-256 |
| --- | ---: | --- |
| `scripts/run-conformance.py` | 8023 | `eb4ef137caba864192988581c66258b973adc6715843ba4170f2c26fdce25e35` |
| `tests/test_run_conformance.py` | 4945 | `fd80cb4128c762ca474d4905283c154ba5371bb24d99f8de9b055ee55aa45110` |
| `docs/roadmap/q03-conformance-pilot.md` | 3356 | `ecd64b00cb5c9aa5c8fe4d65a0f77a5091638a68e1a38c31c9e1487263263173` |
| `.github/workflows/ci.yml` | 11879 | `fc5245ef15f0ac8d0a460b71ded533ad96ad3dcbd795b31f38b3e84221940b5a` |
| `src/oidc.rs` | 16308 | `affdc18fcf9aad41f87ee522f52a0000eb4d58068f0db37e6e05521f5a2ef81b` |
| `src/assembly/oidc.rs` | 78806 | `6747207d82683c9d3844798ac350a80fa8058ad3a52c5aa6be7691e9b2e20dde` |
| `src/issuer.rs` | 12717 | `91a68fad4f9f97d4a87ce0497949b88d4b20fa1fc8de2c34ee6825bfe6fa0bd8` |
| `tests/identity/oidc.rs` | 120199 | `67f12d346528e9415ee47191d1e0e2779234a108ee9cbf0d92e7397d564a4228` |
| `tests/browser.rs` | 11225 | `e3de993b1bb369bf181b9e103ec0715e66c7fc7ade9fb214d0fefe55ce8f3736` |
| `tests/g05_reference_oidc.rs` | 11366 | `64e43530f6c6e69c2fe4f7f9d45fabb2cfb5b1843a6b022eec2a418008602ba6` |
| `tests/identity/saml.rs` | 110934 | `864e4c352374e807e9afe539a794afee7f76f6c51d2ad43ec3a08219f2fc4bc9` |
| `tests/ldap.rs` | 11895 | `3edd73958eabd10db3e000723c5b8f8bd4ad6f387cff8dd57d281177fcd9dcee` |
| `tests/radius_peer.rs` | 8109 | `7c2176d15a013fd0b044d9124632522a7d37f4e729c212816726b5f1aa06095f` |
| `tests/ldap_provider_peer.rs` | 17169 | `3269488d42ce482e03d7f2b5134305d59408a990ac4565e16b5779dfd89cd515` |
| `tests/saml_source_peer.rs` | 39470 | `610e29c32d7eed456c33493aea2bb1eab52fc7a5791d36db5686c59532fc5512` |
| `docs/roadmap/local-wave30-i04-profile-evidence-review.md` | 224901 | `c6f8f594645a2d94dbbdebf33d3f83bb9cec8214380aea263526d39d5c6f244a` |
| `docs/roadmap/d05-acceptance-evidence.md` | 153623 | `87dc6e145f8db722f3e4c2acfa7ddc027cba4c86516acfb2252c928a92ac52f6` |

Static work in this reservation: read Git objects and the exact original row;
parse the two Python sources with `ast.parse` without executing/importing
them; hash the retained public CI data/logs and source objects; compare named
objects across143/544d and the historical Lasso test delta. Final report checks
are `python3 scripts/check-docs.py`, `python3 scripts/check-repo-hygiene.py`,
`git diff --check` and the staged whitespace check, exact source/data pin and
sole-new-path checks. The first docs/link check exited0; all22 tabulated
source/retained-data sizes and SHA-256 values matched; the sole-new-path check
passed with no existing tracked/index changes; unstaged `git diff --check`
exited0. Final staged checks: docs/link checker0; repo hygiene0 (1023 tracked
files); `git diff --cached --check`0; entry-HEAD/sole-added-path/index-versus-file
identity check0. The report is the only added path; existing source, tests,
helpers, workflows, manifests and earlier reports are unchanged.

No upstream suite, test, helper, product function, browser/native/HTTP/PG/Cargo
or network action was run. No certification source was downloaded, no private
credential content inspected, no other worker contacted and no validation slot
taken or released. Original primary/status and all closed rows remain intact;
root owns any later reservation, independent review, integration/publication
and original Q03 disposition.

## Upstream PKCE applicability follow-up — source only, 2026-10-03

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; reservation
`wave30_Q03_upstream_pkce_applicability_followup`, same supporting WT/shell.
Entry HEAD `2b1a2b873d98d04458b4099607b872994352017d` was clean. Only this
report may be appended. Its original23568 bytes/SHA-256
`fb0def83b27ac5db3db158b9194f3fb4a9fc4dc7e87b1a18550e62413cb9e97c`
remain an exact prefix. The original row, ownership/status and dated proposals
above are retained; the new source findings do not recast them as executed.

**Finding:** the unchanged pinned Basic plan contains an ordinary positive
`oidcc-server` flow without PKCE. Published product
`1a517a1a461b7017c353d37a5e498d2c7cfa7985` requires valid S256 for every
client. Therefore this unmodified Basic plan cannot support a complete
successful pilot for the presently claimed mandatory-PKCE profile. This is
a source-derived incompatibility, not an observed suite failure, formal
certification ruling or product defect. Do not relax PKCE or relabel an
expected authorization refusal as a successful happy flow.

### Saved upstream identity and full-body inspection

Root supplied read-only DATA under
`/tmp/riauth-root-q03-pinned-upstream-source`; no query/download occurred here.
The mirror selector is commit `440eec8bac7b12b7389d7ca9cbc459b53507a443`,
with root tree `e03cffcff4031d3e31fd7b37381d63219be334dd`. Saved `tree.json`
is2064761 bytes/SHA-256
`3ec978d90a9de3bb6db8ba648de21ee628eb798f163256bd041fd44e072a1605`,
6502 entries, `truncated:false`; its response `sha` field is the commit
selector440, rather than the reconstructed root-tree ID. Independently
reconstructed all306 directory tree objects from their immediate children's
mode/name/Git-object bytes, checked each declared subtree and obtained root
e03. Unique paths, parent-tree types and all nine supplied source blobs'
`SHA1("blob " + length + NUL + bytes)` and sizes match that saved tree.
This binds the body bytes through the complete saved hierarchy; it does not
independently reproduce a signed commit, official deployed suite or live API.

| Saved source (unique basename in pinned tree) | Bytes | Git blob | SHA-256 |
| --- | ---: | --- | --- |
| `OIDCCBasicTestPlan.java` | 5210 | `ce62d47b9ed99481b1fd0c504d13c94f7196e8d7` | `3633b091fd8d27893e10330f2b9d0cecd9c1e55dcdaba2289dfed74fa4d65eef` |
| `OIDCCServerTest.java` | 3827 | `68ab759a192533ae9ac3498bf3a5ca77819dbe82` | `e0b8474972651f01e97f6bce536ef7cacd6445797dfc90c425f64a1249709b1e` |
| `AbstractOIDCCServerTest.java` | 32843 | `2309043d3db8bda16dd6ffc9d1b4bc896d1532f8` | `2d87e8affd916a8cfbc89161b9f521e25ed29b3b56811f616758c17652bcf226` |
| `OIDCCEnsureRequestWithValidPkceSucceeds.java` | 1604 | `89181d0ed1ce82aa032c0cbc4599e01247985468` | `844b9c506e8125d173d1ab1cdd591f63e66bbbb1df1a56e0febed7165cce4e96` |
| `CreateAuthorizationEndpointRequestFromClientInformation.java` | 1596 | `856968fbc9dfdbd3eeb88f0ea779e225ea001fd5` | `731ca692064e0d74a4d913e788658a8c39d6c51e5e228ad6e0ef55f115ab0edb` |
| `CreateTokenEndpointRequestForAuthorizationCodeGrant.java` | 1062 | `6dd10e11063ba9581c9578bba2951eb3a1c6d623` | `595ff1787f032afa7d256d33b73d190ee16993369d86940d45efe2556a5a571d` |
| `SetupPkceAndAddToAuthorizationRequest.java` | 917 | `d23c04f8b26489a9349e4954d3cb142d469f9b68` | `bab137a2c8b86024e9b9c710922d1c915f5e26a86386f4a7bd4e4b9e7700eda0` |
| `scripts/run-test-plan.py` | 79831 | `45e82abab5713cd03b9305ae2ed5bbac9c3a7ba2` | `2abe903a8458efabda79e19dcb8f2be08a158fbe65d9e9169f1d6140fa36820e` |
| `scripts/test_plan_parser.py` | 3329 | `872aff69b61635a147730343bb4184ef304c92ee` | `2b00870b2dc46f1d44d047c715c0ef6978f24ece97a4a9d3b48ceeea8d01830f` |

The Java files reside under `src/main/java/net/openid/conformance/`: the four
test/base classes in `openid/`, the two request builders in `condition/client/`
and PKCE sequence in `sequence/client/`. All seven complete Java bodies were
read, including the full745-line base, not merely imports or class names.
The entire97-line parser source was read without calling/importing it.
Scoped complete runner reads cover top-level imports/environment setup,
`run_queue`, `run_test_plan`, `run_test_module`, `analyze_plan_results`,
`analyze_result_logs`, CLI argument grammar, plan/config/queue setup and
result/exit handling. The remaining presentation/profile-inventory code in
the1646-line runner, its imported `conformance.py`, deeper Java framework,
variant enums and PKCE leaf-condition implementations were not fully reviewed
or supplied as executable prerequisites. AST parsing of the two saved Python
files executed no imported source, definitions, parser callbacks or main.

### Exact applicability trace

1. `OIDCCBasicTestPlan.java` publishes
   `oidcc-basic-certification-test-plan`/`Basic OP`. Its first list includes
   `OIDCCServerTest.class`; `variantCodeBasic` fixes response type`code`,
   client authentication`client_secret_basic`, response mode`default`.
   It also separately includes the valid-PKCE module in its third list with
   these same fixed variants. Presence of that extra module does not modify
   the first happy-flow module or turn the whole plan into PKCE-only testing.
2. Full `OIDCCServerTest.java` extends `AbstractOIDCCServerTest`, overriding
   token/code/claim validation and post-flow checks, but neither request
   sequence nor token-request construction. The base `onConfigure` is empty;
   normal metadata/static-client selection and Basic authentication setup
   do not introduce PKCE.
3. Base `performAuthorizationFlow`505–511 calls request construction,
   redirect construction and `performRedirect`. `CreateAuthorizationRequestSteps`
   513–537 builds client/redirect/scope, random state, random nonce and
   response type, plus form-post mode only when selected. Its default factory
   539–545 returns that sequence. The complete request-builder body creates
   a fresh object from only `client_id`, `redirect_uri` and optional `scope`;
   a client config field named challenge/verifier is not copied by it.
   No PKCE sequence occurs in this ordinary path.
4. Base `createAuthorizationCodeRequest`629–636 creates a fresh code-grant
   token form and adds selected client authentication. Its complete builder
   writes `grant_type`, `code`, `redirect_uri` and resets headers. No verifier
   is added by the ordinary module. Its callback checks require successful
   authorization/code extraction, then token success and UserInfo; rejection
   is not its positive oracle.
5. Product `src/oidc.rs::validate_authorization`157–166 refuses unless method
   equals`S256`, challenge length43 and base64url decoding yields32 bytes;
   its fixed error states that the requirement applies to every client.
   The branch does not exempt confidential Basic clients. This full module
   is16308 bytes/SHA-256
   `affdc18fcf9aad41f87ee522f52a0000eb4d58068f0db37e6e05521f5a2ef81b`,
   byte-equal to the original544d read. No request/status/token transcript
   was observed in this follow-up.
6. The dedicated module publishes
   `oidcc-ensure-request-with-valid-pkce-succeeds`. Its sequence override
   appends `SetupPkceAndAddToAuthorizationRequest`; its token override calls
   `AddCodeVerifierToTokenEndpointRequest`. The complete PKCE sequence calls
   random verifier, `CreateS256CodeChallenge`, environment exposure and
   `AddCodeChallengeToAuthorizationEndpointRequest` under RFC7636 requirements.
   These are inspected call sites, not an executed request or an independent
   full audit of the unsupplied leaf algorithms.

This one unavoidable ordinary positive is sufficient to reject a proposed
**complete unmodified Basic-plan PASS claim** against this product source.
It does not assert that every Basic test is inapplicable, that the dedicated
module already passes, or that another official suite/profile/version cannot
accommodate the mandatory-PKCE policy. Formal applicability/exceptions and
submission requirements still belong to the chosen official procedure.

### Actual upstream arguments, endpoint roles and private configuration

`run-test-plan.py::parser_args_cli`1045–1060 takes `params` with `nargs='+'`;
it has `--no-parallel`/`--export-dir`, not a `--suite` argument. Main joins
positional strings, then the source grammar scans plan expression plus config
path. The outer local `scripts/run-conformance.py` owns the separate `--suite`
checkout argument and passes `plan` and `config` as positional upstream argv.
Both layers must be distinguished.

| Role | Source-grounded behavior | Still required before runtime |
| --- | --- | --- |
| Suite-management base | Main1128–1257 uses `CONFORMANCE_SERVER` as `api_url_base`, trailing-slash normalizes it, constructs `Conformance(api_url_base, token, verify_ssl)`, waits for suite readiness and requests test-module inventory. Plan/module creation and log links use this base. | Authorized pinned suite deployment/API/account, TLS and reachability. This is **not** the riAuth OP issuer. |
| Suite credential | Non-development branch1191–1194 reads `CONFORMANCE_TOKEN`; development mode can omit it. | Authorized suite API token handled privately; it is not a riAuth client secret/session. No token was inspected or created here. |
| OP under test | Java base configuration fields select `server.discoveryUrl` for discovery, or `server.issuer`, `jwks_uri`, authorization/token/UserInfo endpoints for static metadata. | Actual reviewed OP artifact/source/edition/issuer/security profile and matching private config. An annotation is not a complete config schema or supplied deployment. |
| Client/callback | Static-client field`client.client_id`, Basic field`client.client_secret`; Java base creates/exposes redirect URI and selects registration/authentication variants. | Exact suite-derived callback/alias and normal reviewed client creation authority/receipt/header contract. No client, grant, account or secret was created. |
| Suite URL substitutions | `run_test_plan` replaces `{BASEURL}`, `{LOCALBASEURL}`, `{EXTERNALBASEURL}`, `{HOSTNAME}` and `{BASEURLMTLS}` from suite environment; `EXTERNAL_URL` overrides external base. | Callback routing and OP issuer must be reviewed separately. These substitutions are not evidence of an OP URL or tenant. |

When the suite base is omitted, upstream main tries localhost ngrok discovery
then development defaults; main also reads its `scripts/certs-keys/` directory
**before** parsing CLI options and the `--list` branch. Thus even a list request
is not authorized here as a pure argument/parser check. `DISABLE_SSL_VERIFY`
presence or `CONFORMANCE_DEV_MODE` changes verification; the future security
profile must not silently accept those bypasses. Imported API transport/cert
fixtures remain source/setup prerequisites. No environment values or private
configuration content were read by this audit.

### One limited module-selection proposal, not certification

The smallest source-backed candidate is **only** the published valid-PKCE
module from Basic, with discovery/static registration explicitly selected.
The plan entry itself fixes code/Basic/default; these do not need duplicate
outer variants. Proposed literal `--plan` value,143 ASCII bytes:

```text
oidcc-basic-certification-test-plan[server_metadata=discovery][client_registration=static_client]:oidcc-ensure-request-with-valid-pkce-succeeds
```

This is a source/grammar proposal, **not parsed or executed**, and is below
the unchanged local runner's200-character limit. Saved grammar supports name,
`[name=value]` variants and `:module[,module...]`, then a separate config path;
the config path alphabet is ASCII alphanumeric plus`-_./`, excluding spaces,
colon and backslash. A concrete authorized private path must fit it. No
nested `{op_test}` pairing, sample Node/FAPI client, environment grammar,
restart rerun or expected-failure/skip list is proposed.

Upstream `run_test_plan`149–248 reads config, creates the plan and filters
returned modules to the selected name; the future receipt must verify that
selection resolves to exactly the intended module/variant. A nonempty
returned subset alone is insufficient proof that arbitrary requested names
were all recognized. `--no-parallel` serializes both queue levels. The source
default `CONFORMANCE_RESTART_RETRIES=2` allows up to three attempts; a future
single-invocation pilot would explicitly set the normal environment override
to`0`, with separately reviewed finite supervision rather than hidden retries.
The CLI does not by itself automate the riAuth OP's consent/approval browser
interaction: the selected server module can wait for it. Actual interaction
and cleanup remain prerequisites, with Driver-only desktop preference.

Module execution, source-only metadata preparation and formal certification
are separate reservations. Root already asked once for suite/acceptance
profile/authorized private configuration; the answer is pending. This report
does not repeat that question, query an account or invent its response.
Root's official `connect_op_testing` page review is supplied process context;
its complete page bytes were not supplied/read independently here, so no new
official procedure, approved waiver or certification entitlement is asserted.

### Smallest prospective runner seam and normative-result limits

The existing local runner remains8023 bytes/SHA-256
`eb4ef137caba864192988581c66258b973adc6715843ba4170f2c26fdce25e35`
at1a517a1, equal to the original544d body. Its subprocess launch in `main`
and cleanup/evidence block is the already identified narrow prospective seam:
finite owned-process supervision/capped private captures and retained detailed
module/export evidence **before** grading, while preserving all current
suite/config/output/source/secret guards. No source hunk is implemented or
reserved by this follow-up; exact controller/retention limits require root's
separate source reservation after the input package.

Upstream `run_test_module`251–431 captures each module ID, info and condition
logs; `run_test_plan` optionally calls `conformance.exportjson(plan_id,
output_dir)` after its queue. `analyze_plan_results`619–691 distinguishes
incomplete execution and unexpected failure/warning/skip conditions;
`analyze_result_logs`702–870 examines normative condition results. It admits
several final labels and relies on condition logs for failed results; numeric
process exit alone must not be called a clean module PASS. The pilot needs
the actual selected module/variant, terminal status/result, normative
failure/warning/skip inventory and full private exported result—not only
an output-file count/digest. The imported export/HTTP implementation and
actual export shape/caps were not read, executed or asserted here.

The unchanged local runner hashes temporary stdout/stderr/exports and then
deletes its managed raw files before writing `run.json`; it still reports
`certification_claim:false`. A future retention design must retain bounded
owner-only raw normative exports privately, with sanitized public result
summary and provenance, before cleanup or pass evaluation. No raw credentials,
code/token/state/nonce/verifier/cookie/subject or full suite log/config belong
in the public report. Exception/restart logs can contain sensitive material;
unmodified upstream stdout is not a preapproved public artifact.

The original Q03 result-publication/certification gate remains open. All prior
local/native failures and scoped CI passes stay attributed to their original
pins; none is an OIDF result or a fresh1a517a1 run. No product authorization,
secret/receipt/header/PAM, Group, nonrenewed60s or paused-I/O behavior changed.

### Follow-up static checks and explicit non-execution

Actual independent DATA checks: all nine Git blob/size/SHA bindings,
all306 directory/root-tree bindings, original prefix size/SHA and three
product/source-guide object equalities passed; saved Python AST parses passed.
Full-byte hashing is identity evidence, with the scoped body-read limits above.
`python3 scripts/check-docs.py` exited0; indexed
`python3 scripts/check-repo-hygiene.py` exited0 (1024 paths);
`git diff --check` and `git diff --cached --check` exited0.
The exact23568-byte prefix, nine-row appended source table and sole modified
report scope checks exited0. Final index/file, immutable commit and clean-tree
checks accompany this report-only handoff.

No upstream parser/import/module/main, condition, helper, product function,
certification account/token/client creation, native/browser/Driver, service,
HTTP/provider/network/query/download/dispatch, Cargo/compiler/version or test
was run. No runtime slot was acquired/released, no source/helper/guide/config
was edited, and no alignment/merge/status/main/push/new worker/task/WT/shell
or other-worker contact occurred. Root owns later review/reservation/runtime,
integration/publication and original disposition.
