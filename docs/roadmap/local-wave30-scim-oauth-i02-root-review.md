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

## Later CI observation and I02 guarded transport approval

CI37017294522 at published ae893780 completed failure. Audit and integration
succeeded; formatting and clippy succeeded; the all-targets check again stopped
at the uncorrected SCIM fixture: 16 passed, 8 failed, 18.80 seconds. Root retained
312498 raw log bytes, SHA-256
4f7e5c4a636b0e7c5fca5879f2076d6c96c00a105c21b534c0c798727362351e,
and inspected the target block/eight panic locations. This predates the fixture
publication9b8956f; its newer CI result is pending. No all-green inference.

The first released local whole-target request did not launch Cargo: the existing
CI shell returned login expired. Root did not operate login or reuse its unused
reservation concurrently. One invocation was reassigned to the existing Sol
workflow worker, with exact9b8956f tree alignment and fresh capacity preflight.
Its actual result will be recorded separately. No further pruning is authorized.

Root read the complete I02 design0569450afc67d1de01223bf42f39742193347722,
then independently read the current cookie guard, private browser interaction,
authorize_state/status, source-stage wrapper/runtime and callback renderer.
The approved source-only adapter keeps its page and guarded JSON mutations below
the original return-cookie path. The native original resume navigation retains
query/form_post delivery; no fetch follows an RP redirect and no CSP is widened.
The existing Core resume commits charged inner factor errors and rolls back
outer failures; the adapter must translate only the returned result without
introducing a writer. Binding, expiry, used-state, exact scope and closed outcome
checks remain mandatory. Legacy JSON GET/POST/cancel behavior is protected.

Only the API handoff/rate/route hunk, one additive portal module declaration,
three new portal adapter/page/script files, one appended focused transport
function and the existing report append are reserved. Implementation aligns by
history-preserving merge to9b8956f. Source review, byte preservation and static
checks precede a separate runtime release. No browser/factor/HTTP execution or
I02 completion follows from this design approval. Base-relative JS paths must
also work under a non-root issuer without duplicating its base.

One root documentation patch attempt used an absent context and was refused
before any file change. The subsequent append is additive; no prior review text
was replaced by that failed verification wrapper.

## Local whole-target failure and frozen-cohort follow-up

The reassigned Sol invocation at exact published9b8956f production and fixture
02ce939a exited101: 22 passed, 2 failed, 28.22 seconds. Root fully read the
154-line actual appendix aba4981a and independently rehashed its private0600
raw log e2dc6ca95deb2667af47886d046ec96ebef9ded3b7ae57a1db4cea289c2919eb
and observation559839950bd834ecc6b37dc9c58f8e5c4ad9d119fc7c1b08c5a1b3a4a21ed719.
History's next apply refused an unfinished predecessor; lost-PATCH's job remained
incomplete after the existing wait/step. Six historical failures passed locally.
No cursor/admission state was logged, so their original interleaving is unmeasured.

Root independently read connector_due, claim/step and exact queued admission
release bodies. A stored cutoff excludes a sole newly due row; exhaustion deletes
the cursor transactionally, and the next explicit pass uses a current cutoff.
Terminal rows leave the due index atomically. These two isolated fixtures have
one pending job; the empty-resource history job finishes without HTTP, and the
uncertain PATCH retry reads back the already applied fields before considering
another write. This establishes bounded source sufficiency, not retrospective
measurement or a competing-workload proof.

Source f89b219390be3eeb7446a14793424f503604cd6a adds two direct scheduling calls
and two comments only. Root removed those exact additions and reconstructed
the entire02ce939a fixture; production/crates/manifests/toolchain equal9b8956f.
All assertions, holders and the old three-second wait remain unchanged. Root
fully read static3e4fd938277191100d4aafb37c5762d85b72edac and released ONE
whole-24 repeat in the same private Sol cache with2s disk sampling,9GiB own-group
stop/8GiB floor and no deletion or retry. The changed fixture is unmeasured at
this source review; its actual result must be appended separately.

