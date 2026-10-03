# Wave 30 X02 named network and directory extension audit

Date: 2026-10-03. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Original X02 task: `2175d198-8798-466a-8a46-867693552a0a`.
Reservation: `wave30_X02_named_network_directory_extension_audit`.
Supporting WT: `e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`, branch
`roadmap/local-extension-isolation-wave27`, starting clean HEAD
`98b956552a9aa4a3fedb6ec3be0910335dfae5e6`.
Fixed published source: `1a517a1a461b7017c353d37a5e498d2c7cfa7985`.
Only this new report changes; no alignment or implementation is proposed.

## Recommendation and exact missing input

**Keep X02 demand-driven and todo/unassigned pending one named integration
requirement record.** No such record was found in the task export and bounded
repository requirement/contract corpus described below. The requested extensions
are explicitly outside the present RADIUS/LDAP-provider profiles, or outside
the supported SCIM schemas. Existing core SCIM writes and bounded sorting are
already implemented; treating them as missing X02 work would misstate the source.
No concrete defect requiring an immediate product correction was identified in
the inspected extension boundaries.

The single proposed next input is a record selecting **one** extension and
direction, naming the actual peer product/version and exact required operation
or schema, explaining why the present profile cannot satisfy the integration,
and making that peer available for an authorized, isolated interoperability
check. Its security model must settle the questions in the matrix before source
ownership or runtime is reserved. There is no defensible extension implementation
or executable peer command to select from an absent requirement. This input
must not become a gate on the complete Essentials experience.

This report evaluates the original row; it does not satisfy its real-peer
extension gate or recommend DONE. Root alone decides scope, assignments,
integration, publication and status. I04, P06, Q01 and other accepted closed
rows are not reopened.

## Original row and export provenance

The entire exact-UUID row was read from the project planning export:

`/Users/dominik/.local/share/riwork/orchestrators/projects/891e7443-8dac-4c1b-897f-9e53cb59c7ee/planning/current-tasks.json`

The read snapshot is 244,347 bytes, SHA-256
`5cfd882213745a86dcdeeb7e4debccf24fd22299c0fc7be73d6d988512739752`,
filesystem mtime `2026-10-03T07:27:51.148464+00:00`. It contains 94 rows.
This is a dated local export, not a fresh live RiWork API observation.
X02 is `[P3] X02 — Additional network and directory functionality`,
`status: todo`, `worktree_id: null`, with prerequisites I04, P06 and Q01.
All three prerequisite rows are `done` in this same export.

Exact requested outcome:

> Evaluate RADIUS accounting, CoA, PEAP/TTLS, and additional LDAP/SCIM write or schema behavior.

Exact workstream goal:

> These should not delay a complete Essentials experience. Scope them separately when a real integration requires them.

Exact completion gate:

> Each extension solves a named requirement, has its own security model, and is tested against a real peer.

The full row also requires shared identity, authorization, revocation and
credential-protection semantics, actual relevant implementation/tests/docs/
artifact evidence, and explicit gaps/external prerequisites. Its dated
leave-todo scheduling paragraph is not a new restriction on this authorized
audit; it does not supply a named integration or authorize implementation.
The medium-effort planning recommendation was not used to change the current
model. No task/export/assignment/status was written.

## Requirement search and current contract reconciliation

The search covered the exact task export, fixed Git documentation, the X02
record in `coverage-inventory.json`, the excluded-profile entries in
`capability-matrix.json`, corresponding Markdown inventory/contracts,
capability/limitations pages, protocol guides and relevant recipes. Searches
also inspected matching passages in README, Essentials/Platform guides,
lifecycle, reviewed-client-creation and D01/D05 guide/evidence records. Large
unrelated report histories were not treated as a new customer requirement
database. No moving worker files or private correspondence were inspected.

