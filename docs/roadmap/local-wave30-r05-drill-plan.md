# Wave30 R05: bounded local application recovery drill proposal

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original R05 task
`8c477e5e-8055-4cbe-9f3d-0c69fb19a2e8`; existing worktree
`a1303b57-4a34-487e-9c63-a841f05b51a0`, branch
`roadmap/local-workflow-safety-wave27`. Source and historical evidence were
read from fixed published main
`60437b59933cadd40a1f5fbbb91ba153aee56456`. Own starting HEAD is
`386e7a7464fb7ca33731f1b26a780c5da91d93ad`; no alignment or reset was performed.

The live row is **in_progress** and requests: “Test successful and failed
restores, outages, key/secret loss, and representative application login;
record actual outcomes.” Its workstream gate is: “A fresh operator can
restore the deployment and verify access without relying on undocumented
knowledge.” The latest explicit assignment authorizes this proposal only.
W02/W05 remain DONE; no status change is made or recommended from this audit.

## Delivered cases to retain

These are historical recorded outcomes, not fresh runtime results at
`60437b5`. Their JSON scopes and binary hashes remain authoritative for
the executions they describe. No old evidence file will be rewritten.

| Existing evidence | Actual outcome and limit |
| --- | --- |
| [R05 redb, 2026-09-29](evidence/r05-local-2026-09-29.json) | 16 checks passed: encrypted local source, authenticated archive, source outage/login refusal, wrong-key and tampered-archive refusal with absent output, occupied target preserved, verified restore, pending serving gate, wrong recovery-id and absent-attestation refusal, explicit synthetic reconciliation, old-session refusal and fresh service login. Discovery/JWKS returned 200. No RP callback or code exchange. |
| [R05 logical PostgreSQL, 2026-09-29](evidence/r05-postgres-local-2026-09-29.json) | 16 checks passed on a private PostgreSQL 16.14 cluster. Database outage under the live service gave live 200, ready 503 and `storage_unavailable`; wrong-key restore left the target empty and occupied-target refusal preserved the source. Restored gate completion, old-session refusal and fresh service login passed. No application callback or physical/PITR reconciliation claim. |
| [Native PostgreSQL artifact drill](evidence/r05-native-postgres-2026-09-29/report.json) | 16 checks, `passed_gate_closed`: actual physical base backup, post-snapshot changes and artifact invalidation/refusal. Explicit invalidation was necessary despite preserved lineage. Reconciliation remained pending; no restored serving/application-login success is inferred. |
| [Lost backup key](evidence/d04-backup-key-redb-2026-09-29.json) | Removing the backup key did not stop the running encrypted redb service. A replacement key could create a new verified archive; opening the old archive with that key returned exit 2 `invalid_request`, with output absent. Scratch restore stayed gated and the live inode was unchanged. This does not recover the lost key. |
| [Live database key-file removal](evidence/d04-database-key-removed-redb-2026-09-29.json) | The already open encrypted redb store could still export a verified archive after its key file was removed. Scratch restore used a new database key and remained gated; the live inode was unchanged. No process restart or original-key recovery was claimed. |
| [Database key missing at restart](evidence/d04-database-key-restart-redb-2026-09-29.json) | Restart returned exit 2 `invalid_request` before opening a listener. The original redb inode, size, mtime and SHA-256 remained unchanged. |
| [Recovery completion, redb](evidence/d04-recovery-complete-redb-2026-09-29.json) | Verified restore under a new database key; explicit completion allowed readiness, rejected the pre-restore session and accepted fresh service login/doctor. This does not establish an RP login. |
| [Later native ARM64 encrypted runs](evidence/a08-native-linux-arm64-2026-09-29/encrypted-storage/ENCRYPTED-RUN.md) | Both editions have recorded 16-check logical PostgreSQL recovery success, including completion and fresh service login. The original missing user-creation revision/idempotency failure and corrected result remain recorded. They are not rerun or generalized to the present host/build. |

The existing [R05 guide](recovery-drill-r05.md) explicitly separates service
password login from application sign-in. Both requested script bodies were
read: `scripts/recovery-drill.py` and `scripts/recovery-drill-postgres.py`.
Neither executes `/oauth/authorize`, receives an RP callback or redeems an
authorization code. A successful discovery response and nonempty JWKS alone
do not prove that an application can verify a login token.

