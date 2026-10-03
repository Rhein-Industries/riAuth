# Wave 30 X01 named SAML extension requirement audit

Date: 2026-10-03. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Original X01: `988578f4-8544-4604-a508-3c3c2e676f8b`.
Reservation: `wave30_X01_named_saml_extension_requirement_audit`.
Supporting WT: `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`, existing shell
`2173637e-bdb3-4eba-a128-178f57b41a34`, branch
`roadmap/sol-diagnostics-wave30`. Clean entry HEAD:
`abef1214ce8f005afb15f1ca391261ffc35d3759`.
Immutable source reviewed: `a6d361600a03713fc1b687f367e9db84efe43463`.
Only this new report is owned. The worktree was not aligned to that source.

## Decision and the one next input

**No named application requirement for artifact, SOAP, ECP or encrypted NameID
is recorded in the inspected export, pinned actionability row or selected
integration evidence. No optional-profile implementation hunk is justified.**
The current refusal boundaries agree with the documented browser profile;
their existence is not a product defect under X01's demand-driven wording.
This finding is bounded to the records inspected, not a claim that no external
application could need an extension.

The smallest next input is **one root-reviewed named requirement record**:
application/product and exact version or build; riAuth's required IdP/SP role;
one required profile/binding or encrypted-NameID requirement; its concrete
public requirement or reviewed metadata witness; why the existing signed
Redirect/POST profile and optional assertion encryption cannot satisfy it;
and the available real peer, its provenance and owner-approved test scope.
An optional feature on a vendor's feature list, a product name in a migration
inventory, or a library API does not establish that requirement. No live keys,
credentials, NameIDs, sessions or protocol captures are requested by this
public record. No duplicate user question or peer contact occurred here.

Root can then reserve a security model and one peer fixture for that single
requirement before source ownership. There is no candidate source, diff,
new runtime command or extension pass in this report. X01's original
assignment/status, I04 DONE and distinct Q06/U10 work remain unchanged.
This audit is not implementation, release evidence or completion of the
extension's real-peer gate.

## Exact original row and state provenance

The complete exact-UUID row, including all details, was read from the local
export at:

```text
/Users/dominik/.local/share/riwork/orchestrators/projects/891e7443-8dac-4c1b-897f-9e53cb59c7ee/planning/current-tasks.json
```

This is an **export observation, not a live board query**. Export metadata:
244,354 bytes, mode0644, observed mtime
`2026-10-03T05:51:05.061711+00:00`, SHA-256
`0b38ca3eb8810a4628436eb5083f6329de848f3f4ddaa972c850601ad45c0835`.
The parsed export has94 rows and exactly one matching X01 UUID.

| Original field | Exact observed value |
| --- | --- |
| `id` | `988578f4-8544-4604-a508-3c3c2e676f8b` |
| `project_id` | `891e7443-8dac-4c1b-897f-9e53cb59c7ee` |
| `title` | `[P3] X01 — Additional SAML profiles` |
| `status` / `worktree_id` | `todo` / `null` |
| `created_at` / `updated_at` | `1790534568` / `1790534568` |
| Workstream / phase / prerequisites | Demand-driven extensions /6 / I04, Q01 |
| Details SHA-256 | `61778cf98fe09f8f237b9eebe0926e4b0b0632d97d3d1ca09bf440324aba0bb5` |
| Whole canonical row SHA-256 | `87fcee902238f81c91c41e101b3919050c117d249751aeb596b0915903a600be` |

Canonical row hashing used UTF-8 JSON, sorted keys, compact separators and
`ensure_ascii=False`, without an added newline. The exact acceptance is:

> Evaluate artifact/SOAP/ECP and encrypted-NameID requirements against named target applications.

The exact goal is:

> These should not delay a complete Essentials experience. Scope them separately when a real integration requires them.

The exact gate is:

> Each extension solves a named requirement, has its own security model, and is tested against a real peer.

