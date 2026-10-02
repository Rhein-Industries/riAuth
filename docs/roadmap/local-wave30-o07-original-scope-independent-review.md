# O07 original-scope independent review

Reservation `wave30_O07_original_scope_independent_review`, 2026-10-02.
Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original O07 task
`6c981199-62dd-464d-a12a-4ce4e27f428f`. Independent support used existing
worktree `ed9ac424-59f4-4520-905b-919aea3521eb`, branch
`roadmap/local-revisions-coordination-wave27`, clean starting HEAD
`a646c8f39096f1646b3f5e58e3cf69a24323b4f7`. Primary O07 worktree
`e1b4399a-8c0d-46b8-880c-a71a4ebf53e7` and its assignment are unchanged.

**Recommend retaining in_progress at this pin. One concrete local correction is
the distributed deployment guide's stale per-node forward-auth statement.**
The accepted implementations and actual database/service/recovery checks provide
substantial original-scope evidence. They must not be replaced by an all-todo or
universal deployment gate. They also do not turn either failed small-template
attempt into a pass. This review reserves no runtime or existing-file edit;
root owns the proposed paragraph reservation and final original-row disposition.

## Original row, fixed source and review method

The first project-specific read was
`riwork task list --project 891e7443-8dac-4c1b-897f-9e53cb59c7ee --json`.
The exact selected row was reread after the initial whole-list display truncated.
It reports **in_progress**, assigned to primary e1, with this original outcome:
“Cover small installations and distributed setups, with clear database HA and
recovery responsibilities.” Its workstream goal is to remain easy to run small
while supporting explicit, reliable distributed operation. The completion gate
is that an operator identifies the failing component, what remains safe and the
required corrective action. Prerequisites are O01, O03, O04, O05 and R05.
The row's older scheduling text is historical, not permission to create another
task or worker. Closed O03/S04/O06/R05 and the other closed rows are preserved.

Published audit pin: **`0d090169f20f61f7cb59b685dccb13203526539f`**, tree
`9d1dfa454da9646ca8c7f7df9513eb77a4591e5b`.
Executed CI source pins are separately
**`b619fe25269ccc150e473bbcde47cdb3623ef810`** and
**`b5dcfa9dbb14e953d12edac6a6a1133ecb61033e`**.
All source reads used immutable Git objects; no merge, alignment or file import
was needed. `CONTRIBUTING.md` was read. Its broad build/test guidance is limited
by this explicit source-only reservation.

Full body reads covered the deployment guide, both Compose templates, systemd,
Caddy, HAProxy, PostgreSQL JSON and Dockerfile; the corrected 737-line small
controller including both embedded helper literals, setup order and cleanup;
both PostgreSQL shell harnesses and the CI workflow; both complete
`tests/postgres.rs` test bodies, their service helper and shared identity-effects
helper; availability and restored-state recovery guidance; and the O01 duty
report. Recovery review included the disaster-recovery inventory/restore order/
validation and unrecoverable sections, the R05 root disposition, recorded drill
instructions/results, and the three complete JSON evidence records below.
Selected operations diagnostic/rate/lease sections and complete relevant
production functions were read separately from whole-file identity checks.
Readiness source reads covered `live`, `probe_ok`, `ready`,
`ready_with_check`, every `Cause` mapping, `storage_cause` and `observe`.
The relevant effective-rate and offline-adoption fixtures were also read.

For the large raw CI logs, complete bytes were read, decoded and hashed; checkout,
exact command, named result and summary lines were inspected. This is not a claim
to have reviewed every dependency-install/browser line or every one of the 91
contract bodies. Source/object equality is not additional runtime evidence.
Some combined displays truncated; relevant source intervals were reread in
bounded chunks. No runtime attribution was derived from omitted output.

## Published configurations and operator responsibilities

