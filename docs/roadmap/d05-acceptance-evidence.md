# D05 acceptance evidence matrix

Status: **not passed.** Read on 2026-09-29. This page classifies evidence for
the ten D05 category targets. Every category stays **not passed**. D05 stays
**not passed**. The workstream gate — a new user and a new operator can
independently complete the documented workflows — has no recorded completion.

The machine-readable twin is
[d05-acceptance-evidence.json](d05-acceptance-evidence.json). A status other
than `not_passed` is outside this slice.

## What was read

| Source | Identity |
| --- | --- |
| This worktree | `/Users/dominik/orca/projects/riAuth-public-preview-roadmap-d05-acceptance-evidence-wave24`, branch `roadmap/d05-acceptance-evidence-wave24`, commit `6d9b72bc30523d4310c28ac9348ab7fcd7a6ad46` |
| Accepted integration checkout | `/Users/dominik/orca/projects/riAuth-public-preview-roadmap-integration-accepted`, branch `roadmap/integration-accepted`, commit `52df9a3781bf34c7628a3021c114bf001f27a7fe` (`Exercise reviewed membership on encrypted PostgreSQL`) |
| Orchestration ledger | `/Users/dominik/.local/share/riwork/orchestrators/projects/891e7443-8dac-4c1b-897f-9e53cb59c7ee/planning/accepted-commits.json` |

`6d9b72b` is the parent of accepted integration HEAD. This documentation
commit is added on the D05 branch and does not contain `52df9a3`. The accepted
checkout was read and was not edited, merged, or pushed. The ledger
`integration_head` is `52df9a3781bf34c7628a3021c114bf001f27a7fe`, matching that
checkout. `integration_status` is `local_commits_only_no_merge_to_main`. The
ledger has 378 `integration_validation.checks` entries. One check has
`run_head` `52df9a3`. D05 does not appear in `accepted` or `reviewed_slices`.

The accepted HEAD delta against `6d9b72b` is four files:
`tests/reviewed_memberships_postgres.rs`, `scripts/test-postgres.sh`,
`docs/testing.md`, and `docs/reviewed-group-memberships.md`.

This slice did not run Cargo, a browser, a database, or a peer. Disk space on
the data volume was about 16 GiB free when the read started. Commands below
are citations of existing docs and of the ledger. They were not re-executed
here.

The [A01 coverage inventory](coverage-inventory.md) still says, at base
`96e23e2`, that there is no acceptance program against category targets. That
row is historical planning evidence and is left unchanged, as later Q and D
pages leave their inventory rows unchanged. The category targets cited below
are the frozen checks in [product contracts](product-contracts.md), including
G01–G12 and evidence gates EG01–EG04. Criteria in that contract are not
observed results.

## Evidence classes

| Class | Meaning in this matrix |
| --- | --- |
| `source-only` | A file, test, or document is present in the named commit. Presence is not an execution. |
| `accepted-head local` | A ledger `integration_validation` check whose `run_head` is the current accepted integration HEAD, `52df9a3`. The run used local or disposable processes. |
| `ancestor-local` | A ledger check, in-repo report, or orchestration report whose `run_head` or `source.commit` is an ancestor of `52df9a3`, or a local report whose source commit is not recorded. It was not repeated at `52df9a3` in this ledger. Older result text that says "accepted-head" names the head at that check, which is the `run_head`, not today's `52df9a3`. |
| `actual peer/tenant` | A named external product, directory, IdP, proxy, NAS, or cloud tenant outside this repository's fixtures and loopback programs. |
| `release artifacts` | Files produced by the release workflow for a tag, or an installed copy of those files. A local Cargo debug or dev-profile binary is a different artifact. |
| `missing measurements` | The category target asks for a recorded number, peer version, or independent completion, and this read did not find that record for `52df9a3`. |

A row may carry more than one class. None of these classes, alone or
together, passes a category. `ancestor-local` is listed so a historical pass
is not relabeled `accepted-head local`.

## Verdicts

