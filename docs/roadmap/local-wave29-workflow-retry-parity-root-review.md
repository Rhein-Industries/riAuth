# Root acceptance of workflow retry parity

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; M03 and M07 remain in progress.
Date: 2026-10-02.

Root accepts code `0bc623b9eeb2aa546711d9dd96636b6ee9efe927`, its
[implementation report](local-wave29-workflow-retry-parity-implementation-report.md)
and [independent Opus/Sonnet review](local-wave29-workflow-retry-parity-review.md).
Root inspected the production envelope, domain helpers, clients, key fixtures
and complete reports. The implementation report supplies the executed local
results; independent review is static and does not itself supply test passes.

## Design and evidence decisions

- D1 is accepted as the approved fail-closed policy: a pointer for an already
  completed retirement is inconsistent, so retry returns 409 without mutation.
  Emergency retirement of an ordinary active approval still does not require
  healthy former execution dependencies. This command provides no repair for
  the inconsistent restored-pointer state. The former restored-pointer test
  no longer executes the version helper's never-lower-newer-floor branch.
- D2 is accepted as a bounded-memory, linear traversal contract, with strict
  refusal on undecodable history. First retirement scans all revocations;
  completed replay additionally scans approval history under the writer.
  No index, benchmark, traversal deadline or constant-cost promise is added.
- T1-T4 and L5 remain evidence limits. Revision-overflow injection proves
  rollback of earlier retirement/pin/pointer/sealing writes; audit/receipt
  writes have not yet happened. It does not inject failures after those writes
  or a late review write. Seeded-receipt agent refusal, every live source/config
  drift, all retirement-matrix interfaces and all caller-bound edge cases were
  not separately executed. Source ordering and the shared writer were reviewed;
  the worker's nine R1 functions, six client mocks and six selected regression
  functions are credited within their exact scopes only.
- The explicit approval target is an intentional compatibility change. Release
  notes now describe old-key fingerprint conflict and untargeted refusal. The
  stale retirement handler comment is corrected. No behavior changed in these
  root documentation/comment corrections.

## Integration preservation

The code cherry-pick had one documentation conflict: retain the new targeted
revoke row and the already published source-TOTP row. The source-TOTP route,
handler and rate predicate remain present. All other hunks apply normally;
accepted activation, history/version, configured verifiers, node-security,
Core startup, recovery, Group and credential/header/PAM contracts are retained.
Root verifies exact source blobs where unchanged by the new route, scoped
handler preservation, formatting, documentation and whitespace. Root runs no
Rust test/build or external service in this acceptance batch.

The real server CLI journey, modified SAML definition, full e2e, PostgreSQL and
released artifacts were not executed by this slice. Published Linux CI is the
next runtime gate. This acceptance resolves the bounded review/revoke parity
change and does not close the whole original M03 or M07 task.
