# O06 readiness-cause signal proposal and implementation evidence

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original O06 task
`9899f8e6-05ff-4e11-b9a0-b9ca184221a6`, owned by diagnostics worktree
`f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`. Read-only support investigation in
existing worktree `7c85f5ef-3fac-4f72-aaed-08474d7fb454`, on 2026-10-02.
M03 remains DONE. Root reviews/integrates the received controller slice.

**Implemented under root's exact reservation; the focused target is uncompiled
and unrun, awaiting root's Cargo release.** Code/test commit is `b7bc6ef`.
Accepted proposal checkpoint `b76578e` remains historical evidence; actual
implementation/static evidence is recorded below in a separate report commit.

## Immutable source and verified gap

The original readiness/storage observations use fixed published main
`755a7763e0e2aa7e4d5c92d18c432ebc0c8a3e87` through Git object reads. The own
branch was at runtime-evidence commit `27b04e7` when the proposal was made.
Own App/probes inputs matched these fixed source blobs exactly before edits;
no alignment merge or reset was required for this implementation.

| Fixed source | Git blob | Relevant span |
| --- | --- | --- |
| `src/api/probes.rs` | `790e64f90f0c671ec9f414b18aca56627b3b5b92` | `:39-61`, all readiness decisions |
| `src/api.rs` | `15c8495cd6ddc5df3cccad26ddae147f6aa96415` | `:51-80`, App fields/constructor; `:239-252` and `:623-628`, probe routes |
| `src/api/observability.rs` | `f5e38de085fac1e42db0c57a07f43e0975a88d95` | `:4-24`, Stats; `:126-171`, response counts; `:362-394`, authenticated JSON metrics |
| `src/store.rs` | `8e78d889220c9e956fe00a4f7a69b603063d3ebf` | `:722-735`, Store readiness |
| `src/upgrade/activation.rs` | `c8295c4e103d4dca2ceb358ff4454631d08fbda4` | `:50-65`, fixed activation decode errors; `:168-186`, active fence |
| `src/recovery.rs` | `99718c37c4ec85f8e31ffced5bfb1722427b247d` | `:469-486`, serving/recovery fence |
| `src/postgres_store.rs` | `e3862a96a3fdd509e765b5f72d648a77b336fb24` | `:125-137`, unavailable code; `:166-218`, pool checkout/busy code |
| `src/error.rs` | `5154328d29ee832ddc1ea24f029bfe47028e6211` | `:54-61`, internal-error redaction and existing logging |

`ready` first rejects a saturated application-worker pool only for a role with
the authentication duty. It then tries one of two probe permits, moves the
permit into a blocking storage check, and applies the existing two-second
timeout. Permit failure, timeout, join failure and Store failure all become
the same `unavailable(authentication)` response. There is no retained cause
or readiness-specific event. `/livez` reads static process metadata and does
not perform these checks. `/healthz` uses the same readiness handler.

The public failure remains HTTP 503, code `not_ready`, and one of these two
existing generic descriptions:

- Authentication duty: `Service is not ready to accept authentication requests`.
- Other roles: `Worker storage is not ready`.

The existing success JSON includes issuer only for the authentication duty.
Probe routes remain public; authentication/administration routes remain
unmounted on worker processes. These contracts need no change.

The premise that nothing logs needs a precise qualification. The readiness
handler itself emits nothing, but `Error::internal` logs its input **before**
replacing it with generic `server_error`. PostgreSQL's unavailable mapper
already logs SQLSTATE. For example, the writable check calls
`Error::internal("Storage is not writable")`, so that specific distinction is
already lost by the time the handler sees the Error. The new signal must not
guess it from `server_error`. Existing lower-level logging is outside this
proposal's suppression state and can still repeat.

Stats currently records response classes, latency, admission and work, but
contains no readiness category. Its authenticated metrics handler requires
foreground admission and a Store authorization read; a failing Store or
saturated worker pool can prevent that read. Worker-only routers do not mount
operator metrics. The existing tracing subscriber in `src/cli.rs:1372-1379`
writes to stderr with the default `riauth=info` filter. A fixed operator event
therefore uses the existing restricted log channel across process roles.

