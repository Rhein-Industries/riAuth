# Wave29 A03 original acceptance reconciliation

Recommend **in_progress** for A03. Accepted identity/storage ports, shared
management writers, protocol runtime moves and the independent remote client
are substantial implementation evidence. Accepted A08 evidence also establishes
representative distribution-switch preservation and fail-closed refusal on
native source-built binaries. Nevertheless, `src/registration.rs` still combines
RFC 7591 contracts with Core/concrete-transaction runtime responsibilities.
This is a concrete local boundary gap, independent of official release gates.
The one proposed next slice below is not implemented or claimed by this audit.

## Scope and immutable basis

- Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
- Existing A03 task: `4467345d-7a4f-4bb9-bd5a-586c775b0b44`, title
  `[P0] A03 — Establish real module boundaries`; observed status `in_progress`.
- Existing worktree: `42bb51c6-c198-4adb-bd92-0a5222853231`, branch
  `roadmap/local-module-boundaries-wave27`.
- Audited accepted main: `f0a9edb3275234b891dbe312bf3ce26265587ea9`;
  tree `87985b0d0c4170262abc9710d329db74bf9e15ac`.
- Own pre-audit HEAD: `af4fd9e253b82c3094b21114f623952d968cf377`.
  History is retained. Main was read through immutable Git objects, not merged
  into or substituted for this worktree's product files during this docs-only
  assignment. Every source location below refers to the audited main pin.

The RiWork original task and A08 record were read with the explicit project ID.
Repository/ancestor guidance was inspected: no applicable `AGENTS.md` was found;
`CONTRIBUTING.md` and `SECURITY.md` apply. The current user assignment restricts
this work to read-only reconciliation and this single report, so its no-build/
no-test scope supersedes the contributor guide's general verification campaign.
No board status, main, product file, old source worktree or backup was changed.

## Original task wording

The following acceptance text is verbatim from the assigned RiWork task:

> Requested outcome and acceptance scope:
> Separate identity core, management, storage, protocol adapters, connectors, API types, server assembly, and client responsibilities.
>
> Workstream goal: Make riAuth modular without creating two identity implementations.
> Workstream completion gate: Switching distributions does not create different users, different authorization rules, or silently ignored configuration.
>
> Product boundaries:
> - riAuth Essentials: OIDC/OAuth, passkeys, complete browser self-service, compact administration, groups and claims, API/CLI, audit, backups, LDAP import, and outbound SCIM.
> - riAuth Platform: Essentials plus configurable workflows, broader protocols, advanced federation, cloud connectors, device integrations, and expanded administration.
> - riauthctl: Remote administration, with optional terminal USB-authenticator support separated from the server.
> Both server builds retain the same identity, authorization, revocation, and credential-protection semantics for shared capabilities.
>
> Completion evidence: Review the relevant implementation, tests, documentation, and released artifacts as applicable. Report verification actually performed, remaining gaps, and external prerequisites. Do not mark implementation complete from documentation or a worker report alone.

The task names the workstream `Architecture and separate builds` and proposed
prerequisites A01, A02 and Q01. Its original scheduling instruction to leave it
todo/unassigned predates the explicit continuation assignment and current
assigned/in_progress state. This audit neither invokes that stale scheduling
instruction nor changes the board. The outcome requires responsibility
separation; it does not explicitly require eight independent Cargo crates.
Intra-crate ports can provide real separation, while one crate or a low reference
count alone proves neither completion nor failure.

## Accepted implementation pins

These are accepted-main history pins, distinct from worker-source hashes that
were integrated by cherry-pick. Worker commits are provenance, not asserted main
ancestors.