The full details also preserve the Essentials/Platform/riauthctl boundary,
shared identity/authorization/revocation/credential protections, actual
implementation/test/doc/artifact evidence as applicable, and the prohibition
on declaring implementation complete from a report alone. The row's
historical scheduling text says leave todo/unassigned and authorizes no
worker launch; this supporting audit uses the present explicit reservation
without changing that row or launching anything. Its recommended medium
effort does not change the running worker's model. RiWork Cua.ai Driver
preference is retained; no desktop use was needed or authorized.

The pinned [actionability record](local-wave30-remaining-task-actionability.json)
X01 entry, lines643–661, repeats the same acceptance, goal, gate and details
hash. It explicitly records no named artifact/SOAP/ECP/encrypted-NameID
target requirement and requires application/profile, security model and real
peer before scoping. Its whole-file SHA-256 is
`38eb08fc718875846414cadc7085c2c2ed3f01b3f5fe5c194caf2331c6e15310`.
Scanning all94 export titles/details for the SAML word found I04 and X01;
that textual scan is not a claim to independently audit every other task.

## Current support, explicit refusals and limits

Contracts were read at the fixed source, not inferred from the worktree or
an advertised version. [SAML](../saml.md), the selected SAML rows in the
[capability matrix](../capability-matrix.md) and
[limitations](../limitations.md) state the browser Redirect/POST boundary.

| X01 facet | Current source-backed behavior | Named demand and security/peer boundary |
| --- | --- | --- |
| Artifact | `wire.rs:413–461` accepts only the selected SAML message, RelayState and, for Redirect, SigAlg/Signature; `SAMLart` is not an accepted field. `receive:678–938` accepts AuthnRequest/LogoutRequest, not ArtifactResolve. IdP metadata advertises only Redirect/POST SSO; source metadata advertises POST ACS. `import_metadata:1001–1118` retains POST ACS, records unsupported ACS bindings in `unresolved`, and explicitly records ArtifactResolutionService as outside the profile. An artifact-only SP cannot supply this importer's required POST ACS. | No named application requiring artifact resolution was found. A reviewed import warning is not artifact transport/resolution support. No artifact lifetime, resolver trust, one-use redemption, resolver network access or real artifact-peer oracle is proposed without the target. |
| SOAP | Platform SSO and SLO POST handlers require exactly one form content type. Routes and the XML root/element allowlists do not expose a SOAP service. Source ACS is a bounded form extractor, with only SAMLResponse/RelayState allowed by the source callback. | No named SOAP role/operation requirement was found. HTTP POST is not SOAP support. There is no source-owned SOAP endpoint or backchannel peer proof in this audit; a later model would have to specify the exact operation and authority rather than relax browser form validation. |
| ECP | Only signed Redirect/POST browser requests and POST responses are advertised. AuthnRequest ProtocolBinding, when present, must be POST. The source generates signed Redirect ForceAuthn requests with POST ACS; there is no PAOS/ECP route, schema option or metadata declaration in the inspected product seams. | No named ECP client/application requirement was found. Browser continuation or configurable workflows do not establish ECP. No enhanced client, relay/endpoint authority or nonbrowser authentication model is invented. |
| Encrypted NameID | IdP issuance writes plain NameID inside the separately signed assertion and can encrypt the **whole assertion** before signing the outer response. Source decryption permits only the configured whole-assertion AES-256-GCM/RSA-OAEP structure, verifies the signed outer response before decrypting, then independently verifies the assertion. Its Subject allows NameID/SubjectConfirmation and requires NameID; EncryptedID is not accepted. Authn requested subject and browser logout likewise require plain NameID under their closed element sets. | No named application requiring element-level NameID encryption was found. Assertion encryption does not claim encrypted-NameID support. A target that accepts encrypted assertions might fit today's profile, but that acceptance cannot be assumed for an unnamed target. No identifier decryption/re-encryption or new key purpose is proposed. |

These are source/contract observations, not newly executed refusal cases.
Optional profiles remain outside the supported product boundary, including
when an underlying dependency might contain a related API.