| Original facet | Published configuration/instructions and actual evidence | Boundary |
| --- | --- | --- |
| Small installation | `compose-small.yml`: one integrated UID/GID10001 server, one private named redb volume, read-only runtime root/config, bounded readiness healthcheck, tools-profile keygen/init, restart and graceful stop. `deployment-examples.md` prints image verification, owner-only mounts, password-stdin init, trusted host Caddy, local readiness and public discovery checks. `riauth.service` provides a dedicated-user alternative. Current Linux named single-owner/readiness regressions passed; R05 executed an encrypted-redb CLI/HTTP service and restore. | Those executions do not execute Compose/systemd/Caddy. Both assigned Docker Desktop mechanics attempts failed before application startup. No successful run of the exact printed container layout is established here. |
| Distributed setup | `compose-distributed.yml`: one integrated service per host, external PostgreSQL authority, no shared redb volume, read-only common config/key, explicit `[postgres]` guard, private ephemeral `/data`. Guide initializes once, copies common issuer/edition/revision/security/trust files, budgets eight PG connections per node and restricts backend access to the TLS load balancer. HAProxy checks `/readyz`, overwrites forwarding headers and documents its separate availability owner. | The exact guard is a template fail-closed check for an omitted stanza, not full TOML/database validation. Actual two-process PG failover is credited below; it does not execute two Compose hosts or this HAProxy configuration. |
| Database HA | Availability and deployment guides explicitly assign standby policy, synchronous durability, backups/WAL/PITR, writable routing, promotion and former-primary fencing to the database operator. `synchronous_commit=on` does not configure a standby. b5/b619 CI actually stopped a disposable primary before promoting its synchronous standby and recovered independently running service processes. | No automatic riAuth election/fencing, partition/failback result, deployed RTO/RPO or remote-host HA is inferred. |
| Recovery | Guide stops both service nodes, fences old primary and prints offline invalidate/status/reviewed-ID completion. Recovery/runbook inventory distinguishes archive contents from external keys/config/CA/TLS/connector dependencies. Same-lineage physical restore or possible commit loss needs explicit offline invalidation and persistent-credential review. Actual archive/native/RP evidence is credited below. | A snapshot cannot reconstruct later revocation/rotation. Recovery completion is the operator's attestation, not inferred reconciliation. Compose restore placement remains explicitly untested. |
| Component/safety/remedy gate | `/livez`, readiness observations, authenticated doctor/metrics, diagnostic next actions and linked incident/recovery guides expose bounded evidence. They distinguish storage failure, saturation, compatibility/recovery gates, duty omissions and downstream outcomes. | Readiness is a local observation; it does not certify healthy peers, available signer/tenants, delivered remote effects or storage headroom. |

The printed layout keeps integrated duties on each node. The older O01 report's
open headings are dated slice limits, not reopening O01: its actual gateway/
worker run passed once in 11.78s, with one failed logout delivery observed. It
also observed gateway readiness while the worker was absent, on loopback HTTP
and unencrypted PG. The b619 CI workflow does not select `process_role_postgres`.
Thus operator responsibility for coverage of omitted duties remains explicit.

## Actual Linux CI evidence, kept separate from template checks

The required raw b619 integration log is
`/tmp/riauth-wave30-ci-37041786903/integration-110960005237.log`:
**448957 bytes**, **3005 lines**, SHA-256
`f6b092d43208a33e0ebbdf6804e20e18ca3cd0e351cbf8face8943676e3e86d4`.
Checkout lines121–122 record full b619. The workflow uses Ubuntu24.04 and the
pinned Rust1.98.1 toolchain. These are existing CI executions, not commands run
by this review.

