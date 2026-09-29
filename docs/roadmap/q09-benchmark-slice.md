# Q09 session-read benchmark slice

This is the first published measurement protocol for one equivalent local
operation. It does not close Q09. The numbers a run prints are observations of
that run. They are not a load target, a recovery target, or a comparison
between editions or backends unless those runs were actually recorded.

## Operation

The operation is `GET /api/me`, the HTTP form of `Core::me`. The handler takes
one worker permit and reads the bearer session, the user, and that user's
group index. Essentials and Platform share this Core path. redb and PostgreSQL
are the two store backends behind it. One invocation of the script measures
one binary and one backend.

The default dataset is a fresh `init`: the bootstrap administrator, no extra
users, and no group memberships. `--directory-users` and `--directory-groups`
default to zero. `--directory-users N` creates N accounts in addition to that
administrator. When either flag is greater than zero, the script creates that
many extra users and groups, then adds the administrator and every extra user
to each of those groups. `GET /api/me` walks the administrator's per-user
group index (`index_user_groups`), so those group names are the session-read
dataset. Each flag is at most 32, and the general-request estimate for setup
plus measurement must stay at or below 500. That cap is inside one 128-row
index page. The script reads the administrator, the user list, and the group
list before the timed passes and stops if they disagree with the directory it
created.

Database encryption is off. The listener is loopback HTTP without TLS.
PostgreSQL, when selected, is a disposable loopback cluster with trust
authentication, `sslmode=disable`, `local_unencrypted`, and pool size 8.

A quiet pass sends the session read from one client. An interference pass
repeats it while a second client sends `POST /api/groups` for empty groups,
with `Idempotency-Key` and `If-Match` set from `GET /api/state/revision`.
Empty groups do not add memberships, so they leave the administrator's
session-read index as it was. On a fresh init that index is empty. On a
directory dataset it is the verified directory groups. The writer is there to
overlap the storage writer. Server
background job counters are read from `/api/operations/metrics` around each
pass. Those counters are a separate observation from the group-create client.
The maintenance job cadence is 60 seconds and the logout/SSF delivery cadence
is 2 seconds, so a short run may show only the jobs that actually finished.

## What this script records

Each report uses schema `riauth.benchmark-slice/v1` and sets
`observations_only` and `performance_claim: false`. It records:

- source commit, dirty paths for the build inputs and this script, and the checkout's Rust toolchain pin
- artifact path, byte size, and SHA-256
- capability edition, build features, package version, and target, plus the running instance's edition and storage backend
- hardware description and host load average
- iteration count, warmup, interference cap, directory size, and the estimated general-rate-limit budget
- the dataset: fresh init, or the verified users, groups, memberships, and administrator session-read groups
- for each pass: attempts, successes, error statuses and error codes, nearest-rank p50/p95/p99 latency over every attempt, successful throughput, `ps` RSS/CPU samples of the serve process, server counter deltas, host load, the pace, and whether the pass overlapped the maintenance cadence
- writer successes, conflicts, and errors, and the arithmetic difference of the latency percentiles

The general category allows 600 requests per minute from one address. `GET /api/me`, metrics, revision, and group creation share that category. The script refuses a shape whose estimate is above 500. Login is the separate `login` category. `/readyz` is a probe.

Exit 0 means every measured read succeeded, at least one group create succeeded, and no measured attempt returned 429. Exit 2 means the report was written and the writer completed no group. Exit 3 means a measured attempt returned 429. Exit 4 means some other measured error. Exit 5 means the report was written, a measured pass lasted longer than 60 seconds, and the maintenance `finished` counter did not increase during that pass. Exit 1 means the run stopped before a report.

`--pace-ms` sleeps that many milliseconds between measured reads in both passes. Zero leaves the reads back to back. The sleep is outside each latency sample and inside the pass wall clock, so throughput falls while the latency sample stays the HTTP exchange. The sleep does not add a general request. A pass sets `maintenance_cadence_overlap` only when its wall clock is longer than 60 seconds and the maintenance `finished` counter increases during the pass. The maintenance job uses a 60-second interval and its first tick is immediate, so a pass has to be longer than that interval to contain a later finish.

## Reproduce

Check the script without a riAuth binary:

```sh
python3 scripts/q09_benchmark_slice.py --self-check
python3 -m unittest discover -s tests -p 'test_q09_benchmark_slice.py' -v
```

`--self-check` starts a fixture HTTP server. Its report is not a riAuth
measurement, and the command does not print timings.

A product run needs a binary built from this checkout. Use a private Cargo
target and leave incremental compilation off. Do not point `CARGO_TARGET_DIR`
at another worktree, the integration-accepted checkout, or the main checkout.

```sh
export CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR="$HOME/.cache/riauth-cargo/q09-benchmarks-wave20"
cargo build --locked --no-default-features --features essentials --bin riauth
python3 scripts/q09_benchmark_slice.py \
  --binary "$CARGO_TARGET_DIR/debug/riauth" \
  --backend redb \
  --out /tmp/q09-session-read.json
```

Add `--directory-users N --directory-groups G` for a verified larger directory.
The default remains one administrator and no memberships. `N` is extra users
in addition to that administrator, and `G` is groups. The report names that
directory `q09-Nu-Gg`. `q09-2u-2g` is the administrator plus two extra users
(three accounts) and two groups. `q09-8u-8g` is the administrator plus eight
extra users (nine accounts) and eight groups. Add `--pace-ms 6500` when the
measured pass must outlast the 60-second maintenance cadence. Twelve measured
reads at that pace sleep for 71.5 seconds between attempts.
`--build-profile`, `--build-toolchain`, and `--build-features` are recorded as
supplied. The script still hashes the measured file and does not infer the
compiler from it.

Repeat with `--features platform` and, separately, `--backend postgresql` when
`initdb`, `pg_ctl`, and `createdb` are installed. Keep the JSON from each run.
Compare runs only when the commit, security settings, dataset, and hardware
notes say they are the same measurement.

`--binary` may be relative to the current directory, such as
`target/debug/riauth`. The script resolves that path to an absolute file
before capabilities, init, or serve. init and serve use a disposable directory
as their working directory, and the report records the absolute artifact path.

## Evidence for this slice

The fixture check shipped with the script is `--self-check` and
`tests/test_q09_benchmark_slice.py`. The fixture proves the script's
percentiles, error accounting, redaction, RSS/CPU sampling, background-counter
delta, and group-writer overlap. It also builds a `q09-2u-2g` directory:
two extra users in addition to the bootstrap administrator (three accounts)
and two groups. It requires `Idempotency-Key` and `If-Match` on those writes,
and checks the administrator's session-read groups, the user list, and each
group's members. Password values used for the extra users stay out of the
report. A product run is what meets the real server's revision bumps and
password hashing.

Accepted commit `5892563` follows the benchmark script through `e3b3d59`.
Four small macOS product observations from that same head were reported for
Essentials and Platform, each on redb and PostgreSQL: 80 quiet reads and 80
interference reads, with zero read errors. Those JSON reports are not in this
worktree. Their latency, throughput, and RSS stay in those reports.

Accepted commit `db533e7` follows the benchmark script through `c992815`.
Four same-head `q09-2u-2g` product runs at that commit were reported as
verified for Essentials and Platform, each on redb and PostgreSQL. That name
is two extra users in addition to the administrator (three accounts) and two
groups. The reported result was 40 quiet reads and 40 interference reads,
zero read errors, and writer overlap. Those JSON reports are not in this
worktree. Their latency, throughput, and RSS stay in those reports.

### `q09-8u-8g` on macOS, dev profile

These four product observations were taken from commit
`47aa248ce1c68284773746bbb5fd59c6098afdea` on 2026-09-29. The measured script
is `scripts/q09_benchmark_slice.py`, SHA-256
`515d0ed39887a9180f3f24d36d56b11d28da4dde776f8dcaa0d75c6bb1a01b4e`. Each report
has `source.commit` equal to that commit, `source.dirty_paths` empty,
`product_run: true`, `observations_only: true`, and `performance_claim: false`.
The docs change that cites them does not change the script.

