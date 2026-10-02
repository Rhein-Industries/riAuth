# Root review of M03/M07 disposition

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; date 2026-10-02.

Root reviewed the complete [management disposition](local-wave29-management-final-disposition.md),
the original RiWork M03/M07 rows and current activation wrappers/docs at published
main `93999d15f7681ff918f986a341c48b5556bee24f`.

The reported activation seam is confirmed by source: Core passes a no-op first
activation guard and creates no receipt, while bearer activation honors supplied
revision/key. This contradicts the documented optional-header contract and the
shared same-change/same-outcome gate. No runtime reproduction is credited to
this static review. Root assigned one narrow shared activation envelope to the
existing Claude lane, after notifying the W02 owner. Preserve live replay,
legacy pin repair, authority-before-receipt, first-only guards and durable stale
sealing. Production/test evidence and integration are still pending.

M07's declarative coverage exists in the published manifest and versioned API;
no declarative GUI-only state was found in the mapped paths. Dedicated API
resources, browser-bound ceremonies and operator-only choices are separate
boundaries. Both M03 and M07 remain in progress until the concrete activation
parity seam is reviewed against their shared gate. Applicable external/runtime
and artifact evidence remains explicitly qualified; no universal HA or
all-resource-browser requirement is added.

Root corrected the M07 inventory row and its JSON twin with an explicit current
pin. The earlier wave28 audit is annotated as a historical snapshot; its
original 38-row evidence and JSON are retained. Other roadmap counts and labels
remain the historical inventory, with no new test-execution claim.

Other reported observations (legacy JSON backup audit, user-disable client
intent, self-service channel policy, PAM console precheck and applied-plan CLI
shortcut) remain recorded. They authorize no product change in this slice, and
the accepted credential/header/PAM contracts remain intact.

Only documentation/JSON scope, consistency and whitespace checks are performed
by root. No Rust build/test, benchmark, desktop or service is run in this batch.
No task completion or board mutation is made.
