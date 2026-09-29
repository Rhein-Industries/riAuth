# D05 acceptance evidence matrix

Status: **not passed.** Snapshot date 2026-09-29. This D05 branch was not
reset onto accepted integration. The parent snapshot is `328a3ac`, whose
base is `cf827a9ee103672b329907a433fb96ac868fe178`. Accepted files were
read from ledger head `03124272a6cfe14ac48227c504cc896c2f11d9aa`. Every
category stays **not passed**. D05 stays **not passed**. The workstream
gate — a new user and a new operator can independently complete the
documented workflows — has no recorded completion.

The previous snapshot is `09e68460d7836be8227568fbc84d8516d2d23e84`, the
accepted form of `054ab05`. It described head `52df9a3`. This page keeps those
records and adds the slices accepted after them. A status other than
`not_passed` is outside this slice. The machine-readable twin is
[d05-acceptance-evidence.json](d05-acceptance-evidence.json).

## What was read

| Source | Identity |
| --- | --- |
| This worktree | `/Users/dominik/orca/projects/riAuth-public-preview-roadmap-d05-acceptance-evidence-wave24`, branch `roadmap/d05-acceptance-evidence-wave24`, parent snapshot `328a3ac` on base `cf827a9`. This refresh writes only the two D05 evidence files |
| Ledger head | `/Users/dominik/.local/share/riwork/orchestrators/projects/891e7443-8dac-4c1b-897f-9e53cb59c7ee/planning/accepted-commits.json`. The stored `integration_head` string is `0312427`. That prefix is commit `03124272a6cfe14ac48227c504cc896c2f11d9aa` |
| Accepted checkout | `/Users/dominik/orca/projects/riAuth-public-preview-roadmap-integration-accepted` was read at that same commit and was not edited, merged, or pushed |

The ledger `integration_status` is `local_commits_only_no_merge_to_main`.
It has 456 `reviewed_slices`, 436 `integration_validation.checks`, and 17
`open_checks`. D05 is one reviewed slice, integration commit `09e6846`,
status `task_in_progress`. The task that started this refresh named
`ee88af0d02d4a8c1d2f7a9a903f00890aeaef7b1` and 454 slices. That commit is
an ancestor. The ledger then included W07 at `3cfe28b` and Q06 at
`0312427`. The validation `head` and the Q06 `run_head` are stored as
the seven-character prefix `0312427`.

The previous snapshot's statement that `e926d75` had no ledger check is
stale. Ledger `run_head` `e926d75` records
`CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 cargo test --locked --test pam_management -- --quiet; python3 scripts/check-module-boundaries.py; git diff --check HEAD~1 HEAD`
with result text pass; 6 PAM management tests. The note says broader M03
management parity work remains. That run is source-only and does not pass
administration.

Sentences about commits after `cf827a9` were read from the accepted
objects. Paths that this worktree does not contain are named in backticks
and are not markdown links.

This slice did not run Cargo, a browser, a database, or a peer. Commands
below are citations of docs and of the ledger. They were not re-executed
here.

The [A01 coverage inventory](coverage-inventory.md) at base `96e23e2` is
historical planning evidence and is left unchanged. Category targets are
the frozen checks in [product contracts](product-contracts.md), including
G01–G12 and EG01–EG04. Criteria in that contract are not observed results.

## Evidence kinds

This snapshot uses three kinds. None of them passes a category by itself.

| Kind | Meaning |
| --- | --- |
| `source-only` | A file in the tree, or a recorded run of riAuth against itself or a disposable redb/PostgreSQL cluster the test created. The command may have passed. `run_head` is kept so an older pass is not described as a new one. |
| `local peer` | A third-party program on this host, loopback only. OpenLDAP `ldapsearch` and FreeRADIUS `radclient` are this kind. A named tenant, production directory, hardware NAS, or relying party is a different claim and is still absent. |
| `release/deployment` | A release-workflow artifact, an installed copy of one, or a rehearsal outside disposable fixtures. |

Rows in the category sections that still say `ancestor-local` or
`accepted-head local` are the `09e6846` wording. Read a riAuth or disposable
database run as `source-only`. Read the FreeRADIUS `radclient` run as
`local peer`. Read `release artifacts` as `release/deployment`. A
`missing measurements` row is a gap, not a fourth kind of passing evidence.

## Verdicts

| Category | Status | Strongest record at this snapshot | Gate that remains |
| --- | --- | --- | --- |
| Usability | not passed | source-only password invitation at `58e5ef3` and simulated passkey invitation at `0312427`, each 3/3 | Independent new-user completion, assistive technology, a hardware or synced passkey, external mail |
| Workflows | not passed | source-only configured consent at `a3ccbf1`: prior suite 8/8 and TOTP consent 1/1 | Browser and remembered consent, TOTP-only and recovery-code reauthentication, custom stages |
| Administration | not passed | source-only redb 11/11 and disposable PostgreSQL 9/9 at `cf4529e`, including encrypted reopen and fenced promotion | Human operator journey, broader dependency families, and resources still outside exact multi-party approval |
| Interoperability | not passed | local peer OpenLDAP `ldapsearch` 2.7.1 LDAPS and STARTTLS at `fcba8a5` | Active Directory, a named directory application, a SAML service provider, a hardware NAS, and a cloud tenant |
| Footprint | not passed | source-only macOS edition matrix at `4ca7558` | Linux release-artifact size and a measurement at `0312427` |
| Performance | not passed | source-only `q09-8u-8g` reports at `7aea083` with `performance_claim: false` | Named operator workload, RPO, and RTO |
| Availability | not passed | source-only shared-store refusal at `7e07e748`, separate processes, disposable PostgreSQL | Shared-job leases, peer health, native TLS, and a deployment RTO or RPO |
| Recovery | not passed | source-only disposable drills, 16/16, no source commit in the JSON | Release/deployment restore, key escrow, and a measured recovery time |
| Migration | not passed | source-only synthetic reference relying party at `cf827a9`, focused 1/1 | A real Authentik export, a named relying party, and a production route rollback |
| Security | not passed | source-only A03 source seam through `ee88af0`, and source SPDX binding at `b759216` | Installed-artifact EG03, certification, a release signature, and a release SBOM |

## Slices accepted after `09e6846`

W02, A03, Q04, O03, S04, and G05 each have a reviewed slice on this base.
G05's slice is a disposable in-process rehearsal. None of these slices
meets its category target.

### W02 — source-only

