# Six original-task completion review

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; worktree
`42bb51c6-c198-4adb-bd92-0a5222853231`; 2026-10-02.
Fixed published source: **`da5ff7dcfc3442c302955344229168872911b0ec`**,
tree `0b1e250a1ab216ad3a90e6a460cb4e0b5f098e06`.
Own starting HEAD: `e6b62f3241189211291989d9ea33b3fcedffdee0`.

**Recommend DONE for all six original implementation outcomes below.** Direct
source and assertion review found no missing local scope within root's stated
acceptance. This recommendation changes no status. All six RiWork rows were
observed `in_progress`, assigned to this worktree; A03 remains DONE. Root owns
independent verification, integration and final board disposition.

Only this report changes. There was no product/test edit, merge, reset, Cargo
invocation, build, runtime test, service, benchmark, external lookup, desktop
interaction or worker launch. Current source was read through immutable Git
objects, not substituted into the older branch. Historical executions below
belong to their recorded pins and authors; none was rerun for this review.

## Original acceptance and scope

The six original records were reread with `riwork task list --project
891e7443-8dac-4c1b-897f-9e53cb59c7ee --json`. The original completion-evidence
clause says: “Review the relevant implementation, tests, documentation, and
released artifacts as applicable. Report verification actually performed,
remaining gaps, and external prerequisites. Do not mark implementation complete
from documentation or a worker report alone.” The latest assignment authorizes
this review and supersedes the rows' old scheduling hold.

| Row / exact UUID | Original requested outcome | Original workstream gate | Recommendation |
| --- | --- | --- | --- |
| W07 `63ab3917-0fe0-4d02-819e-fc0c09b72ca6` | Isolate exceptional custom logic with explicit permissions and bounded execution, data access, and network access. | Platform can express conditional enrollment and authentication; Essentials still works without asking its administrator to design a workflow. | DONE: the declared macOS native guest is isolated and bounded; unsupported hosts explicitly refuse. |
| P06 `179af025-34ab-454d-b8b2-768fdbbc30e7` | Implement the attributes, filters, updates, lifecycle behavior, and schema extensions required by supported clients. | Disabling a user produces immediate local action and visible, durable downstream work—without falsely reporting that every remote system has already completed it. | DONE for the advertised core User/Group SCIM contract; no unnamed vendor extension is added. |
| P08 `d0efa744-058e-42ef-b328-e03689094712` | Handle partial failure, retries, ambiguous remote responses, managed-attribute boundaries, and operator reconciliation. | Same provisioning gate as P06. | DONE: durable per-item outcomes and explicit settlement preserve at-least-once limitations. |
| I05 `6e7724c5-3cdd-4a9c-abeb-bbd6e8eb00c9` | Make a token broker optional rather than an undocumented prerequisite. | Every advertised integration has a working setup, lifecycle, and failure-handling path—not just an endpoint or protocol module. | DONE: direct signed service-account grant, setup, fresh-key/expiry and failure handling exist. |
| I09 `7ece8e56-8a0a-4b40-b318-d9abbba1d953` | Cover browser requests/approvals, expiry, parent-owned agents, and device revocation without turning temporary access into permanent membership. | Same integration gate as I05. | DONE: browser and shared writers, expiring projections, child credentials and device revocation are implemented and exercised. |
| G02 `c3b6b41c-4472-4a16-9773-6e6f4ab6a8c4` | Handle subjects, issuers, groups, applications, credentials, and explicit source links deliberately. | The migration report tells an administrator exactly what transfers, what changes, and what still needs work. | DONE: conversion/report and target-bound reimport controls cover each named family. |

Original goals remain: W07 supports complex identity behavior while keeping
Essentials straightforward; P06/P08 support reviewed changes and dependable
automatic synchronization; I05/I09 turn supported protocols into complete,
tested integrations; G02 makes cutover predictable without assuming every
credential or custom policy is portable. Shared identity, authorization,
revocation and credential protection remain requirements. Root explicitly accepts
the W07 supported-host scope and separates I06/Q04 tenant rollout, Windows
installation, G05 customer cutover and official release gates from these rows.

## W07: permissions and native isolation

