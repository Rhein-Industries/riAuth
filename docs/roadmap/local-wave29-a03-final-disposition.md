# A03 final responsibility disposition at accepted e302

Recommend **done** for the original A03 responsibility-separation outcome,
subject to root's closure decision. The concrete registration mixture identified
by the [earlier audit](local-wave29-a03-closure-audit.md) is now accepted. The
previously undisposed service, controller, transport and startup seams below
have distinct owners and dataflows; none establishes a conflicting A03
requirement needing another product change. No implementation slice is proposed
or reserved. This recommendation rests on inspected implementation and accepted
historical execution evidence, not a zero graph count or this report alone.

## Immutable scope and original acceptance

- Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; existing A03 task
  `4467345d-7a4f-4bb9-bd5a-586c775b0b44`.
- Existing worktree `42bb51c6-c198-4adb-bd92-0a5222853231`, branch
  `roadmap/local-module-boundaries-wave27`; pre-report HEAD
  `de1f0e073e25f647c041014c585bb6822bdb6f79`. Own history is retained.
- Audited published main `e302c10edee5b47176c5fc7ae9f255c7cb0336e7`, tree
  `fbedbc5fb08c32432390da1de8760ad65dd8a78e`. Source citations below refer to
  this immutable pin, read through Git objects without merging or product edits.
- Earlier audit `a89b69b18c4a4edbdd7b890569b421b62a013d4d` remains historical
  evidence at its original `f0a9edb3275234b891dbe312bf3ce26265587ea9` basis.
  This supplement resolves its registration gap and responsibility dispositions;
  it does not rewrite that earlier observation.

The original task's exact controlling acceptance is:

> Separate identity core, management, storage, protocol adapters, connectors, API types, server assembly, and client responsibilities.
>
> Workstream goal: Make riAuth modular without creating two identity implementations.
> Workstream completion gate: Switching distributions does not create different users, different authorization rules, or silently ignored configuration.
>
> Both server builds retain the same identity, authorization, revocation, and credential-protection semantics for shared capabilities.

The earlier audit retains the complete verbatim product boundaries and evidence
instruction. Essentials/Platform features remain edition-gated around shared
identity semantics; the remote client and optional terminal USB support remain
separate from the server. A03 does not require eight independent Cargo packages,
the absence of all Core/Tx references, or that every composition function live
under a directory named `assembly`.

## Responsibility mapping and disposition