## Exact approved reservation

Root approved exactly proposal `b76578e113dc3b12e6c7b1ab10c85c6663b82e63`
and recorded `wave30_O06_readiness_cause` for shell
`7a798ec7-bad2-4d52-8bea-afbe1f8e8d2a` in this worktree. The ledger was read
before edits: approved true, runtime_released false. This lane never edits it.
Only these four files are reserved, with code/test and evidence separate:

1. `src/api.rs`: one private shared field
   `readiness: Arc<probes::ReadinessSignal>` alongside `probes`, and one
   `App::new` initialization. One guarded, doc-hidden `test-support` re-export
   of the focused probe harness described below. No Stats, route, admission,
   request, credential, authentication or server hunks.
2. `src/api/probes.rs`: a private finite cause enum, exact Store-error
   classifier, tiny shared signal state and emission helper; the readiness
   decision hunk only. Extract the existing decision body into a private
   runner accepting `FnOnce(&Core) -> Result<()> + Send + 'static`, so the
   production handler supplies exactly `core.store.ready()` and a test can
   deterministically exercise timeout/join branches. Keep `live`,
   `unavailable`, `probe_ok` and the existing CI-owned test module unchanged.
   Add only the guarded test harness at module scope.
3. New `tests/o06_readiness_cause_signal.rs`: ONE focused test function
   `readiness_causes_are_redacted_bounded_observations` and local capture/latch
   helpers. This target is absent from fixed main. No existing test or common
   fixture edits.
4. This proposal/evidence report, with later actual execution recorded
   separately from the proposal checkpoint.

The App field is preferable to a Stats field: it requires no edit to the
concurrently owned `src/api/observability.rs`, no authenticated metric read,
and no additional operator endpoint. No new dependency, feature, config,
Store/schema, policy, telemetry counter, permission or resource is required.

## Fixed fields and bounded storage whitelist

Emit only the following seven structured fields, plus one fixed event message:

- `signal = "readiness_probe"`.
- `scope = "app_local_observation"`.
- `state = "not_ready"` or `"ready"`.
- `cause`: one finite label from the table below.
- `component`: one fixed label from the same table.
- `safe_state`: the fixed text from that row.
- `remedy`: the fixed text from that row.

Warn message: `Readiness failure observed`. Recovery info message:
`Readiness check recovered`. Emit with `parent: None` so inherited request-span
fields are not included. No Error, error code/message, panic value,
request/span identifier, URL, path, database identifier, key, schema value,
agent, credential or configuration value is copied into the event. The
existing tracing formatter can supply its ordinary timestamp/level/target.