Directly inspected `src/workflow/extension_gate.rs::{decode_manifest,project,
execute_guest,fresh_module,admit_sections,launch,wait_for_guest,expire,
isolated_guest_entry}`, `extension_gate/isolation.rs::{command,check_entry,
check_descriptors}`, the Seatbelt profile and native probe. Manifest/stage/hash
binding and the intersection of requested/granted permissions precede execution.
Only granted identifiers enter the bounded frame; passwords and arbitrary
profile blobs do not. The admitted Wasm has no imports, one fixed memory page,
one routing function, bounded locals, translation/fuel/value-stack/input/output
limits and one declared label; its label cannot manufacture authentication proof.
Network permission is explicitly denied in this supported guest profile.

The parent launches the exact server image with cleared environment and default
deny Seatbelt rules. Native entry checks confinement and inherited descriptors
before reading IPC or compiling. Deadline checks occur before accepting an exit
and after joined IO; a running expired/flooding guest is killed/reaped and output
discarded. Unsupported hosts return `ExternalRuntimeRequired` without spawning
or falling back to an unsandboxed guest. The trusted in-process extension host's
cooperative timeout is not substituted for this operator guest boundary.

Read actual assertions in the kernel-isolation, inherited-file/socket/kqueue,
unsandboxed-entry, granted-identifiers, nonreturning/flooding, completed-after-
deadline, installed-image and admission tests. The kqueue assertion independently
proves Darwin closes descriptor 100 across exec; it does not misattribute that
closure to the guest descriptor audit. The complete four gate/isolation/profile/
probe blobs at the final native source `d772d29fbfc4e681774593ebbe32839df5f5f032`
are byte-identical to the fixed source and accepted compatibility integration
`f9b2920dc2509c71b08107202f2c52333405cd93`.

Executed evidence: native `cargo test --locked --lib
workflow::extension_gate::tests -- --test-threads=1`, **19 passed, 0 failed,
95 filtered**, 33.06 s; source stack `6421a1e79392fe483130f727a365066cbdb88ee6`
and final `d772d29fbfc4e681774593ebbe32839df5f5f032`, accepted as
`afd7ad2d7e3299778cfad218d4610ba676c9573b` and
`1610156480a7205d185180bc47c3ec4a28680a9a`. I read the actual
`target/wave27/evidence/tests-final.log` in the read-only extension-isolation
source worktree; SHA-256
`619315c304faf46a656024e1bda649e48f703d5c758fcae088a2f99a7b45701e`.
Its declared environment is macOS 26.2/Apple Silicon, source-built native server,
not an official released artifact. The tracked isolation report and root's
accepted ledger corroborate the final 19 cases.

For the shared workflow gate, also read `src/workflow.rs`'s exact configured
extension/password and conditional session/UV-passkey/enrollment matchers and
`executor.rs::workflow_configured_start`: declared shapes and permission coverage
remain admission requirements. Accepted W02 conditional code
`a03731379e259ed8252d09db5a47d9445776caae` has **6/6** executed tests using
`cargo test --locked --features test-support,fuzzing --test
workflow_configured_conditional_enrollment -- --test-threads=1`; its corrected
existing enrollment fixture at `e5f5facddab4f22651ae671a3c6e864ef504c39a` passed
**1/1** separately. The fixed W02 evidence and closure reports attribute both
runs and retain Essentials built-in behavior. No arbitrary graph, other-host
native runtime, hard real-time scheduler or new network ABI is a W07 closure gate.

## P06: advertised SCIM contract and disable outcome

Read `src/assembly/scim_runtime.rs` metadata/schema, ownership/record binding,
filter parser, sorting/projection, conditional write/delete and complex PATCH
helpers, plus `src/scim_shared.rs` records and `docs/scim.md`. Metadata advertises
the two core User/Group schemas, PATCH, filters, sort, ETags and scoped OAuth
credentials; bulk is false. User names/display/active/external ID, structured
name/emails, write-only password, read-only groups and Group members have defined
behavior. Core URN paths, same-email valuePaths, bounded conjunction/disjunction,
presence, sorting and attributes/excludedAttributes are supported; unsupported
syntax/schema/projection returns an explicit error. Unknown stored subattributes
round-trip without being advertised as a separate vendor extension.