| Record | Finding |
| --- | --- |
| Current task export | A word-delimited search for the requested RADIUS extensions/additional LDAP write or enterprise-schema terms matches X02 itself; no separate named extension request appears in those task details. |
| [Coverage inventory](coverage-inventory.md), X02 / JSON row | Unsupported RADIUS extensions, read-only LDAP provider, no SCIM enterprise schema; `gaps: ["Named target required"]`, no existing X02 tests. This is a roadmap snapshot, not current runtime. |
| [Product contracts](product-contracts.md):149–150 | Initial exclusions include LDAP writes/additional schemas, RADIUS accounting/CoA/PEAP/TTLS and SCIM bulk/sort/nested/custom enterprise schemas. The historical sort exclusion is superseded by current advertised sorting. No guide correction is owned here. |
| [Current capability matrix](../capability-matrix.md):211–214 and [limitations](../limitations.md):11–13 | Current provider/profile boundaries and attributed local peers. These describe supported behavior and missing peer coverage, not a demand for a specific extra capability. |
| [Inbound SCIM recipe](../recipes/platform-inbound-scim.md) | Explicitly no named SCIM client; the HTTP peer is the in-process router. Okta/Entra are examples of clients not connected, not selected requirements. |
| LDAP/SCIM examples and recipes | AD attribute examples and `payroll`, `staff`, example domains/addresses, fixture agents and synthetic identities are configuration examples or test inputs. They do not establish an operator/customer request or real deployment profile. |

An initial broad text search also matched `TTLS` inside `STARTTLS`; subsequent
word-delimited RADIUS extension searches separated those unrelated matches.
No claim is made that unavailable external requirement records do not exist.
No remote query, user question or other-worker contact was made.

## Direction-by-direction source and security boundary

Paths/line numbers in this section refer to the fixed published source above.
The matrix identifies requirements to settle for a later selected extension;
it does not approve those designs or introduce new capabilities.