| Accepted commit | Relevant implementation |
| --- | --- |
| `30ed02fa1e638b68f188d48ad0e010a6e7b34bcb` | Shared identity transaction/module seams |
| `75d3352e829e4593225d5f976fa8e2541d8c0bb9` | Identity hooks assembled outside storage |
| `6796ff4036ce04b044e49205b25a6a9d4dd8a809` | Cloud reconciliation storage assembly |
| `b83bed418fdd5af52449749c5c6c6fdc63b52894`, `19b4ed8dccd293d504230527f01125b443e4cc44` | Cloud Core runtime and operations assembly |
| `9372d55354fd0669927977613a6079a25fb5b29b` | Browser runtime assembly |
| `edab152da75a501d0253948797e17dcf547674c7` | Portal runtime assembly |
| `5087d317fc0dd39fce2a88785d774633707bbda1` | Inbound SCIM runtime/helpers assembly |
| `d3d255b913edfc6839c5ea3fe90ddf801cf784a3` | Protected connector definitions and credential-origin controls |
| `748874e6aee90ecdf38e655d3eea9004cb9319bf`, `a7e44d97207d3e5c5f1f4dbb0fb2f4ebfca236be` | Complete source runtime/private verifier move and evidence report |
| `f5706d9087327018311779dfdbb7fadf2f25b120` | Shared transactional workflow activation/live replay hook |
| `f984a84af950d495642edd3f92508c86e10b0cd7` | Protected standalone-client operations/backup/SSF port |
| `92e0777b4ff663e664ac2fd223432904fdb60eec`, `bb3cefa14aa7a9ca0f5b2a7767e82a4fa65798d8` | Format-3 effective-rate agreement and unsupported-future-format fixture correction |

For source, this audit compared Git blobs against worker implementation
`691ae65bc5e1edd2b1b0e182ab31b431d9e43e9a` and report
`af4fd9e253b82c3094b21114f623952d968cf377`: all eight implementation files and
the report equal their accepted-main blobs. These are `src/source.rs`,
`src/source/{saml,saml_essentials}.rs`, the three new `src/assembly/source_*runtime.rs`
files, `src/assembly.rs` and `scripts/check-module-boundaries.py`. Root separately
reported matching all 54 original function bodies plus canonical/private/cfg/
compatibility wiring. That independent review is credited as prior evidence;
this turn did not rerun its body-comparison checks.

## Responsibility inventory against the full outcome