The inspected [IdP recipe](../recipes/platform-saml-idp.md), lines11–15 and221,
distinguishes its own XML/certificates/ACS and xmlsec1 signer from a deployed
SP. `sp.example.com` is the operator example; `sp.example.test` is a fixture.
Keycloak and Okta in migration inventory are not that fixture or an extension
requirement. The [source recipe](../recipes/platform-saml-source.md), lines17
and118, identifies its in-process Upstream at `enterprise.example.test`;
Okta, Entra, ADFS and Authentik were not connected by that fixture. The
separate Authentik offline export is not an artifact/SOAP/ECP/encrypted-ID
peer. Dated recipe statements that no run occurred during page authorship
are not substituted for the later accepted I04 results below.

## Security behavior that a later single extension must preserve

The existing profile is constrained, rather than a permissive XML or endpoint
adapter. No security change is proposed here.

| Contract | Current witness read and preservation boundary |
| --- | --- |
| Signed browser bindings and pinned trust | `wire.rs:515–677` verifies Redirect over original encoded message/relay/algorithm fields with pinned certs and RSA-SHA256, bounds DEFLATE and rejects trailing data. POST requires one root signature, exact parent reference, enveloped/exclusive transforms and RSA-SHA256/SHA256. XML is bounded48KiB/depth24/nodes2048 with DTD/entity declarations, duplicate IDs and unsupported structure rejected. `receive:678–938` checks registered issuer/destination/ACS, freshness and NameID policy. Any future extension is separately scoped; it cannot become an unsigned browser fallback. |
| Local issuance and one-use browser delivery | `assembly/saml.rs:175–226` verifies the message before recording replay and routes only Authn/Logout. `saml_resume_with:1285–1348` checks the original browser hash, expiry, current client fingerprint/key and deciding session, then issues and removes the request in the transaction. `saml_identity:1349–1390` keeps current authorization, session/assurance/context/scoped claims and exact requested subject. `saml_issue:1392–1552` preserves recipient/audience/request/time/session binding, NameID qualifiers, bounded mapped attributes, signed assertion, optional assertion encryption and signed response. |
| Upstream binding and explicit local linking | Whole `source_saml_runtime.rs` retains exact issuer/destination/InResponseTo/recipient/audience, stable configured NameID and qualifiers, original relay, response+assertion verification, fresh ForceAuthn time and SessionIndex/expiry. Whole `source_finish.rs` rechecks source/account/session/factors and explicit approval; it does not adopt a local account by email or NameID. Whole `management/source_links.rs` retains source/user permission checks, current fingerprint, approval, enabled/admin restrictions and no reassignment to another account. |
| Replay, changed trust and terminal retirement | Whole `source_saml_claim.rs` commits presented changed/disabled/missing-source retirement without rollback revival. Whole `source_saml_record.rs` rechecks the live fingerprint, records assertion replay and seals failure/browser return state in the normal writer. Restoring old pins cannot revive a spent presented login. The source catalog/writer remains permission-protected. |
| Removal and revocation | Whole unlink/pin/write span in `management.rs:2765–2933` requires the owner's fresh local factor session, preserves receipt replay/save, deletes only the owned link, revokes its source sessions and audits. Changed source configuration revokes issued sessions; pin_retired is set only for the pin-write revocations it owns. Browser SLO matches current peer fingerprint/qualifiers/NameID/exact indices and unexpired sessions. Local revocation precedes bounded Redirect/POST propagation; replay/correlation and trust rechecks do not create universal remote session termination. |
| Edition and public transport | Platform route group and feature-gated SAML handlers are unchanged. `edition.rs:455–496` rejects SAML client/source settings on Essentials. Source settings and the SAML model deny unknown fields and expose no optional-profile switch. API source ACS removes its raw browser-return token from the public body and uses the existing cookie handoff. Shared permissions/receipts/credential, optional/required header and PAM contracts are untouched; this is not a fresh audit of every unrelated route. |

If one named requirement arrives, the narrow prospective fixture boundary
is that application's stated role/profile and exact peer build, using its
real protocol implementation rather than relabeling current synthetic XML
or re-signing an old success. The target-specific model must account for
authority/trust/endpoints, correlation/one-use/time, identity/key protection,
withdrawal/removal and bounded failure/cleanup without weakening the above.
Which optional-profile algorithm, route, configuration or fixture to own
cannot be selected until the requirement is known. This is a review
boundary, not a new universal tenant/profile campaign or reserved source seam.