| Facet and direction | Actual current source/API/fixture behavior | Unsupported extension and exact needed peer/security input |
| --- | --- | --- |
| RADIUS accounting: NAS → riAuth | `src/radius.rs::decode`225–270 accepts only packet code1, 20–4096 bytes, correct length and mandatory authenticated Message-Authenticator. `radius_packet`349–445 handles access authentication/replies. Accepted PAP/Access-Request evidence does not exercise accounting. | No accounting receiver/acknowledgement ledger is provided by this path. Name the NAS/version and Start/Interim/Stop profile, required attributes and purpose. Define authenticated NAS/transport custody, stable session/user correlation, duplicates/out-of-order/restarts, retention/privacy and what an accounting event can change. A network usage event must not mint or adopt identity or grant access. |
| Dynamic authorization/CoA: direction not yet selected | The same Access-Request-only parser is not a CoA/Disconnect listener. `docs/radius.md:58` explicitly excludes dynamic authorization and active NAS session termination. Online policy/revocation checks govern subsequent riAuth authentication; the NAS owns its established session and reply enforcement. | First specify riAuth→NAS sender versus NAS→riAuth receiver, exact NAS/version, CoA versus Disconnect and required session selectors/replies. Define authority to request the change, authenticated peer binding, immutable local/NAS session correlation, retries/acknowledgements and honest uncertain/partial effects. Local disable is not proof that an established network session ended. No new termination promise follows from an existing Access-Reject. |
| PEAP: supplicant → riAuth through NAS | EAP configuration/engine implements EAP-TLS; `Conversation::request` emits method13 and `Conversation::step` requires method13. Existing PAP and EAP-TLS certificate/MFA evidence is not a tunneled-password-method pass. | Name supplicant/OS/version, NAS and exact PEAP version/inner method, plus CA/server-name validation and any required credential storage. Decide inner/outer identity binding, credential exposure, crypto-binding/downgrade/refusal, live group/MFA/device policy and replay/key delivery. MS-CHAP is presently unsupported; do not infer it from PAP or TLS support. |
| TTLS: supplicant → riAuth through NAS | No TTLS dispatcher, tunnel or inner authentication profile exists in the inspected method13 engine. The PAP branch rejects CHAP/State and is not an inner TTLS implementation. | Name supplicant/version, TTLS version and exact inner method/attribute mapping. Set verified tunnel trust, inner account binding and factor/password semantics, keying, resumption/downgrade and live revocation. Neither PEAP nor TTLS should silently substitute a weaker assurance claim or optional factor fallback. |
| LDAP provider writes: application → riAuth (Platform) | `src/ldap_server.rs:623–673` returns `UnwillingToPerform` for Modify/Add/Delete/ModifyDN/Compare and `UnavailableCriticalExtension` for other extended requests. There is no provider write API in those branches. Read-only bind/search/page/WhoAmI behavior uses live authority. | Name the application/version, exact write or extended operation, object/attribute set and conflict/precondition behavior. Specify scoped authorization, immutable user/DN ownership, administrator/last-admin protection, rename/deletion/password semantics, atomic validation/audit/revision/replay effects and protocol refusal. Do not enable all LDAP writes merely because one client needs one operation. |
| LDAP provider schema: riAuth → application search view | `src/assembly/ldap_server.rs:650–739` projects `inetOrgPerson` fields/entryUUID and visible `groupOfNames` membership. Email/groups depend on client scopes; user searches are self-bound and agent searches require `ldap.search` plus configured visibility. It advertises neither POSIX nor AD schema emulation. | Name the consuming application/version, exact objectClass/attribute/filter/matching-rule requirements and expected values. Decide identity mapping, uniqueness/rename stability, disclosure scopes and durable versus effective membership. Do not expose private factors/credentials or turn temporary PAM access into directory membership. A mapping example is not AD compatibility proof. |
| LDAP import/upstream writes: riAuth → directory (Essentials and Platform) | `src/directory.rs` connects/binds/searches; `src/assembly/directory.rs` owns reviewed local reconcile/apply and external user-password bind. Configured stable ID/mapping/filter reads already exist. Normal apply changes riAuth records, not upstream LDAP. Accepted OpenLDAP fixture population writes are setup by the test client, not an outbound riAuth writer. | No upstream LDAP add/modify/delete/password-write contract is selected. Name the source directory/version and one exact operation; define target ownership, service authority and secret custody, conditional identity binding, directory overlays/password policy and partial/uncertain remote outcomes. Extra import mappings must be distinguished from a new served schema or write direction. |
| Inbound SCIM writes/schema: client → riAuth (Platform) | `src/api.rs:674–715,1207–1345` wires core User/Group CRUD, list/.search and metadata. `scim_runtime::scim_write`/`scim_delete` use the checked mutation envelope, owned records and shared management writers. Exactly one core schema URN is required; unknown top-level fields reject. Group member values resolve to owned Users. Core name/email/member PATCH, projection and bounded sorts already exist. | Bulk, nested groups and enterprise/custom resource schemas are not advertised. Choose one named client/version and exact URN/attribute/operation, including required mutability/return/filter/sort behavior. Define type/size/duplicate/path validation, ownership, administrators/credential exposure, tombstones, resource versions, atomic multi-operation rollback and live permission/receipt behavior. Preserved unadvertised nested name/email extras are not generic schema support. |
| Outbound SCIM write/schema: riAuth → target (Essentials and Platform) | `src/provisioning.rs:1560–1692,1724–1749` looks up exact issuer-namespaced externalId, binds prior remote ID, GETs current resource, uses conditional managed-field PATCH and authenticated POST. The guide defines reviewed deactivation rather than remote DELETE, and explicit complete managed group-removal checks. No automatic username/email adoption or remote account deletion is implied. | Name the target/version and the specific missing field, schema, write or lookup convention it requires. Fix ownership, exact stable identity, peer ETag/conditional semantics and unmanaged-field preservation before extending dispatch. Any destructive extra operation needs explicit reviewed intent, bounded response validation and observable uncertainty; a queued job/accepted operator resolution is not remote success. |

The existing SCIM resource-ETag policy is route-specific: agent HTTP
PUT/PATCH/DELETE require the quoted resource version; POST creates without it.
`scim_precondition` checks a supplied request context, while context-free Core
calls are not HTTP-header evidence. Management numeric revision and receipt
policies remain distinct. No universal required-header rule is proposed.

