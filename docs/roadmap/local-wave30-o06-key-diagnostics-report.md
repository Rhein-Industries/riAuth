# O06 key-health availability diagnostic — wave 30

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; task `9899f8e6-05ff-4e11-b9a0-b9ca184221a6`; worktree `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`; date 2026-10-02. Implementation object: `320dd96bae47a47e46dd19660cffe4cd881f16d2`. Parent: storage implementation/test/report handoff `c63d40338f7584639f57e74d5a370748f45e195f`. Published assignment base: `2dea9f5df63583caee5cfa99794a5be2c796b9e4`.

This separately reserved local slice diagnoses unavailable full key health and the meaning of already observed signing failures. It does not complete the original O06 key-problems facet or establish full O06 completion. Root alone owns integration, publication and task status. The original six-facet/operator matrix and storage runtime evidence are in [the Sol completion report](local-wave30-o06-sol-completion.md).

## Reservation and concrete gap

After the storage checks handoff, root approved `wave30_O06_key_diagnostics` in the explicit project orchestrator’s ownership record. Scope: a new pure helper, the `operations.rs` module declaration, one JSON metrics field from its already sampled runtime value, one focused test target, one operations paragraph and this separate report. A later root runtime window remains required; the storage lane’s permission was not reused for this target.

Previously, metrics returned `runtime.signing_errors`, but a reader had no explicit diagnosis of full key-health availability, a component/safety statement or a corrective action. Zero could be misread as evidence that every key was healthy. The source counter is an observation of failures in this process, not a check of key material, retained verification keys or every signing domain.

## Implemented output and operator gate

Authorized JSON metrics and the existing `riauth metrics` command now include `key_health` (`riauth.key-health/v1`) for `component: signing_keys`. Its full `status` is always `unavailable`, with reason `full_key_health_not_measured`. The unavailable checks are primary signing material, signing domains, verification-key retention, remote signer and database encryption key. No check is silently reported healthy.

`observations.signing_failures` is exactly the **already sampled** `runtime.signing_errors`; its source is named, scope is `this_process`, and reset is `process_start`. At zero, `signing_failures_status` is `none_observed`, and the next action is `verify_signing_key_health`. Above zero, it is `observed`, and the action is `investigate_observed_signing_failures`. If the sampled field is absent or not a u64, observation is `unavailable` with a null count and `inspect_signing_failure_telemetry`; there is no zero fallback.

The safety statement says this read changes no state, neither zero failures nor readiness establishes key health, and old observed failures identify neither a key nor a cause. Remedies direct operators to restricted logs and scoped procedures for local signing configuration or an explicitly configured remote signer’s availability, access and connectivity. Other full-health checks need their existing operational procedures. No particular KMS cause, outage, key identifier or expired-key problem is inferred from a counter.

The helper reads no key/JWK/PEM/domain/configuration record and performs no new I/O, network call or mutation. The metrics route’s existing live `operations.read` on `operations/metrics` permission still governs access; a narrower `operations/health` grant does not authorize it. Existing runtime fields, Prometheus series, doctor, readiness and signing behavior are unchanged.

| Reserved file | Change |
| --- | --- |
| [key_diagnostics.rs](../../src/operations/key_diagnostics.rs) | Pure projection of sampled runtime count into availability, observation, safety and remedy |
| [operations.rs](../../src/operations.rs) | One additive module declaration |
| [observability.rs](../../src/api/observability.rs) | One additive field assignment after the existing runtime snapshot |
| [o06_key_health_diagnostics.rs](../../tests/o06_key_health_diagnostics.rs) | Focused permission, nonmutation, counter-window and real CLI evidence |
| [operations.md](../operations.md) | One additive key diagnostic paragraph |

## Counter source and fixed pins

At both the published base and this implementation, Platform `Core::sign_jwt` in `src/kms.rs` increments `signing_errors` only when its signing result is an error. The Essentials implementation in `src/kms_essentials.rs` uses the same failure predicate and counter, including refusal of an unsupported remote signing reference. `Telemetry::snapshot` samples the atomic count into `runtime.signing_errors`. The diagnostic preserves that already reported value and makes no second observation that could disagree with the snapshot.