CI37022804623 at9b8956f has successful integration/audit/fmt/clippy jobs or steps
as observed2026-10-02; the all-targets check remains running. No full-green claim.

## Subsequent repeat and published Linux check

The one f89b219 repeat exited101:22passed2failed28.64 seconds. Root fully read
bee4b5f7f53f192feb61727f0a106b1c79e8ff90 and rehashed all3249 raw log bytes
94839820dedfc0bf516850c556a79f182263858235f39eb8b89391f6f516daff.
History and uncertain-PATCH now reached their unchanged final assertions.
Client-credentials' second plan and the last-member helper's initial seed
remained incomplete; no cursor/job-error metadata was captured. Those two
cases had passed in the previous run. This is no all-green or determinism proof.
Root reserved read-only analysis of a private step-helper restart ONLY after
measured cursor exhaustion, with at most one extra pass and no successful
first-dispatch advancement. A concrete protected source proposal must precede
implementation; no such helper change is accepted or run at this review.

Published9b8956f CI37022804623 completed failure: audit/integration/fmt/clippy
succeeded; check110890070823 stopped at SCIM21passed3failed25.41 seconds.
Root downloaded588141 bytes to private0600 storage, SHA-256
f4a0dc82a546d23cbb849938a53349680599a1c21ff8bb8ded283f5fe494e077,
and read the exact24-test block. History had unfinished predecessor409,
last-member helper cursor was1 instead of2, and uncertain-PATCH completion
wasfalse. This publication contained20dd holders, preceding f89's two passes.
Later USB/client/docs/release steps were not reached. The local repeat's
different failure set and this Linux result are both retained.

## Guarded scheduling-helper source proposal

Root fully read10b0e9e055af1644ceff08f8577f11caeae1bf1d and independently
read connector_due selection, claim/step, durable error and finish bodies.
A selected claim unconditionally parks a cursor before dispatch; neither finish
nor error deletes that cursor. In these sequential isolated fixtures, a prior
cursor becoming absent after a successful first call therefore identifies
exhaustion without selection. At most one additional public scheduling pass
can then occur without dispatching a second resource after a selected first pass.
The proof requires no concurrent cursor writer and makes no universal progress claim.

Only the existing private step helper is reserved for the exact proposed change;
all26 caller sites, assertions,20dd holders and f89 direct calls remain protected.
Read errors and the first Core error propagate. The proposal is not write-free:
ordinary scheduling/admission bookkeeping and one fenced resource outcome remain.
No changed helper is compiled/run at this review. Current native22/2 and
published Linux21/3 failures stay intact; latest d5b60bc CI37027444516 is running,
with no green conclusion.

Independent I02 source review identified one uncompiled fixture mismatch:
the new nonroot /identity case reuses an upstream token helper asserting the
root callback URL. Production source_callback_url includes the issuer base path.
The exact protected helper correction needs its own reservation; no production
defect or runtime result is inferred from this static finding.

## Guarded helper: fresh native pass and separate published Linux failure

Root fully read source199980044994d74adcb429bed70ff386399445d8 and its static
receipt7d63aca0d81c6c3fd965f831a92f6253e53059e7. Replacing only the private
step helper with its original body reconstructs the entire f89 fixture.
The guarded second pass requires an existing cursor before and no cursor after
one successful first call; errors propagate. This isolated sequential-fixture
proof does not establish concurrent scheduler behavior or change production.

Root fully read actual2414b5d83bc1ebbe31ffc99684ef7e4896e9d1d1 and all2401
private0600 raw log bytes, SHA-256
aba681faef060b4a920b684a2e12475a7da8b1f378e9d714577aa1fcccf3589f,
plus the9366-byte0600 supervisor record
b76f6df3dc549ba30e92a550a42d27c79e48ac0d5d073c4c0f0ac7555178953a.
The one whole-target Darwin run exited0:24passed0failed0ignored0filtered,
compile4.22 seconds/test29.23 seconds/wrapper35.137593 seconds. All24 named
raw pass lines were read. Owned group84412 was reaped and empty; observed
children were absent. Minimum13683351552 bytes exceeded9GiB stop/8GiB floor;
no stop, deletion or repeat occurred. Slot released before the separate report.
The executed211639968-byte test had private mode0700, not the older755 mode.
A supplemental test-name extraction retained a trailing delimiter; corrected
supplementv2 and original are both retained without changing log or invocation.