For referenced secret files, `tests/identity/operations.rs::
email_capabilities_require_local_smtp_credential_before_serving` contains a
focused missing-file serve refusal and capability checks. That is source
inspection here, not a newly executed result or evidence that deployment
SMTP/TLS/directory/device-trust/RADIUS secrets were reissued. The custody and
secret-file instructions in [disaster recovery](../disaster-recovery.md) and
[operational recovery](../operational-recovery.md) retain those prerequisites.
No fake escrow or external-service recovery will be added to this slice.

## Exact prospective ownership

The following scope was sent to the explicit project orchestrator before
any script/helper edit or runtime. Only this plan is authorized for writing
now; implementation and runtime wait for root reservation.

1. **`scripts/recovery-drill.py`**: supply the already required fresh public
   revision and idempotency inputs for its existing user creation; create one
   public loopback OIDC client through normal client creation with its own
   freshly read revision and idempotency key. Call the RP helper before the
   authenticated backup and again after the existing explicit recovery
   completion, old-session refusal and fresh user login. Preserve every
   existing restore/outage/refusal/attestation check and cleanup. Label IdP
   service login and RP application login separately. Record redacted RP
   outcomes and exact source/build/binary/script/helper provenance.
2. **New `scripts/recovery-drill-oidc.py`**: one Python standard-library local
   RP with a loopback callback and protected resource. Keep random state,
   nonce, S256 verifier and application session private; use new transaction
   and RP session state for each phase. Perform the real public authorization
   and token protocol described below. Verify the fixture's default RS256
   signature with native OpenSSL using the trusted issuer's public JWKS;
   public JWK-to-key serialization is helper code, RSA verification is not.
   Require Python 3 and the OpenSSL executable explicitly; no downloaded
   package or general algorithm/provider expansion.
3. **This plan**, then an actual-evidence append here and one new redacted run
   JSON under `docs/roadmap/evidence/` only if root reserves those writes.
   No existing operator guide or operations token-table edit is needed for
   this first slice; root can integrate the operator-facing instructions.

No PostgreSQL script change is needed for the first redb slice. Later helper
reuse there requires a separate exact reservation and PostgreSQL runtime.
No production, configuration, schema, recovery writer, API, approval,
workflow, credential or client-contract changes are proposed.

### Actual protocol trace to add

- A single local RP listener retains the same exact registered redirect URI
  across source and restored phases. The backed-up client and same issuer
  are used both times. Each phase starts with a fresh RP cookie jar; fresh
  IdP password login is only its authentication prerequisite.
- Submit a unique `response_type=code`, `scope=openid profile`, exact client
  and redirect, state, nonce and S256 challenge to public
  `POST /oauth/authorize`, with explicit `decision=approve` and the private
  fixture service session. Disable automatic redirects. Validate the returned
  callback against the registered loopback origin/path and expected state/iss,
  then follow it without forwarding the IdP Authorization header.
- The RP receives the real callback and checks its outstanding one-use state.
  It exchanges the received code at public `POST /oauth/token` using
  `grant_type=authorization_code`, the exact redirect/client and its verifier.
  Validate issuer discovery endpoints before use; allow only the expected
  loopback issuer and RP endpoints, with bounded requests/body sizes.
- Select the unique matching RS256 signing `kid` from that issuer's JWKS and
  verify the ID-token signature with OpenSSL. Check issuer, audience, nonce,
  expiration/issued time and nonempty subject; verify the access-token binding
  (`at_hash`) and matching userinfo subject. Unexpected algorithms, keys,
  redirects or claims fail the drill rather than receiving a skipped check.
- Only then create the RP's private application session cookie and require a
  successful authenticated request to its protected resource. Compare source
  and restored subject in memory and record only the equality result. Fresh
  callback, token verification and protected access after restore are required;
  an old RP cookie or decoded JWT payload cannot satisfy the result.
- Suppress query-bearing HTTP logs and secret-bearing exception/CLI output.
  Evidence contains fixed check identifiers, statuses, booleans, counts and
  public artifact hashes, not tokens, passwords, authorization codes, state,
  nonce, verifiers, private keys, cookies or raw subjects. Stop both listeners
  and delete the private workspace on every exit, including failures.

This is a synthetic local relying party using actual HTTP and native signature
verification. It does not establish browser UI, SAML, a deployed third-party
application, external TLS, key escrow, multi-node promotion or release artifacts.

## Concrete current-source seams and protections

- At fixed main, `src/cli.rs:2596` requires both `--if-revision` and
  `--idempotency-key` for user creation; the redb drill's call at line 152
  omits them. The PostgreSQL drill already obtains and supplies these inputs.
  Correct only the redb fixture call, without relaxing the public contract.
