# D05 category evaluation refresh proposal

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; task
`6a2381ab-0f2f-48c6-adf9-8115b78cc143`; existing worktree
`ed9ac424-59f4-4520-905b-919aea3521eb`, branch
`roadmap/local-revisions-coordination-wave27`; 2026-10-02.
**Read-only evaluation at published main
`c01c39ab4e092423d5522bedc50fff87656d8c0a`.**
Own starting HEAD `e63f4e8bb449fd3b7e173633babc13522de0be1f`.

## Decision and exact reservation

Reserve **only this new plan** for this assignment. Propose a subsequent
two-file evidence refresh of `docs/roadmap/d05-acceptance-evidence.json` and
`docs/roadmap/d05-acceptance-evidence.md`, subject to root's review of the
specific changes below. Neither existing artifact, production source, test,
guide, task status nor accepted/main branch is edited here. No merge is needed
to inspect immutable objects or commit this proposal.

The old evaluation is materially stale. Accepted implementations and actual
historical executions discharge several former local blockers; they do not
establish every category's hardware, tenant, release or deployment claim.
Do not classify all underlying work as pending, infer an overall pass, or
require a new universal test campaign. **Recommend D05 remain in_progress
pending the original independent new-user/new-operator workflow completion
record.** Updating the evaluation is the next smallest owned work; no new
product defect or implementation seam is established by this audit.

## Original row and controlling goals

Read the live original row with `riwork worktree tasks
ed9ac424-59f4-4520-905b-919aea3521eb --json`, and the relevant original rows
with `riwork task list --project
891e7443-8dac-4c1b-897f-9e53cb59c7ee --json`. D05 is assigned here and
in_progress. Its requested outcome is:

> Evaluate usability, workflows, administration, interoperability, footprint, performance, availability, recovery, migration, and security evidence.

Its workstream goal is to document completed user/operator tasks, rather than
available settings. The gate is that a new user and a new operator can
independently complete the documented workflows. The completion-evidence
clause requires relevant implementation, tests, documentation and artifacts as
applicable, actual verification, gaps and external prerequisites; a report
alone cannot complete code. This assignment supersedes the original row's
old unassigned/scheduling text, without changing the outcome.

A02's `docs/roadmap/product-contracts.md` supplies the frozen Essentials,
additive Platform and separate remote-client boundary, G01–G12 and EG01–EG04.
EG01 requires named peer/profile/direction evidence; EG02 native artifacts
and declared browser/hardware support; EG03 runtime security/recovery;
EG04 equivalent-security measurements and named operator objectives.
These are claim-specific gates, not a requirement to re-run every accepted
implementation before updating an evaluation.

Live status is separate from immutable code provenance. Relevant DONE rows
include A02/A03/A04–A08, U01–U09, M01–M07, W01–W07, P01–P08, I01/I03/I05/I09,
S01/S03–S06, O01–O05, R01–R04, G01–G04, Q01/Q02/Q05/Q07 and D02.
This is credit for their reviewed original scopes, not fresh executions of
all their scenarios. O06, O07, R05 and D01 remain in_progress; A09, Q03/Q04,
Q06/Q08–Q11, D03/D04, G05 and other named integration rows remain separate.
S04 and O03 stay DONE. The held S02 Group stack is not accepted.
Accessibility work is canceled by the assignment; its absence remains a
support-evidence limitation, not authorization to restart U10/Q06 scans.

## Existing artifact identity and required corrections

| Pinned input | Git blob / SHA-256 |
| --- | --- |
| Existing JSON | `501e4b4af8ce1529c52bc3edececbd2f30949a5c` / `679bd9a31f6d6c58c7fc2057d256e6c717a6b642b6c4b75abd613ff669f5b3a8` |
| Existing Markdown | `9e3116c1cc50a984ed3ba3e1613efff560fa271c` / `bc462d87981630c53fec7d0ffe4f2a8602d6877ec451f4a0c33f7b6dee8a95cb` |
| A02 contract | `a9c132f4d4ed82020501cc39e2209863c6b091a9` / `e4a9b20c19a5385e922eadb3eafb9162a93309eed075daa501b28d4bbabe1e2c` |