Ledger integration commit `a3ccbf1dd47a8e25f35ad8d5d505768f32c1b942`.
[Workflows](../workflows.md) now describes three configured consent graphs:
session then consent; session, passkey, then consent; and session, local
password, current TOTP, then consent. Recovery codes are not an alternative
in the TOTP graph.

Ledger `run_head` `a3ccbf1`:

```text
CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=4 cargo test --locked --test workflow_configured_totp_consent --test workflow_configured_consent -- --quiet; python3 scripts/check-docs.py; python3 scripts/check-module-boundaries.py; git show --format= --check HEAD; git status --short
```

Result text: pass; existing consent 8/8, new TOTP consent 1/1. The W02
ledger note says browser-initiated and remembered-consent adapters,
TOTP-only reauthentication, and recovery-code reauthentication remain
unsupported. The ledger `open_checks` entry for unfinished configured
shapes is still present. Workflows stay not passed.

### A03 — source-only

Four ledger checks are on this history:

| `run_head` | Command | Result text |
| --- | --- | --- |
| `97511e4` | `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 cargo test --locked --test cloud_directory cloud_connection_probe_rechecks_revocation_after_upstream -- --exact --quiet; python3 scripts/check-module-boundaries.py; git diff 52df9a3..HEAD --check` | pass; exact midflight revocation regression 1/1 |
| `3aa997b` | `CARGO_BUILD_JOBS=2 cargo test --locked --test cloud_directory cloud_credential_preflight_replays_before_revision_and_provider_access -- --exact --quiet; python3 scripts/check-module-boundaries.py; git diff --check; git status --short` | pass; focused 1/1, zero forbidden refs |
| `8704bb8` | `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=4 cargo test --locked --test cloud_directory cloud_credential_write_rechecks -- --quiet; python3 scripts/check-module-boundaries.py; git show --format= --check HEAD; git status --short` | pass; midflight revocation/revision 2/2 |

The `8704bb8` note says `cloud_operations.rs` has no direct store or
mutation call and that the broader protocol transaction inventory remains.
The open check still says seven protocol files refer to Core and seven
refer to storage.

`1535cdcfce622f9422f59572d7f6e9b31c597e56` is the later A03 integration
commit on this base. Ledger `run_head` `1535cdc`:

```text
CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=4 cargo test --locked --test source_boundary -- --quiet; python3 scripts/check-module-boundaries.py; git show --format= --check HEAD; git status --short
```

Result text: pass; source configuration security boundary 1/1, zero
forbidden refs, clean accepted tree. The test is
`source_configuration_keeps_scoped_receipt_revision_and_audit_order`.
The note says the source-configuration mutation moved from the protocol
facade into assembly and that other source protocol transaction sites
remain. These checks do not pass security or administration.

### Q04 — local peer

Ledger integration commit `fcba8a5e81037edb365c02c291b512576817e920`.
The Q04 note says OpenLDAP `ldapsearch` 2.7.1 exercised separate riAuth
LDAPS and STARTTLS loopback listeners: CA-verified paged service bind,
disabled-user omission, wrong and revoked token rejection, wrong-CA
rejection, and crossed-scheme transport failure. It supports I04. It
claims no Active Directory directory, named production directory
application, SAML service provider, hardware NAS, or supplicant.

Ledger `run_head` `1caa093`:

```text
LDAPSEARCH=/opt/homebrew/opt/openldap/bin/ldapsearch CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 scripts/test-ldap-provider.sh; bash -n scripts/test-ldap-provider.sh; python3 scripts/check-docs.py; python3 -m json.tool docs/roadmap/coverage-inventory.json; git diff e3c26eb..HEAD --check
```

Result text: pass; OpenLDAP ldapsearch 2.7.1 STARTTLS bind/paging/disable/revoke 1/1.

Ledger `run_head` `fcba8a5`:

```text
LDAPSEARCH=/opt/homebrew/opt/openldap/bin/ldapsearch CARGO_BUILD_JOBS=2 bash scripts/test-ldap-provider.sh; python3 scripts/check-docs.py; python3 scripts/check-repo-hygiene.py; bash -n scripts/test-ldap-provider.sh; python3 -m json.tool docs/roadmap/coverage-inventory.json; git show --format= --check HEAD; git status --short
```

Result text: pass; OpenLDAP 2.7.1 LDAPS+STARTTLS ignored peer test 1/1.

The ignored test is
`ldapsearch_starttls_bind_scoped_paging_disable_and_revoke` in
`tests/ldap_provider_peer.rs`. [Platform LDAP provider](../recipes/platform-ldap-provider.md)
records `ldapsearch -VV` as OpenLDAP 2.7.1 (Sep 8 2026 21:55:18), OpenSSL
3.6.4, loopback ports `127.0.0.1:60248` (LDAPS) and `127.0.0.1:60249`
(STARTTLS), paged `uid: ldap-alice` and `uid: ldap-bob`, invalid-credentials
exit 49, certificate-verify failure, disabled-user omission, and revoke.
The check job and the integration job do not call
`scripts/test-ldap-provider.sh`. Interoperability stays not passed.

The FreeRADIUS `radclient` 3.2.10 loopback PAP result at `38f82fe` is the
same kind: local peer, not a hardware NAS.

### O03 — source-only

Ledger integration commit `7e07e748a80b9cb39ffed3da8ce90690d72ab18c`.
[Node security](o03-node-security.md) says O03 stays open. Opening compares
issuer and active capability names, refuses a mismatch before migration or
listen, and leaves listen address, `browser_ui`, and `[process]` local.

Ledger `run_head` `7e07e748`:

```text
CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 cargo test --locked --offline --lib node_security:: -- --quiet; RIAUTH_PG_TEST_TARGET=node_security_postgres CARGO_BUILD_JOBS=2 bash scripts/test-postgres.sh; CARGO_BUILD_JOBS=2 cargo check --locked --offline --no-default-features --features essentials --lib; python3 scripts/check-docs.py; python3 scripts/check-repo-hygiene.py; bash -n scripts/test-postgres.sh; python3 -m json.tool docs/roadmap/coverage-inventory.json; python3 -m json.tool docs/roadmap/capability-matrix.json; git show --format= --check HEAD; git status --short
```

Result text: pass; redb 2/2, disposable PostgreSQL separate-process 1/1.
The page names `agreement_tracks_issuer_and_active_capabilities_only`,
`open_refuses_a_different_active_set_without_rewriting_the_store`, and
`capability_or_issuer_mismatch_binds_nothing_and_preserves_postgres`.
The PostgreSQL run used loopback HTTP without native TLS and
`local_unencrypted` PostgreSQL. It did not promote the standby. The note
says shared-job leases, distributed rate limit, peer health, policy and
trust and key agreement, and native TLS or HA proof are still absent.

