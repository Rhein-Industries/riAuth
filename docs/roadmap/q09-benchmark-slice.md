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

By default the listener is loopback HTTP without TLS and database encryption
is off. PostgreSQL, when selected in that default mode, is a disposable
loopback cluster with SCRAM authentication and a fresh private password,
`sslmode=disable`, `local_unencrypted`, and pool size 8. The password is passed
to `initdb` through an owner-only file and to libpq through its explicit private
environment; it is excluded from printed evidence.

`--secure` is the other bounded mode. It keeps the same dataset, pace, and
general-request estimate. It generates a fresh private CA, a server
certificate whose SAN is `DNS:localhost` and `IP:127.0.0.1`, and a fresh
`database_key_file` of 32 bytes in base64url without padding. The issuer is
`https://127.0.0.1` on the same ephemeral port as the listener. The client
trusts that CA, checks the hostname, and requires the handshake to fail for
an unrelated CA and for the system trust store. Init receives
`--database-key-file`. The TLS file paths are written into the private config
before serve. The store format for that run is `aes256gcm-v1`.

A secure PostgreSQL run listens with TLS and uses `scram-sha-256` on
`hostssl` lines only. `local_unencrypted` is false. libpq preflight connects
with `sslmode=verify-full`, `host=localhost`, `hostaddr=127.0.0.1`, and the
private CA, and it must see server SSL on. The same preflight must fail for
an unrelated CA and for `sslmode=disable`. The connection file riAuth reads
uses `sslmode=require` with that host and hostaddr. This binary's parser
accepts `disable`, `prefer`, and `require`, then the client sets
`SslMode::Require`. rustls checks the hostname against the webpki roots and
`ca_file`. Putting `sslmode=verify-full` in the file riAuth reads is rejected
before connect. Product init with the unrelated CA must exit 6 with
`storage_unavailable` and must not publish the config. Bootstrap sets the
database credential with `initdb --pwfile` and a unix-socket `scram-sha-256`
line, then reloads `pg_hba.conf` to the `hostssl` lines before either libpq
preflight or riAuth init.

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
`initdb`, `pg_ctl`, and `createdb` are installed. Add `--secure` for the
private-CA HTTPS and `database_key_file` mode above. `--self-check` rejects
that flag. A secure run needs `openssl` on `PATH`. A secure PostgreSQL run
also needs `psql`. Keep the JSON from each run.
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

### Paced `q09-2u-2g` Platform redb on macOS, dev profile

This observation is one Platform redb product run on 2026-09-29. The worktree
HEAD was `a4b7a7947b452731a33cd77196c5594d5fce1b3a`, parent
`c7a61c63dc124ddd2af6d91379381d6b9c928af4`. The report has `source.commit`
equal to that HEAD, `source.dirty_paths` empty, `product_run: true`,
`observations_only: true`, and `performance_claim: false`. The measured script
is `scripts/q09_benchmark_slice.py`, SHA-256
`ecd4ce799087ed095c6604a92a18c2af4f02af33adfa7ac36907184bf4d5c510`.
`source.binary_compiler` is `unrecorded`. `source.checkout_rust_toolchain` is
`1.98.1`.

This session did not run Cargo and did not use the accepted `target/`
directory as `CARGO_TARGET_DIR`. The measured file is a private copy of
`target/debug/riauth` from the integration-accepted worktree while that
worktree's HEAD was `c7a61c6`. A later read-only `cmp` found the copy and that
file identical: 290204712 bytes, mtime 2026-09-29T19:11:26+0200, SHA-256
`517093cbca1fdf56cfa093f43b79cd5d30405aa5bc10b68759ca4cbf63d92e6b`. The copy
mode is `-r-xr-xr-x`. The accepted commit time is 2026-09-29T19:10:42+0200.
A byte search of the measured file found rustc
`48a229ceaefd4985c50990b14116b6d856af0985` and the path
`src/assembly/source_saml_return.rs`. The same search found no `c7a61c6` and
no full commit hash. `nm` lists
`__RNvNtNtCs6D1EO0VxpAw_6riauth8assembly18source_saml_return19take_browser_return`.
In the accepted worktree, `src/assembly/source_saml_return.rs` has mtime
2026-09-29T19:10:42+0200. Fingerprint
`target/debug/.fingerprint/riauth-6b3a95903ee814bf/bin-riauth.json` records
features `default`, `essentials`, `platform`, and `test-support`, with empty
rustflags. `invoked.timestamp` there is 2026-09-29T19:11:24+0200.
`target/debug/deps/riauth-a0245249f5a6dba2.d` is 2026-09-29T19:11:08+0200 and
names `source_saml_return.rs`. Capabilities and runtime `build_features` are
`essentials` and `platform`. `Cargo.toml` declares `test-support = []`. The
report build fields are the supplied tokens `debug`, `1.98.1`, and
`default,essentials,platform,test-support`. The package version is `0.1.1`,
capabilities schema `riauth.capabilities/v2`, target `macos` / `aarch64`, and
runtime scope `instance` with storage backend `redb`.

The command, from this worktree, was:

```sh
python3 scripts/q09_benchmark_slice.py \
  --binary /Users/dominik/.cache/riauth-cargo/q09-maint-c7a61c6/riauth \
  --backend redb \
  --iterations 12 \
  --warmup 1 \
  --interference-cap 4 \
  --directory-users 2 \
  --directory-groups 2 \
  --pace-ms 6500 \
  --build-profile debug \
  --build-toolchain 1.98.1 \
  --build-features default,essentials,platform,test-support \
  --out /Users/dominik/.cache/riauth-cargo/q09-maint-c7a61c6/reports/platform-redb-maintenance.json
```

It started at 2026-09-29T17:23:57Z and exited 0 at 2026-09-29T17:26:25Z.
Standard error recorded
`completion=0 product_run=true backend=redb edition=platform dataset=q09-2u-2g directory_users=2 directory_groups=2 verified=true pace_ms=6500 quiet_maintenance_overlap=true interference_maintenance_overlap=true`.

`q09-2u-2g` verified true: the administrator plus `q09-user-0001` and
`q09-user-0002` (three accounts), groups `q09-dir-0001` and `q09-dir-0002`,
six memberships, and three members in each group. The administrator
session-read groups were those two names. The directory is inside one 128-row
index page. The estimated general-request budget is 65 (directory writes 10,
directory revision reads 11, directory verification 4, measured reads 24,
warmup reads 2, metrics 4, revision reads 5, group creates 4, runtime
capabilities 1). Login is the separate login category. TLS was off, database
encryption was off, and the listener was loopback HTTP. `--pace-ms 6500`
sleeps between the twelve measured reads, eleven times, 71.5 seconds, outside
each latency sample and inside the pass wall clock. The throughput figures
include that sleep.

The host was an Apple M4 Max with 16 logical CPUs, 68719476736 bytes of
memory, macOS 26.2, Darwin 25.2.0 arm64 (`dgsPro`), and Python 3.14.6. The
report hardware load average equals the quiet pass start: 7.095703125,
8.64208984375, 11.14697265625. For 12 attempts, nearest-rank p99 is the
slowest attempt, so p99 and max are the same value in both passes. The `ps`
interval is 50 ms. Both passes had 12 attempts, 12 successes, 0 session-read
errors, and no HTTP 429. Maintenance `finished` increased by 1 and
`maintenance_cadence_overlap` is true on both passes. Maintenance `failed`
stayed 0.

| Pass | p50 µs | p95 µs | p99 µs | min µs | elapsed s | success/s | RSS KiB min | RSS KiB max | CPU s | samples |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Quiet | 3350 | 5399 | 5399 | 522 | 71.584353 | 0.167634 | 71856 | 74064 | 0.54 | 1105 |
| Interference | 3419 | 6795 | 6795 | 1435 | 71.587186 | 0.167628 | 74016 | 74320 | 0.53 | 1066 |

Quiet RSS started at 71856 KiB and ended at 73984 KiB. Interference RSS
started at 74016 KiB and ended at 74288 KiB. Sampler read errors were 0.
Quiet host load went from 7.095703125, 8.64208984375, 11.14697265625 to
6.84033203125, 8.14453125, 10.73974609375. Interference host load went from
6.84033203125, 8.14453125, 10.73974609375 to 7.99267578125, 8.20361328125,
10.5546875.

The empty-group writer recorded 4 posts, 2 successes, 2 HTTP 409 conflicts,
0 other errors, and 2 revision refreshes. Writer overlap is true. The
interference-minus-quiet latency summary, in microseconds, is min 913, p50
69, p95 1396, p99 1396, and max 1396.

Server counter deltas during the passes:

| Counter | Quiet | Interference |
| --- | ---: | ---: |
| maintenance finished | 1 | 1 |
| alerts finished | 0 | 1 |
| logout_ssf finished | 36 | 35 |
| mail finished | 14 | 14 |
| reconciliation finished | 14 | 14 |
| provisioning finished | 286 | 287 |
| deactivation finished | 286 | 287 |
| manual_connector finished | 0 | 0 |
| requests_total | 13 | 20 |
| responses_error_total | 0 | 2 |
| rate_limited_total | 0 | 0 |
| worker_rejections_total | 0 | 0 |
| cleanup_count | 1 | 1 |
| cleanup_errors | 0 | 0 |
| write_hold_count | 340 | 345 |
| write_wait_count | 340 | 345 |

Every background `failed` delta in that window is 0. The interference
`responses_error_total` of 2 equals the writer 409 count. The quiet
`requests_total` of 13 is the 12 measured reads plus the closing metrics
read. The interference `requests_total` of 20 adds the writer's opening
revision read, 4 posts, and 2 revision refreshes.

The JSON file is outside this repository:

`/Users/dominik/.cache/riauth-cargo/q09-maint-c7a61c6/reports/platform-redb-maintenance.json`

SHA-256 `5dee2c961051225758e6b77d24ee74b14eff94f9913d6cd1b580b7c367391a85`.
The sibling `.stdout.json` is byte-identical. The completion line is in
`/Users/dominik/.grok/long-running-background-tasks/q09-maint-run.log`.
A text search of the JSON found no password and no session token.

Available space on `/System/Volumes/Data` was 10143980 KiB when the command
started and 10079144 KiB after it exited. Both readings are above 7340032 KiB.
No `riauth-q09-` temporary directory remained. The JSON and the binary copy
were kept. The accepted worktree and main were not edited.

The limits of this observation are the single Platform redb loopback shape,
the macOS dev-profile binary, the Cargo fingerprint's `test-support` feature,
the host load above, and the pace inside the throughput. It assigns no load,
recovery, or capacity target.

### Paced `q09-2u-2g` Platform PostgreSQL on macOS, dev profile

This observation repeats that shape on disposable loopback PostgreSQL. The
worktree HEAD was `551ac636a011750ec2a0c57e734300f6c0020786`. The report has
`source.commit` equal to that commit, `source.dirty_paths` empty,
`product_run: true`, `observations_only: true`, and `performance_claim: false`.
The script SHA-256 is again
`ecd4ce799087ed095c6604a92a18c2af4f02af33adfa7ac36907184bf4d5c510`.
`source.binary_compiler` is `unrecorded`. `source.checkout_rust_toolchain` is
`1.98.1`.

This session did not run Cargo. The measured file is the same immutable copy
recorded above, SHA-256
`517093cbca1fdf56cfa093f43b79cd5d30405aa5bc10b68759ca4cbf63d92e6b`,
290204712 bytes, mode `-r-xr-xr-x`. Capabilities and runtime `build_features`
are `essentials` and `platform`, edition `platform`, version `0.1.1`, target
`macos` / `aarch64`. Runtime scope is `instance` and `storage_backend` is
`postgresql`. The report build fields are the supplied tokens `debug`,
`1.98.1`, and `default,essentials,platform,test-support`. The provenance
limits of that copy are the ones stated for the redb run: the file has no
embedded `c7a61c6`, the Cargo fingerprint includes `test-support`, and
capabilities omit that feature name.

The command, from this worktree, was:

```sh
python3 scripts/q09_benchmark_slice.py \
  --binary /Users/dominik/.cache/riauth-cargo/q09-maint-c7a61c6/riauth \
  --backend postgresql \
  --iterations 12 \
  --warmup 1 \
  --interference-cap 4 \
  --directory-users 2 \
  --directory-groups 2 \
  --pace-ms 6500 \
  --build-profile debug \
  --build-toolchain 1.98.1 \
  --build-features default,essentials,platform,test-support \
  --out /Users/dominik/.cache/riauth-cargo/q09-maint-c7a61c6/reports/platform-postgresql-maintenance.json
```

`initdb` and `pg_ctl` were PostgreSQL 16.14 from Homebrew. The script creates
the cluster inside its `riauth-q09-` temporary directory with `initdb -U
riauth_test --auth=trust --encoding=UTF8 --no-locale`, appends
`listen_addresses = '127.0.0.1'`, a port taken from a `127.0.0.1:0` bind, and
`unix_socket_directories = ''`, then runs `createdb` for `riauth_q09`. The
connection file uses `sslmode=disable`. The report records
`postgres_local_unencrypted: true` and `postgres_pool_size: 8`. The chosen
port numbers are not stored in the JSON. The listener for riAuth is a
separate ephemeral `127.0.0.1` port. TLS was off and database encryption was
off.

It started at 2026-09-29T18:27:01Z and exited 0 at 2026-09-29T18:29:28Z.
Standard error recorded
`completion=0 product_run=true backend=postgresql edition=platform dataset=q09-2u-2g directory_users=2 directory_groups=2 verified=true pace_ms=6500 quiet_maintenance_overlap=true interference_maintenance_overlap=true`.

`q09-2u-2g` verified true with the same accounts, groups, six memberships,
and three members per group as the redb run. The estimated general-request
budget is 65. `--pace-ms 6500` again sleeps 71.5 seconds inside each pass
wall clock and outside each latency sample. Throughput includes that sleep.
For 12 attempts, nearest-rank p99 is the slowest attempt.

The host was the same Apple M4 Max, 16 logical CPUs, 68719476736 bytes,
macOS 26.2, Darwin 25.2.0 arm64 (`dgsPro`), and Python 3.14.6. The report
hardware load average, taken before the measured passes, was 6.3515625,
6.5810546875, 6.7001953125. Both passes had 12 attempts, 12 successes, 0
session-read errors, empty error statuses, and no HTTP 429. Maintenance
`finished` increased by 1 and `maintenance_cadence_overlap` is true on both
passes. Maintenance `failed` stayed 0. `rate_limited_total` and
`worker_rejections_total` stayed 0.

| Pass | p50 µs | p95 µs | p99 µs | min µs | elapsed s | success/s | RSS KiB min | RSS KiB max | CPU s | samples |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Quiet | 6592 | 20989 | 20989 | 3883 | 71.639437 | 0.167506 | 49680 | 74240 | 0.84 | 1188 |
| Interference | 5790 | 16556 | 16556 | 4166 | 71.628744 | 0.167531 | 50032 | 50496 | 0.91 | 1185 |

Quiet RSS started at 73792 KiB and ended at 49984 KiB. Interference RSS
started at 50032 KiB and ended at 50400 KiB. Sampler read errors were 0.
Quiet host load went from 6.0830078125, 6.521484375, 6.67822265625 to
12.5654296875, 7.95703125, 7.17822265625. Interference host load went from
12.5654296875, 7.95703125, 7.17822265625 to 8.640625, 7.68798828125,
7.13427734375.

The empty-group writer recorded 4 posts, 2 successes, 2 HTTP 409 conflicts,
0 other errors, and 2 revision refreshes. Writer overlap is true. The
interference-minus-quiet latency summary, in microseconds, is min 283, p50
-802, p95 -4433, p99 -4433, and max -4433. Each figure is the interference
percentile minus the quiet percentile.

Server counter deltas during the passes:

| Counter | Quiet | Interference |
| --- | ---: | ---: |
| maintenance finished | 1 | 1 |
| alerts finished | 1 | 0 |
| logout_ssf finished | 35 | 36 |
| mail finished | 14 | 14 |
| reconciliation finished | 14 | 14 |
| provisioning finished | 287 | 286 |
| deactivation finished | 287 | 286 |
| manual_connector finished | 0 | 0 |
| requests_total | 13 | 20 |
| responses_error_total | 0 | 2 |
| rate_limited_total | 0 | 0 |
| worker_rejections_total | 0 | 0 |
| cleanup_count | 1 | 1 |
| cleanup_errors | 0 | 0 |
| write_hold_count | 354 | 364 |
| write_wait_count | 354 | 364 |

Every background `failed` delta in that window is 0. The interference
`responses_error_total` of 2 equals the writer 409 count. The quiet
`requests_total` of 13 is the 12 measured reads plus the closing metrics
read. The interference `requests_total` of 20 adds the writer's opening
revision read, 4 posts, and 2 revision refreshes.

The JSON file is outside this repository:

`/Users/dominik/.cache/riauth-cargo/q09-maint-c7a61c6/reports/platform-postgresql-maintenance.json`

SHA-256 `b323aad0c7503ba7cc9f8e5a075d4cb965f55256bf8c94e896c62d23ebf99f86`.
The sibling `.stdout.json` is byte-identical. The completion line is in
`/Users/dominik/.grok/long-running-background-tasks/q09-pg-maint-run.log`.
A text search of the JSON found no password and no session token. The redb
report beside it was left in place.