| Responsibility | Concrete code inspected at the main pin | Accepted evidence and disposition |
| --- | --- | --- |
| Identity core | `src/model.rs` owns User, Group, Client, Identity, Session and Grant records; `src/identity.rs:35`/`:46` enforce account epoch/liveness and session ownership/revocation; `identity/persistence.rs:9` defines `IdentityTx`. Shared durable effects reside in identity's agent/windows credentials, logout queue, signals, downstream and password-history modules. | One shared model/policy implementation is used by both editions. `core.rs:1058`/`:1064` compose protocol proofs before the same identity checks. Identity effects run through a port, with no direct concrete storage/Core import in `identity.rs`. `tests/identity_boundary.rs` and `identity_transition_boundary.rs` contain transaction/epoch/revocation contracts; their presence is inspected, not a new execution claim. |
| Management | `src/management.rs` and children own shared user/group/client/source, credential issuance, registration, consent/device/session, reviewed grants/memberships and Platform SSF/PAM writers. `state.rs:2058` onward invokes the same writers as direct mutations. `core.rs:69` supplies live principal, receipt replay, precondition and write envelope. | Writer ownership is real, rather than duplicated HTTP/CLI/plan policy. The service still imports Core policy/audit helpers and concrete `Tx` (`management.rs:75`–`:96`), and some transport/runtime orchestration remains in resource modules. This is partial responsibility separation, not an independently compiled management layer or a full M03 parity claim. Live-authority-before-replay and first-response credential protections remain visible in accepted code. |
| Storage | `src/store.rs:43` defines injected `RecordTransitions`; Store/Tx, prepared writes, ownership and maintenance own backend/transaction machinery; `postgres_store.rs` supplies the other backend. `assembly.rs:195` supplies identity/security hooks and `:271` maps `IdentityTx` to the existing Tx methods. | Storage does not implement a second identity policy. Epoch normalization, identity effects, SCIM effects and the disabled-account workflow seal are assembled in the caller's transaction. Writer/preview/prepared ordering, indexes, encrypted record IO and optimistic revalidation remain concrete storage responsibilities. Historical A08 backend evidence below supports the shared record contract. Storage is an internal module boundary, not a separate crate. |
| Protocol adapters | `oidc.rs:110` exposes `OidcTx`; authorization, assertion/DPoP, password and other protocols have scoped ports. Browser/portal/SCIM/source Core and concrete-Tx implementations now reside in assembly. Canonical protocol records, validation/data and compatibility paths stay in their original modules; `source.rs:8`/`:10`/`:170` onward preserve the old aliases. | Accepted cloud/browser/portal/SCIM/source reports record body/constructor/order equivalence and their actual narrow checks. The source report records Platform/Essentials library checks, not runtime distribution tests. Registration remains a concrete runtime/contract mixture described below. Compatibility re-exports still create transitive assembly dependencies; zero direct counted edges is not an independent-compilation or acyclic-graph proof. |
| Connectors | `directory.rs`, `provisioning.rs`, `cloud_directory.rs`/shared cloud types, `reconciliation.rs`, `connector_guard.rs` and `connector_definitions.rs` divide peer settings/IO, plans/jobs, review/pagination/lease gates and durable definitions. Protected definitions merge read-only in `core.rs:227`; source identities/links remain canonical source/model records. | Accepted connector port preserves first-owner credential bindings, origin pins, full-human desired-state authorization, digest-only audit, retirement and edition rejection. Provisioning/reconciliation/definition runtime still imports Core/concrete Tx; e.g. `provisioning.rs:124`/`:694`, `reconciliation.rs:13`/`:16`. These are visible assembly/service seams, not hidden by the protocol count. No real-cloud, Group materialization or connector behavior completion is inferred. Those changes remain with their owners. |
| API types | `model.rs:87`–`:126` and `:361` onward contain shared management inputs/bindings; `model/{client_settings,client_config,credential,federation,jwk,exchange,claims,assurance}.rs` hold canonical data types. Protocols own requests such as `RegistrationRequest`. API handlers import these types (`api.rs:26`) and define private transport inputs such as Login (`:1709`) and PasswordChange (`:2283`). | Canonical type extractions and old public aliases prevent duplicate server identity definitions. API shapes are partly shared data and partly HTTP-local DTOs; the independent client uses wire JSON/local DTOs rather than linking server storage. This is not a separately compiled API-types package. Private HTTP-only DTOs are not, by themselves, proof of duplicated policy, but a complete API-type boundary/reuse claim cannot be derived from the protocol graph. No new schema or compatibility checks were run here. |
| Server assembly | `src/assembly.rs` and protocol-specific children own concrete Core impls, Tx-port implementations, Store construction and security hooks. `core.rs` still owns config/store instance, startup gates, shared orchestration, management envelope and policy helpers; API/server/background modules assemble listeners and worker permits/loops. | The moved method/helper bodies actually reside here; they are not forwarding to their former protocol implementations. Config/feature gates and compatibility paths remain. Assembly intentionally knows concrete storage and protocol ports. Remaining registration/service orchestration prevents a claim that every assembly responsibility is separated. Recent shared workflow activation/source factories are accepted dependencies, not targets of this audit. |
| Standalone client | `crates/riauthctl/Cargo.toml` is its own Cargo workspace, has no `riauth`/storage/config dependency, defaults to no features and makes `terminal-usb` optional. `transport.rs:104`/`:143` onward binds discovery/issuer and saved credentials; management/operations modules call HTTP routes. The server Cargo manifest has no terminal USB feature. | Structural client separation is established in code. Historical native dependency closures explicitly excluded server/store/USB from the base client and USB from both servers. The protected operations port is accepted on current main. Complete GUI/CLI/API parity remains M03 acceptance; official shipped client/server artifacts remain the release lanes' gate. Neither limitation requires a second identity implementation. |

The inventory includes server modules excluded from the checker's `PROTOCOL`
set. In particular `registration` and `provisioning` are classified management;
`management`, `workflow`, `reconciliation` and connector definitions fall through
to shared support unless otherwise listed. Therefore graph classification is
not a comprehensive architectural ownership decision.

## Distribution-switch/security gate mapping