| Actual b619 command/result | Exact scope credited |
| --- | --- |
| `PG_BIN="$(pg_config --bindir)" scripts/test-postgres.sh`; lines1370–1394; **2 passed, 0 failed, 0 ignored, 0 filtered**, 15.37s | `postgres_atomicity_shared_sessions_replay_limits_migration_and_fenced_failover` and `postgres_single_connection_streams_and_audits_a_backup`. The harness expands to `cargo test --locked --test postgres -- --ignored --nocapture --test-threads=1`, without adding test-support for this target. |
| Same step logged **1095ms** fenced-crash/promotion recovery, line1390 | Actual disposable synchronous PG failover observation. The timer includes stop, ready/live checks, promotion and Core reconnection/token checks; it is printed before the subsequent HTTP reconnection assertions. It is not deployment outage duration, throughput or an RTO. |
| `PG_BIN="$(pg_config --bindir)" bash scripts/test-contracts-postgres.sh`; lines1396–1508; **91 passed, 0 failed, 0 ignored, 90 filtered**, 172.41s | Expansion: `cargo test --locked --features test-support --test contracts -- --ignored postgres --test-threads=2 --nocapture`. Actual named successes: 45 plain PG, 45 encrypted PG, one PG-archive-to-redb restore. Includes atomicity/authority/receipt/recovery checks; not 91 multi-node/HA/O07-specific tests. |
| `PG_BIN="$(pg_config --bindir)" RIAUTH_PG_TEST_TARGET=q05_replay_concurrency scripts/test-postgres.sh`; lines2864–2882; **1 passed, 0 failed, 0 ignored, 1 filtered**, 11.74s | `q05_postgres_replay_binding_and_interrupted_management`; expansion adds `--features test-support`, ignored/nocapture/single-thread flags. This is its executed schedule, not every selectable PostgreSQL target. |

The full failover test initializes encrypted redb, migrates it to PG, opens two
Core instances and starts two actual `riauth serve` children on loopback.
It verifies shared session access, stable keys/client config, rollback/preview,
prepared-authority revalidation without monopolizing a one-slot pool,
concurrent increments, one-winner code redemption/replay-family refusal and a
shared rate counter. After primary stop, both HTTP nodes return live200/ready503;
after promotion, Core access/refresh/signing and both HTTP authenticated reads
recover. Wrong-key reopen refuses. Its PG archive restores into gated redb,
preserving keys/identity while rejecting the old session. The second test uses
a one-connection PG pool for an actual >1MiB streamed backup and requires both
started/completed audit events. These are real bounded fixtures, not synthetic
peer flags. They use local unencrypted database transport and no load balancer.

The b5 integration raw file
`/tmp/riauth-wave30-ci-37037991415/integration-110940835076.log` is
**448955 bytes**, SHA-256
`980a781ca9afe5fb1083a9be20f5797c934de2d0e7c98a16323bb6993ec96424`.
Its checkout records full b5; the same two PG names passed **2/2 in17.16s**,
with **1119ms** logged recovery, shared contracts **91/91 in208.19s**, and Q05
**1/1 with1 filtered in13.98s**. Earlier run36950097067/job110661000640 is
historical at `2d05c00deed3a6dafcd458a5f1a1a9beb17d0dd8`: raw265770bytes,
SHA-256 `458a30895b5987c0bebe2fb7368a1a32969fa0d41ce90acbfa56f43299ccd186`,
logged recovery1113ms. It is not an execution at today's audit pin.

Current named small-operation/security evidence also appears in the existing
check logs under the actual command
`cargo test --all-targets --features test-support,fuzzing --locked`:

- `api::probes::tests::saturated_application_or_probe_workers_do_not_disable_liveness`;
- `readiness_causes_are_redacted_bounded_observations`;
- `readiness_fails_when_storage_is_invalid_while_liveness_stays_available`;
- `redb_refuses_a_second_or_shared_owner_with_actionable_diagnostics`;
- `node_security::tests::effective_rates_match_all_categories_and_reject_before_startup_writes`;
- `node_security::tests::format2_upgrade_preserves_recorded_policy_and_missing_adoption_is_explicit`;
- `node_security::tests::edition_handoff_preserves_rates_and_refuses_a_different_threshold`.

