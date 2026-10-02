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

## I02 charged-factor expectation: independent source review and one release

Root read the complete four-line `90b6e8d832174df676d12395452e525de827f9e8` diff, its `b729b661aaa85fd11338365e2f9573f8133fdae7` appendix and the full independent `2356a312b6ee2cba30f207ec63f7532662af834d` trace. The unchanged writer binds the stage transaction before the invalid-factor branch persists the whole pending login and charges attempts. The expected-memory clone modeled the charge but omitted that binding. The correction derives the exact value from the previously captured stage record and retains the complete snapshot, operations, refusals and assertions. No stored record or production behavior is changed.

The first failed run did not record its caller or response-mode iteration. This source proof does not retrospectively identify them. The independent review accounted for all twenty earlier snapshot call sites and found no competing concrete omission. Root released exactly one same offline focused filter on clean corrected source, warm existing target, fresh at least 12 GiB launch, two-second samples, owned-group 9 GiB stop and 8 GiB floor. Actual outcome remains pending at this entry; publication stays held until this new fixture is verified. No broader test, browser or provider gate is credited.

## I02 corrected focused filter: retained actual pass

Root independently read and hashed the complete 757-byte mode-0600 Cargo log `62a6a3d17d5ef8aaa630e9bde8c2a62a2ed968669a6a798816ddc4736e4426be`, the 2,080-byte result `a51eda5e0b779b6ab731b22dfe2e7f7c7639aa16a6d4253387d7898eaec9c121` and ten-sample resource file `adcc45c4c2ae14c85605b527e68329610a6305de2b7250a0d104179546ef78d8`. Root fully read the separate `8dbbc74d8c325309b42c5011fb5d36a512fd6719` 112-line actual appendix and checked its byte-prefix preservation. On clean b729/source90b6, the one authorized offline filter exited 0: compilation 5.44 seconds, one passed, zero failed, twenty-three filtered, harness 12.00 seconds, supervisor 19.288 seconds. Only riauth compiled; the existing compact-unwind warning remains recorded.

The passing table-driven fixture includes query/root and form_post/non-root transport, the exact configured callback, guarded headers and binding, charged bad-factor full snapshot, one-use continuation, native handoff, cancellation and replay assertions. This is local HTTP/rendering evidence, not executed frontend JavaScript, a Driver browser, external tenant or release test. The old failed caller remains unknown. Minimum free space was 13,375,266,816 bytes; owned group 27192 was reaped and empty before immediate Cargo release. Root may publish the reviewed implementation and exact fixture corrections; original-row disposition remains under independent review.


## Original I02 disposition and one remaining guide paragraph

Root read the full `51fc964b6cb28f8945b69da9581a6f0017497b98` original-scope disposition and reread the live original five acceptance parts: setup, verified linking, assurance mapping, browser completion and credential/key rotation. Its local-completion recommendation uses inspected implementation and recorded native/HTTP evidence, including the fresh focused pass; it does not claim executed frontend JS, named tenants or a new all-provider campaign. Original status remains unchanged at this entry.

One source-backed documentation omission remains: the embedded-source-stage paragraph still says no browser OTP form exists. Root directly confirmed browser-UI plus explicit HTML handoff, configured-base paths, the labelled factor page, guarded original-cookie POST and native continuation in published source. Root reserves only the exact one-paragraph proposal in `docs/oidc-profiles.md` plus report evidence, with every other guide byte preserved. No additional runtime is required for that paragraph. Closure is held until the reviewed correction is integrated and published.


## Exact guide correction accepted for publication

Root read the full `e8424b55634fb7e98f8209eb94d9cbdf7bf41398` guide diff and `073b4fa35319e506faf8e2c7016aed453426c553` actual appendix. Comparing every line to the published guide finds only paragraph 179 changed; reversing that paragraph restores the entire old file. Browser-enabled HTML handoff and guarded factor completion are now described without changing JSON behavior, original authorization, absence of terminal credentials or the CLI cancellation limitation. Every other guide byte remains intact. Together with inspected implementation and focused/historical evidence, this resolves the last identified local I02 omission. Original-row status remains held until this reviewed correction is published; real frontend JS, tenant, hardware and release conclusions remain unmeasured.

