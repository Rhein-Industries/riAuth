# O03 original-acceptance disposition

**Recommendation: DONE for the original O03 row at published main
`2d71dc61f9f6b3e564372301bbecd1e71db43e33`.** The published implementation,
operator contract, historical focused checks and verified predecessor integration
evidence cover that row. No concrete missing local behavior blocking its
original scope was identified, so no implementation seam or new campaign is
proposed. Root reviews and decides closure; this report changes no task status.

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; existing task
`85240c6b-8c87-4a62-a07e-68c7ed1a5d5a`, worktree
`ed9ac424-59f4-4520-905b-919aea3521eb`, branch
`roadmap/local-revisions-coordination-wave27`. The branch starts this reassessment
at `fee96c332f30f2e401a30b0b59afca2b9f5e2d68`. Only this report is added;
all assessed source and prior reports were read from immutable Git objects.

## Original outcome and closure mapping

The original task row was reread through RiWork:

> Detect mismatched capabilities and security settings; define shared jobs,
> rate limits, and cache freshness.

Its workstream completion gate is: an operator can identify which component is
failing, what remains safe, and what corrective action is required. The row
remains `in_progress` until root acts. The recommendation below assesses that
outcome using the actual published boundaries.

Every source path in this report refers to the pinned published commit above.

| Original requirement | Published definition and safety | Evidence and disposition |
| --- | --- | --- |
| Mismatched issuer, capabilities and authentication settings | `src/node_security.rs` records issuer, compiled active capability names, access/refresh/session lifetimes and password history. Initialization stamps it atomically; `src/core.rs:224-239` enforces it before startup migration, lineage reconciliation, index backfill or edition stamping. Invalid, missing, old or incompatible agreements refuse without selecting a new policy. | Historical process-level PostgreSQL issuer/capability/authentication refusals plus the historical current-format local unit checks below. DONE for the defined agreement. |
| Rate agreement and explicit upgrade/adoption | Format 3 compares every effective threshold from the single 16-category resolver in `src/config.rs`; omission equals explicit default. A differing category names configured/recorded values and its alignment/restart remedy. Formats 1/2 require explicit offline recording; a missing row also requires explicit adoption. A present incompatible format-3 policy cannot be overwritten by that command. | Historical all-16 unit/middleware and real maintenance-command checks; reviewed source still contains those protections. DONE for the startup/offline contract. |
| Shared jobs and target admission | Durable logout/mail/SSF leases pin dispatch and reject stale completion. Offboarding, provisioning, deactivation and reconciliation retain their authority/lease/receipt checks. `src/background/targets.rs` adds bounded shared logical-target admission in the durable claim transaction, or a short manual writer; release compares owner and generation. A full 128-row ledger refuses instead of stealing a live admission. | Historical local admission/rollback/capacity checks and historical real PostgreSQL logout/mail/SSF worker checks; verified current predecessor Q05 schedules additionally exercise offboarding owner reclaim/stale refusal. DONE for the specified coordination semantics. |
| Shared rates including forward auth | `src/api.rs:1103-1119` uses the canonical threshold and PostgreSQL shared counter, through reserved forward-auth permits for that category. redb counters remain process-local. Address grouping, fixed 60-second windows and bounded counter eviction are defined. | Verified predecessor PostgreSQL shared-counter primitive; historical all-16 middleware and router-replacement checks; real proxy integrations below support the surrounding authorization/revocation path. DONE for the defined backend boundary, without claiming a current multi-process all-16 HTTP test. |
| Cache freshness | Outbound SCIM caches compare target configuration, secret fingerprint, expiry and shared per-target owner/generation stamps. Publication compares the observed stamp before committing its successor; stale acquisition cannot publish. A 401 invalidates only the matching local generation and shared stamp, including local counter reset. Tokens remain in process memory; static credentials are reread. Recovery clears stamps/admissions. | Historical focused SCIM cache, stale-publication/invalidation and recovery checks; current pinned implementation inspected. DONE for this cache contract. Verified Access remains explicitly process-local. |
| Operator identifies component, safety and remedy | Startup diagnostics name issuer/capability/authentication/rate agreement failure. Probes report this process's role/duties; background metrics identify job/lane, occupied capacity, deferral and timeout. Authorized delivery/operations reads distinguish durable pending/failed/ambiguous outcomes from runtime completion. `docs/operations.md` describes configuration alignment, offline upgrade, capacity/dependency investigation and safe retry/reconciliation. | Exact operator mapping below is present in published source/docs. DONE for the original operator gate; readiness is local and does not promise a healthy peer fleet or remote completion. |