| File at implementation `320dd96bae47a47e46dd19660cffe4cd881f16d2` | Git blob |
| --- | --- |
| `src/operations/key_diagnostics.rs` | `8cc6f63c651cc05be83a30699296f1d5685876ef` |
| `src/operations.rs` | `2483cb26e789c4cd62b34bf5285c0e375fe9d118` |
| `src/api/observability.rs` | `ccf65a38e420506c1c3cecac80cee783b4327ce6` |
| `tests/o06_key_health_diagnostics.rs` | `53010b843563d7b32c8956a3525ebab89052b305` |
| `docs/operations.md` | `b449672e88297334c67afcf7c9708cf7e0f32230` |
| `src/kms.rs` — unchanged signing writer | `215dda9b27b50cba7fcc837c781dfcddd5ef5b7c` |
| `src/kms_essentials.rs` — unchanged signing writer | `0636cafa8ffad1a0ae5957b8d328ebb04d72f6bf` |
| `src/telemetry.rs` — unchanged counter/snapshot | `467630e21dd325c2bdf3390feaeb8079733b9c9a` |

## Checks and their limits

Scoped standalone rustfmt, `python3 scripts/check-docs.py` and `git diff --check` pass. A source reconstruction proves that removing the single approved module declaration from `operations.rs`, and the single approved field assignment from `observability.rs`, gives the exact parent file bytes. Protected API route/classifier/core/config/node-security/state/workflow/credential writers and both signing implementations remain the exact published-base blobs. The accepted receipt-secret, route-specific header and PAM fallback contracts were not changed.

Two focused test cases are written. The first checks that zero failures still reports unavailable full health, metrics-only access retains storage withholding, a health-only grant and anonymous caller cannot read metrics, doctor keeps its existing administrator-health rule, and stored records remain unchanged. The second explicitly seeds the existing process counter to seven, validates exact snapshot projection without inventing a cause, reopens Core to verify the new Store telemetry window resets to zero while all persisted records stay unchanged, then reads the same diagnosis through the real `riauth metrics` command and checks redaction.

The counter seed is a deterministic telemetry fixture. It is **not evidence of a real cryptographic failure, remote signer outage, KMS diagnosis, key expiration or measured full key health**. A Core/Store reopen in the test creates a fresh telemetry instance inside the test process; it does not claim a deployed server restart. No key record is corrupted or replaced by these fixtures.

Root explicitly released the exact target after S04 and the controller check finished. Exactly one key-target Cargo invocation ran at immutable source `320dd96bae47a47e46dd19660cffe4cd881f16d2`:

```text
cargo test --locked --features test-support --test o06_key_health_diagnostics
```

Result: **2 passed, 0 failed, 0 ignored, 0 filtered**, 5.17 seconds test execution. The test profile compiled in 1m 03s; the monitored runner finished in 70.12 seconds. No compile/test correction or rerun was needed. The native macOS linker `__eh_frame` warning appeared; it was not a test failure. Root was notified immediately that this worker’s Cargo slot was released, before the report was finalized.

Environment: private `CARGO_TARGET_DIR` at this worktree’s `target/`, `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`. Initial free space was 17.742 GiB; the two-second monitor’s minimum was 17.059 GiB, above its enforced 8 GiB floor. Private log: `target/o06-key-health-platform.log`, SHA-256 `66d6bf026b20ee1df7cab42603510d27a9589da9dae1a51f7066b7ff62f8eb58`.

Final scoped rustfmt, docs links/build-layout, whitespace and reserved path/hunk checks passed. Runtime evidence is local macOS/redb in the default Platform build. No additional target, second edition, PostgreSQL, full campaign, real signer failure, cloud/HA/browser/desktop or external evidence is claimed. Shared Essentials counter semantics were inspected at the fixed source pins, not run. The previously handed-off storage target and exact regression passed at their own documented source object; they were not rerun or relabeled as this key check.

## Remaining scope

Full key-health measurement, true remote connector lag and physical storage pressure remain unavailable. Other workers own the configured-controller/missing-schedule reader gap, missing retained offboarding evidence and operator-remedy documentation. Those results are not credited by this report. Root must review the immutable implementation, actual focused runtime evidence and independent original-acceptance assessment before integration and task disposition. O06 remains open.

Desktop was unused. RiWork Cua.ai Driver remains the only desktop provider; its descriptions and current state must be read before any interaction, and this preference remains part of every authorized handoff.