Two later O03 checks are also source-only and do not record a deployment
RTO. Ledger `run_head` `acf1d7c`:

```text
cargo test --locked --features test-support --test logout_lease --test background_logout --test operations --test identity_boundary -- --quiet; cargo test --locked --features test-support --test operations logout_network_failures_are_visible_and_clear_after_success -- --exact --quiet; RIAUTH_PG_TEST_TARGET=job_lease_postgres CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 bash scripts/test-postgres.sh
```

Result text: focused pass for background logout 1/1, identity boundary
6/6, logout lease 1/1, operations logout 1/1, and disposable PostgreSQL
two-worker lease 1/1. The same result says the full operations suite was
11/15, with four backup and schema failures. Ledger `run_head` `75def67`:

```text
cargo test --locked --lib node_security:: -- --test-threads=2; RIAUTH_PG_TEST_TARGET=node_security_postgres bash scripts/test-postgres.sh
```

Result text: pass; redb unit 4/4 and disposable PostgreSQL process and
upgrade 3/3. The note leaves deployment, rollback, and real multi-node
evidence open. Availability stays not passed.

`36ad584` is the earlier process-role commit on this history. Its ledger
result is process-role 6/6 and two-process disposable PostgreSQL 1/1.
That is source-only as well. Public CI, as the node-security page states,
does not select `node_security_postgres`.

### S04 — source-only

Ledger integration commit `e3c26eb2879729ce7971852d62ffdfb0e5083803`.
Ledger `run_head` `e3c26eb`:

```text
RIAUTH_PG_TEST_TARGET=reviewed_memberships_postgres CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 scripts/test-postgres.sh; bash -n scripts/test-postgres.sh; python3 scripts/check-docs.py; git diff 09e6846..HEAD --check
```

Result text: pass; disposable PostgreSQL reviewed-membership suite 6/6,
including fenced standby promotion and receipt/audit replay. The sixth
function is `postgres_z_fenced_standby_promotion_replays_reviewed_membership`.
[Reviewed group memberships](../reviewed-group-memberships.md) says the
test stops the primary with `pg_ctl -m immediate`, promotes the standby
with `pg_ctl promote`, and reads the same membership, audits, and receipt
on the promoted port. It is a loopback drill with trust authentication.
It does not elect a leader, measure a recovery objective, or cover
failback, partitions, or PITR. The encrypted TLS test from `52df9a3`
remains a separate function. That 6/6 result is the `e3c26eb` run. It is
not the latest accepted suite.

Later accepted S04 runs, still source-only, are on the ledger head's
history:

| `run_head` | Command | Result text |
| --- | --- | --- |
| `60e69f3` | `CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 cargo test --locked --test state_reconciliation -- --quiet; RIAUTH_PG_TEST_TARGET=reviewed_memberships_postgres CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 bash scripts/test-postgres.sh; python3 scripts/check-docs.py; git diff --check HEAD~1 HEAD` | pass; 7 redb state reconciliation and 7 disposable PostgreSQL reviewed-membership tests |
| `ef496c8` | `CARGO_BUILD_JOBS=2 cargo test --locked --test state_reconciliation client_name_desired_state -- --quiet; RIAUTH_PG_TEST_TARGET=reviewed_memberships_postgres CARGO_BUILD_JOBS=2 bash scripts/test-postgres.sh; python3 scripts/check-docs.py; python3 scripts/check-repo-hygiene.py; git diff --check HEAD^ HEAD` | pass; redb client-name 2/2, disposable PostgreSQL 8/8 with encrypted reopen and standby promotion |
| `cf4529e` | `cargo test --locked --offline --test state_reconciliation -- --test-threads=8 --skip desired_state_client_and_password_disables_require_review; RIAUTH_PG_TEST_TARGET=reviewed_memberships_postgres ./scripts/test-postgres.sh` | pass; redb 11/11, disposable PostgreSQL 9/9 including encrypted reopen and promotion |

The `cf4529e` redb command skips
`desired_state_client_and_password_disables_require_review`. The 11/11
count is the tests that command ran. At accepted `ee88af0`,
`tests/reviewed_memberships_postgres.rs` has these nine PostgreSQL tests:
`postgres_client_name_desired_state_dependencies_and_replay`,
`postgres_group_desired_state_dependencies_and_replay`,
`postgres_unrelated_revision_still_applies_reviewed_membership`,
`postgres_affected_membership_user_and_policy_deny_stale_apply`,
`postgres_reviewed_membership_reopen_and_receipt_replay`,
`postgres_reviewed_membership_http_execute_race_replays_after_reopen`,
`postgres_encrypted_reviewed_membership_replays_across_reopen`,
`postgres_user_display_name_desired_state_dependencies_and_replay`, and
`postgres_z_fenced_standby_promotion_replays_reviewed_membership`.
Accepted `docs/testing.md` at `ee88af0` says the same runner applies one
group-only desired-state plan, one client display-name plan, and one user
display-name plan. A manifest that also names another family, changes
client scopes, or changes a user's email still conflicts on the global
revision. The promotion sentence is unchanged: loopback trust
authentication, no leader election, no recovery objective, no failback,
no partition, and no PITR.

The `cf4529e` note says broader families and real deployment evidence
remain open. Administration and availability stay not passed.

Ledger `open_checks` entry 14 still says S04 redb concurrency plus
PostgreSQL 3/3, and that standby promotion and encrypted PostgreSQL
remain open. That sentence is stale. `52df9a3` recorded encrypted
reopen, `e3c26eb` recorded promotion at 6/6, and `cf4529e` recorded
redb 11/11 with PostgreSQL 9/9 including both. Correcting the sentence
does not pass S04, administration, or availability.

### G05 — source-only

Ledger integration commit `cf827a9ee103672b329907a433fb96ac868fe178`,
status `task_in_progress`. [G05 local reference OIDC](g05-local-reference-oidc.md)
calls the slice a bounded local source rehearsal. The requested real
target application has not been supplied. No production Authentik export,
relying party, routing change, or rollback was used.

The test is `local_reference_rp_cutover_rehearsal` in
`tests/g05_reference_oidc.rs`. Its source SHA-256 is
`64e43530f6c6e69c2fe4f7f9d45fabb2cfb5b1843a6b022eec2a418008602ba6`, the
hash the page records. The page's own observation command used a private
Cargo target from base `a3ccbf1` plus that test source. The accepted
ledger command, `run_head` `cf827a9`, is:

```text
CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=4 cargo test --locked --test g05_reference_oidc -- --nocapture; python3 scripts/check-docs.py; python3 scripts/check-repo-hygiene.py; git show --format= --check HEAD; git status --short
```

Result text: pass; synthetic reference OIDC rehearsal 1/1 with eight
asserted case IDs. The page names those IDs G05-01 through G05-08:
import and preflight, denied access, successful access and claims,
refresh rotation and replay, MFA, a new riAuth recovery code, logout,
and a local route-file rollback boundary. The test uses a disposable
redb fixture and an in-process reference relying party. It starts no
browser, network listener, or external Authentik service. The route-file
check restores a disposable JSON file and does not prove that Authentik
credentials still work.

That run is source-only. It is not a local peer, and it is not a
release/deployment rehearsal. [Migration](../migration.md) says these
in-process results do not close the real application cutover or rollback
gate. Migration stays not passed.

### O06 — source-only

Three bounded diagnostics are on the ledger. Each page read from
accepted `ee88af0` says O06 stays open. None is a dashboard deployment.
`doctor.healthy`, `/readyz`, and `/livez` keep their existing answers.

| `run_head` | Command | Result text |
| --- | --- | --- |
| `7949aeb` | `CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 cargo test --locked --test offboarding offboarding_diagnostics_reports_incomplete_and_failed_without_secrets -- --exact --quiet; python3 scripts/check-docs.py; python3 scripts/check-repo-hygiene.py; git diff --check ba64b66..HEAD` | pass; focused diagnostics 1/1 |
| `7949aeb` | `CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 cargo test --locked --test offboarding -- --quiet` | FAIL 24/30. The note says two representative failures reproduce on pre-O06 `ba64b66`, and four were not baseline-checked |
| `644242f` | `cargo test --locked --test reconciliation_diagnostics -- --quiet; python3 scripts/check-docs.py; python3 scripts/check-repo-hygiene.py; python3 scripts/check-module-boundaries.py` | pass; O06 diagnostic 1/1 |
| `9a575d5` | `CARGO_BUILD_JOBS=2 cargo test --locked --offline --test offboarding_deactivation_diagnostics -- --test-threads=1; CARGO_BUILD_JOBS=2 cargo test --locked --offline --test offboarding -- --test-threads=4; python3 scripts/check-docs.py; python3 scripts/check-repo-hygiene.py; git diff --check 3f0a9c9..HEAD` | pass; deactivation diagnostics 2/2, offboarding 31/31. The result text says PostgreSQL was ignored in this command and attributes disposable PostgreSQL 1/1 to the source run |

The 24/30 failure is that earlier check. It is not the last offboarding
result. `6121456` later records the full offboarding suite 31/31 twice.
O06 stays `task_in_progress`.

`docs/roadmap/o06-offboarding-diagnostics.md` caps the job aggregate at
50 items and 32 non-succeeded targets, and it scans the whole
`offboard_jobs` bucket with no incomplete-downstream index. Its open
list is dashboards, connector lag, node mismatch, storage pressure and
key problems, failed jobs outside scheduled offboarding, doctor and
`queues.offboard_jobs.failed`, deactivation rows, and a production
deadline for the full job scan.

`docs/roadmap/o06-deactivation-diagnostics.md` pages
`provisioning_deactivations` by 128 and retains at most 50 attention
rows. Each stored value is still decoded in full and has no size cap on
that path. Its open list adds Essentials redacted aggregate, a
`riauthctl` or new `riauth` diagnostics command, deactivation dispatch,
the connector due cursor, mail, provisioning-job error text, and
Prometheus or Grafana.

`docs/roadmap/o06-reconciliation-diagnostics.md` caps the controller
failure aggregate at 50 rows. Its open list adds the 256-job retention
as short of a production backlog deadline, and it does not extend the
offboarding aggregate. Administration stays not passed.

### Q06 — source-only

Ledger `run_head` `58e5ef3`:

```text
(cd tools/browser && npx playwright test invitation-password.spec.js --reporter=line); node --check tools/browser/invitation-password.spec.js; git diff --check b77e637..a4d0a1f
```

Result text: pass; accepted Chromium, Firefox, and WebKit browser
journey 3/3. The note says the test covers keyboard and mobile-viewport
password invitation acceptance, expired and replay rejection, no session
on acceptance, later sign-in and sign-out, and zero passkey requests.
It does not add a physical or synced passkey, a real mobile device, a
screen reader, or external mail. An earlier Chromium virtual-authenticator
invitation at `a72a086` is a separate 1/1 and leaves the same hardware
and mail gates open.

Ledger `run_head` `0312427` is the later simulated passkey invitation:

```text
(cd tools/browser && npx playwright test invitation-passkey-shim.spec.js --reporter=line); node --check tools/browser/invitation-passkey-shim.spec.js; python3 scripts/check-docs.py; git diff --check HEAD^ HEAD
```

Result text: pass; simulated passkey invitation Chromium, Firefox, and
WebKit 3/3. The note says the shim sets user verification itself. It does
not prove browser-native, hardware, synced, phone, mobile OS, screen
reader, or external mailbox behavior. Both journeys use the in-tree
portal fixture. They are source-only, not a local peer. Usability and
workflows stay not passed.

### W07 — source-only

Ledger `run_head` `f911e85` rejects a guest body above the translation
fuel or timeout budget before Wasmi `Module::new`. Commands:

```text
cargo test --locked --offline --features platform --lib extension_gate:: -- --test-threads=2
cargo test --locked --offline --no-default-features --features essentials --lib extension_gate:: -- --test-threads=2; python3 scripts/check-docs.py; git diff --check
```

Result text: pass; Platform extension gate 9/9, then Essentials 3/3.
The note says a guest that fits still validates and executes on the
caller without a wall-clock interrupt, and the native host remains
unwired.

Ledger `run_head` `3cfe28b`:

```text
CARGO_BUILD_JOBS=2 cargo test --locked --offline --features platform --lib extension_gate:: -- --test-threads=2; CARGO_BUILD_JOBS=2 cargo test --locked --offline --no-default-features --features essentials --lib extension_gate:: -- --test-threads=2; python3 scripts/check-docs.py; git diff --check HEAD^ HEAD
```

Result text: pass; Platform extension_gate 10/10, Essentials 3/3. The
note says guest output reads at most one declared workflow label, capped
at 32 bytes, before copying from Wasm memory, and a larger length is
denied without a read. Pinned Wasmi still lacks safe wall-clock
interruption for a pure guest call. The native host and other custom
graphs remain held. Workflows and security stay not passed.