Both D05 artifacts still describe 2026-09-29 integration `0312427`, the
older D05 worktree, 456 reviewed slices and 17 open checks. The JSON's
`acceptance_rule` freezes every status to not_passed for that old slice.
Those are historical observations, not current constraints or live counts.
The ledger read here records reviewed/pushed main `c01c39a`, accepted
`c8d6f87915ac949a71f4ed9028a2f5cc350e77dc` and remote verification.
Its older `latest_actions_run` and `ci_followup_latest` subobjects still
name earlier states; retain their observation times instead of presenting
them as current CI status.

Concrete obsolete holds to supersede in the new current assessment:

- W02 executor/conditional enrollment, W05 review/versioning and W07 native
  guest confinement are accepted within supported scope. Arbitrary graphs
  and other-host native guests are not inferred.
- A03 responsibility separation, M03 shared management and M07 protected
  connector desired state/activation parity are accepted. Replace the old
  seven-protocol-file count and blanket management approval gaps with exact
  supported interface/resource boundaries.
- O03 shared jobs, target admission, SCIM stamps and explicit format-3
  all-sixteen effective rates are accepted. Remove the old assertion that
  these are all still unimplemented.
- S04 narrow families, lookup, persistence interleaving and exact issuer
  ownership are accepted, now with measured writer-cost and ordered
  concurrency evidence. Do not retain its old 3/3-PG or performance-open
  sentence as the present disposition.
- Native ARM64 artifacts, secure paced Q09 observations, logical encrypted
  PG recovery and physical base-backup invalidation evidence exist.
  Replace blanket absence claims while keeping their actual source,
  artifact, topology and reconciliation limits.
- O06 has accepted diagnostics and specific fresh local fixtures, but the
  whole row is still open. Gauge/operator resolution never means verified
  remote delivery or healthy physical storage.
- Retire the old global fmt/Essentials-compilation statements from the
  current verdict unless a current concrete failure is actually evidenced.
  Preserve old failures as historical. Source CI definitions are not passes.

## Exact proposed JSON and Markdown edit

Keep `schema: riauth.acceptance-evidence/v1` and the ten category IDs; no
product/API/schema change is proposed. No script/test consumer of the D05
file/schema was found in the pinned `scripts` or `tests` search.
Use additive evidence fields and retain prior provenance:

| Field or section | Proposed replacement/addition |
| --- | --- |
| `snapshot_date`, `read_on`, `assignment` | 2026-10-02; current explicit project/task/worktree IDs above. |
| `source` | Current own branch/read method and fixed published evaluation pin; actual pre-refresh own HEAD. Do not imply own product tree equals current main. |
| `accepted_integration` | Published main `c01c39a`, accepted head `c8d6f879…`, observed publication/remote verification and `edited_by_this_slice: false`. |
| `prior_snapshots` (new) | Reference the unchanged old JSON/MD blobs above, original dates/pins and prior D05 snapshot commit; preserve all old records with their original basis. |
| `ledger` | Label read-time metadata separately from pinned implementation. Do not copy obsolete open-check totals or stale CI subobjects into current blockers. |
| `acceptance_rule` | Replace the old slice-only freeze: original-row implementation closure is distinct from category/advertised-claim evidence; historical passes retain exact pins; missing inputs stay explicit; root owns completion. |
| `d05_status`, `d05_passed` | `in_progress`, false, with `evaluation_status: evaluated_with_gaps` and the exact independent user/operator gate. No blanket product-not-implemented verdict. |
| `cargo_run_by_this_slice` / new `runtime_run_by_this_slice` | false / false. Historical native/CI runs are records, never new executions. |
| `classes` | Preserve historical meanings; new records distinguish inspected source/criteria, executed local fixture, named local peer, native local artifact and release/deployment evidence. Do not conflate code presence with a passed command. |
| `newer_slices` | Append the accepted records below; keep the old entries under their historical pins, explicitly superseding obsolete `remaining` text in the current assessment. |
| Each category | `status: evaluated_with_gaps`, `category_passed: false` for the full unresolved target, plus `accepted_rows`, `evidence_ids`, `fulfilled_scope`, `unmet_inputs`, `claim_limits` and `next_evaluation`. These booleans do not reopen completed implementation rows. |
| New `evidence_records` | Stable IDs below, exact source/accepted pins, command/filter/features when recorded, counts/results, platform/backend/artifact scope, hash/reference and verification kind. An unavailable count stays absent. |

