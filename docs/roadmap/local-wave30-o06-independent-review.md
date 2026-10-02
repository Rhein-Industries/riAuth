# O06 independent original-acceptance review

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, O06 task
`9899f8e6-05ff-4e11-b9a0-b9ca184221a6`. Fixed published main is
`2dea9f5df63583caee5cfa99794a5be2c796b9e4`. This report is written from worktree
`9c54c023-a95f-48e9-a908-6f97a5677c98`. The O06 implementation lane is the
Sol worktree `f2e8500e`.

The review is read-only and was led by Claude Opus 5.5. Three read-only Sonnet
reviewers each mapped two facets; all three reported `claude-sonnet-5-5`.

## Method and evidence limits

- **Evidence base:** Git objects at the fixed main only. The lead used
  `git show` and `git grep`. Two reviewers also exported that commit with
  `git archive` into the session scratchpad to get line numbers.
- **What was not done:** no build, test, service, merge or product edit, no
  desktop work, and nothing from the working tree was used as evidence.
- **Verification:** the lead re-checked every pin under "Blocking findings"
  in the source. Other items carry the reviewer's pin and are marked
  "reviewer pin".
- **Runtime claims:** behavior is read from code, unless a named test
  exercises it. Nothing here comes from CI browser timing.
- **Not reopened:**
  - the accepted reviewed client-creation receipt, route-specific header and
    PAM fallback contracts;
  - the O03 enforcement decisions on what nodes compare and refuse;
  - the M07 and W02 decisions;
  - the existing redaction rulings.

## Original row

- Requested outcome: "Expose failed jobs, connector lag, node mismatch,
  storage pressure, key problems, and incomplete offboarding."
- Workstream gate: "An operator can identify which component is failing,
  what remains safe, and what corrective action is required."
- Completion evidence: review implementation, tests, documentation and
  released artifacts as applicable.
- Both editions keep the same semantics for shared capabilities.

The gate does not ask for every signal to be a Prometheus alert, for live
firing proof, or for an HA topology. Those are listed separately as
deployed or release evidence.

## Verdict

**O06 is not done.** Each of the six facets has a real operator surface. But
three facets have a local diagnostic that misleads the operator:

- **Incomplete offboarding:** completed jobs are reported as incomplete.
- **Connector lag:** a controller that never ran is reported as all clear.
- **Failed jobs:** the alert cannot be cleared by the documented fix.

In three more places the gate clause cannot be met from what the server
shows:

- **Key problems:** a signer failure cannot be attributed.
- **Node mismatch:** a mismatch after start produces a 503 with no cause.
- **Corrective action:** several action tokens have no documented remedy.

Storage pressure has measurements but no pressure verdict, and no remedy.
Every item below can be fixed locally.

None of the remaining external evidence would close these items. That
evidence is deployed alerting, live connector peers, multi-node PostgreSQL
and a release.

## Blocking findings (lead-verified)

Ranked by gate impact. Each item lists the smallest seam and a prospective
file claim for the Sol lead.

### 1. Failed-job signals cannot be cleared by the documented remedy

Facets: failed jobs and incomplete offboarding. Platform only for offboarding
jobs; the deactivation part covers both editions.

**What happens:**

- **Failed offboarding jobs are permanent.**
  - A failed offboarding job can never leave `failed`:
    - `offboard_cancel` refuses `Done | Failed` (`src/offboarding.rs:993-995`);
    - `offboard_reschedule` accepts only `Scheduled` (`src/offboarding.rs:966`);
    - nothing deletes `offboard_jobs` rows (no delete in `src/offboarding.rs`).
  - The gauge counts it as failed for good (`src/store/maintenance.rs:41-55`).
  - `RiAuthFailedDeliveries` (`riauth_queue_failed > 0`,
    `deploy/riauth-alerts.yml:62-68`) therefore fires indefinitely after one
    failure, whatever the operator does.
- **Resolved deactivations keep the alert firing.**
  - The `provisioning_deactivations` gauge counts `failed` or `stale` status
    and ignores any operator resolution (`src/store/maintenance.rs:56-69`).
  - So after the documented `attest_remote_state` remedy (`riauth provision
    resolve-deactivation`), the aggregate reports `resolved` while the
    gauge and alert keep reporting a failure. That lasts until the 90-day
    retention ends (`src/provisioning/deactivation.rs:36,1067-1078`).

**Scenario:** one offboarding fails because an approver lost authority. The
operator contains the account and offboards it again. The alert never clears
and is eventually silenced, which then hides the next real failure.

**Smallest seam:**

