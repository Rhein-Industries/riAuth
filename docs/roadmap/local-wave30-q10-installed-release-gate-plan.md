# Q10 installed upgrade and recovery gate: pinned source audit

Date: 2026-10-03. Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Original task: `5c0a8de3-c038-47f0-b32a-9d88bbd2e6bd`.
Reservation: `wave30_Q10_exact_installed_release_gate_source_audit`.
Existing worktree: `ed9ac424-59f4-4520-905b-919aea3521eb`.
Own parent: `32cf065659d27a1fb8fdc88852461ead645f084a`.
Published source pin: `544d1340b80cd3e040dc13142cdcbc1d75fea4cb`.

This is a source audit and input proposal. No installed binary, migration,
restore, helper, compiler, service, workflow or proposed command ran in this
slice. The only write is this new report. No source alignment was needed.

## Recommendation and one next prerequisite

The installed gate already performs explicit, token-bound edition handoffs in
both directions. There is no justified production correction from this audit.
Its fresh initialization starts at schema 3 with agreement format 3; its two
server editions have one package version. It therefore does not exercise an
older-schema installed upgrade or a previous-release rollback. Reading the
schema/index and restoring the same-version backup cannot establish those
outcomes.

The smallest next prerequisite is **one root-selected, supported predecessor
and target version/schema pair with its compatible pre-upgrade backup input**.
Root must identify the claimed support boundary before a gate hunk is reserved.
The tuple needs exact source/version/edition/architecture and installed binary
identities for both ends, the predecessor's real schema/index/agreement
format, and a backup made and successfully read by the compatible predecessor
reader. Preserve its configuration, database and backup keys, and referenced
files privately. Obtain the target cohort identity from the separately owned
Q08 audit; this report does not redo that bundle inventory.

Neither LOCAL `9a819317efb3a13fa27cd86f884be2be00898fc0` nor LOCAL
`b619fe25269ccc150e473bbcde47cdb3623ef810` supplies that version pair. Both,
and published `544d1340`, declare server and client version `0.1.1`. Changing
a fixture's schema/version marker or renaming a LOCAL archive would supply a
synthetic source test, not that installed predecessor. If the intended release
claims no supported predecessor transition, record that scope explicitly;
do not silently convert its absence into a passing migration or rollback.

No existing script/workflow/product edit is proposed for immediate application.
After the tuple is reviewed, a separate reservation can decide the smallest
installed older-store drill. The current command has no predecessor input, so
it cannot be presented as already providing that drill. Original Q10 remains
unfulfilled by this report alone; root owns classification and status.

## Original acceptance and export witness

The exact project export was read from
`planning/current-tasks.json` under the project's RiWork orchestrator directory.
It was 244354 bytes, SHA-256
`0b38ca3eb8810a4628436eb5083f6329de848f3f4ddaa972c850601ad45c0835`.
The matching row's project and task UUIDs were checked and its complete details
read. Its outcome is: “Test installed artifacts, schema changes, build
transitions, restore, maintenance tools, and supported rollback.” Its gate
requires the appropriate evidence for security, compatibility, speed and
recovery separately. Completion reviews relevant implementation, tests,
documentation and released artifacts as applicable, and forbids report-alone
implementation completion. Proposed prerequisites are A09, O04, R05 and Q08.

The dated export still records `todo` and historical worktree
`b64ea3dc-db1c-4bf8-b82b-990a52a440fe`. Reading that export is not a fresh
task-API observation or permission to change assignments. This bounded support
reservation leaves the original row unchanged. No callable RiWork orchestration
read/send tool was exposed in this session; no provider substitute was used.