`scim_write` checks owned resource and live authorization through the shared
mutation/precondition envelope, validates the entire patch before writes and
uses management user/group writers. Exact ETags and bound retries survive related
resource version changes; group replacement preserves other owners and unrelated
durable members. `scim_delete` disables through the shared identity writer and
retains a tombstone. Read assertions in `scim_filters`, `scim_projection`,
`scim_patch_compat`, `scim_patch_valuepath`, `scim_pagination` and
`scim_resource_versions`. In particular,
`scim_user_delete_commits_shared_revocation_and_downstream_intent` asserts stale
412 rollback, successful exact-key delete replay, disabled account/raised epoch,
invalid session, one pending downstream row and one delete audit. Filter tests
compare GET with `.search`; email/foreign-member invalid writes preserve snapshots.

Executed accepted evidence: `fa9295fb8ad74dde561f3bb9c14674d1dd5a69ae`,
`cargo test --locked --offline --no-default-features --features platform --test
scim_patch_compat`, **3/3**; `25a01d4f57d2ef66f8de0617e23044b4527641c9`,
`cargo test --test scim_resource_versions`, **8/8**, plus redb paged relation
If-Match **1/1** and disposable plain/encrypted PostgreSQL equivalents **2/2**.
At `221b0585423d09504a611582e3a1e04773a7b92c`, `cargo test --locked --features
test-support --lib --test parser_assurance -- --quiet` for the missing/malformed
PATCH member correction passed library **83/1 ignored** and parser assurance
**6/6**. Source `b834c93cd3eba53ea2e98532eed6c64e3f9b7dc0`, accepted
`7b9253117e51a0676988de5ea26fb43dc2c627e5`, adds the executed redb 130-user
replacement/replay case through `cargo test --locked --features test-support
--test scim_pagination redb_scim_group_member_replace_pages_owned_users -- --exact`,
**1/1**; its source plain/encrypted PG cases were **2/2**.
The accepted execution ledger also records `scim_filters` passing in the focused
combined run at `c1dc4f1c9d9322ec13ce70078845c32cd9dfc088`; no count is inferred
for an individual target from that combined result.

No named vendor was contacted. That is interoperability evidence for Q04, not
proof of a missing advertised core attribute or an invented P06 extension.
Canonical Group values still materialize fully; paged relation scans do not
establish the separate S02 whole-Group memory claim. The pending obligation is
durable downstream work, never a claim that every remote system completed.

## P08: partial work, ambiguity and operator settlement

Read `src/provisioning.rs::{provisioning_reconcile,provisioning_apply_confirmed,
provisioning_step,fence_provisioning,finish_provisioning,provisioning_resolve,
validate_evidence,managed_equal}`, `provisioning/deactivation.rs` remote
verification/close/resolve/dismiss and `dispatch_recovery.rs` validation/record/
recovery writers. Jobs preserve cursor, immutable item/link identity, review,
revision/configuration binding, attempts, uncertainty and dispatch ownership.
Every write, including refreshed-token retry, rechecks the send fence. Conditional
PATCH changes only managed fields; 204 requires read-back before advancement.
Malformed/mismatched/duplicate responses cannot choose another identity. Failure
does not roll back already delivered items or falsely advance the current item.

Retry first observes remote state and uses bounded backoff/attempts. Lost active
Create provenance is committed before dispatch, retained through failure/cleanup
and turned into offboarding intent even without a returned link. Resolve requires
live scope, immutable item visibility and settled dispatch; recovery requires
exact revision/key plus explicit worker-quiescence/provider-settlement attestations.
Dismissal preserves prior uncertainty and evidence and never calls the successful
delivery close path. Those attestations are operator facts, not inferred from
expiry, one GET, a stopped node or rotated credentials.

Read `tests/offboarding.rs` assertions for refused versus applied-but-lost PATCH,
read-before-retry without another PATCH, partial-target outcomes, immutable scoped
disclosure, operator resolution/dismissal, abandoned pins and unlinked Create.
Source transplant `a1893407f95238deb19335a2c3401f24d7dedba8` on
`57f6a5ab57acd7a778d1baf0243687d677ccabfc` executed `cargo test --locked
--no-default-features --features platform --test offboarding p08_`, **11/11**,
and exact `delivery_browser_contract_preserves_ambiguity_and_audits_scoped_dismissal`,
**1/1**. Root accepted the rewritten stack ending
`61636e6708526311df4e14d57658b7e75efa3f1a`; the source commit is not itself a
main ancestor. Its report records initial 9-pass/2-fail fixture expectations and
the final corrections; I credit the final executions, not the initial failures.