### Q11 — source-only

Ledger `run_head` `b759216`:

```text
python3 -m unittest discover -s tests -p test_spdx_sbom.py -v; python3 -m unittest discover -s tests -p test_release_evidence.py -v; python3 -m unittest discover -s tests -p test_installed_release_gate.py -v; python3 scripts/check-release-evidence.py; bash -n scripts/package-release.sh; python3 scripts/check-docs.py; git diff --check HEAD^ HEAD
```

Result text: pass; SPDX 33/33, release evidence 14/14, installed gate
6/6. The source audit reports no release execution and no produced
release SBOM. Accepted `docs/roadmap/q11-release-evidence.md` says
`package-linux` binds the Linux packager to exact Essentials, Platform,
and riauthctl archive and image bytes, and that the fixtures do not run
`package-release.sh`. `sbom.release_sbom_produced` and release execution
stay false. `sbom.packager_source_calls_producer` true records a source
call, not a packager run. Open gates on that page: a signature, a release
SBOM from a real packaged artifact run, a separate review record, a
published asset set checked for a named tag, and Linux ARM64 execution.
The installed-gate unit result is source-only. It is not a
release/deployment artifact. Security, footprint, and recovery stay not
passed.

### A03 source seam — source-only

Accepted source-transaction moves end at ledger head's parent
`ee88af0d02d4a8c1d2f7a9a903f00890aeaef7b1`. Ledger `run_head` `ee88af0`:

```text
CARGO_BUILD_JOBS=2 cargo test --locked --offline --features test-support --test identity browser_link_finish_needs_the_original_fresh_local_session_and_rolls_back_whole -- --test-threads=1; python3 scripts/check-module-boundaries.py; python3 scripts/check-docs.py; git diff --check HEAD^ HEAD
```

Result text: pass; browser link rollback and one-use regression 1/1.
The note says `source_finish_browser` moved into assembly without
changing bind and deliver rollback, the stage and workflow guard, factor
charging, or audit. Wider A03 module-boundary work remains. Earlier
notes on this seam named leftover source writes at `a2481b0` and
`3f0a9c9`; those sentences belong to those commits. This slice did not
re-count protocol files. Ledger `open_checks` entry 7 still says seven
protocol files refer to Core and seven refer to storage, and that cloud
plan, apply, and snapshot cleanup remain in the protocol surface.
Security and administration stay not passed.

## Usability

Target: ordinary account and admin journeys, including A02 E04–E11, on a
declared browser matrix with keyboard, zoom, and assistive technology, plus
the workstream gate that someone other than the author completes the
documented flows.

| Class | Record |
| --- | --- |
| source-only | [Essentials guide](../essentials-guide.md) and [Platform guide](../platform-guide.md) say a claim that a person completed install, sign-in, the OIDC redirect, passkey enrollment, backup, restore, groups, claims, audit, LDAP, or SCIM on this revision needs a run, and that the page does not supply one. |
| ancestor-local | Ledger `run_head` `d039306`: `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 cargo build --locked --example portal_fixture; CARGO_TARGET_DIR=target ./node_modules/.bin/playwright test multi-authenticator.spec.js --project=chromium --project=firefox --project=webkit --reporter=line; node --check tools/browser/multi-authenticator.spec.js; python3 scripts/check-docs.py; git diff 9065f59..HEAD --check`. Result text: pass; headless two-passkey journey Chromium/Firefox/WebKit 3/3. The Q06 ledger note says the journey uses Playwright WebAuthn shims and a CSS mobile viewport. |
| ancestor-local | Ledger `run_head` `183915d` records a prose command, `U01 isolated two-node PostgreSQL bootstrap and Playwright setup.spec.js across Chromium/Firefox/WebKit`, result text `pass; PG 1, browsers 9`. The same ledger field is a description, not a shell line this slice re-ran. |
| ancestor-local | `/Users/dominik/.local/share/riwork/orchestrators/projects/891e7443-8dac-4c1b-897f-9e53cb59c7ee/planning/evidence/u01/evidence.txt`, dated 2026-09-28, base `roadmap/integration-accepted` at `14de533`. It records a Cua.ai Driver session that reached a backup Touch ID prompt. Backup completion was not claimed. It names screenshots `passkey-prompt.png` and `backup-prompt.png` in that directory. |
| missing measurements | No result at `52df9a3` for 320, 768, and 1440 CSS pixels, 200% text scaling, a screen reader, a physical security key, or an independent new user. `58e5ef3` later records a source-only keyboard and mobile-viewport password invitation on three engines. The Q06 note leaves a physical or synced passkey, a real mobile device, a screen reader, and external mail open. The U10 ledger note, integration commit `5892563`, records a focused 320px empty-workspace sign-in and says physical mobile and a real screen reader remain outside that run. |

Blockers: the guides withhold the human-completion claim; the browser runs are
ancestor-local shims; assistive technology and hardware authenticators have
no result at accepted HEAD.

## Workflows

Target: typed versioned workflows for authentication, enrollment, recovery,
consent, and sensitive actions, with a bounded executor. A02 P01 and gates
G05–G06.

| Class | Record |
| --- | --- |
| source-only | [Workflow model](../workflows.md) at `6d9b72b` describes W01, bounded W02 verifier paths, W03 proof provenance, a fail-closed reviewed pin, W06 authoring, and a held W07 extension contract. Configured consent and eight enrollment shapes are described there. Description is not a run at `52df9a3`. |
| ancestor-local | Ledger `run_head` `6d9b72b`: `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 cargo test --locked --test workflow_configured_consent -- --quiet; python3 scripts/check-docs.py; python3 scripts/check-module-boundaries.py; git diff bc3c324..HEAD --check`. Result text: pass; configured consent suite 8/8. `52df9a3` does not change workflow tests. The suite is repeated at `a3ccbf1`, together with the TOTP consent test, in the snapshot section above. |
| missing measurements | No end-to-end concurrency count on both storage backends for a reviewed pin, and no operator completion of a configured workflow on accepted HEAD. |

Blockers, from ledger `open_checks`: W02 still has unfinished configured
shapes, including invitation and other sensitive actions, browser and
remembered consent, and custom stages. W05's configured-run pin lacks a
multi-party approval record, safe resume of changed content, broad
environment binding, and end-to-end concurrency coverage on both storage
backends. The later W02 note at `a3ccbf1` adds the password-and-TOTP
consent graph and leaves browser, remembered-consent, TOTP-only, and
recovery-code adapters unsupported. The Q06 password invitation and the W07 guest cap are source-only records in the snapshot section. They do not finish browser reauthentication, remembered consent, custom stages other than the held guest, or an independent operator. Workflows stay not passed.

