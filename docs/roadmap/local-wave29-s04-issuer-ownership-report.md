# S04 client metadata issuer ownership

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; existing task
`43b4ad2e-5b7c-46db-98ff-148be042d6ac`, worktree
`ed9ac424-59f4-4520-905b-919aea3521eb`, branch
`roadmap/local-revisions-coordination-wave27`. The task remains in progress;
root owns review, integration, push and board reconciliation.

## Baseline and exact commit

History-preserving merge `2d75f62d0cba02ee6900c1761da40916b2430378` brings
reviewed main `f90f7cb83f5ef62d47f7b4b42ec9f6387a7962bb` into this branch.
Three documentation conflicts were resolved by their individual hunks:
`docs/api.md`, `docs/operations.md` and the O03 rate-agreement report. Each
resolved file is byte-identical to pinned main, preserving accepted O06 wording
and root's cross-build format-4 correction note. No history was reset and no
old shared file was substituted.

Implementation `470471e391275fecb2ee1b45cbfba10bf405d31c` changes two files
(268 additions, 8 removals):

- `src/state.rs`: only the issuer ownership projection in
  `client_record_dependency_digest`, reuse of its existing primary-issuer read,
  and the client name/description dependency version constants.
- `tests/state_reconciliation.rs`: one appended fixture,
  `client_metadata_plans_track_only_competing_issuer_ownership`, available under
  `test-support`. Every existing fixture remains unchanged.

The separate report commit changes only this document. No management writer,
issuer validator/assembly, API, workflow, Group representation, connector,
O03/node-security or other plan-family code was edited.

## Corrected behavior and preserved fences

Previously both narrow client metadata plans hashed every other client's
custom issuer. Adding a noncolliding issuer to an unrelated reporting client
therefore replaced a pending portal name plan, despite unchanged portal,
signing, credential, policy and authority dependencies. The new fixture
reproduced that unnecessary replacement on the pinned baseline.

The projection now follows the actual shared writer's ownership reads:
`management::check_client_as` calls `core::validate_client`, which calls
`issuer::validate`; `assembly/issuer.rs` supplies exact competing ownership.
Omitted issuers and explicitly shared primary issuers have no unique-owner
dependency. A non-primary custom issuer binds every other client claiming that
exact string, including disabled clients. Noncolliding issuer changes are
excluded. The existing validators still reject collisions before writes.

Both digest domains advance from `v1` to `v2`. Pending name/description plans
from the old digest must be replanned: current reuse, persistence and unapplied
apply recompute the new domain rather than rewriting retained hashes or
reviews. Completed results retain their existing authorized replay behavior.
The digest-domain change was inspected in source; no older-binary upgrade or
rollback execution was performed.

The whole client, selected signing ring, credential version, referenced
groups/user indexes/source availability, primary issuer, bound listeners and
configured issuer/capability/device policy remain bound. Live principal and
review authority, exact stored plan, expiry, mode, current If-Match, removal
impact/confirmation, change equality, audit and receipt gates remain intact.
Mixed, connector, target-bound and all other global families retain their
existing revision checks. No new narrow family was introduced.

## Focused evidence actually run

Every Cargo invocation used this worktree's private `target/wave27`,
`CARGO_BUILD_JOBS=1` and `CARGO_INCREMENTAL=0`. Observed free space stayed above
32 GiB, safely above the 8 GiB stop floor.

Exact build/run command:

```sh
CARGO_TARGET_DIR="$PWD/target/wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  cargo test --features test-support --test state_reconciliation \
  client_metadata_plans_track_only_competing_issuer_ownership -- --exact
```

1. On pinned main plus the new fixture: failed as expected at
   `client/omitted: unrelated issuer invalidated matching reuse`. The controller
   returned a new plan ID. No product workaround was added to the fixture.
2. After the owned projection/version fix: passed, one test, 10.53 seconds.
3. The same fixture was strengthened to assert the full pre-preview snapshot
   inside every persistence callback, and to share the primary issuer explicitly
   in the omitted/primary positive cases. Exact rerun passed, one test,
   11.39 seconds. No additional production change was needed.

The final fixture covers six positive scenarios: both name and description
callers with omitted, explicit-primary and non-primary custom issuers. Real
unrelated client updates survive matching reuse and deterministic persistence
interleaving, then apply. One reconciliation audit, one apply audit and one
receipt are recorded; exact receipt replay leaves the full snapshot unchanged.

Ten drift combinations (both callers times enabled contender, disabled
contender, edited-client issuer, signing key and planner permissions) are each
checked at unapplied apply and between preview abort and persistence. Every
drift leaves `meta.revision` unchanged. Refusals preserve the entire snapshot,
including credentials, plans, indexes, audits and receipts, apart from the
deliberately committed drift. Persistence leaves no plan. Four ordinary writer
collision attempts and four matching-controller collision attempts also refuse
without changing the snapshot. Raw contender drift represents stale/restored
local data; it is not evidence that ordinary writers admit a collision.

Four existing regressions were run individually from the freshly built binary
`target/wave27/debug/deps/state_reconciliation-4fd9f4fc50528004`, each with
`--exact`:

| Exact filter | Result |
| --- | --- |
| `client_name_desired_state_keeps_credentials_policy_and_authority` | Passed, 2.37 s |
| `client_description_desired_state_keeps_credentials_policy_and_launch` | Passed, 2.44 s |
| `client_name_desired_state_http_replays_the_same_request` | Passed, 2.37 s |
| `client_description_desired_state_http_replays_the_same_request` | Passed, 1.67 s |

`cargo fmt --all -- --check` and `git diff --check` passed. An initial format
check requested layout changes only in the new fixture; formatting that file
resolved it. The compiler retained the existing macOS `__eh_frame` linker
warning. No other product-test failure occurred.

Static source normalization against pinned main confirmed that only the claimed
issuer projection/read reuse and two version constants changed in `state.rs`.
The existing reconciliation test file is an unchanged prefix with one fixture
appended. Final documentation/link and staged-diff checks passed before the
separate report commit.

## Original acceptance and remaining gates

Original acceptance: "Avoid conflicts from unrelated changes while invalidating
plans when relevant policy or authorization dependencies change."

This slice closes the identified issuer false conflict for the two existing
client metadata families while proving the bounded invalidation/refusal cases
above. Recommend accepting the slice after root review, alongside the already
accepted lookup and persistence slices described in the
[connector and S04 report](local-wave28-connector-s04-port-report.md).

The complete S04 task remains in progress. Other state edits, reviewed client
and grant proposals, and LDAP/cloud/SCIM target plans deliberately retain their
global fences; this slice supplies no broader family design or coverage.
Original prerequisites remain M03, Q02 and Q05. The separate workstream gate
requires measured improvement under equivalent security settings and concurrency
evidence preserving identity invariants. This deterministic local redb check is
neither a benchmark nor fresh PostgreSQL, deployed multi-process or release
acceptance. Historical reviewed evidence is not claimed as a rerun here.

No broad tests, benchmarks, PG/cloud/service launch, real external mutation,
new RiWork task/worktree/worker/shell, board change, main edit or push occurred.
The Group hold, connector/global fallback, cloud authorization ordering/routes,
workflow hooks, mail/SSF pins, explicit rate agreement, shared SCIM stamps and
nonrenewed 60-second admission contract remain intact. No deployed HA or atomic
fence for a paused process between admission and external IO is claimed.
