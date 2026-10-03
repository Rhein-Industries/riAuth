# I06: real Entra acquisition source audit and next input

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original I06
`f9eb300d-ed58-4f86-80b4-cab14611af1e`. Supporting worktree
`42bb51c6-c198-4adb-bd92-0a5222853231`, existing shell
`f1575610-c6f0-4dbf-9f33-d01ed057a194`. Reservation:
`wave30_I06_real_entra_acquisition_source_audit`.

**Recommendation:** retain the original row's open disposition. The inspected
local implementation covers all six requested facets; this audit found no
concrete missing local facet to reserve for implementation. The smallest next
step is one explicitly authorized, bounded controlled-Entra lifecycle checkpoint
using existing interfaces. Tenant consent, synthetic test data, credential
rotation authority and an isolated local application target are the missing
inputs. Local mock passes and I10 file diagnosis do not establish those results.
This report neither starts that checkpoint nor changes the assignment/status.

## Original row and source boundary

Read the current project export at
`planning/current-tasks.json` under this project's RiWork orchestrator directory.
The 244,354-byte export SHA-256 is
`0b38ca3eb8810a4628436eb5083f6329de848f3f4ddaa972c850601ad45c0835`;
the exact row remains `todo`, with original primary worktree
`c8271bee-cb63-43d9-a4fb-3b01ef80c51d`. Prerequisites are A06, P01, P03, Q01.
Its original requested outcome, verbatim:

> Validate token acquisition, paging, group membership, identity mapping, removals, and credential rotation.

Its workstream gate, verbatim:

> Every advertised integration has a working setup, lifecycle, and failure-handling path—not just an endpoint or protocol module.

The row also requires relevant implementation/tests/docs/artifacts as applicable,
actual checks and remaining external prerequisites, and forbids completion from
documentation or a worker report alone. This support reservation supersedes its
old no-launch scheduling text only for this read-only audit. Root owns status.

All product/fixture witnesses below are immutable published Git objects at
`544d1340b80cd3e040dc13142cdcbc1d75fea4cb` (abbreviated `544d`). Own clean entry
HEAD was `8a72c757d2ebeef90dbe0667aa2886e16e9f2594`; no alignment/merge was
performed. Full pinned `CONTRIBUTING.md` and `SECURITY.md` were read. They require
secret/personal-data protection and distinguish local fixtures from live peers.
No applicable `AGENTS.md` was found in this worktree/ancestor guidance search.
The explicit one-report/static-only reservation controls this audit's checks;
there is no broad campaign or documentation-index edit.

## Original acceptance mapped to complete source and assertions

Line references here refer to `544d`, not the older checked-out production tree.
The fixture column describes bodies read, not a fresh execution.