## Administration

Target: compact administration, wizards, one management service, delegated
administration, and exact-content review. A02 E12–E14, P14–P16, and G07.

| Class | Record |
| --- | --- |
| accepted-head local | Ledger `run_head` `52df9a3`: `RIAUTH_PG_TEST_TARGET=reviewed_memberships_postgres CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 scripts/test-postgres.sh; bash -n scripts/test-postgres.sh; python3 scripts/check-docs.py; git diff 6d9b72b..HEAD --check`. Result text: pass; disposable PostgreSQL suite 5/5, TLSv1.3 plus aes256gcm-v1 sealed record and encrypted race/reopen. On that commit the functions are `postgres_unrelated_revision_still_applies_reviewed_membership`, `postgres_affected_membership_user_and_policy_deny_stale_apply`, `postgres_reviewed_membership_reopen_and_receipt_replay`, `postgres_reviewed_membership_http_execute_race_replays_after_reopen`, and `postgres_encrypted_reviewed_membership_replays_across_reopen`. |
| source-only | At `6d9b72b`, [testing.md](../testing.md) says this PostgreSQL target does not open an encrypted connection. At `52df9a3`, `docs/testing.md` and `docs/reviewed-group-memberships.md` say one test restarts the disposable primary, verifies a loopback CA, and opens `aes256gcm-v1` records. Those `52df9a3` sentences are in this worktree. Accepted `ee88af0` adds the group, client, and user display-name plans described in the S04 snapshot. This worktree's product files were not updated to that text. |
| ancestor-local | Ledger `run_head` `4357f6b` records `cargo check; cargo test --test admin_ui` with result text `pass; six admin UI cases`. Later ancestor-local `admin_ui` checks include `91f66d6`, `74ee172`, `567ea3e`, `dbcdce2`, `d790f69`, and `5e47d00`. None of those `run_head` values is `52df9a3`. |
| missing measurements | No count of a person completing Applications, People, Groups, or Security administration on accepted HEAD. |

Blockers: `cf4529e` is the latest accepted redb and PostgreSQL drill,
11/11 and 9/9, including encrypted reopen and fenced promotion. `e3c26eb`
is the earlier 6/6 promotion run. Both are source-only disposable
clusters. Neither is a human administration journey. The O06 diagnostics
are bounded reads and leave dashboards, doctor, and a production backlog
deadline open. Ledger `open_checks` still says advanced
client credential and registration changes and other user, group, agent,
federation, key, session, device, and recovery resources remain outside
exact multi-party approval. The M05 ledger note is about logout endpoint
review. Administration stays not passed.

## Interoperability

Target: EG01. Name the application, directory, IdP, proxy, network, or cloud
tenant and its version, then exercise setup, lifecycle, negative inputs, and
failure handling. Mocks and in-process fixtures do not certify a family.
Q04 is "Test real peers" in the coverage inventory. The ledger now has a
Q04 slice at `fcba8a5`. That slice is local peer evidence, recorded in
the snapshot section above. It does not pass this category.

| Class | Record |
| --- | --- |
| source-only | Recipe pages under `docs/recipes/` describe in-tree or loopback fixtures. [Capability matrix](../capability-matrix.md) states the relying-party client is the in-tree axum fixture, the SAML IdP signer is xmlsec1, LDAP import follows disposable loopback OpenLDAP, upstream OIDC uses an in-process token endpoint, inbound SCIM uses `oneshot`, SAML source uses the in-process `Upstream` helper, and outbound SCIM uses a second loopback riAuth router. |
| ancestor-local | Ledger `run_head` `38f82fe`: `RADCLIENT=/private/tmp/riauth-fr-prefix/bin/radclient CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 scripts/test-radius.sh; bash -n scripts/test-radius.sh; python3 scripts/check-docs.py; git diff d039306..HEAD --check`. Result text: pass; FreeRADIUS radclient 3.2.10 real loopback PAP accept/reject/replay 1/1. |
| source-only | [Q03 pilot preflight](q03-conformance-pilot.md) says that on 2026-09-29, at commit `19a69c66b473480c8b570498231fad4d4edb7a31`, the pinned suite checkout, private configuration, `CONFORMANCE_SERVER`, and `CONFORMANCE_TOKEN` were absent. No independent OIDF plan was run. That named commit is not an ancestor of `52df9a3`. The Q03 ledger note says the same inputs are still required. |
| actual peer/tenant | No record in the docs read here, and no ledger check at `52df9a3`, names a completed Okta, Entra, Google, Active Directory, Workspace tenant, Vault, named relying party, named service provider, or named SCIM client run. |
| local peer | OpenLDAP `ldapsearch` 2.7.1 at `fcba8a5` and `1caa093`, and FreeRADIUS `radclient` 3.2.10 at `38f82fe`. Both are loopback. Details and commands are in the snapshot section. |
| release/deployment | No named tenant, production directory, hardware NAS, or relying-party deployment is recorded at `0312427`. |

Blockers: the local peer runs do not name Active Directory, a production
directory application, a SAML service provider, or a hardware NAS. D03's
ledger note says named deployment peers remain open. I10's open check
still asks for a real Workspace or Entra tenant lifecycle. Q03 still has
no independent plan. Interoperability stays not passed.

## Footprint

Target: exact official artifacts, missing-module rejection, and no weakening
of credential protection to reduce footprint (G04, G09). EG02 asks for native
Linux x86-64 and ARM64 packages. Byte size and memory are measurements.

| Class | Record |
| --- | --- |
| ancestor-local | [Q08 edition matrix](q08-exact-edition-bundles.md) records local macOS ARM64 builds at `4ca7558ee8563a0c5eedc1f13d4a9a6faa6a8d4a`, which is an ancestor of `52df9a3`. Essentials compiled 60 capabilities and excluded 26. Platform compiled 86 and excluded none. Direct normal dependencies were 38 and 52. Fresh redb and PostgreSQL readiness and admin login returned 200 on both editions. Native binary SHA-256 values are in that page. `target/dist` had none of the nine expected files per Linux architecture. |
| source-only | The Q08 reproduce command is `python3 scripts/check-exact-edition-matrix.py`. This slice did not run it. |
| release artifacts | No Linux x86-64 or ARM64 archive, container, checksum, or provenance file for `52df9a3` was in the tree or named by a ledger check at that head. The Q08 ledger note says shipped bundles remain unavailable. |
| missing measurements | No byte size or peak RSS for a binary built from `52df9a3`. Debug byte sizes in the Q09 page (Essentials `202086936`, Platform `285351464`) belong to commit `47aa248ce1c68284773746bbb5fd59c6098afdea`. That commit object exists in this repository and `git merge-base --is-ancestor` reports it is not an ancestor of `52df9a3`. |