## I02 original local scope: published guide and root status decision

On 2026-10-02 root verified publication main `b619fe25269ccc150e473bbcde47cdb3623ef810` / accepted `0a243c9afd9c18d38192145e58485357a2f30b93`, equal tree `a627df2ce21a4d255b8c1914d4f543e32f40f4de`. The exact guide correction is published. Root reread the five original acceptance parts, independent disposition and passing focused evidence and marks original I02 DONE for the reviewed local setup, linking, assurance, browser transport and rotation paths. Primary assignment remains unchanged. This decision uses implementation and executed evidence; frontend JavaScript/visible-browser, named tenant, hardware and release conclusions remain unmeasured. I04's separate independent-IdP peer-role residual remains open.

The CLI initially rejected an extra `--project` argument to `task status` with exit 2 and no mutation. Root then used the exact task UUID after an explicit-project row lookup and verified DONE, unchanged project, primary and details with another explicit-project lookup. The retained private status receipt records both operations. Earlier failed fixture evidence and its unrecorded caller remain unchanged.


## Completed b5 CI and later notices failure (2026-10-02)

Public CI37037991415 at immutable b5dcfa9dbb14e953d12edac6a6a1133ecb61033e concluded failure: integration110940835076 and audit110940835496 succeeded; check110940834919 passed its all-target Rust test step, USB boundary and standalone client checks, then failed during generate-third-party-notices.py --check with THIRD_PARTY_NOTICES.md is stale. Release build was skipped. This is not whole-CI success. Root retained the complete818157-byte mode0600 check log, SHA25654b582c1af19299c38a06b77d4aa1e8918f2ffc3ad16f4d7979915c2284e0a5f. Source-only notices diagnosis is separately reserved; no generator, Cargo or dependency operation was run locally by root. Later b619 CI37041786903 remains a separate running observation.


## Selected b5 Linux outcomes from completed logs

Root downloaded and hashed completed check/integration/audit logs without executing products. The [selected actual receipt](evidence/wave30-ci-b5-selected-root-review.json) records exact run/source/job identities and raw hashes. The complete SCIM OAuth block has24 unique passed names, zero failed/ignored/filtered,25.27s. The I02 browser_source_stage_transport_binds_factor_and_renders_both_response_modes name passed in the full24-test source_stage target. Entra certificate-file diagnostic and reports attribution names also passed. Both Lasso tests were ignored: no Linux Lasso lifecycle execution is inferred. Integration succeeded with its named XMLsec/OpenLDAP/browser/proxy/shared-PG blocks; audit succeeded with one allowed warning in each scan. Earlier native/Linux failures remain failed at their pins. The later notices check failed and release build skipped, so whole CI remains failed.

## Completed b619 CI (2026-10-02)

Run37041786903 at b619fe25269ccc150e473bbcde47cdb3623ef810 concluded FAILURE. Integration110960005237 and audit110960005527 succeeded; check110960005512 passed all-target Rust tests and both standalone client checks, then failed generate-third-party-notices.py --check: THIRD_PARTY_NOTICES.md is stale. Release build was skipped. Root retained raw check818156B SHA256c549f7c8cc4c1e66ee8d8420d68a73cb12524ff4964ac9754670a515be55e346, integration448957B SHAf6b092d43208a33e0ebbdf6804e20e18ca3cd0e351cbf8face8943676e3e86d4, audit116631B SHA8440d99f359032c4fba38d4898268fc61aae7908a74b5da1a174423575eeaff1. The full SCIM OAuth block is24 unique named passes/0failed/0ignored/0filtered/25.94s; source_stage24/0/0/0/42.44s includes the corrected I02 browser transport fixture. These selected Linux passes do not convert the later failure into whole-CI success or supply ignored Lasso runtime. The source-backed notices refresh remains separately serialized, with exact expected generated output.