| Facet | Connected implementation and exact assertion witness | Remaining real-Entra observation |
| --- | --- | --- |
| Token acquisition | `src/cloud_directory.rs`: `EntraDirectory::validate`, `validate_entra_endpoint`, `access_token`, `certificate_assertion`, `http_client`, bounded body readers. Exactly one credential mode; tenant-specific v2 token URL/Graph cloud/`.default` agreement; client-credentials grant; fresh PS256 assertion, exact audience/client ID, SHA-256 certificate thumbprint, random jti, 300s validity; sign then verify matching RSA pair before token request. The complete mock token handler checks grant, assertion type/signature/claims and no shared-secret fallback. | Actual consented app obtains a token from its tenant, and that token reads Graph. No token or tenant response was acquired here. |
| Paging/users/groups | `CloudSnapshot::{new,endpoint,advance,bounded}` plus `CloudSnapshotDraft` and separate plan-bound `CloudApplyDraft`: users/groups and selected transitive-members collections; `ConsistencyLevel: eventual`; mandatory first count, exact completed totals, unique IDs/cursors, collection/origin-preserving nextLink. Default five pages/call, 20/collection, 2,000 objects/collection, 200 rows/page, 1 MiB/page, 4 MiB crawl/draft, 8 KiB link, 30s call budget. `entra_transitive_membership_requires_complete_graph_pages` checks two member pages/nested non-user handling and failed-member-page preservation. `entra_controller_rejects_repeated_resume_and_waits_for_complete_source` checks five-page durable progress, failed/repeated resume, reopen and complete reviewed removal. | A real multi-page tenant collection reaches completion under the counts and bounds, including selected membership data. A probe's first users page is insufficient. Graph list pagination is not a remote point-in-time snapshot. |
| Group membership | `parse_user`/member-type handling, selected/chosen group binding, `materialize`, `assembly/cloud_directory_reconcile.rs::{authorize_reconcile,reconcile}`. Only users in the complete user snapshot enter existing allow-listed local groups; unmapped groups are untouched. The counted-snapshot fixture refuses an internally counted member whose user is absent; transitive fixture covers nested group objects without treating them as users. | Exact selected Graph object IDs and independently known direct/transitive user membership agree with the private reviewed plan and local result. Hidden membership needs its declared permission only if selected. No Source Group materialization change is proposed. |
| Identity mapping | Settings identity fingerprint plus forward/reverse `Binding`; `materialize` keeps linked username; shared cloud-owner checks and `write_user_record` preserve ID/subjects and refuse adoption of administrators, existing accounts, LDAP/foreign-directory ownership. `cloud_user_writer_preserves_identity_authority_and_revocation` asserts stable ID/subjects/username across mailbox change, clears email verification, checks scoped denial, exact audit-once replay and session revocation. `tenants_do_not_share_users` separates links. | Known tenant/user object IDs map to expected new local identities; changing a synthetic mailbox keeps the same subject/link. Matching mail is not authorization to adopt an existing account. |
| Removals/review/rollback | `assembly/cloud_directory_plan.rs` performs rollback-only preview, content/authority commitments and fixed expiry; apply checks live authority before replay and again before commit, recomputes impact, checks exact complete entries/revision/config, then shared `ApplyGate` and reconciliation in one mutation. `RemovalImpact::assess`, `large_removal`, `ReviewBinding::confirm`: every missing linked user or managed membership removal requires exact-plan review; explicit disables require review at all-active, >=5 and >=20%, or >=2 and >=50% (equality included). Complete partial-page, counted-snapshot, repeated-page, controller-mode and replay fixtures inspect unchanged identities/groups on refusal. | Tenant-owner-authorized synthetic disable/departure/membership change is reviewed and locally applied; refusal before confirmation leaves canonical state unchanged. Upstream accepted changes and downstream delivery are not rolled back by a local refusal. |
| Credential rotation | Private secret read <=4 KiB on every fresh token request; cert single PEM <=32 KiB, currently valid RSA 2048–8192, owner-only private key <=16 KiB; relative paths resolve from config. Certificate fixture rotates a real generated pair, rejects mismatched pair before token hit, then accepts matching pair against the local mock, with no secret fallback. `secret_file_is_reread_and_private` is a Workspace fixture of the shared reader, not a real Entra secret rotation. I10's complete Entra fixture observes private-file metadata/bounds/redaction and explicitly zero provider requests. | Provider accepts the new secret/certificate during declared overlap; old credential retirement/expiry then fails fresh acquisition as expected. File timestamp/readability is not acceptance, and replay of a saved verification result is not fresh proof. |

The shared cloud writer checks directory and every affected-user authority,
ownership/indexes and security epoch before persisting. `identity.rs`'s complete
`record_transition`/`user_security_transition` bodies revoke owned agents and
Windows credentials, queue logout, emit one disabled signal and enqueue durable
downstream deactivation intent in the same transition. The counted fixture's
complete `Dependents` helper checks those revocations and actor-attributed audit;
its downstream assertion is **pending intent**, not delivered SCIM or a Windows
installation. Re-enable does not restore revoked credentials.

`Store::{write,preview,postgres_write}` bodies were read: redb previews abort;
PostgreSQL previews roll back, and errors return before commit. Planning can save
drafts/plans/audit; failed fetches can save retry bookkeeping. “No account change”
does not mean no bookkeeping write. No local source snapshot freezes Graph, and
no local rollback reverses a remote write already accepted by a peer.

Complete relevant schedule/control bodies were read in `src/reconciliation.rs`:
controller validation/scope/fingerprint/private credential, scoped live agent,
cloud verify/schedule update, scoped job/schedule reads, dispatch/renewal and
transactional apply-lease validation. Dispatch rereads the configured agent
credential and checks current authority/configuration; local completion is
labelled `local_applied`, not remote delivery. No schedule or controller is
required for the smallest manual checkpoint, and no I10 control is reimplemented.
These reconciliation-job source checks do not relax the separate accepted
nonrenewed-60s/paused-I/O delivery boundaries or establish settlement of an old
remote request. Such uncertainty remains operator-owned.

## Executed evidence credited at its own pin