## Accepted evidence and what it cannot establish

The published [I04 independent disposition](local-wave30-i04-original-scope-independent-review.md)
was read through its original profile/historical-execution mapping (lines1–195).
Its body review and accepted result attribution are reused only with the exact
blob comparisons below. The original I04 DONE status is accepted; this review
does not repeat its SAML audit or establish a new current-runtime result.

| Accepted evidence | Actual attributed coverage | Boundary for X02 |
| --- | --- | --- |
| FreeRADIUS radclient3.2.10; accepted `38f82fe7fce02c0bd70db4a4cd22bb15edcea7a6`, source `c51a0ef9bdb1dc03eddee5244871345c1f5a7456` | `radclient_pap_accept_reject_and_cached_replay`: correct PAP Access-Accept/rc0, wrong password Access-Reject/rc1 and exact captured datagram replay without another accept audit. Whole current244-line fixture read. | A real executable client, not a FreeRADIUS server, hardware NAS, accounting/CoA implementation or PEAP/TTLS supplicant. Source linkage of the historical log remains accepted ledger attribution. |
| OpenLDAP ldapsearch2.7.1; accepted `fcba8a5e81037edb365c02c291b512576817e920`, source `d00437fc534faef0b1e2a0bb07a9891d1f018083` | Accepted LDAPS/STARTTLS scoped bind/paging, disabled omission, wrong/revoked agent rc49 and wrong CA/crossed transport refusal. Current fixture/guide identities checked. | Read-only riAuth provider peer evidence, not LDAP writes, AD/POSIX emulation or an upstream directory implementation. |
| CI36950097067/job110661000640 at `2d05c00deed3a6dafcd458a5f1a1a9beb17d0dd8` | Accepted real OpenLDAP slapd STARTTLS import named1/0:205 entries, entryUUID mapping, rename/factor retention, user-password/MFA/recovery, stale/partial/collision/refusal and reviewed removal/session invalidation. | Disposable actual directory server. Import LDAPS/AD overlays/objectGUID combinations were not all executed; no upstream write profile is proven. |
| Failed aggregate CI36951643190/job110666025654 at `d31778f066d3bc79f94829cf6295e1eb8df2a84f` | Accepted named LDAP socket, PAP/RadSec and OpenSSL EAP-TLS passes. EAP method13 uses actual TLS1.3/optional1.2, explicit certificate binding, fragmentation/key checks and live group/binding/CRL revocation. | Aggregate remained failed. OpenSSL is the independent TLS engine in test-owned RADIUS/EAP framing; no real hardware/supplicant, PEAP/TTLS or EAP-over-RadSec universal pass follows. |
| `tests/identity/http.rs:245–400`, complete current named SCIM function read | Router oneshot creation/exact receipt retry, missing bearer/media type, ownership, filtered Core read, invalid atomic PATCH preserving session/group, owned member removal, administrator refusal and delete invalidation are source assertions. | Definition read, no new execution; the router is not a named directory product. Later operations in this function are direct Core calls, not all HTTP methods tested. |
| [SCIM OAuth root review](local-wave30-scim-oauth-i02-root-review.md):277–315, actual `2414b5d83bc1ebbe31ffc99684ef7e4896e9d1d1`, fixture `199980044994d74adcb429bed70ff386399445d8` | Accepted one whole-target Darwin EXIT0/24passed, compile4.22s/test29.23s/wrapper35.137593s, reaped group84412, minimum13,683,351,552B. Local token/SCIM HTTP servers are Rust test fixtures. | Real HTTP transport to synthetic loopback peer implementations, not a named SaaS/directory product or new schema interoperability. Prior16/8,22/2 and separate Linux23/1 failures in the report stay historical failures; local24pass is not whole-CI/current-Linux proof. |

No raw historical log, native artifact, private protocol input, binary, archive
or peer was reopened/executed by this audit. Historical log hashes/results above
are source-report/accepted-ledger attribution, not new log verification.