Each new record will identify whether this audit inspected its raw log,
tracked result JSON, accepted report plus source/assertions, or only criteria.
Do not invent exact commands when a record only supplies prose.
The Markdown twin will mirror metadata, the original gate, the ten-category
mapping, the same evidence IDs and limits, superseded historical holds and
the bounded next steps. Remove the current blanket all-not-passed narrative;
retain genuine unmatched targets and historical failures. No guides, coverage
inventory, workflow, source, tests or other reports are in the follow-up claim.

## Ten-category current mapping

| Category / original target | Accepted scope and evidence to add | Explicit unmet input and next evaluation |
| --- | --- | --- |
| Usability — E04–E11; ordinary browser account journeys; independent completion | U01–U09 and M01/M02 original rows DONE; E-CI browser setup 9 passes and authenticator journeys 22 passes/2 skips; historical D01 partial browser/CLI observations (E-GUIDE). Software WebAuthn and CSS viewports stay labeled. | Independent nontechnical user completion of invitation, passkey, app access, factor/session management and recovery is unrecorded. D01 remains open; physical/synced/phone/real-mail/browser-OS evidence is not supplied by shims. Accessibility remains canceled, not proven or restarted. Obtain the authorized independent journey record via D01. |
| Workflows — P01 and G05/G06 bound proofs; Platform conditional behavior with Essentials defaults | W01–W07 original rows DONE; E-WORK conditional path 6/6 and corrected enrollment 1/1; E-GUEST 19 native cases; reviewed workflow activation/shared optional-required header envelope E-MGMT. Existing configured consent 8/8 plus TOTP 1/1 remain historical. | An independent operator's documented supported workflow setup/completion is not recorded. No universal graph/verifier/protocol permutation is promised; unsupported native hosts refuse. Map D01's operator completion to the supported graph, rather than rebuilding W02/W07 or adding arbitrary-graph acceptance. |
| Administration — E12–E17, P14–P16, G07 equivalent authorized changes | M01–M07 DONE; A03 DONE. Protected connector plan/apply/export/first-owner secret origin and restart-loaded status; E-MGMT shared envelope/receipt/audit evidence, E-S04 relevant invalidation with reuse, E-DIAG bounded actionable reads. | Independent Applications/People/Groups/Security operator journey is unrecorded. O06 wider facets stay open; browser-bound ceremonies and dedicated resources remain explicit boundaries. Retain reviewed creation receipt-secret, route-specific header and PAM fallback contracts. Do not make universal browser resource coverage a new M03 gate. |
| Interoperability — EG01 named product/version/profile/direction/lifecycle/failure | E-CI actual XMLsec, OpenLDAP, nginx/Traefik and real browser fixture executions; older OpenLDAP provider 2.7.1 and FreeRADIUS radclient 3.2.10 (E-PEER); D01 local Lasso/XMLsec observations. P06 and I05 are DONE for supported SCIM/direct Workspace implementation (E-SCIM, E-WSPACE). | Q03 independent plan and named tenant/customer/provider rollouts are unrecorded; no broad AD/Entra/Workspace/Vault/NAS/device compatibility certification. Local peer programs count as genuine peer evidence only for their exact loopback profiles. Ask the owning Q03/Q04/integration rows for their named inputs, not an invented tenant run. |
| Footprint — G04/G09; EG02 exact native artifacts and measured cost | A04–A08 DONE; E-ARM five native ELF64 AArch64 binaries and local archives/images, dependency closures and known byte sizes; E-Q09 historical RSS observations on release-profile local binaries. Current Essentials/Platform boundaries remain additive. | Official same-revision shipped x86-64/ARM64 package/container identities, installation and exact bundle gates remain Q08/Q10/A09/Q11 work. No current-main artifact size/RSS or general memory bound is claimed. Held Group memory limits remain separate. Local production binaries count without pretending they are a published release. |
| Performance — S/O/R/Q equivalent functionality/security; EG04 workload, latency/RSS/throughput/errors/interference | S01/S03/S04 DONE. E-S04 measures actual writer-cost removal with equivalent authority/security and ordered concurrency; E-Q09 records real paced secure session reads, both editions/backends, maintenance overlap, errors and RSS. These supersede blanket absence/fixture-only claims. | Named deployment workload, capacity/lag budgets and load/RPO/RTO objectives remain absent. Q09's pacing makes success/s an observation, not maximum throughput; tiny sample p95/p99 are maxima. No general/statistical speed, whole-deployment capacity or large-Group guarantee. Use existing measurements for bounded claims; request operator thresholds only for broader claims. |
| Availability — probes, single redb owner, defined distributed coordination and fenced PG behavior | O01–O05 DONE; E-O03 issuer/capability/auth/rate agreement, shared lease/admission/cache contracts and explicit offline adoption; E-CI disposable PG replication/promotion/reconnection, 1113 ms logged recovery; E-PAGE restored controller paging. | Deployment topology, external HA/fencing/partition/failback and objective-linked outage/error/RPO/RTO evidence remain unrecorded; O07 is open. Local readiness cannot prove a healthy peer fleet. Sixty-second nonrenewed admission/paused-before-IO limits remain; no lease-expiry inference of worker quiescence. |
| Recovery — R01–R05, G12 and EG03 invalidation/reconciliation | R01–R04 DONE. E-REC logical restore 16/16 per edition on encrypted local PG; real physical PG16.14 base-backup drill 16 checks, old-artifact refusal and gated serving. D04 key-loss/TLS/dump/base/PITR reports retain their actual outcomes and blind spot. | R05/D04 fresh-operator/application-flow completion, external escrow/files/signers/peers and measured deployment recovery/data loss remain open. Physical same-lineage status can say serving allowed before offline invalidation; its pending gate never supplies a completion attestation. Logical fixture completion does not establish a real RP login or PITR deployment readiness. |
| Migration — G01–G05; deliberate identity/credential mapping, cutover and rollback | G01–G04 DONE; E-MIG accepted issuer/subject/group/app/credential-reference/source-link conversion, target-bound reimport/refusals; historical G05 in-process reference RP 1/1 with eight cases. A08 native plain/encrypted edition transitions are E-ARM evidence, not customer migration. | G05 real export/application callback/access/MFA/refresh/logout/recovery/route rollback remains unrecorded. State exactly what transfers versus requires re-enrollment/manual work. No real Authentik export, production subject continuity, cutover duration or older-binary rollback is invented. |
| Security — Q01 contracts, Q02/Q05 execution, G01–G12/EG03, scoped release assurance | Q01/Q02/Q05/Q07 DONE; E-CI 91 named PG contracts across the actual selected plain/encrypted cases plus Q05 schedule; E-GUEST native confinement; E-MGMT authority/replay/secret contracts; E-O03 format-3 pre-write refusal; E-S04 ordered atomic refusal; E-REC restored-artifact denial. E-RELEASE source tooling and local unsigned inventories exist. | Installed current/previous release-artifact EG03, scoped independent conformance/review/signature/attestation/SBOM/publication evidence remain unproved here. A workflow attestation request or source/local unsigned SPDX document is not its execution. Slice reviews do not establish certification. Retain security scenarios and claim limits; no report-alone completion. |

