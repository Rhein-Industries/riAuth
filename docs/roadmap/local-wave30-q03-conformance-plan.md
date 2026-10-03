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