## Exact reuse and immutable read scope

Current whole-file Git blob equality to I04's fixed
`69f46cf75390cde70a992eb528c7ee5f769aeeca` was proved for the four protocol
guides, `src/radius.rs`, `src/radius/eap.rs`, `src/ldap_server.rs`, the three
standalone LDAP/RADIUS fixtures, `tests/identity/network.rs`,
`tests/identity/radius_eap.rs`, `src/assembly/scim_runtime.rs` and both Q01
security documents. Equality permits reuse of those established body reviews;
it does not make their historical executions current whole-source runs.

Direct whole-byte fixture equality to accepted execution/source pins was also
proved independently:

| Current fixture | Prior pin | Bytes / current SHA-256 |
| --- | --- | --- |
| `tests/radius_peer.rs` | `c51a0ef9bdb1dc03eddee5244871345c1f5a7456` | 8109 / `7c2176d15a013fd0b044d9124632522a7d37f4e729c212816726b5f1aa06095f` |
| `tests/ldap_provider_peer.rs` | `d00437fc534faef0b1e2a0bb07a9891d1f018083` | 17169 / `3269488d42ce482e03d7f2b5134305d59408a990ac4565e16b5779dfd89cd515` |
| `tests/ldap.rs` | `2d05c00deed3a6dafcd458a5f1a1a9beb17d0dd8` | 11895 / `3edd73958eabd10db3e000723c5b8f8bd4ad6f387cff8dd57d281177fcd9dcee` |
| `tests/identity/network.rs` | `d31778f066d3bc79f94829cf6295e1eb8df2a84f` | 36336 / `9f3b7e516401bfd1776073129d2b03f1884e3f7545a573003bf485147105d68e` |
| `tests/identity/radius_eap.rs` | same `d31778f…` | 55555 / `6f9d8bfdb0b63ec37692261aa6ccb9393b0fb69e87273b0556b683a774c69391` |
| `tests/scim_oauth.rs` | `199980044994d74adcb429bed70ff386399445d8` | 95086 / `ee69d6311be8bcbe697a902586acb070dfc55a7769758c7ccdd3b41f8f30b218` |

New body reads were the complete RADIUS/LDAP-import/LDAP-provider guides and
RADIUS standalone fixture; SCIM guide1–115; RADIUS parser/PAP/EAP completion
219–473; EAP trust/material1–180 and message/step250–395; LDAP refusal588–699;
LDAP assembly authority306–340, search428–473 and projection650–739;
SCIM metadata1–108, ownership109–155, normalization243–306, precondition1215–1257,
write1415–1515/1590–1755 and delete1755–1805; API route674–717 and adapter1207–1348;
directory apply entry449–481 and password bind624–715; provisioning dispatch
1560–1692/1724–1749; SCIM HTTP fixture245–400 and OAuth fixture1–133.
Function/path inventories supplement those spans, not full-module review.

Relevant Q01 body rereads covered threat-model37–104 and
invariants320–372/511–598; removal safeguard quotas/controller material and
the complete selected removal-policy/limits span186–268 were inspected.
Large initial aggregate outputs truncated; bounded subsequent reads supplied
the relevant spans. No claim is made to have newly read all1,330 invariant
lines, all426 SCIM-guide lines, the entire97,835-byte SCIM runtime or unrelated
desired-state paragraphs. The A03 SCIM move report was read in full for layout
provenance, not substituted for P06 acceptance or peer execution.

| Other selected fixed object | Bytes / SHA-256 |
| --- | --- |
| `src/assembly/scim_runtime.rs` | 97835 / `9cd66862c726c11db78a44a714a1b8ae0c9a1ef2012e82f1a802718170a43c55` |
| I04 independent disposition | 26525 / `9303adce61c43b8cb22899db77e20e7ba0b0997d0f07605bd545a51b2e5168f1` |
| SCIM OAuth/I02 root review | 45189 / `40640b963dbca84e9fb108546032919f857548e818b8cc5d5eb304128eaed952` |
| `docs/removal-safeguards.md` | 18991 / `a8dbccb98b8c0e4956c51a2a781e17cbe1f5c059c52eea489ac06a0250018da8` |

