# Identity policy CI fixture support

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; existing worktree
`ed9ac424-59f4-4520-905b-919aea3521eb`, branch
`roadmap/local-revisions-coordination-wave27`. Date: 2026-10-02.

This is bounded support for two assigned CI fixtures. O03 task
`85240c6b-8c87-4a62-a07e-68c7ed1a5d5a` remains **DONE**, as confirmed by
the read-only RiWork task query. The accepted [root closure](local-wave29-o03-root-closure.md)
is preserved. S04 task `43b4ad2e-5b7c-46db-98ff-148be042d6ac` remains
**in_progress**. This work changes neither disposition nor task status.

## History and exact scope

Published baseline: `d31778f066d3bc79f94829cf6295e1eb8df2a84f`.
History-preserving merge into the existing own branch:
`45526584aedff67aa20e4b3ddb6d09006336875d`, with parents
`b1f79e64f723a1e91b5ed4317de3b760a766b563` and the published baseline.
The merge had no conflicts and retained accepted S04/O03 history and closure.

Fixture commit: `893be4fc0f80bd9da86e98c04a888ca4e7c06601`.
Its complete diff is 15 additions and 1 removal in
[tests/identity/policy.rs](../../tests/identity/policy.rs), confined to setup
inside the two assigned function bodies. The separate evidence commit adds
only this report. The bounded file claim and source-derived mechanisms were
sent to root through RiWork with the explicit project UUID before edits.

No production file or other identity fixture changed. In particular, no
`state.rs`, `config.rs`, provisioning, node-security, API, workflow, OIDC or
operations-family implementation was edited. The existing test helper and its
eight-step limit are unchanged.

## Downloaded CI failure evidence

User-supplied run `36951643190`, job `110666025654`, log
`/tmp/riauth-wave29-failed-36951643190.log`: 1,422 lines; SHA-256
`5a94f6f07d83ae7f4565d8fb032c3a629f31b44cff99ac84e67f7c0078d376f8`.
Its identity result was 171 passed, 7 failed, 3 ignored. This report addresses
only the two assigned policy failures; the other five belong to the OIDC and
operations lanes. No unchanged local baseline test was rerun.

- `platform_conditional_policy_uses_bound_signals_and_projects_only_allowed_claims`
  failed at then-line 71 because the client write required a configured
  `identity.device_trust` verifier.
- `outbound_scim_plans_provision_groups_preserve_remote_attributes_disable_departures_and_stop_stale_jobs`
  failed at then-line 618 after eight steps. The observed job had processed
  1 of 2 resources, attempts 0, no error, `stale: false`, `completed: false`
  and `next_attempt` at the current second.

These are historical Linux CI failures, not new local baseline reproductions.

## Contract inspection and fixture correction

The conditional-policy fixture declares an `ApprovedDevice` conditional claim
mapping. [capability.rs](../../src/capability.rs) validates that dependency
before a client write even when the current password session has no device
proof. Its usable-verifier contract requires the compiled and enabled
capability plus a valid configured verifier.
[device_trust.rs](../../src/device_trust.rs) checks the local JWKS and freshness
configuration. The fixture previously supplied no verifier.

The fixture now generates a separate synthetic public JWKS with the existing
`alternate_client_keys` helper, writes it in its disposable private directory,
sets the local `TrustConfig` and runs `Config::validate`. No device proof or
claim is fabricated. The original assertions still require the group-approved
claim, absence of `device_approved`, refusal after group removal at userinfo
and refresh, refusal of stale password assurance, and refusal of
`ApprovedDevice` access without proof. The unconfigured verified-source and
unrecognized client-asserted predicate rejection assertions also remain.

For SCIM, the read-only inspection covered plan eligibility, current actor and
review authority, revision, target fingerprint, expiry, resource activity,
due timing, admission and settlement in
[provisioning.rs](../../src/provisioning.rs). Successful settlement advances
the cursor, clears the job lease, resets attempts and sets `next_attempt` to
the current second. The existing step helper already accommodates due-index
cursor reuse; increasing its budget would not settle an admission whose
executor has been discarded.