Available space on `/System/Volumes/Data` was 95449536 KiB when the command
started and 95514732 KiB after it exited. Both readings are above 7340032 KiB.
No `riauth-q09-` temporary directory remained, and the copied `riauth` process
was gone. The JSON and the binary copy were kept. This session did not edit
the accepted worktree or main.

The limits of this observation are the single Platform PostgreSQL loopback
cluster with trust authentication and `sslmode=disable`, the same macOS
dev-profile binary, the host load above, and the pace inside the throughput.
It assigns no load, recovery, or capacity target.

### Paced `q09-2u-2g` Essentials on macOS, dev profile

These two observations repeat the same shape on Essentials, first redb and
then disposable loopback PostgreSQL. The worktree HEAD for the build and both
runs was `6ca4779b68cf41e29af490d2aca6ea3d59e1f2bd`. Each report has that
`source.commit`, empty `dirty_paths`, `product_run: true`,
`observations_only: true`, and `performance_claim: false`. The script SHA-256
is `ecd4ce799087ed095c6604a92a18c2af4f02af33adfa7ac36907184bf4d5c510`, the
same script blob as the Platform reports above. `source.binary_compiler` is
`unrecorded`. `source.checkout_rust_toolchain` is `1.98.1`.

The Platform copy above was built from ancestor
`c7a61c63dc124ddd2af6d91379381d6b9c928af4`. This Essentials binary was built
from `6ca4779`. The commits after `c7a61c6` through this HEAD are `8c3aa7d` (the pace harness),
`551ac63` and `6ca4779` (the Platform observation docs), `a5769bb` (an LDAP
directory-import doc), and `7fb40d8` (the SAML source callback claim move).
`git diff c7a61c6 HEAD -- src` lists `src/assembly.rs`,
`src/assembly/source_saml_claim.rs`, and `src/source/saml.rs`.
`src/background.rs`, `src/api/server.rs`, `Cargo.toml`, `Cargo.lock`, and
`rust-toolchain.toml` are the same blobs at both commits. Maintenance and
alerts use a 60-second cadence, and `src/api/server.rs` spawns maintenance
without a platform feature gate. These rows are separate observations. They
are not a same-head comparison of edition latency or throughput.

This session built the binary with a new private target and did not use the
accepted `target/` directory or the Platform copy's directory as
`CARGO_TARGET_DIR`:

```sh
export CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR="$HOME/.cache/riauth-cargo/q09-essentials-6ca4779"
cargo build -j 4 --locked --offline --no-default-features \
  --features essentials,test-support --bin riauth
```

The build started at 2026-09-29T18:37:54Z and Cargo exited 0 at
2026-09-29T18:39:10Z. `rustc --version --verbose` reported rustc 1.98.1
(`48a229ceaefd4985c50990b14116b6d856af0985`, 2026-09-01), host
`aarch64-apple-darwin`, LLVM 22.1.8. The dev-profile file is
`target/debug/riauth` in that private directory. Fingerprint
`debug/.fingerprint/riauth-5d519e6836cb5d8a/bin-riauth.json` records features
`essentials` and `test-support`, with empty rustflags. Its profile field is
the numeric hash `17270959807823809533`. Before either run, the file was
copied to
`/Users/dominik/.cache/riauth-cargo/q09-essentials-6ca4779-kept/riauth` and
made mode `-r-xr-xr-x`. The copy and the built file were byte-identical:
205063032 bytes, SHA-256
`c6f0ba061663393c0d7874e1dac19ccd2bca48d781555e86a7717b4653a8a7dd`.
Capabilities and runtime `build_features` are `essentials`. Edition is
`essentials`, version `0.1.1`, target `macos` / `aarch64`. `Cargo.toml`
declares `test-support = []` and `default = ["platform"]`; this command
selected neither `default` nor `platform`. A byte search found the rustc hash
and the private target path embedded by the compiler. It found no full git
commit and neither `src/assembly/source_saml_claim.rs` nor
`src/assembly/source_saml_return.rs`. Capabilities report
`identity.saml_sources` compiled false and `operations.postgresql` compiled
true. The build log is
`/Users/dominik/.grok/long-running-background-tasks/q09-essentials-build.log`.

Both runs used `--iterations 12 --warmup 1 --interference-cap 4
--directory-users 2 --directory-groups 2 --pace-ms 6500` and the build tokens
`debug`, `1.98.1`, and `essentials,test-support`. `q09-2u-2g` verified true
on both: the administrator plus `q09-user-0001` and `q09-user-0002`, groups
`q09-dir-0001` and `q09-dir-0002`, six memberships, and three members in each
group. The estimated general-request budget is 65. The pace sleeps 71.5
seconds inside each pass wall clock and outside each latency sample.
Throughput includes that sleep. For 12 attempts, nearest-rank p99 is the
slowest attempt. The host was the Apple M4 Max, 16 logical CPUs,
68719476736 bytes, macOS 26.2, Darwin 25.2.0 arm64 (`dgsPro`), and Python
3.14.6. PostgreSQL tools were 16.14 from Homebrew. The PostgreSQL cluster
used the script's ephemeral `127.0.0.1` port, trust authentication, disabled
unix sockets, `sslmode=disable`, `postgres_local_unencrypted: true`, and pool
size 8. The riAuth listener was a separate ephemeral loopback port. TLS and
database encryption were off. The run log is
`/Users/dominik/.grok/long-running-background-tasks/q09-essentials-run.log`.

#### Essentials redb

The redb command started at 2026-09-29T18:39:56Z and exited 0 at
2026-09-29T18:42:21Z. Standard error recorded
`completion=0 product_run=true backend=redb edition=essentials dataset=q09-2u-2g directory_users=2 directory_groups=2 verified=true pace_ms=6500 quiet_maintenance_overlap=true interference_maintenance_overlap=true`.
Runtime `storage_backend` is `redb`. Both passes had 12 attempts, 12
successes, 0 session-read errors, empty error statuses and codes, and no
HTTP 429. Maintenance `finished` increased by 1 and
`maintenance_cadence_overlap` is true. Maintenance `failed` stayed 0.
`rate_limited_total` and `worker_rejections_total` stayed 0. The report
hardware load average, taken before the measured passes, was 10.814453125,
10.41259765625, 8.58056640625.

| Pass | p50 µs | p95 µs | p99 µs | min µs | elapsed s | success/s | RSS KiB min | RSS KiB max | CPU s | samples |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Quiet | 1572 | 3647 | 3647 | 641 | 71.563508 | 0.167683 | 43232 | 67392 | 0.34 | 1166 |
| Interference | 1146 | 2049 | 2049 | 954 | 71.540708 | 0.167737 | 42288 | 44096 | 0.25 | 1250 |

Quiet RSS started at 67136 KiB and ended at 43456 KiB. Interference RSS
started at 43568 KiB and ended at 43232 KiB. Sampler read errors were 0.
Quiet host load went from 10.814453125, 10.41259765625, 8.58056640625 to
11.53857421875, 10.5947265625, 8.78466796875. Interference host load went
from 11.53857421875, 10.5947265625, 8.78466796875 to 16.57177734375,
11.99658203125, 9.44921875.

The writer recorded 4 posts, 2 successes, 2 HTTP 409 conflicts, 0 other
errors, and 2 revision refreshes. Writer overlap is true. The
interference-minus-quiet latency summary, in microseconds, is min 313, p50
-426, p95 -1598, p99 -1598, and max -1598.

| Counter | Quiet | Interference |
| --- | ---: | ---: |
| maintenance finished | 1 | 1 |
| alerts finished | 0 | 0 |
| logout_ssf finished | 36 | 36 |
| mail finished | 14 | 14 |
| reconciliation finished | 14 | 14 |
| provisioning finished | 286 | 287 |
| deactivation finished | 286 | 286 |
| manual_connector finished | 0 | 0 |
| requests_total | 13 | 20 |
| responses_error_total | 0 | 2 |
| rate_limited_total | 0 | 0 |
| worker_rejections_total | 0 | 0 |
| cleanup_count | 1 | 1 |
| cleanup_errors | 0 | 0 |
| write_hold_count | 335 | 339 |
| write_wait_count | 335 | 340 |

Every background `failed` delta is 0. The interference
`responses_error_total` of 2 equals the writer 409 count. Quiet
`requests_total` 13 is the 12 measured reads plus the closing metrics read.
Interference `requests_total` 20 adds the writer's opening revision read, 4
posts, and 2 revision refreshes.

The JSON file is
`/Users/dominik/.cache/riauth-cargo/q09-essentials-6ca4779-kept/reports/essentials-redb-maintenance.json`,
SHA-256 `ccd7d03d43461aa3e3967d32097a39821e379ba4557fb3976d78238b6d5d71f8`.
The sibling `.stdout.json` is byte-identical. A text search found no password
and no session token.

#### Essentials PostgreSQL

