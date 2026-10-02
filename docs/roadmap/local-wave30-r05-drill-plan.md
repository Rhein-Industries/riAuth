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

## Approved implementation and static evidence

Root explicitly reserved `wave30_R05_local_rp_drill`: the two scripts below
and an append to this plan, with Cargo/runtime still held. The named ledger
path is not tracked or locally present in this checkout; the explicit user
reservation supplies authorization. The original 209-line proposal above is
retained byte for byte. This appendix does not replace its historical evidence
or turn proposed runtime outcomes into observations.

Own clean branch aligned to fixed published
`c01c39ab4e092423d5522bedc50fff87656d8c0a` through history-preserving merge
`9823ba830dac6d0e11f7c8f5450f5b26f9d24dc3`, without conflicts, reset or stale
file replacement. All prior history was retained. The accepted production
`src` tree and this branch both resolve to
`3adc2b59c3547d22bff202daccfd8ad97f1e78ab`.

Separate source commit: **`d87c24f34ea811b18c5b3a3d801d54725c246d89`**.

| Exact file | Changes and immutable blob |
| --- | --- |
| `scripts/recovery-drill.py` | 80 additions/11 deletions. Fresh normal revision/idempotency inputs for user and public client creation; exact public callback registration with no secret; source/restored RP invocation; explicit service-login labels; observed source/script/helper/binary hashes and end-of-run binary consistency check; private exception handling and nested RP/service cleanup. Blob `fa9a1e4b54c2bb38318b6ad0ccf4331714d9c0bd`. |
| `scripts/recovery-drill-oidc.py` | New 473-line helper. Actual callback/S256/token flow, RS256 native verification, JWKS/claims/at_hash/userinfo binding, authenticated RP cookie/protected route, fixed failures and bounded private cleanup. Blob `3944ed071cbdb37b6e306c9c67d801029b54f43d`. |

The CLI flags remain unchanged. Legacy offline dispatch is reused; no
maintenance-binary option or PostgreSQL change was added. The fixture creates
an ordinary public client through the current fenced management route and
requires a null client secret and its exact registered redirect. No config
write changes the default review policy; a policy that requires reviewed
creation would refuse this direct fixture request. Confidential issuance,
first-only secret receipts, route-specific retry headers, review/removal/
permission/audit contracts and PAM fallback remain in accepted production.

The helper validates exact discovery endpoints on the known issuer, disables
proxy use and automatic redirects, and follows only the exact loopback
callback with a new cookie opener and no bearer argument. The RP handler
explicitly refuses an Authorization header. Callback state is consumed before
exchange; a cookie is issued only after native signature, time, issuer,
audience, nonce, at_hash and matching userinfo checks. An unauthenticated
protected request must return 403 and the fresh cookie request must return
200. Source/restored subject equality is checked privately. Cookie, verifier,
state, nonce, code, tokens and subject never enter returned evidence.

Python **3.11+** and an **OpenSSL** executable on PATH are explicit helper
dependencies (including the provider's actual version and public executable
hash in runtime observations). RS256 is the exact default fixture algorithm;
other algorithms/providers are refused, not skipped. Public RSA JWK material
is serialized as SPKI; `openssl dgst -sha256 -verify` performs verification.
Native output is discarded, verification files are exclusive mode 0600 inside
a private temporary directory and all raw network/foreign exceptions are
suppressed. HTTP handler/request traceback logging is silent. Bodies are
bounded to 256 KiB, token shapes to 32 KiB, ordinary HTTP/native calls to 5 s
and callback delivery to 30 s. RP shutdown closes its listener/thread and
clears private state; nested finally still stops the IdP if RP cleanup fails.
All of this is source behavior pending runtime evidence.

### Static checks actually run

- `python3` in-memory `ast.parse` and `compile(tree, filename, 'exec')` of both
  scripts passed. Neither module was imported/executed and no `.pyc` file was
  generated by these checks.
- A focused Python AST comparison against `git show c01c39a:scripts/recovery-drill.py`
  passed for seven original functions: `probe`, `start`, `stop`,
  `write_evidence`, nested `cli`, `success` and `refusal` remain AST-equivalent.
  Every original `run` assertion and refusal invocation is retained unchanged.
  All 16 original check IDs retain their order; exactly three RP checks are
  added. The safe assertion exception subclass/readiness error tag changes
  only reporting, with no weakened condition or deadline.
- AST/redaction assertions passed: helper `require` messages are fixed
  literals; explicit failures are fixed literals or forwarded fixed tags;
  no helper `print`, `eval`, `exec` or module-level runtime/listener call;
  login's evidence dictionary contains fixed strings/statuses/booleans/null,
  with no private variable values. `log_message` and `handle_error` are silent;
  foreign errors have no raw string rendering in parent evidence; the exact
  callback-follow call has no bearer argument. Source checks also confirmed
  loopback-only bind, exact callback URL, Authorization refusal, suppressed
  native output and exclusive private files. These are static checks, not
  executed attack/protocol cases.