## Published operator decisions

- Issuer/capability/authentication/rate mismatch: the attempted process does not
  reach normal startup writers/listeners. Preserve the recorded policy and align
  its config/binary; a rate message gives the exact `rate_limits.<category>`
  value. Authentication agreement covers the four recorded integers. Use a
  reviewed edition handoff when changing editions rather than bypassing a gate.
- Old/missing agreement: stop every server/gateway/worker/listener/admin process,
  verify a compatible backup with its keys, align participating configurations,
  then run the source-edition maintenance binary with
  `security-agreement-record --confirm-authentication-policy --confirm-rate-limits`.
  Missing-row adoption additionally needs `--adopt-missing-agreement`. redb
  requires exclusive ownership; the PostgreSQL named-session check supplements
  the operator's stop-all duty and cannot prove a stopped fleet. Matching retry
  returns `recorded: false`; present format-3 disagreement stays unchanged.
- Rate or local capacity pressure: 429 `rate_limited` identifies the address
  bucket; `riauth_rate_limited_total` counts it. `connector_overloaded` /
  `background_overloaded` mean no pass was started. Job/lane metrics and
  `retry_after_ms` direct investigation of connector timeouts, dependencies and
  storage contention. Existing foreground revocation and durable enqueue stay
  separate from downstream admission.
- Running/overdue work: `background_timeout` does not cancel the operation or
  release occupied slots. Manual `connector_operation_pending` may still commit;
  inspect durable state before retry. `target_deferred` combines local busy,
  shared admission and full-ledger causes; it identifies the job/lane, not which
  of those causes occurred. Inspect the relevant target/job and dependency rather
  than force a second claim. Delivery errors and ambiguous writes use their
  existing queue diagnostics and reconciliation procedures; a completed pass
  is not proof of successful remote delivery.
- Configuration and cache boundaries: compiled/enabled capability agreement
  is shared; `configured`, `runtime_ready`, `usable`, process role/listen/browser
  choices and readiness remain local. File-backed trust, keys and credentials,
  trusted-proxy identity policy and participating releases require operator
  coordination. Per-client policy stays on shared client records. Maintenance
  and alerts run on each eligible worker; there is no universal singleton/leader
  promise. SCIM expiry/config/secret/stamp changes require fresh acquisition;
  a second 401 stops the bounded retry. Verified Access tokens remain local.

These instructions are in `docs/operations.md:137-188,210-441`,
`docs/api.md`, `src/api/probes.rs`, `src/api/server.rs` and the named
node-security/background/provisioning paths. They define what remains safe and
what an operator should do without equating readiness with downstream success.

## Verified predecessor integration, exact scope

Root supplied successful integration job `110661000640` of run `36950097067`.
The full downloaded log is `/tmp/riauth-wave29-integration-110661000640.log`
(3005 lines), SHA-256
`458a30895b5987c0bebe2fb7368a1a32969fa0d41ce90acbfa56f43299ccd186`.
Its checkout line 122 records
`2d05c00deed3a6dafcd458a5f1a1a9beb17d0dd8`. Its result lines and final cleanup
were inspected; the file has no failure marker. Job SUCCESS is root's verified
status, not a live GitHub query made during this reassessment.

The scripts' commands were read at the pin rather than inferred from step names.
The default Cargo feature is Platform. The two PostgreSQL script selections
were `postgres` and `q05_replay_concurrency`; no dedicated O03 peer target was
selected. The shared-contract script's default name filter was `postgres`.

