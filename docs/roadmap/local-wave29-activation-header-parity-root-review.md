# Root activation parity acceptance and M07 closure

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; date 2026-10-02.
M07 task `0da684c3-b5cd-45e5-b190-1d0ce97f2c80`.
M03 task `3b9fbfa6-716a-4ce8-842a-2c953bab3d0f` remains open.

Root accepts code `1a21871b477d5e58d89cd82130f8e9c5b63a07d6` and the
[implementation/review evidence](local-wave29-activation-header-parity-report.md).
Only three reserved code/test files and the report are ported. The worker's
alignment merge and duplicate-test cleanup are excluded; accepted main already
contains the canonical CLI fixture.

Root inspected the complete production diff, new fixtures and report. Core
uses Optional and bearer Required through one activation service. Authority
precedes receipts; supplied keys participate in receipt handling, and supplied
revision guards only first activation. Receipt results never answer. A matched
receipt without its approval rolls back. Replayed retains legacy pin repair;
Stale alone maps to nested error so sealing commits. No-header Core/browser
and bearer required-header behavior are retained.

Exact comparisons passed for all changed source blobs. Removing the new helper
and restoring the wrapper reproduces the entire prior approval source: hook,
raw activation, history, review/revoke and labels are byte-identical. Context,
Core startup, config, API rate/header middleware, portal callers and version are
unchanged. Root fmt/docs/whitespace checks pass; root runs no Rust build/test.
A root metadata script initially expected eleven inventory cells rather than
ten; its guard stopped before the row rewrite. Corrected cell bounds preserve
all other inventory rows and historical counts/evidence.

Worker committed-tree evidence: 38 API functions passed, including 8 new
browser/Core cases; 10 approval functions passed, with 6 PostgreSQL cases
ignored; two exact configured-adapter functions passed. Repeats and the 8-case
subset are not additional functions. Clippy/fmt passed. Independent Sonnet
review found no blocker/high/medium. Mutation reproduction was executed by the
implementer only. Merge E0428 and its local alignment cleanup are recorded.

Cross-route key reuse, browser/Core agent-with-key and activation-specific
receipt-expiry/permission cases were not separately added. Core fingerprints
are synthetic; HTTP modes use real middleware. Preamble duplication and
in-process request-context responsibility remain disclosed. This slice runs
no PostgreSQL, real browser UI, external tenant or released artifact.

## Original M07 decision

Root reread the original task and the [resource/interface disposition](local-wave29-management-final-disposition.md).
Published versioned APIs/manifests cover workflows, roles/delegated grants,
connectors, SSF and supported shared records. No declarative GUI-only state was
found. Dedicated APIs, operator settings and browser-bound ceremonies have
explicit boundaries. Accepted cumulative connector/security/client evidence
and corrected activation parity meet the original outcome and its shared
same-change/same-outcome gate for these resources. Root closes M07 only after
this review batch is published. Applicable live-peer, multi-node deployment and
official artifact evidence remains qualified, with no new execution claim.

M03's broader verification remains open while its newly reached credential
boundary CI fixture is corrected/reviewed. Run 36954886983 check 110675857136
expects an agent secret replay; current code returns the accepted one-time
`credential_already_issued` 409. A separate existing lane owns that exact
fixture. Integration job 110675857307 timed out starting browser authorization;
another lane is diagnosing it read-only. Neither establishes an activation
regression, and root makes no green CI claim. The user-accepted credential,
header and PAM contracts remain intact.