All seven named results are `ok` in both b5 and b619. b619 check raw
`/tmp/riauth-wave30-ci-37041786903/check-110960005512.log` is **818156bytes**,
SHA-256 `c549f7c8cc4c1e66ee8d8420d68a73cb12524ff4964ac9754670a515be55e346`;
the relevant line witnesses are1612,2944,3057,3859,1672,1665,1664 respectively.
b5 check raw818157bytes, SHA-256
`54b582c1af19299c38a06b77d4aa1e8918f2ffc3ad16f4d7979915c2284e0a5f`,
job110940834919. No overall/newer-CI or all-deployment pass is derived here.
The check command ignored PG contract variants; the separate integration step
actually ran them. Dedicated ignored `node_security_postgres`, mail/SSF/job
lease and connector targets were not selected by these commands and receive no
invented current-format peer execution credit.

## Actual recovery evidence and preserved failures

| Complete retained record inspected | Actual outcome and original limit |
| --- | --- |
| `docs/roadmap/evidence/r05-local-rp-2026-10-02.json`;6090bytes; SHA-256 `5498bbf1f089224947ef3f0cbc7e163c4aa35683eb8d0256943100f49012df41` | One redb drill, **19/19 passed**, 2026-10-02T11:55:03.815161+00:00 through11:55:10.231016+00:00. Runtime HEAD `b5cea614c4f46d82aff2380c052bd2dffc760f9e`; production c01c39a/tree `3adc2b59c3547d22bff202daccfd8ad97f1e78ab`; binary SHA-256 `0f137475af5a8040d96a794b1ad331e7430be4467046b81b1312fb974b7e8a6a`. Actual command `python3 scripts/recovery-drill.py --binary "$PWD/.target-wave27/debug/riauth" --evidence "$PWD/.target-wave27/r05-wave30-local-rp.json"`. Restore/refusal/outage/gate checks and actual synthetic public OIDC RP callback/S256/nativeRS256/userinfo/fresh-cookie protected access before/after restore passed; no-cookie403/fresh-cookie200. This is historical local c01 execution, not current/released/remote RP or GUI evidence. |
| `docs/roadmap/evidence/r05-postgres-local-2026-09-29.json`;4126bytes; SHA-256 `76ca34df1f243e68e2e43884a6192114756448603d70baa103cb9ce605bb0770` | **16/16 passed**, disposable encrypted PG16.14 archive restore. Database outage yielded live200/ready503/login storage_unavailable(exit6); wrong key left target empty; occupied target stayed safe; gate/session invalidation/fresh login passed. Binary SHA-256 `8e26d768ce419a6b31fc22bd517aaf7b07c359a74fc5801990b1b33271333ffa`. The retained record pins the binary, not a source revision; none is invented. The guide prints the `recovery-drill-postgres.py --binary ... --pg-bin ... --evidence ...` replay interface. Initial CLI restore panic remains documented as an earlier failure. |
| `docs/roadmap/evidence/r05-native-postgres-2026-09-29/report.json`;4989bytes; SHA-256 `0e524b94eb6b913a9d6dca5e0da3dd8dc0b2d8d783555ed2a02d896bf18a01dc` | **16 checks, passed_gate_closed**, native PG16.14 physical base-backup/verify and explicit invalidation;58 restored versus68 later source records. Same lineage incorrectly appeared serving-allowed before explicit invalidation; no restored listener started at that point. Wrong key/live-writer/gate/old-artifact refusals passed; stale password/signing key remained for reconciliation, completion unasserted. Binary SHA-256 `a5c01650917d16c9384b1d87fba373d02e97d5e85eb8a2e625d7c9a8cbcd68ea`; the record does not pin a source revision. No deployed fencing/PITR or credential escrow is inferred. |

The small-template controller has only historical source/render proof at
`97a30bd292da85752da7e71646811a51fd070bac`: preserved safety/topology render
fingerprint `62574f39005fbdfcfaff892a215808998cbcae171d383c3151b16a38d1a25be5`.
That was source-only rendering with the engine disabled. This review did not
run its `--check-source` mode or any other helper mode.

The complete failed JSON records are retained verbatim inside immutable
`docs/roadmap/local-wave30-o07-deployment-plan.md`, blob
`e9c638ae72c7d5d98b856ed50b70a8b98cfea5a3`,52783bytes, SHA-256
`202589fec95f51e4de1724ecde6d887a0732518edfb35e073bb099a60ddd36b3`.
Reconstructing each embedded sorted/indented JSON plus final newline reproduced
the original recorded size and hash; private ignored target files were not
read or changed by this review.

