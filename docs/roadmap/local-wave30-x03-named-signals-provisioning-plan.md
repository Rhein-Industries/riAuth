# X03 named signals/provisioning directions: original-scope source audit

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original task
`db2de8e7-889c-4345-ae4b-3ef5a1c9e798`. Reservation
`wave30_X03_named_signals_provisioning_extension_audit`. Supporting existing
worktree `42bb51c6-c198-4adb-bd92-0a5222853231`, shell
`f1575610-c6f0-4dbf-9f33-d01ed057a194`. Date: 2026-10-03.

## Disposition and one next input

No new extension implementation is justified by the inspected demand record.
Keep X03 demand-deferred pending **one named peer requirement and direction**;
this is not a DONE recommendation under its named-requirement/security/real-peer
gate. Root decides the original row's disposition. Existing supported setup,
revocation and delivery behavior must remain complete without adding these
extensions to the Essentials completion gate.

The concrete current boundary is push-only SSF plus outbound SCIM, with
Workspace/Entra **cloud-to-local directory imports**. Poll delivery, the SSF
verification endpoint, SSF protocol subject-management endpoints and native
cloud-directory writes are separate excluded directions. Signed SET validation
and approved local subject bindings already work; their existence does not mean
the excluded protocol endpoints exist. No source defect in those supported
boundaries was demonstrated by this audit.

[ENT-07](../enterprise/ENT-07.md) names Apple Business Manager as **untested**
and lists real-transmitter inputs. It does not name a tenant, request an
additional method, or authorize registration/mutation. Workspace and Entra
are named supported read providers, not evidence of a requested outbound writer.
Example `tenant-a`, `corp`, `payroll`, `.example` URLs and zero/placeholder tenant
IDs are templates or synthetic fixtures, not customer demand. Do not choose an
extension simply because those names occur in source.

## Original export observation and fixed source