| Category | Status | Strongest record found | Blocker that keeps it open |
| --- | --- | --- | --- |
| Usability | not passed | Ancestor-local headless Playwright at `d039306` | No independent new-user completion, no assistive-technology result, no physical authenticator |
| Workflows | not passed | Ancestor-local configured consent suite, 8/8, at `6d9b72b` | Ledger open checks: unfinished configured shapes, consent, and custom stages |
| Administration | not passed | Accepted-head local reviewed-membership PostgreSQL suite, 5/5, at `52df9a3` | One resource family on a disposable primary. Standby promotion and a human operator journey are absent |
| Interoperability | not passed | Ancestor-local loopback FreeRADIUS 3.2.10 PAP at `38f82fe` | No named external peer or tenant. Q03 independent plan was not run. Q04 has no ledger slice |
| Footprint | not passed | Ancestor-local macOS edition matrix at `4ca7558` | No byte size, RSS bound, or Linux release-artifact size at `52df9a3` |
| Performance | not passed | Ancestor-local `q09-8u-8g` reports at `7aea083` with `performance_claim: false` | No named operator workload, RPO, or RTO. The published numeric table is commit `47aa248`, which is not an ancestor of `52df9a3` |
| Availability | not passed | Ancestor-local fenced primary/standby script at `f8c3602` | [Availability](../availability.md) records no deployment RTO or RPO. The `52df9a3` suite does not promote its standby |
| Recovery | not passed | In-repo disposable drills, 16/16, with binary hashes and no source commit | External gates in those JSON files are still open. Q10 has no installed release assets for this head |
| Migration | not passed | Source preflight and Authentik classification docs | No real Authentik export or application cutover. No cutover duration |
| Security | not passed | Ancestor-local shared contracts at `7edfc42` (54 redb, 54 PostgreSQL) | Q01 catalog was read, not run, for its own evidence labels. No installed-artifact EG03 run, no certification, no release signature |

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
| missing measurements | No result at `52df9a3` for 320, 768, and 1440 CSS pixels, 200% text scaling, keyboard-only completion, a screen reader, a physical security key, or an independent new user. The U10 ledger note, integration commit `5892563`, records a focused 320px empty-workspace sign-in on three browser engines and says physical mobile and a real screen reader remain outside that run. |

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
| ancestor-local | Ledger `run_head` `6d9b72b`: `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 cargo test --locked --test workflow_configured_consent -- --quiet; python3 scripts/check-docs.py; python3 scripts/check-module-boundaries.py; git diff bc3c324..HEAD --check`. Result text: pass; configured consent suite 8/8. `52df9a3` does not change workflow tests. The suite was not repeated at `52df9a3` in the ledger. |
| missing measurements | No end-to-end concurrency count on both storage backends for a reviewed pin, and no operator completion of a configured workflow on accepted HEAD. |

Blockers, from ledger `open_checks`: W02 still has unfinished configured
shapes, including invitation and other sensitive actions, browser and
remembered consent, and custom stages. W05's configured-run pin lacks a
multi-party approval record, safe resume of changed content, broad
environment binding, and end-to-end concurrency coverage on both storage
backends. The same ledger's W02 note says the accepted consent suite is the
exact session-passkey-consent graph only.

## Administration

Target: compact administration, wizards, one management service, delegated
administration, and exact-content review. A02 E12–E14, P14–P16, and G07.

| Class | Record |
| --- | --- |
| accepted-head local | Ledger `run_head` `52df9a3`: `RIAUTH_PG_TEST_TARGET=reviewed_memberships_postgres CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 scripts/test-postgres.sh; bash -n scripts/test-postgres.sh; python3 scripts/check-docs.py; git diff 6d9b72b..HEAD --check`. Result text: pass; disposable PostgreSQL suite 5/5, TLSv1.3 plus aes256gcm-v1 sealed record and encrypted race/reopen. On that commit the functions are `postgres_unrelated_revision_still_applies_reviewed_membership`, `postgres_affected_membership_user_and_policy_deny_stale_apply`, `postgres_reviewed_membership_reopen_and_receipt_replay`, `postgres_reviewed_membership_http_execute_race_replays_after_reopen`, and `postgres_encrypted_reviewed_membership_replays_across_reopen`. |
| source-only | At `6d9b72b`, [testing.md](../testing.md) says this PostgreSQL target does not open an encrypted connection. At `52df9a3`, `docs/testing.md` and `docs/reviewed-group-memberships.md` say one test restarts the disposable primary, verifies a loopback CA, and opens `aes256gcm-v1` records. Those two sentences live on accepted HEAD, not in the `6d9b72b` tree this branch starts from. |
| ancestor-local | Ledger `run_head` `4357f6b` records `cargo check; cargo test --test admin_ui` with result text `pass; six admin UI cases`. Later ancestor-local `admin_ui` checks include `91f66d6`, `74ee172`, `567ea3e`, `dbcdce2`, `d790f69`, and `5e47d00`. None of those `run_head` values is `52df9a3`. |
| missing measurements | No count of a person completing Applications, People, Groups, or Security administration on accepted HEAD. |

Blockers: the accepted-head suite is reviewed group membership on a disposable
primary. Accepted `docs/testing.md` at `52df9a3` says the script still starts
a standby and does not promote it or perform `pg_ctl` failover. Ledger
`open_checks` says post-logout and front/back-channel endpoints gained
coverage in later M05 notes, while advanced client credential and registration
changes and other user, group, agent, federation, key, session, device, and
recovery resources remain outside exact multi-party approval. The M05 ledger
note's latest text is about logout endpoint review, not a full admin journey.

## Interoperability