[Background::shared](../../src/background.rs) uses a weak per-store registry.
[TargetPermit::drop](../../src/background/targets.rs) queues a release on that
executor; the next admission settles it only when owner and generation still
match. The fixture drove direct `Core::provisioning_step` calls without a
retained executor holder. Once the permit and temporary executor references
were dropped, the queued release was lost and the durable admission could
defer another step until its 60-second expiry. This source-derived mechanism
matches the downloaded pending, error-free, one-resource result.

The fixture now retains an unserved source `api::router` for its lifetime,
after configuring its SCIM target. Its App holds the existing shared executor,
so subsequent claims can settle the queued release. This follows the accepted
fixture pattern in
[tests/reconciliation_completion.rs](../../tests/reconciliation_completion.rs).
It starts no additional listener or worker. The destination loopback SCIM
server is the fixture's existing disposable service.

All original assertions remain: agent-scoped plan access, no credential in
the plan, administrator apply refusal, remote user and Group provisioning,
remote-owned attribute preservation, departure disable and Group removal,
and stale-job refusal after agent revocation. No sleeps, clock manipulation,
larger step budget, ledger edits, admission bypass or product changes were
introduced. The retained executor does not change the nonrenewed 60-second
admission limitation or establish paused-before-I/O exclusion or deployed HA.

## Focused checks actually run

Both commands ran in this worktree using the private `target/wave27`, jobs 1,
incremental builds disabled and dev/test debug information disabled. Default
features retain Platform. Each selected exactly one identity fixture:

```sh
env CARGO_TARGET_DIR="$PWD/target/wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test --locked --features test-support,fuzzing --test identity \
  policy_tests::platform_conditional_policy_uses_bound_signals_and_projects_only_allowed_claims \
  -- --exact --test-threads=1
```

Passed: 1 test, 0 failed, 0 ignored, 180 filtered out; 2.63 seconds.
The initial compilation finished in 3 minutes 47 seconds.

```sh
env CARGO_TARGET_DIR="$PWD/target/wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test --locked --features test-support,fuzzing --test identity \
  policy_tests::outbound_scim_plans_provision_groups_preserve_remote_attributes_disable_departures_and_stop_stale_jobs \
  -- --exact --test-threads=1
```

Passed: 1 test, 0 failed, 0 ignored, 180 filtered out; 2.72 seconds.
The warm build finished in 0.36 seconds. Both commands emitted the existing
macOS linker warning about the `__eh_frame` compact-unwind section; neither
had a Rust compilation error or runtime test failure.

`cargo fmt --all -- --check` initially rejected only the layout of the new
`write_private` call. That call was formatted, then the check passed. A focused
Python source comparison removed exactly the two new setup additions and
reproduced the entire pinned fixture file byte-for-byte. It also confirmed the
fixture was the only modified file before its commit. `git diff --check` and
the staged fixture whitespace check passed. `python3 scripts/check-docs.py`
passed. Exact changed-path and pinned-path checks passed and confirmed the
published O03 root closure is byte-identical to the baseline.

Disk space remained above the 8 GiB floor; the lowest observed free space was
24,115,896 KiB (about 23.0 GiB). No accepted target, PostgreSQL, cloud service,
benchmark, browser or full Rust suite was used. No desktop interaction occurred.

## Review recommendation and remaining limits

Accept the two fixture setup corrections as support for the reported CI
failures, subject to root review and integration. The exact local tests pass
with all prior security assertions preserved. These results do not constitute
a new Linux CI run, an overall green result or evidence for the five other
identity failures. No production defect requiring a new owned hunk was found
in this bounded inspection.

O03 closure remains preserved. S04 remains in progress with its separate
original acceptance and measured-security/performance/concurrency evidence
gate; this support work adds no S04 measurement or closure claim. Root owns
integration, publication and any board decision; no main edit, push, task
status change, new task, worktree, RiWork shell or worker was performed.
