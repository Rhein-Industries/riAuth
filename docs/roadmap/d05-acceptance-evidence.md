# D05 acceptance evidence matrix

**Current status: in_progress; evaluated_with_gaps; full D05 gate false.**
Snapshot/read date: 2026-10-02. Fixed published evidence pin: `c01c39ab4e092423d5522bedc50fff87656d8c0a`.
Dated live original-row read: `2026-10-02T11:40:43.377271+00:00`; this metadata is separate from pinned source.

The [JSON twin](d05-acceptance-evidence.json) retains schema `riauth.acceptance-evidence/v1`.
This follows the approved [proposal](local-wave30-d05-evaluation-plan.md), commit `da787b4525a368220c63995ea1129d5f7243754e`.
Only these two evaluation artifacts change.

## Original outcome and gate

Evaluate usability, workflows, administration, interoperability, footprint, performance, availability, recovery, migration, and security evidence.

Document completed user and operator tasks, not merely available settings.

Full gate: **A new user and a new operator can independently complete the documented workflows.**

Original-row implementation acceptance is distinct from scoped category/advertised-claim evidence. Historical passes retain exact source/artifact/command provenance; missing inputs remain explicit. Full D05 gate stays false pending independent documented new-user/new-operator completion. No blanket alltodo/allpassed, universal fresh-run or report-alone code completion. Root owns statuses.

Targets use [A02 product contracts](product-contracts.md), G01–G12 and EG01–EG04.
A02 E/P/G IDs are capability/gate IDs; accepted_rows below are original backlog IDs.

## Provenance and live state