| Original gate | Current accepted implementation | Existing measured evidence and limit |
| --- | --- | --- |
| Same users/identifiers | One model/store format; `edition/transition.rs:207` locks/reassesses the plan and changes only revision/security agreement/edition-version-provenance/history metadata. It does not create replacement accounts. | A08 native redb/PostgreSQL transitions preserve shared rows in both directions; encrypted fixtures have two identities and live non-admin login. No fresh cross-build execution at `f0a9edb` is claimed. |
| Same authorization rules for shared capabilities | Shared Core principal/client policy plus management writers and identity liveness; edition features select availability, not permission. `node_security.rs:111` checks issuer, authentication policy, rates and shared active capabilities during handoff. | Historical native fixtures retain two grants and check a live auditor grant after both handoffs. This is representative authorization evidence, not every permission/protocol branch or a deployed mixed-node matrix. Platform-only retained authority blocks downgrade rather than becoming silently inactive. |
| Revocation continuity | Epoch changes, delegated-grant deletion, child credential revocation, logout and deactivation intent stay in `identity.rs:102` in the same transaction. `core.rs:1045` checks bearer expiry/revocation and `identity.rs:46` checks bound-session revocation/owner. | Encrypted native fixtures retain two revoked sessions, compare logical row hashes and exercise logout/live login. Same-edition redb recovery and encrypted PostgreSQL restore reject old restored sessions. Restoring is a separate recovery path, not a reason to weaken edition switching. |
| Credential protection | Shared `crypto.rs:130`/`:140` password hashing/verification, `password.rs:22` policy port, `assembly/password.rs:11` concrete history/revocation/audit adapter, shared TOTP/passkey implementations and encrypted Store IO. Management issuance stores hashes/markers and keeps generated credentials out of receipts. | Native encrypted drills compare protected logical credential/key/history rows (three credential-category rows in the fixture) and PostgreSQL raw ciphertext; wrong/missing database keys are refused. User rows including their password hashes are also part of the preserved shared-row snapshot. No hardware passkey/USB journey or all-factor enrollment matrix is inferred from this fixture. |
| No silently ignored configuration | Config has strict Serde inputs and calls `edition::validate_config` (`config.rs:520`). `edition.rs:383` rejects unsupported config/settings/retained records; transition plans bind full config/store and recheck under writer lock. `core.rs:216` checks issuer, edition, read-only connector merge, node agreement and capabilities before migration/startup writes. | Historical native matrices reject incompatible config, direct wrong-build opens, stale store/config plans, policy/capability drift and other writers. Current format-3 checks also refuse old/missing agreements until explicit offline recording; focused accepted O03 evidence covers this later behavior. Plain/encrypted cross-build tests from the historical pin are not relabeled as format-3 executions. |
| Shared build implementation and distribution inclusion | `Cargo.toml` makes Platform additive to Essentials; `lib.rs` uses shared records and explicit Platform/stub wiring. Protocol/runtime moves preserve cfgs and old paths. Source SAML types remain canonical; stub rejection remains fail-closed. | Historical native matrix: 60 Essentials/87 Platform capabilities, 27 excluded from Essentials. Those are counts of the measured older build, not a new current-main capability count. Task product boundaries specify availability, not proof of every real peer or workflow graph. W02's pending session-only password/TOTP graph work is outside this pinned audit. |

## Reused A08 evidence: exact pins and measured scope

A08 task `2bc5afe9-1ca0-42b5-952a-11b154d6647c` was observed `done`. Its original
outcome is: “Preserve identifiers and stored configuration; block incompatible
downgrades rather than silently weakening policies.” This audit credits that
accepted outcome; it does not reopen A08 merely because official release assets
remain absent. The older runbook's recommendation to keep A08 open predates
root's later original-acceptance reconciliation.

All tracked evidence links here are read at `f0a9edb`, under
[`evidence/a08-native-linux-arm64-2026-09-29/`](evidence/a08-native-linux-arm64-2026-09-29/RUN.md).
These are source-built **native Linux ARM64** products in a local Docker VM,
not emulated x86-64 products, cross-compiled binaries without execution, or
official released artifacts. Five product ELFs were checked as ELF64/AArch64;
the recorded target is `aarch64-unknown-linux-gnu`, Rust
`1.98.1 (48a229cea 2026-09-01)`.