No fixture, helper, provider, product binary or artifact was executed in this
audit. These are recorded historical results, with their stated evidence limits.

| Evidence | Exact attribution/result | Limit |
| --- | --- | --- |
| Original I06 focused snapshot work | Public project `planning/worker-reports/i06-final.json`: author `5c48e35e3c162e89c4d842c15ce5bf592b6baa67`, accepted equivalent `d7f01826d760f5407ee7c9215ad605fe9089fce7`; reports the counted-snapshot test PASS and diff check PASS. Full current test body matches the author's body. | Local Graph mock; this final handoff does not give an exact argv/raw-log hash, so none is invented. Nested/guest/hidden/index-lag tenant acceptance was explicitly unverified. |
| Accepted certificate work | `6ea7f765ec468be6a73a5b0f408bf681e8253d12`, credited in pinned original-row closure audit. Current complete certificate fixture and provider/mock bodies read. | Accepted source/fixture, not a separately reconstructed tenant execution or new individual pass count. |
| Historical complete cloud target | Published six-task/D03 evidence records `03606ed7cd42ea3f716e3f8dcbebf001da9c3470`: `cargo test --locked --test cloud_directory -- --quiet`, 33/33; same target with test-support, 35/35. At `4df94b61edf9161cac5217b00691e55fdb6ee6ed`: `cargo test --locked --features test-support --test cloud_directory -- --quiet`, 36/36. | Aggregate cloud mock results, not 36 Entra recipes. Twelve selected complete function spans match `4df94` below; whole current suite/helper equivalence or a fresh suite pass is not claimed. Raw historical logs were not reopened here. |
| Accepted A03 focused cloud behavior | Full executed-check section of `local-wave27-module-boundaries-report.md`: locked/offline test-support cloud target, seven passes: quota (1), `cloud_connection_probe_` (2), missing-config authorization (1), credential replay preflight (1), credential write revocation (1), credential write revision (1). Source/report code pins `77c62c50c64a92bacab29bdd9e6a729569ffe500` and `3a4cd2eaf98b19c3af20c5a1510dd9cf4cd8cdb0`. | Mixed shared/Workspace operational behavior. It does not add a real Entra peer result. |
| Fresh, accepted I10 diagnosis | Source `ef80983e13fa5923ae5c2a29b520e415e1205240`; report-only execution HEAD `0e0d658f1f148d16db50c7f9efa6e78f6b8e6502`. Root's full I10 review section credits `cloud_operations_entra_certificate_private_file_status_is_scoped_and_redacted`: exit 0, 1 passed/0 failed, 47 filtered, 2.12s; 72.178s monitored invocation, Darwin arm64, Rust/Cargo 1.98.1. | Full function read and byte-equal at `544d`; it explicitly asserts zero token/directory requests and full durable snapshot equality. No crypto/pair validity, tenant acquisition, full crawl, apply or remote delivery is inferred. |

I10's historical command was:

```sh
env CARGO_TARGET_DIR="$PWD/.target-wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support,fuzzing --test cloud_directory cloud_operations_entra_certificate_private_file_status_is_scoped_and_redacted -- --exact --test-threads=1
```

Its root review records complete raw-log SHA-256
`8561bc484c37c18e7589924c5e4b84d18d86e5a77b9734dfdc9cfdd264468502`
and observation SHA-256
`11d09656ca2681b3600dd6260ce4852ed7c38401462b811b837f428b0e531a48`;
root rehashed the 0600 captures, reaped the owned Cargo group and released its
slot. Minimum recorded disk was 9.714 GiB. This audit read that published
receipt/report; it did not reopen private captures or repeat their verification.
The native linker warning and separately recorded broad CI/SCIM failures remain
historical; this report claims no all-green suite.

The pinned `local-wave28-task-closure-audit.md` I06 section explicitly retains
unavailable real-tenant token/paging/groups/mapping/removal/rotation evidence.
`ENT-04.md` also expressly requires a controlled tenant before deployment.
Neither I10's locally completed seven facets nor this source audit replaces
that named original residual. A stale coverage-inventory description is not
authority to discard the now-present certificate/resume implementation.

## One smallest next checkpoint: inputs before any run

No product hunk is proposed. Root should reserve one controlled-Entra checkpoint
only after all following inputs are supplied through private operator handling;
none was inspected or acquired here.