## Accepted peer evidence and immutable role distinctions

The complete274-line public
[I04 root receipt](evidence/wave30-i04-lasso-idp-root-review.json) and
complete281-line [original-scope independent review](local-wave30-i04-original-scope-independent-review.md)
were read. Their histories and scope limits are attributed to their pins;
private artifacts/logs, native libraries and external peers were not reopened.

The native IdP run used source HEAD
`4ed5b494454da33bd27bc046084522c05a55e2a5`, corrected test source
`24c0dc921703432ff04a5cab7cc771c0c9d7df3e`, production
`ae8937800254a1ad4296ea257de1eccc4780e45b`, and the exact
`lasso_idp_redirect_post_source_lifecycle_replays_and_live_trust` filter.
Recorded `2026-10-02T19:50:10.426059+00:00` to
`2026-10-02T19:50:16.472821+00:00`: Cargo0,1passed/0failed,
test2.63s/wrapper6.046559s, owned child group reaped/empty and pipe EOF.
That is a local native macOS ARM64 Lasso peer result, not this audit's run.

Current C IdP mode, lines616–864, and current Rust native-wire/lifecycle
functions, lines512–1070, were read directly as source. The C supplies
authentication/consent for a synthetic principal and uses native FORCE
signature processing, persistent NameID, Redirect request/POST response,
and public SessionIndex fields set before native signing. The Rust verifies
both signatures/parents/pins and native wire recipient/audience/correlation/
times; explicitly asserts absence of EncryptedAssertion and EncryptedID;
then walks explicit link, spent callback/finish, unrelated-owner refusal,
unlink, token revoke, live pin withdrawal, restoration nonrevival and signed
wrong-IdP terminal denial. The designed five native calls reached the
accepted pass at its recorded source. Its empty mappings/groups/ACRs,
unencrypted persistent login-only peer does not test X01's extensions.

| Evidence role | Exact recorded or recomputed identity |
| --- | --- |
| Public I04 root receipt | SHA-256 `61f47b4c7438577c8169b7e16e8b456bbe93c332ca6a83168d8a471f369bf6c6` |
| Public original-scope review | SHA-256 `9303adce61c43b8cb22899db77e20e7ba0b0997d0f07605bd545a51b2e5168f1` |
| Recorded native helper binary | `951465d744c1bf99e7ed91fc414337d00e960c24a1715977cce0e14535794bff` |
| Recorded matching Lasso dylib | `0af7c7ccfda4fe8d20c6ccdf5974a006b2b59a2d50244be95ea197c2d1f72cde` |
| Recorded log / runtime evidence | `f0807e70fc5ab811d18aa2daac6df649be29b47b5f69dc8975834c0958769602` / `316302890bdc93373572c01dcbf5918d0a687b20146a0ba47b7d404f977a0288` |
| Actual test at24c | 39,539 bytes / `b196cba1eea40a67c7d70c6b463ae50ec6897d68c5d81052e33eb3bec2ae5894` |
| Current test at a6 | 39,470 bytes / `610e29c32d7eed456c33493aea2bb1eab52fc7a5791d36db5686c59532fc5512` |

The current test is exactly the actual test minus one69-byte unused POST
constant. Whole-byte deletion equality was independently recomputed; it
does not create a new compilation/native result for that derivative.
Current C SHA matches the recorded C source exactly. Product source,
test/C source, executable, native dependency, workflow configuration, actual
run and published tree remain separate identities. A version such as0.1.1
or Lasso2.9 alone proves none of those equalities or optional-profile support.
No current released asset was downloaded, rehashed, run or credited.

Older accepted SP Redirect→POST Lasso and native SLO results retain pins
`62ce3d83980c52f5242353952a3c4a0c7fdc6b7a`,
`8d389d4ca919918e8a9ddf5f4531effc55fb3757`,
`25cd9b517a932fa72251291433ec9ef6f2ac890d` and
`743dfedfc1c70b0b2b08b0bd4e7318b064eb949e`. They cover their local
persistent browser profile, signature/trust/Conditions rejection and separate
helper Recipient comparison, not a named production application or X01.
The Lasso minimum-signature setter is unexported in the documented bottle;
the recorded helper Recipient check is not mislabeled as a Lasso API check.