| Historical pin | Meaning |
| --- | --- |
| `f3aba63ac3b824843a40b99623f1619ef8edc19f` | Unchanged native product binary source for Essentials/Platform server and maintenance plus base riauthctl |
| `730d373078c4b128373d7afa5f3e278f63b065fd` | Final original native validator tip, including PostgreSQL issuer/format-2 agreement/version assertions |
| `9f8efaf0372e02b0bc0042251d591f1facc0f783` | Encrypted transition validator and strengthened source/probe fixtures |
| `082563b362acb218f276e4dbcaf476e303b561b7` | Supplement source-head/recovery validator pin; did not rebuild product binaries |

The strengthened cross-build test, shell driver, Store probe source and encrypted
validator have identical bytes at the latter two pins. The stored final source
log has four passing executions: Platform and Essentials on plain redb, followed
by Platform and Essentials on encrypted redb. Its SHA-256 is
`7c3cb11e3e5b9c5e9c15f84d97eba3605cef519b61fdcac635e321c219438863` (5,686 bytes),
which this audit read and matched. These source executions were on the host;
the separately recorded native installed drills used the unchanged Linux
ARM64 product ELFs. The test-only native Store probe was a measurement helper,
not a replacement product binary: SHA-256
`58b1aac1fa628a8f038e7029aedb406357d8e85dce95e0580f4d895e6a03b3f4`.

| Evidence | Recorded actual result |
| --- | --- |
| [Original runbook](evidence/a08-native-linux-arm64-2026-09-29/RUN.md), original v2 manifest | Plain redb installed Essentials→Platform→Essentials, direct downgrade rejection, signing-key preservation, isolated backup/restore gate; local archive/image smoke and integrity; independent locked dependency closures 280 Essentials/329 Platform/106 client normal packages. Base client excludes server/store/USB; server closures exclude terminal USB. |
| [Plain PostgreSQL v2](evidence/a08-native-linux-arm64-2026-09-29/local-postgres-transition-v2.json) | PostgreSQL 17.11 native installed transition both directions, 22 shared rows preserved at upgrade and 25 at downgrade, other-client refusal, all three server opens/login, issuer/authentication and coordinated active-capability/edition/version metadata assertions. |
| [Encrypted redb v4](evidence/a08-native-linux-arm64-2026-09-29/encrypted-storage/local-encrypted-redb-transition-v4.json) | Passed, 30/33 logical shared rows preserved at upgrade/downgrade; fixture counts identities 2, credentials 3, grants 2, revocations 2. Missing/wrong key, stale full-store/config token, policy/agreement drift, wrong-build open and live writer refusals. |
| [Encrypted PostgreSQL v4](evidence/a08-native-linux-arm64-2026-09-29/encrypted-storage/local-encrypted-postgres-transition-v4.json) | Passed, 36/39 logical rows preserved; raw ciphertext equality checked at both handoffs (report count 36 is upgrade). `aes256gcm-v1`, the same live-user/grant/revocation fixture and drift/refusal checks, connected-client refusal. |
| [Encrypted supplement runbook](evidence/a08-native-linux-arm64-2026-09-29/encrypted-storage/ENCRYPTED-RUN.md), both `local-pg-restore-{essentials,platform}.json` and `same-edition-recovery.json` | Local logical PostgreSQL encrypted backup/restore and same-edition redb recovery passed for both editions. Wrong-key/occupied-target restore refusals, serving gated until completion, old-session refusal, restored login/health. This supplement supersedes the original manifest's local PostgreSQL-restore gap; deployed/physical/PITR recovery remains external. |
| [Integration review](evidence/a08-native-linux-arm64-2026-09-29/integration-review.json) | Prior root review verified 61 original-v2 references and 43 supplement references, all bytes matching; official_release false, new_tests_run false. This audit does not claim to repeat all 104 referenced-file validations. |

Recorded binary SHA-256 pins:

| Product | SHA-256 |
| --- | --- |
| Essentials `riauth` | `86a799d3060e5e602c7e458a9c409ffc8c06f881e2d426a1e6d9be226337445d` |
| Essentials `riauth-maintenance` | `c8164dfaa1a21c2bc8ac485eee02789d1aa6c8b2739904021485474cbd4350c2` |
| Platform `riauth` | `78209c1766eb53be741de0946a0e4ce4bc3f2217deda84a6e4f4bda85fc42b62` |
| Platform `riauth-maintenance` | `91f850d63a36290afe6f7b49c1943bea496b0754f50dcaa6b716af4790e8543e` |
| Base `riauthctl` | `573de0718e3301f2553145eb3d8fd01ce82bc69525a2176f075abc612c02a1e1` |

Evidence-file SHA-256 pins, matched from the tracked Git bytes in this audit:

- `native-arm64-evidence-v2.json`:
  `638822d3c437d06678d27d4fb68987ebd2f6a884dd36c5843e9addf7727b5c7e`.
- `encrypted-storage/native-arm64-encrypted-evidence.json`:
  `c7d168d65d5b5296f61d3ec7b06688ec7839080be9ef98a573f735f44d1ebfd6`.
- `local-postgres-transition-v2.json`:
  `f49a9462d27e952074df79624ec978264501ba9409322bee559d8ba9a99958dd`.
- `encrypted-storage/local-encrypted-redb-transition-v4.json`:
  `973d476c59a08c611a461fc2937efb1ff6b4ed38a05923c1a306e525fba39403`.
- `encrypted-storage/local-encrypted-postgres-transition-v4.json`:
  `86f331f46c288e4737038544a49c3fc4fc20fe4453c69944fdd970407539a2ea`.
- `integration-review.json`:
  `514c858f806fcef543dc9aa86806698fde70f08d50830052c642e8491ee63c0e`.

The recorded locks are server
`b5c9d11c001244b8017303ce8c20516e02946483845eee40720b0910758d4426`
and client `6f546c966c70505b8ef204cea811f7300b1b359de63f01a8389caadc3164b964`.
The encrypted validator SHA-256 is
`09e137d88eb8e873a488448b7bdfdbe80d2327fbca716a93f445eaaae1ea9e8e`;
the strengthened plain PostgreSQL validator is
`d6b0d023164756b2fcabe1a89794400d5ee70b43b6b4874abed3fd2f6a2cce7d`.
Recorded binary/lock hashes identify the historical measurement, not a claim
that this turn rehashed or ran the binaries.

Current main is newer than those product binaries. In particular its
format-3 node-security agreement records all 16 effective HTTP thresholds;
historical products measured format 2. The accepted
[O03 report](local-wave29-o03-rate-agreement-report.md) records seven focused
node-security unit passes, one all-category local counter test, one actual
maintenance-command adoption/upgrade test and an Essentials check. It explicitly
does not claim PostgreSQL or new cross-build execution. Its handoff checks
preserve rates, and startup no longer silently adopts an absent agreement.
Root corrected the now-valid format-3 `future` fault to unsupported format 4
in accepted `bb3cefa`. The old cross-build results are credited at their exact
historical pins, not relabeled as a run of current main or pending W02 work.

## Local gaps versus external artifact gates

The concrete local gap is visible without any graph heuristic:
`registration.rs:3`/`:8` import Core and Tx; `:109`–`:151` contain four Core
entrypoints; `RegistrationAuthority` (`:50`–`:93`) looks up live token/creator
authority and consumes use through concrete Tx; `check_creator` (`:153`) performs
stored agent/user authorization. These are runtime/assembly responsibilities
mixed with serializable RFC 7591 contract/record definitions. Moving their whole
implementation set to assembly would establish the next real boundary while
retaining the single management mutation implementation.

Other visible ownership seams need later disposition, not invented completion:
management's Core/Tx helper dependencies, connector/provisioning/controller
runtime composition, API data/transport-contract ownership, and Core startup/
listener/background assembly. Some dependencies are intentional service
composition rather than defects. This audit identifies no new distribution
identity/security behavior bug and does not require separate crates merely to
reduce counts. It also does not certify all these seams as fully separated.