- Give failed offboarding jobs a terminal exit: an explicit acknowledge or
  dismiss with audit, or retention like the other queues (SSF 7 days, mail 8,
  deactivations 90).
- Exclude deactivations whose resolution is satisfied from the `failed` gauge
  predicate. That is an index-predicate change, so bump `INDEX_VERSION` and
  handle the rebuild.

**Prospective files:**

- `src/offboarding.rs`
- `src/store/maintenance.rs`
- `src/provisioning/deactivation.rs` if needed
- `tests/o06_operations_evidence.rs`, `tests/offboarding.rs`,
  `tests/provisioning_deactivation_diagnostics.rs`
- `docs/operations.md`, `docs/enterprise/ENT-10.md`,
  `docs/deactivation-delivery.md`

### 2. Completed offboardings turn into permanent "incomplete" attention

Facet: incomplete offboarding, Platform only. This behavior is derived from
code; no test covers it.

**What happens:**

- **Delivered rows expire.** A delivered deactivation row is closed with
  `next_attempt = at` (`src/provisioning/deactivation.rs:195-215`). Ninety days
  later it is deleted (`RETAIN_SECONDS`, `:36`; `cleanup`, `:1067-1078`).
- **The job rollup turns the gap into "incomplete".**
  - `downstream_rollup` maps the missing row to `delivery_record_expired` and
    does not count it as delivered (`src/offboarding.rs:315-336`).
  - The job's downstream state therefore becomes `incomplete`
    (`src/offboarding.rs:341-349`).
  - `needs_attention` lists a done job whose state is `incomplete`
    (`src/offboarding.rs:482-492`).
- **The item outranks real work.** `attention_rank` places it at rank 1, ahead
  of `pending` (`src/offboarding.rs:494-505`). Offboarding jobs are never
  pruned.
- **It is documented as intended.** `docs/enterprise/ENT-10.md:41` lists
  "`expired` (retention elapsed)" under `incomplete`. No test asserts
  `delivery_record_expired`; it appears only in docs.

**Scenario:** every fully delivered offboarding older than 90 days reads as
`incomplete` with `delivery_record_expired`. It inflates the incomplete
counts and competes with genuine failed deliveries for the 50 rows. Ordering
falls to job id (`src/offboarding.rs:1080-1087`), so real items can be
truncated away. The operator cannot tell which offboardings are actually
unfinished.

**Smallest seam (needs a design choice):**

- *Preferred:* record each target's terminal delivery outcome on the job
  before the row can expire, so the rollup keeps "delivered" after retention.
- *Alternative:* report targets whose record expired as a distinct state that
  is counted but not listed for attention. A target that was never delivered
  is already listed before its row expires.
- Either way, add a test-clock case past 90 days.

**Prospective files:**

- `src/offboarding.rs:261-357,482-505`
- `src/provisioning/deactivation.rs` if the outcome is stamped at close
- `tests/offboarding.rs` or `tests/o06_offboarding_overdue.rs`
- `docs/enterprise/ENT-10.md:41`, `docs/deactivation-delivery.md:195-197`,
  `docs/operations.md:161`

### 3. A configured connector controller that never ran is silent

Facet: connector lag, both editions. This is the smallest seam in this
review.

**What happens:**

- `reconciliation_diagnostics` iterates stored schedules only and never reads
  `config.reconciliation_controllers` (`src/reconciliation.rs:942-996`, loop at
  `:950`).
- Schedules are created only by `sync_reconciliation_schedules`
  (`src/reconciliation.rs:998`). That runs only from `reconciliation_process`
  (`:1395-1411`), which runs only on a process with the background-jobs duty
  (`src/api/server.rs:50`, `src/background.rs:1382`).
- The only statement of this limit is on a roadmap page,
  `docs/roadmap/o06-reconciliation-diagnostics.md:63`. The operator pages do
  not mention it: `docs/operations.md:171`, `docs/api.md:336`,
  `docs/connector-incidents.md`.

**Scenario:** a gateway-only deployment, or one where no worker has started,
configures an LDAP controller.

- `GET /api/operations/reconciliation` returns `schedules: 0` and no
  attention rows, the same as a deployment with no controllers.
- `check_worker_duty` exists to report a missing worker, but it can never fire
  here.
- The Platform cloud view already reports `not_started`
  (`src/assembly/cloud_operations.rs:83-93`), but LDAP and SCIM have no
  equivalent.

**Smallest seam:**

- For each configured controller scope with no stored schedule, add an
  attention row: `record: "controller"`, `state: "not_started"`,
  `next_action: "check_worker_duty"`.