Own branch `roadmap/local-revisions-coordination-wave27`, pre-refresh HEAD `da787b4525a368220c63995ea1129d5f7243754e`,
existing WT `ed9ac424-59f4-4520-905b-919aea3521eb`; project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`,
D05 task `6a2381ab-0f2f-48c6-adf9-8115b78cc143`. Own product tree is not asserted equal to the pin.
Source was read from immutable objects without merge/reset/import or main/accepted edits.
Accepted integration `c8d6f87915ac949a71f4ed9028a2f5cc350e77dc` and pushed main are the
publication observation in the approved proposal, not a new remote lookup.

S04/O03 remain DONE. O06 `9899f8e6-05ff-4e11-b9a0-b9ca184221a6` remains **in_progress**
at the dated board read; E-DIAG covers accepted earlier slices at c01c39a, not active/new work.
D01/O07/R05 remain active. Other Q08/Q09/Q10/Q11/tenant/cutover inputs stay in their own rows.
The JSON retains exact observed task UUIDs/statuses/worktree assignments; this refresh changes none.

Old integration counts, all-not-passed flags, slice-only status freeze and stale ledger CI
subobjects are historical. The prior complete snapshot below does not override this evaluation.

## Evidence kinds

Source/criteria inspection, executed local fixture, named local peer, native local artifact
and release/deployment evidence remain distinct. Each record identifies whether existing raw
logs, tracked result JSON or accepted source/review records were inspected. Passing software
fixtures/peer utilities do not certify all peers, hardware or shipped bundles.

## Ten-category current evaluation

| Category | Status | Accepted scope and evidence | Unmet input / next evaluation |
| --- | --- | --- | --- |
| Usability | evaluated_with_gaps; full target false | U01–U09 and M01/M02 original rows DONE; E-CI browser setup 9 passes and authenticator journeys 22 passes/2 skips; historical D01 partial browser/CLI observations (E-GUIDE). Software WebAuthn and CSS viewports stay labeled. | Independent nontechnical user completion of invitation, passkey, app access, factor/session management and recovery is unrecorded. D01 remains open; physical/synced/phone/real-mail/browser-OS evidence is not supplied by shims. Accessibility remains canceled, not proven or restarted. Obtain the authorized independent journey record via D01. |
| Workflows | evaluated_with_gaps; full target false | W01–W07 original rows DONE; E-WORK conditional path 6/6 and corrected enrollment 1/1; E-GUEST 19 native cases; reviewed workflow activation/shared optional-required header envelope E-MGMT. Existing configured consent 8/8 plus TOTP 1/1 remain historical. | An independent operator's documented supported workflow setup/completion is not recorded. No universal graph/verifier/protocol permutation is promised; unsupported native hosts refuse. Map D01's operator completion to the supported graph, rather than rebuilding W02/W07 or adding arbitrary-graph acceptance. |
| Administration | evaluated_with_gaps; full target false | M01–M07 DONE; A03 DONE. Protected connector plan/apply/export/first-owner secret origin and restart-loaded status; E-MGMT shared envelope/receipt/audit evidence, E-S04 relevant invalidation with reuse, E-DIAG bounded actionable reads. | Independent Applications/People/Groups/Security operator journey is unrecorded. O06 wider facets stay open; browser-bound ceremonies and dedicated resources remain explicit boundaries. Retain reviewed creation receipt-secret, route-specific header and PAM fallback contracts. Do not make universal browser resource coverage a new M03 gate. |
| Interoperability | evaluated_with_gaps; full target false | E-CI actual XMLsec, OpenLDAP, nginx/Traefik and real browser fixture executions; older OpenLDAP provider 2.7.1 and FreeRADIUS radclient 3.2.10 (E-PEER); D01 local Lasso/XMLsec observations. P06 and I05 are DONE for supported SCIM/direct Workspace implementation (E-SCIM, E-WSPACE). | Q03 independent plan and named tenant/customer/provider rollouts are unrecorded; no broad AD/Entra/Workspace/Vault/NAS/device compatibility certification. Local peer programs count as genuine peer evidence only for their exact loopback profiles. Ask the owning Q03/Q04/integration rows for their named inputs, not an invented tenant run. |
| Footprint | evaluated_with_gaps; full target false | A04–A08 DONE; E-ARM five native ELF64 AArch64 binaries and local archives/images, dependency closures and known byte sizes; E-Q09 historical RSS observations on release-profile local binaries. Current Essentials/Platform boundaries remain additive. | Official same-revision shipped x86-64/ARM64 package/container identities, installation and exact bundle gates remain Q08/Q10/A09/Q11 work. No current-main artifact size/RSS or general memory bound is claimed. Held Group memory limits remain separate. Local production binaries count without pretending they are a published release. |
| Performance | evaluated_with_gaps; full target false | S01/S03/S04 DONE. E-S04 measures actual writer-cost removal with equivalent authority/security and ordered concurrency; E-Q09 records real paced secure session reads, both editions/backends, maintenance overlap, errors and RSS. These supersede blanket absence/fixture-only claims. | Named deployment workload, capacity/lag budgets and load/RPO/RTO objectives remain absent. Q09's pacing makes success/s an observation, not maximum throughput; tiny sample p95/p99 are maxima. No general/statistical speed, whole-deployment capacity or large-Group guarantee. Use existing measurements for bounded claims; request operator thresholds only for broader claims. |
| Availability | evaluated_with_gaps; full target false | O01–O05 DONE; E-O03 issuer/capability/auth/rate agreement, shared lease/admission/cache contracts and explicit offline adoption; E-CI disposable PG replication/promotion/reconnection, 1113 ms logged recovery; E-PAGE restored controller paging. | Deployment topology, external HA/fencing/partition/failback and objective-linked outage/error/RPO/RTO evidence remain unrecorded; O07 is open. Local readiness cannot prove a healthy peer fleet. Sixty-second nonrenewed admission/paused-before-IO limits remain; no lease-expiry inference of worker quiescence. |
| Recovery | evaluated_with_gaps; full target false | R01–R04 DONE. E-REC logical restore 16/16 per edition on encrypted local PG; real physical PG16.14 base-backup drill 16 checks, old-artifact refusal and gated serving. D04 key-loss/TLS/dump/base/PITR reports retain their actual outcomes and blind spot. | R05/D04 fresh-operator/application-flow completion, external escrow/files/signers/peers and measured deployment recovery/data loss remain open. Physical same-lineage status can say serving allowed before offline invalidation; its pending gate never supplies a completion attestation. Logical fixture completion does not establish a real RP login or PITR deployment readiness. |
| Migration | evaluated_with_gaps; full target false | G01–G04 DONE; E-MIG accepted issuer/subject/group/app/credential-reference/source-link conversion, target-bound reimport/refusals; historical G05 in-process reference RP 1/1 with eight cases. A08 native plain/encrypted edition transitions are E-ARM evidence, not customer migration. | G05 real export/application callback/access/MFA/refresh/logout/recovery/route rollback remains unrecorded. State exactly what transfers versus requires re-enrollment/manual work. No real Authentik export, production subject continuity, cutover duration or older-binary rollback is invented. |
| Security | evaluated_with_gaps; full target false | Q01/Q02/Q05/Q07 DONE; E-CI 91 named PG contracts across the actual selected plain/encrypted cases plus Q05 schedule; E-GUEST native confinement; E-MGMT authority/replay/secret contracts; E-O03 format-3 pre-write refusal; E-S04 ordered atomic refusal; E-REC restored-artifact denial. E-RELEASE source tooling and local unsigned inventories exist. | Installed current/previous release-artifact EG03, scoped independent conformance/review/signature/attestation/SBOM/publication evidence remain unproved here. A workflow attestation request or source/local unsigned SPDX document is not its execution. Slice reviews do not establish certification. Retain security scenarios and claim limits; no report-alone completion. |

### Usability

Target: E04–E11; ordinary browser account journeys; independent completion

Accepted original rows: `U01`, `U02`, `U03`, `U04`, `U05`, `U06`, `U07`, `U08`, `U09`, `M01`, `M02`.
Evidence IDs: `E-CI`, `E-GUIDE`.
Active rows remain separate: `D01`.

Fulfilled scope: U01–U09 and M01/M02 original rows DONE; E-CI browser setup 9 passes and authenticator journeys 22 passes/2 skips; historical D01 partial browser/CLI observations (E-GUIDE). Software WebAuthn and CSS viewports stay labeled.

Unmet input / next evaluation: Independent nontechnical user completion of invitation, passkey, app access, factor/session management and recovery is unrecorded. D01 remains open; physical/synced/phone/real-mail/browser-OS evidence is not supplied by shims. Accessibility remains canceled, not proven or restarted. Obtain the authorized independent journey record via D01.

Claim limits: Software authenticators and CSS viewports are not physical hardware/mobile OS/external mail. Accessibility remains canceled and unproved.

### Workflows

Target: P01 and G05/G06 bound proofs; Platform conditional behavior with Essentials defaults

Accepted original rows: `W01`, `W02`, `W03`, `W04`, `W05`, `W06`, `W07`.
Evidence IDs: `E-WORK`, `E-GUEST`, `E-MGMT`.
Active rows remain separate: `D01`.

Fulfilled scope: W01–W07 original rows DONE; E-WORK conditional path 6/6 and corrected enrollment 1/1; E-GUEST 19 native cases; reviewed workflow activation/shared optional-required header envelope E-MGMT. Existing configured consent 8/8 plus TOTP 1/1 remain historical.

Unmet input / next evaluation: An independent operator's documented supported workflow setup/completion is not recorded. No universal graph/verifier/protocol permutation is promised; unsupported native hosts refuse. Map D01's operator completion to the supported graph, rather than rebuilding W02/W07 or adding arbitrary-graph acceptance.

Claim limits: Supported graphs and native macOS guest isolation do not establish arbitrary graphs or other-host guest execution. Essentials keeps built-in defaults.

### Administration

Target: E12–E17, P14–P16, G07 equivalent authorized changes

Accepted original rows: `M01`, `M02`, `M03`, `M04`, `M05`, `M06`, `M07`, `A03`, `S04`.
Evidence IDs: `E-MGMT`, `E-S04`, `E-DIAG`.
Active rows remain separate: `D01`, `O06`.

Fulfilled scope: M01–M07 DONE; A03 DONE. Protected connector plan/apply/export/first-owner secret origin and restart-loaded status; E-MGMT shared envelope/receipt/audit evidence, E-S04 relevant invalidation with reuse, E-DIAG bounded actionable reads.

Unmet input / next evaluation: Independent Applications/People/Groups/Security operator journey is unrecorded. O06 wider facets stay open; browser-bound ceremonies and dedicated resources remain explicit boundaries. Retain reviewed creation receipt-secret, route-specific header and PAM fallback contracts. Do not make universal browser resource coverage a new M03 gate.

Claim limits: Preserve reviewed client-creation receipt-secret, first-only disclosure, per-route optional/required headers and PAM fallback; dedicated APIs and browser-bound ceremonies remain explicit boundaries.

### Interoperability

Target: EG01 named product/version/profile/direction/lifecycle/failure

Accepted original rows: `D02`, `I01`, `I03`, `I05`, `P06`.
Evidence IDs: `E-CI`, `E-PEER`, `E-GUIDE`, `E-SCIM`, `E-WSPACE`.

Fulfilled scope: E-CI actual XMLsec, OpenLDAP, nginx/Traefik and real browser fixture executions; older OpenLDAP provider 2.7.1 and FreeRADIUS radclient 3.2.10 (E-PEER); D01 local Lasso/XMLsec observations. P06 and I05 are DONE for supported SCIM/direct Workspace implementation (E-SCIM, E-WSPACE).

Unmet input / next evaluation: Q03 independent plan and named tenant/customer/provider rollouts are unrecorded; no broad AD/Entra/Workspace/Vault/NAS/device compatibility certification. Local peer programs count as genuine peer evidence only for their exact loopback profiles. Ask the owning Q03/Q04/integration rows for their named inputs, not an invented tenant run.

Claim limits: Named utilities prove only exercised local peer versions/directions/profiles, not family-wide certification or a named tenant.

### Footprint

Target: G04/G09; EG02 exact native artifacts and measured cost

Accepted original rows: `A04`, `A05`, `A06`, `A07`, `A08`.
Evidence IDs: `E-ARM`, `E-Q09`.

Fulfilled scope: A04–A08 DONE; E-ARM five native ELF64 AArch64 binaries and local archives/images, dependency closures and known byte sizes; E-Q09 historical RSS observations on release-profile local binaries. Current Essentials/Platform boundaries remain additive.

Unmet input / next evaluation: Official same-revision shipped x86-64/ARM64 package/container identities, installation and exact bundle gates remain Q08/Q10/A09/Q11 work. No current-main artifact size/RSS or general memory bound is claimed. Held Group memory limits remain separate. Local production binaries count without pretending they are a published release.

Claim limits: Source-built local artifacts and historical RSS are distinct from current-main shipped artifact support. Held Group/memory work is separate.

### Performance

Target: S/O/R/Q equivalent functionality/security; EG04 workload, latency/RSS/throughput/errors/interference

Accepted original rows: `S01`, `S03`, `S04`.
Evidence IDs: `E-S04`, `E-Q09`.

Fulfilled scope: S01/S03/S04 DONE. E-S04 measures actual writer-cost removal with equivalent authority/security and ordered concurrency; E-Q09 records real paced secure session reads, both editions/backends, maintenance overlap, errors and RSS. These supersede blanket absence/fixture-only claims.

Unmet input / next evaluation: Named deployment workload, capacity/lag budgets and load/RPO/RTO objectives remain absent. Q09's pacing makes success/s an observation, not maximum throughput; tiny sample p95/p99 are maxima. No general/statistical speed, whole-deployment capacity or large-Group guarantee. Use existing measurements for bounded claims; request operator thresholds only for broader claims.

Claim limits: Actual bounded S04 writer-cost improvement is established; counts and elapsed are not statistical/general throughput. Q09 pacing/sample limits are retained.

### Availability

Target: probes, single redb owner, defined distributed coordination and fenced PG behavior

Accepted original rows: `O01`, `O02`, `O03`, `O04`, `O05`.
Evidence IDs: `E-O03`, `E-CI`, `E-PAGE`.
Active rows remain separate: `O06`, `O07`.

Fulfilled scope: O01–O05 DONE; E-O03 issuer/capability/auth/rate agreement, shared lease/admission/cache contracts and explicit offline adoption; E-CI disposable PG replication/promotion/reconnection, 1113 ms logged recovery; E-PAGE restored controller paging.

Unmet input / next evaluation: Deployment topology, external HA/fencing/partition/failback and objective-linked outage/error/RPO/RTO evidence remain unrecorded; O07 is open. Local readiness cannot prove a healthy peer fleet. Sixty-second nonrenewed admission/paused-before-IO limits remain; no lease-expiry inference of worker quiescence.

Claim limits: 60-second nonrenewed shared admission and stale completion do not exclude paused external IO after reclaim. Local readiness/disposable promotion are not deployed HA or fleet health.

### Recovery

Target: R01–R05, G12 and EG03 invalidation/reconciliation

Accepted original rows: `R01`, `R02`, `R03`, `R04`.
Evidence IDs: `E-REC`, `E-ARM`.
Active rows remain separate: `R05`.

Fulfilled scope: R01–R04 DONE. E-REC logical restore 16/16 per edition on encrypted local PG; real physical PG16.14 base-backup drill 16 checks, old-artifact refusal and gated serving. D04 key-loss/TLS/dump/base/PITR reports retain their actual outcomes and blind spot.

Unmet input / next evaluation: R05/D04 fresh-operator/application-flow completion, external escrow/files/signers/peers and measured deployment recovery/data loss remain open. Physical same-lineage status can say serving allowed before offline invalidation; its pending gate never supplies a completion attestation. Logical fixture completion does not establish a real RP login or PITR deployment readiness.

Claim limits: Same-lineage physical restore needs explicit offline invalidation and credential/key reconciliation. Local logical completion is not real RP flow, escrow or deployed PITR proof.

### Migration

Target: G01–G05; deliberate identity/credential mapping, cutover and rollback

Accepted original rows: `G01`, `G02`, `G03`, `G04`, `A08`.
Evidence IDs: `E-MIG`, `E-ARM`.

Fulfilled scope: G01–G04 DONE; E-MIG accepted issuer/subject/group/app/credential-reference/source-link conversion, target-bound reimport/refusals; historical G05 in-process reference RP 1/1 with eight cases. A08 native plain/encrypted edition transitions are E-ARM evidence, not customer migration.

Unmet input / next evaluation: G05 real export/application callback/access/MFA/refresh/logout/recovery/route rollback remains unrecorded. State exactly what transfers versus requires re-enrollment/manual work. No real Authentik export, production subject continuity, cutover duration or older-binary rollback is invented.

Claim limits: Accepted conversion and target-bound reimport do not prove real customer cutover, production route rollback or older executable compatibility.

### Security

Target: Q01 contracts, Q02/Q05 execution, G01–G12/EG03, scoped release assurance

Accepted original rows: `Q01`, `Q02`, `Q05`, `Q07`, `A03`, `M03`, `M07`, `W03`, `W05`, `W07`, `O03`, `S04`.
Evidence IDs: `E-CI`, `E-GUEST`, `E-MGMT`, `E-O03`, `E-S04`, `E-REC`, `E-RELEASE`.

Fulfilled scope: Q01/Q02/Q05/Q07 DONE; E-CI 91 named PG contracts across the actual selected plain/encrypted cases plus Q05 schedule; E-GUEST native confinement; E-MGMT authority/replay/secret contracts; E-O03 format-3 pre-write refusal; E-S04 ordered atomic refusal; E-REC restored-artifact denial. E-RELEASE source tooling and local unsigned inventories exist.

Unmet input / next evaluation: Installed current/previous release-artifact EG03, scoped independent conformance/review/signature/attestation/SBOM/publication evidence remain unproved here. A workflow attestation request or source/local unsigned SPDX document is not its execution. Slice reviews do not establish certification. Retain security scenarios and claim limits; no report-alone completion.

Claim limits: Source review, workflow attestation requests, unsigned SPDX inventories and current parser contracts are not installed-release/independent certification/signing/older-binary executions.

## Accepted evidence records

Commands below are historical citations, never invoked by this refresh. Counts reused across
categories and subsets/repeats are not new independent executions. Source commits may have
reviewed equivalent integration ports; their existence alone does not establish main ancestry.

### E-CI

Downloaded `/tmp/riauth-wave29-integration-110661000640.log`, SHA-256 `458a30895b5987c0bebe2fb7368a1a32969fa0d41ce90acbfa56f43299ccd186`, run `36950097067`, checkout `2d05c00deed3a6dafcd458a5f1a1a9beb17d0dd8`. Root verified this named job SUCCESS; this audit verified raw hash/commands/results. Three XMLsec filters under `--test identity --locked -- --ignored` each passed 1. Default `scripts/test-postgres.sh` selects `--test postgres -- --ignored --nocapture --test-threads=1`: 2 passed/17.51s. Shared script selects `--features test-support --test contracts -- --ignored postgres --test-threads=2 --nocapture`: 91 passed/90 filtered/209.68s. `RIAUTH_PG_TEST_TARGET=q05_replay_concurrency` selected one ignored PostgreSQL schedule test: 1 passed/1 filtered/13.98s. OpenLDAP, Rust browser, portal_browser, nginx outpost and Traefik tests each passed 1; Playwright setup 9, authenticator step 22 with 2 skipped Firefox/WebKit invitation-passkey cases. Default Cargo edition was Platform. No dedicated node_security_postgres/job/mail/SSF peer target ran, and the 91 is not 91 O03 tests.

Kind: executed local fixture. Verification: existing raw-log hash and command/result inspection.
Runtime run by this refresh: false.
Referenced commits (roles/source-versus-accepted relation are specified above): `2d05c00deed3a6dafcd458a5f1a1a9beb17d0dd8`.

Historical recorded command/expansion where available. Further commands, environment, templates and precise run attribution remain in the pinned references. Empty list does not invent an execution. Nothing invoked by this refresh.

Recorded commands/expansions:

- `RIAUTH_TEST_XMLSEC="$(command -v xmlsec1)" cargo test --test identity --locked saml_independent_xmlsec -- --ignored`
- `RIAUTH_TEST_XMLSEC="$(command -v xmlsec1)" cargo test --test identity --locked saml_source_independent_xmlsec -- --ignored`
- `RIAUTH_TEST_XMLSEC="$(command -v xmlsec1)" cargo test --test identity --locked saml_logout_independent_xmlsec -- --ignored`
- `PG_BIN="$(pg_config --bindir)" scripts/test-postgres.sh`
- `cargo test --locked --test postgres -- --ignored --nocapture --test-threads=1`
- `PG_BIN="$(pg_config --bindir)" bash scripts/test-contracts-postgres.sh`
- `cargo test --locked --features test-support --test contracts -- --ignored postgres --test-threads=2 --nocapture`
- `PG_BIN="$(pg_config --bindir)" RIAUTH_PG_TEST_TARGET=q05_replay_concurrency scripts/test-postgres.sh`
- `scripts/test-ldap.sh`

Existing raw logs rehashed without execution:

- `/tmp/riauth-wave29-integration-110661000640.log`: SHA-256 `458a30895b5987c0bebe2fb7368a1a32969fa0d41ce90acbfa56f43299ccd186` (265770 bytes).

Immutable references, all at `c01c39ab4e092423d5522bedc50fff87656d8c0a`:

| Path | Git blob | SHA-256 |
| --- | --- | --- |
| `docs/roadmap/local-wave29-o03-final-disposition.md` | `98fe4106f02b2c156c4cf9c8a700fde4d0e784ae` | `b1733bab8b2969faee7cc8d5e22d52bcf3fc0bcf11947aed8c84d2807d657fa4` |
| `.github/workflows/ci.yml` | `7c724fd7f4dc210a2268f5a702bdc09b1a10d76b` | `fc5245ef15f0ac8d0a460b71ded533ad96ad3dcbd795b31f38b3e84221940b5a` |
| `scripts/test-postgres.sh` | `c8e3bd778f59322e02114445fdac370d6609bdd0` | `b73f809ca4cec8d5412571c06dc2753178b336ad019b4089567b5557892bf12b` |
| `scripts/test-contracts-postgres.sh` | `ddad88960767ac53915023a6c251eb8b9a2d345b` | `f1741bcc204b0ba610f47275da1ddb510576f96d65fab511a3cf84c1f14f1760` |
| `tests/postgres.rs` | `fa5bca7b35f5b6256b36dcb2c438736e980ee8bf` | `126b0a69e8c756ff060419557a4a1c24fd231c55da68f57e30744ac2f3ef2d02` |
| `tests/contracts/shared.rs` | `6d881fbd547fdf29e60a0c4414853903a9134520` | `e48b42379dbb1092644ae421bf934f557912e4ad775687d70bf6a60db4cc6092` |
| `tests/q05_replay_concurrency.rs` | `d3d685ce1776d07c9004b5638f13676c2fac4d0e` | `d6ab278a4f3ddb3674338e96c3ef833ccb018b86d6e89cead3ac7826bdea0a6d` |

### E-WORK

`local-wave29-w02-closure-root-review.md`, conditional evidence and source executor/model. Accepted conditional code `a03731379e259ed8252d09db5a47d9445776caae`: `cargo test --locked --features test-support,fuzzing --test workflow_configured_conditional_enrollment -- --test-threads=1`, 6/6. Separate corrected existing enrollment source `24a09dff4cbb6adf9a385913b92ac82a7b0cf41e`, accepted `e5f5facddab4f22651ae671a3c6e864ef504c39a`: exact named regression 1/1. Counts are separately attributed historical local redb/software-WebAuthn execution, not repeated here.

Kind: executed local fixture. Verification: accepted page/source/review records inspected; no original runtime log rehash.
Runtime run by this refresh: false.
Referenced commits (roles/source-versus-accepted relation are specified above): `24a09dff4cbb6adf9a385913b92ac82a7b0cf41e`, `a03731379e259ed8252d09db5a47d9445776caae`, `e5f5facddab4f22651ae671a3c6e864ef504c39a`.

Historical recorded command/expansion where available. Further commands, environment, templates and precise run attribution remain in the pinned references. Empty list does not invent an execution. Nothing invoked by this refresh.

Recorded commands/expansions:

- `cargo test --locked --features test-support,fuzzing --test workflow_configured_conditional_enrollment -- --test-threads=1`
- `cargo test --locked --features test-support,fuzzing --test workflow_configured_enrollment configured_passkey_enrollment_binds_fresh_proof_and_commits_once_after_restart -- --exact --test-threads=1`

Immutable references, all at `c01c39ab4e092423d5522bedc50fff87656d8c0a`:

| Path | Git blob | SHA-256 |
| --- | --- | --- |
| `docs/roadmap/local-wave29-w02-closure-root-review.md` | `8a1e10a666efc2ec41e87f986557c26cdfc7ca96` | `84f508418db3e1198154b9d75ce97a915968c5516d431f76d6ec144081126fde` |
| `docs/roadmap/local-wave29-w02-enrollment-fixture-evidence.md` | `86ed195a40f7fc416be3efaa3bb85559fd776e31` | `84be6897de47f06c23a2bce504d8bbfe7795925648a9414f544e50a3bff661fc` |
| `src/workflow/executor.rs` | `711e52d69ade546949664e1661e2e437e3c73f40` | `5c819345b915d760fa9a5c545a12e8c68b0d7b3dac5cd26b8954a4e6cfe99e5b` |
| `tests/workflow_configured_conditional_enrollment.rs` | `54163b8bbf0bf94b1ae45c94843dd815448ea4f7` | `62d9d604dcbd1395e008f7f070ac2a6bd1c283f61d6d004fb5f7f84441a07ad6` |
| `tests/workflow_configured_enrollment.rs` | `95ea1fcfdff4bf14f835bd5240662f1949de1f29` | `b14c16d5a0fe6f57f9595aa061f0b3bdcbd7d43040804213db16a919290d3baa` |

### E-GUEST

`local-wave27-extension-isolation-report.md` and six-task review/root disposition; native source `d772d29fbfc4e681774593ebbe32839df5f5f032`; `cargo test --locked --lib workflow::extension_gate::tests -- --test-threads=1`, 19 passed/95 filtered/33.06s. Accepted source equivalence and log SHA `619315c304faf46a656024e1bda649e48f703d5c758fcae088a2f99a7b45701e` were verified by the independent/root reviews; this audit read those records, not the raw guest log. Native macOS confinement/fuel/deadline/descriptor/proof limits, unsupported-host refusal; no other-OS or released-guest proof.

Kind: executed local fixture. Verification: accepted page/source/review records inspected; no original runtime log rehash.
Runtime run by this refresh: false.
Referenced commits (roles/source-versus-accepted relation are specified above): `d772d29fbfc4e681774593ebbe32839df5f5f032`.

Historical recorded command/expansion where available. Further commands, environment, templates and precise run attribution remain in the pinned references. Empty list does not invent an execution. Nothing invoked by this refresh.

Recorded commands/expansions:

- `cargo test --locked --lib workflow::extension_gate::tests -- --test-threads=1`

Immutable references, all at `c01c39ab4e092423d5522bedc50fff87656d8c0a`:

| Path | Git blob | SHA-256 |
| --- | --- | --- |
| `docs/roadmap/local-wave30-six-task-root-disposition.md` | `fa62ae6056e4c81f412021abef7117f48c18006e` | `1ece5d6f4f6c6d0f264ddfc744e794ccf94724977a69cf65a457e258ca53ba75` |
| `src/workflow/extension_gate.rs` | `a6d9876e5b44b65932a9f779f5e272a2eb6e5fd4` | `db5f3e1360ac5fb87b9c3d6f7f5526ed4b43e9888e78c576194c5ed293ec4495` |
| `src/workflow/extension_gate/isolation.rs` | `6c1b790233b0e05eaa37aeff511bca474f856e5c` | `0865277b37199fc89496e5d90d2d502c1d7dcf705eba6475b63c2cb4403b6d22` |
| `src/workflow/extension_gate/guest-macos.sb` | `b0c1de53e46ebeb05c2c0c0ece86de556157584e` | `2df8d89f35f46185cbc056f20c16f7936fe07016f51f6b74efad2f009fd84204` |
| `src/workflow/extension_gate/native-probe.c` | `84953f942ff4936bf96ab66852589c7927fd93a2` | `c153883db31856bc3b696afa40d400a3e50eeccde981e90b6c139ed29602b385` |
| `docs/roadmap/local-wave27-extension-isolation-report.md` | `887bd872211a76369ef094facfc172bf81d2e030` | `bd178bcda440fcaf75353b912ec040de5b371788b9f93cf9916c49023016e3a0` |

### E-MGMT

`local-wave30-m03-root-disposition.md`, `local-wave29-activation-header-parity-root-review.md`, protected connector port and original management completion matrices. Activation source `1a21871b477d5e58d89cd82130f8e9c5b63a07d6`: historical 38 API functions (including 8 new Core/browser cases), 10 approval functions with 6 PG cases ignored, 2 configured-adapter functions; subsets/repeats are not extra passes. Core Optional and bearer Required share authority-before-receipt, first-only revision checks and stale sealing. M03/M07 DONE, backed by current source and accepted code/evidence, not the old pending disposition. Reviewed client-creation receipt recovery, first-only credential behavior and PAM fallback are deliberate accepted contracts.

Kind: executed local fixture. Verification: accepted page/source/review records inspected; no original runtime log rehash.
Runtime run by this refresh: false.
Referenced commits (roles/source-versus-accepted relation are specified above): `1a21871b477d5e58d89cd82130f8e9c5b63a07d6`.

Historical recorded command/expansion where available. Further commands, environment, templates and precise run attribution remain in the pinned references. Empty list does not invent an execution. Nothing invoked by this refresh.

Immutable references, all at `c01c39ab4e092423d5522bedc50fff87656d8c0a`:

| Path | Git blob | SHA-256 |
| --- | --- | --- |
| `docs/roadmap/local-wave30-m03-root-disposition.md` | `4d62e7110d66893ae40556833c5137c5782db81f` | `37f5168ced288a477721b22d8f7494cdc7ffd04fb4cd0256339ce2d810125db1` |
| `docs/roadmap/local-wave29-activation-header-parity-root-review.md` | `e90cab28b97093eca4976342b84860df13e1bdc5` | `ea8c96a8cc64ecd9a2896792443688851a510f76b11d27b62272df0984d29109` |
| `docs/roadmap/local-wave28-connector-s04-port-report.md` | `32c8a24a42488560755b0a255377bece2a224a40` | `51b8782a584ce2aa9aa706a26d3c4670df81ef266aaa5e201706ce40caee5c1a` |
| `src/workflow/approval.rs` | `a00ba161034453f795be7c740b5c1835d751e4d9` | `31085b8ee934db9503970bcd7f83f07951c10d506be951690b247a8bf9f12397` |
| `src/connector_definitions.rs` | `debf30cb8711c8ab25afa900c87970fd963de04a` | `5fc82b513478553caeb531ba78844562e2a6f76e4d1900921417de8565caaf31` |
| `src/connector_guard.rs` | `5db8330a03f426744b5fbe8ad26ddca470a43bd2` | `53d74f54baed00a3b44d0bd98d6bd09e6b6820dfd3fee60684ed6515f50035d3` |

### E-O03

`local-wave29-o03-final-disposition.md` and root closure; format-3 accepted code/rate report. Historical current-format local `node_security::` checks: 7 after recorded fixture correction; exact all-16 HTTP threshold and explicit maintenance adoption fixtures: 1 each. Dedicated three-process issuer/capability/auth mismatch checks and logout 1/mail 2/SSF 2 worker cases belong to older format-1/2-era PG evidence. Local admission module 2, SCIM freshness/stale-publication/invalidation and recovery-stamp exact filters passed under their recorded wave27 pins. Do not call older process checks current-format-3 peers or claim old binary execution.

Kind: executed local fixture. Verification: accepted page/source/review records inspected; no original runtime log rehash.
Runtime run by this refresh: false.

Historical recorded command/expansion where available. Further commands, environment, templates and precise run attribution remain in the pinned references. Empty list does not invent an execution. Nothing invoked by this refresh.

Recorded commands/expansions:

- `cargo test --features test-support --lib node_security:: -- --test-threads=1`
- `target/wave27/debug/deps/rate_limits-31b1b65edd128268 every_http_category_uses_the_recorded_effective_threshold --exact`
- `target/wave27/debug/deps/maintenance_cli-4bd6cfe2941e92b2 rate_agreement_upgrade_and_missing_adoption_require_explicit_flags --exact`

Immutable references, all at `c01c39ab4e092423d5522bedc50fff87656d8c0a`:

| Path | Git blob | SHA-256 |
| --- | --- | --- |
| `docs/roadmap/local-wave29-o03-final-disposition.md` | `98fe4106f02b2c156c4cf9c8a700fde4d0e784ae` | `b1733bab8b2969faee7cc8d5e22d52bcf3fc0bcf11947aed8c84d2807d657fa4` |
| `docs/roadmap/local-wave29-o03-root-closure.md` | `a25c2ab5bbdd85aac5881803b7cd817aad6bc40d` | `a2e03b49962ef82ca6c86f5edb2fabb38904e100eeb7d282c5353950e0177fdb` |
| `docs/roadmap/local-wave29-o03-rate-agreement-report.md` | `552b502283ae1cf7f4afd531f4e05bb55a01dfab` | `0838012462bd6e54268e7701d17ae8f02c65fd8b10851f461290937d15d90130` |
| `src/node_security.rs` | `da029997ad9c1a805cc3c6a1f6a50954f79a16c3` | `979f1d22a4835bce97866e302dc2a8a21df4f0d383d4bd1e6960b6a5d7a3bcb7` |
| `src/config.rs` | `930e2f6e72f6bb748cf961c02a18faeb88ff9540` | `6bf9a9fc6e4274a87fe175752edee4ac8b8995011af71e7756bf64a5cdf740d5` |
| `src/core.rs` | `f02efcde5e9604b7251a8171e0aa437dbd8d8ae2` | `13da3f54b28a130cd73e5380253248c650f110b4c4bfb563c88382f55cc74304` |
| `src/background/targets.rs` | `d1537a7a3592c8bc4a2c3b5440ce640f287843db` | `41479b1e60ee72fcf6fd684d54f3d8855f8650e91fd8b8acde561e2207c8edcd` |
| `src/provisioning/token_freshness.rs` | `565ca1882932e549fe19008e492d5590bff0fc03` | `155b1a7e755a6e982be97be9fd7cc3b84790a83a0b9f9aa6e46ab8a8f37cc49d` |

### E-S04

Source `5120a0ddb51d015fbf89fbca19da127444e26f8b`, accepted `57ed95fef056afc9402a65c4b576d0461b7fed30`, report `local-wave30-s04-completion-plan.md` and root disposition. Exact `cargo test --locked --features test-support,fuzzing --test state_reconciliation s04_scoped_reuse_equivalent_authority_workload -- --exact --ignored --test-threads=1 --nocapture`: 1 passed/17 filtered/20.47s. Raw log hash `6287eb35f69017af138304ca7ca96624765d7a7b60ba49efab936ae2781e0e6d` reverified without rerunning. Four families/eight writes/three AB-BA-AB pairs: 96 calls per lane; global control 192 writer holds/96 commits/749695 µs occupancy, scoped reuse zero; 495196 µs commit time is included in occupancy. Sixteen apply refusals, 24 ordered two-thread persistence cases and 24 apply/replay/current-permission cases passed. Same-build conservative reuse control follows identical security checks; counts/cost are not general throughput. Root `d23dabbf9c8a9c150987e18f4f416bf93f52562a` removes one needless test borrow, without remeasurement.

Kind: executed local fixture. Verification: existing raw-log hash and command/result inspection.
Runtime run by this refresh: false.
Referenced commits (roles/source-versus-accepted relation are specified above): `5120a0ddb51d015fbf89fbca19da127444e26f8b`, `57ed95fef056afc9402a65c4b576d0461b7fed30`, `d23dabbf9c8a9c150987e18f4f416bf93f52562a`.

Historical recorded command/expansion where available. Further commands, environment, templates and precise run attribution remain in the pinned references. Empty list does not invent an execution. Nothing invoked by this refresh.

Recorded commands/expansions:

- `cargo test --locked --features test-support,fuzzing --test state_reconciliation s04_scoped_reuse_equivalent_authority_workload -- --exact --ignored --test-threads=1 --nocapture`

Existing raw logs rehashed without execution:

- `/tmp/riauth-wave30-s04-equivalent-authority.log`: SHA-256 `6287eb35f69017af138304ca7ca96624765d7a7b60ba49efab936ae2781e0e6d` (18610 bytes).

Immutable references, all at `c01c39ab4e092423d5522bedc50fff87656d8c0a`:

| Path | Git blob | SHA-256 |
| --- | --- | --- |
| `docs/roadmap/local-wave30-s04-root-disposition.md` | `1b7ec026a920f4daf0be0b4276cc2e7d9afafc58` | `61ca9fa505a424dddc2b36126458ec765dbf3c512cc6f09ba5deb5921f1e4469` |
| `docs/roadmap/local-wave30-s04-lint-ci-report.md` | `977b2c37083aa4d0d6f08586615dab80e3be93e2` | `a43669c3c8367ba31d2e6d8fc6067c805de47facdd4910aa080a7f0ef19f2d1f` |
| `src/state.rs` | `97c57ac0b3040961139258651d0ae39b6c15c825` | `ac7a222e1e70e2831f879b87a6f745c349ded13ddac7af88ae8f1d6f01a3d2b1` |
| `tests/state_reconciliation.rs` | `bd010df7904baa0c36daec2c34e964978f36dbd4` | `895ac818b49160fdfcb42c8e2ba601adf21d225893276ac975aec6f99b129c81` |
| `docs/roadmap/local-wave30-s04-completion-plan.md` | `ad65c0adf95e32b339cd2bef69eb6f86f168421a` | `1ae8e7c28253483addc341e72cb7e2f641b0d848769557e760cb740a21b2e526` |

### E-PAGE

Pagination source `9810c33edba8027433c1a4e24442d878710b9a8a`, accepted `1955cf28d52861cacf33fe8626342a2ab8954492`; unchanged `tests/reconciliation_jobs.rs` blob `aa1c3bdc70b232796dfb0c150179b8165384b85a`. Exact locked test-support,fuzzing named filter/single-thread: fixed-da5 baseline exit101, 0pass/1fail/1filtered/1.44s at line229; corrected exit0, 1pass/1filtered/1.71s. Raw log SHAs `5e8be0ae942ccbed9118b9b76a03bf6dc1383d762e13e938ef0a3eff3ae0c7fb` and `2cc8e8b515edf992ed22d42ea4e596ae8ae45df75271ddc6954537754bcd17b5` reverified. Current finish/owner-generation release body preserved; current whole file also includes the separately accepted O06 diagnostic reader. No new Linux or release-failure fault injection is inferred.

Kind: executed local fixture. Verification: existing raw-log hash and command/result inspection.
Runtime run by this refresh: false.
Referenced commits (roles/source-versus-accepted relation are specified above): `1955cf28d52861cacf33fe8626342a2ab8954492`, `9810c33edba8027433c1a4e24442d878710b9a8a`.

Historical recorded command/expansion where available. Further commands, environment, templates and precise run attribution remain in the pinned references. Empty list does not invent an execution. Nothing invoked by this refresh.

Recorded commands/expansions:

- `cargo test --locked --features test-support,fuzzing --test reconciliation_jobs controller_resumes_more_than_four_scim_snapshot_pages_without_spending_retries -- --exact --test-threads=1`

Existing raw logs rehashed without execution:

- `/tmp/riauth-wave30-reconciliation-pagination-baseline.log`: SHA-256 `5e8be0ae942ccbed9118b9b76a03bf6dc1383d762e13e938ef0a3eff3ae0c7fb` (1338 bytes).
- `/tmp/riauth-wave30-reconciliation-pagination-corrected.log`: SHA-256 `2cc8e8b515edf992ed22d42ea4e596ae8ae45df75271ddc6954537754bcd17b5` (772 bytes).

Immutable references, all at `c01c39ab4e092423d5522bedc50fff87656d8c0a`:

| Path | Git blob | SHA-256 |
| --- | --- | --- |
| `docs/roadmap/local-wave30-reconciliation-pagination-ci-report.md` | `57298fb21f9c38a87b90901e478895883f35e02f` | `f9e6f7e2291dc8c1ce0fc69da46d83c16997aa94f8a627d1eac5ac1958ba7008` |
| `docs/roadmap/local-wave30-reconciliation-settlement-root-review.md` | `5e6ce08c4e67ef3ce7bd3e9481288374e25a8c94` | `10d8223bf8f1db14763a6525d3fab4d4f306d8c286d70dbc07d17009b41f2dcc` |
| `src/reconciliation.rs` | `f15ef16a97091817418ce95b14f3d59583caadc8` | `a5b61194c36b432ece01910b4f49a720c918d69045464cc259876d85e8acb589` |
| `tests/reconciliation_jobs.rs` | `aa1c3bdc70b232796dfb0c150179b8165384b85a` | `ffd3c16817b337b195f0e09de4a7ad441dd7a097c33007c770afd6bc35b48076` |

### E-DIAG

Current O06 storage/key/controller/missing-evidence/gauge reports and root reviews. Storage gives explicit capacity-unmeasured remedies; key output refuses to invent custody/health; configured-unscheduled controllers are visible without writes. Configured-controller exact fixture 1 pass/1.06s; gauge exact fixture initially failed stale 422/400 expectation, then 1 pass/2.14s after the recorded fixture-only correction. Gauge revision8→9 repair preserves unresolved uncertainty; resolution/waiver is not delivery. This is accepted bounded diagnostic evidence, not whole O06 or physical-pressure completion.

Kind: executed local fixture. Verification: accepted page/source/review records inspected; no original runtime log rehash.
Runtime run by this refresh: false.

Historical recorded command/expansion where available. Further commands, environment, templates and precise run attribution remain in the pinned references. Empty list does not invent an execution. Nothing invoked by this refresh.

Recorded commands/expansions:

- `cargo test --locked --features test-support --test o06_unscheduled_controller_diagnostics configured_controllers_without_stored_schedules_need_operator_attention -- --exact --test-threads=1`
- `cargo test --locked --features test-support,fuzzing --test o06_resolved_deactivation_gauge operator_resolution_updates_failed_gauge_and_rebuilds_legacy_indexes -- --exact --test-threads=1`

Immutable references, all at `c01c39ab4e092423d5522bedc50fff87656d8c0a`:

| Path | Git blob | SHA-256 |
| --- | --- | --- |
| `docs/roadmap/local-wave30-o06-controller-diagnostics.md` | `01193a08c1059770aecc3adc618f860d63ef02e7` | `c9a0f5fb41e8d329af23ee7a39706abc3864fd7361fc8e84829466361b449e8d` |
| `docs/roadmap/local-wave30-o06-controller-root-review.md` | `bbaebade58119d3fa0b521c1dfde1245bd290915` | `b4ab7128da0239879de67d0a0d9056f72195ed1a40ada7f4f3e1816753b093ea` |
| `docs/roadmap/local-wave30-o06-gauge-root-review.md` | `b4442714913840c4d0407d914325944e1fffe897` | `76216266a9ed0e15a59247be600a37ff4b3b5029d96b3e4480b60dcdeed47e47` |
| `docs/roadmap/local-wave30-o06-resolved-deactivation-gauge.md` | `b792a3d598e7b7676c8d97cde72b15603cff91fe` | `b3c34144d5a7eeea3bafa15863d135d1f941e88cf25646096d6e06a0df66f8e2` |
| `docs/roadmap/local-wave30-o06-storage-root-review.md` | `257932cd6e7a8b7cc2ef8d64e56f561bec0a68b3` | `458fb262b4233bfe26323e3a5e864cb1cae1f40d6dc9ff2d68775f446b655147` |
| `docs/roadmap/local-wave30-o06-key-root-review.md` | `98e6aa86b9343a561cd877d470f064b4ed517e41` | `1fd8e67d9d1c3358863a1b5b8bd8f78ef1a53ca8d4a76b99757037db9acd8be9` |
| `docs/roadmap/local-wave30-o06-missing-evidence-root-review.md` | `1bccc026fecb33aba2f21948aeb4175e88347469` | `8875e232cc6f650890d7f5ca0f2a30fadfd3c14f703f2085875945c95e616e41` |

### E-ARM

Tracked A08 `RUN.md`, `native-arm64-evidence-v2.json`, encrypted supplement and `integration-review.json`. Product binaries `f3aba63ac3b824843a40b99623f1619ef8edc19f`; plain validator `730d373078c4b128373d7afa5f3e278f63b065fd`, encrypted validator `9f8efaf0372e02b0bc0042251d591f1facc0f783`, recovery supplement `082563b362acb218f276e4dbcaf476e303b561b7`. Native Linux AArch64 plain/encrypted redb/PG edition handoffs, identity/auth/refusal checks and local archives/images. Encrypted logical rows retained 30/33 redb and 36/39 PG with raw ciphertext equality. Historical format2, not a current format3 execution. Local binary sizes: server Essentials 36390184 B, Platform 49302784 B; maintenance 13310224/16783720 B; client 8101312 B. Earlier `6ca4779` late smoke/installed-transition failures remain historical and are not erased by later local transition successes. `release_gate_result: false`; no official release identity.

Kind: native local artifact. Verification: tracked result JSON and accepted review inspection; no original binary/log rehash.
Runtime run by this refresh: false.
Referenced commits (roles/source-versus-accepted relation are specified above): `082563b362acb218f276e4dbcaf476e303b561b7`, `730d373078c4b128373d7afa5f3e278f63b065fd`, `9f8efaf0372e02b0bc0042251d591f1facc0f783`, `f3aba63ac3b824843a40b99623f1619ef8edc19f`.

Historical recorded command/expansion where available. Further commands, environment, templates and precise run attribution remain in the pinned references. Empty list does not invent an execution. Nothing invoked by this refresh.

Immutable references, all at `c01c39ab4e092423d5522bedc50fff87656d8c0a`:

| Path | Git blob | SHA-256 |
| --- | --- | --- |
| `docs/roadmap/evidence/a08-native-linux-arm64-2026-09-29/RUN.md` | `3b771bf995513ede2f7047d592116c6ec1f36a29` | `dd6a3be1226b7c0d464b6639f40baa2150797e98ada4b9929a199705c105aa28` |
| `docs/roadmap/evidence/a08-native-linux-arm64-2026-09-29/native-arm64-evidence-v2.json` | `a888a1f8c617aaa9b3bdec9b09eaa306063eefee` | `638822d3c437d06678d27d4fb68987ebd2f6a884dd36c5843e9addf7727b5c7e` |
| `docs/roadmap/evidence/a08-native-linux-arm64-2026-09-29/encrypted-storage/ENCRYPTED-RUN.md` | `390f5c2d9275243941565b6ad1982ba7d8144919` | `ecb58a107f46ab1a0970bf0e1a90c3d18a2c9b8ada155391fcdae6a7f6c56c36` |
| `docs/roadmap/evidence/a08-native-linux-arm64-2026-09-29/encrypted-storage/native-arm64-encrypted-evidence.json` | `359c0e17e3e8e6ced06e9235106ddfd29878baba` | `c7d168d65d5b5296f61d3ec7b06688ec7839080be9ef98a573f735f44d1ebfd6` |
| `docs/roadmap/evidence/a08-native-linux-arm64-2026-09-29/integration-review.json` | `74e69594ec5086c4a40a03d53fc18737cb74c670` | `514c858f806fcef543dc9aa86806698fde70f08d50830052c642e8491ee63c0e` |
| `docs/roadmap/q08-native-arm64-local-6ca4779.md` | `6d26f9c39ac1d75dcc4d0e04fabcf740c96214be` | `781ef31c36293a7e7f93d43b412bdf83fd2d2e47a792c489d7cc42438cbc0b26` |

### E-Q09

Current `q09-benchmark-slice.md`, including later paced macOS, plaintext ARM64 and secure ARM64 sections, rather than only its old self-check. Secure harness `4ab934a44a80062dda0dba5d8640d396631a3ef0` and release-profile product copies built from `6ca4779b68cf41e29af490d2aca6ea3d59e1f2bd` ran four edition/backend combinations with private-CA HTTPS, aes256gcm-v1 and PG SCRAM/hostssl; product file sslmode=require, libpq verification uses verify-full. Each quiet/interference pass: 12 reads/12 successes/no read errors, maintenance delta1, 71.57–71.66s; writer 2success/2expected409. Twelve samples make p95/p99 the maximum; 6500ms pacing is inside reported success/s, outside latency samples. Historical recorded RSS maxima 50408–55992 KiB across these passes; no external signer/published artifact/current-main or maximum capacity claim. Accepted page/hash references are read here; external report files are not rehashed in this audit.

Kind: native local artifact. Verification: accepted page/source/review records inspected; no original runtime log rehash.
Runtime run by this refresh: false.
Referenced commits (roles/source-versus-accepted relation are specified above): `4ab934a44a80062dda0dba5d8640d396631a3ef0`, `6ca4779b68cf41e29af490d2aca6ea3d59e1f2bd`.

Historical recorded command/expansion where available. Further commands, environment, templates and precise run attribution remain in the pinned references. Empty list does not invent an execution. Nothing invoked by this refresh.

Immutable references, all at `c01c39ab4e092423d5522bedc50fff87656d8c0a`:

| Path | Git blob | SHA-256 |
| --- | --- | --- |
| `docs/roadmap/q09-benchmark-slice.md` | `c08d2468411430146d93260d135f57977cecf2f5` | `e01d264f5eb41cef2b8dee189a1dc33b1925135636637b67c34af4ee4e53d9a7` |
| `scripts/q09_benchmark_slice.py` | `558cc1a22a69007ab70a6c481c9bf90cfbf58cd0` | `399bd242519bdae15483f2d450ef9c1bf7b8b397a25bfff1179abefcaf9c7e49` |

### E-REC

Tracked encrypted A08 PG restore JSONs: each edition 16 passed checks, binary SHAs `86a799d3060e5e602c7e458a9c409ffc8c06f881e2d426a1e6d9be226337445d` / `78209c1766eb53be741de0946a0e4ce4bc3f2217deda84a6e4f4bda85fc42b62`; outage, wrong key/occupied target, gated serve, explicit fixture reconciliation, old-session refusal and fresh service user/admin login. Separately tracked R05 native PG16.14 physical-base-backup report has 16 checks and binary SHA `a5c01650917d16c9384b1d87fba373d02e97d5e85eb8a2e625d7c9a8cbcd68ea`, without a source-commit field: same-lineage blind spot, offline invalidation and artifact/serve refusal, gate left pending. These are distinct drills, not 32 full deployed recovery passes. D04 manual physical/PITR observations and logical completion limits stay explicit.

Kind: executed local fixture. Verification: tracked result JSON and accepted review inspection; no original binary/log rehash.
Runtime run by this refresh: false.

Historical recorded command/expansion where available. Further commands, environment, templates and precise run attribution remain in the pinned references. Empty list does not invent an execution. Nothing invoked by this refresh.

Immutable references, all at `c01c39ab4e092423d5522bedc50fff87656d8c0a`:

| Path | Git blob | SHA-256 |
| --- | --- | --- |
| `docs/roadmap/recovery-drill-r05.md` | `1dfe95476f7b38d7bb2db916ac645fd9f0d199f1` | `ef531c22177e20ccbb16066107caddc8b578ac46829c218924d8a9ec5c0429ab` |
| `docs/roadmap/evidence/a08-native-linux-arm64-2026-09-29/encrypted-storage/local-pg-restore-essentials.json` | `6d4755faaaa4ca1f5eb7501e50122be38fa90d91` | `eb1f30d115838098dc08d6731e8bcd93c34bf3b4121aa2a587a935babf14dbea` |
| `docs/roadmap/evidence/a08-native-linux-arm64-2026-09-29/encrypted-storage/local-pg-restore-platform.json` | `e46218d8cbd2f3f5164a1e2a8f1a7374beb74b3e` | `9a6c81b113ab0cd8943b2d5c4c11a7dcca810044cc277562d33ebae97fc25a50` |
| `docs/roadmap/evidence/r05-native-postgres-2026-09-29/report.json` | `ac811bf862fa61edd977919a8f94b05b2ebf1176` | `0e524b94eb6b913a9d6dca5e0da3dd8dc0b2d8d783555ed2a02d896bf18a01dc` |
| `docs/roadmap/evidence/d04-postgres-base-backup-2026-09-29.json` | `43c737a1e7fb970104ea3241dbbe4ee28e7691e4` | `86d1f5b84cef7b69511e2960352a7247eb46bc1c02a898db9a44d62e6cda9e80` |
| `docs/roadmap/evidence/d04-postgres-pitr-2026-09-29.json` | `215bcc9fe6625a052ee8b4048dd4f3a753e0128a` | `accb6f10e6c7e2c1a6fa91ee08ddef1d82be116d0dc42c177e20b66718100b90` |

### E-MIG

Accepted six-task review/root disposition reads conversion/reimport source and assertions; `convert`/`classify_issuer` are equivalent to executed `3ddcab4d43cfa1ce41222b6253dedbed573c0c64` at its reviewed pin. G01–G04 original rows DONE; G05 `cf827a9` synthetic reference RP rehearsal 1/1 with eight case IDs remains historical, not customer migration. No real-export or rollback claim.

Kind: executed local fixture. Verification: accepted page/source/review records inspected; no original runtime log rehash.
Runtime run by this refresh: false.
Referenced commits (roles/source-versus-accepted relation are specified above): `3ddcab4d43cfa1ce41222b6253dedbed573c0c64`.

Historical recorded command/expansion where available. Further commands, environment, templates and precise run attribution remain in the pinned references. Empty list does not invent an execution. Nothing invoked by this refresh.

Immutable references, all at `c01c39ab4e092423d5522bedc50fff87656d8c0a`:

| Path | Git blob | SHA-256 |
| --- | --- | --- |
| `docs/roadmap/local-wave30-six-task-completion-review.md` | `d5c7d440116fbb762eb6858310a749a21a90b0fa` | `9f871d21f7d00b0e8a2b582f3765ada117a8e983f8d436d64631b26457065aef` |
| `docs/roadmap/local-wave30-six-task-root-disposition.md` | `fa62ae6056e4c81f412021abef7117f48c18006e` | `1ece5d6f4f6c6d0f264ddfc744e794ccf94724977a69cf65a457e258ca53ba75` |
| `src/migration.rs` | `be21fbdfa8cac065ba0a796f41bb6214c9396e9c` | `5f923acc9befca63aebb97d04bc29462d36a3e8e573a79513f847dd1fbead152` |

### E-SCIM

Six-task independent/root review and source/assertions: P06 supported core schemas, conditional tombstone disable/revocation/downstream intent and resource-version controls; redb 130-user Group replace/replay exact fixture 1/1 and plain/encrypted PG cases 2/2 at accepted `7b9253117e51a0676988de5ea26fb43dc2c627e5`. I05 direct broker-optional signed service-account grant and endpoint/key/expiry/failure checks; `4df94b61edf9161cac5217b00691e55fdb6ee6ed` recorded cloud_directory test-support 36/36. Local fake peers, no cloud mutation or real tenant.

Kind: executed local fixture. Verification: accepted page/source/review records inspected; no original runtime log rehash.
Runtime run by this refresh: false.
Referenced commits (roles/source-versus-accepted relation are specified above): `4df94b61edf9161cac5217b00691e55fdb6ee6ed`, `7b9253117e51a0676988de5ea26fb43dc2c627e5`.

Historical recorded command/expansion where available. Further commands, environment, templates and precise run attribution remain in the pinned references. Empty list does not invent an execution. Nothing invoked by this refresh.

Recorded commands/expansions:

- `cargo test --locked --features test-support --test scim_pagination redb_scim_group_member_replace_pages_owned_users -- --exact`

Immutable references, all at `c01c39ab4e092423d5522bedc50fff87656d8c0a`:

| Path | Git blob | SHA-256 |
| --- | --- | --- |
| `docs/roadmap/local-wave30-six-task-completion-review.md` | `d5c7d440116fbb762eb6858310a749a21a90b0fa` | `9f871d21f7d00b0e8a2b582f3765ada117a8e983f8d436d64631b26457065aef` |
| `docs/roadmap/local-wave30-six-task-root-disposition.md` | `fa62ae6056e4c81f412021abef7117f48c18006e` | `1ece5d6f4f6c6d0f264ddfc744e794ccf94724977a69cf65a457e258ca53ba75` |
| `src/assembly/scim_runtime.rs` | `cd5cf20402e04719d3e6dd6e0d61a87b5f990aee` | `9cd66862c726c11db78a44a714a1b8ae0c9a1ef2012e82f1a802718170a43c55` |
| `tests/scim_pagination.rs` | `8bc319b4771f63b92b028518863998a47c81ac43` | `abfdfe15374d3a9f23a79501793db79dd6b469210a9591a664f3ec207ddda697` |

### E-WSPACE

Six-task independent/root review and source/assertions: P06 supported core schemas, conditional tombstone disable/revocation/downstream intent and resource-version controls; redb 130-user Group replace/replay exact fixture 1/1 and plain/encrypted PG cases 2/2 at accepted `7b9253117e51a0676988de5ea26fb43dc2c627e5`. I05 direct broker-optional signed service-account grant and endpoint/key/expiry/failure checks; `4df94b61edf9161cac5217b00691e55fdb6ee6ed` recorded cloud_directory test-support 36/36. Local fake peers, no cloud mutation or real tenant.

Kind: executed local fixture. Verification: accepted page/source/review records inspected; no original runtime log rehash.
Runtime run by this refresh: false.
Referenced commits (roles/source-versus-accepted relation are specified above): `4df94b61edf9161cac5217b00691e55fdb6ee6ed`, `7b9253117e51a0676988de5ea26fb43dc2c627e5`.

Historical recorded command/expansion where available. Further commands, environment, templates and precise run attribution remain in the pinned references. Empty list does not invent an execution. Nothing invoked by this refresh.

Recorded commands/expansions:

- `cargo test --locked --features test-support --test cloud_directory -- --quiet`

Immutable references, all at `c01c39ab4e092423d5522bedc50fff87656d8c0a`:

| Path | Git blob | SHA-256 |
| --- | --- | --- |
| `docs/roadmap/local-wave30-six-task-completion-review.md` | `d5c7d440116fbb762eb6858310a749a21a90b0fa` | `9f871d21f7d00b0e8a2b582f3765ada117a8e983f8d436d64631b26457065aef` |
| `docs/roadmap/local-wave30-six-task-root-disposition.md` | `fa62ae6056e4c81f412021abef7117f48c18006e` | `1ece5d6f4f6c6d0f264ddfc744e794ccf94724977a69cf65a457e258ca53ba75` |
| `src/cloud_directory.rs` | `f912db10b2a9ca52f4bdfef6ae2ab5042545fd16` | `398de960390c70e0c427aa2fee0c140a2fb8edbdbaee817399ee441989de15bf` |
| `docs/enterprise/ENT-03.md` | `8d2a0af34d794f01a3aaf87c0f687dbc04ee469e` | `36e70cefa0ad03cffe6f1d2df71f9227109354324d4045e720b43ed61168692e` |

### E-PEER

Older actual OpenLDAP provider 2.7.1 LDAPS/STARTTLS and FreeRADIUS radclient 3.2.10 PAP accept/reject/replay remain named local peers. `d01-platform-cli-walkthrough.md` records eleven partial disposable guide runs, including separate remote client/maintenance, OpenLDAP, loopback SCIM, XMLsec and GNU Lasso. Binary snapshots/partial browser paths and refusals are retained; they are not eleven complete new-user/operator journeys. Essentials guide explicitly withholds such completion.

Kind: named local peer. Verification: accepted page/source/review records inspected; no original runtime log rehash.
Runtime run by this refresh: false.

Historical recorded command/expansion where available. Further commands, environment, templates and precise run attribution remain in the pinned references. Empty list does not invent an execution. Nothing invoked by this refresh.

Immutable references, all at `c01c39ab4e092423d5522bedc50fff87656d8c0a`:

| Path | Git blob | SHA-256 |
| --- | --- | --- |
| `docs/recipes/platform-ldap-provider.md` | `156d298511009bf88f47fda44f92c04c4aa3765f` | `82f5ab4ac875f6e8e58631e1e77f81ae16c9d16e4ea4428ddac394e9e7258f9d` |
| `docs/roadmap/d01-platform-cli-walkthrough.md` | `ef367ba40fdec9b26e6bef462d9a07de830e0d3b` | `1ba4800909b714cca3061cb4db9123d593944b4a23a3d3c5648cd50ef1d82eb8` |

### E-GUIDE

Older actual OpenLDAP provider 2.7.1 LDAPS/STARTTLS and FreeRADIUS radclient 3.2.10 PAP accept/reject/replay remain named local peers. `d01-platform-cli-walkthrough.md` records eleven partial disposable guide runs, including separate remote client/maintenance, OpenLDAP, loopback SCIM, XMLsec and GNU Lasso. Binary snapshots/partial browser paths and refusals are retained; they are not eleven complete new-user/operator journeys. Essentials guide explicitly withholds such completion.

Kind: executed local fixture. Verification: accepted page/source/review records inspected; no original runtime log rehash.
Runtime run by this refresh: false.

Historical recorded command/expansion where available. Further commands, environment, templates and precise run attribution remain in the pinned references. Empty list does not invent an execution. Nothing invoked by this refresh.

Immutable references, all at `c01c39ab4e092423d5522bedc50fff87656d8c0a`:

| Path | Git blob | SHA-256 |
| --- | --- | --- |
| `docs/roadmap/d01-platform-cli-walkthrough.md` | `ef367ba40fdec9b26e6bef462d9a07de830e0d3b` | `1ba4800909b714cca3061cb4db9123d593944b4a23a3d3c5648cd50ef1d82eb8` |
| `docs/essentials-guide.md` | `2b8408f90d82bb35d49fc1ebd3512d9a1a853962` | `80f54441cd5cca1cd3c8521ba91b125db0ac8a86c8ee4055986b8f69402c67ee` |
| `docs/platform-guide.md` | `46f41f11fe211343be9059966253118401ffb4a0` | `5d9e10463138e1582e735c01c2713bf416012cb3a17b5f0c3e1d3e6fbadc0af3` |

### E-RELEASE

Current `q11-release-evidence.md`, packager/checker/attestation source and SECURITY.md. Local unsigned SPDX inventories tied to preserved A08 assets list 301/353/121 packages; these differ deliberately from older normal closure counts 280/329/106. Current workflow requests pinned attestations; this audit found no executed release bundle/signature/attestation/publication record at the pin. Do not say tooling or local inventories are absent, or extrapolate absence beyond the inspected evidence.

Kind: inspected source/criteria. Verification: accepted page/source/review records inspected; no original runtime log rehash.
Runtime run by this refresh: false.

Historical recorded command/expansion where available. Further commands, environment, templates and precise run attribution remain in the pinned references. Empty list does not invent an execution. Nothing invoked by this refresh.

Immutable references, all at `c01c39ab4e092423d5522bedc50fff87656d8c0a`:

| Path | Git blob | SHA-256 |
| --- | --- | --- |
| `docs/roadmap/q11-release-evidence.md` | `a3ed1a0381f4507e678ddc8a54ab4c7bfbd5519b` | `5a3a0981ebdf7215c497fd748f32934e1a2cc7ba3cef4c98982c2c70f0fe1f39` |
| `scripts/package-release.sh` | `2663c67109f017170412d66eddaf3126f90c089f` | `3060b16559b004e6e213d7ed484f575a7375df3d92d29b39fcf0726f936a2f0a` |
| `.github/workflows/release.yml` | `897df176d6dd773f9e86f8dfdb3655003c5c32ca` | `da8607c5ea33a906185141e0caebec18c2282c22676981e0c659fd65eef4369f` |
| `SECURITY.md` | `047208fb72e97fc942fe5d4d988b162c28f80d5c` | `2556771d58d09a2a4754e7484645b7e948b84286ef0d21cc66169b920a31c4d8` |

## CI and transfer limits

Root verified predecessor run `36950097067`, integration `110661000640`, SUCCESS at
`2d05c00deed3a6dafcd458a5f1a1a9beb17d0dd8`. E-CI rechecks its actual raw hash/results.
The 91 selected PG contracts are not 91 O03-specific tests; dedicated current-format
node_security/job/mail/SSF peer targets did not run. A successful named job is not a
fresh c01c39a execution or an overall-green run.

The approved proposal compared predecessor node_security/config/target-admission/token-
freshness, PG/Q05/shared-contract test, CI and both PG script blobs unchanged at c01c39a.
State has separate E-S04 evidence; changed adapters get no new execution credit from old CI.
The publication-time note cited by that proposal recorded run `36998781947` pending;
separate failed checks coexist with integration/audit success. No live GitHub query is made.
Root d23dabb test-only lint correction and current pagination/O06 composition remain intact
without remeasurement, new Cargo or an invented corrected Linux outcome.

## Superseded historical holds

Credit accepted W02/W05/W07 executor/versioning/confinement; A03 responsibility separation;
M03/M07 shared management and protected connector desired state; O03 rate/job/cache coordination;
and S04 families/reuse/persistence/issuer plus measured security/concurrency gate. Native ARM64
artifacts, later paced secure Q09 observations, encrypted logical PG restore and physical
base-backup invalidation replace blanket absence statements. O06 accepted diagnostic slices
remain bounded evidence for an active original row.

Old protocol-file/compilation/fmt/3-of-3-PG wording and former implementation holds remain
historical below; their presence is not a present blocker. Definitions and slice reviews are
also not executed release, deployed HA or independent conformance results.

## Remaining gate and preserved boundaries

One independently recorded documented journey for a new user and a new operator using matching artifacts/configuration and supported ordinary Essentials invitation/factor/app/session/recovery and operator setup/management/safe recovery procedures. D01 owns this walkthrough; consume reviewed actual outcomes instead of launching a duplicate campaign here.

Claim-specific hardware/tenant/release/escrow/measurement inputs in categories remain explicit; no universal fresh-run or HA condition.

Held canonical Group and canceled accessibility remain explicit. Preserve physical hardware,
tenant/release/escrow/PG-topology/HA/older-binary limitations, 60-second nonrenewed admission
and paused-before-IO distinction, receipt-secret, route headers, PAM fallback and all live
permission/review/receipt/removal/audit/credential contracts. Root alone integrates, pushes
and changes statuses; D05 remains in_progress until its independent gate is evidenced.

## Checks actually performed

Only live original-row reads, immutable object/path inspection, existing log rehash/result
filtering, input JSON parsing, JSON/Markdown correspondence, exact reference/link checks and
Git whitespace/two-file scope checks are used. No Cargo/runtime/test/benchmark/service/desktop,
PG/cloud mutation, new task/worktree/worker, board/main/accepted change or push occurred.
Readiness keeps the Cargo slot.

The initial immutable reference guard stopped before writes at guessed src/approval.rs; pinned tree inspection corrected it to src/workflow/approval.rs. No artifact/source/runtime was changed by that failed guard. A second guard treated an explicitly cited 40-character fixture blob as a commit; classifying referenced Git objects by their actual type corrected it before any writes. Approved proposal separately records two reference-check guards and a draft delimiter correction.

Results: all document checks passed. Complete old JSON data, the ten historical
newer-slice prefix entries and verbatim old Markdown are preserved; ten categories
and seventeen evidence IDs agree. Verified 77 unique fixed-pin path/blob/SHA-256
references, 21 correctly typed Git objects and four retained raw-log hashes.
`python3 scripts/check-docs.py` passed Markdown links/build-directory layout;
`git diff --check` and two-file scope passed. The approved proposal and all
source/tests are unchanged. No runtime was performed.

## Preserved historical snapshot

The following 2026-09-29 text is retained **verbatim**. Its statuses, source/integration pins,
commands, counts, failures, old remaining text and slice-only freeze belong to that historical
snapshot. They are not current verdicts. The complete original JSON data is embedded unchanged
under `prior_snapshots[0].snapshot` in the twin.
Original JSON blob `501e4b4af8ce1529c52bc3edececbd2f30949a5c`, SHA-256
`679bd9a31f6d6c58c7fc2057d256e6c717a6b642b6c4b75abd613ff669f5b3a8`;
original Markdown blob `9e3116c1cc50a984ed3ba3e1613efff560fa271c`, SHA-256
`bc462d87981630c53fec7d0ffe4f2a8602d6877ec451f4a0c33f7b6dee8a95cb`, preserved at
`c01c39ab4e092423d5522bedc50fff87656d8c0a`.

<!-- D05 HISTORICAL SNAPSHOT BEGIN -->

# D05 acceptance evidence matrix

Status: **not passed.** Snapshot date 2026-09-29. This D05 branch was not
reset onto accepted integration. The parent snapshot is `328a3ac`, whose
base is `cf827a9ee103672b329907a433fb96ac868fe178`. Accepted files were
read from ledger head `03124272a6cfe14ac48227c504cc896c2f11d9aa`. Every
category stays **not passed**. D05 stays **not passed**. The workstream
gate — a new user and a new operator can independently complete the
documented workflows — has no recorded completion.

The previous snapshot is `09e68460d7836be8227568fbc84d8516d2d23e84`, the
accepted form of `054ab05`. It described head `52df9a3`. This page keeps those
records and adds the slices accepted after them. A status other than
`not_passed` is outside this slice. The machine-readable twin is
[d05-acceptance-evidence.json](d05-acceptance-evidence.json).

## What was read

| Source | Identity |
| --- | --- |
| This worktree | `/Users/dominik/orca/projects/riAuth-public-preview-roadmap-d05-acceptance-evidence-wave24`, branch `roadmap/d05-acceptance-evidence-wave24`, parent snapshot `328a3ac` on base `cf827a9`. This refresh writes only the two D05 evidence files |
| Ledger head | `/Users/dominik/.local/share/riwork/orchestrators/projects/891e7443-8dac-4c1b-897f-9e53cb59c7ee/planning/accepted-commits.json`. The stored `integration_head` string is `0312427`. That prefix is commit `03124272a6cfe14ac48227c504cc896c2f11d9aa` |
| Accepted checkout | `/Users/dominik/orca/projects/riAuth-public-preview-roadmap-integration-accepted` was read at that same commit and was not edited, merged, or pushed |

The ledger `integration_status` is `local_commits_only_no_merge_to_main`.
It has 456 `reviewed_slices`, 436 `integration_validation.checks`, and 17
`open_checks`. D05 is one reviewed slice, integration commit `09e6846`,
status `task_in_progress`. The task that started this refresh named
`ee88af0d02d4a8c1d2f7a9a903f00890aeaef7b1` and 454 slices. That commit is
an ancestor. The ledger then included W07 at `3cfe28b` and Q06 at
`0312427`. The validation `head` and the Q06 `run_head` are stored as
the seven-character prefix `0312427`.

The previous snapshot's statement that `e926d75` had no ledger check is
stale. Ledger `run_head` `e926d75` records
`CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 cargo test --locked --test pam_management -- --quiet; python3 scripts/check-module-boundaries.py; git diff --check HEAD~1 HEAD`
with result text pass; 6 PAM management tests. The note says broader M03
management parity work remains. That run is source-only and does not pass
administration.

Sentences about commits after `cf827a9` were read from the accepted
objects. Paths that this worktree does not contain are named in backticks
and are not markdown links.

This slice did not run Cargo, a browser, a database, or a peer. Commands
below are citations of docs and of the ledger. They were not re-executed
here.

The [A01 coverage inventory](coverage-inventory.md) at base `96e23e2` is
historical planning evidence and is left unchanged. Category targets are
the frozen checks in [product contracts](product-contracts.md), including
G01–G12 and EG01–EG04. Criteria in that contract are not observed results.

## Evidence kinds

This snapshot uses three kinds. None of them passes a category by itself.

| Kind | Meaning |
| --- | --- |
| `source-only` | A file in the tree, or a recorded run of riAuth against itself or a disposable redb/PostgreSQL cluster the test created. The command may have passed. `run_head` is kept so an older pass is not described as a new one. |
| `local peer` | A third-party program on this host, loopback only. OpenLDAP `ldapsearch` and FreeRADIUS `radclient` are this kind. A named tenant, production directory, hardware NAS, or relying party is a different claim and is still absent. |
| `release/deployment` | A release-workflow artifact, an installed copy of one, or a rehearsal outside disposable fixtures. |

Rows in the category sections that still say `ancestor-local` or
`accepted-head local` are the `09e6846` wording. Read a riAuth or disposable
database run as `source-only`. Read the FreeRADIUS `radclient` run as
`local peer`. Read `release artifacts` as `release/deployment`. A
`missing measurements` row is a gap, not a fourth kind of passing evidence.

## Verdicts

| Category | Status | Strongest record at this snapshot | Gate that remains |
| --- | --- | --- | --- |
| Usability | not passed | source-only password invitation at `58e5ef3` and simulated passkey invitation at `0312427`, each 3/3 | Independent new-user completion, assistive technology, a hardware or synced passkey, external mail |
| Workflows | not passed | source-only configured consent at `a3ccbf1`: prior suite 8/8 and TOTP consent 1/1 | Browser and remembered consent, TOTP-only and recovery-code reauthentication, custom stages |
| Administration | not passed | source-only redb 11/11 and disposable PostgreSQL 9/9 at `cf4529e`, including encrypted reopen and fenced promotion | Human operator journey, broader dependency families, and resources still outside exact multi-party approval |
| Interoperability | not passed | local peer OpenLDAP `ldapsearch` 2.7.1 LDAPS and STARTTLS at `fcba8a5` | Active Directory, a named directory application, a SAML service provider, a hardware NAS, and a cloud tenant |
| Footprint | not passed | source-only macOS edition matrix at `4ca7558` | Linux release-artifact size and a measurement at `0312427` |
| Performance | not passed | source-only `q09-8u-8g` reports at `7aea083` with `performance_claim: false` | Named operator workload, RPO, and RTO |
| Availability | not passed | source-only shared-store refusal at `7e07e748`, separate processes, disposable PostgreSQL | Shared-job leases, peer health, native TLS, and a deployment RTO or RPO |
| Recovery | not passed | source-only disposable drills, 16/16, no source commit in the JSON | Release/deployment restore, key escrow, and a measured recovery time |
| Migration | not passed | source-only synthetic reference relying party at `cf827a9`, focused 1/1 | A real Authentik export, a named relying party, and a production route rollback |
| Security | not passed | source-only A03 source seam through `ee88af0`, and source SPDX binding at `b759216` | Installed-artifact EG03, certification, a release signature, and a release SBOM |

## Slices accepted after `09e6846`

W02, A03, Q04, O03, S04, and G05 each have a reviewed slice on this base.
G05's slice is a disposable in-process rehearsal. None of these slices
meets its category target.

### W02 — source-only

Ledger integration commit `a3ccbf1dd47a8e25f35ad8d5d505768f32c1b942`.
[Workflows](../workflows.md) now describes three configured consent graphs:
session then consent; session, passkey, then consent; and session, local
password, current TOTP, then consent. Recovery codes are not an alternative
in the TOTP graph.

Ledger `run_head` `a3ccbf1`:

```text
CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=4 cargo test --locked --test workflow_configured_totp_consent --test workflow_configured_consent -- --quiet; python3 scripts/check-docs.py; python3 scripts/check-module-boundaries.py; git show --format= --check HEAD; git status --short
```

Result text: pass; existing consent 8/8, new TOTP consent 1/1. The W02
ledger note says browser-initiated and remembered-consent adapters,
TOTP-only reauthentication, and recovery-code reauthentication remain
unsupported. The ledger `open_checks` entry for unfinished configured
shapes is still present. Workflows stay not passed.

### A03 — source-only

Four ledger checks are on this history:

| `run_head` | Command | Result text |
| --- | --- | --- |
| `97511e4` | `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 cargo test --locked --test cloud_directory cloud_connection_probe_rechecks_revocation_after_upstream -- --exact --quiet; python3 scripts/check-module-boundaries.py; git diff 52df9a3..HEAD --check` | pass; exact midflight revocation regression 1/1 |
| `3aa997b` | `CARGO_BUILD_JOBS=2 cargo test --locked --test cloud_directory cloud_credential_preflight_replays_before_revision_and_provider_access -- --exact --quiet; python3 scripts/check-module-boundaries.py; git diff --check; git status --short` | pass; focused 1/1, zero forbidden refs |
| `8704bb8` | `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=4 cargo test --locked --test cloud_directory cloud_credential_write_rechecks -- --quiet; python3 scripts/check-module-boundaries.py; git show --format= --check HEAD; git status --short` | pass; midflight revocation/revision 2/2 |

The `8704bb8` note says `cloud_operations.rs` has no direct store or
mutation call and that the broader protocol transaction inventory remains.
The open check still says seven protocol files refer to Core and seven
refer to storage.

`1535cdcfce622f9422f59572d7f6e9b31c597e56` is the later A03 integration
commit on this base. Ledger `run_head` `1535cdc`:

```text
CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=4 cargo test --locked --test source_boundary -- --quiet; python3 scripts/check-module-boundaries.py; git show --format= --check HEAD; git status --short
```

Result text: pass; source configuration security boundary 1/1, zero
forbidden refs, clean accepted tree. The test is
`source_configuration_keeps_scoped_receipt_revision_and_audit_order`.
The note says the source-configuration mutation moved from the protocol
facade into assembly and that other source protocol transaction sites
remain. These checks do not pass security or administration.

### Q04 — local peer

Ledger integration commit `fcba8a5e81037edb365c02c291b512576817e920`.
The Q04 note says OpenLDAP `ldapsearch` 2.7.1 exercised separate riAuth
LDAPS and STARTTLS loopback listeners: CA-verified paged service bind,
disabled-user omission, wrong and revoked token rejection, wrong-CA
rejection, and crossed-scheme transport failure. It supports I04. It
claims no Active Directory directory, named production directory
application, SAML service provider, hardware NAS, or supplicant.

Ledger `run_head` `1caa093`:

```text
LDAPSEARCH=/opt/homebrew/opt/openldap/bin/ldapsearch CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 scripts/test-ldap-provider.sh; bash -n scripts/test-ldap-provider.sh; python3 scripts/check-docs.py; python3 -m json.tool docs/roadmap/coverage-inventory.json; git diff e3c26eb..HEAD --check
```

Result text: pass; OpenLDAP ldapsearch 2.7.1 STARTTLS bind/paging/disable/revoke 1/1.

Ledger `run_head` `fcba8a5`:

```text
LDAPSEARCH=/opt/homebrew/opt/openldap/bin/ldapsearch CARGO_BUILD_JOBS=2 bash scripts/test-ldap-provider.sh; python3 scripts/check-docs.py; python3 scripts/check-repo-hygiene.py; bash -n scripts/test-ldap-provider.sh; python3 -m json.tool docs/roadmap/coverage-inventory.json; git show --format= --check HEAD; git status --short
```

Result text: pass; OpenLDAP 2.7.1 LDAPS+STARTTLS ignored peer test 1/1.

The ignored test is
`ldapsearch_starttls_bind_scoped_paging_disable_and_revoke` in
`tests/ldap_provider_peer.rs`. [Platform LDAP provider](../recipes/platform-ldap-provider.md)
records `ldapsearch -VV` as OpenLDAP 2.7.1 (Sep 8 2026 21:55:18), OpenSSL
3.6.4, loopback ports `127.0.0.1:60248` (LDAPS) and `127.0.0.1:60249`
(STARTTLS), paged `uid: ldap-alice` and `uid: ldap-bob`, invalid-credentials
exit 49, certificate-verify failure, disabled-user omission, and revoke.
The check job and the integration job do not call
`scripts/test-ldap-provider.sh`. Interoperability stays not passed.

The FreeRADIUS `radclient` 3.2.10 loopback PAP result at `38f82fe` is the
same kind: local peer, not a hardware NAS.

### O03 — source-only

Ledger integration commit `7e07e748a80b9cb39ffed3da8ce90690d72ab18c`.
[Node security](o03-node-security.md) says O03 stays open. Opening compares
issuer and active capability names, refuses a mismatch before migration or
listen, and leaves listen address, `browser_ui`, and `[process]` local.

Ledger `run_head` `7e07e748`:

```text
CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 cargo test --locked --offline --lib node_security:: -- --quiet; RIAUTH_PG_TEST_TARGET=node_security_postgres CARGO_BUILD_JOBS=2 bash scripts/test-postgres.sh; CARGO_BUILD_JOBS=2 cargo check --locked --offline --no-default-features --features essentials --lib; python3 scripts/check-docs.py; python3 scripts/check-repo-hygiene.py; bash -n scripts/test-postgres.sh; python3 -m json.tool docs/roadmap/coverage-inventory.json; python3 -m json.tool docs/roadmap/capability-matrix.json; git show --format= --check HEAD; git status --short
```

Result text: pass; redb 2/2, disposable PostgreSQL separate-process 1/1.
The page names `agreement_tracks_issuer_and_active_capabilities_only`,
`open_refuses_a_different_active_set_without_rewriting_the_store`, and
`capability_or_issuer_mismatch_binds_nothing_and_preserves_postgres`.
The PostgreSQL run used loopback HTTP without native TLS and
`local_unencrypted` PostgreSQL. It did not promote the standby. The note
says shared-job leases, distributed rate limit, peer health, policy and
trust and key agreement, and native TLS or HA proof are still absent.

Two later O03 checks are also source-only and do not record a deployment
RTO. Ledger `run_head` `acf1d7c`:

```text
cargo test --locked --features test-support --test logout_lease --test background_logout --test operations --test identity_boundary -- --quiet; cargo test --locked --features test-support --test operations logout_network_failures_are_visible_and_clear_after_success -- --exact --quiet; RIAUTH_PG_TEST_TARGET=job_lease_postgres CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 bash scripts/test-postgres.sh
```

Result text: focused pass for background logout 1/1, identity boundary
6/6, logout lease 1/1, operations logout 1/1, and disposable PostgreSQL
two-worker lease 1/1. The same result says the full operations suite was
11/15, with four backup and schema failures. Ledger `run_head` `75def67`:

```text
cargo test --locked --lib node_security:: -- --test-threads=2; RIAUTH_PG_TEST_TARGET=node_security_postgres bash scripts/test-postgres.sh
```

Result text: pass; redb unit 4/4 and disposable PostgreSQL process and
upgrade 3/3. The note leaves deployment, rollback, and real multi-node
evidence open. Availability stays not passed.

`36ad584` is the earlier process-role commit on this history. Its ledger
result is process-role 6/6 and two-process disposable PostgreSQL 1/1.
That is source-only as well. Public CI, as the node-security page states,
does not select `node_security_postgres`.

### S04 — source-only

Ledger integration commit `e3c26eb2879729ce7971852d62ffdfb0e5083803`.
Ledger `run_head` `e3c26eb`:

```text
RIAUTH_PG_TEST_TARGET=reviewed_memberships_postgres CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 scripts/test-postgres.sh; bash -n scripts/test-postgres.sh; python3 scripts/check-docs.py; git diff 09e6846..HEAD --check
```

Result text: pass; disposable PostgreSQL reviewed-membership suite 6/6,
including fenced standby promotion and receipt/audit replay. The sixth
function is `postgres_z_fenced_standby_promotion_replays_reviewed_membership`.
[Reviewed group memberships](../reviewed-group-memberships.md) says the
test stops the primary with `pg_ctl -m immediate`, promotes the standby
with `pg_ctl promote`, and reads the same membership, audits, and receipt
on the promoted port. It is a loopback drill with trust authentication.
It does not elect a leader, measure a recovery objective, or cover
failback, partitions, or PITR. The encrypted TLS test from `52df9a3`
remains a separate function. That 6/6 result is the `e3c26eb` run. It is
not the latest accepted suite.

Later accepted S04 runs, still source-only, are on the ledger head's
history:

| `run_head` | Command | Result text |
| --- | --- | --- |
| `60e69f3` | `CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 cargo test --locked --test state_reconciliation -- --quiet; RIAUTH_PG_TEST_TARGET=reviewed_memberships_postgres CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 bash scripts/test-postgres.sh; python3 scripts/check-docs.py; git diff --check HEAD~1 HEAD` | pass; 7 redb state reconciliation and 7 disposable PostgreSQL reviewed-membership tests |
| `ef496c8` | `CARGO_BUILD_JOBS=2 cargo test --locked --test state_reconciliation client_name_desired_state -- --quiet; RIAUTH_PG_TEST_TARGET=reviewed_memberships_postgres CARGO_BUILD_JOBS=2 bash scripts/test-postgres.sh; python3 scripts/check-docs.py; python3 scripts/check-repo-hygiene.py; git diff --check HEAD^ HEAD` | pass; redb client-name 2/2, disposable PostgreSQL 8/8 with encrypted reopen and standby promotion |
| `cf4529e` | `cargo test --locked --offline --test state_reconciliation -- --test-threads=8 --skip desired_state_client_and_password_disables_require_review; RIAUTH_PG_TEST_TARGET=reviewed_memberships_postgres ./scripts/test-postgres.sh` | pass; redb 11/11, disposable PostgreSQL 9/9 including encrypted reopen and promotion |

The `cf4529e` redb command skips
`desired_state_client_and_password_disables_require_review`. The 11/11
count is the tests that command ran. At accepted `ee88af0`,
`tests/reviewed_memberships_postgres.rs` has these nine PostgreSQL tests:
`postgres_client_name_desired_state_dependencies_and_replay`,
`postgres_group_desired_state_dependencies_and_replay`,
`postgres_unrelated_revision_still_applies_reviewed_membership`,
`postgres_affected_membership_user_and_policy_deny_stale_apply`,
`postgres_reviewed_membership_reopen_and_receipt_replay`,
`postgres_reviewed_membership_http_execute_race_replays_after_reopen`,
`postgres_encrypted_reviewed_membership_replays_across_reopen`,
`postgres_user_display_name_desired_state_dependencies_and_replay`, and
`postgres_z_fenced_standby_promotion_replays_reviewed_membership`.
Accepted `docs/testing.md` at `ee88af0` says the same runner applies one
group-only desired-state plan, one client display-name plan, and one user
display-name plan. A manifest that also names another family, changes
client scopes, or changes a user's email still conflicts on the global
revision. The promotion sentence is unchanged: loopback trust
authentication, no leader election, no recovery objective, no failback,
no partition, and no PITR.

The `cf4529e` note says broader families and real deployment evidence
remain open. Administration and availability stay not passed.

Ledger `open_checks` entry 14 still says S04 redb concurrency plus
PostgreSQL 3/3, and that standby promotion and encrypted PostgreSQL
remain open. That sentence is stale. `52df9a3` recorded encrypted
reopen, `e3c26eb` recorded promotion at 6/6, and `cf4529e` recorded
redb 11/11 with PostgreSQL 9/9 including both. Correcting the sentence
does not pass S04, administration, or availability.

### G05 — source-only

Ledger integration commit `cf827a9ee103672b329907a433fb96ac868fe178`,
status `task_in_progress`. [G05 local reference OIDC](g05-local-reference-oidc.md)
calls the slice a bounded local source rehearsal. The requested real
target application has not been supplied. No production Authentik export,
relying party, routing change, or rollback was used.

The test is `local_reference_rp_cutover_rehearsal` in
`tests/g05_reference_oidc.rs`. Its source SHA-256 is
`64e43530f6c6e69c2fe4f7f9d45fabb2cfb5b1843a6b022eec2a418008602ba6`, the
hash the page records. The page's own observation command used a private
Cargo target from base `a3ccbf1` plus that test source. The accepted
ledger command, `run_head` `cf827a9`, is:

```text
CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=4 cargo test --locked --test g05_reference_oidc -- --nocapture; python3 scripts/check-docs.py; python3 scripts/check-repo-hygiene.py; git show --format= --check HEAD; git status --short
```

Result text: pass; synthetic reference OIDC rehearsal 1/1 with eight
asserted case IDs. The page names those IDs G05-01 through G05-08:
import and preflight, denied access, successful access and claims,
refresh rotation and replay, MFA, a new riAuth recovery code, logout,
and a local route-file rollback boundary. The test uses a disposable
redb fixture and an in-process reference relying party. It starts no
browser, network listener, or external Authentik service. The route-file
check restores a disposable JSON file and does not prove that Authentik
credentials still work.

That run is source-only. It is not a local peer, and it is not a
release/deployment rehearsal. [Migration](../migration.md) says these
in-process results do not close the real application cutover or rollback
gate. Migration stays not passed.

### O06 — source-only

Three bounded diagnostics are on the ledger. Each page read from
accepted `ee88af0` says O06 stays open. None is a dashboard deployment.
`doctor.healthy`, `/readyz`, and `/livez` keep their existing answers.

| `run_head` | Command | Result text |
| --- | --- | --- |
| `7949aeb` | `CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 cargo test --locked --test offboarding offboarding_diagnostics_reports_incomplete_and_failed_without_secrets -- --exact --quiet; python3 scripts/check-docs.py; python3 scripts/check-repo-hygiene.py; git diff --check ba64b66..HEAD` | pass; focused diagnostics 1/1 |
| `7949aeb` | `CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 cargo test --locked --test offboarding -- --quiet` | FAIL 24/30. The note says two representative failures reproduce on pre-O06 `ba64b66`, and four were not baseline-checked |
| `644242f` | `cargo test --locked --test reconciliation_diagnostics -- --quiet; python3 scripts/check-docs.py; python3 scripts/check-repo-hygiene.py; python3 scripts/check-module-boundaries.py` | pass; O06 diagnostic 1/1 |
| `9a575d5` | `CARGO_BUILD_JOBS=2 cargo test --locked --offline --test offboarding_deactivation_diagnostics -- --test-threads=1; CARGO_BUILD_JOBS=2 cargo test --locked --offline --test offboarding -- --test-threads=4; python3 scripts/check-docs.py; python3 scripts/check-repo-hygiene.py; git diff --check 3f0a9c9..HEAD` | pass; deactivation diagnostics 2/2, offboarding 31/31. The result text says PostgreSQL was ignored in this command and attributes disposable PostgreSQL 1/1 to the source run |

The 24/30 failure is that earlier check. It is not the last offboarding
result. `6121456` later records the full offboarding suite 31/31 twice.
O06 stays `task_in_progress`.

`docs/roadmap/o06-offboarding-diagnostics.md` caps the job aggregate at
50 items and 32 non-succeeded targets, and it scans the whole
`offboard_jobs` bucket with no incomplete-downstream index. Its open
list is dashboards, connector lag, node mismatch, storage pressure and
key problems, failed jobs outside scheduled offboarding, doctor and
`queues.offboard_jobs.failed`, deactivation rows, and a production
deadline for the full job scan.

`docs/roadmap/o06-deactivation-diagnostics.md` pages
`provisioning_deactivations` by 128 and retains at most 50 attention
rows. Each stored value is still decoded in full and has no size cap on
that path. Its open list adds Essentials redacted aggregate, a
`riauthctl` or new `riauth` diagnostics command, deactivation dispatch,
the connector due cursor, mail, provisioning-job error text, and
Prometheus or Grafana.

`docs/roadmap/o06-reconciliation-diagnostics.md` caps the controller
failure aggregate at 50 rows. Its open list adds the 256-job retention
as short of a production backlog deadline, and it does not extend the
offboarding aggregate. Administration stays not passed.

### Q06 — source-only

Ledger `run_head` `58e5ef3`:

```text
(cd tools/browser && npx playwright test invitation-password.spec.js --reporter=line); node --check tools/browser/invitation-password.spec.js; git diff --check b77e637..a4d0a1f
```

Result text: pass; accepted Chromium, Firefox, and WebKit browser
journey 3/3. The note says the test covers keyboard and mobile-viewport
password invitation acceptance, expired and replay rejection, no session
on acceptance, later sign-in and sign-out, and zero passkey requests.
It does not add a physical or synced passkey, a real mobile device, a
screen reader, or external mail. An earlier Chromium virtual-authenticator
invitation at `a72a086` is a separate 1/1 and leaves the same hardware
and mail gates open.

Ledger `run_head` `0312427` is the later simulated passkey invitation:

```text
(cd tools/browser && npx playwright test invitation-passkey-shim.spec.js --reporter=line); node --check tools/browser/invitation-passkey-shim.spec.js; python3 scripts/check-docs.py; git diff --check HEAD^ HEAD
```

Result text: pass; simulated passkey invitation Chromium, Firefox, and
WebKit 3/3. The note says the shim sets user verification itself. It does
not prove browser-native, hardware, synced, phone, mobile OS, screen
reader, or external mailbox behavior. Both journeys use the in-tree
portal fixture. They are source-only, not a local peer. Usability and
workflows stay not passed.

### W07 — source-only

Ledger `run_head` `f911e85` rejects a guest body above the translation
fuel or timeout budget before Wasmi `Module::new`. Commands:

```text
cargo test --locked --offline --features platform --lib extension_gate:: -- --test-threads=2
cargo test --locked --offline --no-default-features --features essentials --lib extension_gate:: -- --test-threads=2; python3 scripts/check-docs.py; git diff --check
```

Result text: pass; Platform extension gate 9/9, then Essentials 3/3.
The note says a guest that fits still validates and executes on the
caller without a wall-clock interrupt, and the native host remains
unwired.

Ledger `run_head` `3cfe28b`:

```text
CARGO_BUILD_JOBS=2 cargo test --locked --offline --features platform --lib extension_gate:: -- --test-threads=2; CARGO_BUILD_JOBS=2 cargo test --locked --offline --no-default-features --features essentials --lib extension_gate:: -- --test-threads=2; python3 scripts/check-docs.py; git diff --check HEAD^ HEAD
```

Result text: pass; Platform extension_gate 10/10, Essentials 3/3. The
note says guest output reads at most one declared workflow label, capped
at 32 bytes, before copying from Wasm memory, and a larger length is
denied without a read. Pinned Wasmi still lacks safe wall-clock
interruption for a pure guest call. The native host and other custom
graphs remain held. Workflows and security stay not passed.

### Q11 — source-only

Ledger `run_head` `b759216`:

```text
python3 -m unittest discover -s tests -p test_spdx_sbom.py -v; python3 -m unittest discover -s tests -p test_release_evidence.py -v; python3 -m unittest discover -s tests -p test_installed_release_gate.py -v; python3 scripts/check-release-evidence.py; bash -n scripts/package-release.sh; python3 scripts/check-docs.py; git diff --check HEAD^ HEAD
```

Result text: pass; SPDX 33/33, release evidence 14/14, installed gate
6/6. The source audit reports no release execution and no produced
release SBOM. Accepted `docs/roadmap/q11-release-evidence.md` says
`package-linux` binds the Linux packager to exact Essentials, Platform,
and riauthctl archive and image bytes, and that the fixtures do not run
`package-release.sh`. `sbom.release_sbom_produced` and release execution
stay false. `sbom.packager_source_calls_producer` true records a source
call, not a packager run. Open gates on that page: a signature, a release
SBOM from a real packaged artifact run, a separate review record, a
published asset set checked for a named tag, and Linux ARM64 execution.
The installed-gate unit result is source-only. It is not a
release/deployment artifact. Security, footprint, and recovery stay not
passed.

### A03 source seam — source-only

Accepted source-transaction moves end at ledger head's parent
`ee88af0d02d4a8c1d2f7a9a903f00890aeaef7b1`. Ledger `run_head` `ee88af0`:

```text
CARGO_BUILD_JOBS=2 cargo test --locked --offline --features test-support --test identity browser_link_finish_needs_the_original_fresh_local_session_and_rolls_back_whole -- --test-threads=1; python3 scripts/check-module-boundaries.py; python3 scripts/check-docs.py; git diff --check HEAD^ HEAD
```

Result text: pass; browser link rollback and one-use regression 1/1.
The note says `source_finish_browser` moved into assembly without
changing bind and deliver rollback, the stage and workflow guard, factor
charging, or audit. Wider A03 module-boundary work remains. Earlier
notes on this seam named leftover source writes at `a2481b0` and
`3f0a9c9`; those sentences belong to those commits. This slice did not
re-count protocol files. Ledger `open_checks` entry 7 still says seven
protocol files refer to Core and seven refer to storage, and that cloud
plan, apply, and snapshot cleanup remain in the protocol surface.
Security and administration stay not passed.

## Usability

Target: ordinary account and admin journeys, including A02 E04–E11, on a
declared browser matrix with keyboard, zoom, and assistive technology, plus
the workstream gate that someone other than the author completes the
documented flows.

| Class | Record |
| --- | --- |
| source-only | [Essentials guide](../essentials-guide.md) and [Platform guide](../platform-guide.md) say a claim that a person completed install, sign-in, the OIDC redirect, passkey enrollment, backup, restore, groups, claims, audit, LDAP, or SCIM on this revision needs a run, and that the page does not supply one. |
| ancestor-local | Ledger `run_head` `d039306`: `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 cargo build --locked --example portal_fixture; CARGO_TARGET_DIR=target ./node_modules/.bin/playwright test multi-authenticator.spec.js --project=chromium --project=firefox --project=webkit --reporter=line; node --check tools/browser/multi-authenticator.spec.js; python3 scripts/check-docs.py; git diff 9065f59..HEAD --check`. Result text: pass; headless two-passkey journey Chromium/Firefox/WebKit 3/3. The Q06 ledger note says the journey uses Playwright WebAuthn shims and a CSS mobile viewport. |
| ancestor-local | Ledger `run_head` `183915d` records a prose command, `U01 isolated two-node PostgreSQL bootstrap and Playwright setup.spec.js across Chromium/Firefox/WebKit`, result text `pass; PG 1, browsers 9`. The same ledger field is a description, not a shell line this slice re-ran. |
| ancestor-local | `/Users/dominik/.local/share/riwork/orchestrators/projects/891e7443-8dac-4c1b-897f-9e53cb59c7ee/planning/evidence/u01/evidence.txt`, dated 2026-09-28, base `roadmap/integration-accepted` at `14de533`. It records a Cua.ai Driver session that reached a backup Touch ID prompt. Backup completion was not claimed. It names screenshots `passkey-prompt.png` and `backup-prompt.png` in that directory. |
| missing measurements | No result at `52df9a3` for 320, 768, and 1440 CSS pixels, 200% text scaling, a screen reader, a physical security key, or an independent new user. `58e5ef3` later records a source-only keyboard and mobile-viewport password invitation on three engines. The Q06 note leaves a physical or synced passkey, a real mobile device, a screen reader, and external mail open. The U10 ledger note, integration commit `5892563`, records a focused 320px empty-workspace sign-in and says physical mobile and a real screen reader remain outside that run. |

Blockers: the guides withhold the human-completion claim; the browser runs are
ancestor-local shims; assistive technology and hardware authenticators have
no result at accepted HEAD.

## Workflows

Target: typed versioned workflows for authentication, enrollment, recovery,
consent, and sensitive actions, with a bounded executor. A02 P01 and gates
G05–G06.

| Class | Record |
| --- | --- |
| source-only | [Workflow model](../workflows.md) at `6d9b72b` describes W01, bounded W02 verifier paths, W03 proof provenance, a fail-closed reviewed pin, W06 authoring, and a held W07 extension contract. Configured consent and eight enrollment shapes are described there. Description is not a run at `52df9a3`. |
| ancestor-local | Ledger `run_head` `6d9b72b`: `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 cargo test --locked --test workflow_configured_consent -- --quiet; python3 scripts/check-docs.py; python3 scripts/check-module-boundaries.py; git diff bc3c324..HEAD --check`. Result text: pass; configured consent suite 8/8. `52df9a3` does not change workflow tests. The suite is repeated at `a3ccbf1`, together with the TOTP consent test, in the snapshot section above. |
| missing measurements | No end-to-end concurrency count on both storage backends for a reviewed pin, and no operator completion of a configured workflow on accepted HEAD. |

Blockers, from ledger `open_checks`: W02 still has unfinished configured
shapes, including invitation and other sensitive actions, browser and
remembered consent, and custom stages. W05's configured-run pin lacks a
multi-party approval record, safe resume of changed content, broad
environment binding, and end-to-end concurrency coverage on both storage
backends. The later W02 note at `a3ccbf1` adds the password-and-TOTP
consent graph and leaves browser, remembered-consent, TOTP-only, and
recovery-code adapters unsupported. The Q06 password invitation and the W07 guest cap are source-only records in the snapshot section. They do not finish browser reauthentication, remembered consent, custom stages other than the held guest, or an independent operator. Workflows stay not passed.

## Administration

Target: compact administration, wizards, one management service, delegated
administration, and exact-content review. A02 E12–E14, P14–P16, and G07.

| Class | Record |
| --- | --- |
| accepted-head local | Ledger `run_head` `52df9a3`: `RIAUTH_PG_TEST_TARGET=reviewed_memberships_postgres CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 scripts/test-postgres.sh; bash -n scripts/test-postgres.sh; python3 scripts/check-docs.py; git diff 6d9b72b..HEAD --check`. Result text: pass; disposable PostgreSQL suite 5/5, TLSv1.3 plus aes256gcm-v1 sealed record and encrypted race/reopen. On that commit the functions are `postgres_unrelated_revision_still_applies_reviewed_membership`, `postgres_affected_membership_user_and_policy_deny_stale_apply`, `postgres_reviewed_membership_reopen_and_receipt_replay`, `postgres_reviewed_membership_http_execute_race_replays_after_reopen`, and `postgres_encrypted_reviewed_membership_replays_across_reopen`. |
| source-only | At `6d9b72b`, [testing.md](../testing.md) says this PostgreSQL target does not open an encrypted connection. At `52df9a3`, `docs/testing.md` and `docs/reviewed-group-memberships.md` say one test restarts the disposable primary, verifies a loopback CA, and opens `aes256gcm-v1` records. Those `52df9a3` sentences are in this worktree. Accepted `ee88af0` adds the group, client, and user display-name plans described in the S04 snapshot. This worktree's product files were not updated to that text. |
| ancestor-local | Ledger `run_head` `4357f6b` records `cargo check; cargo test --test admin_ui` with result text `pass; six admin UI cases`. Later ancestor-local `admin_ui` checks include `91f66d6`, `74ee172`, `567ea3e`, `dbcdce2`, `d790f69`, and `5e47d00`. None of those `run_head` values is `52df9a3`. |
| missing measurements | No count of a person completing Applications, People, Groups, or Security administration on accepted HEAD. |

Blockers: `cf4529e` is the latest accepted redb and PostgreSQL drill,
11/11 and 9/9, including encrypted reopen and fenced promotion. `e3c26eb`
is the earlier 6/6 promotion run. Both are source-only disposable
clusters. Neither is a human administration journey. The O06 diagnostics
are bounded reads and leave dashboards, doctor, and a production backlog
deadline open. Ledger `open_checks` still says advanced
client credential and registration changes and other user, group, agent,
federation, key, session, device, and recovery resources remain outside
exact multi-party approval. The M05 ledger note is about logout endpoint
review. Administration stays not passed.

## Interoperability

Target: EG01. Name the application, directory, IdP, proxy, network, or cloud
tenant and its version, then exercise setup, lifecycle, negative inputs, and
failure handling. Mocks and in-process fixtures do not certify a family.
Q04 is "Test real peers" in the coverage inventory. The ledger now has a
Q04 slice at `fcba8a5`. That slice is local peer evidence, recorded in
the snapshot section above. It does not pass this category.

| Class | Record |
| --- | --- |
| source-only | Recipe pages under `docs/recipes/` describe in-tree or loopback fixtures. [Capability matrix](../capability-matrix.md) states the relying-party client is the in-tree axum fixture, the SAML IdP signer is xmlsec1, LDAP import follows disposable loopback OpenLDAP, upstream OIDC uses an in-process token endpoint, inbound SCIM uses `oneshot`, SAML source uses the in-process `Upstream` helper, and outbound SCIM uses a second loopback riAuth router. |
| ancestor-local | Ledger `run_head` `38f82fe`: `RADCLIENT=/private/tmp/riauth-fr-prefix/bin/radclient CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 scripts/test-radius.sh; bash -n scripts/test-radius.sh; python3 scripts/check-docs.py; git diff d039306..HEAD --check`. Result text: pass; FreeRADIUS radclient 3.2.10 real loopback PAP accept/reject/replay 1/1. |
| source-only | [Q03 pilot preflight](q03-conformance-pilot.md) says that on 2026-09-29, at commit `19a69c66b473480c8b570498231fad4d4edb7a31`, the pinned suite checkout, private configuration, `CONFORMANCE_SERVER`, and `CONFORMANCE_TOKEN` were absent. No independent OIDF plan was run. That named commit is not an ancestor of `52df9a3`. The Q03 ledger note says the same inputs are still required. |
| actual peer/tenant | No record in the docs read here, and no ledger check at `52df9a3`, names a completed Okta, Entra, Google, Active Directory, Workspace tenant, Vault, named relying party, named service provider, or named SCIM client run. |
| local peer | OpenLDAP `ldapsearch` 2.7.1 at `fcba8a5` and `1caa093`, and FreeRADIUS `radclient` 3.2.10 at `38f82fe`. Both are loopback. Details and commands are in the snapshot section. |
| release/deployment | No named tenant, production directory, hardware NAS, or relying-party deployment is recorded at `0312427`. |

Blockers: the local peer runs do not name Active Directory, a production
directory application, a SAML service provider, or a hardware NAS. D03's
ledger note says named deployment peers remain open. I10's open check
still asks for a real Workspace or Entra tenant lifecycle. Q03 still has
no independent plan. Interoperability stays not passed.

## Footprint

Target: exact official artifacts, missing-module rejection, and no weakening
of credential protection to reduce footprint (G04, G09). EG02 asks for native
Linux x86-64 and ARM64 packages. Byte size and memory are measurements.

| Class | Record |
| --- | --- |
| ancestor-local | [Q08 edition matrix](q08-exact-edition-bundles.md) records local macOS ARM64 builds at `4ca7558ee8563a0c5eedc1f13d4a9a6faa6a8d4a`, which is an ancestor of `52df9a3`. Essentials compiled 60 capabilities and excluded 26. Platform compiled 86 and excluded none. Direct normal dependencies were 38 and 52. Fresh redb and PostgreSQL readiness and admin login returned 200 on both editions. Native binary SHA-256 values are in that page. `target/dist` had none of the nine expected files per Linux architecture. |
| source-only | The Q08 reproduce command is `python3 scripts/check-exact-edition-matrix.py`. This slice did not run it. |
| release artifacts | No Linux x86-64 or ARM64 archive, container, checksum, or provenance file for `52df9a3` was in the tree or named by a ledger check at that head. The Q08 ledger note says shipped bundles remain unavailable. |
| missing measurements | No byte size or peak RSS for a binary built from `52df9a3`. Debug byte sizes in the Q09 page (Essentials `202086936`, Platform `285351464`) belong to commit `47aa248ce1c68284773746bbb5fd59c6098afdea`. That commit object exists in this repository and `git merge-base --is-ancestor` reports it is not an ancestor of `52df9a3`. |

Blockers: local macOS dev builds at an ancestor are not the release
footprint. Linux artifact size and an accepted-head memory bound are absent.

## Performance

Target: EG04. Record workload, dataset, hardware, security settings, peak
RSS, p50/p95/p99, successful throughput, errors, and background interference.
Product load, RPO, and RTO require a named operator workload. There is no
promised numeric speed in the product contract.

| Class | Record |
| --- | --- |
| source-only | [Q09 benchmark slice](q09-benchmark-slice.md) and `scripts/q09_benchmark_slice.py` define one `GET /api/me` measurement. Reports set `observations_only` and `performance_claim: false`. `python3 scripts/q09_benchmark_slice.py --self-check` measures a fixture server, not riAuth. |
| ancestor-local | Ledger Q09 integration commits include `7aea083b3894cc964e75786200c9e7d952e8ab01`, an ancestor of `52df9a3`. Four orchestration files, `q09-8u-8g-*-accepted-7aea083.json` under the planning directory, have `source.commit` `7aea083b3894`, `product_run: true`, `observations_only: true`, `performance_claim: false`, and dataset `q09-8u-8g`. This slice read those fields and did not adopt the latency numbers as a target. The Q09 ledger note says those four macOS dev-profile reports were checked and that sustained background cadence, named thresholds, secure deployment variants, and Linux ARM64 release artifacts remain outstanding. |
| source-only | The numeric table in the Q09 page is commit `47aa248`, which is not an ancestor of accepted HEAD. Those figures stay observations of that other commit. Cache paths named on that page were not re-hashed here. |
| missing measurements | No session-read report whose `source.commit` is `52df9a3`. No TLS, database-encryption, or external-signing variant at accepted HEAD. No run long enough to overlap the 60-second maintenance cadence. No named operator RPO or RTO. |

Blockers: `performance_claim` is false on the reports that were inspected.
The S02 ledger note says the held v9 Group chunk mirror stays unaccepted
because 4 MiB lookups regressed and strict memory and performance gates are
unmet. [Testing](../testing.md) describes `scripts/characterize-contention.sh`
and `scripts/measure-backup-memory.sh` as local observation tools. This slice
found no accepted-head output file for either script.

## Availability

Target: probes, a single redb owner, PostgreSQL multi-node behavior with
fencing, and a measured outage result. The contract does not treat PostgreSQL
as database high availability by itself.

| Class | Record |
| --- | --- |
| source-only | [Availability](../availability.md) says `/readyz` returns 503 while storage is unavailable and `/livez` stays independent of storage. It says the local harness enables synchronous replication and checks primary crash, fencing, standby promotion, reconnection, and surviving session and refresh tokens. The same page says the fixture does not set an RTO or RPO for a deployment. |
| ancestor-local | Ledger `run_head` `f8c3602`: `CARGO_BUILD_JOBS=2 bash scripts/test-postgres.sh`. Result text: pass; fenced primary/standby failover. |
| source-only | `e3c26eb` promotes a disposable standby for reviewed membership. `cf4529e` repeats promotion inside the 9/9 PostgreSQL suite and still records no outage duration or RTO. `7e07e748` refuses an issuer or active-capability mismatch. `75def67` later records node-security redb 4/4 and disposable PostgreSQL 3/3 for token lifetimes and password history. `acf1d7c` records a two-worker logout lease 1/1 and a full operations suite of 11/15. The deployment, rollback, and real multi-node gates in those notes stay open. |
| release/deployment | No deployment RTO, RPO, or failed-request count is recorded at `0312427`. |

Blockers: the promotion and the node-security refusal are disposable
source-only runs. [Node security](o03-node-security.md) leaves shared-job
leases, peer health, and native TLS open. Enterprise features still need
their own multi-node acceptance. Availability stays not passed.

## Recovery

Target: G12 and EG03 restored-state invalidation, plus a rehearsed restore
with the deployment's keys and peers. [Testing](../testing.md) asks for
measured recovery time and data loss before an application moves.

| Class | Record |
| --- | --- |
| ancestor-local | [R05 local drill](recovery-drill-r05.md) cites [r05-local-2026-09-29.json](evidence/r05-local-2026-09-29.json) and [r05-postgres-local-2026-09-29.json](evidence/r05-postgres-local-2026-09-29.json). Both have `result` `passed` and 16 checks. Schemas are `riauth.recovery-drill/v1` and `riauth.postgres-recovery-drill/v1`. Binary SHA-256 values are `c125134b04154c39d58f953342cc20c2b50b512ebfc58e6fa6cf240a84aa2400` and `8e26d768ce419a6b31fc22bd517aaf7b07c359a74fc5801990b1b33271333ffa`. Neither file has a source commit field. Scopes are disposable localhost redb and a disposable loopback PostgreSQL cluster. |
| source-only | Reproduce commands in the R05 page are `python3 scripts/recovery-drill.py` and `python3 scripts/recovery-drill-postgres.py` against a binary built in a private Cargo target. This slice did not run them. |
| ancestor-local | The R05 ledger note says an accepted-tree PostgreSQL drill was rerun 16/16 and that deployment PITR and failover remain open. The JSON `external_gates` arrays name lost backup or database key escrow, missing configured secret files, Vault Transit and other external services, PostgreSQL PITR and multi-node failover, and a real OIDC or SAML relying party. The PostgreSQL file also names a TLS PostgreSQL connection as an external gate. |
| release artifacts | [Q10 installed release gate](q10-installed-release-gate.md) records this command at accepted head `679f2927885d9dc4dcc1c881fd972bcade70e07a`, an ancestor of `52df9a3`: `python3 scripts/check-installed-release-gate.py target/dist 679f2927885d9dc4dcc1c881fd972bcade70e07a Rhein-Industries/riAuth 1 1`. The page says `target/dist` did not exist and the command exited 1. The Q10 ledger note says missing local `target/dist` fails closed and actual Linux x86-64 and ARM64 runner execution remains outstanding. |
| missing measurements | No recovery time or data-loss figure for `52df9a3`, and no restore of a release artifact. |

Blockers: the 16-check drills are disposable observations whose source commit
is not in the JSON. Their own gate lists are still open. [Operational
recovery](../operational-recovery.md) says that page does not finish D04.

## Migration

Target: preflight, identity continuity, rehearsed cutover, and rollback.
G01–G03. A route in an inventory is remediation, not conversion.

| Class | Record |
| --- | --- |
| source-only | [Migration](../migration.md) documents Authentik bundle conversion and `riauth migration-preflight --file <input>`. For any system other than the Authentik bundle, `ready_for_plan` stays false and no manifest is produced. Classifications are not evidence that an application or factor works after cutover. |
| source-only | The G01 ledger row lists source commits `6dccbdf` and `97f27ec`. Neither is an ancestor of `52df9a3`. Its integration commits `c12bffe` and `248207d` are ancestors. The row's remaining text says real source-export peer migration and identity continuity are separate G02/G03. This slice did not re-run a migration at those integration commits. |
| source-only | The G02 ledger note says reimport tests passed and that a real Authentik export and application cutover remain open. The G04 note says re-enrollment copy was reviewed and that no real Authentik export was run. Those notes are ledger text. This slice did not re-run them at `52df9a3`. |
| source-only | G05 integration commit `cf827a9` runs `local_reference_rp_cutover_rehearsal` against a disposable redb fixture and an in-process reference relying party. The ledger result is focused 1/1 with eight asserted case IDs. The page keeps G05 in progress. |
| release/deployment | No cutover duration, export byte count, live relying-party subject, production route change, or Authentik rollback sign-in is recorded at `cf827a9`. |

Blockers: source preflight does not move a directory. The G05 run does not
supply the exact target: a real application's successful and denied access,
claims, MFA, refresh, logout, recovery, and rollback. Migration stays not
passed.

## Security

Target: Q01's 35 invariants as contracts, EG03 runtime demonstration on
installed artifacts, and release evidence that includes signatures and a
review record. A contract paragraph is not a passing test.

| Class | Record |
| --- | --- |
| source-only | [Invariants](../security/invariants.md) says existing tests were read, not run, and that Q02/Q05 recipes in that catalog's header are not claims of passing coverage. The current evidence-boundary table still marks several domains as partial or intended policy. [Threat model](../security/threat-model.md) maps the same boundaries. |
| ancestor-local | Ledger `run_head` `7edfc42`: `cargo test --locked --features test-support,fuzzing --test contracts --test identity_boundary --test bootstrap --test q05_replay_concurrency --test scim_oauth --test removal_safeguards --test workflow_model -- --quiet` (result text: pass; 54 redb shared contracts) and `CARGO_BUILD_JOBS=2 bash scripts/test-contracts-postgres.sh` (result text: pass; 54 PostgreSQL shared contracts). |
| ancestor-local | Ledger `run_head` `f8c3602`: `RIAUTH_PG_TEST_TARGET=q05_replay_concurrency CARGO_BUILD_JOBS=2 bash scripts/test-postgres.sh`. Result text: pass; Q05 PostgreSQL replay binding. |
| source-only | [Q11 release evidence](q11-release-evidence.md) says the release workflow, packager, and bundle checker do not sign artifacts. The Q11 ledger note records 26/26 producer tests, 13/13 release-evidence tests, and a 355-package source SPDX document at integration commit `281493d`. That note calls the document a source document. The Q11 page says a source SPDX file is not a release SBOM. This slice did not open the orchestration SPDX file and did not treat it as release evidence. |
| source-only | A03 checks through `ee88af0` move cloud reads, the credential write, the source-configuration mutation, and the browser source-finish transaction into assembly. Wider module-boundary work remains. `b759216` binds the source SPDX producer to the Linux packager's exact archive and image names. The accepted audit keeps `release_executed` and `release_sbom_produced` false. |
| release/deployment | No signature, published GitHub release, packaged release SBOM, or installed-release success on shipped Linux archives is recorded for `0312427`. Ledger `open_checks` says Q11 still requires signing, an SBOM, an independent release review, publication, a checked asset set, and Linux ARM64 execution. |

Blockers: source-only contract tests, A03 assembly checks, and a source
SPDX file leave the security category open. The product contract says
missing tests do not deliver the defaults. There is no certification
result and no EG03 run of installed artifacts at this head. Ledger
`independent_review` notes are slice comments, not a D05 category result.

## Ledger open checks that still block D05

The ledger `open_checks` array has 17 entries at this read. The ones that
block a category above are included there. The rest also stay open and are
not waived by this matrix:

- I08 Windows signing and LogonUI need a real Windows VM.
- Linux ARM64 shipped-bundle validation is outstanding.
- I10 verify-controller readiness is point-in-time; direct browser route and
  CSRF, revoked-agent and file-failure cases, and a real Workspace or Entra
  tenant remain.
- Essentials `cargo test --test identity` is not feature-gated and cannot
  compile Platform-only SAML, RADIUS, and SCIM tests.
- A03 protocol files still refer to Core and to storage.
- M03 still has direct mutation surfaces outside the one management service.
- S02 ordinary LDAP search no longer loads full Group values; Group writes
  and offline index rebuild still do, and whole-operation memory bounds
  remain open.
- The S04 open-check sentence still says PostgreSQL 3/3 and that standby
  promotion and encrypted PostgreSQL remain open. `cf4529e` records redb
  11/11 and PostgreSQL 9/9, including encrypted reopen and fenced
  promotion. Those drills are source-only. S04, administration, and
  availability stay not passed.
- `cargo fmt --all -- --check` fails on the accepted tree. The ledger
  names a 19,306-line diff and a format-only sweep. That check does not
  pass a category.

## What would be required before a category could pass

This slice does not perform these steps. They are the blockers, written as
the missing evidence:

1. Record the category's exact target at the commit being accepted.
   A source-only or local-peer pass stays in that kind until the target
   itself is the thing that was run.
2. For interoperability, name the external product and version and keep the
   log. A loopback OpenLDAP or FreeRADIUS run stays local peer.
3. For footprint, performance, availability, and recovery, attach the
   release-artifact hashes when the claim is about a release, and record
   RSS, latency, error counts, outage time, and data loss on that artifact.
4. For usability, workflows, administration, and migration, record an
   independent person completing the documented path, including the failure
   stops the runbooks name.
5. Leave the category `not_passed` until that record exists. Do not treat
   this file, the coverage inventory, or a ledger review note as that record.

<!-- D05 HISTORICAL SNAPSHOT END -->