| Original facet | Current definition at 544d1340 | Actual evidence credited here | Remaining exact boundary |
| --- | --- | --- | --- |
| Installed artifacts | Checked cohort, native install, capability/version/feature/target checks, maintenance hash binding | Historical native LOCAL archive runs and installed local drill below | No exact checked-tag installed-gate success established for 544d1340 |
| Schema changes | Atomic schema 1/2 to 3 migration and newer-schema/index refusal | Source regression definitions; no fresh execution of them here | Current installed drill initializes schema 3; genuine older-store input missing |
| Build transitions | Read-only preflight, exact plan token and atomic E→P→E activation | f3 ARM local; 9a ARM PostgreSQL format-3 sample; b619 x64 encrypted container cohort | These are their named LOCAL cohorts, not current shipped artifacts |
| Restore | Authenticated v3, isolated target, imported-state invalidation and explicit recovery gate | f3 same-edition native restore and 16 PostgreSQL checks per edition | No current checked-tag restore execution credited |
| Maintenance tools | Edition-matched standalone init/keygen/restore, Platform handoff and source-edition agreement recording | Historical installed maintenance operations above | Old-agreement upgrade and release-input identity require explicit reviewed inputs |
| Supported rollback | Direct incompatible build refuses; documented rollback restores a compatible pre-upgrade backup | Same-version isolated restore and direct downgrade refusal, historical source guards | No installed predecessor-version restore/startup pair established |

No universal provider, host, human-study, full-suite, deployed-HA or new-run gate
is added to the original row. Historical PostgreSQL/restore evidence is useful
within its actual scope; it does not become exact release evidence by age or
by a later source publication.

## Current orchestration: complete body review

All 421 lines of `scripts/check-installed-release-gate.py`, all 127 lines of
`scripts/check-release-bundle.py`, all 85 lines of `scripts/package-release.sh`
and all 186 lines of `.github/workflows/release.yml` were read from immutable
544d1340 objects. These are body reads, not filename/hash-only checks. The
installed script is byte-identical to accepted explicit-handoff commit
`847439171c97e5d53bcc2f22be331d54cca9b99a`. The complete diff from initial
`4f5e735429694d73ef66a52923fc0dae3f28d74b` was also read: the changes add
package-document byte binding and the explicit upward handoff. No proposed
candidate was imported, evaluated or run.

Release workflow lines 15–39 validate the pushed tag/version and current main,
then require the reusable CI workflow. Lines 40–120 declare two native Linux
package jobs, build the two editions and maintenance binaries, package/smoke
them, and execute the installed gate before artifact upload. Later attestation
requests and upload are source definitions; publish requires both packages and
the combined bundle check before a draft release. There is no manual Q10
dispatch in this workflow. No tag, run, signature, attestation or publication
identity is invented here. The historical draft/public observations in Q11
are dated observations, not a new remote check.

The installed gate's `verify_assets` (85–137) binds repository, source, positive
run/attempt, native architecture, locks, toolchain, build features and current
package bytes before install. `install_member`/`install` (140–172) extract only
the selected regular binary/license/notices and bound binary size; maintenance
bytes must match provenance. `verify_installed_metadata` (175–216) checks
edition, features, target and common version. `riauthctl` is installed and its
version checked; this drill's authenticated requests use the installed
server's legacy CLI, not a claim of complete riauthctl command parity.

`drill` (277–385) then performs:

1. Essentials maintenance keygen and fresh init, local readiness/JWKS/login,
   and authenticated, verified encrypted `riauth.backup/v3` export. Its init
   has no `--database-key-file`, so this particular redb store is plaintext at
   rest; encrypted backup is a different property.
2. Platform preflight expects exit 5 and the node-security blocker. A ready
   plan and exact token activation precede Platform serve; revision advances,
   JWKS and administrator login are checked.
3. Direct Essentials reopening refuses. Preflight requires both version
   activation and edition provenance blockers. Platform maintenance plans and
   activates the exact Essentials token, then Essentials serves with the same
   JWKS and working login.
4. Essentials maintenance restores the original backup into a new redb
   directory. Status must show a pending recovery id and serving closed;
   direct serve refuses. Explicit completion with that observed id and the
   persistent-credentials confirmation precedes fresh serve. The old session
   must fail and fresh login/JWKS must work.

`main` (388–413) checks checkout HEAD against the requested source, performs
asset validation, requires Linux, installs into an owned TemporaryDirectory,
and emits `riauth.release-gate/v1` only after all assertions. `schema` and
`index_version` in the drill result are observations from current init, not
measurements of a schema change. `drill(binaries, root)` has no predecessor
parameter. The Q10 guide's old step 2 omits the upward plan/activation that
current source already requires; that stale prose is not a missing product
handoff. No guide edit is claimed in this reservation.