- First actual attempt at HEAD `394a4623d7b77cdc8d483a30ec9863144999c525`,
  original controller97a30bd: **exit1**, one image-provenance check passed,
  cleanupok. **1745bytes**, SHA-256
  `308dfe8d3b424ff9a4e54369d809f560847b6aefda43a83ca229842b34068adc`.
  Its generic `uid_mapping_unsupported` conflated a nonzero helper/setup result
  with a postcondition mismatch; no narrower original cause is established.
- Corrected actual attempt at HEAD `1291f52c2135ec6b1b2070fb669ec814d58da4b6`,
  controller `c8a34c8df5184d89f7a36d933f2ec172d01471cd`: **exit1**, one
  image-provenance check passed, cleanupok. **1878bytes**, SHA-256
  `351349aa31e383fabd76a13dce188019437c9174af56d92a9824888abdba8af9`.
  Actual empty_directory_ownership child_rc0, UID/GID0:0, mode448(0700),
  refused against10001:10001/0700. This proves the failed numeric postcondition,
  not its Desktop/kernel/filesharing origin. Cleanupfailures[] is the
  controller's recorded proof, not an independent engine inventory.

Both used the old local f3aba63 Essentials image, exact template60437b5 and the
same probe image, timeout180. Each ran once. Neither reached port selection,
render/init, credentials, application volume/service, readiness/login/restart,
second-owner or missing-key assertions. No root application, origin diagnosis,
repair/fallback or retrospectively successful mechanics claim is justified.
The separately owned A09 container design/native Linux cohort remains pending;
this report neither duplicates it nor prescribes another invocation.

## Component, safety and remedy mapping

| Observable component | What remains safe or unknown | Published remedy/evidence |
| --- | --- | --- |
| redb ownership/filesystem | A second owner is refused; sharing a file/volume is not a distributed mode. Missing key refusal cannot prove an already running process stopped. | One dedicated owner/local locking storage; stop actual owner before offline access; correct private key/mount or isolated archive restore. Operations guide and actual named ownership/recovery checks. |
| HTTP/probe/PG pool or unavailable storage | Generic ready503 does not identify a remote cause. `/livez` remains independent; timed-out work retains its permit. | Finite readiness event cause/component/safe_state/remedy; check foreground/probe occupancy, PG pool/connectivity or restricted diagnostic. Both actual failover HTTP outages and named readiness checks are credited. |
| startup agreement/compatibility | Refused issuer/capability/authentication/effective-rate drift precedes startup writes. Files and peer versions are not distributed/discovered automatically. | Align recorded settings; for old agreements stop every writer, verify compatible backup and use both confirmation flags. adopt-missing is only for a reviewed absent row, not present old/conflicting policy. Existing format3 cannot be silently overwritten. |
| background/connector work | Local readiness/duty flag is not successful job or delivery evidence; separate job outcomes/owner-generation checks still apply. Nonrenewed admission lasts60s and cannot fence a paused process's later remote IO. | Inspect lane/queue/scoped controller or downstream diagnostic; restore the named authority/dependency, preserve receipt keys and ambiguous evidence; never delete admissions or force a second dispatch based on timeout alone. |
| PostgreSQL primary/restore | riAuth does not choose durability or fence a primary. A physical same-lineage rollback can retain revoked credentials. | Database operator stops/fences, promotes only under its verified durability contract; possible commit loss uses stopped-node invalidate, persistent-credential reconciliation and reviewed-ID completion before readiness/application checks. |
| TLS proxy/load balancer/external keys | A local healthy store does not verify certificate/trust/public path, external signing or restore escrow. | Explicit proxy/header/firewall/certificate owner, public discovery/application checks, externally retained keys/config/dependency inventory. These are deployment inputs, not universal new product gates. |