Blockers: local macOS dev builds at an ancestor are not the release
footprint. Linux artifact size and an accepted-head memory bound are absent.

## Performance

Target: EG04. Record workload, dataset, hardware, security settings, peak
RSS, p50/p95/p99, successful throughput, errors, and background interference.
Product load, RPO, and RTO require a named operator workload. There is no
promised numeric speed in the product contract.

| Class | Record |
| --- | --- |
| source-only | [Q09 benchmark slice](q09-benchmark-slice.md) and `scripts/q09_benchmark_slice.py` define one `GET /api/me` measurement. Reports set `observations_only` and `performance_claim: false`. `python3 scripts/q09_benchmark_slice.py --self-check` measures a fixture server, not riAuth. |
| ancestor-local | Ledger Q09 integration commits include `7aea083b3894cc964e75786200c9e7d952e8ab01`, an ancestor of `52df9a3`. Four orchestration files, `q09-8u-8g-*-accepted-7aea083.json` under the planning directory, have `source.commit` `7aea083b3894`, `product_run: true`, `observations_only: true`, `performance_claim: false`, and dataset `q09-8u-8g`. This slice read those fields and did not adopt the latency numbers as a target. The Q09 ledger note says those four macOS dev-profile reports were checked and that sustained background cadence, named thresholds, secure deployment variants, and Linux ARM64 release artifacts remain outstanding. |
| source-only | The numeric table in the Q09 page is commit `47aa248`, which is not an ancestor of accepted HEAD. Those figures stay observations of that other commit. Cache paths named on that page were not re-hashed here. |
| missing measurements | No session-read report whose `source.commit` is `52df9a3`. No TLS, database-encryption, or external-signing variant at accepted HEAD. No run long enough to overlap the 60-second maintenance cadence. No named operator RPO or RTO. |

Blockers: `performance_claim` is false on the reports that were inspected.
The S02 ledger note says the held v9 Group chunk mirror stays unaccepted
because 4 MiB lookups regressed and strict memory and performance gates are
unmet. [Testing](../testing.md) describes `scripts/characterize-contention.sh`
and `scripts/measure-backup-memory.sh` as local observation tools. This slice
found no accepted-head output file for either script.

## Availability

Target: probes, a single redb owner, PostgreSQL multi-node behavior with
fencing, and a measured outage result. The contract does not treat PostgreSQL
as database high availability by itself.

| Class | Record |
| --- | --- |
| source-only | [Availability](../availability.md) says `/readyz` returns 503 while storage is unavailable and `/livez` stays independent of storage. It says the local harness enables synchronous replication and checks primary crash, fencing, standby promotion, reconnection, and surviving session and refresh tokens. The same page says the fixture does not set an RTO or RPO for a deployment. |
| ancestor-local | Ledger `run_head` `f8c3602`: `CARGO_BUILD_JOBS=2 bash scripts/test-postgres.sh`. Result text: pass; fenced primary/standby failover. |
| source-only | `e3c26eb` promotes a disposable standby for reviewed membership. `cf4529e` repeats promotion inside the 9/9 PostgreSQL suite and still records no outage duration or RTO. `7e07e748` refuses an issuer or active-capability mismatch. `75def67` later records node-security redb 4/4 and disposable PostgreSQL 3/3 for token lifetimes and password history. `acf1d7c` records a two-worker logout lease 1/1 and a full operations suite of 11/15. The deployment, rollback, and real multi-node gates in those notes stay open. |
| release/deployment | No deployment RTO, RPO, or failed-request count is recorded at `0312427`. |

Blockers: the promotion and the node-security refusal are disposable
source-only runs. [Node security](o03-node-security.md) leaves shared-job
leases, peer health, and native TLS open. Enterprise features still need
their own multi-node acceptance. Availability stays not passed.

## Recovery

Target: G12 and EG03 restored-state invalidation, plus a rehearsed restore
with the deployment's keys and peers. [Testing](../testing.md) asks for
measured recovery time and data loss before an application moves.

| Class | Record |
| --- | --- |
| ancestor-local | [R05 local drill](recovery-drill-r05.md) cites [r05-local-2026-09-29.json](evidence/r05-local-2026-09-29.json) and [r05-postgres-local-2026-09-29.json](evidence/r05-postgres-local-2026-09-29.json). Both have `result` `passed` and 16 checks. Schemas are `riauth.recovery-drill/v1` and `riauth.postgres-recovery-drill/v1`. Binary SHA-256 values are `c125134b04154c39d58f953342cc20c2b50b512ebfc58e6fa6cf240a84aa2400` and `8e26d768ce419a6b31fc22bd517aaf7b07c359a74fc5801990b1b33271333ffa`. Neither file has a source commit field. Scopes are disposable localhost redb and a disposable loopback PostgreSQL cluster. |
| source-only | Reproduce commands in the R05 page are `python3 scripts/recovery-drill.py` and `python3 scripts/recovery-drill-postgres.py` against a binary built in a private Cargo target. This slice did not run them. |
| ancestor-local | The R05 ledger note says an accepted-tree PostgreSQL drill was rerun 16/16 and that deployment PITR and failover remain open. The JSON `external_gates` arrays name lost backup or database key escrow, missing configured secret files, Vault Transit and other external services, PostgreSQL PITR and multi-node failover, and a real OIDC or SAML relying party. The PostgreSQL file also names a TLS PostgreSQL connection as an external gate. |
| release artifacts | [Q10 installed release gate](q10-installed-release-gate.md) records this command at accepted head `679f2927885d9dc4dcc1c881fd972bcade70e07a`, an ancestor of `52df9a3`: `python3 scripts/check-installed-release-gate.py target/dist 679f2927885d9dc4dcc1c881fd972bcade70e07a Rhein-Industries/riAuth 1 1`. The page says `target/dist` did not exist and the command exited 1. The Q10 ledger note says missing local `target/dist` fails closed and actual Linux x86-64 and ARM64 runner execution remains outstanding. |
| missing measurements | No recovery time or data-loss figure for `52df9a3`, and no restore of a release artifact. |