Q08 owns full shipped bundle inventory. Here the bundle/packager review only
establishes the gate's input and release ordering; no archive/layer/binary was
opened, extracted or executed.

## Format 3, schema, edition and recovery invariants

Complete production node-security bodies through `parse` (1–324), complete
`src/upgrade.rs` and `src/upgrade/activation.rs`, all 584 transition lines, all
794 recovery lines, all 458 offline CLI lines and all 398 backup CLI lines were
read. Relevant Core initialization/open (115–250), legacy restore/import
(368–591), whole `commit_restore` (594–769), and streamed scan/verify/restore
(557–810) were read. Unrelated Core/operations methods and the remaining
node-security unit-test suffix are not claimed as complete body reads.

- Core init writes schema 3/current index and agreement format 3. Core open
  checks issuer, retained edition dependencies, merged connector configuration,
  node agreement and capabilities before migration or startup backfills. A
  mismatch cannot be remedied by automatic startup adoption.
- Agreement format 3 compares issuer, active capabilities, token lifetimes,
  password history and all 16 effective rate thresholds. Omitted defaults and
  explicit defaults agree. Old formats 1/2 and a missing row refuse unchanged.
  Offline recording requires stopped writers, backup and BOTH confirmation
  flags. Only a reviewed absent row gets `--adopt-missing-agreement`; it does
  not overwrite present incompatible policy. A matching format-3 retry writes
  nothing. PostgreSQL's connected-client check is not proof all nodes stopped.
- `upgrade::migrate` uses one writer, accepts only schema 1–3, rejects future
  schema/index and incompatible prior activation, backfills schema-1 seeds,
  rebuilds indexes, records migrations and advances revision. There is no
  reverse schema migration; recorded reversibility is pre-upgrade restore.
  Activation's version comparison is package-semver based. Same-version source
  builds with matching compiled capabilities are not separate release cohorts
  to that fence. No older-binary execution proof is inferred from strict parser
  definitions.
- Edition plan covers complete config/store state and source authority;
  activation rechecks token, writer ownership, PostgreSQL other clients and
  source markers under the writer. It changes target stamps/history/revision,
  not identity records. Old/missing agreement, retained Platform dependencies,
  pending recovery and lineage changes cannot be cleared by a narrow handoff.
- V3 restore authenticates ordered records/trailer/transcript/EOF and schema/
  issuer before output, then re-authenticates during import. It requires a new
  output and, for PostgreSQL, an empty isolated target checked again under the
  writer. Key/config checks remain mandatory. Import, restored-state
  invalidation, index rebuild and identity/key validation commit together.
  Reopening the private candidate runs Core checks before publishing config.
  An old/incompatible agreement can therefore fail that reopen after import;
  it is not automatically upgraded by restore. Discard only the owned failed
  target and preserve source/backup. Do not serve a private pending candidate.
- Recovery invalidates transient authentication and admissions/freshness state,
  advances authority epochs/revision, retains persistent identity/credential/
  receipt contracts, and keeps serving closed until exact-id completion and
  explicit persistent-credential reconciliation. That confirmation is an
  operator assertion, not proof external credentials were reconciled.

The relevant instructions were also read in `docs/operations.md` (137–155,
451–503), `docs/editions.md`, `docs/recovery.md`, and the operational-recovery
failure/rollback section (890–920). Pre-upgrade backup and compatible reader
matter: an older v1/v2-only reader cannot be credited with reading a v3 backup.
Rollback uses a new target and compatible old reader, never an old binary on
the newer modified store. A reader predating restored-state protections needs
its actual security limits recorded; the current fence cannot be imputed to it.

The accepted nonrenewed 60-second admission limitation, SCIM stamps, connector
global fallback, receipt-secret handling, route-specific headers, PAM fallback
and Group scope remain untouched. None establishes paused-IO exclusion,
remote-peer understanding or deployed HA.

## Named historical executions and failures