Historical CI36950097067/job110661000640 at
`2d05c00deed3a6dafcd458a5f1a1a9beb17d0dd8` records three separate
XMLsec IdP/source/logout filter passes in5.94/5.63/7.78s. XMLsec is the
independent crypto program, not a complete IdP/SP implementation. Test-authored
XML, assertion encryption and source-stage/HTTP bodies retain their specific
scope. The failed aggregate CI36951643190/job110666025654 at
`d31778f066d3bc79f94829cf6295e1eb8df2a84f` retains its individual named
passes and failed aggregate disposition; no all-green or whole-current-a6
runtime inference is made from old CI or workflow text.

Both earlier SLO attempts remain failed: original inner cause unknown, then
native request stage102/exit1 with a stale expected(-111) assertion and later
positives unreached. The IdP ac869/9ec96 attempt remains Cargo101/E0308 at
two temporary-owned Asn1Time borrowing sites, with no native oracles reached;
its recorded log SHA remains
`380ebe9e2f4ef43d173a8e1affa929a77235dd024b125fbe18c7b11e8d5bbe5f`.
Corrected24c pass and later unused-constant cleanup do not retroactively
change those failures. I04's accepted bounded DONE disposition is preserved.

## Read coverage, identity proofs and actual static limits

All product reads came from immutable a6 Git objects. Full original X01
details, `docs/saml.md`, `CONTRIBUTING.md`, `SECURITY.md`, `src/saml.rs`,
`src/saml/wire.rs`, `src/saml/logout.rs`, `source_saml_runtime.rs`,
`source_saml_claim.rs`, `source_saml_record.rs`, `source_finish.rs`,
`source_catalog.rs`, `management/source_links.rs`, `source/saml.rs` and
`source/saml_types.rs` were read in full. Selected complete model SAML
definitions; assembly metadata/start/initiate/resume/identity/issuance/logout
bodies; management unlink/pin/write bodies; API SAML route group/response/
SSO/source-ACS/SLO handlers; edition refusal; recipe requirement/peer spans;
and matrix/limitation rows were read. C lines616–864 and native test
lines512–1070 were source-body reads. Remaining C/test helpers and other
assembly/API bodies were not fully reread; historical full-review coverage
is explicitly delegated to the accepted independent report, not invented
as this turn's full-body read. The large I04 profile-history report was
metadata-only here.

Fifteen SAML/model/assembly/source/link/edition source files compare byte-equal
between a6 and the accepted production ae pin: `src/saml.rs`,
`src/saml/wire.rs`, `src/saml/logout.rs`, `src/source/saml.rs`,
`src/source/saml_types.rs`, `src/model/client_settings.rs`,
`src/assembly/saml.rs`, `src/assembly/saml_logout.rs`,
`src/assembly/source_saml_runtime.rs`, `src/assembly/source_saml_claim.rs`,
`src/assembly/source_saml_record.rs`, `src/assembly/source_finish.rs`,
`src/assembly/source_catalog.rs`, `src/management/source_links.rs` and
`src/edition.rs`. This list is an identity proof; semantic body-read coverage
is specified above. API differs, the
C native IdP addition differs from ae, and the new peer test did not yet
exist at ae. Those differences prevent claiming the whole current tree was
executed by the older native filter. The complete5,488-byte management
unlink/pin/write span also equals ae, SHA-256
`6e3d177327ffc588013bd7308a524529b6fba1f09c4857af177690c2b067d877`.