- Add counts `controllers_configured` and `controllers_not_started`.
- Keep the 50-row cap and ordering (jobs first).
- Document that the read reflects the answering node's file configuration.

**Prospective files:**

- `src/reconciliation.rs`
- `tests/o06_reconciliation_completion_age.rs`
- `docs/operations.md:171`, `docs/api.md:336`,
  `docs/roadmap/o06-reconciliation-diagnostics.md:63`,
  `docs/connector-incidents.md`

### 4. A remote-signer failure cannot be attributed, and `doctor` stays green

Facet: key problems. Remote signers are Platform only; the counter is shared.

**What happens:**

- **`doctor` checks only stored metadata.**
  - `doctor` calls `keys.active.jwk()` (`src/operations.rs:85-86`).
  - For a remote key, `jwk()` checks only the stored record's internal
    consistency (`src/crypto.rs:415-425`).
- **The real binding check runs only when signing.**
  - The comparison with `config.signers` (signer name, `key_version`,
    `public_jwk`) runs only at sign time (`src/kms.rs:114-119`).
  - No other code checks that binding (`git grep` of `signers.get`).
- **Every failure looks the same.**
  - Every remote failure returns the same `signer_unavailable` 503,
    "Configured signing service failed; no token was issued" (`src/kms.rs:90-96`).
  - That covers a binding mismatch, a token or CA file problem, a transport
    error, a bad status, a decode error, and a failed signature check
    (`src/kms.rs:52-86,136-139`).
  - `src/kms.rs` has no log statement. `riauth_signing_errors_total` is one
    unlabeled counter (`src/kms.rs:98-108`).

**Scenario:**

- The trigger is one of:
  - a signer edited or removed in `riauth.toml`;
  - a different pinned version on one node;
  - a token file whose mode is too open.
- `doctor`, `/readyz` and `GET /api/keys` stay green.
- Every token issue then returns 503.
- `RiAuthSignerFailures` fires, but nothing tells the operator which signer
  failed or whether to fix configuration or wait for Vault. Those corrective
  actions are opposite.
- `docs/disaster-recovery.md:282` adds to this by saying `doctor` reports
  "signing-key health".

**Smallest seam:**

- *Attribution:* classify the failure reason as a bounded token
  (`binding_mismatch`, `credential_file`, `transport`, `status`,
  `signature_check`). Log it once per state change with the signer name and
  no secrets, and expose the last reason and time in a read.
- *Binding check:* add a static, no-network binding check for every stored
  remote key (and each signing domain) to an operator read. Reuse the
  `src/kms.rs:114-119` filter. Report `affects_readiness: false`, a per-key
  `next_action`, and no key material.
- Do not change the pinned `doctor` field list
  (`tests/doctor_page_counts.rs:19-33`, reviewer pin). Correct
  `docs/disaster-recovery.md:282`.

**Prospective files:**

- `src/kms.rs`, `src/kms_essentials.rs` (same shape, remote reported as
  requiring Platform)
- `src/operations.rs` or a new key-health module
- `src/api.rs`, `src/cli.rs`
- `tests/identity/operations.rs`: this is the existing Vault test at `:5050`,
  reviewer pin. The CI identity lane owns `tests/identity/*`, so add a new
  test file unless the lead coordinates.
- `docs/kms.md`, `docs/operations.md`, `docs/disaster-recovery.md`

### 5. Corrective action is undocumented for many emitted action tokens

Gate clause 3, both editions. Docs only.

Outside `docs/roadmap`, these emitted tokens appear in no operator doc:

- reconciliation `inspect_connector_and_replan`,
  `refresh_authority_and_replan`, `wait_for_worker` and `inspect_controller`
  (`src/reconciliation.rs:555-563,649-653`);
- offboarding-job `inspect_local_failure`, `wait_for_local_retry` and
  `confirm_waiver_not_delivery` (`src/offboarding.rs:509-569`, reviewer pin
  for the full list).

`inspect_provisioning_job` and `review_ambiguous_delivery` are named in
`docs/operations.md` but not tied to a command; the remedies exist by state
name in `docs/scim.md:124-130` (reviewer pin).

By contrast, the deactivation and SSF tokens have tables
(`docs/deactivation-delivery.md:180-199`, `docs/ssf-delivery.md:152-156`).

**Seam:**

- Add one table in `docs/operations.md`: queue or route, the read to use,
  each token, and the command or decision.
- Add a reconciliation section in `docs/connector-incidents.md` with the
  aggregate read and the token remedies.