All claims in this section come from pinned accepted records, not this slice's
runtime. Complete A08 RUN and encrypted supplement bodies and both 16-check
PostgreSQL restore JSON records were read. Public A09 JSON was parsed fully;
the relevant controller phases, helper outcomes, cleanup and limitations were
inspected. Their transport inventories were not independently re-extracted or
recomputed here.

- Q10's original `679f2927885d9dc4dcc1c881fd972bcade70e07a` invocation was
  `python3 scripts/check-installed-release-gate.py target/dist 679f2927885d9dc4dcc1c881fd972bcade70e07a Rhein-Industries/riAuth 1 1`:
  exit 1, missing dist, before provenance/install. Run fields `1 1` were local
  placeholders. Preserve this failure.
- The same Q10 record reports four metadata tests passed under
  `python3 -m unittest discover -s tests -p 'test_installed_release_gate.py' -v`.
  Current source defines six: complete manifest, missing asset, checksum/revision,
  rewritten-checksum byte mismatch, missing package document and missing run
  identity. The historical four are not a current six-test execution and do
  not run installed binaries or recovery.
- `6ca4779b68cf41e29af490d2aca6ea3d59e1f2bd` local ARM installed drill failed
  on the active-capability mismatch before Platform serve; separate same-edition
  restores passed. Preserve that failure.
- A08 product `f3aba63ac3b824843a40b99623f1619ef8edc19f`, final validator
  `730d373078c4b128373d7afa5f3e278f63b065fd`, recorded exit 0 from
  `python3 /evidence/container/local-installed-smoke.py` inside its owned native
  Linux ARM container. It reached E→P→E, encrypted v3 backup, isolated restore,
  recovery refusal/completion, downgrade refusal and stable keys. This was a
  LOCAL installed-drill adapter: GitHub run fields were unset, dist absent,
  official `verify_assets`/release entrypoint not reached.
- That record's source commands were edition preflight (3/3), ignored
  cross-build Platform (1/1), ignored cross-build Essentials (1/1), and
  `--lib node_security::tests` (4/4). The later encrypted supplement records
  four ignored cross-build executions: two editions × plaintext/encrypted redb.
  These are historical source tests, not current binaries or current format 3.
- Encrypted transition validator `9f8efaf0372e02b0bc0042251d591f1facc0f783`
  recorded native local `check-local-encrypted-edition-transition.py --backend
  redb` and `--backend postgresql` passes on unchanged f3 binaries/probe.
  Redb preserved 30/33 logical rows; encrypted PostgreSQL preserved 36/39
  logical/ciphertext rows. Missing/wrong keys, authority/policy/token drift,
  old/absent/future agreement, live redb and connected PostgreSQL refusals were
  checked. Current encrypted helper lines 55–62 still require format 2;
  this historical oracle cannot simply be run/credited as current format 3.
- Restore adaptation `082563b362acb218f276e4dbcaf476e303b561b7` recorded
  `recovery-drill-postgres.py --binary /artifacts/<edition>/riauth
  --maintenance-binary /artifacts/<edition>/riauth-maintenance --pg-bin
  /usr/lib/postgresql/17/bin --evidence <owned-path>` passing 16 named checks
  each for Essentials and Platform on f3/PG 17.11: tooling/isolated cluster,
  source readiness/login, backup/outage/restart, wrong key/occupied target,
  verified encrypted restore, gated serve, wrong recovery id/missing
  attestation, completion, old-session refusal and restored health/login.
  This is logical restore, not physical/PITR or previous-release rollback.
- Preserved A08 failures include the PostgreSQL backend-drain retry, initial
  provenance-order archive oracle, image maintenance-boundary setup, probe
  compilation PATH, redb initial-revision oracle and first Platform restore's
  omitted revision/idempotency inputs. Later passing evidence does not turn
  those attempts into passes.
- A09 native x64 `37046857550` / `110970324302` passed 11 recorded steps,
  including five LOCAL native archives, at product b619. Its root record says
  archive slice passed, official release false, shared gate not_run; root did
  not retain/hash the outer ZIP. It is not Q10's shipped installed drill.