The fixed source includes subsequent admission-lifetime fixture corrections:
source `e5dbbffed88baaa25749e5258fc64669fb6883d1` ran six distinct exact filters,
**one pass each**, and `95c590ba575d5c50397034f7be5452225d30171a` ran the seventh
`operator_dismissal_preserves_ambiguous_intent_and_requires_scoped_review`,
**1 passed/30 filtered**, 3.71 s, accepted as
`ae44623d2e33fc29c82ee4ebd459c4b377966ce5`. Read the actual command/result
sections and root equivalence reviews in the fixed offboarding reports. They
preserve original assertions and add normal router/admitted-plan lifecycle,
not raw lease edits. No current full Linux-suite pass is inferred.

External settlement/discovery and old-worker quiescence remain prerequisites
when those ambiguous operator paths are used. This implements their honest
handling; it does not promise exactly-once or prevention of remote IO after a
paused admitted process resumes. Missing pre-upgrade provenance cannot be
reconstructed; source-job retention and explicit review remain conservative.

## I05: broker-optional direct Workspace grant

Read `src/cloud_directory.rs::{WorkspaceDirectory::validate,
validate_direct_workspace_endpoints,http_client,direct_access_token}`, the cloud
runtime plan/apply assembly and `docs/enterprise/ENT-03.md`. Direct credentials
exclude broker credentials, require a bounded owner-only service-account key and
delegated subject, sign RS256 JWT bearer assertions with issuer/sub/audience,
fixed read-only scopes and one-hour expiry, and reject token lifetime below
60 seconds. The key is reread for plan and apply. Official production endpoints,
no redirects and no ambient proxies protect assertion/bearer delivery; literal
same-origin loopback fake peers require `test-support`. Broker mode remains
explicitly optional. Tokens/keys do not enter plans; retry, complete pagination,
quota/fingerprint and removal review protections remain shared.

Read actual fake-peer signature/audience/issuer/sub/kid/scope/lifetime validation
and `workspace_direct_service_account_assertion_and_expiry`, hostile endpoint/
redirect and ambient-proxy test assertions. They prove signed direct grants with
no broker secret, rotated-key apply, short-lifetime refusal before directory IO,
and no bearer forwarding. Original direct source
`edc9013c8320c8e072132ed4103903cc874b322d`, endpoint correction
`2693a3f5a3eccb6c4b410c05f3c5e5e32b6c3f2e` and proxy correction
`10cc73ad721aac9e9aa43b41e694a3cc4a0becc3` were accepted as
`55950faf993382f08087097ca3aca134f4de245c`,
`ff40b13a6a31cbe94fb5f06af2fe80984ed7e981` and
`a94a045c04a61a696b8635bfe50eaccb1617e0d7` respectively.

Executed accepted `03606ed7cd42ea3f716e3f8dcbebf001da9c3470`:
`cargo test --locked --test cloud_directory -- --quiet`, **33/33**, and the
same target with `--features test-support`, **35/35**. At
`4df94b61edf9161cac5217b00691e55fdb6ee6ed` that test-support target passed
**36/36**. The three direct token/client/endpoint helper bodies from that later
executed pin are byte-identical at fixed source despite subsequent module moves.
Setup names domain-wide delegation's numeric client ID, exact scopes, dedicated
Directory reader, private key replacement and operator failure/retry steps.
Real tenant delegation and rollout remain I06/Q04 evidence; no Google call was
made and no production tenant/released-artifact compatibility is certified.

## I09: temporary access and dependent credentials

Read `src/management/pam.rs` request/decision/review/revoke writers,
`assembly/pam.rs::{pam_actor,extra_groups}`, portal request/review scripts,
`agent.rs::{authority_active,parent_active}`, identity agent/device revocation
effects and their caller in `src/identity.rs`. Approval requires a distinct live
configured human; browser writes preserve same-origin/portal admission and exact
revision/key. Grants are separate expiring/revocable records projected at use,
not `Group.members` writes. Every child-agent use checks enabled/unexpired parent
authority; parent disable removes token indexes and disables owned agents in the
same transition. Device revoke removes pending tickets; user disable revokes
devices/tickets atomically. Re-enable does not resurrect these credentials.

