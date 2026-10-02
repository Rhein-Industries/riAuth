# riauthctl connector-status export port and M03/M07 closure mapping

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`. M03 task
`3b9fbfa6-716a-4ce8-842a-2c953bab3d0f` and M07 task
`0da684c3-b5cd-45e5-b190-1d0ce97f2c80`, worktree
`9c54c023-a95f-48e9-a908-6f97a5677c98`, branch
`roadmap/local-management-wave27`. Both tasks stay in progress until root
review. Lane worker: Claude Opus 5.5. Implementer and reviewer subagents
reported `claude-sonnet-5-5`.

## Commits

| Commit | Kind | Content |
| --- | --- | --- |
| `a355cdd96a99bb975047e1fe51572e418a12ef8d` | Alignment merge | Main `3a46e983ae24530661c42f73e8d6e739574448f4` merged into the branch. Main's published operations, workflow and backup equivalents won every conflict: the riauthctl README, backup and workflow docs, capability matrix, operations and workflow docs, `src/cli/workflows.rs` and `tests/m03_workflow_approval_e2e.rs` equal main. |
| `687ee6c7dffde90f66c57fb971d0fc3e3ce55a17` | Code port | riauthctl `export` connector-status summary as a strict allowlist, its focused tests, one README sentence and one capability-matrix sentence. |
| `4cf53d226d4b12d7fe6465c378f0295c9f47d879` | Evidence provenance | `tests/m03_connector_ldap_e2e.rs`. The file at `687ee6c` is byte-identical to this commit (`git diff 4cf53d2 687ee6c -- tests/m03_connector_ldap_e2e.rs` is empty). |

After `687ee6c` the branch differs from main in these files only:

- Claimed port: `crates/riauthctl/src/management.rs` (export summary and
  `connector_status`), the export tail of
  `crates/riauthctl/tests/m03_parity_ops.rs`, the export sentence of
  `crates/riauthctl/README.md`, one sentence of `docs/capability-matrix.md`,
  and `tests/m03_connector_ldap_e2e.rs`.
- Not claimed, present on the branch for owner review:
  `tests/connector_workflow_boundary.rs` (workflow-review and browser
  workflow-editor refusal of a workflow plus connector plan; main carries both
  guards but not this test) and one registration typed-body limit test in
  `tests/m03_cli_limits.rs` (main's server CLI uses the same typed reader).

## Source equivalence

- `connector_status` and the export summary originate in `ee2b2ab`, which the
  wave 28 M07 review accepted separately for connector export status. `687ee6c`
  ports it onto main's export, after main's early UTF-8 output guard and its
  `secrets_included` and `authorization_header` refusals, and tightens it.
  Nothing else in `management.rs` changes.
- The export test tail originates in `ee2b2ab` (`export_server` and two
  tests) and is replaced by three stricter tests.
- No server, workflow, approval, core, state, configuration, recovery or
  connector production file differs from main.

## What the client now enforces

`riauthctl export` prints `connectors` only when the server returns it. The
server's shape (`src/connector_definitions.rs` `status`) is accepted
exactly:

- **Status:** `connector_secret_dir` must be a strict boolean, and
  `definitions` an array of at most 1024 rows.
- **Each row:**
  - `kind` is one of `ldap`, `workspace`, `entra`, `scim`;
  - `id` follows the server's name rule;
  - `revision` and `loaded_revision` are an unsigned integer or null;
  - `digest` is null or exactly 43 unpadded base64url characters, which is
    what `crypto::digest` produces;
  - `loaded_in_this_process` and `restart_required` are strict booleans.
- **Retired rows:** for a definition retired while the process still runs
  it, `retired` and `loaded_digest` are copied, as `riauth export` shows them.

Every other field at any level is never printed. A status outside this shape
is refused with the fixed message "Export connector status is malformed",
without echo, before the manifest file is written.

## Checks actually run

All with a private `CARGO_TARGET_DIR` under this worktree, `CARGO_BUILD_JOBS=1`,
`CARGO_INCREMENTAL=0` and `--locked`.

| Check | Result |
| --- | --- |
| `cargo test --manifest-path crates/riauthctl/Cargo.toml --no-default-features --locked --test m03_parity_ops export -- --test-threads=1` | 3 passed |
| riauthctl clippy `--all-targets -D warnings`, fmt, `--features terminal-usb` check | clean |
| `scripts/check-docs.py` | passed |
| `cargo test --features test-support --locked --test m03_connector_ldap_e2e -- --ignored` after building riauthctl | 1 passed: the real server's status flows through the validator, stored and then loaded after restart |
| riauthctl `Cargo.toml` and `Cargo.lock` against main | unchanged; `unsafe_code = "forbid"` |

The three export tests cover:

- active, stored-but-not-loaded and retired rows, with secret-bearing extras
  planted at status and row level, none of which is printed;
- an absent status and an empty status;
- 67 malformed shapes, each refused before the manifest is written and
  without echo.

No full suite, PostgreSQL, external LDAP, cloud or release-artifact check was
run. The LDAP evidence is a source-built loopback fixture.

Proposed CI addition for root: add `--test m03_connector_ldap_e2e` to the
"Check standalone client against the real server binaries" step.

## Independent review of `687ee6c`

A read-only Sonnet reviewer (reported model `claude-sonnet-5-5`) found no
high or medium issue. It confirmed:

- **Preservation:** the delta against main is one hunk in `export` plus the
  new helpers.
- **Allowlist:** the output is built only from validated copies, and the
  error text is fixed.
- **Shape agreement:** every status main's server can emit is accepted; the
  1024 bound is above the server's per-kind caps.
- **Write order:** a refusal happens before the manifest write.
- **Tests:** they would fail both on main's export and on a version that
  copied extras. The fixture blob equals `4cf53d2`.

Low residuals, not changed in this slice:

- **L1, fail-closed status:** an explicit `"connectors": null` is refused,
  while `riauth export` treats it as absent. Main's server never emits null.
  A future server shape (a fifth kind, another digest form) would block
  `riauthctl export` until the client is upgraded.
- **L2, looser than the server:** a null `revision` or `digest` is accepted on
  a non-retired row, and `loaded_digest` is copied from any row. None of
  these can carry a secret.
- **L3, test gaps:** nothing pins dropping `retired: false`, a null
  `loaded_digest`, exactly 1024 rows, or the directory unset with stored rows.
- **L4, pre-existing in main:** the summary's `revision` is copied from the
  server without validation.

## Closure mapping: M03

Original acceptance: "GUI, CLI, and API must share authorization,
validation, transactions, idempotency, and audit behavior." Gate: "The same
change has the same permission checks and outcome regardless of which
interface submits it."

| Resource | Interfaces in main (API, browser, server CLI, riauthctl) | Shared writer and outcome | Remaining concrete work |
| --- | --- | --- | --- |
| Users | all four | one writer; the changed-email verification rule now in the shared writer | none known |
| Groups and reviewed memberships | all four | one writer; distinct author, reviewer and executor | none known |
| Clients: create, policy, endpoint, status, secret rotation | all four | one writer; reviewed changes on all four | Reviewed client-creation execution keeps the generated secret in its receipt for 24 hours while every other issuer stores a marker; needs a decision (wave 28 report options) and implementation |
| Delegated grants, immediate and reviewed | all four | one writer; browser immediate path added | none known |
| Agents | API, server CLI, riauthctl; no browser | one writer; marker receipts | Browser has no agent management; product decision whether it should |
| Registration templates, signing keys, invitations, Windows devices, client and RADIUS certificates, sources | API, server CLI, riauthctl; browser only for invitations | one writer each; marker receipts for issued credentials | Browser coverage is a product decision; riauthctl keys, invitations, certificates and sources have mock-level proof only |
| Temporary access (PAM) | all four | one writer | Browser requires `Idempotency-Key` and `If-Match`; the bearer API accepts neither and checks `If-Match` only when sent. Needs a decision (options recorded) |
| Retry headers for full human administrators | API, browser, both CLIs | per route | Grant set, reviewed stage/approve/execute/cancel, `source_put` and registration-template creation accept a full human administrator without headers, while peer writes return 428; the browser and riauthctl always send both. Needs a decision |
| Offboarding, directory and SCIM operations | API, server CLI, riauthctl; browser for deactivation dismissal and cloud schedule | one writer | riauthctl directory and provisioning have mock-level proof only |
| SSF administrator streams, backup | API, server CLI, riauthctl | one writer; backup verified client-side | SSF real-server proof not available locally (needs a Platform build with a valid JWKS) |
| Desired-state plan, apply, export | API, server CLI, riauthctl; browser for workflow plans | one writer; removal confirmation by exact plan id | none known |
| Workflow review, activation, revocation | all four | shared activation hook with live replay | none known for this lane |
| Real-binary parity in CI | CI runs eight riauthctl fixtures and the redb backend comparison | | Full four-backend PostgreSQL comparison and the LDAP connector fixture are not in CI |
| Released artifacts | none | | Official build artifacts and an installed-binary proof are external prerequisites |

## Closure mapping: M07

Original acceptance: "Manage workflows, roles, connectors, and other
supported resources through versioned APIs and manifests—not GUI-only state."

| Resource | Versioned API | Manifest | Remaining concrete work |
| --- | --- | --- | --- |
| Workflow definitions | yes | `workflows` | none known |
| Workflow approvals | bearer API, server CLI, riauthctl, browser | not a manifest resource (approval is a reviewed action) | none known |
| Roles (delegated grants) | yes | `delegated_grants` (privileged changes stay reviewed) | none known |
| Connectors: LDAP, Workspace, Entra, SCIM | desired-state API and export status | four tables plus `retired_connectors`, operator pins | This port awaits root review and integration. Activation happens at process start, per process, not live. Per-connector reconciliation modes and controllers remain `riauth.toml`-only. `connector_definitions_postgres` is not in CI |
| SSF administrator streams | yes | `ssf_streams` | none known |
| Agents, registration templates, signing keys, certificate bindings, Windows devices | yes | not manifest resources (one-time credentials or relative lifetimes) | Not GUI-only; manifest coverage is a product decision |
| Listeners, PAM approvers, reviewed-membership and reviewed-client settings | no | `riauth.toml` only | Kept operator-only by design; a change would need a security decision |
| Admin passkey-first user creation | browser ceremony | none | Interactive WebAuthn ceremony; the only browser-only management state found |
| External evidence | | | Live LDAP, Workspace, Entra and SCIM peers, deployed multi-node PostgreSQL, and released artifacts |

Neither task is recommended for closure from this slice. M03 still has the
three contract decisions, real-server proof for the mock-only riauthctl
verbs, PostgreSQL parity in CI and released-artifact evidence. M07 still has
this port's review and integration, its CI coverage, the restart-only
activation and operator-only controller limits, and external connector
evidence.