| Cause | Component | Fixed safe_state | Fixed remedy |
| --- | --- | --- | --- |
| `application_workers_saturated` | `application_workers` | Readiness refuses authentication traffic; this probe starts no storage check and does not cancel existing work. | Inspect foreground workload and worker latency; reduce load and retry when capacity is available. |
| `probe_capacity_exhausted` | `readiness_probe_permits` | This probe starts no new storage check; checks already running retain their permits. | Wait for outstanding checks; inspect storage latency if probe capacity remains occupied. |
| `storage_probe_timeout` | `storage_readiness_probe` | Storage readiness is unknown; the blocking check may still run and retains its permit until it ends. | Inspect storage connectivity and pool delays; retry after the outstanding check completes. |
| `storage_probe_join_failed` | `storage_readiness_probe` | The check failed to return normally; usable storage has not been confirmed. | Inspect process and runtime stability and retry the readiness check. |
| `storage_pool_busy` | `postgres_connection_pool` | Readiness is unconfirmed; existing work is not cancelled by this probe. | Inspect database pool occupancy and workload; retry after connections become available. |
| `storage_unavailable` | `storage_backend` | Storage access failed; this observation establishes no specific network, credential or peer cause. | Inspect database connectivity and availability through existing authorized operator procedures. |
| `storage_format_not_ready` | `storage_compatibility_gate` | The compatibility fence remains in force; this probe performs no migration. | Stop incompatible writers and follow the backed-up offline procedure with a compatible release. |
| `storage_activation_not_ready` | `storage_activation_gate` | This process cannot confirm the active storage contract; the serving fence remains in force. | Stop incompatible writers and follow the reviewed activation or recovery procedure with the compatible release. |
| `storage_recovery_required` | `storage_recovery_gate` | The restored-state gate remains in force; this check does not attest reconciled credentials. | Inspect recovery status and complete the existing operator reconciliation procedure before resuming serving. |
| `storage_lineage_changed` | `storage_recovery_gate` | The stored lineage fence refuses serving; this check does not apply recovery or establish freshness. | Follow the existing stopped-process recovery procedure before reopening and reconciling the store. |
| `storage_readiness_unknown` | `storage_readiness` | The storage check failed; no narrower cause or health conclusion is available. | Inspect restricted storage diagnostics and the existing compatibility and recovery procedures. |
| `readiness_check_passed` | `readiness_probe` | This probe completed its storage check and required capacity observation; continuous or remote health is not established. | Continue normal readiness monitoring. |

The five handler branches select their cause directly. Classify a Store Error
only inside the Store-result failure branch. Match the following bounded
status/code whitelist, with exact whole-string equality where listed. Output
the table's independent constants, never the matched message:

| Status and code | Source-identifiable input | Output cause |
| --- | --- | --- |
| 503 / `storage_busy` | Pool checkout's stable code | `storage_pool_busy` |
| 503 / `storage_unavailable` | PostgreSQL unavailable mapper's stable code | `storage_unavailable` |
| 400 / `invalid_request` | `Storage schema or index revision is not ready` | `storage_format_not_ready` |
| 400 / `invalid_request` | `Stored version activation is malformed; use a compatible release` | `storage_activation_not_ready` |
| 400 / `invalid_request` | `Stored version activation format requires a newer release` | `storage_activation_not_ready` |
| 400 / `invalid_request` | `Store has no version activation; reopen it with a compatible release` | `storage_activation_not_ready` |
| 400 / `invalid_request` | `Store activation differs from this process; stop incompatible writers and restart with the active release` | `storage_activation_not_ready` |
| 409 / `conflict` | `` Restored state requires reconciliation; run `riauth recovery status` `` | `storage_recovery_required` |
| 409 / `conflict` | `PostgreSQL storage lineage changed; reopen the store to apply the recovery policy` | `storage_lineage_changed` |
| Everything else | Including `server_error`, changed literals, malformed data or an arbitrary message/code | `storage_readiness_unknown` |

No substring/prefix matching, arbitrary code emission, dynamic error formatting
or extra storage read is proposed. The input literals above are source
constants, not live store contents. This whitelist will conservatively fall
back if a future release changes them. PostgreSQL-specific branches will have
synthetic Error evidence in this focused test, not a live database claim.

## Deterministic observation state

`ReadinessSignal` holds one small `Mutex<SignalState>` shared by App clones.
State consists solely of `last: Option<Cause>` (initially None) and
`seen_failures: u16` (initially zero), with one bit for each of the eleven
failure categories. No clock, task, history list, collector IO or persisted
state is added.

| Observation | State update | New event |
| --- | --- | --- |
| First success | last = passed; mask remains zero | None |
| Repeated success | No change | None |
| First failure, or a category not yet seen in this failure episode | last = cause; set its bit | One warn with that cause's fixed fields |
| Repeated failure category, including return to a previously seen category | last = cause; retain mask | None |
| Success after failure | last = passed; clear mask | One fixed recovery info |
| Next failure after that success | Start a new failure episode | One warn |