## Evidence records proposed for the refresh

All paths below were read at the fixed pin unless a different source/run pin is
explicit. Commands here are historical citations, **not commands executed by
this evaluation**. Do not add their counts together as independent categories.

| ID | Exact evidence, execution and attribution |
| --- | --- |
| E-CI | Downloaded `/tmp/riauth-wave29-integration-110661000640.log`, SHA-256 `458a30895b5987c0bebe2fb7368a1a32969fa0d41ce90acbfa56f43299ccd186`, run `36950097067`, checkout `2d05c00deed3a6dafcd458a5f1a1a9beb17d0dd8`. Root verified this named job SUCCESS; this audit verified raw hash/commands/results. Three XMLsec filters under `--test identity --locked -- --ignored` each passed 1. Default `scripts/test-postgres.sh` selects `--test postgres -- --ignored --nocapture --test-threads=1`: 2 passed/17.51s. Shared script selects `--features test-support --test contracts -- --ignored postgres --test-threads=2 --nocapture`: 91 passed/90 filtered/209.68s. `RIAUTH_PG_TEST_TARGET=q05_replay_concurrency` selected one ignored PostgreSQL schedule test: 1 passed/1 filtered/13.98s. OpenLDAP, Rust browser, portal_browser, nginx outpost and Traefik tests each passed 1; Playwright setup 9, authenticator step 22 with 2 skipped Firefox/WebKit invitation-passkey cases. Default Cargo edition was Platform. No dedicated node_security_postgres/job/mail/SSF peer target ran, and the 91 is not 91 O03 tests. |
| E-WORK | `local-wave29-w02-closure-root-review.md`, conditional evidence and source executor/model. Accepted conditional code `a03731379e259ed8252d09db5a47d9445776caae`: `cargo test --locked --features test-support,fuzzing --test workflow_configured_conditional_enrollment -- --test-threads=1`, 6/6. Separate corrected existing enrollment source `24a09dff4cbb6adf9a385913b92ac82a7b0cf41e`, accepted `e5f5facddab4f22651ae671a3c6e864ef504c39a`: exact named regression 1/1. Counts are separately attributed historical local redb/software-WebAuthn execution, not repeated here. |
| E-GUEST | `local-wave27-extension-isolation-report.md` and six-task review/root disposition; native source `d772d29fbfc4e681774593ebbe32839df5f5f032`; `cargo test --locked --lib workflow::extension_gate::tests -- --test-threads=1`, 19 passed/95 filtered/33.06s. Accepted source equivalence and log SHA `619315c304faf46a656024e1bda649e48f703d5c758fcae088a2f99a7b45701e` were verified by the independent/root reviews; this audit read those records, not the raw guest log. Native macOS confinement/fuel/deadline/descriptor/proof limits, unsupported-host refusal; no other-OS or released-guest proof. |
| E-MGMT | `local-wave30-m03-root-disposition.md`, `local-wave29-activation-header-parity-root-review.md`, protected connector port and original management completion matrices. Activation source `1a21871b477d5e58d89cd82130f8e9c5b63a07d6`: historical 38 API functions (including 8 new Core/browser cases), 10 approval functions with 6 PG cases ignored, 2 configured-adapter functions; subsets/repeats are not extra passes. Core Optional and bearer Required share authority-before-receipt, first-only revision checks and stale sealing. M03/M07 DONE, backed by current source and accepted code/evidence, not the old pending disposition. Reviewed client-creation receipt recovery, first-only credential behavior and PAM fallback are deliberate accepted contracts. |
| E-O03 | `local-wave29-o03-final-disposition.md` and root closure; format-3 accepted code/rate report. Historical current-format local `node_security::` checks: 7 after recorded fixture correction; exact all-16 HTTP threshold and explicit maintenance adoption fixtures: 1 each. Dedicated three-process issuer/capability/auth mismatch checks and logout 1/mail 2/SSF 2 worker cases belong to older format-1/2-era PG evidence. Local admission module 2, SCIM freshness/stale-publication/invalidation and recovery-stamp exact filters passed under their recorded wave27 pins. Do not call older process checks current-format-3 peers or claim old binary execution. |
| E-S04 | Source `5120a0ddb51d015fbf89fbca19da127444e26f8b`, accepted `57ed95fef056afc9402a65c4b576d0461b7fed30`, report `local-wave30-s04-completion-plan.md` and root disposition. Exact `cargo test --locked --features test-support,fuzzing --test state_reconciliation s04_scoped_reuse_equivalent_authority_workload -- --exact --ignored --test-threads=1 --nocapture`: 1 passed/17 filtered/20.47s. Raw log hash `6287eb35f69017af138304ca7ca96624765d7a7b60ba49efab936ae2781e0e6d` reverified without rerunning. Four families/eight writes/three AB-BA-AB pairs: 96 calls per lane; global control 192 writer holds/96 commits/749695 µs occupancy, scoped reuse zero; 495196 µs commit time is included in occupancy. Sixteen apply refusals, 24 ordered two-thread persistence cases and 24 apply/replay/current-permission cases passed. Same-build conservative reuse control follows identical security checks; counts/cost are not general throughput. Root `d23dabbf9c8a9c150987e18f4f416bf93f52562a` removes one needless test borrow, without remeasurement. |
| E-PAGE | Pagination source `9810c33edba8027433c1a4e24442d878710b9a8a`, accepted `1955cf28d52861cacf33fe8626342a2ab8954492`; unchanged `tests/reconciliation_jobs.rs` blob `aa1c3bdc70b232796dfb0c150179b8165384b85a`. Exact locked test-support,fuzzing named filter/single-thread: fixed-da5 baseline exit101, 0pass/1fail/1filtered/1.44s at line229; corrected exit0, 1pass/1filtered/1.71s. Raw log SHAs `5e8be0ae942ccbed9118b9b76a03bf6dc1383d762e13e938ef0a3eff3ae0c7fb` and `2cc8e8b515edf992ed22d42ea4e596ae8ae45df75271ddc6954537754bcd17b5` reverified. Current finish/owner-generation release body preserved; current whole file also includes the separately accepted O06 diagnostic reader. No new Linux or release-failure fault injection is inferred. |
| E-DIAG | Current O06 storage/key/controller/missing-evidence/gauge reports and root reviews. Storage gives explicit capacity-unmeasured remedies; key output refuses to invent custody/health; configured-unscheduled controllers are visible without writes. Configured-controller exact fixture 1 pass/1.06s; gauge exact fixture initially failed stale 422/400 expectation, then 1 pass/2.14s after the recorded fixture-only correction. Gauge revision8→9 repair preserves unresolved uncertainty; resolution/waiver is not delivery. This is accepted bounded diagnostic evidence, not whole O06 or physical-pressure completion. |
| E-ARM | Tracked A08 `RUN.md`, `native-arm64-evidence-v2.json`, encrypted supplement and `integration-review.json`. Product binaries `f3aba63ac3b824843a40b99623f1619ef8edc19f`; plain validator `730d373078c4b128373d7afa5f3e278f63b065fd`, encrypted validator `9f8efaf0372e02b0bc0042251d591f1facc0f783`, recovery supplement `082563b362acb218f276e4dbcaf476e303b561b7`. Native Linux AArch64 plain/encrypted redb/PG edition handoffs, identity/auth/refusal checks and local archives/images. Encrypted logical rows retained 30/33 redb and 36/39 PG with raw ciphertext equality. Historical format2, not a current format3 execution. Local binary sizes: server Essentials 36390184 B, Platform 49302784 B; maintenance 13310224/16783720 B; client 8101312 B. Earlier `6ca4779` late smoke/installed-transition failures remain historical and are not erased by later local transition successes. `release_gate_result: false`; no official release identity. |
| E-Q09 | Current `q09-benchmark-slice.md`, including later paced macOS, plaintext ARM64 and secure ARM64 sections, rather than only its old self-check. Secure harness `4ab934a44a80062dda0dba5d8640d396631a3ef0` and release-profile product copies built from `6ca4779b68cf41e29af490d2aca6ea3d59e1f2bd` ran four edition/backend combinations with private-CA HTTPS, aes256gcm-v1 and PG SCRAM/hostssl; product file sslmode=require, libpq verification uses verify-full. Each quiet/interference pass: 12 reads/12 successes/no read errors, maintenance delta1, 71.57–71.66s; writer 2success/2expected409. Twelve samples make p95/p99 the maximum; 6500ms pacing is inside reported success/s, outside latency samples. Historical recorded RSS maxima 50408–55992 KiB across these passes; no external signer/published artifact/current-main or maximum capacity claim. Accepted page/hash references are read here; external report files are not rehashed in this audit. |
| E-REC | Tracked encrypted A08 PG restore JSONs: each edition 16 passed checks, binary SHAs `86a799d3060e5e602c7e458a9c409ffc8c06f881e2d426a1e6d9be226337445d` / `78209c1766eb53be741de0946a0e4ce4bc3f2217deda84a6e4f4bda85fc42b62`; outage, wrong key/occupied target, gated serve, explicit fixture reconciliation, old-session refusal and fresh service user/admin login. Separately tracked R05 native PG16.14 physical-base-backup report has 16 checks and binary SHA `a5c01650917d16c9384b1d87fba373d02e97d5e85eb8a2e625d7c9a8cbcd68ea`, without a source-commit field: same-lineage blind spot, offline invalidation and artifact/serve refusal, gate left pending. These are distinct drills, not 32 full deployed recovery passes. D04 manual physical/PITR observations and logical completion limits stay explicit. |
| E-MIG | Accepted six-task review/root disposition reads conversion/reimport source and assertions; `convert`/`classify_issuer` are equivalent to executed `3ddcab4d43cfa1ce41222b6253dedbed573c0c64` at its reviewed pin. G01–G04 original rows DONE; G05 `cf827a9` synthetic reference RP rehearsal 1/1 with eight case IDs remains historical, not customer migration. No real-export or rollback claim. |
| E-SCIM / E-WSPACE | Six-task independent/root review and source/assertions: P06 supported core schemas, conditional tombstone disable/revocation/downstream intent and resource-version controls; redb 130-user Group replace/replay exact fixture 1/1 and plain/encrypted PG cases 2/2 at accepted `7b9253117e51a0676988de5ea26fb43dc2c627e5`. I05 direct broker-optional signed service-account grant and endpoint/key/expiry/failure checks; `4df94b61edf9161cac5217b00691e55fdb6ee6ed` recorded cloud_directory test-support 36/36. Local fake peers, no cloud mutation or real tenant. |
| E-PEER / E-GUIDE | Older actual OpenLDAP provider 2.7.1 LDAPS/STARTTLS and FreeRADIUS radclient 3.2.10 PAP accept/reject/replay remain named local peers. `d01-platform-cli-walkthrough.md` records eleven partial disposable guide runs, including separate remote client/maintenance, OpenLDAP, loopback SCIM, XMLsec and GNU Lasso. Binary snapshots/partial browser paths and refusals are retained; they are not eleven complete new-user/operator journeys. Essentials guide explicitly withholds such completion. |
| E-RELEASE | Current `q11-release-evidence.md`, packager/checker/attestation source and SECURITY.md. Local unsigned SPDX inventories tied to preserved A08 assets list 301/353/121 packages; these differ deliberately from older normal closure counts 280/329/106. Current workflow requests pinned attestations; this audit found no executed release bundle/signature/attestation/publication record at the pin. Do not say tooling or local inventories are absent, or extrapolate absence beyond the inspected evidence. |