- `src/cli.rs:2691` similarly fences direct client writes. Fixture client
  creation uses the fresh revision and unique receipt key. The default private
  init policy has `reviewed_client_creation=false`; no setting is altered to
  evade review. If that assumption fails, stop and report the dependency.
  `src/management.rs:2663` continues to require a reviewed creation change
  when enabled. No confidential secret is needed or generated; accepted review
  authority and first-only secret/receipt behavior remain untouched.
- `src/cli.rs:1382` still dispatches legacy local commands through
  `cli::local::from_legacy`. Although the PostgreSQL drill supports a separate
  maintenance binary, the redb script does not need a new option for this
  source build. An initial intermediate commentary suggested otherwise;
  inspection corrected that suggestion before the ownership proposal.
- `src/api.rs:1466` accepts the public authorization form and delegates the
  bearer session to `Core::authorize`; `src/assembly/oidc.rs:842` and the
  explicit approval arm issue a real redirect/code. The public token handler
  at `src/api.rs:1673` delegates the form to the code exchange, whose exact
  client/redirect/verifier/expiry/identity and replay checks remain unchanged.
- `src/crypto.rs:351` generates RS256 by default; its JWKS includes RSA n/e,
  algorithm and kid. Signature verification uses that actual public material,
  not merely a nonempty-key check or an unsigned token decode.

The revision/header additions are fixture compatibility, not new management
semantics. Review/removal/receipt/audit/permission protections, credential
mutation/recovery gates, shared activation contracts and PAM fallback remain
unmodified.

## One prospective runtime command and prerequisites

Run from this existing worktree only, after root reserves implementation and
releases runtime:

```sh
python3 scripts/recovery-drill.py --binary "$PWD/.target-wave27/debug/riauth" --evidence "$PWD/.target-wave27/r05-wave30-local-rp.json"
```

One same-build binary serves the source and restored redb phases and handles
their CLI calls. The evidence output must be new and private. The current
cache is not claimed to contain a binary from `60437b5`. Root must authorize
clean history-preserving alignment and separately release any necessary
focused `cargo build --locked --bin riauth` to establish exact build provenance
before that invocation. Required private environment: `.target-wave27`,
jobs=1, incremental=0, dev/test debug=0; stop near 8 GiB free. No Cargo, binary
execution, service, PostgreSQL, browser, desktop or external-host run was
performed for this plan. Do not silently reuse an unpinned old executable.

The runtime will retain all existing bounded CLI/readiness/stop deadlines,
add bounded RP/protocol/OpenSSL calls, and record failure honestly. Only a
concrete correction within reserved scope can justify repeating that one
drill. No broad campaign, historical check rerun or invented external outcome
is proposed.

## Audit evidence and remaining dependencies

Read the original live row using `riwork task list --project
891e7443-8dac-4c1b-897f-9e53cb59c7ee --json`, repository contributor/security
guidance, both scripts, the named historical evidence and fixed source traces
above. No repository/ancestor AGENTS.md was found. The explicit read-only
assignment overrides the contributor guide's general full-check campaign.

Actual document checks: `python3 scripts/check-docs.py` exited 0 (Markdown
links/build-directory layout); `git diff --check` exited 0. A report-only
Python check passed final newline, trailing-whitespace, balanced code-fence
and assignment/source-pin assertions. Status/scope inspection showed only
this new report, with no tracked source or staged changes. No runtime failure
or successful application login was observed; neither was executed.

Pinned script blobs: redb `0ee28365f6eeec7d1d2d26b4ee0e262b4171b23e`;
PostgreSQL `6a1fcb41443f63500ae125dda819a2bdc76c8844`. Historical R05 guide
blob `1dfe95476f7b38d7bb2db916ac645fd9f0d199f1`; original redb/PG evidence
blobs `24ad3e45be86154c1d0c277b043b46d848fa8c61` and
`6a49ccb2b447c384ce01203d732a6f3fe30a3694`. These are read provenance, not
fresh behavior verification.

Only this report is written. The proposal was delivered to the orchestrator
with the explicit project ID before script edits/runtime. Root owns scope
reservation, build/runtime scheduling, review/integration/publication and
task status. Local RP success would close the concrete representative
application-login gap, while the original delivered refusal/outage/key-loss
observations remain scoped to their recorded fixtures. Real secret
reprovisioning, escrow retrieval, PostgreSQL physical/PITR reconciliation and
deployment-specific fencing require their actual operators/environments and
remain distinct. No report-only DONE recommendation is made.