## Later terminal CI observation at 0d090169

Root downloaded and rehashed the completed [run37046811570](https://github.com/Rhein-Industries/riAuth/actions/runs/37046811570) check job110979122967:438532 bytes, SHA256344b1f11389e1e250385f38f43bf82b233768b241725f59f5728af81df0db481, mode0600. Checkout is0d090169f20f61f7cb59b685dccb13203526539f. The job failed at THIRD_PARTY_NOTICES.md stale check; this predates the actual generated notices refresh published94054b3c. Integration110979122687 and audit110979123094 completed successfully at the same pin, with their separately recorded raw hashes. The overall run remains failed; the newer940 run was still in progress when observed, with audit passed. No historical CI outcome is rewritten as green.

## Later terminal audit/integration jobs at94054b3

Root queried run37053472817 and downloaded only its completed jobs. Audit110998234590 and integration110998234910 are SUCCESS; check110998234935 remains in_progress in that observation, so no whole-CI conclusion is inferred. Checkout94054b3 is present in both raw logs. Audit92,370-byte SHA0e4a8fc3bb28cd9b0d5a64c45a9a51fe83694ea1ef2a936535eae8205c22b9da and integration265,773-byte SHA4bb59cf29d99b8195465db1ac84601f2551f9de8f7c65dd78dd7c50bb7d18774 are retained mode0600 under root temporary evidence. The latter records independent XMLsec IdP/source/logout1/0 each, PostgreSQL2/0 and shared contracts91/0, Q051/0, OpenLDAP1/0, real browser RP1/0, portal1/0, nginx1/0 and Traefik1/0. Root inspected exact commands and test-summary blocks; none is attributed as executing the later69/5dcd/dc0 native IdP/observer source. Prior notices-failed aggregates remain failed. The ongoing check and later queued publications require their own terminal records.

## Terminal success at94054b3, with exact source scope

Run37053472817 now concluded SUCCESS: audit110998234590, integration110998234910 and check110998234935 all succeeded. Root retained and rehashed the complete873553-byte mode0600 check log SHA2921a4efb17d9e13c4bc225d1210ad7f188ee9ea23cc274ab69ba273a177aa9d, confirmed checkout94054b3c9b674e445893b52c1d7de29703fca73c and inspected named fixture lines/test-summary blocks and terminal step results. It records203 successful Rust test summaries and no failed summaries; documentation/notices and release build also completed successfully. The [terminal root receipt](evidence/wave30-ci-940-terminal-root-review.json) retains exact job/step metadata. Previously retained audit/integration hashes remain unchanged.

This is actual whole-CI success for94054b3 only. Earlier notices-failed runs remain failed; queued69 run37056602599 was cancelled with no jobs. Later native I04/observer/container source is not credited as executed by this older checkout. No ignored Lasso runtime, external tenant, current packaged release or D01 confidential browser success is inferred.


## Root later CI receipt at56bd, 2026-10-02

Run37061329345/check111020195345 finished FAILED at exact56bd082; audit and
integration succeeded. The CLI target reports21pass/1fail/0ignored/0filtered
in23.47s. cli_certificate_bind_and_revoke_require_retry_binding failed at
tests/cli.rs:954:9 with server exited early during the shared startup helper.
Numeric server exit and stderr were discarded, so no cause or certificate
failure is inferred. The full320779-byte check log SHA256bad1860b1454adc7db80f5fcb35c621e46fc70214855deb2bfe63d4f5dd1dc45
is retained privately; [terminal receipt](evidence/wave30-ci-56bd-terminal-root-review.json)
records exact provenance and missing evidence. Root reserved read-only startup
source diagnosis; no new test or source correction ran. The genuine whole
940 CI success remains dated to its pin; later/current source is not all-green.

## Later exact b71 CI success, 2026-10-02

Root read terminal run `37063876066`: audit `111027210441`, integration
`111027210794`, and check `111027210609` all completed successfully at exact
`b71b7b0041a549793233e8c7a81bbb61797e20f3`. The retained check log is852480 bytes,
SHA-256 `a7f6383497ccbf4b9149ec7bdeb290a4ca8245d77e2d7273d518ecd3b56aae1f`,
mode0600, with203 successful Rust summaries and zero failed summaries. Named CLI
certificate startup, background capacity, reports attribution, Entra diagnostic and
I02 browser-stage HTTP/rendering filters passed. Clippy, USB boundary, standalone
client/dependency/real-server checks, docs/notices and release build succeeded.

[The exact root receipt](evidence/wave30-ci-b71-terminal-root-review.json) preserves
job/step/source identities and bounded body-read limits. This dated success does not
change the earlier56 startup failure or borrow runtime for later source. No board
status changed; the separate readiness probe correction remains source-first.


## Terminal CI at 74e106b (2026-10-03 root review)

Run37067070690 at immutable 74e106b819e7186d0ac41964cedbe8217fa7e881 completed SUCCESS: audit111040417731, integration111040417946 and check111040417878 each succeeded. The [root terminal receipt](evidence/wave30-ci-74e-terminal-root-review.json) records the saved API jobs/steps and mode0600 raw check log:873544 bytes, SHA25655879429bc2649c5918675caa2927a81b42932ce653344b6344cc82d98ab808c. Checkout lines110/122 match; all203 Rust test summaries are successful, zero failed. Selected CLI certificate binding, background capacity, reports attribution, Entra credential-file diagnostic and I02 browser-transport names passed. Docs/notices and release-build steps succeeded.

This whole-run result applies only to74e106b. It predates the f142ca4 passive CLI startup correction and supplies no runtime proof for that correction, a rare race campaign, confidential D01 journey, A09 shared/container gate or later source. Prior CI56 failure and unknown startup-child outcome remain failed/unknown. Root read the complete terminal API plus bounded raw checkout, test summaries, selected passes, docs/release lines; no fresh full integration-log body review is claimed.

An initial receipt-generation assertion used a nonexistent head_sha API key instead of headSha and failed before any write. A subsequent add pathspec therefore failed; the working tree remained unchanged. The corrected generation uses the already retained terminal API/raw log, without a second download.


## Root review: one CLI filter and A09 diagnostic memory receipts (2026-10-03)

Read the complete actual appendices at CLI `674e04874bbcfa1536507e28a5fec629a86ceb72` and PG memory `406efd2`, and the full independent source review `f1e9e386a74cdde44f3adaf6ac1ef5241c9b270b`. Each immutable commit has only its reserved report path. Whole author prefixes and current published/staged report bytes were preserved; only exact author suffixes were appended. The independent report is a new exact copy. Earlier blocked designs and failed observations retain their dated outcomes.

CLI: root read the complete 3579-byte console (SHA256 `6b80ef1ca164b534c202cb411b60350bb35f91677266456c500d78383362afc3`), closed final stdout and numeric wrapper receipt. One exact selected local Darwin ARM64 test passed, 1 passed / 21 filtered; Cargo/controller/wrapper exits 0. Final post-save time 61.955337500s stayed within the unchanged acceptance limit. Owned leader 59319 was reaped and the receipt reports zero remaining owned members. Minimum sampled free space 25116266496 bytes across 32 samples. Three debug-stripping SIGABRT warnings and the compact-unwind warning are retained. This local regression does not establish the lost CI56 child cause, a Linux fix, rare-race elimination, or a later whole-CI result. Cargo was released before report work.

A09 memory: root read retained child/controller/numeric exit and complete final parent JSON, verified hashes/modes and derived 89 distinct pass records in eight groups (9/5/24/14/20/8/8/1). Child exit 0, stderr empty, owned child 68399 reaped, final post-save time 0.08579179178923368s. Full exact input pins and authoritative snapshot-equality semantics remain unchanged. The independent review read complete archives and found no remaining corrected binding/signature/selector blocker; its 89 cases are source-only in that audit. The author actual run is separately credited as memory diagnostic compatibility/privacy/fallback evidence. It does not observe PostgreSQL rows, explain the historical hosted inequality, or pass the shared native gate. No Cargo or desktop was used; the temporary validation lane was released before the next separately reserved retained-archive invocation.

Root made no product/helper/test/workflow edit, repeated invocation, status change, or inference of browser/artifact/full-row completion in this receipt integration.

Integration check correction: the first root scope assertion counted tracked diff paths only and refused because the new independent report was still untracked. The assertion was corrected to include Git untracked paths; no author bytes or runtime were changed or repeated. Docs and whitespace had already passed.


## Root review: retained archive actual pass and D01 continuation design (2026-10-03)

Full `ec8a252352c1b340842cd3acca1d3ea8f05b701b` actual appendix read. Root read actual invocation/final closed JSON, child envelope, complete outer journal and all42 case rows; verified modes and identities. Parent/child0, reaped/owned group empty, final post-save0.5080366251058877s. Actual42 unique ordered cases comprise3positive/39specific refusals; first positive is the original retained full saved archive, six derived legacy objects remain strictly bound. Opaque layer bytes were hash-streamed only, never extracted or executed. Historical container37061329815 stays FAILED; UID/mount/nonroot/E-P-E gates remain unreached. Validation lane was released before the new separately reserved hosted PG count-observer run37077820186. No Docker/Cargo/desktop was used by the archive check.

Full new D01 `9f3a4a372b53fc6d73d5df45ad1a8d217ad60879` report and all485lines of the candidate read. Root independently applied all4hunks to the immutable31581-byte c5 cell, reproducing the entire32502-byte candidate SHA256 `7ab23019e838512a84600d4069fef2e419b9bb7f07ff5aaa2c7477af956f9ca8`. It narrows password clearing to password dispatch and cleanup entry, permits helperzero only for the declared read-only final protected snapshot, and drains partial controller lines within the same invocation/deadline. Exact native controller waits for the stop marker after helperzero; it does not finish the outer controller merely because the helper succeeded. All following snapshot/outcome proof remains required. Independent review is separately assigned; this is unexecuted design and not a browser pass or runtime release. The incomplete preparation/seed/private bridge design remains separately owned. No root product/source/status edit occurred.


## Hosted A09 fixed-count observation: actual failure (2026-10-03)

ONE run37077820186/job111071534441 failed at focused-shared-helper (3.204996s), AssertionError/full snapshot equality, after preceding strict CLI4/HTTP403/access_denied assertions. [Full sanitized root receipt](evidence/wave30-a09-hosted-counts-37077820186.json) records exact workflow7869/validator8a2/product9a and verified output artifact11257332482 (ZIP17555B, SHA596b7754c542722f2b976147a0b0f35ba6111f296d1abc0a17890c38e81648b8). Fixed count projection actually reports ONE changed http_rates row, no additions/removals, and all other categories unchanged (including51 protected_or_other rows). Exact changed key/value/field remains unrecorded. This new observation does not retrospectively assign the earlier unobserved hosted failures or permit broad rate-row exclusions. Source-backed exact expected admission effects require separately reserved review before any gate correction.

Cleanup failures empty, remaining owned processes0, fixture/private scratch removed, no live owned PostgreSQL; six samples minimum115572277248B/maxgap2.000676759s. Finalize step returned failure because the original sample failed; cleanup failures are empty. Runtime lane is released. Helper-redacted pending report is not a successful or completed full gate. Transport and five original pinned archive/binary validations are run-attributed; root fetched only the four sanitized documents and verified the outer digest, not native archive execution. A first root member guard mistakenly expected helper.json and refused before extraction; corrected exact helper-redacted.json used the already verified ZIP, no redownload or runtime repeat. No A09/status closure follows.