## Provenance transfer and actual checks in this audit

The downloaded integration result belongs to predecessor `2d05c00`, not a
fresh `c01c39a` run. Immutable comparisons found its node_security, config,
target admission, token-freshness helper, PostgreSQL/Q05/shared-contract test,
CI and two PG script blobs unchanged at this pin. State changed after that
run and receives its separate E-S04 evidence. Other changed adapters are not
credited with new executions merely because that older integration passed.

At the evaluation pin, directly read startup agreement-before-writes,
shared admission owner/generation release, SCIM stamp publication/invalidation,
state reuse/persistence/apply gates and reconciliation finish/settlement.
Selected implementation/tests and accepted independent reviews support the
implementation dispositions; neither an existing report nor a board status
alone is treated as product evidence. Source and artifact hashes identify
exact records, not an execution of every named path.

Pinned anchors: state blob `97c57ac0b3040961139258651d0ae39b6c15c825`;
node-security `da029997ad9c1a805cc3c6a1f6a50954f79a16c3`;
reconciliation `f15ef16a97091817418ce95b14f3d59583caadc8`;
S04 workload `bd010df7904baa0c36daec2c34e964978f36dbd4`.
S04/PAGE worker hashes are not direct main ancestors; their reviewed ports
and equivalence records provide acceptance provenance. Never import their
whole old files. Native A08/Q09 runs remain their own older binary pins.