Current `src/api/probes.rs` maps eleven bounded failure categories without
arbitrary error text/paths. It emits at most one of each per failed episode and
one observed recovery, retaining the generic public refusal. Internal unknowns
stay unknown. The operator guidance and diagnostics already satisfy much of
the row's failure/safety/remedy purpose; a new dashboard or every-provider
campaign is not proposed.

## One smallest source-backed correction, proposed only

`docs/deployment-examples.md` lines190–191, whole blob
`a773996bd0f73e2f7591f6226a450fda1a68a0d5`,12004bytes, SHA-256
`be5b5cf733d5f015c4ee5aa59ea95551b2a02b64dc361f7a28e7c9814bb1223b`,
still says:

```text
node. PostgreSQL shares core session, replay, rate-limit and delivery state;
the forward-auth limit remains per node. Platform connector, device,
```

Propose reserving **only those two lines**, preserving every other byte:

```text
node. PostgreSQL shares core session, replay, rate-limit and delivery state,
including the `forward_auth` counter under the agreed effective rate limits. Platform connector, device,
```

No defaults, init commands, role choice, edition restrictions, tenant limitations
or protected workflow changes follow. The next existing sentence/link continues
to send readers to availability; that guide already describes redb local counts
and the stopped-node/both-confirmation upgrade procedure, including absent-row
adoption limits. The edit corrects an operational capacity misconception:
adding PG nodes does not multiply a configured forward-auth limit.

Exact source witness at all three pins: `src/api.rs`1113–1135 resolves the
semantic category value then uses `Store::shared_rate_limit` for **every PG
category**, including forward_auth via `App::run_forward`; redb uses App-local
in-memory counts. `run_forward`141–147 uses reserved forward permits.
`src/store.rs`736–770 commits the category/address/60s counter before returning.
`src/config.rs`186–204/525–546 defines/resolves all16 defaults (forward_auth6000).
`src/node_security.rs`86–103/186–223 compares effective agreement values;
`src/core.rs`216–239 enforces agreement before migration/backfill/activation.
Current availability opening and operations rate/upgrade text agree with this
source. This is a guide defect, not reopened O03 implementation.

Prospective verification requires only exact old-blob/one-occurrence checks,
in-memory reverse replacement proving all other bytes identical, published
caller/config/agreement comparison, `python3 scripts/check-docs.py`,
`git diff --check` and sole-guide/report scope. No runtime/Cargo/container test
is warranted for the sentence. **No existing-file reservation is taken or
applied here.** Root may assign it to primary e1 or another owned slice.

## Recommendation, remaining scope and actual checks here

The factual guide correction is one concrete local operational gap. Separately,
the repository still lacks a successful observation of the selected printed
small-container mechanics: both actual attempts failed before init. Existing
current Linux service/PG/contract evidence is useful and must count, but does
not execute its image/mount/healthcheck topology. Root's pending container/native
cohort can address that particular evidence gap; no duplicate runtime or broad
new campaign is requested. The distributed guide already clearly allocates
database HA/recovery responsibilities, backed by real bounded database/service
and recovery executions. It does not claim deployment-wide HA.

Therefore this pin is **not an unconditional original O07 DONE recommendation**.
After correcting the one stale sentence and assessing the already reserved
deployment evidence, root can decide the original tested-configuration outcome.
No requirement for every host, physical device, browser, tenant, release asset,
full CI rerun or recruited human study is added. A particular production
deployment still supplies its keys, trust, network, DB durability/fencing and
recovery reconciliation inputs. Those are not automatic perpetual task blockers.

Static witness checks established identical objects at published0d090169,
b619 and b5 for the deployment guide/availability, both templates, systemd,
Caddy/HAProxy/PG JSON/Dockerfile, small controller, both PG shell harnesses,
PG tests and CI workflow, API/config/node-security/store/core/probes. Their
equality transports inspected source semantics, not whole-tree/released binary
equivalence or new execution. In particular:

