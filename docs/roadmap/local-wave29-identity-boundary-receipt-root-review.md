# Wave29 identity receipt fixture root review

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; supporting M03 task
`3b9fbfa6-716a-4ce8-842a-2c953bab3d0f`. Reviewed against published main
`07176766473f8ef27d5188b001e89d06c7411a9a` on 2026-10-02.

Accepted source code `b20e74242158ea47b3b2acbe1ac0c42308bbbe00` as
`ea17c3378ce9144788fa1321b6dc01263a774f29`; separate source evidence
`e24f7de69e32f7c9da84c453054662b1f7ef7cc2` as
`c052554acfc9099190bec9a1dbd99bfcb1eb3b17`. Preparatory source merge is
excluded. The only code delta is the named management receipt fixture in
`tests/identity_boundary.rs`; no production contract changed.

## Review and evidence

The existing agent writer discloses a credential once, stores a redacted
issuance receipt and refuses exact retries with 409 `credential_already_issued`.
The old fixture expected the first secret response again. The correction
preserves first issuance, fingerprint conflicts and revision checks, and
asserts the exact refusal, no credential echo and complete snapshot equality.
It also checks ordinary group-result replay and both requests' refusal after
live administrator authority is removed. This preserves the distinction
between issuance and ordinary mutation receipts.

The worker reproduced the old oracle failure locally, then passed the single
exact fixture across plain and encrypted redb. Root inspected the diff and
evidence, compared accepted file blobs to the immutable source commits,
checked that source outside the named fixture is unchanged, and ran formatting,
documentation and whitespace checks. Root ran no Rust build or test. Exact
commands, failures and evidence limits are in the
[worker report](local-wave29-identity-boundary-receipt-ci-report.md).

Downloaded check job `110680696292` in run `36955560372` repeats the exact
409 issuance failure on the preceding published tree: five tests passed and
one failed in this target. That run's integration and audit jobs succeeded,
including the real-browser step that had timed out in run `36954886983`.
Neither result proves the browser timeout's cause. Current-main run
`36957782065` was still running when reviewed and predates this fixture fix.

## Scope and task disposition

This is a CI fixture correction with local synthetic raw-Core/redb evidence.
It is not whole interface parity, a Linux suite pass or release evidence.
M03 remains in progress pending root reconciliation of its original outcome
and published CI. M07 remains done. W02's final disposition is separately
being reviewed; its prospective conditional enrollment adapter has no code
authorization from this review. S04 and O06 retain their existing scopes.