The PostgreSQL command started at 2026-09-29T18:42:21Z, after the redb
process had exited, and exited 0 at 2026-09-29T18:44:48Z. Standard error
recorded
`completion=0 product_run=true backend=postgresql edition=essentials dataset=q09-2u-2g directory_users=2 directory_groups=2 verified=true pace_ms=6500 quiet_maintenance_overlap=true interference_maintenance_overlap=true`.
Runtime `storage_backend` is `postgresql`. Both passes had 12 attempts, 12
successes, 0 session-read errors, empty error statuses and codes, and no
HTTP 429. Maintenance `finished` increased by 1 and
`maintenance_cadence_overlap` is true. Maintenance `failed`,
`rate_limited_total`, and `worker_rejections_total` stayed 0. The report
hardware load average was 16.57177734375, 11.99658203125, 9.44921875.

| Pass | p50 µs | p95 µs | p99 µs | min µs | elapsed s | success/s | RSS KiB min | RSS KiB max | CPU s | samples |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Quiet | 4657 | 8932 | 8932 | 2544 | 71.595949 | 0.167607 | 62032 | 67216 | 0.58 | 1247 |
| Interference | 5361 | 10236 | 10236 | 3736 | 71.616158 | 0.167560 | 43744 | 63168 | 0.77 | 1210 |

Quiet RSS started at 67104 KiB and ended at 62656 KiB. Interference RSS
started at 62816 KiB and ended at 43856 KiB. Sampler read errors were 0.
Quiet host load went from 16.57177734375, 11.99658203125, 9.44921875 to
13.18408203125, 12.154296875, 9.73388671875. Interference host load went from
13.18408203125, 12.154296875, 9.73388671875 to 11.48388671875, 12.0,
9.8720703125.

The writer recorded 4 posts, 2 successes, 2 HTTP 409 conflicts, 0 other
errors, and 2 revision refreshes. Writer overlap is true. The
interference-minus-quiet latency summary, in microseconds, is min 1192, p50
704, p95 1304, p99 1304, and max 1304.

| Counter | Quiet | Interference |
| --- | ---: | ---: |
| maintenance finished | 1 | 1 |
| alerts finished | 1 | 0 |
| logout_ssf finished | 35 | 36 |
| mail finished | 14 | 14 |
| reconciliation finished | 14 | 14 |
| provisioning finished | 286 | 287 |
| deactivation finished | 286 | 287 |
| manual_connector finished | 0 | 0 |
| requests_total | 13 | 20 |
| responses_error_total | 0 | 2 |
| rate_limited_total | 0 | 0 |
| worker_rejections_total | 0 | 0 |
| cleanup_count | 1 | 1 |
| cleanup_errors | 0 | 0 |
| write_hold_count | 348 | 360 |
| write_wait_count | 348 | 360 |

Every background `failed` delta is 0. The interference error total equals the
writer 409 count, and the request totals have the same composition as the
redb run.

The JSON file is
`/Users/dominik/.cache/riauth-cargo/q09-essentials-6ca4779-kept/reports/essentials-postgresql-maintenance.json`,
SHA-256 `c167db90264d6de018dde46c4107f870883008d0949f723116f8d64fcb2b90e9`.
The sibling `.stdout.json` is byte-identical. A text search found no password
and no session token.

Available space on `/System/Volumes/Data` was 88414068 KiB before the build,
84018572 KiB after it, 83830164 KiB before the redb run, and 81848912 KiB
after the PostgreSQL run. Each reading is above 8388608 KiB. No `riauth-q09-`
directory remained, and the copied Essentials process was gone. The Platform
copy and its two reports were not replaced. This session did not edit the
accepted worktree or main.

The limits of these observations are the macOS dev profile, the empty
`test-support` feature in the Cargo fingerprint, trust authentication with
`sslmode=disable` on the PostgreSQL cluster, loopback HTTP, the host load
above, and the pace inside the throughput. They assign no load, recovery, or
capacity target.

### Paced `q09-2u-2g` Linux ARM64 release binaries in Docker's VM

These four observations repeat that shape in order: Essentials redb,
Essentials PostgreSQL, Platform redb, then Platform PostgreSQL. Each report
has `source.commit` `cc186af5777e44c74f4dfe3fc67886573f745b66`, empty
`dirty_paths`, `product_run: true`, `observations_only: true`, and
`performance_claim: false`. The script SHA-256 is
`ecd4ce799087ed095c6604a92a18c2af4f02af33adfa7ac36907184bf4d5c510`.
`source.binary_compiler` is `unrecorded`. `source.checkout_rust_toolchain`
is `1.98.1`. That commit is the harness checkout where the script ran.
`6ca4779` is an ancestor of it, and the script blob is the same at both
commits.

The measured binaries were compiled from
`6ca4779b68cf41e29af490d2aca6ea3d59e1f2bd`. Q08's `container/build.sh`
printed `source_commit=6ca4779b68cf41e29af490d2aca6ea3d59e1f2bd` and ran:

```sh
cargo build --release --locked --no-default-features \
  --features essentials --bins --target aarch64-unknown-linux-gnu -j 4
cargo build --release --locked --no-default-features \
  --features platform --bins --target aarch64-unknown-linux-gnu -j 4
```

with `CARGO_INCREMENTAL=0` and `CARGO_BUILD_JOBS=4`. Neither command names
`test-support`. At that commit `Cargo.toml` sets `default = ["platform"]`
and `platform = ["essentials", ...]`. `--no-default-features` leaves the
default set off. The platform command still compiles the essentials
dependency, so capabilities `build_features` are `essentials` for the first
binary and `essentials`, `platform` for the second. The report `build`
tokens are `features` `essentials` or `platform`, `profile` `release`, and
`toolchain` `1.98.1`. `rustc -Vv` in the Q08 build log reported rustc 1.98.1
(`48a229ceaefd4985c50990b14116b6d856af0985`, 2026-09-01), host
`aarch64-unknown-linux-gnu`, LLVM 22.1.8, and cargo 1.98.1. Cargo finished
the essentials server in 3m 07s and the platform server in 3m 20s, both as
the `release` profile. The build container uname was `Linux 08ba64ae7e60
7.0.12-linuxkit #1 SMP PREEMPT Fri Aug 14 16:27:59 UTC 2026 aarch64`.

