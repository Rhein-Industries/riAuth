# Q09 whole-deployment measurement: bounded source plan

2026-10-03. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original Q09
`58a9602e-2449-42cf-82bc-635806c92176`. Reservation
`wave30_Q09_deployment_measurement_source_plan`, existing supporting WT
`a1303b57-4a34-487e-9c63-a841f05b51a0`, shellb471. Inspection pin:
`544d1340b80cd3e040dc13142cdcbc1d75fea4cb`. Own clean entry:
`c12b38e2b0ecee72e8db7da07bb82f06af795db0`; no alignment or merge.

**Q09 is not a DONE candidate on this evidence.** S02 establishes a bounded
raw-fetch materialization improvement with slower measured latency. Existing
Q09 observations establish named HTTP workloads, serving-process CPU/RSS,
success/error counts and scheduled-pass overlap. They do not measure complete
deployment IO, and PostgreSQL resource costs are outside their PID sampler.
The smallest proposed next slice is an opt-in Linux owned-deployment resource
sampler around the existing secure redb workload, for one matched Essentials/
Platform artifact pair. Source materialization and runtime remain separately
held. This report implements no sampler and supplies no new measurements.

## Original row and completion boundary

Read the complete exact Q09 row from project `planning/current-tasks.json`:
244,354B, SHA-256
`0b38ca3eb8810a4628436eb5083f6329de848f3f4ddaa972c850601ad45c0835`.
It is `[P2] Q09 — Publish reproducible benchmarks`, exported status `todo`,
original primary WT `eec3b31b-c4ca-4be5-882c-82c349b1c69e`, updated1790894719.
This is a local export observation, not a live board query. The older
`delegation-current-tasks.json` export,244,599B/SHA-256
`6b85e2ead36eca22533ad1ddbb24f6388c630c24bb37c686e6aa08b3e118c8c8`,
has identical acceptance wording and an earlier `in_progress` observation.
No status reconciliation or assignment change is made here.

The complete outcome is:

> Compare equivalent functionality and security settings; measure whole-deployment cost, latency, successful throughput, errors, and background interference.

Prerequisites are S01, S02, S03, O05 and Q08. The workstream continues through
the features; its gate requires appropriate, separate evidence for security,
compatibility, speed and recovery. Passing one category establishes no other.
Both editions retain shared identity, authorization, revocation and credential
protections. Completion requires applicable implementation, tests, docs and
artifact evidence with actual checks and external prerequisites, rather than
a report alone. Historical no-launch scheduling does not enlarge this current
explicit report-only reservation. Original primary and all closed rows remain.

## What S02 actually measured and accepted