| File | Immutable Git blob |
| --- | --- |
| `scripts/check-deployment-small.py` | `fa77efc53aefcf167bd4f1ce8cb2f60d62bdbb74` |
| `scripts/test-postgres.sh` | `c8e3bd778f59322e02114445fdac370d6609bdd0` |
| `scripts/test-contracts-postgres.sh` | `ddad88960767ac53915023a6c251eb8b9a2d345b` |
| `tests/postgres.rs` | `fa5bca7b35f5b6256b36dcb2c438736e980ee8bf` |
| `.github/workflows/ci.yml` | `7c724fd7f4dc210a2268f5a702bdc09b1a10d76b` |
| `src/api.rs` | `a19ed8e0e875acec096ce55e665f90997a18760c` |
| `src/node_security.rs` | `da029997ad9c1a805cc3c6a1f6a50954f79a16c3` |
| `src/api/probes.rs` | `c1404fd4ada77604f74f4ddf1a5a19bafbcf712b` |

Actual local checks: read-only explicit-project row, immutable path/blob/hash
comparisons, raw-log byte/hash and named-result inspection, JSON decoding and
two failed-record size/hash reconstructions, Python AST parsing of the small
controller and its two helper literals **without execution**. Before adding
this report, `python3 scripts/check-docs.py` exited0 with
`Markdown links and build-directory layout checked`; whitespace and clean-tree
checks passed, with no preexisting checker flags. Final report validation uses
the same docs/whitespace checks, exact proposed hunk reverse comparison and
sole-new-report scope. No failed product execution or runtime correction occurred
in this review. Early oversized read displays are reading limitations only.

This report is the sole changed file. No other histories, sources, templates,
controllers, scripts, tests, guides or artifacts are changed. No new helper,
native/PG/Docker/provider/browser/service/Cargo command, desktop/network action,
worker contact, task/board mutation, merge/alignment, main edit or push occurred.
All authorization/receipt/removal/audit/credential, Group hold, shared admission,
SCIM cache/generation, header and PAM contracts remain untouched.

## Applied distributed forward-auth paragraph correction, 2026-10-02

Reservation `wave30_O07_distributed_forward_counter_correction`, project
`891e7443-8dac-4c1b-897f-9e53cb59c7ee`, original O07
`6c981199-62dd-464d-a12a-4ce4e27f428f`, existing independent support WTed9.
Root approved exactly the two-line proposal in report commit
`674af2cfcbdd92f8c72b0309866b866044d14e78`. This appendix records its application;
the full preceding 307-line, 26732-byte report remains unchanged, SHA-256
`5c1a35a6bc712e7d8d8c64bb2b6411b8ce076af58b1b3395f55e5a070cc8447b`,
blob `ecbb87f48c283a11301cdc3410753edec7d05b01`. Its proposed/unapplied wording
and pin-specific recommendation remain historical. No failed runtime receipt
or older execution limit is superseded by this documentation correction.

### Immutable applied hunk and byte preservation

Current published baseline was resolved to
**`94054b3c9b674e445893b52c1d7de29703fca73c`**. Its complete deployment guide
equaled own parent674af2cf's guide blob, so no merge/alignment/import was needed.
Applied guide-only commit:
**`35fb124b4759f22450e72ec6855af15fd134fa61`**, parent674af2cf.
It changes only [the deployment guide](../deployment-examples.md), exactly
two insertions/two deletions at lines190–191. The old/new text is precisely the
approved pair already printed above; this applied record uses the immutable
commit/hunk mapping rather than embedding another Markdown diff.

| Exact guide | Git blob | Bytes | SHA-256 |
| --- | --- | --- | --- |
| Published94054b3 and own674af2cf before correction | `a773996bd0f73e2f7591f6226a450fda1a68a0d5` | 12004 | `be5b5cf733d5f015c4ee5aa59ea95551b2a02b64dc361f7a28e7c9814bb1223b` |
| Applied35fb124 guide | `2c7c23fa4fe2a2506761fa304fe6542ffdf5dcda` | 12040 | `eaabf9409dfc7f508efd7522f29d47a194be72036e6c3c24ac5b1f66daa2cb82` |

