# Wave30 D04: original runbook and evidence review

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Task: `ec76d0c5-2efe-4005-bb49-1f3b54878146` (D04).
Existing worktree: `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`.
Date: 2026-10-02.
Audited published commit: `2f9affb0c3772f5ff09c07bf2f171a180dce8967`.
Worker starting HEAD: `e46224490582da8f9930b13a60b92dde8c145646`.

**The eight incident facets have documented entry points, but this audit does
not establish the independent user/operator completion gate.** One concrete
missing step was found in the numbered upgrade procedure: it directs an
existing-store operator to start before following the required offline
security-agreement upgrade. A single-paragraph follow-up was sent to the
project orchestrator before any existing-guide edit. It remains a proposal.

Only this review report is authorized for writing. Published objects were read
with `git show`/`git grep`; this worktree was not aligned or merged. The accepted
O06 implementation and evidence remain independent, and its original-row review
belongs to the separately assigned worktree. No D01/R05 runtime was repeated.
Root owns follow-up reservations, integration, publication and task status.

## Original live row and reservation

The live row was reread through `riwork worktree tasks
f2e8500e-2e56-47e3-b60e-9f81bbc8cff2 --json`. It is `in_progress` and assigned
here. Its requested outcome is:

> Cover lockout, credential incidents, failed connectors, outages, key loss,
> restore, migration, and rollback.

Its workstream goal is “Document completed user and operator tasks, not merely
available settings.” Its gate is “A new user and a new operator can independently
complete the documented workflows.” Prerequisites are U09, O06, O07, R05 and G05.
Its evidence clause requires reviewing implementation, tests, documentation and
released artifacts as applicable, reporting actual checks and external
prerequisites, and not completing implementation from documentation or a worker
report alone. The current assignment overrides the row's old scheduling hold.

The project ownership ledger's approved `wave30_D04_runbook_review` entry reserves
only `docs/roadmap/local-wave30-d04-runbook-review.md`, with runtime unreleased.
Repository contributor guidance was read; no repository or ancestor `AGENTS.md`
was found. The explicit report-only assignment governs this slice instead of the
contributor guide's general full-check campaign. Desktop preference remains
RiWork Cua.ai Driver, with descriptions and state read before separately
authorized use. No desktop interaction is needed or performed here.

## Supported paths through the eight facets

Commands below are source/runbook mappings at the published pin, not commands
executed by this worker. Paths and secret-bearing destinations remain private.