Separately, published d5b60bc12aaafa258daddd64b40879afacb87bc1 CI37027444516
completed failure. Audit110905570995 and integration110905571524 succeeded;
fmt/clippy passed. Check110905571536 stopped at SCIM23passed1failed25.51
seconds: reviewed_last_group_member_removal_requires_complete_remote_membership
at tests/scim_oauth.rs2404, unchanged initial seed incomplete. Root downloaded
and read the exact test/failure block in585959 private0600 bytes, SHA-256
234a6c4d8f955265c6f83bb1e0ed84af839908deadfeb31e543df3a2e0f04ecf.
That published fixture contains f89's explicit passes, not1999800's guarded
helper. Later USB/client/docs/release steps were not reached. Earlier running
observations and all prior failures remain historical; local24pass is not a
Linux-fixed or whole-CI-green claim.

## I02 corrected fixture and focused private-cache release

Root fully read independent7df439a1a258eaaddba7dde5fad67e0e07aea53e:
its concrete F1 is the configured nonroot callback compared to a root-only
mock assertion, with no other identified production/security defect by inspection.
Source196ac2096f79e5cc3c7d53ffa990ccdeba11a439 captures and clones the configured
callback and retains exact equality. Root reversed all three substitutions and
reconstructed the entire36ccc64 test; the complete appended browser test is
unchanged. The callback correction is not an HTTP/form_post pass.

Root fully read cd17c3bcffeba750d7827a54fdcf8d6a7470edba resource appendix.
Existing own target-wave27/cargo has matching native1.98.1/default+test-support
profiles/dependencies, while changed production and the new test still require
rebuild. Estimated additional2–3GiB is not measured peak. Root released only
one offline named source_stage filter to that existing cache after a fresh
at-least12GiB preflight, jobs1/inc0/dev+testdebug0,2s samples/owned9GiB stop/
8GiB floor/private capped0600 captures and bounded owned-group cleanup.
I02 owns the serialized Cargo lane. No runtime result is available at this
release; no browser, provider, extra target, retry or cache deletion is authorized.

## I02 first focused execution: compilation succeeded, snapshot failed

The single released corrected196ac209 filter ran at clean cd17c3bc.
Root read all1322 private0600 log bytes and the2147-byte supervisor result.
Log SHA-256 is9ed4e8a9a01a40f929ba51b343a0ee96cd26fe83972c475271319c0c4047e567.
Compilation completed59.03 seconds; raw selected test failed0passed1failed
23filtered in3.25 seconds. Supervisor total64.104 seconds and its4.891-second
fixture-phase interval are separate measurements. No compiler diagnostic failure
is established by Cargo's final test-failed summary. Only riauth rebuilt;
no third-party rebuild, deadline/disk stop or supervisor exception occurred.

The panic in tests/common/mod.rs71 says one source_logins record changed,
without dumping values. No caller backtrace identifies the HTTP/test step, and
neither response-mode completion is credited. Owned group99685 was waited and
empty before/after cleanup; minimum13210279936 bytes exceeded9GiB stop/8floor.
Peak target growth236486656 bytes is an observed allocation delta, not a whole
compiler-memory peak. Cargo released immediately; no automatic repeat occurred.

Root's source-derived candidate is the new bad-factor expected snapshot:
source_runtime assigns the original stage transaction to pending.authentication,
and unchanged source_finish persists that pending plus attempts on wrong OTP.
The new test currently expects only the attempt increment. This source trace
does not retrospectively identify the actual caller or stored value. Existing
independent reviewer is reserved for read-only diagnosis and a smallest exact
expected-record proposal; no snapshot exclusions, writer changes or source/
runtime correction are approved yet. Accepted staging remains unpublished while
this concrete new fixture failure is unresolved. I02 remains open.