From `6ca4779` to `cc186af`, `git diff -- src` names `src/assembly.rs`,
`src/assembly/source_saml_record.rs`, and `src/source/saml.rs`. At `cc186af`,
`Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, and
`scripts/q09_benchmark_slice.py` are the same blobs as `6ca4779`. These
Linux binaries were compiled before that SAML record move. Essentials
capabilities report `identity.saml_sources` compiled false. Platform
capabilities report it compiled true. Both report version `0.1.1` and target
`linux` / `aarch64`. The macOS rows above used dev-profile binaries on
Darwin. The macOS Essentials fingerprint also named `test-support`. These
four rows stand on their own measurements. The secure-mode section below
uses a later script blob. `git diff 6ca4779 -- src` at that later checkout
still names only those three SAML files, and the Cargo inputs above stay the
same blobs.

`file` and `readelf` in the Q08 build log, and `file` again in the
measurement container, describe both `riauth` servers as ELF 64-bit LSB pie
executables, ARM aarch64, dynamically linked, interpreter
`/lib/ld-linux-aarch64.so.1`, for GNU/Linux 3.7.0, stripped, Class ELF64,
Machine AArch64. Essentials BuildID
`2768345aaf193ff729e6a55d4d690b04b0c09f27`, 36324648 bytes, SHA-256
`5feac36b7ad5d292cafb192b673f4353f8fa424e90a35d667d8fa7f41cac74eb`. Platform
BuildID `d7afe1ac9db44bec234f7aca3dbcdd4405428650`, 49302784 bytes, SHA-256
`606a2ce5d72de16fe8749efdb3504fd81471c88118008180667a013ccffea79d`. The
private copies are
`/Users/dominik/.cache/riauth-cargo/q09-linux-arm64-6ca4779/essentials/riauth`
and
`/Users/dominik/.cache/riauth-cargo/q09-linux-arm64-6ca4779/platform/riauth`,
mode `-r-xr-xr-x`. Those hashes match the Q08 artifact files and the
`sha256sum` lines in that build log. This session did not run Cargo and did
not edit the Q08 tree. Each report stores `artifact.path` as the container
mount, `/evidence/bin/essentials/riauth` or `/evidence/bin/platform/riauth`.

The measurement image is the local tag `q09-linux-arm64-runtime:cc186af`,
id `sha256:a04d38b65247359a288a139686b942190737eca25ef1ccf504b93c7a532fb4fd`,
linux/arm64. Its base is
`docker.io/library/rust:1.98.1-trixie@sha256:a8a5f0a1e5fe7dfe1d352591e4a1c7dd2c08fd70475cae872cf3458ba0df0546`,
plus Python, procps, and PostgreSQL. The run identity was
`uid=10000(q09bench)`, Debian GNU/Linux 13 (trixie), Python 3.13.5,
procps-ng 4.0.4, and `initdb (PostgreSQL) 17.11 (Debian 17.11-0+deb13u1)`.
That uid ran the benchmark and `initdb`. The container had no memory limit
flag. MemTotal was 8021540 kB, `nproc` was 16, and cgroup `memory.max` was
`max`. Process uname was `Linux c274e21b3548 7.0.12-linuxkit #1 SMP PREEMPT
Fri Aug 14 16:27:59 UTC 2026 aarch64`. Docker server 29.7.2 reports
linux/arm64. The Mac host in the launcher log is Darwin `dgsPro` 25.2.0
arm64, Apple M4 Max, `hw.memsize` 68719476736, 16 CPUs, macOS 26.2. The
linuxkit kernel is Docker Desktop's Linux VM. These files are the local Q08
build copies from `6ca4779`. The logs record no release-workflow run id and
no published bundle name.

The container set `safe.directory=*` so uid 10000 could read the mounted
worktree. The script then recorded the checkout above. A `ps` probe before
the product runs returned `rss_kib` 21500 and `cpu_seconds` 0.0. procps-ng
4.0.4 prints CPU time in whole seconds, and the product samples are 0.0 or
1.0. Sampler read errors on those passes are 0. The script file stayed as it
was, so the protocol hash above is the script file hash.

Every run used a new directory and the binary of that edition, with
`--iterations 12 --warmup 1 --interference-cap 4 --directory-users 2
--directory-groups 2 --pace-ms 6500`. The verified dataset is `q09-2u-2g`:
administrator `admin`, extra users `q09-user-0001` and `q09-user-0002`,
groups `q09-dir-0001` and `q09-dir-0002`, three members in each group, and
six memberships. The estimated general-request budget is 65. TLS and database
encryption are off. PostgreSQL runs set `postgres_local_unencrypted: true`
and pool size 8, with trust authentication, `sslmode=disable`, and ephemeral
loopback ports. The listener is `127.0.0.1` ephemeral. The 6500 ms pace sits
between measured reads, inside the pass wall clock and outside each latency
sample. For 12 attempts, nearest-rank `ceil(n*q)` selects the maximum for
both p95 and p99. Every pass had 12 attempts, 12 successes, 0 session-read
errors, empty error statuses and codes, maintenance `finished` delta 1,
`failed` 0, and `maintenance_cadence_overlap` true. Elapsed time is 71.53 to
71.61 seconds. The writer on every run recorded 4 posts, 2 successes, 2 HTTP
409 conflicts, 0 other errors, 2 revision refreshes, and overlap true. Quiet
`requests_total` is 13 and interference `requests_total` is 20. Interference
`responses_error_total` is 2, the same count as the writer 409 responses.
`rate_limited_total` and `worker_rejections_total` are 0. `cleanup_count` is
1 and `cleanup_errors` is 0. Mail and reconciliation `finished` are 14.
`manual_connector` finished is 0. Every background `failed` delta is 0. On
every pass, RSS at the start equals the minimum and RSS at the end equals
the maximum.

Q08's local installed-smoke note records a separate transition. Platform
could not open an Essentials store because the configured active capabilities
did not match the initialized instance, and a preflight refused disabling
`access.temporary_entitlements`. Each run here started from a fresh directory
and the matching edition binary. That transition stays with Q08.

The inner log is
`/Users/dominik/.cache/riauth-cargo/q09-linux-arm64-6ca4779/reports/run.log`,
SHA-256
`e7f6077489223f423fdbc60b2022b0aa90238c42591d408423887237fae3eb72`.
It ends `END fail=0` and `RUN_OK`. The host launcher log is
`/Users/dominik/.grok/long-running-background-tasks/q09-linux-run.log`,
SHA-256
`9db149c9e806d6c0288dc678f202ac333ee9e2b8ea748eff959c5639a3c1cb01`.
The image build log is
`/Users/dominik/.grok/long-running-background-tasks/q09-linux-image.log`,
SHA-256
`6125aa7d937bb4993c1a89d9827c1d3a7b9568c43f30e0b85ff4704fde8d4bb3`.

#### Essentials redb

The redb command started at 2026-09-29T19:00:51Z and exited 0 at
2026-09-29T19:03:15Z. Standard error recorded
`completion=0 product_run=true backend=redb edition=essentials dataset=q09-2u-2g directory_users=2 directory_groups=2 verified=true pace_ms=6500 quiet_maintenance_overlap=true interference_maintenance_overlap=true`.
Runtime `storage_backend` is `redb`. The report hardware load average, taken
before the measured passes, was 0.6689453125, 0.99951171875, 1.1220703125.

| Pass | p50 µs | p95 µs | p99 µs | min µs | elapsed s | success/s | RSS KiB min | RSS KiB max | CPU s | samples |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Quiet | 1112 | 2377 | 2377 | 274 | 71.531113 | 0.167759 | 48540 | 49108 | 0.0 | 1274 |
| Interference | 1488 | 3234 | 3234 | 457 | 71.546728 | 0.167723 | 49224 | 49312 | 0.0 | 1277 |

Quiet host load went from 0.6689453125, 0.99951171875, 1.1220703125 to
0.56640625, 0.8837890625, 1.0703125. Interference host load went from
0.56640625, 0.8837890625, 1.0703125 to 0.173828125, 0.6953125,
0.9892578125. The interference-minus-quiet latency summary, in
microseconds, is min 183, p50 376, p95 857, p99 857, and max 857.

| Counter | Quiet | Interference |
| --- | ---: | ---: |
| maintenance finished | 1 | 1 |
| alerts finished | 0 | 1 |
| logout_ssf finished | 35 | 35 |
| mail finished | 14 | 14 |
| reconciliation finished | 14 | 14 |
| provisioning finished | 286 | 286 |
| deactivation finished | 286 | 286 |
| manual_connector finished | 0 | 0 |
| requests_total | 13 | 20 |
| responses_error_total | 0 | 2 |
| rate_limited_total | 0 | 0 |
| worker_rejections_total | 0 | 0 |
| cleanup_count | 1 | 1 |
| cleanup_errors | 0 | 0 |
| write_hold_count | 335 | 339 |
| write_wait_count | 335 | 339 |

The JSON file is
`/Users/dominik/.cache/riauth-cargo/q09-linux-arm64-6ca4779/reports/essentials-redb-maintenance.json`,
SHA-256 `3e4ab3f00a8ffa4c85ad9127ca276790c920d6abcdc3d7391deea5b6c2b2625e`.
The sibling `.stdout.json` is byte-identical, so it has the same hash. A
text search found no password and no session token.

#### Essentials PostgreSQL

The PostgreSQL command started at 2026-09-29T19:03:15Z and exited 0 at
2026-09-29T19:05:40Z. Standard error recorded the same completion line with
`backend=postgresql edition=essentials` and both maintenance overlaps true.
Runtime `storage_backend` is `postgresql`. The report hardware load average,
taken before the measured passes, was 0.173828125, 0.6953125, 0.9892578125.

| Pass | p50 µs | p95 µs | p99 µs | min µs | elapsed s | success/s | RSS KiB min | RSS KiB max | CPU s | samples |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Quiet | 5746 | 23371 | 23371 | 1833 | 71.600958 | 0.167596 | 48640 | 49056 | 0.0 | 1299 |
| Interference | 5680 | 11581 | 11581 | 3270 | 71.597363 | 0.167604 | 49168 | 49320 | 1.0 | 1283 |

Quiet host load went from 0.15966796875, 0.68359375, 0.98388671875 to
0.44091796875, 0.6611328125, 0.95556640625. Interference host load went from
0.44091796875, 0.6611328125, 0.95556640625 to 0.2353515625, 0.55029296875,
0.89404296875. The interference-minus-quiet latency summary, in
microseconds, is min 1437, p50 -66, p95 -11790, p99 -11790, and max -11790.

| Counter | Quiet | Interference |
| --- | ---: | ---: |
| maintenance finished | 1 | 1 |
| alerts finished | 0 | 1 |
| logout_ssf finished | 35 | 35 |
| mail finished | 14 | 14 |
| reconciliation finished | 14 | 14 |
| provisioning finished | 286 | 287 |
| deactivation finished | 286 | 287 |
| manual_connector finished | 0 | 0 |
| requests_total | 13 | 20 |
| responses_error_total | 0 | 2 |
| rate_limited_total | 0 | 0 |
| worker_rejections_total | 0 | 0 |
| cleanup_count | 1 | 1 |
| cleanup_errors | 0 | 0 |
| write_hold_count | 348 | 360 |
| write_wait_count | 348 | 360 |

The JSON file is
`/Users/dominik/.cache/riauth-cargo/q09-linux-arm64-6ca4779/reports/essentials-postgresql-maintenance.json`,
SHA-256 `0e6f24f6a2b9029dd409aad95457d81b407a843f4e72632d67b3d92c84e9c789`.
The sibling `.stdout.json` is byte-identical. A text search found no password
and no session token.

#### Platform redb

The redb command started at 2026-09-29T19:05:40Z and exited 0 at
2026-09-29T19:08:03Z. Standard error recorded the same completion line with
`backend=redb edition=platform` and both maintenance overlaps true. Runtime
`storage_backend` is `redb` and edition is `platform`. The report hardware
load average, taken before the measured passes, was 0.2353515625,
0.55029296875, 0.89404296875.

| Pass | p50 µs | p95 µs | p99 µs | min µs | elapsed s | success/s | RSS KiB min | RSS KiB max | CPU s | samples |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Quiet | 971 | 2053 | 2053 | 354 | 71.539595 | 0.167739 | 52740 | 53368 | 0.0 | 1282 |
| Interference | 1036 | 3062 | 3062 | 511 | 71.53881 | 0.167741 | 53464 | 53524 | 1.0 | 1283 |

Quiet host load went from 0.2353515625, 0.55029296875, 0.89404296875 to
0.4208984375, 0.50634765625, 0.8486328125. Interference host load went from
0.4208984375, 0.50634765625, 0.8486328125 to 0.3212890625, 0.44580078125,
0.80078125. The interference-minus-quiet latency summary, in microseconds,
is min 157, p50 65, p95 1009, p99 1009, and max 1009.

| Counter | Quiet | Interference |
| --- | ---: | ---: |
| maintenance finished | 1 | 1 |
| alerts finished | 0 | 0 |
| logout_ssf finished | 35 | 36 |
| mail finished | 14 | 14 |
| reconciliation finished | 14 | 14 |
| provisioning finished | 287 | 286 |
| deactivation finished | 287 | 286 |
| manual_connector finished | 0 | 0 |
| requests_total | 13 | 20 |
| responses_error_total | 0 | 2 |
| rate_limited_total | 0 | 0 |
| worker_rejections_total | 0 | 0 |
| cleanup_count | 1 | 1 |
| cleanup_errors | 0 | 0 |
| write_hold_count | 341 | 344 |
| write_wait_count | 341 | 344 |

The JSON file is
`/Users/dominik/.cache/riauth-cargo/q09-linux-arm64-6ca4779/reports/platform-redb-maintenance.json`,
SHA-256 `e1d5105355e551fac587c99e42c2f2e14791223f5bb6bd88bec87e30a62a4865`.
The sibling `.stdout.json` is byte-identical. A text search found no password
and no session token.

#### Platform PostgreSQL

The PostgreSQL command started at 2026-09-29T19:08:03Z and exited 0 at
2026-09-29T19:10:31Z. Standard error recorded the same completion line with
`backend=postgresql edition=platform` and both maintenance overlaps true.
Runtime `storage_backend` is `postgresql`. The report hardware load average,
taken before the measured passes, was 0.3212890625, 0.44580078125,
0.80078125.

| Pass | p50 µs | p95 µs | p99 µs | min µs | elapsed s | success/s | RSS KiB min | RSS KiB max | CPU s | samples |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Quiet | 6138 | 9809 | 9809 | 2947 | 71.60338 | 0.16759 | 53232 | 53700 | 0.0 | 1288 |
| Interference | 6617 | 13722 | 13722 | 2313 | 71.607087 | 0.167581 | 53704 | 53880 | 1.0 | 1287 |

Quiet host load went from 0.5361328125, 0.48828125, 0.81298828125 to
0.2880859375, 0.42724609375, 0.76904296875. Interference host load went from
0.2880859375, 0.42724609375, 0.76904296875 to 0.13330078125, 0.3447265625,
0.7109375. The interference-minus-quiet latency summary, in microseconds, is
min -634, p50 479, p95 3913, p99 3913, and max 3913.

| Counter | Quiet | Interference |
| --- | ---: | ---: |
| maintenance finished | 1 | 1 |
| alerts finished | 0 | 1 |
| logout_ssf finished | 35 | 35 |
| mail finished | 14 | 14 |
| reconciliation finished | 14 | 14 |
| provisioning finished | 286 | 287 |
| deactivation finished | 286 | 287 |
| manual_connector finished | 0 | 0 |
| requests_total | 13 | 20 |
| responses_error_total | 0 | 2 |
| rate_limited_total | 0 | 0 |
| worker_rejections_total | 0 | 0 |
| cleanup_count | 1 | 1 |
| cleanup_errors | 0 | 0 |
| write_hold_count | 353 | 365 |
| write_wait_count | 353 | 365 |

The JSON file is
`/Users/dominik/.cache/riauth-cargo/q09-linux-arm64-6ca4779/reports/platform-postgresql-maintenance.json`,
SHA-256 `ee87cfcdd28ae69e5ea65255bdb1a9f460d453a6ef2225578f49c1af2a969a02`.
The sibling `.stdout.json` is byte-identical. A text search found no password
and no session token.

Available space on `/System/Volumes/Data` was 68039628 KiB at
2026-09-29T19:00:51Z and 61793948 KiB when the container exited at
2026-09-29T19:10:32Z. Both readings are above 8388608 KiB. The named container
`q09-linux-arm64-cc186af` was removed. No `riauth-q09-` directory remained.
The macOS Platform copy, the macOS Essentials copy, and their four maintenance
reports still have the hashes recorded above. This session did not edit the
accepted worktree or main.

The limits of these observations are the Docker Desktop linuxkit VM, the
release profile of the `6ca4779` binaries, trust authentication with
`sslmode=disable`, loopback HTTP, whole-second procps CPU samples, the load
averages above, and the pace inside the throughput. They assign no load,
recovery, or capacity target.

### Paced `q09-2u-2g` Linux ARM64 secure mode in Docker's VM

These four observations repeat that paced shape with `--secure`, in the same
order: Essentials redb, Essentials PostgreSQL, Platform redb, then Platform
PostgreSQL. Each report has `source.commit`
`4ab934a44a80062dda0dba5d8640d396631a3ef0`, empty `dirty_paths`,
`product_run: true`, `observations_only: true`, and `performance_claim:
false`. Schema is `riauth.benchmark-slice/v1`. The script SHA-256 is
`399bd242519bdae15483f2d450ef9c1bf7b8b397a25bfff1179abefcaf9c7e49`.
`source.binary_compiler` is `unrecorded`. `source.checkout_rust_toolchain`
is `1.98.1`. That commit is the harness checkout where the script ran. The
plaintext rows above store script SHA-256
`ecd4ce799087ed095c6604a92a18c2af4f02af33adfa7ac36907184bf4d5c510` at
checkout `cc186af`.

The measured files are the same private copies named above, still mode
`-r-xr-xr-x`. Essentials SHA-256
`5feac36b7ad5d292cafb192b673f4353f8fa424e90a35d667d8fa7f41cac74eb`, 36324648
bytes. Platform SHA-256
`606a2ce5d72de16fe8749efdb3504fd81471c88118008180667a013ccffea79d`, 49302784
bytes. Both were compiled from
`6ca4779b68cf41e29af490d2aca6ea3d59e1f2bd`. Report `build` tokens are
`features` `essentials` or `platform`, `profile` `release`, and `toolchain`
`1.98.1`. Capabilities `build_features` are `essentials` on the first binary
and `essentials`, `platform` on the second. Both report version `0.1.1` and
target `linux` / `aarch64`. `artifact.path` is `/evidence/bin/essentials/riauth`
or `/evidence/bin/platform/riauth`. `Cargo.toml`, `Cargo.lock`, and
`rust-toolchain.toml` match `6ca4779`. The SAML record move named above is
still the only `src` difference from that commit, so these binaries predate
it. These rows are a separate observation from the plaintext Linux rows: a
later harness, private-CA HTTPS, and `aes256gcm-v1` storage.

The image is the same local tag `q09-linux-arm64-runtime:cc186af`, id
`sha256:a04d38b65247359a288a139686b942190737eca25ef1ccf504b93c7a532fb4fd`,
linux/arm64. The container name was `q09-linux-secure-arm64`, label
`q09=linux-secure-arm64`, user `10000:10000`. It set `safe.directory=*` and
mounted the worktree read-only, the binaries read-only, and the report
directory writable. Process uname was `Linux 273a73bfc263 7.0.12-linuxkit #1
SMP PREEMPT Fri Aug 14 16:27:59 UTC 2026 aarch64`. Python was 3.13.5 and
`cpu_count` was 16. MemTotal and cgroup `memory.max` are unrecorded for this
container. The Mac host in the launcher log is Darwin `dgsPro` 25.2.0 arm64
and `hw.memsize` 68719476736. The linuxkit kernel is Docker Desktop's Linux
VM. The report limitations record no external signer and no published release
identity.

Every run used a new directory and `--iterations 12 --warmup 1
--interference-cap 4 --directory-users 2 --directory-groups 2 --pace-ms 6500
--secure`. The verified dataset is `q09-2u-2g`: administrator `admin`, extra
users `q09-user-0001` and `q09-user-0002`, groups `q09-dir-0001` and
`q09-dir-0002`, three members in each group, and six memberships. The
estimated general-request budget is 65. Settings record `tls: true`,
`database_encryption: true`, issuer `loopback https base`, and listen
`127.0.0.1 ephemeral`. `postgres_local_unencrypted` is false. PostgreSQL
pool size is 8. The redb reports leave the pool null.

Each run generated a fresh private CA, a server certificate, and a fresh
`database_key_file`. The certificate SAN is `DNS:localhost` and
`IP:127.0.0.1`, and the issuer host is `127.0.0.1` on the listener port. The
client checks the hostname against that private CA. The harness records
`matching_ca_readyz` true only after `/readyz` returned HTTP 200 to that CA.
An unrelated CA and the system trust store were refused before an HTTP
status. `operations.native_tls` and `operations.encrypted_storage` are
usable. The key file is mode `0600`, base64url without padding, 32 decoded
bytes. Storage format is `aes256gcm-v1`. `credential_bytes_in_store` is
false.

The connection file this binary reads accepts `sslmode` values `disable`,
`prefer`, and `require`, and the client then sets `SslMode::Require`. rustls
checks the hostname against the webpki roots and `ca_file`.
`sslmode=verify-full` in that file is rejected before connect, so the harness
keeps `sslmode=require` there, with `host=localhost` and
`hostaddr=127.0.0.1`. libpq preflight is the path that uses
`sslmode=verify-full` and the private CA. That limit was in the harness
before this sequence, and both PostgreSQL reports record it. Each PostgreSQL
report has `operations.postgresql` usable, `server_ssl` true, auth
`scram-sha-256`, `pg_hba` `hostssl 127.0.0.1/32` and `::1/128`,
`local_unencrypted` false, `libpq_sslmode_verify_full` true,
`libpq_unrelated_ca_refused` true, and `libpq_sslmode_disable_refused` true.
Product init against the unrelated CA exited 6 with `storage_unavailable`,
HTTP 503, and `config_published` false. The redb reports have no `postgres`
object and say this run uses the redb backend.

The 6500 ms pace sits between measured reads, inside the pass wall clock and
outside each latency sample. For 12 attempts, nearest-rank `ceil(n*q)`
selects the maximum for both p95 and p99. Every pass had 12 attempts, 12
successes, 0 session-read errors, empty error statuses and codes, maintenance
`finished` delta 1, `failed` 0, and `maintenance_cadence_overlap` true.
Elapsed time is 71.574398 to 71.657009 seconds. The writer on every run
recorded 4 posts, 2 successes, 2 HTTP 409 conflicts, 0 other errors, 2
revision refreshes, and overlap true. Quiet `requests_total` is 13 and
interference `requests_total` is 20. Interference `responses_error_total` is
2. `rate_limited_total` and `worker_rejections_total` are 0. `cleanup_count`
is 1 and `cleanup_errors` is 0. Reconciliation `finished` is 14.
`manual_connector` finished is 0. Logout/SSF `finished` is 35 on each quiet
pass and 36 on each interference pass. Mail `finished` is 13 on the
Essentials redb, Essentials PostgreSQL, and Platform redb quiet passes, and
14 on the Platform PostgreSQL quiet pass and on every interference pass.
Every background `failed` delta is 0. On every pass, RSS at the start equals
the minimum and RSS at the end equals the maximum. Sampler read errors are 0.
procps in this image prints CPU time in whole seconds, so the samples are 0.0
or 1.0. Platform PostgreSQL quiet CPU is 1.0 and its interference CPU is 0.0.
The other three runs record quiet CPU 0.0 and interference CPU 1.0. The
one-minute load average values in these four reports span 0.18115234375 to
1.3203125.

The inner log is
`/Users/dominik/.cache/riauth-cargo/q09-linux-arm64-secure/reports/run.log`,
SHA-256
`655c333863c30ed0173c611d88425e818f67477ba0afed728f4b8be99bc87945`.
It ends `RUN_OK`. The host launcher log is
`/Users/dominik/.grok/long-running-background-tasks/q09-linux-secure-run.log`,
SHA-256
`ceb7aa3e276e611d13925490db02ae984cf80d4da57aff83db15fc46b88bef36`.
It records `DOCKER_STATUS 0` and `RUN_OK`.

#### Essentials redb

The redb command started at 2026-09-29T19:45:51Z and exited 0 at
2026-09-29T19:48:15Z. Standard error recorded
`completion=0 product_run=true backend=redb edition=essentials dataset=q09-2u-2g directory_users=2 directory_groups=2 verified=true pace_ms=6500 secure=true quiet_maintenance_overlap=true interference_maintenance_overlap=true`.
Runtime `storage_backend` is `redb`. The report hardware load average, taken
before the measured passes, was 1.3203125, 1.1396484375, 1.00732421875.

| Pass | p50 µs | p95 µs | p99 µs | min µs | elapsed s | success/s | RSS KiB min | RSS KiB max | CPU s | samples |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Quiet | 4491 | 11331 | 11331 | 2339 | 71.591694 | 0.167617 | 49940 | 50408 | 0.0 | 1288 |
| Interference | 4205 | 9416 | 9416 | 2325 | 71.578967 | 0.167647 | 50504 | 50540 | 1.0 | 1292 |

Quiet host load went from 1.29443359375, 1.13720703125, 1.0068359375 to
0.85009765625, 1.0439453125, 0.9833984375. Interference host load went from
0.85009765625, 1.0439453125, 0.9833984375 to 0.55419921875, 0.92529296875,
0.947265625. The interference-minus-quiet latency summary, in microseconds,
is min -14, p50 -286, p95 -1915, p99 -1915, and max -1915.

| Counter | Quiet | Interference |
| --- | ---: | ---: |
| maintenance finished | 1 | 1 |
| alerts finished | 1 | 0 |
| logout_ssf finished | 35 | 36 |
| mail finished | 13 | 14 |
| reconciliation finished | 14 | 14 |
| provisioning finished | 287 | 286 |
| deactivation finished | 287 | 286 |
| manual_connector finished | 0 | 0 |
| requests_total | 13 | 20 |
| responses_error_total | 0 | 2 |
| rate_limited_total | 0 | 0 |
| worker_rejections_total | 0 | 0 |
| cleanup_count | 1 | 1 |
| cleanup_errors | 0 | 0 |
| write_hold_count | 336 | 339 |
| write_wait_count | 336 | 339 |

The JSON file is
`/Users/dominik/.cache/riauth-cargo/q09-linux-arm64-secure/reports/essentials-redb-secure.json`,
SHA-256 `cc8c417c1e965e0caee60e8b9b9aa42b25186c5816d6f2644d2d09abba8ef3f2`.
The sibling `.stdout.json` is byte-identical. A text search found no
password, no session token, and no PEM private-key banner.

#### Essentials PostgreSQL

The PostgreSQL command started at 2026-09-29T19:48:15Z and exited 0 at
2026-09-29T19:50:41Z. Standard error recorded the same completion line with
`backend=postgresql edition=essentials secure=true` and both maintenance
overlaps true. Runtime `storage_backend` is `postgresql`. The report hardware
load average, taken before the measured passes, was 0.55419921875,
0.92529296875, 0.947265625.

| Pass | p50 µs | p95 µs | p99 µs | min µs | elapsed s | success/s | RSS KiB min | RSS KiB max | CPU s | samples |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Quiet | 9683 | 14087 | 14087 | 3445 | 71.633726 | 0.167519 | 50752 | 51164 | 0.0 | 1284 |
| Interference | 9125 | 11152 | 11152 | 5106 | 71.626331 | 0.167536 | 51272 | 51452 | 1.0 | 1279 |

Quiet host load went from 0.59033203125, 0.9267578125, 0.94775390625 to
0.18115234375, 0.73046875, 0.87548828125. Interference host load went from
0.18115234375, 0.73046875, 0.87548828125 to 0.57080078125, 0.71484375,
0.857421875. The interference-minus-quiet latency summary, in microseconds,
is min 1661, p50 -558, p95 -2935, p99 -2935, and max -2935.

| Counter | Quiet | Interference |
| --- | ---: | ---: |
| maintenance finished | 1 | 1 |
| alerts finished | 1 | 0 |
| logout_ssf finished | 35 | 36 |
| mail finished | 13 | 14 |
| reconciliation finished | 14 | 14 |
| provisioning finished | 287 | 286 |
| deactivation finished | 287 | 286 |
| manual_connector finished | 0 | 0 |
| requests_total | 13 | 20 |
| responses_error_total | 0 | 2 |
| rate_limited_total | 0 | 0 |
| worker_rejections_total | 0 | 0 |
| cleanup_count | 1 | 1 |
| cleanup_errors | 0 | 0 |
| write_hold_count | 349 | 359 |
| write_wait_count | 349 | 359 |

The JSON file is
`/Users/dominik/.cache/riauth-cargo/q09-linux-arm64-secure/reports/essentials-postgresql-secure.json`,
SHA-256 `f6fd102c82108288299b655932dbaff8d9d80b6db26a148ac01433468a3157dd`.
The sibling `.stdout.json` is byte-identical. A text search found no
password, no session token, and no PEM private-key banner.

#### Platform redb

The redb command started at 2026-09-29T19:50:41Z and exited 0 at
2026-09-29T19:53:05Z. Standard error recorded the same completion line with
`backend=redb edition=platform secure=true` and both maintenance overlaps
true. Runtime `storage_backend` is `redb` and edition is `platform`. The
report hardware load average, taken before the measured passes, was
0.57080078125, 0.71484375, 0.857421875.

| Pass | p50 µs | p95 µs | p99 µs | min µs | elapsed s | success/s | RSS KiB min | RSS KiB max | CPU s | samples |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Quiet | 5965 | 10922 | 10922 | 1912 | 71.598955 | 0.1676 | 53968 | 54516 | 0.0 | 1275 |
| Interference | 3746 | 6696 | 6696 | 2021 | 71.574398 | 0.167658 | 54700 | 54716 | 1.0 | 1285 |

Quiet host load went from 0.765625, 0.7529296875, 0.869140625 to
0.30517578125, 0.62109375, 0.8125. Interference host load went from
0.30517578125, 0.62109375, 0.8125 to 0.529296875, 0.61181640625,
0.79541015625. The interference-minus-quiet latency summary, in
microseconds, is min 109, p50 -2219, p95 -4226, p99 -4226, and max -4226.

| Counter | Quiet | Interference |
| --- | ---: | ---: |
| maintenance finished | 1 | 1 |
| alerts finished | 1 | 0 |
| logout_ssf finished | 35 | 36 |
| mail finished | 13 | 14 |
| reconciliation finished | 14 | 14 |
| provisioning finished | 286 | 286 |
| deactivation finished | 286 | 286 |
| manual_connector finished | 0 | 0 |
| requests_total | 13 | 20 |
| responses_error_total | 0 | 2 |
| rate_limited_total | 0 | 0 |
| worker_rejections_total | 0 | 0 |
| cleanup_count | 1 | 1 |
| cleanup_errors | 0 | 0 |
| write_hold_count | 340 | 344 |
| write_wait_count | 340 | 344 |

The JSON file is
`/Users/dominik/.cache/riauth-cargo/q09-linux-arm64-secure/reports/platform-redb-secure.json`,
SHA-256 `e88bba86b5a11a5c68bd6235b833461198d52f474de7092fda3c2bc55351be6a`.
The sibling `.stdout.json` is byte-identical. A text search found no
password, no session token, and no PEM private-key banner.

#### Platform PostgreSQL

The PostgreSQL command started at 2026-09-29T19:53:05Z and exited 0 at
2026-09-29T19:55:31Z. Standard error recorded the same completion line with
`backend=postgresql edition=platform secure=true` and both maintenance
overlaps true. Runtime `storage_backend` is `postgresql` and edition is
`platform`. The report hardware load average, taken before the measured
passes, was 0.529296875, 0.61181640625, 0.79541015625.

| Pass | p50 µs | p95 µs | p99 µs | min µs | elapsed s | success/s | RSS KiB min | RSS KiB max | CPU s | samples |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Quiet | 10462 | 19071 | 19071 | 3773 | 71.657009 | 0.167464 | 55260 | 55680 | 1.0 | 1273 |
| Interference | 9996 | 11876 | 11876 | 5059 | 71.640703 | 0.167503 | 55808 | 55992 | 0.0 | 1283 |

Quiet host load went from 0.7275390625, 0.65185546875, 0.8076171875 to
0.28662109375, 0.529296875, 0.751953125. Interference host load went from
0.28662109375, 0.529296875, 0.751953125 to 0.36669921875, 0.49365234375,
0.72216796875. The interference-minus-quiet latency summary, in
microseconds, is min 1286, p50 -466, p95 -7195, p99 -7195, and max -7195.

| Counter | Quiet | Interference |
| --- | ---: | ---: |
| maintenance finished | 1 | 1 |
| alerts finished | 0 | 1 |
| logout_ssf finished | 35 | 36 |
| mail finished | 14 | 14 |
| reconciliation finished | 14 | 14 |
| provisioning finished | 287 | 287 |
| deactivation finished | 287 | 287 |
| manual_connector finished | 0 | 0 |
| requests_total | 13 | 20 |
| responses_error_total | 0 | 2 |
| rate_limited_total | 0 | 0 |
| worker_rejections_total | 0 | 0 |
| cleanup_count | 1 | 1 |
| cleanup_errors | 0 | 0 |
| write_hold_count | 354 | 365 |
| write_wait_count | 354 | 365 |

The JSON file is
`/Users/dominik/.cache/riauth-cargo/q09-linux-arm64-secure/reports/platform-postgresql-secure.json`,
SHA-256 `b8946dec02ebb45aa06a9e7dd7b60c770117621a3c614d2f5d62fa1acfada961`.
The sibling `.stdout.json` is byte-identical. A text search found no
password, no session token, and no PEM private-key banner.

Available space on `/System/Volumes/Data` was 43990744 KiB at
2026-09-29T19:45:50Z and 33787468 KiB when the container exited at
2026-09-29T19:55:31Z. Both readings are above 8388608 KiB. The named container
`q09-linux-secure-arm64` was removed. No `riauth-q09-` directory remained.
The plaintext JSON files named above still hash to the SHA-256 values in that
section. The private binary copies still hash to the values above. This
session did not run Cargo, did not edit the Q08 tree, and did not edit the
accepted worktree or main.

The limits of these observations are Docker's linuxkit VM, the release
profile of the `6ca4779` binaries, loopback HTTPS with one fresh private CA,
`aes256gcm-v1` from a fresh `database_key_file`, PostgreSQL `scram-sha-256`
on `hostssl` with the product file at `sslmode=require` and libpq preflight
at `sslmode=verify-full`, whole-second procps CPU samples, the load averages
above, and the pace inside the throughput. They assign no load, recovery, or
capacity target. They record no external signer and no published release
identity.

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
binary hash to the build command, the dev profile, and rustc 1.98.1. The
paced Platform `q09-2u-2g` runs on redb and PostgreSQL record
maintenance-overlap observations of the `c7a61c6` copy. The paced Essentials
`q09-2u-2g` runs on redb and PostgreSQL record that overlap for the `6ca4779`
dev-profile binary. The paced Linux ARM64 rows record that overlap for both
editions and both backends inside Docker Desktop's linuxkit VM. Their
`source.commit` is harness checkout `cc186af`. The measured files are the
release-profile `aarch64-unknown-linux-gnu` copies Q08 built from `6ca4779`.
The `--secure` Linux ARM64 rows record the same overlap on those copies with
private-CA HTTPS and `aes256gcm-v1` storage. Their `source.commit` is harness
checkout `4ab934a`. The PostgreSQL rows in that set record libpq
`sslmode=verify-full`, and the connection file the binary reads uses
`sslmode=require`.

Full publication still requires:

- a Linux x86-64 packaged artifact when a release build is the claimed artifact, and a published Linux ARM64 package identity for that release; the ARM64 rows on this page are the local VM copies above
- an external signer when the claimed deployment uses one; these `--secure` rows record a fresh loopback private CA and a fresh `database_key_file`, and they record no external signer and no published release identity
- a repeat on a host whose load is the condition being studied, or on a quiet host when that is the condition; the plaintext Linux ARM64 container passes recorded a one-minute load average from 0.13330078125 to 0.6689453125 beside MemTotal 8021540 kB, the secure Linux ARM64 passes recorded a one-minute load average from 0.18115234375 to 1.3203125, the paced Essentials macOS passes recorded 10.814453125 to 16.57177734375, the paced Platform PostgreSQL passes recorded 6.0830078125 to 12.5654296875, the paced Platform redb passes recorded 6.84033203125 to 7.99267578125, and the `q09-8u-8g` runs recorded 10.4560546875 to 11.3759765625, on 16 cores
- no numeric load, RPO, or RTO target until a named operator workload exists

The coverage inventory row for Q09 still says the benchmark gap is open. This
slice does not change that row.