| Observed command / expansion | Exact relevant result | Credit boundary |
| --- | --- | --- |
| `PG_BIN="$(pg_config --bindir)" scripts/test-postgres.sh`; expands to `cargo test --locked --test postgres -- --ignored --nocapture --test-threads=1` | `postgres_atomicity_shared_sessions_replay_limits_migration_and_fenced_failover` and `postgres_single_connection_streams_and_audits_a_backup`; 2 passed, 17.51s, log 1370-1394. | The first alternates two Cores through five allowed shared-counter hits and a refused sixth (`tests/postgres.rs:906-919`). It stops the primary before standby promotion; the logged recovery is 1113 ms. Also covers shared identities/replay and service reconnect. This is a disposable PostgreSQL drill and a counter primitive, not all-16 HTTP/forward-auth threshold or deployed HA proof. |
| `PG_BIN="$(pg_config --bindir)" bash scripts/test-contracts-postgres.sh`; expands to `cargo test --locked --features test-support --test contracts -- --ignored postgres --test-threads=2 --nocapture` | 91 passed, 90 filtered, 209.68s, log 1396-1508. | The named subset below supports identity/transaction/authority invariants. The whole 91 is not credited to O03 and does not select dedicated node-security, admission or cache peer tests. |
| `PG_BIN="$(pg_config --bindir)" RIAUTH_PG_TEST_TARGET=q05_replay_concurrency scripts/test-postgres.sh`; adds `--features test-support --test q05_replay_concurrency -- --ignored --nocapture --test-threads=1` | `q05_postgres_replay_binding_and_interrupted_management`; 1 passed, 1 filtered, 13.98s, log 2864-2882. | `run_suite` covers recovery login/proof/management/refresh/factor races, interrupted offboarding authority and offboarding lease reclaim after reopen, including stale-owner refusal. It does not execute the dedicated mail/logout/SSF lease targets. |

The shared-contract supporting subset is exactly these seven families, each
with both `::postgres` and `::postgres_encrypted` successful log entries:
`backend_parity::concurrent_claim_is_single_use`,
`backend_parity::fault_rollback_and_commit_visibility`,
`disable_reenable_revokes_dependents`, `identity_and_issuer_continuity`,
`offboard_intent_durable_cancel`, `offboard_retry_rechecks_authority` and
`password_attempts_and_change` (14 named passes, not 91 O03-specific passes).

The same completed job also passed these surrounding integrations:

- Three separate `cargo test --test identity --locked <filter> -- --ignored`
  invocations with `RIAUTH_TEST_XMLSEC`: filters `saml_independent_xmlsec`,
  `saml_source_independent_xmlsec`, `saml_logout_independent_xmlsec`; exact
  tests `saml_tests::saml_independent_xmlsec_sign_verify_and_decrypt`,
  `saml_source_tests::saml_source_independent_xmlsec_responses`,
  `saml_logout_tests::saml_logout_independent_xmlsec` (one pass each).
- `scripts/test-ldap.sh`: `openldap_plans_stable_ids_tls_login_mfa_and_fail_closed_sync`
  passed. Real-browser Rust commands select `--test browser` and
  `--test portal_browser`, `--locked -- --ignored`, with `RIAUTH_TEST_BROWSER`:
  `browser_terminal_login_callback_and_signed_backchannel_logout` and
  `portal_terminal_sign_in_access_changes_and_responsive_interactions` passed.
- `cargo test --test outpost --locked -- --ignored` with nginx/browser env:
  `nginx_forward_auth_terminal_sso_headers_and_revocation` passed (1 filtered).
  `cargo test --test outpost_traefik --locked -- --ignored` with Traefik env:
  `traefik_forward_auth_real` passed (13 filtered). These exercise proxy flows,
  not multi-node forward-auth numeric threshold disagreement.
- Playwright setup: 9 passed. The authenticator journey step reported
  **22 passed, 2 skipped**, not 24 passes; the skipped invitation-passkey cases
  were Firefox/WebKit. These are supporting protocol/user-flow evidence and
  supply no new O03 peer-version, cache or admission proof.