Serialize the state update and event emission in this one short critical
section, with no await, Store call or detached work under its lock. Recover a
poisoned lock without printing its contents. Concurrent outcomes are ordered
by the signal lock, as **observations**; the signal is not a process-wide
consensus or a guarantee of current health. A success that completes after an
earlier failure can produce the recovery observation. A timed-out detached
check's late completion produces no second observation.

Within an uninterrupted failing episode, at most eleven new cause events
are emitted. The mask also prevents timeout/capacity alternation from producing
a new warning on every probe. One recovery event closes that episode. Real
success/failure oscillation can create new episodes; no time-based rate-limit
or false stability claim is introduced. State resets when App is constructed
and is neither durable audit evidence nor exported public status.

## One focused verification target, awaiting runtime

The guarded, doc-hidden `ReadinessProbeTest` harness is re-exported only with
the existing `test-support` feature. It wraps an existing App clone and exposes
only: construction from App, cloned worker/probe semaphores for controlled
permit holds, the actual ready and live handlers' Value results, and a
`ready_with_check(FnOnce(&Core) -> Result<()> + Send + 'static)` call to the
same private runner. No timeout override or production injection state exists.
The timeout stays two seconds in both calls.

The single test will use the existing tracing-subscriber dependency with a
local collecting Layer, filtering only this fixed readiness event. It will
assert the seven-field contract and absence of synthetic secret/error/path/URL
values without dumping fixture snapshots or errors. Planned evidence:

- First successful check and repeated success emit nothing. Live calls emit
  nothing and do not change the observation state.
- Holding the real worker permits selects saturation only for authentication
  roles and starts no injected Store callback. Holding the probe permits
  selects capacity. Public handler status/code/descriptions remain exact.
- Exact source whitelist inputs select the bounded rows; unknown codes,
  wrong statuses, changed literals, generic internal errors and messages
  carrying synthetic private values select the fixed unknown row. Repeated
  and returning causes produce no duplicate event within the episode.
- A valid local redb fixture's compatibility fence is exercised through the
  actual Store check; snapshot equality across the probe is checked by changed
  keys only. The test does not run a migration or reload a policy.
- Two injected blocking checks signal entry through oneshots and wait on
  explicit release channels. Their real two-second timeouts leave both probe
  permits occupied; the next probe observes capacity. Releasing both checks
  and reacquiring the two permits proves completion without an arbitrary
  sleep. Late completions emit no recovery event. A subsequent successful
  probe emits exactly one recovery and resets the suppression mask.
- A synthetic panic in the injected blocking check selects join failure
  without copying its panic contents into the signal. Use a fixed nonprivate
  panic literal. A fresh failure after recovery emits again.
- A valid non-authentication role fixture ignores occupied application-worker
  permits, retains the worker-specific generic error body, and never adds
  issuer to readiness success. These are handler/fixture checks, not a
  service/browser/remote-peer coverage claim.

Queued command, **not executed or runtime-authorized**:

```sh
env CARGO_TARGET_DIR="$PWD/target/wave30-o06-readiness" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support --test o06_readiness_cause_signal readiness_causes_are_redacted_bounded_observations -- --exact --test-threads=1
```

Root alone allocates this runtime. Keep the private target in this existing
worktree and the 8 GiB free-space floor. No existing CI-owned target is queued.

## Historical proposal checks

Original O06 row/current ownership and local instructions were read. No local
or fixed-tree AGENTS.md was found. CONTRIBUTING's general full-check advice is
superseded here by the explicit read-only assignment and root-owned runtime.
Source and task/ownership reads establish this proposal; they establish no
runtime result. Fixed blob pins and target absence were verified. Exact
whitelist source literals, the twelve emitted cause rows, report-only staged
scope/whitespace, and `python3 scripts/check-docs.py` were checked successfully.

At proposal checkpoint `b76578e`, no Rust source/test, Cargo input,
observability, config, state, workflow,
diagnostic API, service, desktop, main, push, task/status or ownership-ledger
edit occurred. No other worker was contacted or created.

## Actual implementation and static evidence