| Original responsibility | Owner, concrete function and dataflow at e302 | Disposition against A03 |
| --- | --- | --- |
| Identity core | Canonical User/Group/Client/Identity/Session/Grant records remain in `src/model.rs`. `identity::validate_user` and session validation use `identity/persistence.rs::IdentityTx` for epoch, enabled-account, ownership and revocation checks. `assembly.rs:273` implements that port over the caller's Tx. | One shared identity/policy implementation, with storage effects supplied through a port. Edition-specific proofs feed shared identities rather than another user model. Accepted identity/transition boundary implementations and A08 preservation evidence support this separation. |
| Management | `core.rs:69::mutation_checked` establishes the live principal, receipt/precondition and write envelope; `Core::create_user`/`update_user` call `management::create_user`/`update_user`. `management.rs:1310::write_user_record` owns authority/ownership validation, the users write and revocation/audit effects in that same Tx. Manifest `state::reconcile` calls `write_desired_user`, `write_group`, `write_client` and shared grant writers; inbound SCIM calls `write_scim_user`, which converges on `write_user_record`. `management::write_client_as` owns grant revocation and the client write/audit. | Intentional shared transactional service. Concrete Tx and Core policy/helper access permit one atomic authorized mutation across entrypoints. Removing them merely to lower references would not separate a conflicting responsibility. No duplicated HTTP-versus-manifest identity writer is established by these seams; full M03 surface parity is a distinct acceptance question. |
| Storage | `store.rs:43::RecordTransitions` supplies injected callbacks; Store/Tx and `postgres_store.rs` own backend IO, indexing, preparation and transaction machinery. `assembly.rs:197::IdentityTransitions` supplies shared identity/security effects and `IdentityTx` wiring within the existing transaction. | Storage owns persistence and transaction mechanics, while identity owns policy. The callback dependency is intentional composition rather than a second backend-specific identity implementation. Native plain/encrypted redb and PostgreSQL evidence below exercises the shared durable records. |
| Protocol adapters | OIDC and other protocol modules retain scoped ports and canonical protocol records. Accepted cloud/browser/portal/SCIM/source runtimes implement Core/concrete-Tx adapters in `src/assembly/*`. At e302, `registration.rs` retains records/Serde/schema and its crate-private compatibility alias; `assembly/registration_runtime.rs` owns the complete four Core methods, five authority methods/private field and creator check. Management remains the client writer. | The concrete registration gap is resolved. Worker code `8ef7efc01ce4f4e3057c12bca7b96b35e2a2c1bc` and report `de1f0e073e25f647c041014c585bb6822bdb6f79` were reviewed and published at e302. Root independently checked exact authority/Core/helper chunks, private chain and callers; this turn inspected the accepted bodies and credits that prior review. Compatibility paths are deliberate adapters, not proof that bodies remain in their old protocol modules. |
| Connectors, provisioning and controllers | `connector_definitions::reconcile` (`:1145`) authorizes and writes desired definitions in the caller's Tx; `merge` (`:1404`) resolves local credential origins and constructs validated process configuration read-only. `provisioning_step` claims a job, projects canonical User/Group records to remote resources and fences remote IO; `fence_provisioning` (`:1786`) rechecks actor, review, lease, revision, fingerprint and current source before dispatch. `finish_provisioning` (`:1847`) stores projection links and queues shared `identity::downstream` deactivation effects. `Core::execute_reconciliation` (`reconciliation.rs:1241`) refreshes credentials, checks current schedule/authority and invokes directory/cloud/provisioning service entrypoints; `validate_apply_lease` revalidates inside the apply transaction. | Desired-state writer, connector IO/projection, and durable scheduling have different responsibilities. Their Core/Tx access assembles configuration, live authorization and fenced application operations; remote links/job records do not constitute a competing local identity model. Moving complete controllers solely because they invoke services has no identified conflicting A03 requirement. Group materialization, connector peer completion and management parity remain their owners' rows. |
| API types | `model.rs:361` onward owns canonical NewUser/UserPatch/NewClient/ClientPatch inputs; canonical settings/credential/federation/JWK/claims types have shared model modules. HTTP-local `api.rs:1709::Login` feeds `login`/`Core::login_for`; `:2283::PasswordChange` feeds `change_password`/the Core service through credential admission. Registration owns its protocol request. | Intentional transport DTOs describe wire input and pass it to shared services. A private HTTP-only record does not duplicate credential protection or authorization policy. Canonical shared records and compatibility paths supply the reusable boundary; an independently compiled API-types crate is not an original requirement. No M03 API change is proposed. |
| Server assembly, startup, listeners and background | `Core::open` constructs Store; `open_store` (`core.rs:216`) checks issuer/edition, merges connectors read-only, enforces the node agreement and capability dependencies before migration/recovery writes. `api/server.rs::serving_preflight` checks gates before `start_role` composes listeners/background work. `start_listeners` dispatches protocol adapters; `start_background` gives service closures to `Background`. `background.rs::shared`, `execute` and `spawn` own per-store admission, permits, deadlines and scheduling. | Intentional application composition and execution support. Core holds the shared configured services; the server selects roles/adapters; background owns bounded execution. These functions invoke policy owners rather than reimplementing identity. Their location outside `src/assembly` does not establish a responsibility conflict or justify another move. Accepted workflow/source factories and activation hooks remain intact. |
| Standalone client | `crates/riauthctl/Cargo.toml` declares its own workspace, no server/store/config dependency, empty default features and optional `terminal-usb`. Client transport binds issuer/discovery and calls remote management/operations routes. Server dependencies exclude terminal USB; accepted historical dependency-closure evidence checks both servers and the base client. | Concrete client/server separation exists. Client wire DTOs and remote requests do not implement another server identity policy. Complete client surface parity and official packaged-client execution belong to their separate gates. |

The connector activation seam is explicit, not silently discarded configuration:
`config.rs:60` defines the operator's optional `connector_secret_dir` switch;
planning rejects its absence (`connector_definitions.rs:758`). With it unset,
stored definitions are deliberately dormant; with it set, merge validates rows,
pins, first-owner bindings, TOML conflicts and the merged config. Essentials
rejects stored Workspace/Entra rows before this opt-out branch. `status` (`:1487`)
reports the enable switch, stored revision, process-loaded revision and
`restart_required`; retirement remains visible until restart. This is documented
in `docs/agent.md:90` and exposed through state export/client operations. An
explicit reported local activation choice is different from an edition silently
ignoring unsupported configuration.

## Distribution/security evidence and distinct release gates