The current ledger note reports Public CI `36998781947` pending at
publication; earlier integration/audit successes coexist with unrelated
failed check jobs. No live GitHub query or overall-green claim is made here.
The last pagination local baseline/corrected execution and S04 workload were
not repeated. Document/path/JSON inspection, immutable comparisons, log hash
and result filtering and whitespace/single-file scope checks are the only
checks performed by this proposal. The first object-reference guard accidentally
classified a numeric GitHub run ID as hexadecimal and stopped; excluding numeric
IDs corrected that document-check script. A second filename guard assumed this
new report named itself; narrowing the expected nonrepository name to AGENTS.md
corrected that check. A draft closing-backtick typo was also fixed. None changed
evidence, existing artifacts or product source.
CONTRIBUTING.md and SECURITY.md were read;
no applicable ancestor/repository/docs AGENTS.md was found. The bounded
no-runtime assignment controls over the contributor guide's general campaign.

## Smallest remaining evaluation work and handoff

After root reviews this exact proposal, refresh **only the two D05 artifacts**
against the fixed pin: populate these records and ten mappings, preserve
historical failures/pins, distinguish implementation completion from scoped
claim gaps, and validate JSON/Markdown correspondence, links, paths and
whitespace. No Cargo/runtime is needed for that evidence reconciliation.
Readiness retains the sole Cargo slot.

