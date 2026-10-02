# O03 edition fixture compatibility follow-up

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; existing O03 task
`85240c6b-8c87-4a62-a07e-68c7ed1a5d5a`, worktree
`ed9ac424-59f4-4520-905b-919aea3521eb`, branch
`roadmap/local-revisions-coordination-wave27`. O03 remains in progress;
root owns review, integration, publication and task reconciliation.

## History and bounded changes

This slice starts at clean `e529fc8cc78165c2e8057a4a1e7bcd2ded93093c`,
retaining the existing history-preserving merge of published
`3a46e983ae24530661c42f73e8d6e739574448f4`, accepted S04/O03 work and the
previous lockout fixture correction. No reset or new merge was needed.

Implementation `e50358e34510676bc2b8e77119b5611b3f2c38e1` changes only two
claimed files (25 additions, 7 removals):

- `tests/edition_activation_provenance.rs`: only the Platform fixture
  `platform_activation_retains_dependency_after_configuration_changes`.
- `scripts/check-exact-edition-matrix.py`: only the SAML candidate's threshold
  and the explanatory comment above the candidate setup.

The separate evidence commit changes only this report. The bounded claim was
sent through RiWork with the explicit project UUID before edits. No product
security, initialization, maintenance, node agreement, schema or workflow code
changed. `config.rs` remains excluded under W02 ownership. No other fixture,
including the in-memory rate/browser candidates, was edited.

## Preserved purpose and agreement semantics

The Platform fixture still initializes SAML 50 with device trust disabled,
records both Platform configuration dependencies, then removes those settings
from a separate candidate. Its original Essentials-target read-only preflight
assertions still require the retained provenance blockers and a complete
unchanged store snapshot.

The reduced candidate now explicitly fails `Core::open`. Its error identifies
the active-capability disagreement, which `node_security::decide` checks before
rate disagreement. A second complete snapshot comparison proves refused
startup changed no record, including agreement, provenance, audit or credentials.
The comparisons inspect all records but do not print private snapshot values
on failure. Successful reopening uses the original agreed configuration,
preserving both the SAML 50 rate and disabled capability. The fixture then
asserts the original provenance is retained. No different policy is adopted
or forced into an existing format-3 row.

The matrix script's SAML candidate changes from 10 to explicit default 30.
`src/config.rs:167` defines SAML 30; its effective resolver at lines 495-518
treats omission and explicit default equally for the complete rate agreement.
`src/edition.rs:434-442` rejects presence of the `saml` override in Essentials
regardless of its numeric value. Therefore the same setting still exercises
the Essentials Platform-only configuration boundary while the Platform positive
uses the initialized effective policy.

The matrix retains its separate genuine disabled-capability mismatch refusal,
including its diagnostic expectation. Artifact identity, exact edition build
commands, dependency checks, copied-binary hashes, release availability gates,
source revision/dirty checks and storage cases are unchanged. This is a source
comparison, not a newly executed matrix or artifact-validation result.

## Focused evidence actually performed

Every Cargo command uses this worktree's private `target/wave27`, jobs 1 and
incremental builds disabled. No full suite or exact edition matrix was run.

```sh
env CARGO_TARGET_DIR="$PWD/target/wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  cargo test --locked --no-default-features --features platform,test-support \
  --test edition_activation_provenance \
  platform_activation_retains_dependency_after_configuration_changes -- --exact
```

Passed: 1 test, 0 failures, 2 filtered out, 1.16 seconds. Compilation took
1 minute 24 seconds and emitted the existing macOS `__eh_frame` compact-unwind
size warning. The original incompatible fixture was not executed again; its
baseline incompatibility was source-derived in the preceding audit.

```sh
env CARGO_TARGET_DIR="$PWD/target/wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  cargo test --locked --no-default-features --features essentials,test-support \
  --test edition_activation_provenance \
  essentials_open_rejects_recorded_platform_dependency_before_mutation -- --exact
```

Passed: 1 test, 0 failures, 1 filtered out, 1.37 seconds; compilation took
1 minute 18 seconds. This existing exact counterpart checks the other
conditional build gate and the Essentials read-only refusal contract; it was
not edited. Other provenance fixtures were filtered out. Essentials compilation
emitted three unused-function/method warnings from the existing passkey and
session code: `discard_workflow_registration`, `workflow_register_start_in` /
`workflow_register_verify_in`, and `PostLogoutReturn::allowed_by`. No local
test failure or subsequent correction occurred in either focused check.

Python `compile(source, filename, 'exec')` passed without executing or importing
the matrix script. A complete Python AST comparison against baseline, after
normalizing exactly one SAML candidate string from 10 to 30, passed. This
establishes that every other executable script statement is unchanged. A
separate source comparison confirmed unchanged Rust imports and byte-identical
fixtures outside the claimed Platform function. The changed-path check before
adding this report found exactly the two claimed files, preserving production
code and root's future-format-4 correction.

`cargo fmt --all -- --check` and `git diff --check` passed.
`python3 scripts/check-docs.py` and the staged whitespace check passed after
writing this report. Post-test free space was 28,526,816 KiB (about 27.2 GiB),
above the 8 GiB stop floor throughout the observed checks.

## Remaining acceptance and dependencies

The original O03 outcome is to detect mismatched capabilities/security settings
and define shared jobs, rate limits and cache freshness, with actionable
operator diagnostics. This follow-up aligns two evidence fixtures with the
accepted agreement and directly checks one refused startup's complete lack of
mutation. It does not change the product contract or establish deployed HA.

Recommend root review/integrate this bounded code and evidence pair. The matrix
positive/Essentials rejection remain source-derived until an authorized matrix
run; no current matrix pass, release artifact result or green CI rerun is
claimed. In-memory rate/browser candidates from the prior report remain
report-only and await owner assessment. Shared job/cache freshness and external
operation acceptance retain their prior residuals, including the 60-second
non-renewed admission and paused-before-external-IO limitations. No PostgreSQL,
cloud, browser, benchmark, service or cross-build campaign was run. No new task,
worktree, worker or RiWork shell was created, and no board/status, main/accepted
branch or push operation was made. S04, Group hold, SCIM stamps, global fallback,
cloud ordering and workflow hooks remain intact.