## Historical focused evidence, credited as historical

The following were reread from published reports, not rerun in this reassessment.

| Published report and recorded commands/tests | Recorded result and qualification |
| --- | --- |
| `docs/roadmap/o03-node-security.md`, historical `RIAUTH_PG_TEST_TARGET=node_security_postgres ./scripts/test-postgres.sh` (`--locked --features test-support`, ignored target) | Three process tests passed in 12.11s: `authentication_policy_mismatch_binds_nothing_and_preserves_postgres`, `capability_or_issuer_mismatch_binds_nothing_and_preserves_postgres`, `format1_record_is_one_row_and_a_different_policy_binds_nothing`. These were format-1/2-era runs, with no HTTP bind and unchanged stored records on refusal. They are not current-format-3 peer executions. |
| Same report, `RIAUTH_PG_TEST_TARGET=job_lease_postgres`, `ssf_lease_postgres`, `mail_lease_postgres` script selections, all adding `--locked --features test-support -- --ignored --nocapture --test-threads=1` | `two_workers_pin_one_logout_delivery`: 1 pass, 12.15s. SSF: `two_workers_pin_one_ssf_delivery` and `two_workers_pin_one_ssf_delivery_with_database_key`, 2 passes, 18.90s. Mail: corresponding `two_workers_pin_one_mail_delivery` / `_with_database_key`, 2 passes, 45.89s. Historical real worker processes on disposable PostgreSQL; stale finishes preserve successors. SSF/mail included record encryption, loopback transport remained unencrypted. |
| `docs/roadmap/local-wave27-revisions-coordination-report.md`: `cargo test --locked --offline --features test-support --lib background::targets::tests -- --test-threads=1` and exact `background::tests::busy_target_cannot_starve_unrelated_manual_or_due_work` / `manual_connector_overload_preserves_foreground_and_durable_work` | Admission module 2 passes; each background fixture 1 pass. Deferral, expiry, owner/generation stale release, claim rollback, live-ledger capacity and foreground isolation. Separate executors on one redb store; no live PostgreSQL peer or HA evidence. |
| Same report: exact `--test rate_limits shared_forward_auth_counter_survives_router_replacement`; exact `--test scim_oauth shared_freshness_invalidates_local_cache_and_fences_late_publication`; exact library `provisioning::token_freshness::tests::late_invalidation_does_not_clear_a_newer_issuance`; exact `--test recovery restore_invalidates_connector_admission_and_oauth_cache_stamps`, with `--locked --offline --features test-support` | One pass each. Counter middleware uses a redb transaction plus an unused PostgreSQL config marker; no connection was opened. SCIM checks establish the local/shared-stamp stale publication and invalidation contract, including local generation reset. Existing OAuth cached/expiry and bounded-401 tests also passed on the recorded built binary. |
| `docs/roadmap/local-wave29-o03-rate-agreement-report.md`: `cargo test --features test-support --lib node_security:: -- --test-threads=1` | Seven local unit checks passed after the recorded fixture correction. Covers issuer/capabilities/authentication, all-16 semantic thresholds and unchanged refusal snapshots, strict format-1/2 upgrade/missing adoption and edition rate preservation. This is the current-format local evidence, not a PostgreSQL peer run. |
| Same report, recorded built binaries: exact `every_http_category_uses_the_recorded_effective_threshold` and `rate_agreement_upgrade_and_missing_adoption_require_explicit_flags` | One pass each: all 16 recorded limits agree with caller behavior on memory and local transactional paths; real maintenance child commands require flags/adoption consent and preserve present agreement policy. `tests/node_security_postgres.rs` only compiled with `--no-run` in this slice. |
| Published lockout and edition fixture reports | Exact CLI lockout test passed (27.10s). Exact Platform provenance refusal/unchanged snapshot/original-config reopen passed (1.16s), exact Essentials refusal counterpart passed (1.37s). Matrix SAML default-30 adaptation had Python syntax/AST comparison only; the full matrix was not executed. |