| Fixed source path | Bytes | SHA-256 |
| --- | ---: | --- |
| `src/saml.rs` | 16922 | `034fe4264f5ce6c50a4038af69144000686eff40bc3f9124cc82cfa86b5fd6b1` |
| `src/saml/wire.rs` | 42227 | `b288c3b1dff5d43b3b80a9d72f8dbbe4117abe9c7272fab35b1ad2628494d756` |
| `src/saml/logout.rs` | 23423 | `a4d62338c5dc4327c9c85e35ecf23d9bb60f86c5246a9c2c723ebaedca81962b` |
| `src/assembly/saml.rs` | 66221 | `614bc71ca3bd99a19a9c46067fba4c450f63871965466cff9558f189673c1892` |
| `src/assembly/source_saml_runtime.rs` | 29138 | `4aaf924ff1b01f4e93e0afdaebe7ede0663868bb74055874ecee9e4cce9edcfe` |
| `src/assembly/source_saml_claim.rs` | 2216 | `31a59561efc3f575ac578c982e623bd858fd28156a13c3ef3c9fd076f0e44c61` |
| `src/assembly/source_saml_record.rs` | 3137 | `f90d1dc22ad1a6030da916ca3bffa8e872bb9b92a37834aefa529884b362a2da` |
| `src/assembly/source_finish.rs` | 11754 | `433a01e0495a1564c8a3e4d422c5b235b4aa0c7b0451a74da4d801e3d7de4497` |
| `src/management/source_links.rs` | 5193 | `7bd06e4e8fba71d7b7b87ea96a87e594e36a754d29a8f6f6151f3ee0e0c6a37a` |
| `src/api.rs` | 147317 | `1bbb387dd627f1fd56b7dfe408ea17eef48131f927597da1b81b21ed5f64b8ca` |
| `scripts/lasso-saml-sp.c` | 34120 | `1b23f51314038ff15caa3aeb8c31acccb8bcd26d6fb702115767d0f68e4dc186` |
| `docs/saml.md` | 24469 | `9184e8f34f7b2ced74fa8ea390ade8ffbba041f80f758cd926481f973c0fce56` |

Actual inspection errors are retained. An initial lookup incorrectly assumed
the old wave28 closure JSON contained X01; zero matches triggered an
assertion/exit1. The exact local export was then used. Guessed paths
`src/api/saml.rs` and `src/model/saml.rs` were absent in a6; the real API
and client-settings paths were identified and read. A later identity utility
incorrectly assumed the new peer test existed at ae; Git128 caused that
utility's exit1. Corrected metadata and the exact unused-constant inverse
completed exit0. Some combined doc/search output truncated; full SAML guide
and selected decisive recipe spans were reread separately. Broader search
results were navigation aids, not claims to read every matching body. These
are static inspection errors, not product, compiler or protocol executions.

## Static checks and root handoff

Actual static checks completed exit0: original export/row and pinned
actionability JSON parse/hash/acceptance checks; the fifteen protected-file
equalities; management-span equality; current C-source identity; and the
whole actual-test-to-current69-byte deletion inverse. No candidate AST,
extension fixture or execution envelope was created or run.

`python3 scripts/check-docs.py` passed before creating this report and with
the new report, checking Markdown links and build-directory layout.
`git diff --check` and `git diff --cached --check` passed.
`python3 scripts/check-repo-hygiene.py` passed with1,064 indexed files.
Both checker source files compare byte-equal to the fixed a6 objects:
SHA-256 `925418bb155b59554666603efce4af16160ee5e00df9c409d46a3381b390ae82`
and `a3b6855473e0d68308d0f7691bb2ccff766b49f21abd68d6c766e75725d31ece`.
Report UTF-8/newline/NUL/trailing-whitespace and exact source/task/project
pins passed. Staged scope is exactly this one new path; no other tracked
or untracked change exists. Final immutable report hash/commit and clean
scope are returned separately to root after the final static checks.

Only one new reserved report is to be committed. No source/test/helper/
workflow/existing doc was edited, no merge/alignment or worker/peer contact
occurred, and no assignment/status/main/push changed. No private files or
cleanup resources were opened or removed. No runtime, native/library/helper,
compiler/Cargo/version, provider/service/HTTP/network query/download/dispatch,
browser/Driver or desktop was invoked. D01 memory owns the validation lane;
this audit acquired/released no slot. Historical failures and source/artifact/
workflow distinctions remain intact. Root alone reviews, integrates, reserves
any follow-up and decides original statuses. The one next input remains the
named requirement record above; there is no source ownership request until
that record selects a real requirement and peer.