Read the complete [root review](https://github.com/Rhein-Industries/riAuth/blob/544d1340b80cd3e040dc13142cdcbc1d75fea4cb/docs/roadmap/local-wave30-s02-root-review.md), complete
[independent actual review](https://github.com/Rhein-Industries/riAuth/blob/544d1340b80cd3e040dc13142cdcbc1d75fea4cb/docs/roadmap/local-wave30-s02-independent-actual-scope-review.md)
at `472a2eb481fc710de0702c7169e7f93665af3484`, and the complete actual-result
appendix at lines5364–5641 of
[S02's report](https://github.com/Rhein-Industries/riAuth/blob/544d1340b80cd3e040dc13142cdcbc1d75fea4cb/docs/roadmap/local-wave30-s02-index-pagination-plan.md). Also read complete
project receipts `wave30-s02-lint-corrected-measurements-root-review.json`,
`wave30-s02-lint-corrected-runtime-cleanup-root-review.json` and
`wave30-s02-original-status-root-disposition.json`.

The actual run HEAD was `536ab811c3cf396df5ceae6509e109164cec9b2c`, authored
fixture `ec4316259b3cb04496e5467336ecebee15b92f2c`: one selected ignored
test, Cargo0/libtest1pass,139 records,120 samples and12 summaries. This was
default-Platform, macOS ARM64, test-support, native encrypted redb, comparing
the complete frozen old Group listing against the accepted paging method on
identical seed copies. The largest raw fetch fell513→128; all513 rows and
native stored-value bytes remained. Five fetches replaced one. Zero measured
commits/writer holds, complete results and state, public refusal/writer/
revocation and ordered-snapshot/eight-call overlap assertions passed with
their recorded limits. All owned threads and fixture directories were joined/
removed; root also verified Cargo leader/group cleanup before release.

| Pair/scope | Control p50 ms | Paged p50 ms | Paged increase |
| --- | ---: | ---: | ---: |
| 0/scoped | 40.989792 | 43.016042 | 4.943304% |
| 0/admin | 45.120750 | 47.825917 | 5.995395% |
| 1/scoped | 41.275125 | 42.980917 | 4.132736% |
| 1/admin | 45.819458 | 47.776625 | 4.271476% |
| 2/scoped | 41.291042 | 43.030750 | 4.213282% |
| 2/admin | 45.984250 | 47.733167 | 3.803296% |

All six p95 comparisons were also slower. Root accepted original S02's
bounded materialization outcome and security/concurrency gate, publishing
544d and recording DONE. This supplies no heap/RSS/IO/throughput, PG/Linux/
release/full-CI or latency-improvement evidence. Historical type and CI lint
failures remain failed. Canonical stored expiry was tested, not elapsed
natural TTL; legacy enabled-child/disabled-parent and between-actual-page
injection were not claimed.

The whole-file identities read are root review18,405B/SHA
`4fdc1500c7044a678e830efd1843c0297bae93f1f78402db8bfeda8628c347e9`,
independent review24,383B/SHA
`7311e103de369062b6f4989caa975bbf34f6b695c1e2c212afa2a2d404679607`.
Measurement receipt4,380B/SHA
`84e8899a3dc1df556b47dfe7309ecc1adff097ee9f02ba6a13963d96e13a0386`;
cleanup receipt1,425B/SHA
`1c0368200524cb6e12d559739a0c56d247b8bc8b33908fd92f495e74e09d5887`;
disposition698B/SHA
`a4a106892b284bad61a5e5e1e3b6bb52067908b69da6316fe47af3eaa47ed112`.
These are accepted historical observations, not re-executed checks.

## Existing HTTP/load/deployment evidence

Read [Q09's published protocol/evidence](q09-benchmark-slice.md), its named
historical profile sections and remaining gate, plus all28 local project Q09
JSON reports in `planning/` and `planning/evidence/`. Parsed complete JSON
data and projected source/artifact/profile, attempts/success/errors, latency,
throughput, CPU/RSS and overlap. Read the four named secure JSON reports under
the recorded `q09-linux-arm64-secure/reports` location and verified their
whole hashes against the published page. No old executable, secret file,
protocol capture or Grok task was opened or invoked.

| Historical evidence | Actual measurement credit | Limit relevant to Q09 |
| --- | --- | --- |
| Four fresh-init5892563 JSON reports; four db533e7 directory reports | Both editions/backends;80 or40 reads per quiet/interference pass, all session reads successful; named dev binaries, CPU/RSS and nearest-rank latency | Short windows; no maintenance-cadence proof, deployment IO or PG process accounting |
| Eight `q09-8u-8g`47aa248/accepted7aea083 JSON reports |40+40 successful reads per report; exact nine-account/eight-group directory; dated binary identities | Unencrypted macOS dev profile; too short for cadence and only2–4 PID samples per pass; not a capacity claim |
| Four paced macOS reports: Essentials6ca4779, Platformc7a61c6 |12+12 successful reads; maintenance finished+1 in both passes;4 writer posts,2 successes,2 conflicts in each report | HTTP/plain storage; loaded host; dev/test-support identities differ; throughput includes pacing |
| Four Linux ARM64 plaintext maintenance reports at harnesscc186af | Both editions/backends, release copies from6ca4779; same successful paced workload and cadence | Docker linuxkit VM; CPU samples quantized to whole seconds; no release package or full deployment costs |
| Four secure Linux ARM64 reports at harness4ab934a | HTTPS matching-CA success and unrelated/system-CA refusals; encrypted records; PG SCRAM/hostssl and wrong-CA/no-TLS refusals; all paced reads successful | Same dated6ca4779 files, local VM/private CA/local signer; sampler still only serving PID; no IO |
| O07 small-template actual original and diagnostic attempts | Preserved failures and cleanup; image provenance checked | Neither reached application startup; diagnostic postcondition UID/GID0/0 versus required10001/10001; no deployed benchmark |

Concrete secure historical figures follow. Every pass has12 attempts/12
successes/0 read errors, cadence overlap true and nearest-rank p95=p99=max.

| Edition/backend | Quiet/interference p50 µs | Quiet/interference p95 µs | Quiet/interference success/s | JSON SHA-256 |
| --- | --- | --- | --- | --- |
| Essentials/redb |4491 /4205 |11331 /9416 |0.167617 /0.167647 |`cc8c417c1e965e0caee60e8b9b9aa42b25186c5816d6f2644d2d09abba8ef3f2` |
| Essentials/PG |9683 /9125 |14087 /11152 |0.167519 /0.167536 |`f6fd102c82108288299b655932dbaff8d9d80b6db26a148ac01433468a3157dd` |
| Platform/redb |5965 /3746 |10922 /6696 |0.167600 /0.167658 |`e88bba86b5a11a5c68bd6235b833461198d52f474de7092fda3c2bc55351be6a` |
| Platform/PG |10462 /9996 |19071 /11876 |0.167464 /0.167503 |`b8946dec02ebb45aa06a9e7dd7b60c770117621a3c614d2f5d62fa1acfada961` |

The secure report sizes are13,149/14,373/13,151/14,404B respectively.
Measurement script hash equals today's399bd2, but product build pin is
`6ca4779b68cf41e29af490d2aca6ea3d59e1f2bd`, not544d or current S02.
Essentials server36,324,648B/SHA
`5feac36b7ad5d292cafb192b673f4353f8fa424e90a35d667d8fa7f41cac74eb`;
Platform49,302,784B/SHA
`606a2ce5d72de16fe8749efdb3504fd81471c88118008180667a013ccffea79d`.
The report records supplied release/1.98.1 feature labels; the published
Q08 build-source evidence supplies their dated provenance, not a new build.
Do not compare these timings to S02's different513-Group complete Core call
as if operation/directory/artifact/security/environment were equivalent.

For a concrete dev comparison, Platform redb's paced report12,401B/SHA
`5dee2c961051225758e6b77d24ee74b14eff94f9913d6cd1b580b7c367391a85`
at `a4b7a7947b452731a33cd77196c5594d5fce1b3a` observed3350/3419µs p50,
0.167634/0.167628 success/s,0.54/0.53 CPU seconds and74,064/74,320KiB
maximum serve RSS. Its PostgreSQL sibling12,410B/SHA
`b323aad0c7503ba7cc9f8e5a075d4cb965f55256bf8c94e896c62d23ebf99f86`
observed6592/5790µs p50 and0.84/0.91 serve CPU seconds. The PG daemon,
connections and storage IO were not included in those CPU numbers.

Read the complete deployment helper and both failure/cleanup appendices in
[O07's report](local-wave30-o07-deployment-plan.md). The first1745B evidence
SHA is `308dfe8d3b424ff9a4e54369d809f560847b6aefda43a83ca229842b34068adc`;
the diagnostic1878B evidence SHA is
`351349aa31e383fabd76a13dce188019437c9174af56d92a9824888abdba8af9`.
The first cause was underclassified; the second recorded the failed ownership
postcondition. Neither is retroactively passed. Later A09c12/x86 cohort
success is its own dated artifact/refusal/recovery evidence, with no measured
deployment CPU/IO/throughput comparison. It supplies no new ARM result.

## Source witnesses and exact missing measurement

These witnesses use fixed544d Git objects, not stale imports into this WT.
Whole-file hash comparison is distinct from selected-body review.

| Source | Complete identity / reviewed bodies |
| --- | --- |
| [Q09 helper](../../scripts/q09_benchmark_slice.py) |87,645B, SHA`399bd242519bdae15483f2d450ef9c1bf7b8b397a25bfff1179abefcaf9c7e49`; budget229–292, security796–877/1146–1248, sampler406–514, HTTP517–577, counters580–622, start/stop637–687, complete measured_pass/group_writer1251–1366, setup1375–1517, run_slice1529–1772 and main2016–2134 |
| [Deployment helper](../../scripts/check-deployment-small.py) |39,920B, SHA`7e6e2265dad539356a4566131d62dd7904e19df7bba451d4e44588a23c77837d`; complete737 lines, including owned image/source/render checks and cleanup; a mechanics/refusal harness, not a resource-cost sampler |
| [Core](../../src/core.rs) |57,913B, SHA`686e7e732f256e7b435ab69991b71fb26d7467214877d2e0f0c840741446caea`; complete me body uses session/principal and groups_for, not S02's list_groups call |
| [HTTP observations](../../src/api/observability.rs) |17,023B, SHA`3f208782e1d31738581c1c6cf3bbfc42291d0ff5e1df32b4f872ae73cb1cae48`; complete counters/admission/observe/metrics/Prometheus bodies read |
| [API](../../src/api.rs) |147,317B, SHA`1bbb387dd627f1fd56b7dfe408ea17eef48131f927597da1b81b21ed5f64b8ca`; App/admit/blocking49–216 and rate category/authorization middleware1030–1160 |
| [Store](../../src/store.rs) |82,583B, SHA`dfb62f2e482e9e334e148a932c7900607f37297748f0c1f90a8ef65f77babc0f`; allocation cache constants272–274 and full cached_allocation/spawn542–654 |
| [Telemetry](../../src/telemetry.rs) |24,046B, SHA`d23e9a03f2129601a1a37ace5ad80b0473bbc63a6ddb47be75728882fb0e12fd`; Reads275–337 and full Telemetry/snapshot425–521 |
| [Background](../../src/background.rs) |60,309B, SHA`09e550db452b9c8f2d3cf2ad9f87af82ee430e51ed21adf3c6f62f72ed9d0aa8`; lanes/cadences25–100 and complete execute/spawn294–398 |

`read_ps(pid)` obtains only serve RSS and cumulative CPU time. `measured_pass`
passes exactly `server.pid`; no PG PID, process cohort, kernel IO or deployment
cache-charge counter is captured. `ProcessSampler.stop` uses a timed join;
legacy cleanup ignores directory-removal errors. These definitions are not
the stronger future joined/absence proof required below. No production defect
is inferred from this report-only source inspection.

Existing latency includes HTTP/TLS exchange and body read, while percentiles
include every attempt and successful throughput divides successful reads by
whole pass time. Twelve reads with eleven6500ms sleeps spend71.5s in pacing;
this is offered-load success throughput, not maximum capacity. Writer409s are
reported separately and appear in server errors; warmups/setup and closing
metrics alter global counters. Counter deltas explicitly include the closing
metrics request. They must not be relabeled as read errors or discarded.

Metrics authenticate with live operations.read before allocation cache lookup;
operations/storage has its own permission. Allocation refresh is at most one
background thread per Store, TTL30s/max-stale300s, and unavailable remains
unavailable. Allocated file/relation bytes, logical returned-value bytes and
disk-free movement are not IO. Allocation excludes WAL/backups and is not
physical capacity. Preserve the cache; record existing availability/freshness
if retained from the same metrics response, without new refresh/scrape traffic.

Admission remains8 workers/4 credential/16 forward permits, two-second wait,
and password permits survive disconnect. Background deadlines remain60s,
capacities2/2/1/1 and original cadences; maintenance60s is not shortened.
Shared HTTP counters may write before authorization in PG; redb counters are
in memory. Failure floor, Argon2 hashing, stored expiry, format-3 security
agreement, live authority and header/receipt behavior must stay active.
The S01 characterization script/selected test metadata and testing115–129
describe an in-process release workload with back-to-back background passes;
that differs from deployed cadences and carries no new actual benchmark credit.

## ONE proposed seam: secure local redb deployment resource accounting

Prospective ownership request, not implementation: isolated additions in
`scripts/q09_benchmark_slice.py` at ProcessSampler/measured_pass/run_slice/main
and one focused addition in `tests/test_q09_benchmark_slice.py`. No production,
configuration-contract, schema, writer, feature, deployment-template or workflow
change. Retain the legacy mode/schema and its historical results exactly.

Proposed opt-in interface: `--deployment-sampler linux-cgroup-v2` plus an
explicit fresh delegated leaf under a root-reviewed owner path. Restrict this
first mode to real product `--secure --backend redb`; reject fixture mode,
PG, remote services and unsupported hosts. One leaf contains only the owned
serving process and its descendants, including its native TLS/redb/background
threads. Native TLS/storage/local signing are in-process; no proxy, database,
external signer, provider or other hidden deployment component is configured.
The observer/load generator remains outside the leaf and its own cost is
recorded separately, never subtracted as an assumed zero. A future PG profile
would require postmaster/WAL/all child processes in the owned deployment scope;
this proposal does not approximate it using serve PID data.

New sampler responsibilities are bounded, passive reads of an owned leaf's
`cpu.stat`, `memory.current`, `memory.stat`, `memory.events`, `io.stat` and
membership/population identity, with monotonic clocks and complete retained
samples. Preserve the existing request timer and sample window. Record CPU
usage delta at microsecond resolution, sampled charged-memory maximum/start/
end and file/anonymous breakdown, charged block read/write bytes and operation
counts by privately bound device, and event deltas. These are kernel-charged
deployment metrics, not heap, summed RSS, device-global IO, power/currency cost
or a hard unsampled maximum. File-cache hits can legitimately yield zero
charged reads; missing files/controllers/fields or failed parsing yield
unavailable/refusal, never fabricated zero. Device names/paths remain private.
Do not force fsync, drop caches, change scheduler/IO priority or disable duties.

Before first measured request, prove leaf ownership/absence of foreign tasks,
move the exact owned serving process into it and verify task membership and
PID/start identity. No unrelated PID adoption or parent cgroup manipulation.
Use counter baselines after placement/setup, then start/end and finite samples
for each complete quiet/interference window. Membership loss, identity change,
counter regression, ownership mismatch, unknown member or missing accounting
must refuse the cost grade. Cgroup attachment/kernel charging details and
launch/cleanup bodies require exact source review before materialization;
the filesystem permission/controller prerequisite is unmeasured here.

### Closed workload and equivalent profile

One future serialized pair only: Essentials then Platform, same native Linux
host/architecture, source pin544d (or one newly root-reviewed identical selected
product tree), release profile, locked dependencies, essentials versus platform
features without test-support/fuzzing. Each artifact gets a fresh secure redb
fixture with the exact existing `q09-2u-2g` operation: three accounts, two
groups, six memberships;12 measured GET/api/me attempts per quiet/interference
pass, one warmup,6500ms pace, at most four existing empty-group writer posts.
That is four windows/48 measured attempts in the pair. Existing per-cell
general budget65 remains; the sampler adds no HTTP/SQL/provider traffic.
Normal public revision/idempotency headers and authority are preserved.

Common normalized profile must match before comparing: verified membership
oracle/response functionality, native HTTPS SAN/CA checks, AES256GCM32-byte key
and0600 file protection, normal password-hash parameters, token lifetimes,
authentication/rate agreement, default600 general rate, admission/background
budgets and cadence, local signer, no upstream/proxy/connector targets and
identical storage/mount/CPU/memory/IO limits. Private generated keys/tokens,
issuer port and directory identities differ; security strength does not.
No benchmark option disables a guard or changes the accepted agreement.
Retain matching-CA success/unrelated/system-trust refusal and exact dataset
validation; count every read error/429/503, writer success/conflict/error,
maintenance finish/failure and admission rejection. Unexpected writer errors
also prevent a successful comparison. No rerun discards a slower observation.

Keep all latency samples and descriptive p50/p95/p99; for n12 p95/p99 remain
the maximum. Keep pacing inside throughput's denominator and label the offered
load. Require each window to exceed60s and observe maintenance finish, with
actual background/queue counts from existing metrics captures. Concurrent
writer success inside a pass and duties finishing inside it establish that
limited overlap, not causal isolation of per-duty cost or busy remote queues.
Compare whole-window CPU/charged memory/IO and latency/error counts, without
promising speedup. This common session-read deployment cannot establish S02
Group-list speed, application login throughput or arbitrary provider capacity.

### Exact prerequisites and future validation boundary

The two matching544d release artifacts are **not identified in the inspected
measurement evidence**. No fresh build or artifact search ran. Old6ca4779 Q09
binaries, A09's datedb619 cohort files, and S02's macOS test-support fixture
are not substitutions. Root must select and independently bind exact artifact
SHA/size/architecture/edition/capabilities to the complete product `src`/
`crates` tree, Cargo.toml/Cargo.lock/toolchain/build settings and actual locked
build receipt. Harness hash/execution HEAD stays distinct from binary source.
Supplied `--build-*` labels alone are inadequate provenance. Pin any future
builder before a separate release; this report authorizes no Cargo.

Root also needs one named owned native Linux host with delegated cgroup-v2
CPU/memory/IO accounting, no foreign tasks in selected leaves, readable finite
counter files, compatible filesystem/device attribution and a closed owned
resource budget. Record actual kernel/CPU count/model/RAM/storage topology/
cgroup limits, host load and concurrent-work condition. A Docker VM must stay
labeled a VM. The current WT's environment is not proof of this prerequisite;
no host/resource/version/process probe or ARM live-job query ran here.

One focused future stdlib validation target, after exact source reservation:
`python3 -m unittest discover -s tests -p test_q09_benchmark_slice.py -k DeploymentSampler` with
synthetic finite counter/membership files and no product/HTTP/cgroup mutation.
The class is prospective and does not exist yet. Cases must independently
check CPU/memory/IO deltas, valid zero IO versus unavailable, malformed/duplicate/
oversize/nonbool values, counter regression, foreign membership/PID reuse,
exact pre/post windows, capped capture, no secret projection and joined
observer failure. No invented case count or pass is credited. Native leaf
placement/accounting/cleanup needs a distinct reviewed actual measurement,
not these synthetic parser assertions.

Future runtime needs one exact fully reviewed launcher/pair argv and artifact
pins before release, exclusive0700 fixture/0600 no-follow evidence, finite
captured raw output saved before grading, no inherited proxy/secret overrides,
no redirects/network beyond own loopback, bounded response/input/output and
owned subprocess/session identity. Proposed outer bound720s including60s
cleanup reserve, per-HTTP maximum5s for timed reads and existing bounded setup
operations; sample cost counters at100ms and disk at2s with explicit maximum
gap. Require a fresh11GiB planning start, own stop9GiB before mandatory8GiB
floor, and no build/download/cache deletion inside the measurement.

Every exit must stop only owned children, join observer/writer/drain threads,
consume real numeric child statuses and verify group/leaf emptiness before
removing only the owned leaf/fixture. Existing timed joins and
`rmtree(ignore_errors=True)` are insufficient as the new grade's cleanup proof;
the opt-in mode must retain checked cleanup outcomes and refuse timeout/failure.
Save complete private outcome/elapsed/exit/counts/hashes before expectations,
with only fixed redacted categories and numeric resource/count public fields.
No raw arguments/environment/config/URI/session/token/key/row or arbitrary
exception message enters public evidence. No automatic repeat on refusal.

## Disposition, limits and actual checks

Recommend root reserve the sampler's exact source design/materialization only
after agreeing the native host and artifact prerequisite. This addresses one
concrete Q09 gap, not the entire publication gate. Representative directory,
network distance, loaded queues, PG/WAL, proxy/upstream/external signing,
released package/host inputs are required only for the deployment being claimed.
No invented tenant, all-platform or universal feature campaign is proposed.
Whole-Q09 completion remains root-owned and unsupported by this report alone.

Fixed authority, reviewed creation receipt-secret behavior, route-specific
optional/required headers, PAM fallback and held authoritative-Group designs
remain untouched. Shared connector admission is still nonrenewed60s, with no
atomic paused-before-IO fence; this local empty-target profile tests no repair
or general remote-delivery guarantee. Recovery/compatibility evidence is not
converted into performance evidence. I02/I10/R05/W02/W05 and original A09/D01
dispositions receive no new claim. Driver-only preference remains for any
future separately authorized desktop; none was used.

Actual work here: original export/body and guidance reads; immutable Git source
and complete accepted S02 actual/independent/root disposition reads; named
historical JSON data/hash/profile/count projections; selected metrics/admission/
cache/security bodies and complete deployment helper review; static AST parsing
and source identities; report links/hygiene/whitespace/sole-path scope checks.
Some combined tool displays truncated; relevant selected bodies/records were
reread in bounded outputs. The first docs check failed on three relative S02
links because those accepted reports are absent from this deliberately
unaligned WT. Links were changed only in this new report to the exact544d
published objects; no missing report was imported. No source/test/runtime failure was discovered by
execution because none was attempted. No reviewed function/helper/module,
benchmark, test, compiler/native/version, service, process signal, HTTP/SQL/
provider/network, browser/Driver, query/download, worker contact or managed
resource was used. Only this new report is written and committed. No lane was
acquired or released; ARM37101183416 remains outside this audit. Root alone
owns future reservation, integration, publication and task status.