The seven current-format local unit names are
`agreement_tracks_issuer_and_active_capabilities_only`,
`open_refuses_a_different_active_set_without_rewriting_the_store`,
`open_refuses_a_different_authentication_policy_without_rewriting_the_store`,
`record_upgrades_a_format1_agreement_without_a_partial_rewrite`,
`effective_rates_match_all_categories_and_reject_before_startup_writes`,
`format2_upgrade_preserves_recorded_policy_and_missing_adoption_is_explicit`,
`edition_handoff_preserves_rates_and_refuses_a_different_threshold`.

The older node-security page explicitly labels its pre-format-3 narrative
historical. Its former automatic missing-row adoption and node-local-only rate
threshold statements are superseded by format 3 and `docs/operations.md`; they
are not current definitions. Earlier reports' open recommendations preceded
this original-scope reconciliation and the newly verified integration evidence.

## Transfer to the pin, limits and final recommendation

An immutable blob comparison verified 21 relevant source/test/CI paths equal
between the successful predecessor and the published pin: node security/config/
Core, background/admission, provisioning/cache/deactivation/reconciliation/
recovery, logout/SSF/mail/offboarding, PostgreSQL/Q05/shared-contract sources,
CI and its two PostgreSQL scripts. The six-file delta adds the source-TOTP
adapter and its docs/test. `src/api.rs` also adds `/source-totp` to the existing
configured-source `source_start` category predicate; its rate resolver/counter
body is unchanged. No new route execution result is inferred from predecessor
CI. Root's unsupported future-format-4 fixture correction remains published.

The integration log does **not** run `node_security_postgres`,
`job_lease_postgres`, `mail_lease_postgres` or `ssf_lease_postgres`. Their
current-format-3 versions remain unexecuted by that job. No process-level
format-3 threshold disagreement, old-binary execution, remote cache peer or
deployed-fleet result is invented. Older-binary refusal is a strict-schema/source
and local parser contract; rollback still needs a compatible pre-upgrade backup.

Shared connector admission is **60 seconds, non-renewed**. It is not
full-lifetime cross-process I/O exclusion. Local permits stay held until actual
finish, but a paused admitted process can resume external I/O after another
process reclaims admission. Logout/mail/SSF dispatch pins have the corresponding
paused-before-send limitation; stale completion fences remain separate. SCIM
stamps do not atomically fence cache reads against later external I/O. The
bounded counter ledger can evict old windows under saturation. File-based trust
and participating release coordination remain operator duties. These published
limits are part of the defined behavior and do not establish deployed HA.

Root reports that this run's all-target check failed the unrelated browser-TOTP
fixture assigned to W02, and latest run `36951643190` was still in progress at
handoff. This assessment claims a successful named predecessor integration job,
not an overall green run or a finished latest run. Generic release gates and
other owners' regressions remain their own review/publication decisions.

**Recommend root close the original O03 row as DONE after reviewing this
mapping.** Keeping it in progress solely for universal HA, equality of every
local setting, or a generic release hold would add a completion condition absent
from that row. Its implementation and bounded evidence meet the requested
coordination/operator outcome; the limits above remain explicit release and
deployment qualifications. No smallest missing local seam is proposed because
none was found blocking that outcome.

S04 remains separate: its measured performance improvement under equivalent
security settings and concurrency gate are not discharged by this O03 report.
No S04 product work, measurement or broad benchmark was performed or proposed.

## Checks performed in this reassessment

Only read-only RiWork task retrieval/reporting, immutable Git object/path/blob
comparisons, exact log filtering/checksum and documentation inspection were used
before writing this report. Pinned-path/name checks, `python3 scripts/check-docs.py`
and staged whitespace validation passed. Only this new document is committed.
No implementation/build/test, main merge/reset/source edit, status/board change,
new task/worktree/worker/RiWork shell, push, service, desktop or cloud operation
was performed. Root receives the exact report commit for review and closure.