1. **Authority and target.** Explicit tenant-owner permission for this named
   nonproduction tenant/application and separately for synthetic user/mailbox,
   membership/disable/departure changes, app credential registration/retirement
   and cleanup. Graph connector authority is read-only; those tenant changes
   are performed by the authorized tenant operator, not an invented connector
   writer. An isolated disposable Platform instance with exact reviewed
   source/toolchain/features/binary hashes, encrypted current-format-3 store,
   issuer/owned listener and compatible offline backup is required. No released
   artifact, production tenant, existing customer's local identities or extra
   SaaS is implied.
2. **Consent and identifiers.** Configured tenant ID, client ID, matching cloud
   token/Graph origins and exact `.default` scope; consented application read
   permissions listed by pinned ENT-04: `User.Read.All`, `Group.Read.All`,
   `GroupMember.Read.All`; `Member.Read.Hidden` only for an intentionally selected
   hidden-membership group. No directory-write permission is required by this
   connector. These are the pinned guide's setup, not a new live Microsoft
   policy lookup or confirmation of consent. Tenant operator verifies actual
   granted scopes and network/TLS/proxy policy before release.
3. **Bounded, independently known data.** A small isolated tenant within the
   current collection/page/byte bounds and selected group ID-to-existing-local-
   group map. `/users` enumerates the whole tenant, not a configurable test-user
   filter: a production-sized tenant cannot be made safe merely by a username
   prefix. Include known object IDs and expected direct/transitive membership,
   synthetic mail/display-name change, one membership departure and an explicit
   disable or previously linked user departure. Include an actual nextLink
   witness; for users the requested top is 200, so 201 suitable synthetic users
   can provide a bounded multi-page cohort, subject to actual Graph behavior.
   Do not require all guest/hidden/SaaS profiles. If the chosen profile needs
   them, declare the data/permission rather than crediting an unrelated mock.
4. **Credential mode and rotation.** For the smallest first cohort choose one
   supported mode explicitly. Certificate mode already has the dedicated
   pair-rotation fixture: registered current and replacement public certs,
   valid matching RSA private keys and declared overlap/retirement authority;
   owner-only bounded private files are prepared outside tracked source. The
   existing secret mode instead needs current/replacement client secrets and
   overlap/retirement authority. Both supported modes remain separately named;
   one actual cohort does not prove the other. Never place credential values in
   argv, environment dumps, public records, shell tracing or this report. A
   mismatched pair must fail before token acquisition without secret fallback.
5. **Local review and evidence.** The same live operator owns each exact plan
   and apply; an agent needs `directory.read`/`directory.sync` on the exact
   `entra/<id>`, `user.write` for every affected username and `group.members`
   for every mapped group. Existing human/agent header policy, receipt secrecy,
   credential-once and PAM fallback remain unchanged. Private 0700 workspace,
   0600 session/credential/plan/log/archive files, capped output and a redacted
   public receipt need a separately reviewed collector. Page/count evidence
   must be real bounded provider/progress evidence, not inferred from mock
   names or from `connected=true`. No such collector is implemented here.

### Existing command and request surface, prospective only

Read the complete `src/cli.rs::cloud_directory` body and its enum/flags, plus
`src/cli/transport.rs::{new,session,authentication,call,call_with_review,
call_with_if_match,call_with_review_and_match}`. These use the existing private
session/agent files, encode the quoted revision header, and send the same exact
review ID in both accepted confirmation headers. The CLI automatically resumes
`snapshot_in_progress`; a plan file must be new and is written privately. Apply
first compares its saved file against the server plan. Neither progress nor an
edited local plan can become removal authority.

After root authorizes execution and a private session is established, the
existing printed operations can be instantiated with operator-reviewed paths:

```sh
riauth --server "$I06_ISSUER" --session-file "$I06_SESSION" --output-file "$I06_PRIVATE/list.json" directory entra list
riauth --server "$I06_ISSUER" --session-file "$I06_SESSION" --output-file "$I06_PRIVATE/plan-result.json" directory entra plan corp --out "$I06_PRIVATE/plan.json"
riauth --server "$I06_ISSUER" --session-file "$I06_SESSION" --if-revision "$I06_REVIEWED_REVISION" --output-file "$I06_PRIVATE/apply-result.json" directory entra apply --plan "$I06_PRIVATE/plan.json"
```

These variables denote a future private fixture, not measured actual inputs.
No command above ran. Review the entire private plan before apply. Add the
existing `--confirm-removals` only for that reviewed plan when its impact
requires confirmation; use a new filename/plan/revision after any conflict or
source change. Do not reuse a confirmation for a replacement plan or add a
blanket required header to optional routes. Use the reviewed agent revision/key
requirements if the separately selected actor is an agent. No CLI alias/new
operator interface is needed.