| Facet | Runbook entry, component and operator decision | Safety, remedy and completion boundary |
| --- | --- | --- |
| Lockout | [Administrator lockout](../admin-lockout.md): distinguish the local attempt lock from missing factors and from no remaining administrator. Keep the service running if a second enabled human administrator can sign in; read `revision`, then `user passwd NAME --password-stdin` and/or `user reset-mfa NAME`, each with its own current revision and idempotency key. If nobody can sign in but the store opens, use [offline break-glass](../operational-recovery.md#break-glass-administrator). | Password change clears `attempts` while retaining factors; MFA reset removes factors while retaining an active attempt lock. Read revision again between writes. Passkey-only and exposed/unproven accounts have explicit offline reset refusals. `recover-admin` requires stopping store owners, does not create an unknown account and does not clear a restored-state gate. Historical library and CLI tests covered the second-admin local path in both editions; browser/SMTP/LDAP and independent operator execution are separate. |
| Credential incidents | [Credential compromise](../credential-compromise.md): select a single session, all sessions for a user, lost passkey, agent/client secret, or default/named signing domain. Read the correct caller's session inventory before `session revoke ID`; use `user revoke-sessions`, `passkey remove`, `agent rotate/revoke`, `client rotate-secret`, or `rotate-key` for the chosen scope. | Preserve caller/resource permissions and the route's actual header contract. Direct user/agent/client/key writes require the revision/idempotency pair; single-session revocation has its documented optional receipt conditions. First committed agent/client secret issuance uses a new private destination; an exact retry cannot disclose the direct issuance secret again. Local revocation, queued delivery and remote acceptance are separate facts. Key rotation retains public verification keys through their retention windows and does not recall offline JWTs or RP sessions. This incident guide records source behavior, not a performed credential-compromise drill. |
| Failed connectors | [Connector incidents](../connector-incidents.md): start with capabilities, `doctor`, metrics and detailed delivery/job/controller reads. Identify LDAP, outbound SCIM, Workspace/Entra, SMTP, Vault or alert delivery before following that component's procedure. Reconciliation/provisioning aggregates and scoped job errors supplement process counters; a failed job is not uniformly a failed worker pass. | Probes do not contact those providers. Readable credential files and zero process counters do not verify a provider. Incomplete scans cannot authorize removals. For ambiguous SCIM delivery, inspect the target before replacement work; `provision stop` can leave an in-flight item ambiguous, `provision resolve` records observation, and `retry-deactivation` queues reevaluation rather than delivering immediately. LDAP-bound password login fails closed without a local-password fallback; that is distinct from the preserved PAM fallback contract. Remote signer errors retain fixed diagnostic reasons and no credential details. External tenant/receiver incident completion is not supplied by these source reads. |
| Outages | [Recovery decisions](../operational-recovery.md#decision), [availability](../availability.md) and [restored-state recovery](../recovery.md#postgresql-responsibilities-and-boundary): separate a stopped HTTP process, unavailable database, worker/admission shortage, connector dependency, and incompatible startup agreement/activation. Use `/livez`, `/readyz`, `doctor` and the relevant attention read. | A live process can answer liveness while storage readiness and login fail. PostgreSQL leader election and former-primary fencing belong to the database operator. Do not blindly repeat a possibly committed management write; preserve its original receipt key. A promotion with possible lost commits requires offline invalidation/reconciliation before serving, even when lineage is unchanged. Historical R05 includes actual local service/database outage results; it does not establish deployed HA, a production RTO/RPO or safety of a paused former writer. |
| Key loss | [Lost backup key while serving](../operational-recovery.md#backup-key-lost-store-still-serving), [database key removal](../operational-recovery.md#database-key-file-removed-while-serving) and [unrecoverable cases](../disaster-recovery.md#unrecoverable-cases): identify the backup encryption key, live database encryption key, signing domain, or referenced external secret. If the open store remains readable, generate a new private backup key and take/verify a new authenticated archive before losing that process. | A replacement key cannot open an old archive. File removal from an already running encrypted process is not loss of its in-memory key; successful `doctor`/backup there does not prove restart. Native encrypted database copies need the original database key; an authenticated archive can be restored under a new database key. If both needed keys/materials are unavailable, take the documented unrecoverable/reinitialize decision. There is no built-in escrow or passphrase recovery. Custody/retrieval from a deployment's escrow remains its operator's evidence. |
| Restore | [Disaster recovery order](../disaster-recovery.md#restore-order), [archive restore](../operational-recovery.md#archive-restore) and [native recovery](../operational-recovery.md#database-native-recovery): choose a compatible edition/reader; provision keys and referenced files; restore to a new redb directory or empty isolated PostgreSQL target; inspect the pending recovery ID; reconcile persistent credentials; explicitly complete that ID; validate one isolated serving process before moving traffic. | Whole-archive authentication precedes output; occupied/wrong-key targets and missing attestation/wrong recovery ID have recorded refusals. Omitting the output database key creates plaintext. A reopen failure can leave imported state and a pending config: discard/recreate that target rather than serve it. Archive authentication establishes integrity, not newest state or remote secret validity. A native same-cluster copy/PITR can retain lineage and needs explicit `recovery invalidate --database-restored`. Password login is an IdP session check; discovery/JWKS alone is not application login. R05 owns the separate current representative-RP work. |
| Migration | [Authentik migration](../migration.md): inventory complete exports and exact issuer/subject contracts; convert/preflight privately; resolve every blocker; validate, plan, review and apply a ready manifest; rehearse each selected application's actual issuer, subject, groups, factors, token and logout behavior. [Availability](../availability.md) separately documents offline redb-to-PostgreSQL storage migration. | Conversion never executes arbitrary source expressions, merges identities by email, or proves tenant/RP cutover. `ready_for_plan: false` produces no applicable manifest. Preserve Authentik and routing for rollback. State plan/apply carries its own reviewed plan/removal controls; generic direct-write header rules must not be invented for it. Storage migration requires a stopped source and isolated target, matches a retry prefix, commits bounded pages and publishes config only after final comparison. Group/formula and request-input limits below constrain these workflows independently. |
| Rollback | [Upgrade and rollback](../operations.md#upgrade-and-rollback): record binary/edition/hash; verify a pre-upgrade backup with a reader the rollback binary understands; stop all writers; perform the current agreement/edition handoffs; validate before traffic. On rollback, stop new writers and restore into a new target using a compatible reader, then reconcile the restored-state policy before serving where implemented. Authentik cutover rollback returns RP routing/configuration to the preserved source deployment. | Archive v3 compatibility, security-agreement format 3 and logical schema 3 are different fences. Do not point an old binary at a store the new binary modified. Old binaries that ignore activation/recovery fences do not gain those guarantees. Restoring a backup cannot merge post-snapshot identities, revocations or sessions. The numbered upgrade start step needs the narrow correction below; historical successful scratch restores do not prove a released-image or deployment rollback. |

`Core::doctor` still checks the active public JWK and enabled administrator count;
its `encrypted_at_rest` flag reports configured database-key use. It does not read
that key file anew, contact Vault or verify every signing domain. The accepted
key-health diagnostics and storage decorator remain separate operator outputs.
The configured allocation budget compares only a valid, fresh sample of its
declared store scope. Unconfigured, missing, stale, failed or mismatched samples
remain unknown/unavailable; a budget ratio is not filesystem free space, physical
headroom, write admission or an assertion that every storage domain is healthy.
WAL, backups and other files excluded from that allocation still need their own
host/database monitoring.

## One exact proposed follow-up hunk

**Proposed path:** `docs/operations.md`, only numbered step 4 under `Upgrade and
rollback` (published line 152). No existing-guide edit is made here. Root must
coordinate its concurrent ownership and reserve this hunk before implementation.

At the audited pin, `Core::open_store` calls `node_security::enforce` at
`src/core.rs:230`, before `upgrade::migrate` at line 232, lineage invalidation,
backfill and activation writes. `node_security::parse` refuses formats 1 and 2;
`enforce` refuses a missing agreement. Therefore step 4's direct-start promise
cannot carry such an existing store through migration. The necessary explicit
offline procedure already exists at operations lines 469–499, but the numbered
upgrade workflow does not direct the operator to it before starting.

```diff
-4. Replace the binary and start the new service against the **existing** configuration and store. Its first open performs any required migration. Check `/readyz`, `riauth doctor`, administrator login and representative OIDC/SAML/application flows before restoring traffic.
+4. Replace the binary. Before the first start, follow [Shared policy and offline upgrade](#rate-limits-and-admission) for an older or missing security agreement, using the source-edition maintenance binary and both confirmation flags; add `--adopt-missing-agreement` only for reviewed missing-row adoption. Then start the new service against the **existing** configuration and store. Its first open performs any required migration. Check `/readyz`, `riauth doctor`, administrator login and representative OIDC/SAML/application flows before restoring traffic.
```

Here “source-edition maintenance binary” means the current release's maintenance
tool compiled for the store's source edition, as the linked procedure requires.
The link retains the exact command, all-process stop, issuer/capability/policy/rate
alignment, refusal to overwrite conflicting format-3 policy, PostgreSQL client
count limitation, exclusive redb ownership and backup-based rollback. The change
does not alter any startup algorithm, recorded agreement, writer or readiness
gate. It does not duplicate D01 guide changes or R05 application runtime.

Initial claim and this proposal were delivered using `riwork orchestrator send
--project 891e7443-8dac-4c1b-897f-9e53cb59c7ee` before writing this report and before
any existing-doc edit. No follow-up production/doc reservation is inferred from
delivery or elapsed time.

Two other source-backed stale statements are recorded for root's disposition,
without proposing an additional owned edit in this slice:

- `docs/credential-compromise.md:110` maps all other HTTP statuses to exit 1 after
  listing 400/422, 401, 403 and 409/412/428. Current `src/cli.rs:82` also maps
  429/502/503/504 to exit 6 and marks that exit retryable. The general failure
  paragraph omits this class. The individual historical refusals remain intact.
- `docs/availability.md:3` calls `forward_auth` a per-process exception while its
  later agreement paragraph and `docs/operations.md:453` describe a shared
  counter. Current `src/api.rs:1116` calls `Store::shared_rate_limit` for PostgreSQL
  and uses the reserved forward permits for that category. The opening exception
  is stale; more nodes do not establish a larger configured PostgreSQL limit.

No broken migration plan/apply command was established: `Command::Apply` verifies
the saved server plan, refuses unread removals without confirmation, resolves
private secret references and submits that plan. Missing direct-write revision
flags must not be invented as a diagnosis for that example.

## Historical actual records retained at their pins

These are recorded executions from 2026-09-29, not new runs at `2f9affb`. A check
marked passed can deliberately observe a refusal. “Observed” and
“passed_gate_closed” are preserved as recorded, not relabeled as completed
recovery. D04 JSON records retain binary hashes but do not supply a Git source
revision for every binary. Their evidence blob plus binary hash is the available
pin; this review does not invent a missing build provenance chain.

| Record under `docs/roadmap/evidence/` | Recorded result and actual checks | Important failed/refused or unfinished result |
| --- | --- | --- |
| `d04-backup-key-redb-2026-09-29.json` | `passed`, 21 checks; Platform encrypted loopback redb; replacement key and new verified v3 archive, scratch restore. | Old archive with replacement key: exit 2 `invalid_request`, output absent. Original database key stayed present; scratch restore stayed gated. No completion, restored serve or restored login. |
| `d04-database-key-removed-redb-2026-09-29.json` | `passed`, 18 checks; Platform process exported after its database-key file was removed; scratch restore used a new database key; live inode unchanged. | No process restart, recovery completion or restored login. `doctor` reporting encryption remained a configured flag, not a current file check. |
| `d04-database-key-restart-redb-2026-09-29.json` | `passed`, 11 checks; Essentials restart attempted after file removal. | Exit 2 `invalid_request` in 153 ms, no listener, no timeout at the 15-second bound. Original redb inode/size/mtime/hash unchanged. This is an actual restart refusal. |
| `d04-recovery-complete-redb-2026-09-29.json` | `passed`, 18 checks; Essentials verified v3 restore under a new database key; explicit reviewed completion; restored ready 200 and fresh password login/doctor. | Pre-restore session: exit 3 `invalid_token`. This proves local IdP session rejection/fresh login, not an RP callback, token verification, SAML, escrow or official release artifact. |
| `d04-postgres-tls-2026-09-29.json` | `passed`, 7 checks; PostgreSQL 16.14 loopback, private CA, DNS SAN `localhost`; Platform init/serve/ready/login/doctor and `pg_stat_ssl` TLS observation. | Wrong CA and `127.0.0.1` hostname each exit 6 `storage_unavailable`; no output config or relation created. No external host/public CA/client certificate/database encryption key. |
| `d04-postgres-dump-restore-2026-09-29.json` | `observed`, 5 checks; custom logical dump to a fresh database, 25 rows; same system identifier, changed database/table OIDs. | Read-only status false with no pending ID; first serve applied `storage_lineage_changed`, removed one session/token and exited 5 before listener. Follow-up old-session/password requests were connection refused, not authentication rejections. Password/key reconcile counts each 1; completion unrun. |
| `d04-postgres-base-backup-2026-09-29.json` | `observed`, 8 checks; actual plain physical base backup, same cluster IDs/OIDs and timeline 1. | Before explicit invalidation: status true, ready 200 and old session accepted. Invalidation applied `database_restore`, removed one session/token and left password/key reconciliation. Next serve exit 5 without listener; doctor connection refused. Completion unrun. The old-session acceptance is retained. |
| `d04-postgres-pitr-2026-09-29.json` | `observed`, 10 checks; actual WAL recovery to a named restore point, timeline 1→2 but same lineage IDs; restored 25 rows/revision 0 versus later 32 rows/revision 1. | Status true before explicit invalidation. Invalidation ran before any restored serve; first serve exit 5 without listener, doctor connection refused. Password/key reconciliation remained, completion unrun. Timeline change alone did not establish detection. |
| `r05-local-2026-09-29.json` | `passed`, 16 checks; encrypted loopback redb archive; local source outage; restored gate completed; old session rejected, fresh service login/discovery/JWKS. | No-session backup and stopped-source login: exit 1 `operation_failed`. Wrong key and tamper: exit 2, output absent. Occupied directory: exit 5, marker preserved. Pending serve/wrong recovery ID: exit 5. Missing attestation: exit 2, gate preserved. No RP protocol login. |
| `r05-postgres-local-2026-09-29.json` | `passed`, 16 checks; encrypted local PostgreSQL archive into empty target; completed gate, old-session refusal and fresh service login/discovery/JWKS. | Database outage: live 200, ready 503, login exit 6 `storage_unavailable`. Wrong key: exit 2, target empty. Occupied target: exit 5, source preserved/serving. Pending serve/wrong ID exit 5; missing attestation exit 2. No physical/PITR completion or RP protocol login. |
| `r05-native-postgres-2026-09-29/report.json` | `passed_gate_closed`, 16 checks; real physical base backup with post-snapshot password/key changes; preserved lineage; explicit offline invalidation and direct artifact refusal checks. | Live-writer/occupied-target/wrong-key/current-writer refusals retained; occupied target's fixture exit is 1. Session/bearer `invalid_token`, code/refresh `invalid_grant`, proof `account_code_invalid` after policy. Restored stale password/key remained; no completion attestation or restored application serving. |

The lockout guide separately records two successful library runs and two CLI
process runs, Platform and Essentials, on disposable tempfile redb. It retains
wrong-password exits 3, lock exits 6, missing direct-write flags exit 1 before
HTTP, stale revision/reused key conflicts exit 5, password reuse exit 2, exact
password retry without a second revision advance, and successful relief-admin
access. Its library mail proof was read from an unsent outbox; that was not SMTP
delivery or browser completion. The guide is pinned below; the test source is
not a fresh operator execution in this review.

### Evidence and binary provenance

| Evidence path under `docs/roadmap/evidence/` | Git blob at the published pin | Executed binary SHA-256 |
| --- | --- | --- |
| `d04-backup-key-redb-2026-09-29.json` | `002c6b3aa73d44e9d89ab0061b367b76a0ddd3ec` | `d3b0fef5f892db201aab906a221d4f13c34bb984f43cdc793398db6d08a0efb7` |
| `d04-database-key-removed-redb-2026-09-29.json` | `ef9536733e252f80571cf3c5df2191ec3679aae6` | `7ede8878de9ad3d3a1b560ac41b6e15830cc54f81a6b187e674a9174cea0948d` |
| `d04-database-key-restart-redb-2026-09-29.json` | `350bf1172beefb5c949117460879fa831b540c9d` | `264ed196ec6366cd96aac6c2c058fa605af15541efd7160496e05f0a6d115e2f` |
| `d04-recovery-complete-redb-2026-09-29.json` | `4c890e7b9a8e1095b949766eef23f693c99ee13e` | `264ed196ec6366cd96aac6c2c058fa605af15541efd7160496e05f0a6d115e2f` |
| `d04-postgres-tls-2026-09-29.json` | `d2a4578c0d7861f0e9635bbea899feb7aa840509` | `de06f9b46ce3e4a929d4d065681325d664b9aedb6485f649ec098a57c22a6069` |
| `d04-postgres-dump-restore-2026-09-29.json` | `a27a78f169295a6bbb0d2133c8e96b903cc78719` | `de06f9b46ce3e4a929d4d065681325d664b9aedb6485f649ec098a57c22a6069` |
| `d04-postgres-base-backup-2026-09-29.json` | `43c737a1e7fb970104ea3241dbbe4ee28e7691e4` | `de06f9b46ce3e4a929d4d065681325d664b9aedb6485f649ec098a57c22a6069` |
| `d04-postgres-pitr-2026-09-29.json` | `215bcc9fe6625a052ee8b4048dd4f3a753e0128a` | `de06f9b46ce3e4a929d4d065681325d664b9aedb6485f649ec098a57c22a6069` |
| `r05-local-2026-09-29.json` | `24ad3e45be86154c1d0c277b043b46d848fa8c61` | `c125134b04154c39d58f953342cc20c2b50b512ebfc58e6fa6cf240a84aa2400` |
| `r05-postgres-local-2026-09-29.json` | `6a49ccb2b447c384ce01203d732a6f3fe30a3694` | `8e26d768ce419a6b31fc22bd517aaf7b07c359a74fc5801990b1b33271333ffa` |
| `r05-native-postgres-2026-09-29/report.json` | `ac811bf862fa61edd977919a8f94b05b2ebf1176` | `a5c01650917d16c9384b1d87fba373d02e97d5e85eb8a2e625d7c9a8cbcd68ea` |

Replacement-key maintenance binary: `aa9eba54aa19b8d25ed2383276df6649c9f94f06f7012ac8ccad1cbe2ff24f5a`.
Database-key-removal backup maintenance binary:
`b987de72bb557822ea7e00563dc99b19616258a8c7e40e394eee2d5ae35b51ae`.
Native PostgreSQL fixture SHA-256:
`bd34a8a4b4438c1ccb348fd20d9b867a3ad46a988a672adb0733400b96d1890b`.
The separate D01 historical account associates its `de06f9b…` source-built
snapshot with `58357fde77211e62dc51c14fb3fc216bdf143ceb`. This does not make
the D04 records current-build or official-artifact evidence.

## Limits that must remain separate

- **Artifact and format:** a package version string of `0.1.1` on a historical
  source-built executable is not proof of an official released binary/image.
  The guide explicitly notes released v0.1.1's v1/v2 reader versus the newer v3
  stream. Use actual binary hashes/reader capability, edition and referenced
  file compatibility; do not infer them from version text. Security-agreement
  format 3 is independent from archive v3 and logical schema 3.
- **Physical and logical:** archive import invalidates in the import transaction.
  Fresh-database dump lineage can trigger on open; same-cluster physical/PITR
  restores can preserve the compared IDs, including after a timeline change.
  The historical native cases do not attest the stale persistent password/key
  review. Client-count checks see sessions named `riauth`, not every paused or
  relabeled/pooler client. Stop/fence all nodes independently.
- **Tenant and application:** external LDAP/SCIM/cloud/mail/Vault/alert reads and
  actual provider acceptance are different. Local fixtures do not certify a
  tenant. A fresh IdP login and JWKS response do not finish RP sign-in; D01/R05/G05
  retain their separately owned application/workflow evidence and limits. This
  audit requests no extra external, browser or desktop campaign.
- **Escrow and file custody:** riAuth does not recover a lost encryption key or
  referenced secret file. Keep keys, config, external material and post-snapshot
  incident/audit evidence outside the host-loss domain. Reissuing an external
  credential and retrieving an escrowed key need their actual issuing/custody
  system; a temporary replacement key is not that retrieval.
- **Paused I/O and deadlines:** backup export checks bounded queue writes against
  its stall/deadline/shutdown state; the producer is a blocking task. CLI receipt
  has an idle-wait bound, then synchronous local file write/sync and blocking
  verification. Restore performs file/database work with cancellation checks.
  None of these checks proves a blocked OS/storage call or a paused process can
  be forcibly ended by that deadline. Use the documented bounds for the calls
  they control; do not infer hard I/O interruption, guaranteed write safety or
  an RTO. No paused-I/O run was performed here.
- **Groups and expressions:** `group_ancestors` rejects cycles and more than 1024
  ancestors, walking every exported group. Excluded groups are not flattened,
  and duplicate group names block. Expression parsing accepts at most 32 checks
  and depth 8; exact factoring tests up to 12 distinct independent group atoms
  into the supported condition shape. This is a bounded grammar and exact
  Boolean translation, not arbitrary Python/flow equivalence or a whole-input
  memory bound. Direct-group claim mappings require the stated membership
  equivalence after flattening.
- **Input and allocation:** local `ImportAuthentik` and `read_manifest` use
  `fs::read` before JSON/TOML parsing; they are not streaming paged importers.
  HTTP state plan/apply each have a 2 MiB body limit (`src/api.rs:570,574`),
  including the apply request's resolved secrets. That is separate from the
  generic 32 KiB route limit, private secret-file limits and the migration
  graph/formula bounds. Offline storage migration uses 128-record/8 MiB pages,
  while archive envelope/stream limits are different again. The runbook does
  not prove arbitrary-size input, physical storage headroom or safe automatic
  splitting of a reviewed migration. A request/input refusal must stop the
  operator for an explicit reviewed plan, not a relaxed permission/removal gate.

## Source and guide pins

All blobs below belong to audited commit
`2f9affb0c3772f5ff09c07bf2f171a180dce8967`; line references above refer to
those objects, even when the worker checkout's file differs.

| Path | Git blob |
| --- | --- |
| `docs/admin-lockout.md` | `0f3ab36243e5298aab203d4aef75dee37fc3c5ec` |
| `docs/credential-compromise.md` | `70addbd96e380480def0d602656339571e834479` |
| `docs/connector-incidents.md` | `309e26e3870b9a2154449f7acf2657bcae0c34d7` |
| `docs/operational-recovery.md` | `728a6c39ef50009b2b91bff5627d2a3262e590b1` |
| `docs/disaster-recovery.md` | `4bda09415a15e211394514ebe30930d635036ea9` |
| `docs/recovery.md` | `77ef778c176e0cc665c0a8d682ae1fa1db48aacd` |
| `docs/migration.md` | `9fb749445449bc14d42f1e5184d5515241ddef67` |
| `docs/availability.md` | `301ac7a029ccebbdcdae6c470747d719912c693d` |
| `docs/operations.md` | `652cbc779670b7dfa94abb768286e39802afbab4` |
| `docs/roadmap/local-wave30-d01-walkthrough-report.md` | `af738194b86825af62c684dc44ecc5bdeaec87d4` |
| `docs/roadmap/local-wave30-r05-drill-plan.md` | `98b2f6ebc925c51421add49d5d39239d6f11708e` |
| `src/cli.rs` | `581b8ee69ff73b1512fa583cc6be5dbc9f0745ee` |
| `src/cli/local.rs` | `57eccb2d1a7461e3b8b2442830984890233b8e8a` |
| `src/cli/backup.rs` | `904b48b7aaf62df57edaa306eeeec5f2751cc7a9` |
| `src/core.rs` | `f02efcde5e9604b7251a8171e0aa437dbd8d8ae2` |
| `src/node_security.rs` | `da029997ad9c1a805cc3c6a1f6a50954f79a16c3` |
| `src/recovery.rs` | `99718c37c4ec85f8e31ffced5bfb1722427b247d` |
| `src/operations.rs` | `31779eb7574041502f99304f54ba38032ad9f4d4` |
| `src/operations/stream.rs` | `58af716554cfd91f0d2eb669c9a54bff31facda9` |
| `src/api.rs` | `b2097b0aeeb5a5e12d6586628fe2d84a6dd9ed72` |
| `src/api/backup.rs` | `356c3684ce89bf38c44337a3a33352efb8bd0904` |
| `src/api/observability.rs` | `ce5058c231fc860feb572868e868db735508b69e` |
| `src/operations/storage_diagnostics.rs` | `483df8327822c6e5e0074dc1925efa6ae4bebf86` |
| `src/migration.rs` | `be21fbdfa8cac065ba0a796f41bb6214c9396e9c` |
| `src/kms.rs` | `9fd987eda0df672aa3e58ee3ad7cfddda1d3b699` |
| `src/kms_essentials.rs` | `3acb4be056709fe284f5f16dfd858e8c1af9f975` |
| `tests/admin_lockout.rs` | `bdc9f1efc8ed37f7f7df3db27a60fd609310480f` |
| `tests/admin_lockout_cli.rs` | `32905ac359907d377066c3b25e7926d766b93c8c` |

## Actual checks and remaining disposition

Performed only read-only task/ledger/guidance queries, fixed-object guide/source
inspection, and JSON parsing of the eleven historical recovery records. Their
scopes, outcomes, check counts, refusals and artifact pins were read rather than
inferred from file names. No Cargo, binary execution, service, database, browser,
desktop, external request, source merge, main/push or task-status mutation ran.

The first report-only static check exited 1 because it treated the relative
anchor inside the fenced proposed diff as a link in this report. The checker
was corrected to ignore fenced blocks and verify that proposed anchor against
its actual target document. This was a document-checker error, with no product
execution or changed runbook. The corrected static check exited 0: all eight
facet rows present, 39 Git blob pins and 11 recorded binary hashes matched,
16 guide links/anchors resolved at the published pin, and the proposed old
paragraph bytes and destination anchor matched. Final newline, whitespace and
balanced fences passed. These are document checks, not product acceptance.
`git diff --cached --check` exited 0; scope inspection showed only this new
report, with no preexisting staged or tracked changes and unchanged worker HEAD.

The next concrete implementation is the proposed single upgrade paragraph, only
after root reserves it. Root can assess D04's eight-facet/operator decision scope
without demanding every possible host, tenant, escrow or physical-pressure
matrix. The independent user/operator gate still requires evidence from the
documented chosen workflows; this worker's source audit and report are neither
that implementation nor an independent operator completion. D01/R05 runtime,
external custody/provider/deployment acceptance and root's final row review
remain with their existing owners. No D04 closure/status change is made here.
