# Wave 30 remaining gates — root review

Reviewed against published `6b4db4f0317e4427c187f55e063989ccba124217` and the
explicit incoming deltas. This is a review record, not a new product execution.

## Background-capacity CI correction

Linux run 37003702884 check job 110828435533 passed Clippy but failed the named
library fixture at background.rs:1235: 123 passed, 1 failed, 1 ignored. Root read
the exact downloaded log, SHA-256
`1f3a81c11050ecd46aeb1414b1339d5c18c1b78191750316a10ee9fe6430fb02`.
Integration was still in progress at review time; audit passed. The library
failure stopped before the reconciliation-safety integration target.

Root read the exact execute/Running/drained bodies and the 12-line fixture delta
`934fcf14bf7c0ebeb1a0bf88087601a3a90ec20e`. The finishing task drops its
Running guard, job permit, then lane permit. The existing drained helper only
waits for job capacity. Waiting for all lane slots after every stalled pass has
already received its release prevents reuse before the lane is free. The test
owns this executor; its foreground App uses a different executor. The bounded
three-second semaphore barrier changes only this test's final loop and retains
all assertions, admission refusal, deadline, cancellation and production code.
The actual failing interleaving remains inferred from source, not logged.

Root explicitly released one exact corrected test. Its raw 14-line log was read
and hashed: `f0c82ebb68fb3248f2bc319569b92bbc389aa9b76c74d4b4ad838a67f5f306b0`.
It exited 0, one passed/138 filtered, 3.04s; build 1m06s, existing native unwind
warning only. This is local compile/regression evidence, not Linux confirmation
or a race campaign. The [worker record](local-wave30-background-capacity-ci-report.md)
preserves both phases. Root did not run Cargo or tests.

## D01 operator checkpoint

Root read the exact incoming appendix `9cde81dd93ff171fa54194b2ed514142451484d9`
and all 34 redacted JSONL records. File SHA-256
`a476b366da75b7a7c52a9f5b3edd5b615ea72beeb527d7801289501334774f3e`, mode0600,
27 command exit records all zero, both owned servers exited zero, final listener
absent and disposable lab removed. Matching c01-source Essentials server,
maintenance and base-client build fingerprints/hashes were independently reviewed.

Printed init/CLI login/doctor/client creation/discovery/groups/claims/audit,
online backup and closed offline restore succeeded. The restored store stayed
pending and was never served or attested; only the original lab was restarted.
The [actual report](local-wave30-d01-walkthrough-report.md) distinguishes these
operator commands from installation, browser-user/passkey, RP, invitation and
external-peer tasks. A separate independent browser-user checkpoint is reserved;
no result from it is credited here. D01 remains in progress.

## D05 dated acceptance snapshot

Root read the new two-artifact narrative and checked prior JSON value equality,
old ten-slice prefix, preserved Markdown and exact incoming bytes at
`eb1412aa6799ab959b90bdf71f29c983e15b67aa`. The independent Sol review
`09398ada2f8e01fe08ef50c4bbae4830ba906bac` found no blocking preservation,
correspondence or inspected-provenance error. Its 77 path/hash checks are distinct
from its twelve full review-body reads and selected result inspections.

Accept the dated c01 snapshot, ten evaluated-with-gaps categories and seventeen
evidence IDs with the full gate false. Later O06 closure and D01/R05 executions
do not silently overwrite that historical snapshot. Independent documented
user/operator completion remains explicit. [Review](local-wave30-d05-independent-review.md).
D05 remains in progress.

## O07 observed host prerequisite

Root read the exact setup-only correction `c8a34c8df5184d89f7a36d933f2ec172d01471cd`,
static record and actual appendix `fe68307ca86d2e3b29a896cc8ca170766fdbf545`.
Root also read and hashed the entire actual mode0600 JSON:
`351349aa31e383fabd76a13dce188019437c9174af56d92a9824888abdba8af9`.
The original failed JSON remains preserved separately. Both commands ran once.

The corrected attempt exited1: helper child_rc0, reported UID/GID0:0 and mode0700
instead of required10001:10001. This is an observed postcondition mismatch; its
underlying Docker Desktop/filesystem cause is not established. Only installed
image provenance passed. No service, credentials, application volume, selected
port or host-visibility stage was reached. Harness cleanup reported success;
there was no separate independent engine-inventory check. No permission bypass,
root application, image rebuild, daemon/context change or alternate topology was
used. A supporting host and later current/released/distributed inputs are still
required; old f3 image provenance is template-mechanics evidence only.
[Actual limits and full record](local-wave30-o07-deployment-plan.md).
O07 remains in progress.

## Root verification and ownership

Root read diffs and actual records, checked hashes/preservation, parsed Python ASTs
without execution, ran native formatting for the changed Rust fixture, documentation
and Git whitespace checks. Root did not build or run services, tests or containers.
Publication and task decisions remain root-owned. Closed original W02/W05/M03/M07/
S04/O03/O06 gates and credential/header/PAM/admission/paused-I/O limitations remain.
No current whole-CI, physical-capacity, tenant, hardware, release or HA claim is made.

## Root review of the notices, observer, x86 and IdP source batch

2026-10-02. Reviewed staging before this receipt: `0f33265b93fb10c409a37a2d98d6e63a77decd84`.
Root read the full new IdP C mode and 1,072-line Rust target, independent
260-line review `7ecc79bde1627f4729c28f918d0d4cf5effe4c9f`, and compiler
205-line appendix `ac869dff2f931a5bfbaa6d4c7563991f9c7c5f0d`. Root also
read the 8,279-byte actual wrapper and complete controls/preflight/result/artifact/
dependency records, independently checking private modes, sizes and hashes.
Metadata and compiler returned zero; both owned groups were reaped and absent.
The 40,880-byte arm64 helper hash is
`951465d744c1bf99e7ed91fc414337d00e960c24a1715977cce0e14535794bff`.
This establishes C compile/link only. The protected 347-path author baseline
remains ae893780; the five later I02 portal/API paths differ from b619 as
disclosed, while selected SAML hooks/manifests are equal. No worker alignment
merge or stale production import was integrated. A separately released one-filter
Rust/lifecycle run is pending; no protocol result or I04 closure is inferred.