Code/test commit: `b7bc6efe0f10419e419cdbc16e7d2792b72777cb`, exactly these
three files:

| File | Implemented blob |
| --- | --- |
| `src/api.rs` | `b2097b0aeeb5a5e12d6586628fe2d84a6dd9ed72` |
| `src/api/probes.rs` | `c1404fd4ada77604f74f4ddf1a5a19bafbcf712b` |
| `tests/o06_readiness_cause_signal.rs` | `03cec60161c51d95f7f5e3001e675cd2c0baa79e` |

The code implements the exact fixed fields/whitelist and observation state
above. The runner's production caller supplies `|core| core.store.ready()`.
The guarded harness wraps the same runner; no production selector, injected
check storage, timeout override, extra route or public cause body exists.
Late detached check completion returns only to the dropped join handle and
cannot change observation state. Existing lower-level logs retain their prior
behavior; the new signal does not infer unwritable storage from an internal
error or establish that all logging is suppressed/redacted.

The one focused function is present with these deterministic fixture oracles:
all sixteen whitelist/fallback inputs receive fresh episodes so suppression
cannot mask classifier errors; a complete failing episode observes all eleven
categories once, including returning categories, then resets on observed
success. A real redb compatibility refusal uses the actual production callback
and checks full stored snapshots by changed keys only. Two latch-controlled
blocking checks exercise the unchanged two-second deadline, retained permits,
capacity refusal, channel release and no late recovery. A fixed nonprivate
panic supplies join failure. A validated worker-role Core clone proves occupied
foreground permits do not prevent its Store callback, while generic worker
failure/success bodies and issuer omission remain exact. The collector filters
only the new event, checks its seven fields plus the fixed message, and rejects
inherited request spans or synthetic private/error material. These are
**unexecuted test assertions**, not credited product results.

Executed static checks, all passed:

- Native formatting/syntax check:
  `rustfmt --edition 2024 --config skip_children=true --check src/api.rs src/api/probes.rs tests/o06_readiness_cause_signal.rs`.
  The skip_children setting prevents formatting unreserved child modules.
- Removing the exact guarded export, private field and constructor addition
  reconstructs the entire fixed App source byte-for-byte. Its routes,
  authentication, admission, credentials, background and Stats code is untouched.
- The entire probes prefix before `ready` and the complete existing
  `#[cfg(test)] mod tests` tail are byte-identical to fixed main. This includes
  `live`, `unavailable`, `probe_ok` and the existing CI-owned test.
- Static assertions confirm the production Store callback, single real
  two-second timeout, permit binding inside the blocking task, guarded harness
  struct/impl/export, two explicit `parent: None` events, and exactly one new
  focused test function.
- All twelve four-field cause/component/safety/remedy tuples match the accepted
  proposal table exactly. The seven exact storage literals and two stable pool
  codes remain the bounded classifier input contracts.
- Code-only staged file scope and `git diff --cached --check` passed. Report
  documentation validation uses `python3 scripts/check-docs.py`; evidence is
  committed separately from code/test.

Native rustfmt verifies parsing and formatting, not type checking. No Cargo,
rustc, product test, new dependency/feature, database/remote service, browser or
desktop run occurred for this readiness slice. The source commit and these
scope proofs were sent through the explicit project orchestrator before any
runtime. The exact new command remains queued after gauge and pagination;
root's runtime_released value remains false.

## Remaining scope

Implementation and static scope evidence are complete within the reservation.
Compilation and the one focused test require root's runtime release; later
actual results/failures must be appended separately. Root review/integration
remains. O06 stays open and M03 stays done. No other worker/task/worktree was
created or contacted. No observability/metrics, Store/schema, policy, config,
state, workflow, permission/resource, main, push, service, status or ownership
record was edited. `Store::ready`, its activation/recovery checks, and their
reads remain unchanged. Runtime security-agreement/policy reload or all-setting
agreement is outside this observation seam.
Desktop preference remains RiWork Cua.ai Driver, descriptions/current state
first; no desktop interaction is needed.
