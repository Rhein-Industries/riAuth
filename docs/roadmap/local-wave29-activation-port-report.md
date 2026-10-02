# Reviewed workflow approval service and client port

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; tasks M03
`3b9fbfa6-716a-4ce8-842a-2c953bab3d0f` and M07
`0da684c3-b5cd-45e5-b190-1d0ce97f2c80`.

## Fixed sources and port boundaries

Root ports the corrected cumulative service at
`546e20d5c943c7a02807f3037f4a010748f18d63`, incorporating
`7035f4725a20bc4913534276816eea40dfebcb79`, with workflow client provenance
`ee2b2ab3d74c0b523d5f281d1bfa19b99ed1737b` and corrected retry documentation
`2209c98a5afd7a1b0da741c30145e192d3a250f0`. Product base is published main
`f90f7cb83f5ef62d47f7b4b42ec9f6387a7962bb`. Preparation merges and old
activation shortcut `32f9aed` are not accepted as integration commits.

The port includes only:

- Exact corrected `src/api/workflow.rs` approval routes and adapters; every
  existing workflow handler remains intact. Published Platform route/context
  wiring in `src/api.rs` is unchanged.
- New server `src/cli/workflows.rs` and its three additive `src/cli.rs` wiring
  hunks. Root qualifies its comment for first-only revision checking.
- Exact standalone `crates/riauthctl/src/workflow.rs`, three additive main
  wiring hunks, the workflow mock target and source-built real-binary fixture.
  Root corrects fixture comments from stored receipt to revalidated view.
- Exact final `tests/workflow_approval_api.rs` definitions.
- The previously omitted one-line browser workflow-editor connector exclusion
  in `src/portal/admin.rs`; definitions and retirements cannot accompany an
  editor workflow apply. Other editor behavior is unchanged.
- Narrow API/workflow/client/capability documentation, retaining published
  rate-agreement and backup/output-guard contracts. Root documents valid legacy
  replay pin repair and the original If-Match fingerprint requirement.

No stale workflow approval/executor, core, state, recovery, configuration,
node-security, management export, credential, callback or connector file is
copied. All these product blobs remain identical to the product base. Held
client connector-status/export helpers, tests and unrelated source documents
remain excluded.

## Activation and evidence

The bearer adapter uses one existing writer. Human administrator authority
precedes receipt work. Both headers are required. Existing receipt validation
checks actor/key digest, exact request fingerprint, current permissions and
expiry but never returns its stored success. Only first activation compares the
live revision and writes a receipt. A matched receipt without an approval
causes outer error and rolls every activation write back. Healthy retries call
the accepted shared hook; they can repair a valid legacy missing pin without
new approval, audit, revision or receipt. Stale selection returns `Ok(Err)`
inside the transaction, committing run sealing before the outward 409.
Review and revocation retain their existing shared mutation receipt envelope.

Claude source evidence: 21 workflow API tests passed; accepted workflow approval
suite 10 passed with 6 PostgreSQL tests explicitly ignored; targeted Clippy,
format and documentation checks passed. One source-built standalone workflow
approval journey passed against the corrected service. Mutation checks reported
five failures with the old receipt shortcut and four without receipt validation.
These runs belong to the source lane and are not fresh root execution claims.

Root reviewed the full corrected service/client source, shared authority,
receipt and transaction helpers. An independent Codex static review of the
fixed correction confirms the transaction trace and cumulative port exclusions;
its final report is separate. Root formatting, documentation, whitespace,
immutable source/wiring/guard preservation comparisons passed. Root ran no
Rust build, test, external service or browser campaign for this port.

## CI fixture wiring and remaining gates

The new public check-job step builds the independent client into the isolated
CI checkout's root target so existing fixtures can find the sibling binary. It
runs eight explicitly selected ignored local source-binary targets plus the
redb-only backend comparison. It does not enable unrelated ignored tests,
PostgreSQL peer fixtures, or release acceptance. YAML parsing and the extracted
step's shell syntax were checked. The step has not yet passed on integrated CI.

M03's original permission/validation/transaction/idempotency/audit parity and
M07's workflows/roles/connectors/other-supported-resource coverage remain the
closure gates. Root will reconcile full resource coverage, the documented
interface contracts and integrated CI separately from tenant, peer and official
artifact evidence. This port alone makes no whole-task completion claim and
performs no board change.