- Fix the stale lines found on the way:
  - `docs/roadmap/o06-offboarding-diagnostics.md:27` says "`riauthctl` has no
    offboarding command"; riauthctl has `offboard diagnostics`.
  - `docs/roadmap/o06-offboarding-overdue.md:16` cites `Core::maintenance`,
    which does not exist; it is `Core::cleanup`.
  - `docs/deactivation-delivery.md:31-33` calls the redacted aggregate
    "the Platform route", which contradicts the shared route.

**Prospective files:** `docs/operations.md`, `docs/connector-incidents.md`,
`docs/scim.md`, `docs/deactivation-delivery.md`, the two roadmap pages.

### 6. A node mismatch after start yields a 503 with no cause

Facet: node mismatch, both editions. Exposure only; O03 enforcement is
unchanged.

**What happens:**

- `/readyz` maps every `store.ready()` failure to the same 503 body and logs
  nothing (`src/api/probes.rs:46-60`). That covers activation by another
  release, the recovery gate and a read-only PostgreSQL.
- The cause string exists, "Store activation differs from this process; stop
  incompatible writers and restart with the active release" (reviewer pin
  `src/upgrade/activation.rs:181-182`), but nobody sees it.
- Refusal when the store opens is adequate. The process exits 2 with
  `error: <message>`, and rate mismatches name the category, both values and
  the fix.
- Capability and lifetime or password-policy refusals name only the
  category, not the setting, and there is no read of the recorded agreement
  (reviewer pins `src/node_security.rs:30-33,197-212`).
- `docs/availability.md` still tells operators to run
  `security-agreement-record --confirm-authentication-policy`. The command
  now refuses without `--confirm-rate-limits` (`src/cli/local.rs:361-366`).

**Smallest seam (first part):**

- Log the `ready()` reason once per state change with `tracing::warn!`. Keep
  the public probe body generic.
- Add a test that plants a different version activation and expects `/readyz`
  503 and `/livez` 200. No such test exists (reviewer finding).
- Fix the `docs/availability.md` command.

**Second part (optional):** a read-only `riauth-maintenance
security-agreement-status` that diffs the stored agreement against this
configuration. It reuses the existing comparison and does not change the
enforcement messages, which tests pin.

**Prospective files:** `src/api/probes.rs`, `tests/operations.rs`,
`docs/operations.md`, `docs/availability.md`,
`docs/enterprise/PLATFORM-04.md`. For the second part,
`src/node_security.rs`, `src/cli/local.rs` and `tests/maintenance_cli.rs`.

### 7. Storage pressure has no pressure verdict and no remedy

Facet: storage pressure, both editions. Reviewer pins, with lead spot-checks
on readiness.

**What is there:**

- `GET /api/operations/storage` reports physical allocated bytes.
- There are writer-wait (alerted), pool, queue and worker-rejection series.

**What is missing:**

- **No verdict.** `capacity` is hard-coded `unknown` and `occupancy_ratio` is
  null (`src/store.rs:146-149`). No configuration key exists to make them
  real. The allocation document has no `next_action`.
- **No remedy.** No operator page says what to do when the store grows,
  the pool times out (`storage_busy`) or the volume fills.
- **Write failures look healthy.**
  - A failed redb write becomes a generic 500. `Error::internal` logs the
    underlying error (`src/error.rs:54-61`).
  - Meanwhile `/readyz` and `doctor.healthy` stay green. `ready()` only reads,
    and `healthy` is `enabled_administrators > 0` (`src/operations.rs:100`).

**Smallest seam:**

- An optional validated `storage.capacity_bytes`. With it,
  `configured_capacity_bytes`, `occupancy_ratio` and `capacity: "configured"`
  are filled in, plus a ratio gauge and a rule.
- A `next_action` keyed by status, and an operator remedy table covering
  allocation growth, pool saturation or timeouts, and a full volume.
- Separately, a storage write-failure counter classified by backend and I/O
  kind.
- Measuring redb filesystem capacity needs a new dependency, because the
  crate forbids unsafe code. Treat it as optional.

**Prospective files:**

- `src/config.rs`, `src/store.rs`, `src/operations.rs`,
  `src/api/observability.rs`, `src/telemetry.rs`
- `deploy/riauth-alerts.yml`, `deploy/riauth-grafana.json`,
  `scripts/check-grafana-dashboard.py`
- `tests/storage_allocation.rs`, whose field list must change
- `docs/operations.md`

## Adequate as implemented (no change needed)

