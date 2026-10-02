# Wave 30 SCIM OAuth fixture and I02 source review

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Root review date: 2026-10-02.
Published input: `ae8937800254a1ad4296ea257de1eccc4780e45b`.
Accepted published input: `67594e7fa15574756a5876acb1ff670c64418f72`.

## SCIM OAuth historical failure

The check job `110858262621` in Public CI run `37013383299` checked out
`9a819317efb3a13fa27cd86f884be2be00898fc0`. Its `scim_oauth` target
reported 16 passed and 8 failed in 18.66 seconds. Audit and integration passed;
the full check job failed. This review credits neither a green full CI run nor
execution of a target that the failed job did not reach.

Root read the failure block and the relevant direct-Core fixture paths. The
retained 312,377-byte log has SHA-256
`d71f45cbf65285336738a99a9d77e48201e09d0010757e638e47dbb205e84a64`.
The [worker diagnosis](local-wave30-scim-oauth-ci-report.md) preserves the exact
failures, source lines and limitations.

`claim_provisioning` obtains the shared background executor and target admission.
The registry holds a weak reference. Dropping a target permit queues its
owner/generation release on that executor; another direct step needs the same
executor to settle that queue. A fixture with no retained executor holder can
lose that queue and meet the durable target's still-live 60-second admission on
its next step. Refused admission makes that step return without progress.

All eight failing fixtures step the same target more than once. The sixteen
passing fixtures step each target at most once; the unauthorized case steps two
different targets. This correspondence and the source trace support the fixture
correction. The log itself does not record the executor lifetime, and earlier CI
reach of this target after admission leases were added remains unverified.

## Exact approved fixture delta

Author source: `20dd4eed2985ecb1017e95f12836e2da063566a7`.
Author static evidence: `308eb69a2f36010eaf6fab3474ac4556e47d745c`.
Root ports: `fc0ecaf` and `2995340`.

Only `tests/scim_oauth.rs` changes in the source commit: 20 lines added and 4
replaced. The original blob is
`eea5290b7c4cb49ec60c76059919a5318f05c1f1`; the corrected blob is
`02ce939a673c500296a3bf7feb1154180f39a20e`.

Six named test bodies retain an unserved router after creating their provisioner.
The private `staff_removal_at_group_step` helper retains a seventh router and
returns it with its existing tuple. Both callers bind it through their remaining
direct steps. `Router` was already imported. Root inspected every changed hunk
and the helper/caller lifetimes.

Root independently removed exactly those seven holder blocks and restored the
helper return type, returned tuple and two rustfmt-wrapped caller bindings. The
result equals the entire fixed `9a81931` fixture byte for byte. This proves that
all existing assertions, operation counts, due writes, retry behavior, removal
review, uncertain-PATCH and rotation checks are retained. It is a source-scope
proof, not runtime evidence.

No production, admission, lease, schema, clock, sleep, artificial probe, raw
ledger deletion or other test change is included.

## Runtime reservation and actual hold

The sole prospective command is:

```sh
env CARGO_TARGET_DIR="$PWD/target/wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support,fuzzing --test scim_oauth -- --test-threads=1
```

It covers all 24 tests. No narrower eight-test command, baseline, repeat or race
campaign is authorized by this review. The source remains uncompiled and unrun
at this phase. A09 manual remote run `37016520583` owns the serialized Cargo
slot. The fixture run requires its exit and a separate root release with
credible local disk headroom, a 9 GiB own-workload stop and an 8 GiB floor.

Local free disk was about 8.27 GiB at this review. Root requested a read-only
inventory of stale executables in the CI worker's private target. The earlier
ten-file prune authorization is exhausted. This request authorizes no deletion,
no additional target and no runtime.

## I02 bounded source report

Author report: `3d5855623ff5cc1310a889d41768771bb8533aad`.
Root port: `e328bc5959f91ee78c4a586e48bd3630abc3678a`.
Only [the initial I02 report](local-wave30-i02-federation-plan.md) is integrated.
Root read the full report, original row, embedded callback/resume handlers,
source-stage Core continuation and distinct standalone portal source route.

The reachable embedded source-stage continuation returns `local_factor_required`
JSON when a local TOTP is required after non-MFA upstream authentication. Its
fixture deliberately follows that GET with a manual JSON POST. The separate
standalone portal OTP transaction and configured-source bearer reauthentication
have different bindings. Their existence does not supply this embedded browser
continuation.

Root reserves a report-only transport design before source implementation. The
design must preserve JSON GET/POST compatibility, factor-in-query refusal,
Origin/custom-header enforcement, exact stage authorization and browser binding,
charged wrong-factor attempts, cancellation, expiry and one-use completion.
Query and form-post outcomes must use the existing callback renderer. No global
CSP relaxation, new credential/bearer, changed factor writer or absorbed protocol
gate is approved here. The original I02 row remains in progress; its primary
worktree assignment is unchanged.

## Checks and limits

Root performed the complete fixture reconstruction, immutable source/diff and
report-body review, aggregate whitespace check and repository documentation
check. No root Cargo, product, service, browser or native peer runtime ran.

The current SCIM Linux failure remains a failure. No corrected Linux, whole-CI,
remote tenant, physical storage, release or I02 browser completion claim follows
from this source review. I10, O06, R05, D03, D04, S04 and O03 remain closed under
their accepted original scopes.

## Reviewed private-target cleanup

The worker inventory `cc4b329d2b2a4b4dfc5a994f3e1338bd023ea6a5` and
actual cleanup `6acd927b3647717eaeb1ff3e465c480df784a796` are preserved in
the SCIM report. Root verified the immutable 0600 inventory manifest SHA-256
`b5fd6399c9fc1b2a47498eba342d58956aa8cf8e3bb3dff5e17d75e83831c113`.
Root independently rehashed all sixteen selected files and checked exact parent,
regular/non-symlink status, single link, mode, size, old feature fingerprints and
absence of live process references. These were metadata/live-reference checks;
no executable was run by this review.

Root authorized only the sixteen-file option, totaling 3,390,210,480 bytes.
After the worker's repeated checks, its one literal-path removal exited zero.
Root independently confirmed all sixteen paths absent. The worker's measured
free space rose from 8.04 to 11.19 GiB; root subsequently observed 11.18 GiB.
The private manifest and review receipt remain retained. This authorization is
exhausted. No additional cache, accepted binary, log, evidence or archive
removal is authorized. The SCIM target is still held after the remote A09 job.

## Later Public CI observation

Run `37014466702` at `3a57affd9023a48c32085d6bcfb1d17ca4feb901`
finished with audit and integration successful and the check job failed.
Fmt and clippy passed. The check job `110867414051` again reached SCIM OAuth:
16 passed, 8 failed, no ignored/filtered tests, 23.92 seconds. Root inspected
the eight failure locations and confirmed this fixture blob equals `9a81931`.
The later publication did not contain the staged router holders.

Root retained the 593,358-byte check log at mode0600, SHA-256
`af1813deb8c35ba28d113c6af9d8c599d1555ff7d0fa23c36c960948b2bd0830`.
Two extraction guards raised `StopIteration` because literal caret-rendered ANSI
sequences interrupted the target header. Matching the independent target/header
parts in the same retained bytes allowed exact extraction; this was a read-only
wrapper correction, not another CI run or product failure. Two root JavaScript tool wrappers also failed parsing
before any shell command ran; corrected quoting preserved the same inputs.
Subsequent USB/client/docs/release steps were skipped. The distinct manual ARM
run remains in progress. No full-green claim is made.