Actual in-memory verification found exactly one old pair, replaced it with the
approved new pair, and confirmed that reversing only that pair reproduces the
complete published baseline. The following `Platform connector, device,`
sentence through EOF is byte-identical, including the availability/limitations
links. Every other guide byte, default, command, template restriction and
historical fixture limitation is preserved. The original report was unchanged
through the guide commit. Its full prefix is preserved by this separate append.

### Current published source witnesses actually read

The following published94054b3 bodies were read, not only hash-compared. Their
whole blobs also equal the earlier immutable0d090169 witnesses:

| File and read interval | Published94054b3 blob | Contract supporting the correction |
| --- | --- | --- |
| `src/api.rs`140–147/1101–1135 | `a19ed8e0e875acec096ce55e665f90997a18760c` | Outpost authorization uses category forward_auth; every PG category uses the shared counter. Forward-auth counting uses reserved forward admission; redb counting uses the App's local map. |
| `src/store.rs`736–770 | `8e78d889220c9e956fe00a4f7a69b603063d3ebf` | Address/category counter is committed in a bounded writer under a 60-second window before returning. |
| `src/config.rs`186–204/525–547 | `0fc0b9a530440c66555eae9d9ee976d1c03549c3` | Same semantic resolver supplies all sixteen effective defaults/overrides, including forward_auth6000; omission equals the explicit default. |
| `src/node_security.rs`86–103/184–223 | `da029997ad9c1a805cc3c6a1f6a50954f79a16c3` | Recorded agreement compares effective rates, issuer, active capabilities and authentication policy; mismatch retains the recorded policy and supplies category/action diagnostics. |
| `src/core.rs`216–239 | `f02efcde5e9604b7251a8171e0aa437dbd8d8ae2` | Agreement enforcement precedes startup migration/backfill/activation writes. |
| `docs/availability.md` opening paragraph | `c557be3aa89a6ceaa2290f46b8bd2c2f5fc45378` | PG shares forward_auth; adding nodes does not raise the configured shared limit; redb counts in its owning process. |

These source reads and identity checks add no new execution. They justify the
guide's shared-counter wording under agreed effective limits. No production
API/store/config/agreement behavior, stopped-node adoption/rollback procedure,
writer, permission, receipt or admission contract changed.

### Actual static checks and remaining original scope

Before the guide commit, exact proposal/reversal/whole-blob checks passed;
`git diff --cached --name-only` contained only the guide, and the reviewed diff
had precisely the approved two lines. `git diff --cached --check` and committed
`git show --format= --check` passed. Actual commands:

- `python3 scripts/check-docs.py`: exit0,
  `Markdown links and build-directory layout checked`.
- `python3 scripts/check-repo-hygiene.py`: exit0,
  `Tracked-file hygiene checked (943 files)`.

Neither checker had preexisting flags, and no checker/cache cleanup occurred.
The guide commit left a clean worktree. Final append checks verify the full
original report byte-prefix, immutable applied-guide/source pins, only-report
append scope, guide preservation, docs/hygiene and staged/committed whitespace.
No failed static check or correction occurred in this slice.

The local guide misconception identified by the independent audit is corrected.
**O07 remains open pending root's assessment of the separately owned Sol6
container cohort.** The two failed small-container receipts remain failed at
their exact recorded sizes/hashes/classification limits; all historical
PG/R05/b5/b619 command/count/source boundaries above remain intact. This paragraph
does not create a runtime pass, close the original tested-configuration outcome,
or imply deployed HA, renewed admission or paused-process external-IO exclusion.
No additional host/provider/browser/release campaign is proposed.

Only the guide-only commit and this separate report append are produced. No
other paragraph/index/template/helper/script/product/test/artifact or report
changed; no runtime/Cargo/Docker/native/library/provider/browser/network/desktop
action, slot, worker contact, new task/worktree/shell, status/board change,
merge/alignment, main edit or push occurred. Closed I02/I10/D03/D04/O06/R05/W02/W05
and all prior security contracts remain preserved. Root alone reviews,
integrates, publishes and decides original O07 status.