Read `management.rs` Windows enrollment/credential-once writer and
`assembly/radius.rs` certificate binding: live temporary authority fences new
agent-controlled credential exposure before rotation or binding becomes usable.
Known exposure prevents later approval/projection. Browser/PAM assertions cover
foreign/self/cross-origin denial, scoped review, exact-key one-audit decisions and
revoke, expiry, invalid old access tokens, restart and unchanged durable membership.
`parent_user_ownership_constrains_agents` asserts disable/re-enable cannot revive
an owned credential. Windows assertions cover unchanged old credentials/tickets
on denied enrollment, post-expiry/revoke issuance, old-ticket invalidation, actual
device revoke and later approval denial. RADIUS assertions cover duplicate existing
binding versus denied new binding with unchanged snapshot.

Executed accepted evidence: `74ee17294cc214f4b17e0494aae1555cd7bfd082` ran four
named admin/PAM browser-contract filters, **one pass each**, including
`configured_approver_browser_review_uses_scoped_reads_and_exact_retries`.
`0d0850ce4d8b0dec427145b44f5033b192f1d699`: `cargo test --test pam`, **5/5**;
`82461a0bc2b7aae4709916e1cd76eb1d0e2cff55`: `cargo test --locked --test
pam_management -- --quiet`, **5/5**; cleanup writer
`e926d75fd0e58ab743c98a5e3d5dc0cc09378c78`: same target, **6/6**.
That complete PAM writer blob remains identical at fixed source.
The existing `parent_user_ownership_constrains_agents` body is identical to
`248207d4c187806ab597dcd9eb373b3197ccf105`, whose accepted
`cargo test --locked --all-targets --features test-support,fuzzing -- --quiet`
passed with external-service cases explicitly ignored. This credits the
parent lifecycle assertions at that historical pin, not a fresh whole-suite run.
`044a984caf208f85f2fef170c4ad7266507b4683` executed exact
`radius_eap_tests::agent_eap_bind_cannot_capture_temporary_access_and_replays_existing_binding`,
**1/1**. `bc3c3248dfa55217845602f7f4ab9d5c6a331b8e` executed the then-named
`agent_enrollment_fences_temporary_access_and_preserves_device_replay`, **1/1**;
`36e20364458ffbb6e376c7b83d116f2788592469` executed `windows_login`, **11/11**.
The identity agent/device effect blobs at that pin remain identical.

Subsequent accepted credential-once changes
`b22996523f6ba4b9bf90619f3cfe99630926edc1` /
`b8794610ad53de6a85f61b878d7b4aa9bf1d7783` deliberately renamed the Windows
case to `agent_enrollment_fences_temporary_access_and_never_replays_device_secret`:
current exact retry asserts 409 `credential_already_issued`, no secret reissue.
I inspected those current assertions; the older 1/1 and 11/11 are historical,
not claimed as executions of the renamed fixture. Accepted M03 receipt/header/
PAM fallback decisions are retained. Legacy records without issuer provenance
remain documented; no retroactive provenance or Windows LogonUI installation
is added as an I09 gate. Host installation belongs to its explicit integration row.

## G02: deliberate conversion and identity continuity

Read `src/migration.rs::{convert,classify_issuer,proven_target,application_access}`
and credential/source-link/report branches; `state.rs::{target_identity,
fingerprint_identity,preserve_proven_bindings}` and live target checks.
Conversion binds the literal target issuer and exported account identity, keeps
proven subject/client issuer/sector and source issuer/subject ownership, flattens
group ancestry and translates only supported exact application policy/mapping
forms. Missing/ambiguous policy or source connection cannot silently authorize,
merge by email or create an unproven link. Reimport cannot rename/reassign the
recorded account or alter/remove its Authentik identity attribute. Both plan and
apply reject stale target fingerprints and conflicting source/subject owners.

The generated migration report classifies items exact/convertible/manual/
unsupported, includes reasons/actions/blockers and only emits a usable manifest
when blockers are absent. Authorized supported password-hash/TOTP references are
explicit; fresh password/reset or verified source login is distinct from copying
an old credential. Unsupported passkeys, sessions/tokens, signing keys and custom
policies require re-enrollment/reconfiguration. `ready_for_plan` is not a claim
that cutover passed. `docs/migration.md` explains each operator choice.