The equivalent existing bearer routes are `GET /api/entra-directories`,
`POST /api/entra-directories/corp/plan`,
`GET /api/entra-directory-plans/{exact-id}` and
`POST /api/entra-directory-plans/{exact-id}/apply`. Confirmation is the exact
`X-riAuth-Confirm-Cloud-Removals` or shared `X-riAuth-Confirm-Removals` ID;
duplicates/conflicts/malformed values refuse. Scope-first
`/api/cloud-directories/entra/corp/test-connection` and receipt-aware
`/verify-credential` can establish token plus first-page reachability only.
Fresh provider verification needs a new authorized invocation; a receipt replay
makes no provider call. A file-read/mtime observation is not a successful rotation.

Proposed order is acquire/complete plan without account changes, review/apply
the exact synthetic cohort, observe stable-link mailbox change, then review a
known membership/disable/departure change and exercise missing-confirmation
refusal before its authorized exact confirmation. Credential replacement then
needs fresh provider acquisition/complete planning under the new credential,
followed by tenant-owner retirement of the old credential and the explicitly
authorized negative observation. Private epoch/session/dependent observations
remain local proof; no Windows install or downstream delivery is inferred.
Stop dependent steps on the first unexpected result; no automatic retry,
permission expansion, weakened count/review check or source correction.

### Resource, privacy and rollback conditions for a future reservation

No current capacity/artifact/tenant/network measurement is claimed. Root must
review matching Platform artifacts and separately approve one finite fixture
before any product execution. Proposed prepared-input checkpoint limit is
20 minutes, with each source call retaining its fixed 30s budget and each plan
its fixed five-minute expiry; preparation and eventual-index settlement cannot
silently extend an existing plan. Initial capacity >=12 GiB, nominal 2s samples,
stop only the exact owned process group at 9 GiB, preserve the 8 GiB floor; no
cache deletion or build is part of this proposal. No slot was acquired/released
during this audit; A09 ARM `37101183416` owns the active validation/Cargo lane.

Use the isolated issuer/unused listener and owned PID/group, finite request and
log caps, finally stop/join/reap and listener/group absence evidence. Tenant
owner cleans up only explicitly authorized synthetic resources and retires
credentials under its own approved procedure. Preserve private audit/evidence;
public outcome is finite status/count/hash only, with no tokens, keys, private
paths, raw provider errors or personal records. No browser is needed; if later
separately necessary, only RiWork Cua.ai Driver after tool/state/permission review.

Refused local apply rolls back local identities/groups/credentials and leaves
the exact reviewed state unchanged, while fetch bookkeeping may change. A
post-commit recovery uses a compatible private pre-change backup under the
existing offline format-3 recovery procedure; restored storage remains CLOSED
until its existing recovery/config/credential gates are satisfied. Re-enable
does not resurrect revoked dependents. No live-store replacement, adoption
bypass, remote rollback or exactly-once delivery promise is introduced. Preserve
held Group, receipt-secret, optional/required headers, PAM, nonrenewed-60s and
paused-I/O/operator-settlement contracts.

## What was read and what was only hashed

Fully read at `544d`: `src/cloud_directory.rs` including its unit section;
`src/cloud_directory_types.rs` and `src/cloud_operations.rs`; all seven cloud assembly files (runtime, plan,
snapshot, reconcile, catalog, budget, operations and their protocol records);
complete `tests/contracts/cloud_mock.rs`; complete ENT-04, cloud-operations and
removal-safeguards guides. The contract mock is Workspace, not an Entra tenant.

Selected complete bodies were read in the larger files: cloud-owner/staging/
writer chain in management; cloud quota validation/private file/path handling in
config; the identity transition, store read/write/preview/PostgreSQL-write
chain; reconciliation functions listed above; API bearer plan/apply/control
handlers and removal-header parser; CLI functions listed above. Hashing those
whole larger files is provenance, not a full-file body-review assertion.

Fixture review read the complete local HTTP fixture/config/token/user/group/
transitive-member/credential generator bodies, `exercise`, `linked_pair`,
`Dependents`, and the named bodies in the matrix; also complete partial-page,
retry-budget, secret-file, repeated-page and reviewed denial/replay functions.
Unrelated remainder of the 5,539-line cloud test file was only searched/hashed.
Read complete relevant I06/I10 and executed-check sections of the published
closure/root/A03/D03/six-task reports, rather than claim every unrelated report
section was read. No private credential, artifact, tenant or raw capture read.