Two read-only guessed-path lookups (`src/api/scim.rs` and
`src/assembly/provisioning.rs`) failed because those paths do not exist in the
pin. The actual `src/api.rs` and `src/provisioning.rs` locations were then
read; neither lookup changed files or supplied evidence.

## One later requirement record, with preserved security and peer gate

Root needs one record containing the following concrete inputs before choosing
a source or test slice:

1. Named integration owner/use case and actual peer product, version and role;
   exactly one requested direction/operation/schema/profile from the matrix.
2. The failing required behavior or configuration, exact expected fields and
   outcome, and why current supported PAP/EAP-TLS, LDAP reads/import or SCIM
   core writes cannot satisfy it. Include a sanitized wire/schema example only
   when available; no customer or failure is invented here.
3. Authenticated peer/transport trust and private credential custody; immutable
   identity/session/resource binding; scoped live authority, assurance and
   revocation; exact mutation/retry/audit/removal and remote uncertainty model.
4. Actual real-peer access/provenance and a permitted isolated fixture. Define
   one positive selected behavior and its necessary authentication, wrong-target,
   replay, permission/revocation and malformed/partial-input refusals, with local
   state/audit comparisons and separate remote observations where relevant.

The existing [shared invariants](../security/invariants.md) and
[removal safeguards](../removal-safeguards.md) remain the foundation, not evidence
that an extra protocol is secure. Complete snapshots, stable ownership and exact
review commitments cannot be replaced by successful connectivity. Partial data
cannot authorize removal; temporary/effective membership stays distinct from
durable directory membership. Local retirement, queued work, operator attestation
and actual verified peer delivery remain separate outcomes.

Accepted credential-issuance receipt behavior, route-specific optional/required
headers, PAM fallback, Group ownership/elevation safeguards and existing shared
60-second boundaries remain unchanged. A future implementation must use the
shared writers and transaction/audit contracts rather than introduce a second
identity or permissive protocol-specific writer. Unknown/unselected required
features continue to refuse; neither a library's capability nor one passing
fixture promises generic NAS/directory/SCIM compatibility.

No prospective runtime command, resource allocation or capacity estimate is
invented before selecting that peer/profile. The exact command, finite bounds,
private input/output and owned cleanup would require a separate concrete source
and runtime reservation after the record exists. No mandatory all-peer campaign,
hardware/tenant study or release gate is added to Essentials by this audit.

## Static checks and handoff

Actual checks for this slice are read-only Git object/path/function inventories,
full original-row/prerequisite extraction, fixed snapshot hashes, the scoped
requirement searches, whole-byte comparisons above, new-report local-link and
text-hygiene checks, whitespace and sole-file diff/staging checks. There is no
new protocol runtime, test, compiler/typecheck, service, network, desktop,
artifact or performance evidence. No runtime/Cargo/desktop slot was acquired or
released. Repository guidance was read; no applicable AGENTS file was found.

Protected prior report bytes are retained: A09 original review39894B/SHA-256
`f436963da68568b8979e1a5bc4d5c3618ee425af598882ebcd93026236b6acfc`,
Q08 plan24056B/`7eb7d9bc655580de20bb6a1e88fe59734a96a4ec3dbb6c0f5e47bb7004c3ac1d`,
Q11 plan27685B/`8b87b1a906cb48f8b341b8577236c86a0ddda7f33bfad550b0b02d3d6bd9353c`,
S02 plan283833B/`3f8929fc6f5bbdd7f45c4af80bf4319ec8715fb07f7ae0e874e1080d753f9921`.
Their passes, failures, limits and root-owned dispositions remain intact.

Explicit-project handoff: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, original X02
`2175d198-8798-466a-8a46-867693552a0a`. Accept this bounded evaluation after
review; retain the original todo/unassigned state pending the one missing named
requirement record and its independent security/real-peer evidence. This report
does not implement or close X02 and changes no accepted code or guide.