Target: EG01. Name the application, directory, IdP, proxy, network, or cloud
tenant and its version, then exercise setup, lifecycle, negative inputs, and
failure handling. Mocks and in-process fixtures do not certify a family.
Q04 is "Test real peers" in the coverage inventory. The ledger has no
`reviewed_slices` entry whose code is Q04.

| Class | Record |
| --- | --- |
| source-only | Recipe pages under `docs/recipes/` describe in-tree or loopback fixtures. [Capability matrix](../capability-matrix.md) states the relying-party client is the in-tree axum fixture, the SAML IdP signer is xmlsec1, LDAP import follows disposable loopback OpenLDAP, upstream OIDC uses an in-process token endpoint, inbound SCIM uses `oneshot`, SAML source uses the in-process `Upstream` helper, and outbound SCIM uses a second loopback riAuth router. |
| ancestor-local | Ledger `run_head` `38f82fe`: `RADCLIENT=/private/tmp/riauth-fr-prefix/bin/radclient CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 scripts/test-radius.sh; bash -n scripts/test-radius.sh; python3 scripts/check-docs.py; git diff d039306..HEAD --check`. Result text: pass; FreeRADIUS radclient 3.2.10 real loopback PAP accept/reject/replay 1/1. |
| source-only | [Q03 pilot preflight](q03-conformance-pilot.md) says that on 2026-09-29, at commit `19a69c66b473480c8b570498231fad4d4edb7a31`, the pinned suite checkout, private configuration, `CONFORMANCE_SERVER`, and `CONFORMANCE_TOKEN` were absent. No independent OIDF plan was run. That named commit is not an ancestor of `52df9a3`. The Q03 ledger note says the same inputs are still required. |
| actual peer/tenant | No record in the docs read here, and no ledger check at `52df9a3`, names a completed Okta, Entra, Google, Active Directory, Workspace tenant, Vault, named relying party, named service provider, or named SCIM client run. |
| missing measurements | No peer version, no conformance result, and no failure-handling log for a named tenant at accepted HEAD. |

Blockers: loopback FreeRADIUS is a local program result at `38f82fe`, which
is an ancestor, and it is not a deployed NAS or a tenant. D03's ledger note
says an OpenLDAP 2.7.1 local import sequence was reviewed and that named
deployment peers remain open. I10's open check still asks for a real
Workspace or Entra tenant lifecycle.

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
| accepted-head local | The `52df9a3` reviewed-membership command starts that PostgreSQL script for one target. Accepted `docs/testing.md` says this target does not promote the standby. |
| missing measurements | No outage duration, no failed-request count, and no RTO or RPO at `52df9a3`. |

Blockers: the failover pass is an ancestor-local disposable cluster. The only
accepted-head database run does not promote its standby. Enterprise features
still need their own multi-node acceptance; the availability page says a
single-process regression does not establish that.

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
| missing measurements | No cutover duration, no export byte count, and no relying-party subject check against a live application at `52df9a3`. |

Blockers: source preflight and ancestor-local unit results do not move a
directory. The product contract's Essentials-to-Platform transition still
needs the exact artifacts named under footprint and recovery.

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
| release artifacts | No signature, published GitHub release, or installed-release gate success is recorded for `52df9a3`. Ledger `open_checks` says Q11 still requires signing, an SBOM, an independent release review, publication, a checked asset set, and Linux ARM64 execution. |
| missing measurements | No certification result. The Q03 section above records the missing pilot inputs. No EG03 run of the installed `52df9a3` artifacts. |

Blockers: ancestor-local contract tests and a source SPDX file leave the
security category open. The product contract says missing tests do not
deliver the defaults. Ledger `independent_review` contains many source-review
notes that use the word pass. Those notes are review comments on slices.
They are not a D05 category result, and this page does not promote them.

## Ledger open checks that still block D05

The ledger `open_checks` array has 16 entries at this read. The ones that
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
- S04 reviewed-membership redb and PostgreSQL cases passed at the time of
  that open-check text; the text still names standby promotion. The later
  `52df9a3` check covers encrypted PostgreSQL for reviewed membership and
  does not promote the standby. S04 is not closed by this page.

## What would be required before a category could pass

This slice does not perform these steps. They are the blockers, written as
the missing evidence:

1. Repeat the category's command at the commit being accepted, and record
   that commit as `run_head`. An ancestor pass stays `ancestor-local`.
2. For interoperability, name the external product and version and keep the
   log. A loopback fixture stays local.
3. For footprint, performance, availability, and recovery, attach the
   release-artifact hashes when the claim is about a release, and record
   RSS, latency, error counts, outage time, and data loss on that artifact.
4. For usability, workflows, administration, and migration, record an
   independent person completing the documented path, including the failure
   stops the runbooks name.
5. Leave the category `not_passed` until that record exists. Do not treat
   this file, the coverage inventory, or a ledger review note as that record.