| Source witness at `544d` | Full-object SHA-256 |
| --- | --- |
| Cloud protocol, 62,889 bytes; blob `f912db10b2a9ca52f4bdfef6ae2ab5042545fd16` | `398de960390c70e0c427aa2fee0c140a2fb8edbdbaee817399ee441989de15bf` |
| Runtime assembly, 8,788 bytes | `904fd7eff03d6514a7260688baf99b7e5bc807b3dd20e8c410cc3d4fcf750417` |
| Plan assembly, 18,339 bytes | `3326251ffb076f23c719aab2a44c5024611f6a08871e60fc23249a0822a4bddd` |
| Reconcile assembly, 13,421 bytes | `4e7d080126d7eff6d77e4a93a015fa83e18cd52ff01556dc1e0b9d79c7ecb801` |
| Operational assembly, 11,665 bytes | `e35ed66bd4724858d0ea9b0fbeabd93737f10128608a54a6e0ef8d0bf2a52ab9` |
| Cloud fixture, 184,099 bytes; blob `796ad459470e95fbba562fce378ee5be1d1e8d35` | `56a59a810b5cbfd4e8cb2758e9d67424925a3008a0ebdfd38de0a1f6db291048` |
| ENT-04, 10,530 bytes | `ecf4275ecc47b02ab2bd493dcb2546d2f464d8b35617d0f173cb3e3287d4dd57` |
| Removal safeguards, 18,991 bytes | `a8dbccb98b8c0e4956c51a2a781e17cbe1f5c059c52eea489ac06a0250018da8` |
| I10 root review, 17,497 bytes; relevant I10 section read | `f8f63c3f1c1548cdcb98e12af998de98fce92b5eb10f32d5a0bf06434189b38b` |

Data-only comparisons extracted complete top-level `fn` spans through their
unindented closing brace. Twelve current spans byte-match executed `4df94`:
Entra links wrapper, transitive-members, counted-snapshot, certificate rotation,
cloud user writer, controller modes, partial pagination, Entra resume, tenant
separation, retry budget, shared secret-file reader and repeated-pages/totals.
This is body preservation, not compilation or proof of unchanged shared fixtures.
The dedicated I10 function matches its own `ef80983` source; it is absent at
`4df94`, as expected. Representative complete-span identities:

| Function | Bytes / SHA-256 |
| --- | --- |
| Counted-snapshot Entra test | 4,600 / `94b6723d87a7c3e0fb22902bd8575c0286121647bae8cc7e88bbd878a07db5b3` |
| Certificate rotation test | 2,681 / `4e5650a4097aec269db691b06e9d4bfeb1f25dee007d2cdf660bc29d3c4323f4` |
| Entra resume test | 3,604 / `74f0ae04bf5d35ef7e4239355ee70ff50a344f4165d53e9608e218f3b991b482` |
| I10 private-file test | 7,662 / `c6ffa1abbd0c1fa646e9fb9f54b017d033859d46e6dc200d962e185265b71edd` |

## Static checks and scope

Checks actually performed: read-only `pwd`, Git status/object/metadata reads,
`rg`/bounded complete-body reads, JSON parsing of the exact original row and
public historical handoff, SHA-256/source-span comparisons. An overlarge initial
source-data batch was truncated and could not be decoded; bounded reads replaced
it. One unquoted search path glob refused in zsh; the quoted search replaced it.
Neither attempted product execution or changed files.

Final checks: `python3 scripts/check-docs.py` exited 1 solely for the five existing
private build-directory names `target-wave29-source`, `target-wave28-scim`,
`target-wave28-portal`, `target-wave28` and `target-wave27`; no Markdown link
failure was reported. The report-only link/trailing-whitespace check passed.
After staging only this new report, `python3 scripts/check-repo-hygiene.py`
passed (1,023 tracked files), and `git diff --cached --check` passed. No checker,
directory, cache or existing document was changed to hide that baseline failure.
The source-span/data readers completed without importing archived/product code.

No tenant acquisition, compile, test, helper import, harness,
native/version command, browser/Driver, network/provider/service, other-worker
contact, status change or runtime slot action occurred. Earlier D01 source and
evidence, main/accepted worktrees and all existing files remain untouched.
