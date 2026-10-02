# Password recovery link CI root review

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; 2026-10-02.

Accept fixture source `9c3b9fae2901b67955fb0f9a2a8e4ad4ad9a274a` and its
[separate evidence](local-wave29-password-recovery-link-ci-report.md).
Only the assertion block in `sign_in_pages_offer_password_recovery` within
[tests/password_browser.rs](../../tests/password_browser.rs) changes.

The published Linux failure at main `979e7153f494cf827a0e39a75a78986544a92c42`,
run `36963622023`, assumed that the link ID immediately preceded its href.
Root independently inspected the portal template and capability script/CSS:
the correctly rendered anchor has its capability declaration between those
attributes. The link is present and retains its relative recovery URL and gate.
No production omission or capability defect was established.

The corrected check finds exactly one anchor bearing the required ID, then
requires exactly one expected ID, relative `/account/reset` href and
`identity.email_password_reset` capability attribute on that same anchor.
It tolerates unrelated attribute placement and refuses duplicates of the
checked attributes. Password controls and every account-script and forbidden
string check remain unchanged. No helper or dependency is added.

Root reconstructed the entire original test file by replacing only that block
with its old assertion. The prefix, signature, suffix and all other fixtures
are byte-identical to the published baseline. The worker's own alignment merge
has exactly that baseline tree; no production or dependency change accompanies
the fix. Exact immutable fixture/evidence blobs, formatting, documentation and
whitespace were checked. Root ran no Rust build or runtime test.

The worker ran the exact filter twice: a diagnostic baseline retained the old
assertion and failed while printing the correct gated anchor; the corrected
filter passed once. These are local in-process HTTP/redb results, not a browser
or JavaScript execution claim. No broad test campaign or new Linux success is
inferred. Publication triggers a new CI run; overall CI remains unverified
until that run completes. W02/W05 stay done and no task status changes.