The smallest concrete input still needed for the original D05 workstream
gate is **one independently recorded documented journey for a new user and a
new operator**, using the selected matching artifacts/configuration and
ordinary supported Essentials path through invitation, factor/app/session
management and recovery, plus operator setup/management and safe recovery
verification. D01 owns that walkthrough; use its actual results when reviewed
rather than launch a duplicate service/browser campaign here. Record which
steps need physical authenticators, mailbox or external application and which
could complete locally. Wider category measurements/tenants/releases remain
separately named inputs, not a universal fresh-run or universal HA condition.

No implementation, build/test/benchmark, service, desktop, secret inspection,
cloud/PG mutation, external lookup/message, new task/worker/worktree/managed
shell, board change, main/accepted edit or push occurred. Only explicit-project
root coordination is used. Any future desktop work uses RiWork Cua.ai Driver
MCP after reading descriptions/current state and reports missing setup or
permissions; no provider switch. Preserve held Group and canceled accessibility,
reviewed receipt-secret behavior, route-specific optional/required headers,
PAM fallback, permission/review/receipt/removal/audit/credential protections,
sixty-second nonrenewed admission and paused-before-IO limits.

Root alone authorizes the two-artifact edit, reviews/integrates/pushes and
decides D05 status. This proposal changes no task disposition.