- **Redacted aggregates.** Reconciliation, provisioning, both deactivation
  routes, offboarding and SSF are adequate:
  - permission-gated and read-only, with `affects_readiness: false`;
  - 50-row caps and paged scans;
  - each names the job, target, stream or scope;
  - redb tests run in CI on the Platform build.
- **Edition split.** It matches code and docs.
- **Offboarding overdue detection.** It matches the claim path and has a
  documented containment step: disable the user, then restore the worker
  duty. Local revocation is separated from remote completion.
- **Connector lag wording.** Completion age is honestly labelled as local
  controller completion, not remote lag, on every surface. The overdue rule
  is `2 × interval + 300 s`.
- **Node agreement refusal at open.** It writes nothing before refusing,
  exits 2, and gives the exact fix for rate mismatches.
- **Allocation reading.** It is correct, bounded and cached single-flight,
  and its `unknown` capacity is truthful.
- **Signing failures fail closed.** `sign_jwt` returns 503 and verifies the
  remote signature against the pin. `docs/connector-incidents.md` describes
  what stays available.
- **Deactivation and SSF token tables.** They match the emitted vocabulary,
  and no documented token is never emitted.

## Recommended, not blocking under the row text

These are reviewer pins and lead judgment:

- **Reconciliation failures on dashboards.** They have no gauge or alert, and
  `riauth_background_failed_total` does not see stored job failures. The API
  read is the exposure. Adding count-only gauges and a rule would put the
  failures on the dashboard.
- **Missing alerts and panels.** PostgreSQL pool saturation and timeouts have
  none.
- **Signing counter gaps.**
  - The counter has no reason label.
  - SAML XML signing is uncounted.
  - No test proves the counter increments on a real signing failure.
- **Retained-key headroom.** `GET /api/keys` counts expired retained keys.
  There is no next-expiry or headroom field and no test of the 32-key refusal.
- **Manual-review completions.** In `ManualReview` mode a run that drafts a
  plan for review counts as completed, so completion age stays fresh while
  nothing is applied. No operator page says so.
- **Cloud view.** The per-connector view omits completion age and overdue.
- **SAML certificate drift.** After a key rotation, SAML certificate drift
  surfaces only as 400 errors.
- **Split deployments.** The worker role has no metrics route, so its signing
  and maintenance counters cannot be scraped. This needs an O01 design
  decision.
- **CI coverage.**
  - Essentials diagnostics tests do not run in CI.
  - The `postgres_*` diagnostics tests do not run.
  - `scripts/check-grafana-dashboard.py` and `promtool` are not run in CI.
- **Accepted redaction (needs an orchestrator ruling to change).** A deleted
  account's failed deactivation is count-only in Essentials.

## External, deployed or release evidence (separate from code closure)

- Alerts and panels shown firing on a deployed Prometheus with routing, and
  Grafana. A loopback import exists.
- **Remote connector lag.** The server stores no remote high-water mark, and
  connectors do full snapshot crawls. A truthful remote lag needs connector
  protocol work and live LDAP, Entra, Workspace and SCIM peers.
  - Lead judgment: local completion age, labelled as such, satisfies
    "expose connector lag" for code closure.
  - Root's earlier residual on a remote high-water signal is a product
    decision for root. It is not a diagnostic seam.
- A current multi-process PostgreSQL mismatch drill, with load-balancer
  drain behavior.
- Host disk, WAL and tablespace capacity; a real disk-full drill.
- Vault reachability, HSM custody, and TLS and certificate expiry monitoring.
- An official release containing these diagnostics. The audit records that
  `v0.1.1` predates them.

## Corrections to earlier O06 records

- **`docs/roadmap/o06-acceptance-audit.md`:**
  - It rates incomplete offboarding "Met locally". Findings 1 and 2 show
    misleading signals in that facet.
  - Its node-mismatch row says the refused process "logs one generic line".
    It prints `error: <message>` and exits 2 with no tracing line.
  - Rate refusals now name the category and both values.
  - It cites "activation and readiness tests". No test drives the
    post-start activation mismatch through `/readyz`.
- **`docs/roadmap/local-wave28-task-closure-audit.md:471`:** it cites
  "doctor signals" for node agreement. No `doctor` field reports the
  agreement (`src/operations.rs:100-111`).

## Remaining scope

O06 stays in progress. The Sol lead can claim findings 1–7. Suggested order:

1. Finding 3, which is the smallest and removes a false all-clear.
2. Findings 1 and 2, the two misleading offboarding signals. Finding 2 needs
   a design choice.
3. Finding 4.
4. Finding 5, docs only.
5. Finding 6.
6. Finding 7.

Root decides task status.