A08 task `2bc5afe9-1ca0-42b5-952a-11b154d6647c` is accepted **done**. Reuse its
tracked [native evidence](evidence/a08-native-linux-arm64-2026-09-29/RUN.md),
[encrypted supplement](evidence/a08-native-linux-arm64-2026-09-29/encrypted-storage/ENCRYPTED-RUN.md)
and [integration review](evidence/a08-native-linux-arm64-2026-09-29/integration-review.json):

| Original gate | Accepted evidence and exact measured basis |
| --- | --- |
| Same users and authorization across distribution switches | Native source-built Linux ARM64 product binaries from `f3aba63ac3b824843a40b99623f1619ef8edc19f`, original validator `730d373078c4b128373d7afa5f3e278f63b065fd`: plain redb and PostgreSQL Essentials→Platform→Essentials, retained records, issuer/authentication policy and login checks. Plain PostgreSQL preserved 22/25 prior rows. The encrypted supplement below adds real non-admin user and auditor-grant assertions. |
| Shared revocation and credential protection | Encrypted validator/source fixtures `9f8efaf0372e02b0bc0042251d591f1facc0f783`, unchanged native product binaries at `f3aba63`: encrypted redb retained 30/33 logical rows; encrypted PostgreSQL 36/39 logical rows with ciphertext equality at both handoffs. Fixtures included two users, credential records, grants and revoked sessions; live credential/grant and old-session checks, missing/wrong key refusal and restored-session rejection were exercised. Recovery supplement pin `082563b362acb218f276e4dbcaf476e303b561b7` used those unchanged products. |
| No divergent policy or silently ignored configuration | A08 checked wrong-build opens, absent/old/future/shared-capability agreement drift, authentication-policy drift, stale full-store/config plans, incompatible configuration and writer/client refusal. Four host source cross-build executions separately covered both editions on plain/encrypted redb. Current e302 code still gates config/edition/capability dependencies and startup agreement before serving; `node_security::enforce` now compares strict format-3 effective rates as well. Historical native runs measured format 2 and are not asserted as new e302 executions. |
| Official artifact, release and deployment gates | Separate A09/Q08/Q10/Q11 and relevant recovery/deployment rows. Local A08 products are source-built native ELF64 AArch64 in a local Docker VM, not official released artifacts; manifests retain `release_gate_result: false`. Official architecture/package sets, build provenance, signatures/attestations/SBOM, publication, previous-version installed rollback and deployment/escrow evidence remain unproved here. These distinct gates do not negate the fulfilled A03 module-responsibility outcome. |

The prior integration review verified 61 original-manifest and 43 supplement
references, all bytes matching, with no new tests and `official_release: false`.
This turn read those accepted records; it did not re-execute them or rehash the
historical artifacts. Representative historical behavior evidence plus current
shared implementation supports A03; universal factor/hardware/production
coverage or a fresh current-main cross-build is not claimed.

## Work performed and closure handoff

Read original A03/A08 task records with the explicit project ID, existing guidance
and audits, immutable e302 source bodies/dataflows, and accepted historical
evidence. Only this report is changed; staged document scope and whitespace are
checked with Git. No product check, boundary checker, build, test, benchmark,
desktop inspection, external lookup/service, new task/worktree/worker/managed
shell, board mutation, main edit or push was performed.

The new concrete disposition is that each formerly unresolved composition seam
above has an intentional owner/dataflow and no identified local A03 conflict.
Registration's accepted exact move resolves the earlier concrete gap. Recommend
root close A03 **done** against the original outcome, retain the accepted A08
credit, and track release/deployment and other lanes' functional acceptance on
their own rows. Root owns review, integration and task status; this report makes
no status change and claims no W02 config/source-TOTP/executor or M03 API work.

## Root closure decision

Root accepts **A03 done** against its original responsibility-separation outcome.
The reviewed implementation slices, including the exact registration move, and
the eight responsibility dispositions above support that decision. Root also
checked the current identity persistence port/assembly callbacks, management
transaction writers, connector reconciliation/read-only startup merge, server
preflight/role composition and separate client manifests. Current product files
remain the audited e302 versions; the intervening O03 change affects only two
evidence fixtures and their report.

Accepted historical A08 measurements are credited at their recorded source and
artifact pins. Root inspected the tracked integration review and prior accepted
evidence; this closure runs no new Rust build, runtime test or historical drill.
It does not claim current-main native format-3 cross-build execution or official
release/deployment completion. Those distinct gates and other lanes' functional
acceptance remain on their existing tasks. After publication, root updates only
A03's existing RiWork task status; no new task, worktree or worker is created.
