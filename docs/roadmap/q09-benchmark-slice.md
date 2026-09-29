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

The dataset is a fresh `init`: the bootstrap administrator, no extra users,
and no group memberships. Database encryption is off. The listener is loopback
HTTP without TLS. PostgreSQL, when selected, is a disposable loopback cluster
with trust authentication, `sslmode=disable`, `local_unencrypted`, and pool
size 8.

A quiet pass sends the session read from one client. An interference pass
repeats it while a second client sends `POST /api/groups` for empty groups,
with `Idempotency-Key` and `If-Match` set from `GET /api/state/revision`.
Empty groups do not add memberships, so the session read still walks an empty
per-user index. The writer is there to overlap the storage writer. Server
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
- iteration count, warmup, interference cap, and the estimated general-rate-limit budget
- for each pass: attempts, successes, error statuses and error codes, nearest-rank p50/p95/p99 latency over every attempt, successful throughput, `ps` RSS/CPU samples of the serve process, server counter deltas, and host load
- writer successes, conflicts, and errors, and the arithmetic difference of the latency percentiles

The general category allows 600 requests per minute from one address. `GET /api/me`, metrics, revision, and group creation share that category. The script refuses a shape whose estimate is above 500. Login is the separate `login` category. `/readyz` is a probe.

Exit 0 means every measured read succeeded, at least one group create succeeded, and no measured attempt returned 429. Exit 2 means the report was written and the writer completed no group. Exit 3 means a measured attempt returned 429. Exit 4 means some other measured error. Exit 1 means the run stopped before a report.

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

Repeat with `--features platform` and, separately, `--backend postgresql` when
`initdb`, `pg_ctl`, and `createdb` are installed. Keep the JSON from each run.
Compare runs only when the commit, security settings, dataset, and hardware
notes say they are the same measurement.

## Evidence for this slice

The check that runs with the source is `--self-check` and
`tests/test_q09_benchmark_slice.py`. The fixture proves the script's
percentiles, error accounting, redaction, RSS/CPU sampling, background-counter
delta, and group-writer overlap. It does not enforce riAuth's `If-Match`
rules; a product run is what meets the real server.

No riAuth binary was executed for this slice. This worktree had no `target/`
directory, and a full Cargo build was not started. The binary compiler and
build profile are therefore unrecorded. There is no session-read latency,
throughput, RSS, or error count for Essentials, Platform, redb, or PostgreSQL
in this commit.

## Relationship to the earlier harnesses

[S01 contention characterization](../../scripts/characterize-contention.sh)
runs an in-process release workload, including a `session_reads` phase, and
prints p50/p95/max plus telemetry deltas. It does not record p99, RSS, CPU, a
binary hash, or successful throughput. Its directory is hundreds of users and
groups, so its session-read numbers are a different workload from this fresh
init.

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

Full publication still requires product runs, not the fixture:

- Essentials and Platform binaries from the same commit, each measured on redb and on PostgreSQL, with the four reports kept
- the binary hash tied to the build command, profile, and toolchain that produced it
- Linux x86-64 and Linux ARM64 packaged artifacts when a release build is the claimed artifact
- a named larger directory when the claim is about group-index or scan cost, recorded beside this fresh-init slice
- TLS, database encryption, and external signing included when the claimed deployment uses them
- a run long enough to overlap the 60-second maintenance cadence when the claim is about that background job
- host load left in the report, and the run repeated when the machine is busy enough to dominate the samples
- no numeric load, RPO, or RTO target until a named operator workload exists

The coverage inventory row for Q09 still says the benchmark gap is open. This
slice does not change that row.