The last accepted source report's explicit inventory recorded protocol→Core
3→0 and protocol→storage 3→1, the remaining storage edge being a cloud cleanup
unit test; assembly→Core 48→51 and assembly→storage 43→45. Those counts are
historical slice results, not a checker rerun here. The checker omits parent
imports, macros, dispatch/runtime calls, and puts registration outside the
protocol group. Relocating real bodies matters; classifying an adapter as
management or keeping a compatibility re-export can make graph zero coexist
with this concrete registration gap.

Official exact x86-64/ARM64 release sets, checked-tag/GitHub run provenance,
official package/installed-release execution, signatures/attestations/SBOM,
publication/deployment, installed previous-version rollback and production
topology/secret escrow remain unproved by the local A08 artifacts. Every reused
native transition manifest/report has `release_gate_result: false`. These are
external release/deployment gates (A09/Q08/Q10/Q11 and relevant recovery rows),
not an excuse to discount already accepted native A08 implementation evidence
or to manufacture another product slice. This audit inspected local accepted
evidence only and made no external release lookup.

## One proposed next owned slice — registration runtime assembly

Request root's ownership decision for an A03-only follow-up covering
`src/registration.rs`, new `src/assembly/registration_runtime.rs`, additive
`src/assembly.rs` wiring, a narrow registration guard in
`scripts/check-module-boundaries.py` and its separate report. No file is claimed
or edited for that proposal now.

Move the complete unchanged four-method Core impl, `RegistrationAuthority`
with its five methods/private record field, and `check_creator` into the new
assembly runtime. Keep `RegistrationTemplate`, `InitialAccess` with its record
fields/view, and `RegistrationRequest` canonical at their existing paths, plus
the old crate-private authority path through a compatibility re-export. Moving
the whole authority type preserves the private field; its methods and the
existing InitialAccess fields are already crate-private. No visibility widening
or production edit to management/API/core/source/workflow is proposed.

Preserve exact method/helper bodies and call sites. In particular
`management.rs:3194` must still validate live initial token/creator **before**
receipt replay, permit an exact final-use replay before requiring another use,
validate the request, consume the use, write the client and audit in the same
transaction. Registration-template issuance must still use
`create_registration_template_issuing`/`issue_credential_once` and keep the
generated token out of receipts. Existing Core signatures, record/Serde/schema
definitions, registry keys, edition checks and current management ownership stay
intact. The ten runtime function bodies can be compared exactly; a narrow
negative guard should cover registration irrespective of its current graph
classification. A concrete unforeseen private/caller seam would be reported to
root before broadening scope. This is a proposal, not permission to implement
or a promise that this one slice closes all of A03.

W02 owns the session-only conditional password/TOTP graph in `workflow.rs`,
executor password/TOTP and isolated approval label. This audit proposes no
overlap there, nor any edit to the accepted shared activation hook/source
factories. Management owns its writers/API adapters; revisions owns connector/
admission/rate state and storage/revision behavior. The proposed compatibility
path is intended to preserve those callers byte-equivalently and requires
root's coordination before any future move.

## Verification actually performed and handoff

This turn read immutable accepted/historical Git trees, selected source bodies,
manifests/runbooks and the original RiWork records; compared the nine source
integration blobs; computed the listed tracked evidence hashes; and read/hashed
the preserved final source cross-build log. It did not run that log's commands,
rehash all historical products, repeat the root's 104-file evidence review,
run the boundary checker, build, test, benchmark, inspect a desktop, or contact
an external service. No new task/worktree/worker/managed shell was created.
Only this report is committed; document-only staged scope and whitespace are
checked with Git. No Cargo target or runtime process was needed.

Root should retain A03 **in_progress**, credit the accepted component and A08
evidence above, and decide the proposed registration ownership. A03 can be
recommended done only after the full remaining responsibility inventory is
accepted against its original wording, with the distribution/security evidence
and applicable artifact limitations recorded. Neither the latest zero direct
protocol/Core count nor a report alone meets that outcome. Root owns board
changes, integration and push; this audit makes no task-status mutation.
