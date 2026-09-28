# Q02 shared contracts: identity, credential, connector and job slices

The same bodies in [shared.rs](shared.rs) run through the factories in
[backend.rs](../common/backend.rs). [contracts.rs](../contracts.rs) selects
plaintext redb, encrypted redb, plaintext PostgreSQL and encrypted PostgreSQL.
Every case uses the real `Core`/`Store` implementation. Backend branches select
storage and custody configuration, never the expected security outcome.

These slices follow the reviewed A01 INV-1–INV-4 recommendation and the Q01
[invariant catalog](../../docs/security/invariants.md). They exercise the current
single server implementation. It does not establish Essentials/Platform,
architecture, interface or released-artifact parity. Production behavior,
dependencies, lockfiles and the Q01 documents are unchanged.

| Shared contract | Q01 / Q02 coverage and persisted-state oracle |
| --- | --- |
| `identity_and_issuer_continuity` | RI-ACC-001, RI-DIST-001 / C01: preserve local IDs, issuer, signing keys, live sessions, subject and refresh continuity on reopen; reject another issuer without changing records. |
| `disable_reenable_revokes_dependents` | RI-SES-004, RI-MGT-004 / C01/C04/C08: core, manifest/apply and SCIM PATCH disable writers; an originally unspent code rejects both while disabled and after re-enable without state or audit mutation; old sessions/access/refresh and child agents/devices remain revoked; SSF and RP logout queue once. Existing `common::security` assertions are reused. |
| `proof_account_session_request_binding` | RI-SES-002/003 / C03: real password sessions and request-bound proof through authorization; reject another account, same account's other session, nonce and identical request's other instance; rightful consumption creates one code and cannot be repeated. |
| `code_binding_and_verified_replay` | RI-SES-003 / C03: client, redirect and PKCE rejection leave the grant/family intact; correctly bound replay revokes only its family. |
| `refresh_rotation_and_verified_replay` | RI-SES-003/005 / C03: wrong-client use cannot spend or revoke; rotation replaces the handle; bound replay revokes original and replacement grants while unrelated authority survives. |
| `live_group_policy_revalidation` | RI-AUTH-001 / C05: membership removal blocks authorization, UserInfo, proxy access and refresh of already-issued credentials; unrelated client/session remains usable. |
| `prepared_authority_revalidation` | RI-STORE-001 / C08: the existing prepared-store primitive accepts an interleaved real account disable, then rejects staged effects against its stale user/session reads. |
| `prepared_deadline_revalidation` | RI-STORE-001, RI-SES-005 / C08: the existing test clock crosses a read dependency's deadline without a write; no staged effect commits. Requires `test-support`. |
| `last_admin_failure_is_atomic` | RI-ACC-002, RI-STORE-001 / C04/C08: a password/history update staged before last-admin rejection rolls back all records, indexes and audit; the original credential still works. |
| `http_mutation_receipts_and_audit` | RI-MGT-001/003/004, RI-STORE-001/002 / C04/C08/C09: real HTTP fingerprints and If-Match, actor-scoped receipts, exact retry result/secret, conflicting reuse, permission reduction/revocation, one committed audit per mutation and selected secret redaction. |
| `plan_binding_atomicity_and_retry` | RI-MGT-002/004, RI-STORE-001/002 / C04/C08/C09: actor/issuer/content/revision binding, preview isolation, missing-secret rollback after staged group creation, exact apply retry, one apply audit and selected plan/export/audit secret redaction. |
| `application_writers_share_management_seam` | RI-MGT-001/002/003/004, RI-STORE-001/002 / C04/C08/C09 (M03 first slice): one application write seam behind HTTP (`/api/clients`, the CLI's transport) and desired-state reconcile. Both writers refuse confidential↔public type changes with the same error and no state change; `client.write` alone cannot rotate or change authentication through either; an ambient SSO cookie with `X-riAuth-Portal` cannot reach management writes; direct retries replay the exact result/secret, stale If-Match and stale plans change nothing; each committed direct write advances revision once with one correlated `client.*` audit, plan apply records one `client.reconcile` and one `state.apply`; no secret, digest or agent credential appears in audit, plan or export. |
| `password_attempts_and_change` | RI-CRED-001/002, RI-STORE-001 / C02: failed attempts persist without changing the credential; lockout denies a valid password; admin reset clears attempts and revokes old authority; rejected password reuse is atomic; owner reauthentication changes the password once while another account remains live. |
| `totp_and_recovery_code_binding` | RI-CRED-001/002 / C02: enrollment is account/session-bound; invalid confirmation leaves state unchanged; confirmation revokes old authority; a TOTP step cannot be replayed; only the owner's fresh MFA session can rotate recovery codes; wrong password preserves a code, but a valid factor with rejected transaction binding consumes it without creating a session. |
| `passkey_ceremony_binding_and_replay` | RI-CRED-001/002, RI-STORE-001 / C02: a wrong-account or same-account/different-session registration finish cannot enroll; a duplicate finish cannot enroll twice; authentication rejects another account's transaction, wrong origin and stale counter, with each specific ceremony present beforehand and absent afterward; only the owner can remove the key, which revokes the live passkey session and invalidates an older challenge. |
| `account_reset_binding_and_atomicity` | RI-CRED-003, RI-SES-004, RI-STORE-001 / C02/C03: unknown/unverified request responses do not enqueue mail; email change and wrong-purpose/reused-password completion reject atomically; a successful reset consumes its proof once, revokes old authority, preserves TOTP/recovery factors and requires the new password plus a one-time factor. |
| `account_proof_supersession_and_expiry` | RI-CRED-003, RI-SES-003/005 / C02/C03: reset request throttling leaves the current proof intact; a later request replaces that proof; the superseded code cannot change a credential; the current code resets once and revokes its old session; an exact-deadline code cannot mutate the user or audit. Requires `test-support`. |
| `verification_proof_binding_and_replay` | RI-CRED-002/003 / C02: a stale session cannot request verification; wrong-purpose and old-email codes leave state unchanged; a current code verifies only its bound account/email once, while the other account remains unverified. Requires `test-support`. |
| `invitation_acceptance_revalidates_creator` | RI-CRED-003, RI-MGT-004, RI-STORE-001 / C02/C04: duplicate and wrong-purpose requests are atomic; revoking a scoped creator blocks acceptance without enabling the pending user or adding group membership; an administrator's separate invitation succeeds once, with one acceptance audit. |
| `passkey_registration_requires_user_verification` | RI-CRED-001/002 / C02: an unverified authenticator's response consumes only its registration ceremony; it cannot enroll a key, change the user or revoke sessions; a verified response enrolls once and revokes the old account session. |
| `passkey_management_requires_fresh_mfa` | RI-CRED-002, RI-SES-005 / C02: password-only sessions cannot enroll or remove after a passkey exists; a previously sufficient MFA session becomes too old at 301 seconds without mutating state; an expired login ceremony cannot mint a session; fresh passkey MFA permits one removal and revokes account sessions. Requires `test-support`. |
| `offboard_intent_durable_cancel` | RI-CON-004, RI-MGT-004, RI-STORE-001 / C06/C08: a scoped actor creates one durable scheduled intent that survives reopening; duplicate/denied calls add no job or audit; cancellation and its exact retry do not disable the user or execute the job. |
| `offboard_retry_rechecks_authority` | RI-CON-004, RI-MGT-004, RI-STORE-001 / C06/C08: an injected precommit failure keeps the user live and records a durable retry without execution audit; a second worker cannot claim the live lease; revoking the creator before commit gives one terminal job failure and one matching audit, with credential/session state intact. Requires `test-support`. |
| `cloud_snapshot_apply_atomic_retry` | RI-CON-001/002, RI-MGT-004, RI-STORE-001 / C06/C08: a real loopback Workspace feed creates one durable reviewed plan; wrong actor, malformed/partial/changed snapshots and reduced user authority cannot apply it; reduced authority denies before a fetch, added authority invalidates the review, and a second-entry local collision rolls back the first staged account with no partial user/index/binding/audit mutation; restored authority applies once and an exact retry makes no network call or state change. |
| `database_native_restore_policy` | RI-STORE-004, RI-SES-004 / C10: a database-native copy (PostgreSQL template clone, copied redb file) holding live sessions, grants and a pending code is recovered: PostgreSQL detects the changed lineage on open, a second connected handle cannot run recovery, and the operator command gives the same end state on every mode. No restored session/grant/code works, epochs and revision advance by the stride, replay records, IDs and JWKS persist, reopening changes nothing, readiness waits for the attestation and new sign-in works. |
| `recovery_status_never_creates_or_writes_a_store` | RI-STORE-004 / C10: `recovery status` on a never-initialized store reports not serving and leaves no redb file or PostgreSQL schema; on a store with a pending gate it reports the gate and leaves every record (and the redb file bytes) unchanged. |
| `indexed_user_group_membership` | RI-STORE-001 / C08: group membership index pages preserve ordered results across more than one page, mutations, rollback, reopen/rebuild and snapshot interleaving on each backend mode. |

Snapshot comparisons include metadata, revision, indexes, queues, replay records,
receipts and audit. They compare an individual fixture before/after a transition;
random IDs, keys and timing are not compared between unrelated instances. Oracles
derive from required outcomes and public behavior, not the implementation helper
being tested. Authorized secret returns and recoverable receipts are intentional;
the redaction contracts cover ordinary audit/plan/export output.

The HTTP mutation oracle allows only the separate middleware request-counter
records (`http_rates/`, `index_expiry_http_rates/`, `index_counts/http_rates`) to
change on a rejected request or retry. PostgreSQL persists these counters before
authorization; redb counts in per-node memory. This observed operational
difference is not normalized out of credential, receipt, revision or audit state.
Counter creation/counting is checked separately. Snapshot failures report changed
record names, never private keys or recoverable fixture values.

The cloud apply oracle allows only the connector's separate retry ledger
(`cloud_directory_runs/`) to change when a fetch fails or a remote snapshot
differs from the plan. User, binding, plan, revision and audit state remain in
the rejection snapshot. A terminal offboarding failure intentionally changes
its job, audit and indexes and advances revision by exactly one; the contract
checks that account, session and unrelated records do not change.

The database-native restore contract states one observed backend difference:
only PostgreSQL has a storage lineage, so only there does opening the copy apply
the policy before the operator command, and only there can a second store handle
open while recovery runs (redb's file lock refuses it). The final security
state after `recovery invalidate` is the same oracle on all four modes.

## Run

From the repository root, ordinary tests need no external services:

```sh
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=target CARGO_PROFILE_DEV_DEBUG=0 cargo test --locked --features test-support --test contracts
```

This selects 60 redb cases and explicitly ignores 60 PostgreSQL cases. Without
`test-support`, five clock/deadline bodies are absent: 50 run and 50 are ignored.
A PostgreSQL skip is not backend evidence.

Install/use local PostgreSQL programs (`initdb`, `pg_ctl`) and run:

```sh
CARGO_PROFILE_DEV_DEBUG=0 bash scripts/test-contracts-postgres.sh
```

The script creates a fresh loopback-only cluster under `target/`, a private
connection file and a marker, then runs the 60 ignored PostgreSQL cases with
`CARGO_BUILD_JOBS=2`. Each fixture creates its own empty database. Before any
database creation/drop, the fixture checks the actual server data directory
against the cluster's `primary` directory, rather than trusting only an environment
variable or marker. Pools close before their database is dropped; the script stops
and removes only its own cluster. Do not supply a production connection. There is
no automatic redb fallback if PostgreSQL setup or a contract fails.

The public check job's existing all-target test command includes ordinary cases;
the integration job invokes this script separately from replication/failover.
If PostgreSQL programs or local execution are unavailable, report that prerequisite
and the ignored cases explicitly. The disposable contract cluster does not test
PostgreSQL TLS, replication, failover, migration, production fencing or actual
multi-process races.

## Future matrix expansion

These bounded slices cover the current shared `Core`/`Store` contract foundation.
The matrix below lists broader invariant work; no row claims whole-domain
acceptance.

| Q02 matrix | Current shared evidence | Still open |
| --- | --- | --- |
| C01 identity/session | Stable issuer/identity reopen and selected disable/re-enable writers. | Browser/CLI adapters, all opaque classes and actual distribution transitions. |
| C02 credential/recovery | Password, TOTP, recovery, passkey and selected mail proofs. | Other password writers, mail classes, browser/CLI completion, device/directory paths and other passkey ceremonies. |
| C03 proof/grant binding | Selected session, code, refresh and mail proofs. | Resource, scope, key, algorithm, OAuth/SAML/DPoP/assertion and other proof paths. |
| C04 management parity | Selected direct core, HTTP, plan/receipt and scoped creator checks; application writes share one seam across HTTP, CLI (`tests/cli.rs`) and plan apply, with the browser cookie boundary refused. | Users, groups, SCIM, directory, invitation and dynamic-registration writers; full actor/role/retry matrix. |
| C05 policy consumers | Group policy at selected online consumers. | PAM, client/resource/ACR/device/source/consent and wider adapters. |
| C06 connector/jobs | One Workspace plan/apply and local offboarding intent, retry and authority paths on four backend modes. | Entra, LDAP, SCIM/provisioning, other offboarding states, malformed/large/empty snapshots, remote partial completion and worker crash/recovery. |
| C07 workflow/device | No shared body. | Required stages, device/certificate/peer binding and future adapters. |
| C08 transaction/audit | Four-mode atomicity, HTTP receipts, plan rollback, prepared checks, cloud staged-write rollback and offboarding fault/audit cases. | All writers, signer faults, actual interleavings and transport parity. |
| C09 secrets | Selected plan/export/audit redaction. | Errors, logs, metrics, ordinary CLI and custody modes. |
| C10 recovery | Database-native restore policy and read-only recovery status on four backend modes; restore-path policy in `tests/recovery.rs` (redb output only). | Archive integrity matrix, factor/agent/device/receipt/job reconciliation, interrupted recovery and external dependencies. |
| C11 build/runtime | No two-build claim. | Compiled product boundaries and transition matrix. |

Exact resolved-secret byte approval and cached-result resource authorization
remain Q01 review questions. Selected audit assertions do not prove every
writer's audit parity. PostgreSQL backend parity here does not imply
product/build parity.

Prepared tests lift existing store regressions and make no claim about actual
password or signer operations paused during verification. Q05's broader replay,
concurrent-winner, revocation, expiry and interruption schedules remain separate
work. The current-source Q02 contract foundation is ready for scoped acceptance
review; architecture/engine integration and two-build or interface parity need
their own evidence.

## Recorded validation

Wave 4 local validation on 2026-09-28 used the existing Rust and PostgreSQL
toolchain at HEAD `c8060b7bb86b28915ed166ca0eccd64435aa2750`. Every Cargo
build/test/lint command used `CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=target
CARGO_PROFILE_DEV_DEBUG=0`; the PostgreSQL script supplies the first two and
uses the worktree's absolute `target/` path. Exact logs and the scoped Q02
coverage decision are in `target/riwork/Q02-wave4-acceptance.md`.

| Command/check | Result |
| --- | --- |
| `cargo test --locked --features test-support --test contracts -- --quiet` | Exit 0: 46 passed on plaintext/encrypted redb, 46 PostgreSQL ignored. |
| `CARGO_PROFILE_DEV_DEBUG=0 bash scripts/test-contracts-postgres.sh` | Exit 0: 46 passed on isolated plaintext/encrypted PostgreSQL, 46 redb filtered; disposable cluster removed. |
| `cargo test --locked --test contracts -- --quiet` | Exit 0: 36 passed, 36 PostgreSQL ignored; five bodies require `test-support`. |
| Four focused `tests/identity.rs` auth/passkey regressions | Exit 0: one passed per filter. |
| Featured and default `cargo clippy --all-targets --locked -- -D warnings` | Exit 0 in both configurations, no warnings. |
| `cargo fmt --all -- --check`, `python3 scripts/check-docs.py`, `python3 scripts/check-repo-hygiene.py`, `bash -n scripts/*.sh`, `git diff --check` | Exit 0. Tracked hygiene checked 227 files; working-file hygiene is recorded in the wave 4 handoff. |

The redb command's PostgreSQL ignores count only as skips; the isolated-cluster
run supplies backend evidence. No production code, dependency, lockfile or Q01
document changed. No security invariant violation was demonstrated by these
corrections. The earlier wave 3 handoff records offboarding, cloud-directory
and storage scenario suites that were not repeated in wave 4.