The private target was
`/Users/dominik/.cache/riauth-cargo/q09-8x8-47aa248`, with
`CARGO_INCREMENTAL=0` and `CARGO_BUILD_JOBS=6`. The commands were
`cargo build --locked --no-default-features --features essentials --bin riauth`
and the same command with `--features platform`. Cargo printed
`Finished dev profile [unoptimized + debuginfo]` after 1m 21s for Essentials
and after 53.58s for Platform. `rustc -vV` was 1.98.1
(`48a229ceaefd4985c50990b14116b6d856af0985`, 2026-09-01), host
`aarch64-apple-darwin`, LLVM 22.1.8. Each report's `build.profile` is the
supplied token `debug`, `build.toolchain` is `1.98.1`, and `build.features`
is `essentials` or `platform`. The script records those tokens as supplied and
hashes the measured file.

Each binary was copied out of `debug/riauth` before the next feature build:

| Edition | Path | Bytes | SHA-256 |
| --- | --- | --- | --- |
| Essentials | `/Users/dominik/.cache/riauth-cargo/q09-8x8-47aa248/kept/essentials-debug/riauth` | 202086936 | `e36a139ae67559dd221ca28798999e8ad55e0ab7868a573f363e44371aac5d25` |
| Platform | `/Users/dominik/.cache/riauth-cargo/q09-8x8-47aa248/kept/platform-debug/riauth` | 285351464 | `9df248426ab9fedc04bfdcc0fe340eb85d767dd3ba417274f917a7e89f783a9d` |

Capabilities schema was `riauth.capabilities/v2`, package version `0.1.1`,
target `macos` / `aarch64`. Runtime scope was `instance`. Platform's recorded
build features were `essentials` and `platform`.

The shape was `--iterations 40 --warmup 5 --interference-cap 30
--directory-users 8 --directory-groups 8`. The dataset name is `q09-8u-8g`:
8 extra users in addition to the bootstrap administrator (nine accounts),
8 groups, 72 memberships, and 9 members in each group. The
administrator session read listed `q09-dir-0001` through `q09-dir-0008`, and
`dataset.verified` is true. The estimated general-request budget is 337, under
the 500 cap. TLS was off and database encryption was off. The PostgreSQL runs
used one disposable loopback cluster, trust authentication, `sslmode=disable`,
`local_unencrypted`, and pool size 8.

The host was an Apple M4 Max with 16 logical CPUs, 68719476736 bytes of
memory, macOS 26.2, Darwin 25.2.0 arm64, and Python 3.14.6. The four runs
started at 2026-09-29T05:19:59Z, 05:20:05Z, 05:20:11Z, and 05:20:17Z. Every
measured pass recorded the same host load average at the start and end of the
pass. The one-, five-, and fifteen-minute load averages were:

| Run | Load average during both passes |
| --- | --- |
| Essentials redb | 11.3759765625, 11.75048828125, 9.306640625 |
| Essentials PostgreSQL | 11.10546875, 11.68798828125, 9.298828125 |
| Platform redb | 10.4560546875, 11.54345703125, 9.26171875 |
| Platform PostgreSQL | 10.65966796875, 11.5673828125, 9.283203125 |

All four processes exited 0. Each pass has 40 attempts, 40 successes, 0
errors, and no HTTP 429. Writer overlap is true. For 40 attempts, nearest-rank
p99 is the slowest attempt, so p99 and max are the same value in every pass
below. The `ps` interval is 50 ms, and these passes lasted 0.017 to 0.114
seconds, so the RSS figure is the maximum of 2 to 4 samples.

Quiet pass:

| Run | p50 µs | p95 µs | p99 µs | min µs | elapsed s | success/s | RSS KiB max | CPU s | samples |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Essentials redb | 378 | 538 | 952 | 326 | 0.017403 | 2298.45982 | 66736 | 0.01 | 2 |
| Essentials PostgreSQL | 1887 | 2254 | 2376 | 1784 | 0.078368 | 510.415669 | 66768 | 0.04 | 3 |
| Platform redb | 430 | 550 | 827 | 345 | 0.01843 | 2170.374369 | 74144 | 0.02 | 2 |
| Platform PostgreSQL | 1901 | 2218 | 3074 | 1742 | 0.07905 | 506.009393 | 73760 | 0.05 | 3 |

Interference pass:

| Run | p50 µs | p95 µs | p99 µs | min µs | elapsed s | success/s | RSS KiB max | CPU s | samples |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Essentials redb | 429 | 595 | 937 | 315 | 0.018789 | 2128.87209 | 67072 | 0.02 | 2 |
| Essentials PostgreSQL | 2324 | 4167 | 4548 | 1905 | 0.113862 | 351.301938 | 67008 | 0.11 | 4 |
| Platform redb | 431 | 623 | 957 | 361 | 0.019422 | 2059.568693 | 74384 | 0.02 | 2 |
| Platform PostgreSQL | 2246 | 4103 | 5431 | 1868 | 0.112714 | 354.879048 | 74032 | 0.11 | 4 |

The empty-group writer finished these HTTP results before the read loop ended:

| Run | Posts | Successes | 409 conflicts | Other errors | Revision refreshes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Essentials redb | 5 | 3 | 2 | 0 | 2 |
| Essentials PostgreSQL | 27 | 14 | 13 | 0 | 13 |
| Platform redb | 6 | 3 | 3 | 0 | 3 |
| Platform PostgreSQL | 27 | 14 | 13 | 0 | 13 |

On the interference pass, `responses_error_total` equals that 409 count
(2, 13, 3, and 13). The session-read error count stays 0. The maintenance
`finished` delta is 0 on every pass. Provisioning `finished` is 1 on the two
PostgreSQL interference passes and 0 on the other six passes.

The JSON files are outside this repository:

- `/Users/dominik/.cache/riauth-cargo/q09-8x8-47aa248/reports/essentials-redb.json`
- `/Users/dominik/.cache/riauth-cargo/q09-8x8-47aa248/reports/essentials-postgresql.json`
- `/Users/dominik/.cache/riauth-cargo/q09-8x8-47aa248/reports/platform-redb.json`
- `/Users/dominik/.cache/riauth-cargo/q09-8x8-47aa248/reports/platform-postgresql.json`

Each `.stdout.json` beside those files is byte-identical. The completion line
is in the matching `.stderr` file and in
`/Users/dominik/.grok/long-running-background-tasks/q09-8x8-runs-47aa248.log`.
The build log is
`/Users/dominik/.grok/long-running-background-tasks/q09-8x8-build-47aa248.log`.
No edition or backend was skipped, and none of the four runs failed.

## Relationship to the earlier harnesses

[S01 contention characterization](../../scripts/characterize-contention.sh)
runs an in-process release workload, including a `session_reads` phase, and
prints p50/p95/max plus telemetry deltas. It does not record p99, RSS, CPU, a
binary hash, or successful throughput. Its directory is hundreds of users and
groups, so its session-read numbers are a different workload from this fresh
init and from `q09-8u-8g`, which is eight groups on one 128-row index page.

S02's `indexed_user_group_membership` contract and
`scripts/test-contracts-postgres.sh` check the group index. S03's logout and
SSF probes in `tests/contention.rs` check writer behavior while a claim is
held. O05's `tests/worker_capacity.rs` checks admission queues. The
[capacity planning](../operations.md#capacity-planning) note still asks for a
deployment measurement before any capacity expectation. Those harnesses remain
in place. This script does not replace them.

Q08's [edition matrix](q08-exact-edition-bundles.md) shows that a fresh redb
or PostgreSQL instance can become ready and accept one administrator login on
either edition. That is a single login, not this repeated session read. The
[edition reference](../editions.md) is the statement that both artifacts share
Core.

## Remaining Q09 gate

The `q09-8u-8g` macOS dev-profile runs above keep four reports and tie each
binary hash to the build command, the dev profile, and rustc 1.98.1. Full
publication still requires:

- Linux x86-64 and Linux ARM64 packaged artifacts when a release build is the claimed artifact
- TLS, database encryption, and external signing included when the claimed deployment uses them
- a run long enough to overlap the 60-second maintenance cadence when the claim is about that background job; these measured passes lasted 0.017 to 0.114 seconds and recorded a maintenance `finished` delta of 0
- a repeat of this dataset on a host whose load is the condition being studied, or on a quiet host when that is the condition; these runs recorded a one-minute load average from 10.4560546875 to 11.3759765625 on 16 cores
- no numeric load, RPO, or RTO target until a named operator workload exists

The coverage inventory row for Q09 still says the benchmark gap is open. This
slice does not change that row.