- A09 native ARM PostgreSQL `37082572962` / `111086077482` passed the focused
  shared helper in 20.219462 s using product 9a and PG 16.15. It recorded
  format 3/all effective rates, 57 preserved upgrade and 68 downgrade rows,
  ordinary identity/authorization sample, authentication/general-rate and
  other-client refusals, and E→P→E. Full shared gate not_certified and release
  gate false remain. Cleanup reports one reaped process/no live owned PG;
  pidfile verification false remains an explicit limit. This sample was
  plaintext PostgreSQL, not encrypted restore.
- A09 x64 prebuilt `37099059596` / `111134825816` passed its 11 checks at
  product b619/helper `6726dfd945445873214e934aa1f20815dc99ab24660b2205a45fd43c1486f221`.
  It exercised encrypted redb/format 3, keys/ordinary authorization/logouts,
  issuer/policy/rate refusals and exact E→P→E offline packets: 18 fresh reader
  observations, 10 packets, 2700 owned steps joined, zero remaining owned
  groups/containers/volumes. Physical redb bytes differed by packet; protected
  logical comparisons supplied the oracle. No build that run, official release
  false, shared full gate not_run. Earlier failed causes stay UNKNOWN.

Current source also defines
`operations_tests::schema_upgrade_is_atomic_preserves_credentials_and_rejects_future_versions`
in `tests/identity/operations.rs` (5007), and
`newer_activation_evidence_blocks_rollback_without_mutating_identity_state`
in `tests/operations.rs` (233). Both complete named bodies were read. The
former constructs a synthetic schema-1 store; the latter checks future release,
index, capability and revision using full snapshots. Their definitions are
relevant security oracles, but no actual named execution from current 544d1340
was established here. Their checks are not an installed older-release input.

A09 ARM `37101183416` was described by the assignment as owning the validation
lane. Its state/result was not queried. No current runtime pass is credited.

## Future commands and bounded execution prerequisites

These are prospective commands from existing source, not executed or authorized
by this report. The exact current installed entrypoint, in the verified
release-cohort checkout and native Linux architecture, is:

```sh
python3 scripts/check-installed-release-gate.py "$verified_dist" \
  "$verified_source" Rhein-Industries/riAuth "$actual_run_id" "$actual_run_attempt"
```

All variables must name the actual cohort supplied by the release/Q08 owner;
`git rev-parse HEAD` and provenance must agree with source. LOCAL archives,
placeholder run ids, or current checkout binaries are not substitutions.
This command still covers same-version init/handoff/restore only. There is no
existing CLI flag that supplies the unresolved predecessor tuple.

For a reviewed old-agreement store, after backup and stopping every writer,
the current source-edition maintenance command is exactly:

```sh
riauth-maintenance --config "$reviewed_config" --json security-agreement-record \
  --confirm-authentication-policy --confirm-rate-limits
```

Only a deliberately reviewed missing row adds `--adopt-missing-agreement`.
Use the source-edition build implementing this procedure; do not infer that an
older binary understands format 3. Handoffs use `transition-plan --target
<edition>` and `transition-activate --target <edition> --token <observed-token>`
on the exact reviewed config. Restore uses the compatible reader's actual
`restore --backup <verified-backup> --key-file <private-key> --out <new-target>`
contract, plus its supported database-key/backend flags. The earlier reader's
support must be inspected before these arguments are treated as runnable.
Recovery completion uses the pending id actually returned and requires real
reconciliation; no canned id or automatic confirmation is a test of that work.

Before a later runtime reservation, root must supply the tuple above and one
exclusive, owned native execution window after the preceding job exits.
Use a private 0700 fixture/output parent and 0600 inputs/captures; no shared or
deployed database. For any separately authorized build, private target,
jobs 1/incremental 0/dev+test debug 0 and locked inputs remain required. For
hosted preparation retain the accepted 30 GiB start/10 GiB stop/8 GiB floor,
finite deadlines and sampled resources; sampling is not a hard disk quota.
No tool setup/download/build is released here.