- `git diff --check` and `git diff --cached --check` exited 0. The source
  commit's staged paths were exactly the two scripts. `git diff c01c39a --
  src Cargo.toml Cargo.lock rust-toolchain.toml` was empty; production/build
  inputs match the reviewed pin. Before this appendix, the plan was verified
  byte-identical to `49b319a2b431f4c73ab997b8938f12c8b044f212`.

One preparatory AST checker exited 1: its first callback-call selector also
matched the authorization call because both first arguments are subscripts.
The selector was corrected to identify `locations[...]` specifically; the
rerun passed. This was a checker error, not an observed protocol, compiler,
service or product failure. Source review also tightened discovery field
types, fixture session/redirect validation, file-handle closure and partial
RP startup cleanup before the source commit. No runtime was repeated.

### Binary provenance and held build/runtime

Read-only metadata/hashes, without executing either artifact:

| Cached artifact | Observed SHA-256 and size |
| --- | --- |
| `.target-wave27/debug/riauth` | `51aba4dc554fc70ee596d91bea0ec38dba0b8050177048db77bb545725095c25`, 259865728 bytes; mtime_ns `1790936463461041879`. |
| `.target-wave27/debug/riauth-maintenance` | `f1cc5b151453ecfda03b67b2c80ad4b8f0c1b61d69ae610e22ff6de0b4d2126e`, 68323984 bytes. Not needed by the proposed command. |

The Cargo fingerprints list feature sets but have no source-commit field.
This prior cache predates the newly imported production changes; it is not
attested as a `c01c39a` executable. `shutil.which` identified the candidate
`/opt/homebrew/bin/openssl`; its version/verification were not executed or
claimed. A fresh same-source build is required unless root supplies actual
matching provenance. Observed Git/source hashes in the script are not a
substitute for the operator's actual binary build record.

The exact **one proposed build**, sent to the explicit project orchestrator
before any Cargo use, remains held:

```sh
env CARGO_TARGET_DIR="$PWD/.target-wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo build --locked --bin riauth
```

After separate root release/build verification, the exact **one redb drill**
remains held:

```sh
python3 scripts/recovery-drill.py --binary "$PWD/.target-wave27/debug/riauth" --evidence "$PWD/.target-wave27/r05-wave30-local-rp.json"
```

Root schedules the sole slot; no Cargo slot was acquired or released by this
implementation turn. Check disk before any authorized build and stop near
8 GiB free. No Cargo, native OpenSSL or riAuth binary, test suite, service,
desktop/browser, PostgreSQL, external host or escrow procedure was executed.
No new worker/task/worktree or board/main/push mutation was made. Only these
three approved files are changed across source and append-only report commits.
The source hash/static-check handoff was sent with the explicit project ID;
root reviews/releases/integrates/publishes. Original R05 remains in_progress,
and W02/W05 remain DONE. Actual local RP outcomes, build provenance and cleanup
observations are pending the two separate runtime releases; the historical
PG/key-loss and real secret/escrow/deployment limits above remain distinct.

### Focused interruption cleanup follow-up

Separate source follow-up **`b9668be3649cecc3d126c9ffe5bee0ab3d8d4769`** adds
20 lines/removes 1 across the same two scripts. Scoped SIGINT/SIGTERM handlers
raise a fixed drill failure so controlled interruption unwinds the existing
RP/IdP/private-workspace finally paths. Further signals during that unwind do
not repeatedly interrupt cleanup; previous signal handlers are restored after
the attempt. Partial RP startup now also shuts down a started thread before
closing its listener. Forced process/host termination cannot be established
by static inspection; no signal/runtime execution was performed.

Final script blobs after both source commits: redb
`ada2dfa93ccac1ec132cc15fb4a2b5a042fad878`; RP helper
`3be747d03146f1bcaa3ec012ee8d173b61fa737d` (478 lines). The final focused
AST/syntax/original-contract/redaction/callback checks were rerun because
these concrete cleanup hunks changed source; they passed without importing
the modules or installing signal handlers. Staged whitespace/scope checks
also passed; the follow-up contains only the same two script paths.

`python3 scripts/check-docs.py` exited 0 for the evidence appendix (Markdown
links/build-directory layout). Report whitespace/fence/scope assertions and
the exact original-proposal byte-prefix comparison passed. No Cargo/native
verifier/protocol run is inferred from these checks. Source and final report
hashes are sent to root with project ID
`891e7443-8dac-4c1b-897f-9e53cb59c7ee`; the one build and one redb runtime
command remain queued for separate explicit release.

## Authorized matching-source build: actual outcome

Root explicitly released the sole Cargo slot for exactly the previously
proposed `cargo build --locked --bin riauth`, with private `.target-wave27`,
jobs=1, incremental=0 and dev/test debug=0. **One invocation exited 0**, dev
unoptimized, Cargo-reported duration **1m 23s**. No other bin, test, maintenance
or client build was run. The redb RP drill was not released or executed.

Build HEAD: `7229c6942dab2b874e182f3a1cba80a7f48fba18`. Before and after
the build, the tracked tree was clean and the production/build-input diff
against reviewed `c01c39ab4e092423d5522bedc50fff87656d8c0a` was empty.
Both production `src` trees remain
`3adc2b59c3547d22bff202daccfd8ad97f1e78ab`. This appendix changes only
documentation; it does not rebuild or alter the artifact.

| Build input | Git blob | SHA-256 |
| --- | --- | --- |
| `Cargo.toml` | `5660d4bb922fcdc5bfe05d7502585980f4720d06` | `58e5ef824ed96290179c9f76fea208dc37173caeee21b6ce8d37f8a6dd1abcb8` |
| `Cargo.lock` | `f1b819d47d204d73617b095513f0c6ab6eb8aa4e` | `b5c9d11c001244b8017303ce8c20516e02946483845eee40720b0910758d4426` |
| `rust-toolchain.toml` | `c3f67b6771b777215340531caf051bc25cef066c` | `887f9be066a15585a2c583578e84b0fcb541126d81546276bad3d2ff00d61167` |

Actual `rustc -Vv`: Rust **1.98.1**, commit
`48a229ceaefd4985c50990b14116b6d856af0985`, commit date 2026-09-01,
host `aarch64-apple-darwin`, LLVM 22.1.8. Actual `cargo -Vv`: Cargo
**1.98.1**, commit `797e8a9bca276c1c9f9f738d2a20f484fa4eea9d`, commit date
2026-08-05, host `aarch64-apple-darwin`. The full version outputs are retained
in the private provenance file, including Cargo's reported native library/OS
metadata. These are local build observations, not deployment/release evidence.

The newly written `riauth-76fa2f24f27377d8/bin-riauth.json` Cargo fingerprint
records actual features **`default`, `essentials`, `platform`**, without
test-support/fuzzing. Its SHA-256 is
`c6417922280c0fc42574e171e543783ee55756acb5224212dce685d6a2dc8339`.

Built artifact **`.target-wave27/debug/riauth`**:

- SHA-256 **`0f137475af5a8040d96a794b1ad331e7430be4467046b81b1312fb974b7e8a6a`**.
- Size **260096784 bytes**; mtime_ns `1790941283520579084`.
- Hash and source/manifest equality were checked after completion. The binary
  itself was not executed. Both future drill phases must use this one artifact;
  the old cached hash above is historical and was not used for an RP run.

The build emitted one macOS linker warning: `__eh_frame` exceeded the 16 MiB
compact-unwind encoding limit, with a possible exception-handling performance
effect. Compilation/linking still exited 0. No build failure, retry or source
correction was observed or made during this build.

Disk was sampled before, during and after the build. Start free space:
12934201344 bytes (**12.046 GiB**). Minimum sampled free space:
11797745664 bytes (**10.988 GiB**). Post-build hash/provenance check:
12891217920 bytes (**12.006 GiB**). Every sample stayed above the explicit
**9 GiB stop threshold**, with an **8 GiB floor**; no stop was triggered.
No cache, artifact or evidence was deleted. These are sampled measurements,
not a continuous minimum or a throughput/memory benchmark.

Immediately after exit/hash/disk inspection, root received the exact outcome
with explicit project ID and **CARGO SLOT RELEASED**. The slot was released
before this evidence append. No Cargo slot is retained for documentation or
for the still-held drill.

Private evidence retained under this worktree (exclusive mode 0600):

| File | SHA-256 |
| --- | --- |
| `.target-wave27/r05-wave30-build-provenance.json` | `165badd69d553c8d786b66f5c390f1eda20b8e7e7558dfb3203e9b01e7e9a52d` |
| `.target-wave27/r05-wave30-build.log` | `583316bcade84398d74abedcd329a278c7e4e1e1b3d277bdfe7f6743a7cf5620` |

The log preserves normalized Cargo output. Evidence-log assembly initially
hit a tool-host `TextEncoder` availability error after the provenance JSON
was already safely written. Numeric code-point encoding corrected only that
assembly step; the existing JSON was retained and no build/artifact was
repeated, replaced or deleted. This is not a Cargo or product runtime failure.

Remaining release dependency: root reviews this exact artifact and separately
authorizes the one previously proposed redb RP drill. No riAuth/OpenSSL binary,
service, protocol, protected-resource, signal, PostgreSQL, external escrow or
browser/desktop outcome was tested in this build turn. R05 remains in_progress;
W02/W05 remain DONE. Root alone owns integration/publication/status. Production,
scripts, manifests and all accepted contracts remain unchanged by this appendix.

Actual appendix checks: `python3 scripts/check-docs.py` exited 0; whitespace,
balanced fences, byte-prefix retention, single-report scope and empty reviewed
production/build-input diff checks passed. Both private evidence hashes/modes
and provenance JSON fields were verified. These are evidence/document checks;
no runtime release is implied.