Root read the full 516-line D01 preparation/cleanup proposal `3822aca11830ab582a955914457fad330285d8a1`,
including the complete 193-line candidate archive. Only the preparation
integer changes 30 to 180; original START+840, inclusive900, helper and
four-refusal limit remain intact. The earlier prepared attempt remains failed,
helper never started, and its true first-cleanup-to-final-absence 60-second bound
is unproven. The proposed direct owned Driver kill/end/readback sequence can be
measured; the MCP schema supplies no hard call cancellation/deadline. A new
separately released one-attempt fixture is pending. D01/D05 remain open.

The actual unmodified offline notices generator and subsequent check both
returned zero in the reviewed `41eea6a`/`e4512d2` pair. Root read both private
logs and the full 8,321-byte observation, checked their hashes/modes, and
reversed the exact single client-lock header-field change. No manual generated
edit or full-CI success is credited. Historical b5/b619 jobs still failed at
notices after their named Rust tests passed.

The shared-helper observer has source-preservation review and all 84 corrected
memory cases passed; the first zero-case preparation failure is retained.
It observes fixed finite failure frames and does not repair or retrospectively
diagnose the earlier PostgreSQL phase_exit. Root native x86 artifact receipt
and BuildKit registry/source pin are included with their own provenance limits.
No container or complete shared gate was run by this batch.

Actual root checks at reviewed staging: docs checker exit0, repository hygiene
exit0 (1,035 tracked files), cumulative whitespace exit0. The batch has no
production src/crates/Cargo/toolchain change. No board status changed;
root alone owns subsequent runtime, integration and original acceptance.

## Review of O07 correction and newly retained failure boundaries

Root fully read the307-line independent O07 review674af2c and92-line actual-static appendix9475ed0. The exact two-line guide correction35fb124 replaces the incorrect per-node PostgreSQL forward-auth claim with the shared counter under agreed effective rate limits. Reversal reconstructs the entire prior deployment guide, with every other byte preserved. Small-container mechanics remain unexecuted successfully and await the separate reviewed cohort; this paragraph alone does not close O07.

Root fully read the735-line D01 actual appendixc88b297, including its exact193-line executed controller and complete fixed private result/cleanup documents. Protected-before403 passed; the next root page produced unexpected_failure with zero Authorization refusals. Password/consent/callback/crypto/protected200 were not reached. Owned resources are gone; final absence was observed about119s after controller completion, so the later5.165s contiguous cleanup sequence cannot prove whole60s cleanup. A separate finite failure-observer design and first-observation cleanup design are reserved; no blind real-browser repeat is released.

Root fully read the179-line failed I04 compile receipt9ec96bc8 and91-line static correction4ed5b494. The one24c0dc9 certificate block introduces owned Asn1Time locals then borrows them. Reversing that block reproduces the entire39,498-byte previous target, and pinned cached OpenSSL0.10.81 public signatures support the ownership correction. Root rustfmt check passed. The old E0308 compilation remains failed, with no native/lifecycle execution. Exactly one separately released filtered run is pending; no outcome is credited here.

The current shared-store observer receipt separately retains run37054511216 failure, fixed AssertionError source frame helper150, complete sanitized capture hashes/cleanup and source-backed collection-read expectation diagnosis. A genuine administration mutation refusal is being designed without production authority changes. Historical unknown failure causes remain unknown. All original task statuses and closed acceptance rows remain unchanged. Root documentation/hygiene/cumulative whitespace checks are required before publication; no Cargo/native/browser/container runtime is performed by root in this review.

## Actual native IdP-to-Core pass and warning-only source cleanup

Root read the complete1,055-byte private Cargo log and12,603-byte fixed observation, checked their0600 modes/hashes, and independently reversed the three fresh supervisor literals to the entire previously reviewed wrapper. One authorized filter on24c0dc9/clean4ed5b49 passed1/0/0filtered, build2.59s/test2.63s/outer6.046559s. Owned group39829 was reaped/absent, no signals or safety stop; four disk samples had minimum22,505,332,736 bytes. All monitored source/helper/Lasso/default-library/manifests pins stayed identical. Root fully read actual153-line appendix08131c2 and preserved its entire prior report prefix. The [root receipt](evidence/wave30-i04-lasso-idp-root-review.json) retains exact provenance and metadata.

The passed source reaches native wrong-SP signature refusal, actual Lasso double-signed persistent POST, public Core callback verification, explicit link/retry/unlink/federation restoration, live trust withdrawal, sealed denial/full-snapshot/audit/replay checks and exact five-child/60s limits. This selected local macOS/Lasso2.9 source role complements the earlier actual SP/SLO evidence. It does not establish browser, external tenant, other OS or entire current published-tree execution. Original I04 disposition has a separate independent read-only review; no task status changes in this receipt. All earlier native and compilation failures remain failed, with the first old native cause unknown.

The actual log also contains an unused POST constant warning and existing native unwind-table warning. Root238618f removes only the unused declaration in the new Rust fixture; adding it back reproduces the entire executed24c test. No expression, assertion, operation, input or native helper changes. Root rustfmt and byte checks passed; no separate post-cleanup Cargo/clippy run is claimed. The warning cleanup is source-backed and the actual pass provenance remains24c. The exclusive Cargo/preparation lane was released before report work.