Current gate limits are concrete: 120 s per captured CLI process, 25 s readiness
loop with individual 2 s HTTP waits, and direct-server terminate/wait 5 s then
kill/wait 5 s in `finally`; an owned TemporaryDirectory holds the fixture.
The release job has a 120-minute job timeout. The script does not itself provide
a process-group supervisor, total drill deadline, capture-byte quota or resource
sampler. A later reservation must state who supplies those bounds and verifies
owned child absence and fixture cleanup, without claiming A09's controller is
automatically inherited. Persist finite status/exit/hashes and redacted results
before grading; preserve the first failure. Do not publish credentials, backup
contents or raw private service logs.

For the selected pair, expected evidence must distinguish refusal of direct
rollback on the upgraded store from a successful compatible pre-upgrade restore
into a new target. Check actual pre/post schema/index/activation/agreement,
identity and credential preservation, fresh readiness/login and old-session
policy appropriate to the actual reader. Unsupported keys/config/policy/edition
must remain fail closed; neither startup adoption nor editing markers is an
acceptable way to make the fixture pass. This is one selected supported path,
not a new universal release matrix.

## Immutable witnesses and actual static checks

All source paths below refer to 544d1340 objects. Hashes are source/data identity
checks, not evidence those bodies executed.

| Path | Bytes / lines | SHA-256 |
| --- | --- | --- |
| `scripts/check-installed-release-gate.py` | 22321 / 421 | `cbe8dcb42b4b1c36dc8df00204103b9c955feb8057275a441f60f3072d2bf8b5` |
| `scripts/check-release-bundle.py` | 6473 / 127 | `3272d70201eb8eb91039e90f87a69b694d6ee7fd95cd5f1f4d70b990c6d7a001` |
| `scripts/package-release.sh` | 4762 / 85 | `3060b16559b004e6e213d7ed484f575a7375df3d92d29b39fcf0726f936a2f0a` |
| `.github/workflows/release.yml` | 9864 / 186 | `da8607c5ea33a906185141e0caebec18c2282c22676981e0c659fd65eef4369f` |
| `src/node_security.rs` | 37959 / 956 | `979f1d22a4835bce97866e302dc2a8a21df4f0d383d4bd1e6960b6a5d7a3bcb7` |
| `src/upgrade.rs` | 3417 / 83 | `3958ce14d5b72a045ddad203e1d0f60a312a2b8eb40cd52c7c2bca4b3c685e6e` |
| `src/upgrade/activation.rs` | 6820 / 186 | `60b619e17d6a3a5992127420e84197e5214ce69b08a2247d07b1d42203a93cff` |
| `src/edition/transition.rs` | 23502 / 584 | `301548a213f422e3dfd37914a83f7bf93f8b69ae7f4d0753fc2dbd61295201cb` |
| `src/recovery.rs` | 28367 / 794 | `ff40bc88774658da7de79bfaabd845463ea338fe88847e3c570fbf690f3e1602` |
| `src/cli/local.rs` | 16437 / 458 | `63e988df595df7b6d6e410574e46904064697b4a78b7e4617430607375dcd0d5` |
| `src/operations.rs` | 49469 / 1266 | `38fc39aa54bb97b5d0cc2bf71969823e40337ab1d4604d47b9016760a805b02d` |
| `src/operations/stream.rs` | 26873 / 810 | `52e7b483cc94aaeecbdfecc6ddbf44e57cb09f573fcce9977958bbcfe64baf94` |
| `src/cli/backup.rs` | 13756 / 398 | `bb663d8fa7c883a332dbf0a33491ce4fc49e6d7fea6d0655119727d38bf7314c` |
| `scripts/check-local-encrypted-edition-transition.py` | 22045 / 398 | `09e137d88eb8e873a488448b7bdfdbe80d2327fbca716a93f445eaaae1ea9e8e` |
| `scripts/check-local-edition-transition-postgres.py` | 31339 / 556 | `32a2952e3b4c3ff2ab73a747238862d3446c31bf4896c869bc9efacb1ba8fe49` |
| `tests/test_installed_release_gate.py` | 5911 / 137 | `9bfc4130a0b4859825691b24ae3e9b155dab6a273ca066ebf35a479429828a12` |
| `docs/roadmap/q10-installed-release-gate.md` | 5281 / 97 | `174ce6aa0473ad5ea026d6a2771417eb5183f7a213ae5d87f8cadfe6514366a3` |

