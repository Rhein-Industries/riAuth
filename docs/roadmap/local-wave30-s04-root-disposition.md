# Wave30 S04 root disposition

## Original row and decision

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task
`43b4ad2e-5b7c-46db-98ff-148be042d6ac`.

**Accept and recommend DONE after publication.** The original outcome is avoiding
unrelated-change conflicts while invalidating relevant policy and authorization
dependencies. Its workstream gate requires measured improvement under equivalent
security and concurrency checks preserving identity invariants. The accepted four
narrow families, matching-plan lookup, live persistence checks and exact custom
issuer ownership projections implement that bounded outcome. Other shapes retain
the global fence; this decision does not expand them or accept the held Group stack.

## Independent review actually performed

Root read the entire 504-line workload and production diff from immutable source
`5120a0ddb51d015fbf89fbca19da127444e26f8b`, the exact report extension
`4d74e62f067f6bc7ef6ac1657f9a05d878654f2a`, and original RiWork acceptance.
Source reconstruction proved the common controller body byte-identical to fixed
`da5ff7dcfc3442c302955344229168872911b0ec` after reversing just the conservative
control guard. Production selects false; the hidden control is test-support only.
Existing test contents are an unchanged prefix. All apply, persistence, authority,
digest, removal, receipt and credential gates are retained.

The closed seed is copied only after dropping its Core. Full durable snapshots,
normalized configuration, token/principal records, initial authority digest, changes
and removal impact are compared before measurement. Unrelated real writes and
setup/security/apply assertions are outside each counter window. Three fresh pairs
use AB/BA/AB ordering across four families. The control's additional global reuse
check follows the same live dependency checks; it measures same-build conservative
replan cost, not an older binary or a weakened security implementation.

Root verified log SHA-256
`6287eb35f69017af138304ca7ca96624765d7a7b60ba49efab936ae2781e0e6d`, parsed all
24 cost rows and eight security rows, and recomputed the reported totals. The first
row follows libtest's test-name prefix; root corrected its JSON extraction for that
prefix. This was a root parsing correction, not a product or runtime failure.

## Actual runtime evidence

The worker ran exactly one reserved ignored workload: **1 passed, 17 filtered,
20.47 seconds**, compilation 1m08s, no correction or rerun. Across 96 controller
calls per lane, the global control measured 192 writer holds, 96 commits and
749,695 microseconds of writer occupancy; scoped reuse measured zero for each.
Commit time of 495,196 microseconds is included in occupancy. Elapsed totals
1,069,620 versus 107,675 microseconds are secondary local observations.

The same passing workload executed 16 policy/authority apply refusals with full
post-drift snapshot equality, 24 channels-ordered two-thread persistence cases,
and 24 final applies with exact replay and revoked-authority replay refusal.
Relevant writers and no-revision permission drift cannot silently reuse or apply
an invalid plan. Credential, key and unrelated user/client fields are checked.
The common persistence test path is identical in both labels because the control
changes reuse only. The [worker report](local-wave30-s04-completion-plan.md)
contains every paired observation, exact command and source mappings.

Root did not run Cargo or claim fresh Linux/PostgreSQL evidence. Scoped rustfmt,
documentation and whitespace checks are integration checks. Native macOS linker
warning and local redb/debug-profile limits are retained. This is measured native
writer-cost reduction under equivalent security in this bounded workload, with
ordered concurrency evidence; it is not statistical throughput, large-Group,
historical-binary, distributed HA or paused external-IO proof.

## Publication and remaining boundaries

Integrate only proposal, implementation and actual-evidence commits, not the
worker's alignment merge. Root owns publication and board status. M03, Q02 and
Q05 prerequisite state is checked before closing. The new CI pagination failure
is tracked separately as support work; neither its failure nor an overall CI pass
is silently attributed to this workload. O03 remains DONE and O06 remains open.
Accepted credential receipt, route-specific retry headers and PAM fallback
contracts are unchanged.