The exact original row was read from the local project export
`planning/current-tasks.json` under
`/Users/dominik/.local/share/riwork/orchestrators/projects/891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
This is an **export observation, not a live board query**. The snapshot is
244,354 bytes, SHA-256
`0b38ca3eb8810a4628436eb5083f6329de848f3f4ddaa972c850601ad45c0835`.
The row title is `[P3] X03 — Additional Shared Signals and provisioning directions`,
observed status `todo`, `worktree_id: null`, prerequisites P01, P02, I05, I06,
Q01. Its original assignment/status were not changed by this supporting audit.

Requested outcome, verbatim:

> Scope polling, stream verification, subject management, and outbound cloud-directory connectors separately.

Workstream goal, verbatim:

> These should not delay a complete Essentials experience. Scope them separately when a real integration requires them.

Completion gate, verbatim:

> Each extension solves a named requirement, has its own security model, and is tested against a real peer.

The row also requires applicable implementation/tests/docs/artifacts, truthful
checks and prerequisites, and forbids completion from documentation or a worker
report alone. Its historical scheduling text supplies no execution authority.
The present reservation permits this one new report only.

All new product/fixture reads used immutable published
`a6d361600a03713fc1b687f367e9db84efe43463`. Own clean entry HEAD was
`894e746ba03aa22926ee2f53cc6d614035555ac4`, branch
`roadmap/local-module-boundaries-wave27`. No checkout alignment or merge occurred.
Full pinned `CONTRIBUTING.md` and `SECURITY.md` were read. The report-only/static
reservation excludes their broad build/test campaign and documentation-index
edit. Earlier applicable-guidance inspection found no ancestor `AGENTS.md`;
the pinned tracked-name inventory likewise contains none.

The same export observes P01 `5a02a9e4-88a4-4b8b-a3de-8f66836d0560`,
P02 `8283d652-7211-43c5-a4a5-1e65bd124e9d`,
I05 `6e7724c5-3cdd-4a9c-abeb-bbd6e8eb00c9` and
I10 `2d472504-3063-4554-b546-674e31d06661` as DONE;
I06 `f9eb300d-ed58-4f86-80b4-cab14611af1e` as todo. These observations do not
reopen closed outcomes, certify a real tenant, or supersede root's live statuses.

## Separate direction matrix

| X03 direction | Exact supported boundary at the fixed source | Separate requirement/security input before a claim |
| --- | --- | --- |
| Polling | `src/ssf.rs::{metadata,normalize_method}` advertise/accept only push (`urn:ietf:rfc:8935`, plus the legacy push alias). `src/api.rs` exposes push/config/admin routes, no RFC 8936 poll endpoint. Local due-queue polling and cloud pagination are different operations. | Named transmitter/receiver and poll role/profile; queue acknowledgement, retention, retry, replay and owner-isolation contract. No automatic conversion of the current push lease into a poll queue. |
| Stream verification | `verify_set`, `jose::PublicJwks::verify_set/verify_claims`, and `assembly/ssf.rs::accept_set` authenticate SETs. Metadata/API do not advertise or implement the SSF stream-verification endpoint. `SUPPORTED` includes only account-disabled, session-revoked and credential-change. | Named peer's precise verification/challenge exchange, stream owner, expected event/profile and correlation/expiry/replay rules. A successfully validated SET is not proof of that exchange. |
| Subject management | `management/ssf_streams.rs::{resolve_subject_ids,bind_subjects,persist_subject_replacement,reconcile_manifest_stream}` implement approved **local** exact mappings to existing users. `PUT /api/ssf/admin/streams/{id}/subjects` replaces that mapping and cancels pending disclosure. | Named peer's add/remove-subject protocol contract and who may enroll which subject; distinguish remote registration from local authorization. Never infer an account link from email or let `ssf.configure` grant trust/binding authority. |
| Outbound cloud directory | `provisioning.rs` sends SCIM Users/Groups reads, conditional PATCH and POST through reviewed jobs. Workspace/Entra connectors acquire read tokens, GET cloud users/groups/members, then apply local reconciliation. No native Workspace Admin SDK/Graph user/group write adapter is present in this supported profile. | One named target and exact requested write/lifecycle/managed-field contract, provider permissions, identity ownership, removal floors and ambiguous-outcome settlement. First determine whether its actual SCIM endpoint meets the existing profile; do not turn read credentials into native cloud-write authority. |

The exclusions are explicit in [product contracts](product-contracts.md),
`docs/limitations.md`, the Shared Signals capability row and the X03 coverage
row. The demand audit searched the original export details and the scoped
ENT-03/04/07, product-contract, limitation, capability, coverage and remaining-task
documents. Only X03's export row names these additional directions; no selected
tenant or extension requirement was found in that inspected corpus. This is not
a claim about uninspected private communications.

## Complete source and assertion witnesses

Inbound verification bodies read in full: `ssf.rs::{metadata,normalize_method,
push_url,authorization_header,transmitter_issuer,audience,binding_subject,
verify_set,subject_of,apply_event}` and the `SubjectId` implementation;
`jose.rs::{verify_set,verify_set_signature,verify_claims}`;
`assembly/ssf.rs::accept_set`; API `ssf_stream_subjects` and `ssf_events` plus the
full SSF route block. Incoming bodies are bounded to 16 KiB. Pinned-key algorithm,
issuer/audience, `secevent+jwt`, forbidden header/claim, bounded `iat`/JTI,
subject-format/issuer and nested-subject rules precede action. Issuer/JTI replay
is committed with the action; duplicates acknowledge without another revocation.
Unknown/unlinked subjects acknowledge with redacted `ssf.ignored`, never create
users. A valid account-disable cannot disable the last enabled administrator.
Credential-change invalidates sessions/epoch; it does not replace a password.

Complete stream-writer bodies read: `admin_caller`, `config_caller`,
`StreamReceipt::{current,replay,save}`, `require_revision`, `write`,
`require_config_target`, `prepare_admin_delivery`, `resolve_subject_ids`,
`canonical_subjects`, `insert_admin_stream`, `persist_subject_replacement`,
`create_admin`, `create_receiver`, `bind_subjects`, `update_receiver`,
`reconcile_manifest_stream`, and `ssf.rs::{admin_allow,config_allow,owner_of}`.
Live authorization precedes receipt replay; scoped writes require current
revision, and result/receipt/audit commit atomically. Basic credentials cannot
create trust. A service token must be a live unbound service grant with
`ssf.configure`; receiver configuration cannot supply peer JWKS or local users.
Standard stream ownership and eight-per-owner/24-shared/32-total limits remain.
Trust/binding administration uses `ssf.manage`; manifest configuration cannot
convert a standard stream or mutate immutable administrator trust. Delivery
authorization requires encrypted storage and stays write-only. Changing bindings,
endpoint or authorization cancels queued disclosure, with the documented
already-admitted in-flight limitation retained.

Complete outbound bodies read: all `src/identity/signals.rs` records/enqueue
implementations; `ssf.rs::{claim_deliveries,begin_dispatch,finish_delivery}` and
`assembly/ssf.rs::deliver_once`. Shared transitions enqueue within their caller's
transaction, deduplicate intent and retain fixed JTI/transition time. Claim and
pre-send pin use a nonrenewed 60-second owner lease; a stale finish cannot
overwrite the newer attempt. Five-second HTTP timeout/no redirects and bounded
attempts do not create atomic network fencing. A process suspended after its
pin may later send after replacement: delivery is at least once and receiver JTI
deduplication remains necessary. Outbound credential-change currently emits
`update`, including enrollment/deletion, and extension credential types require
receiver agreement. Neither detail is silently redesigned here.

Complete relevant outbound-SCIM bodies read: `Target::{validate,bearer_fenced}`,
`provisioning_apply_confirmed`, `provisioning_step`, `fence_provisioning`,
`finish_provisioning`, `validate_remote_removals`, and `authorized_fenced`.
Target/actor/review/revision/managed-link/removal checks bind a durable job.
External-ID lookup never adopts by email; conditional ETag updates preserve
unmanaged fields; 204 PATCH is read back before advancing. Incomplete membership,
unexpected managed drift, missing active state and unresolved prior Create refuse
unsafe dispatch. OAuth refresh/write retry retain owned/live fences. A verified
late Create still records deactivation intent if local disable won. Started or
legacy attempts cannot be time-cleared into success; quiescence/remote settlement
and operator evidence are distinct from a local receipt or elapsed lease.

Full current fixture/assertion bodies read:

- `tests/ssf.rs`: `inbound_sets_disable_only_the_linked_account`,
  `ssf_subject_bindings_include_format_and_subject_issuer`,
  `ssf_set_profile_rejects_forbidden_claims_and_uses_bounded_iat`,
  `duplicate_set_acknowledges_without_reapplying_session_revocation`, and
  `outbound_configuration_and_inbound_trust_have_separate_authority`, with
  signer/stream/SET-construction helpers. They assert scoped visibility,
  tamper/profile refusal, cross-stream isolation, format/subject-issuer identity,
  replay without another epoch bump, and configure/manage/service-token separation.
- Entire `tests/ssf_management.rs`: HTTP create/update/delete exact receipts,
  changed-request/stale-revision/cross-owner refusal, one audit/revision,
  deleted-row replay and revoked-caller denial; administrator routes share them.
- Entire `tests/ssf_lease.rs`, including complete `prove`: plain/encrypted redb,
  legacy decode, expiry/replacement/stale finish, cancellation, fixed SET identity
  and one local POST. The initially truncated middle was reread in bounded spans.
- The complete 6,031-byte outbound-SCIM test in `tests/identity/policy.rs`:
  a second riAuth loopback peer, actor-bound plan, Users then Groups, preservation
  of a remote-owned field, reviewed departure disable/group-emptying and stale
  authority after agent revocation. This is a current definition, not a fresh pass.

Larger modules/test files and the PostgreSQL lease fixture were searched/hashed
as stated, not falsely credited as complete body reads. Full ENT-07 was read;
selected SCIM delivery/outcome/removal sections were read. No private credential,
raw SET, remote tenant or retained runtime capture was opened.

## Reused cloud/identity reads only after equality

The prior [I06 source audit](local-wave30-i06-real-entra-plan.md) is byte-equal to
its published copy at this pin. Before reuse, data-only whole-object comparisons
against `544d1340b80cd3e040dc13142cdcbc1d75fea4cb` confirmed equality for
`src/cloud_directory.rs`, cloud runtime/snapshot/reconcile/plan/operations assembly,
`src/cloud_operations.rs`, `src/cloud_directory_types.rs`, `src/config.rs`,
`src/reconciliation.rs`, `src/identity.rs`, identity persistence, `src/store.rs`,
`src/cli.rs`, `tests/cloud_directory.rs`, `tests/contracts/cloud_mock.rs`,
ENT-03/04 and removal safeguards. Thus the previously read complete provider,
OAuth/certificate, snapshot, local writer, schedule/control and focused fixture
bodies are reusable as source evidence. No whole larger file is newly claimed
read from its hash. G05's previously read migration/state/factor/recovery bodies,
guides and whole reference fixture likewise passed immutable equality first;
they do not supply an X03 real-peer extension result.

Workspace direct authorization is broker-optional, signed service-account
delegation with fixed Directory read-only scopes and pinned production endpoints;
the key is freshly read and no ambient proxy/redirect is allowed. Entra uses one
private shared-secret or matching certificate mode, fresh PS256 assertion and
tenant/Graph-cloud agreement. Complete counted same-origin pagination, exact
subject/owner binding, quotas/fingerprints, live authority, review/removal floors
and local transaction semantics remain. These outgoing token/GET requests are
not outbound identity creation/deletion. I06's real tenant acquisition/rotation
inputs remain separate. I10's seven operational facets and zero-provider-call
diagnostic PASS are not remote-write or connection evidence.

P01's unified reconciliation modes and P02's scoped schedules/jobs/retries/live
authority remain accepted responsibilities. Schedule source's `local_applied`
result does not assert remote delivery. No controller or factory redesign,
connector definition/admission change, Source Group materialization, permission
expansion, removal waiver, receipt-secret exposure or header/PAM change is proposed.

## Recorded execution and non-equivalence limits

The public local `planning/accepted-commits.json` was read selectively as DATA:
639,464 bytes, SHA-256
`b146a96488a646da28ed9513aaf1c28707c04e475f14c0e04d8e546fb51b1e5b`.
Full relevant review entries and actual command/result records were inspected,
not only report paths. No raw historical runtime log was rehashed here.

| Historical execution/source pin | Recorded result and its boundary |
| --- | --- |
| SSF management source `c811ee698e61b43de821529719af805ee72e08ff`; accepted run `2863e1fd1b77cf329a79cd21dd0398df5cbd3273` | `CARGO_BUILD_JOBS=2 cargo test --locked --features test-support --test ssf_management --test ssf -- --quiet`: existing SSF 20/20, management 1/1. Current whole `tests/ssf.rs` and `tests/ssf_management.rs` byte-match that source. Local keys/listeners, not ABM or an extension peer. |
| SSF lease source `aaaa7ea45636c4eec4f29825a6d43ed635bf1eea`; accepted `49fd94ffa55c1fa563a20807214344222aa7727f`, clarified by `c0196efb58df1c079b3fa9a7e2a064406ceb2dba` | Review records plain/encrypted redb lease 2 and diagnostics 2 passes; native PG16 two-worker held-receiver cases 2 passes with one POST/attempt. Current whole redb/PG lease fixture blobs match source. PG fixture body was hash-compared, not fully read or run here. Preserve the explicit paused-IO correction; no exactly-once or current-format-3 deployment inference. |
| SSF diagnostics run `a24f5607a368ae3e2b734344a5c794460a7a34cf` | Recorded locked/offline `--test ssf_delivery_diagnostics -- --test-threads=1 --color never`: redb 2/2, PG ignored; separate source PG page test 1/1. Nonfatal linker warning retained. Diagnostic GET is not verification, polling or delivery. |
| M07 source `3b4402aac3ce67018d8b1069db6d945d68cf69b4`, accepted compatibility correction `43ff7feb307562d2a0a109a858f17bec7560c17c` | Review records Platform manifest 7, existing management 1, Essentials rejection 1 passes. Also retains twelve pre-existing full Essentials library failures/strict diagnostics. No broad Essentials pass or extension acceptance follows. |
| Outbound SCIM run `b918977540bbd0eb6c1cebd55f8107eb53aefe95` | Locked exact `policy_tests::outbound_scim_plans_provision_groups_preserve_remote_attributes_disable_departures_and_stop_stale_jobs`: 1/1 after two initial failures at `24b08ea` from a fixed two-step assumption. Current body differs by three retained-executor/router setup lines; it is not byte-equal or newly executed here. |
| I05/Entra mock suite `4df94b61edf9161cac5217b00691e55fdb6ee6ed` | Accepted six-task/D05/I06 records `cargo test --locked --features test-support --test cloud_directory -- --quiet`, 36/36. Direct Workspace, counted/resumed Entra and credential/ownership fixtures are local mocks; not actual tenants or native cloud writes. |
| I10 source `ef80983e13fa5923ae5c2a29b520e415e1205240`, execution-report HEAD `0e0d658f1f148d16db50c7f9efa6e78f6b8e6502` | Accepted exact Entra private-file diagnostic: exit 0, 1 passed/0 failed, 47 filtered, 2.12 s. It asserts zero token/directory requests and durable snapshot equality. No provider acquisition or delivery is inferred. |

The old outbound-SCIM body is 5,804 bytes/SHA-256
`4b7ea4638a604b9e845831442a293c401ad28c8d49fdb850fef196922869bf54`;
current is 6,031 bytes/SHA-256
`d668442b795cf2e8c70040eea4a3cf607c9e29ab6d88e0ae5eeea023f10a7314`.
The data-only diff retains all assertions and adds the source router to release
settled permits; no current execution is invented. Even whole fixture equality
does not establish equal dependencies, feature flags, configuration, artifact or
runtime behavior. Matching version strings alone are insufficient. These are
source-built/local fixture records, not official release/container/workflow,
tenant or new extension observations. Historical jobs=2 commands are recorded,
not proposed as new builds. No full-suite/CI/shared-gate green claim is made.

## One exact requirement record before any implementation

Supply or locate one owner-approved record containing:

1. Named peer product/version/profile and authorized isolated tenant/environment;
   accountable owner; exactly one direction from polling, stream verification,
   protocol subject management or native outbound cloud writes. State the precise
   operation unavailable through existing push/local-binding/SCIM/import paths.
2. The peer's immutable/public protocol/config contract and endpoints, expected
   issuer/audience/subject format, registration/credential scope and rotation
   ownership. Private credential values remain outside source and this report.
3. A finite success/denial/replay/cross-subject/removal/expiry/failure and cleanup
   oracle for that operation. For remote writes, explicitly define managed fields,
   stable external identity, exact review/removal floors, retry/idempotency support
   and how an ambiguous accepted request is settled. No assumed exactly-once.
4. Explicit permission for the corresponding real-peer exercise and remote cleanup
   actions; bounded test identities and retention/redaction policy. Public evidence
   carries pins and finite outcomes, not SETs, tokens, private keys, sensitive
   identifiers or credentials. Real-peer authorization must be supplied separately
   from the user's request to finish local work.

That is the sole next slice proposed here: bind one demand record to the exact
current boundary before code ownership. If the peer only needs signed push and
approved mappings, use the existing supported surface and verify it separately;
do not fabricate an X03 extension. If it demonstrably needs an excluded direction,
root can reserve the smallest protocol-specific hunk and security model after
reading its contract. No new task, provider, universal feature list or production
write path is reserved by this plan. There is no justified literal implementation
hunk until the missing record selects the protocol role and operation.

Any later desktop work remains RiWork Cua.ai Driver MCP-only, with descriptions
and current state read first; absent setup/permissions must be reported without
changing providers. All runtime remains held. D01 memory owns the validation
lane; this audit acquired or released no slot. Prior G05/I06/D01 evidence and all
closed rows remain untouched. Preserve receipt-secret, route-specific optional/
required headers, PAM fallback, held Group, signal revocation, nonrenewed 60-second
admission, paused-IO and truthful at-least-once/settlement contracts.

## Pin identities and actual static checks

The following whole-object hashes are identities at fixed `a6d361600a03713fc1b687f367e9db84efe43463`,
not claims of full-file reads or new executions:

| Path | Bytes | SHA-256 |
| --- | ---: | --- |
| `src/ssf.rs` | 26410 | `7369acf5902587483c9d4f34dc1b5f27455b7d48cd042ce70ecb97127e937d11` |
| `src/assembly/ssf.rs` | 19865 | `231c8a52ed6312c8c10f30815fce9081383ea22ea93839f6207ba7635ae2bc5f` |
| `src/management/ssf_streams.rs` | 29955 | `b2457f4c167b57cfd3c8b1e89db600980661cec8066dfcbaa1c6cba59c09dbb0` |
| `src/identity/signals.rs` | 5015 | `c658aa600257eb816122654e30d70167830cc394446debc4309ab721848fa8f9` |
| `src/jose.rs` | 11997 | `107f85b6dd9335f8dde1184fe97135d8eb98f5bece1a700a174196b1ed9bf979` |
| `tests/ssf.rs` | 81777 | `1699765fe388a54e113d859723604e5b7f9ff0256802345d28fad9101ac38f6e` |
| `tests/ssf_management.rs` | 8566 | `ab3a0db2fbd15a8471cf101e55974637673c0e0cf4fc5f5902d33d0f18bd7501` |
| `tests/ssf_lease.rs` | 21389 | `de5743fb9ff1e3678298957482e2b060ea54c4382b9ca84e3dc1857ad5661f49` |
| `tests/ssf_lease_postgres.rs` | 29659 | `fc5e3306e7b98a6e7d2fcd57a936248cb98b89c43b32838138e452ac1dbb196a` |
| `src/provisioning.rs` | 120970 | `de84ce7ac733f7373e016e18d8cc2898922c0e1f0a48ab90f8f44494fa6fc260` |
| `src/cloud_directory.rs` | 62889 | `398de960390c70e0c427aa2fee0c140a2fb8edbdbaee817399ee441989de15bf` |
| `docs/enterprise/ENT-07.md` | 16398 | `f3a2f76105e4a9ef5334b827b8a94b7778ff809c9e8c8e65ef510f352aa025ee` |

Read-only checks actually performed: cwd/clean HEAD, bounded immutable Git/source
reads, export/ledger JSON parsing, hash and fixture/body comparisons, scoped
demand searches and literal diff. An initial guessed `docs/shared-signals.md`
read returned path-not-found; tracked names identified ENT-07. The old I06 report
is absent at its pre-report product pin, as expected; current published report
equals own retained copy. A ledger reader stopped on a non-dictionary check
entry after printing the SCIM result; a dictionary-filtered DATA reader replaced
it. Oversized initial outputs were truncated; critical complete bodies were
reread in bounded spans. None of these static-reader failures is a product or
peer runtime result.

Final static results actually obtained:

- Export/ledger observations, immutable source hashes/equalities and the exact
  outbound fixture diff: PASS as detailed above; no archived function was called.
- All 15 full commit pins resolve with `git cat-file -e`; three report-relative
  Markdown links exist. Report whitespace/newline and staged `git diff --check`:
  PASS.
- `python3 scripts/check-repo-hygiene.py`: EXIT 0, 1,025 staged tracked files.
- `python3 scripts/check-docs.py`: EXIT 1 solely for pre-existing root directories
  `target-wave29-source`, `target-wave28-scim`, `target-wave28-portal`,
  `target-wave28`, `target-wave27`; no Markdown-link failure. No checker or
  private target was changed or cleaned.
- Staged-name/unstaged/untracked scope and entry-HEAD assertions: PASS, only this
  new report; all prior files/history retained. No active executable non-sample
  commit hook was present. Exact report commit and clean final state accompany
  handoff; post-commit `git show --format= --check` verifies committed whitespace.

No source/test/helper/workflow/existing-doc write, alignment, merge, runtime,
Cargo/compiler/native product/tool/version, provider/HTTP/network/query/download/
dispatch, browser/Driver, service, managed shell/task/worker/worktree creation,
other-worker contact, private cleanup, main/push or status mutation occurred.
Root owns source review, integration, separately authorized runtime and statuses.