Read the actual `tests/identity/operations.rs` conversion, issuer, application
bindings, source connection, reimport and stale-target assertions. The first
conversion applies the manifest, logs in, checks exported UID and ancestor groups,
and verifies portal application/launch metadata; incomplete mapping definitions
produce blockers and no manifest. The source-link case signs in through a local
upstream peer to the precise migrated account and refuses unexported/duplicate
connections. Reimport tests reject changed UUID/pk/name/subject/issuer/sector/link
ownership; target-claim test rejects stale plan and fresh planning with 409 while
retaining the other claimant.

Executed accepted `679f2927885d9dc4dcc1c881fd972bcade70e07a`:
`cargo test --test identity authentik_reimport`, **2/2**; target-claim filter,
**1/1**; `cargo test --test identity
desired_state_and_verified_login_share_source_link_ownership_without_reassignment`,
**1/1**.
`3ddcab4d43cfa1ce41222b6253dedbed573c0c64`: reimport **3/3** and target-claim
**1/1**, plus Essentials library check. The complete `convert` and
`classify_issuer` bodies remain byte-identical; `proven_target` only replaces
`let Some(state) = state else { return None; };` with `let state = state?;`.
Current conversion/report fixture corrections at source
`e08ef8cef4d103e578aec0537fe86bb592187b06`, accepted
`b6f1e130ec58ba706b5085c9e0302fa642bda2ce`, ran the three exact
`operations_tests::{authentik_import_preserves_exported_subjects_and_blocks_incomplete_translation,
authentik_preflight_classifies_every_exported_item,
authentik_preflight_fails_closed_on_missing_or_mismatched_resolutions}` filters
with `cargo test --locked --features test-support,fuzzing --test identity
<filter> -- --exact --test-threads=1`: **one pass each**, 180 filtered per run.
I read their fixed execution report and current assertions, including exported-
secret nondisclosure. Private customer export, actual DNS/RP change and cutover
are G05 work, not a missing converter or another original G02 requirement.

## Evidence provenance, checks and remaining scope

Historical accepted command/results above were read from the project's
`planning/accepted-commits.json` execution ledger (snapshot SHA-256
`57db3e906c0440efff48b86f5cbcd2c75eefa7f6da1b3c2f37e283f2a26a0702`), its
P06/P08 source execution reports, the W07 raw final log and fixed tracked W02,
offboarding and identity-operations execution/root-review reports. Accepted
integration objects were distinguished from non-ancestor source commits.
Metadata-only inventory paths and historical todo/remaining labels do not
establish failure after root's stated original-scope disposition.

Checks performed here: explicit-project six-row reads; applicable CONTRIBUTING/
SECURITY guidance and no applicable AGENTS file found; fixed-source runtime,
validation, writer, documentation and test assertion inspection; historical
command/result inspection; Git object/ancestry and the explicitly identified
blob/body equivalence comparisons; report UUID/pin/scope and Markdown/whitespace
checks. No new execution evidence is generated for any product behavior.

`git diff --check` passed. `python3 scripts/check-docs.py` reported only five
pre-existing root directories (`target-wave27`, `target-wave28`,
`target-wave28-portal`, `target-wave28-scim`, `target-wave29-source`) forbidden by
its layout rule. No Markdown link failure was reported. These older build
directories were left untouched; the new report received a separate scoped
Markdown-reference/acceptance/pin check: six original outcome/gate/UUID mappings,
assigned `in_progress` states, 44 existing Git objects, the fixed tree and sole-file
scope all passed. This is a disclosed repository check
failure, not a passing whole-repository documentation claim.

No new local implementation slice is proposed or reserved. Applicable remaining
evidence is specifically: W07 other macOS/architecture or released-bundle
validation; P06 named-client interoperability; P08 real provider settlement and
old-worker quiescence when recovering an ambiguous dispatch; I05 tenant delegation
and I06/Q04 rollout; I09 real Windows installation and disclosed legacy issuer
provenance limits; G02 private customer cutover under G05. None is relabeled as
completed, and none is an extra implementation gate for these six recommendations.
Official artifact/release/deployment certification and overall green CI remain
separate. Desktop provider preference remains **RiWork Cua.ai Driver only**;
no desktop was necessary or used. Root alone sets statuses and integrates/pushes.