Blockers: the 16-check drills are disposable observations whose source commit
is not in the JSON. Their own gate lists are still open. [Operational
recovery](../operational-recovery.md) says that page does not finish D04.

## Migration

Target: preflight, identity continuity, rehearsed cutover, and rollback.
G01–G03. A route in an inventory is remediation, not conversion.

| Class | Record |
| --- | --- |
| source-only | [Migration](../migration.md) documents Authentik bundle conversion and `riauth migration-preflight --file <input>`. For any system other than the Authentik bundle, `ready_for_plan` stays false and no manifest is produced. Classifications are not evidence that an application or factor works after cutover. |
| source-only | The G01 ledger row lists source commits `6dccbdf` and `97f27ec`. Neither is an ancestor of `52df9a3`. Its integration commits `c12bffe` and `248207d` are ancestors. The row's remaining text says real source-export peer migration and identity continuity are separate G02/G03. This slice did not re-run a migration at those integration commits. |
| source-only | The G02 ledger note says reimport tests passed and that a real Authentik export and application cutover remain open. The G04 note says re-enrollment copy was reviewed and that no real Authentik export was run. Those notes are ledger text. This slice did not re-run them at `52df9a3`. |
| source-only | G05 integration commit `cf827a9` runs `local_reference_rp_cutover_rehearsal` against a disposable redb fixture and an in-process reference relying party. The ledger result is focused 1/1 with eight asserted case IDs. The page keeps G05 in progress. |
| release/deployment | No cutover duration, export byte count, live relying-party subject, production route change, or Authentik rollback sign-in is recorded at `cf827a9`. |

Blockers: source preflight does not move a directory. The G05 run does not
supply the exact target: a real application's successful and denied access,
claims, MFA, refresh, logout, recovery, and rollback. Migration stays not
passed.

## Security

Target: Q01's 35 invariants as contracts, EG03 runtime demonstration on
installed artifacts, and release evidence that includes signatures and a
review record. A contract paragraph is not a passing test.

| Class | Record |
| --- | --- |
| source-only | [Invariants](../security/invariants.md) says existing tests were read, not run, and that Q02/Q05 recipes in that catalog's header are not claims of passing coverage. The current evidence-boundary table still marks several domains as partial or intended policy. [Threat model](../security/threat-model.md) maps the same boundaries. |
| ancestor-local | Ledger `run_head` `7edfc42`: `cargo test --locked --features test-support,fuzzing --test contracts --test identity_boundary --test bootstrap --test q05_replay_concurrency --test scim_oauth --test removal_safeguards --test workflow_model -- --quiet` (result text: pass; 54 redb shared contracts) and `CARGO_BUILD_JOBS=2 bash scripts/test-contracts-postgres.sh` (result text: pass; 54 PostgreSQL shared contracts). |
| ancestor-local | Ledger `run_head` `f8c3602`: `RIAUTH_PG_TEST_TARGET=q05_replay_concurrency CARGO_BUILD_JOBS=2 bash scripts/test-postgres.sh`. Result text: pass; Q05 PostgreSQL replay binding. |
| source-only | [Q11 release evidence](q11-release-evidence.md) says the release workflow, packager, and bundle checker do not sign artifacts. The Q11 ledger note records 26/26 producer tests, 13/13 release-evidence tests, and a 355-package source SPDX document at integration commit `281493d`. That note calls the document a source document. The Q11 page says a source SPDX file is not a release SBOM. This slice did not open the orchestration SPDX file and did not treat it as release evidence. |
| source-only | A03 checks through `ee88af0` move cloud reads, the credential write, the source-configuration mutation, and the browser source-finish transaction into assembly. Wider module-boundary work remains. `b759216` binds the source SPDX producer to the Linux packager's exact archive and image names. The accepted audit keeps `release_executed` and `release_sbom_produced` false. |
| release/deployment | No signature, published GitHub release, packaged release SBOM, or installed-release success on shipped Linux archives is recorded for `0312427`. Ledger `open_checks` says Q11 still requires signing, an SBOM, an independent release review, publication, a checked asset set, and Linux ARM64 execution. |

Blockers: source-only contract tests, A03 assembly checks, and a source
SPDX file leave the security category open. The product contract says
missing tests do not deliver the defaults. There is no certification
result and no EG03 run of installed artifacts at this head. Ledger
`independent_review` notes are slice comments, not a D05 category result.

## Ledger open checks that still block D05

The ledger `open_checks` array has 17 entries at this read. The ones that
block a category above are included there. The rest also stay open and are
not waived by this matrix:

- I08 Windows signing and LogonUI need a real Windows VM.
- Linux ARM64 shipped-bundle validation is outstanding.
- I10 verify-controller readiness is point-in-time; direct browser route and
  CSRF, revoked-agent and file-failure cases, and a real Workspace or Entra
  tenant remain.
- Essentials `cargo test --test identity` is not feature-gated and cannot
  compile Platform-only SAML, RADIUS, and SCIM tests.
- A03 protocol files still refer to Core and to storage.
- M03 still has direct mutation surfaces outside the one management service.
- S02 ordinary LDAP search no longer loads full Group values; Group writes
  and offline index rebuild still do, and whole-operation memory bounds
  remain open.
- The S04 open-check sentence still says PostgreSQL 3/3 and that standby
  promotion and encrypted PostgreSQL remain open. `cf4529e` records redb
  11/11 and PostgreSQL 9/9, including encrypted reopen and fenced
  promotion. Those drills are source-only. S04, administration, and
  availability stay not passed.
- `cargo fmt --all -- --check` fails on the accepted tree. The ledger
  names a 19,306-line diff and a format-only sweep. That check does not
  pass a category.

## What would be required before a category could pass

This slice does not perform these steps. They are the blockers, written as
the missing evidence:

1. Record the category's exact target at the commit being accepted.
   A source-only or local-peer pass stays in that kind until the target
   itself is the thing that was run.
2. For interoperability, name the external product and version and keep the
   log. A loopback OpenLDAP or FreeRADIUS run stays local peer.
3. For footprint, performance, availability, and recovery, attach the
   release-artifact hashes when the claim is about a release, and record
   RSS, latency, error counts, outage time, and data loss on that artifact.
4. For usability, workflows, administration, and migration, record an
   independent person completing the documented path, including the failure
   stops the runbooks name.
5. Leave the category `not_passed` until that record exists. Do not treat
   this file, the coverage inventory, or a ledger review note as that record.