Historical records read at that pin:

| Record | SHA-256 |
| --- | --- |
| [A08 RUN](evidence/a08-native-linux-arm64-2026-09-29/RUN.md) | `dd6a3be1226b7c0d464b6639f40baa2150797e98ada4b9929a199705c105aa28` |
| [Encrypted supplement](evidence/a08-native-linux-arm64-2026-09-29/encrypted-storage/ENCRYPTED-RUN.md) | `ecb58a107f46ab1a0970bf0e1a90c3d18a2c9b8ada155391fcdae6a7f6c56c36` |
| [Essentials PostgreSQL restore](evidence/a08-native-linux-arm64-2026-09-29/encrypted-storage/local-pg-restore-essentials.json) | `eb1f30d115838098dc08d6731e8bcd93c34bf3b4121aa2a587a935babf14dbea` |
| [Platform PostgreSQL restore](evidence/a08-native-linux-arm64-2026-09-29/encrypted-storage/local-pg-restore-platform.json) | `9a6c81b113ab0cd8943b2d5c4c11a7dcca810044cc277562d33ebae97fc25a50` |
| [A09 native x64 root review](https://github.com/Rhein-Industries/riAuth/blob/544d1340b80cd3e040dc13142cdcbc1d75fea4cb/docs/roadmap/evidence/wave30-a09-native-x86-root-review.json) | `ccf7c3fb6c0bd85816097c32a24a1c1b0a67b9597346d571cd68975a4680b1af` |
| [A09 native ARM PG root review](https://github.com/Rhein-Industries/riAuth/blob/544d1340b80cd3e040dc13142cdcbc1d75fea4cb/docs/roadmap/evidence/wave30-a09-native-pg-37082572962-root-review.json) | `750b4c95b2598c3cb53a8a1b477638c7ac43aab70762efb786797a0a3089189d` |
| [A09 x64 bind replay root review](https://github.com/Rhein-Industries/riAuth/blob/544d1340b80cd3e040dc13142cdcbc1d75fea4cb/docs/roadmap/evidence/wave30-a09-linux-bind-replay-actual-root-review.json) | `fa9de597e3a5b74e20a317097c9d0622db5c15880c94a6d802f87595345db1c8` |

The two complete current local transition helper bodies and six-test fixture
file were read. The imported SPDX producer was parsed as data; its relevant
package-byte validator was read, not its entire inventory implementation.
Selected Q11 release/history prose was read for the source/attestation boundary;
its full bundle audit belongs elsewhere. No official release identity was
verified remotely.

Actual checks in this slice: Git object resolution/body reads and SHA/line
counts; Python `ast.parse` on six immutable Python files (installed gate,
bundle gate, both local transition helpers, SPDX source and metadata tests);
TOML data parsing of server/client versions at 544d1340, 9a and b619; whole
installed-script equality to 8474391 and original-to-current diff inspection;
Psych parsing of release YAML and exact two native matrix architectures;
`bash -n` on the pinned packager; docs and tracked-file hygiene baseline.
These passed. Some combined source display calls were truncated; relevant
missing spans were reread in bounded chunks rather than counted as read.
No parser imported the candidate, and no function/oracle was evaluated.

Post-write validation: all 17 source-table byte/line/SHA witnesses and all seven
historical-record hashes matched immutable objects; all full commit pins
resolved. The `tests/identity.rs` module binding was checked, correcting the
draft's schema-test prefix from `policy_tests` to `operations_tests` before
commit. This was a report correction, not a test failure or execution.
`python3 scripts/check-docs.py` passed both before and after the report;
`python3 scripts/check-repo-hygiene.py` passed on the preexisting 958 tracked
files and on the 959-file index containing this report. Staged docs and
`git diff --cached --check` passed; staged scope contained only this new report.
The report's final evidence wording was then updated before committing.
No runtime lane was taken or released, no current ARM result queried, and no
task status changed.
