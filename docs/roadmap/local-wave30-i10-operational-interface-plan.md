# I10 operational-interface audit and bounded proposal

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original I10 task
`2d472504-3063-4554-b546-674e31d06661`; existing worktree
`a1303b57-4a34-487e-9c63-a841f05b51a0`, branch
`roadmap/local-workflow-safety-wave27`. Audit date: 2026-10-02.

**One concrete local gap: configured outbound SCIM has no connection check
before delivery.** Its plan builds local desired resources without contacting
the target; its first configured peer read belongs to delivery and can proceed
to a write. Propose one scoped Core/API probe, with one focused test target.
No production implementation or runtime is authorized by this report.

## Original outcome, current state and source pin

The live row was read with the explicit project ID. It is `in_progress` and
assigned to this worktree. Its unchanged details SHA-256 is
`ed01a543d76a0509cda1b5a8441929f201b36b869c389c58198cd0b2fff76b58`.
Requested outcome:

> Include configuration validation, test connection, mappings, schedules, secret rotation, job history, and actionable errors.

Workstream goal: turn supported protocols into complete, tested integrations.
Completion gate:

> Every advertised integration has a working setup, lifecycle, and failure-handling path—not just an endpoint or protocol module.

Original proposed prerequisites are M03, P02 and P08. The old row's scheduling
instruction to leave it unassigned is superseded by the explicit current
assignment. No status mutation occurred. R05, W02 and W05 remain DONE.

All product references and line numbers below refer to immutable published
main **`88790deb62d32c84fa17dceb12cd93a727224e94`**, tree
`91008c3b893848354ebea6c578a212d31545d2eb`. Files were inspected with
`git show`, `git grep` and `git ls-tree`; nothing was imported from another
worktree. Own initial HEAD was `0cc3515eb54a3bcbef83e7fbe03a00ddbcea6cbc`,
clean. It was not merged, reset or aligned during this audit.

CONTRIBUTING.md and SECURITY.md were read at the fixed pin. No applicable
ancestor, repository, docs or roadmap AGENTS.md was found. This read-only
assignment overrides the contributor guide's general full-check campaign.

## Advertised connector inventory and intentional interfaces

The accepted connector-definition catalog has exactly four kinds:
`connector_definitions::Kind::{Ldap,Workspace,Entra,Scim}` and `ALL`
(`src/connector_definitions.rs:49-57`). This is the bounded connector inventory
mapped here, not a claim to exhaust every Platform protocol or integration.
`src/agent.rs:301,331,366-367,396-397` and `capability.rs:728-731` identify their
advertised capabilities and configuration predicates: LDAP import and outbound
SCIM are shared; Workspace and Entra require Platform. A capability's local
configuration/readiness does not measure peer reachability.

| Advertised integration | Existing setup/lifecycle/operator path | Evidence and remaining boundary |
| --- | --- | --- |
| LDAP import/password binding | Server directory configuration or reviewed stored definition; Core `directories`, `directory_plan`, `directory_apply_confirmed`; bearer API and both CLIs `directory list/plan/apply`; optional `ldap/<id>` controller. | Accepted disposable OpenLDAP import/STARTTLS/login/removal result and real-binary stored-definition/restart fixture. No Active Directory/customer-directory acceptance inferred. |
| Outbound SCIM | Server target configuration or reviewed stored definition; Core targets/plan/apply/jobs/stop/resolve and deactivation settlement; bearer API and both CLIs `provision`; optional `scim/<id>` controller. | Accepted loopback delivery, managed-field preservation, reviewed removal and stale-authority evidence. **A local plan is not a target connection test.** External SaaS setup and credential acceptance remain peer prerequisites. |
| Google Workspace import | Configured directory/stored definition; shared cloud Core plan/apply; bearer/server CLI/riauthctl list/plan/apply; browser and bearer operations/probe/credential/controller verification/schedule controls. | Accepted scoped/redacted operational and signed service-account fixture evidence. Requires authorized Workspace delegation/tenant for real-provider results. Optional broker remains optional. |
| Microsoft Entra import | Configured directory/stored definition; same cloud operation paths, with configured Graph mappings and secret or certificate credentials. | Accepted local token/paging/rotation/refusal definitions and scoped operations evidence. Requires an authorized tenant, approved permissions and operator-controlled rotation/removal fixtures for real-provider results. |

The API is an intentional operator interface. Neither a shared browser button
nor a dedicated CLI alias is required to invent a new writer. Protocol services
outside this four-kind catalog retain their own original rows and documented
interfaces: upstream source ceremonies, inbound SCIM/LDAP provider, SAML,
RADIUS, SSF, device/Windows/PAM, proxy, Vault, SMTP and alert routing are not
declared complete by this connector audit. Their named peer/device/deployment
prerequisites are not evidence of a missing directory-control implementation.

## Seven operational facets at the fixed pin

| Original facet | LDAP | Outbound SCIM | Workspace / Entra |
| --- | --- | --- | --- |
| Configuration validation | `Config::validate` (`config.rs:585-595`), `Directory::validate` (`directory.rs:96`) enforce bounded DNs/filters/attributes and LDAPS, mandatory STARTTLS or explicit literal loopback. Stored definitions use the same checks. | `config.rs:621-630`, `Target::validate` (`provisioning.rs:85`) enforce canonical HTTPS/loopback, explicit groups and exactly one static-token/OAuth mode. Plan refuses missing mapped local groups (`:911-919`). | `config.rs:597-619`; `cloud_operations` (`assembly/cloud_operations.rs:15`) uses structural validation and reports missing mapped local groups. |
| Test connection | `directory_plan` (`assembly/directory.rs:184,311`) advances an authenticated bounded source crawl through `Directory::service` (`directory.rs:201`). It persists drafts/plans, but does not apply account changes. This is an existing intentional operator test path. | **Missing configured-target pre-delivery probe.** `provisioning_plan` (`:895-1090`) scans only local users/links. First network read is `provisioning_step` (`:1448,1494`). | Core `cloud_test_connection` (`assembly/cloud_operations.rs:167`) obtains a token/first users response, with current scope checked before and after. Bearer and guarded browser routes exist; credential verification separately persists a redacted audited result. |
| Mappings | Operator config specifies immutable ID, username, display/email attributes, prefix and explicit local-group filters (`directory.rs:70-87`); directory list exposes selected local groups. Plans preview effects. | Config selects groups/export_groups; plan projects enabled non-admin users and managed group membership (`provisioning.rs:966-1029`). Managed remote attributes stay bounded by the reviewed plan. | Operations returns mapped groups, attributes, prefix and reconciliation mode; full plan/apply checks source-specific membership and ownership. |
| Schedules | Optional server-configured `ldap/<id>` controller. | Optional server-configured `scim/<id>` controller. | Optional `workspace/<id>` / `entra/<id>` controllers; existing cloud schedule API/browser controls update persisted cadence/enabled state. |
| Secret rotation | Replace the existing owner-only server password file; service bind rereads it each time (`directory.rs:201-215`). A new path/configuration uses normal operator loading. A successful plan tests the new bind; a readable file alone proves no remote acceptance. | Static bearer reread each call; OAuth private secrets reread before deciding whether a cached bearer is reusable (`provisioning.rs:124-152`). Changed content invalidates that local cache match. File replacement is operator-owned; no remote client needs the file or its path. | Server-side replacement, next token request rereads; `verify-credential` and `verify-controller` give saved sanitized point-in-time outcomes. No file timestamp/readability substitutes for provider acceptance. |
| Job history | Optional controller schedules/jobs are scoped API reads (`reconciliation.rs:920,935`). Manual plans retain their actor-bound ID/status and audit, rather than fabricating controller jobs. | `provisioning_jobs` (`:1306`) and both CLIs expose retained delivery progress/outcomes. Controller history is separate. Terminal records are compact and bounded, not an unlimited archive. | Operations includes up to ten sanitized recent reconciliation summaries, gated by sync authority; global scoped job/schedule reads remain available. |
| Actionable errors | Fixed `directory_unavailable` advises checking bind credentials/paged-results support and retrying a complete snapshot (`directory.rs:287`). [Connector incidents](../connector-incidents.md#ldap) covers private-file/log checks, TLS/outage and safe refusal. | Scoped detailed jobs expose attempt/item/error state; `provisioning_job_diagnostics` (`:1331`) gives redacted counts and fixed next actions. Stop/observation/dispatch-recovery/deactivation settlement are intentional operator actions, not silent retry or a delivery claim. | Fixed connection/validation errors, controller/job next actions, credential/controller checks and [cloud operations](../cloud-directory-operations.md) distinguish readable/ready/connected/local completion from remote delivery. |

Common schedules are implemented in `reconciliation.rs:250-358`: the four
controller scopes are explicit, cadence is 60–86,400 seconds, authority belongs
to the configured live agent and its bounded private credential. Reads are
`GET /api/reconciliation/schedules` and `/jobs` (`api.rs:372-375`). A configured
controller may not yet have a stored schedule/job; absence is not synthesized
success. LDAP/SCIM schedule configuration and controller credential replacement
remain intentional server-operator actions, not missing browser controls.

Stored connector setup is likewise deliberate: `connector_definitions.rs:135`
reuses validation without loading secrets, and `merge` (`:1404`) checks operator
credential pins/first-owner destination binding before loading the process
configuration. Configuration activation is per process at startup. Reviewed
desired-state/API/CLI paths do not grant live arbitrary file/network authority.
This audit does not reopen M07, M03, O03 or those file-secret boundaries.

## Exact missing behavior and smallest prospective reservation

An operator with `provisioner.sync` on `provisioner/payroll` can configure the
target and create a valid plan even if the target is unavailable or the private
credential will be rejected. `provisioning_plan` validates configuration/groups,
walks canonical users and prior links, and writes the reviewed plan. It never
calls `Target::http`, `Target::bearer` or a peer endpoint.

The configured remote-read path starts only after `provisioning_step` claims a
delivery job, then builds the resource URL/filter and calls `authorized_fenced`
(`provisioning.rs:1448-1505`). The enclosing delivery proceeds to create/update
logic when that lookup succeeds. Applying a plan solely to test connectivity
therefore authorizes actual delivery. An empty plan does not solve this: it has
no resource to read and can finish without contacting the target.

The existing route block (`api.rs:330-370`) exposes targets, plans, apply, jobs,
stop, observations and dispatch/deactivation recovery. Neither it nor the two
`ProvisionCommand` enums has a configured-target probe. Whole tracked
src/crates/docs/scripts/tests search found no outbound `test_connection`,
`connection_probe` or `/targets/{id}/test-connection` implementation.

`riauth scim ServiceProviderConfig` is not that missing path: `cli.rs:1608-1620`
builds `/scim/v2/...` on its selected riAuth issuer, and Remote resolves that
issuer/saved credential (`cli/transport.rs:81`). It does not resolve a configured
outbound target, its private server file, OAuth grant or target CA. The incident
runbook's `doctor`, target/job lists and health probes also do not contact it.
No printed configured-target connection procedure was found in the inspected
SCIM/OAuth/incident guides. This is an implementation gap, distinct from an
unmeasured SaaS peer, not a conclusion drawn from a missing generic GUI.

**One proposed slice only; root must reserve it before edits:**

| Prospective file/hunk | Bounded change |
| --- | --- |
| `src/provisioning.rs`, new Core method next to `provisioning_targets`/`provisioning_plan`, plus isolated private first-page validator next to HTTP helpers | `Core::provisioning_test_connection(token, target_id)`. Authorize exact `provisioner.sync` before target lookup or file access; recheck live caller before token/request/retry and after response. Resolve only configured Target URL/CA/static or supported OAuth credential. One logical authenticated `GET /Users?startIndex=1&count=1`; return fixed redacted connection status, check time and actionable failure. |
| `src/api.rs`, one registration in the existing provisioning-target block and one handler next to `provisioning_plan` | `POST /api/provisioning/targets/{id}/test-connection`; extract current bearer and delegate through `run_connector(ConnectorWork::target("scim", &id), ...)`, retaining shared lane/target admission/context/release. No independent credential/provisioning writer. |
| `tests/i10_scim_connection_probe.rs` (new) | One bounded local target covering authorization, configured authentication/rotation, response/refusal/redaction, no delivery effects and released capacity. No existing helper/test rewrite. |

Use the accepted `authorized_fenced` credential/401 path (`:2461`) and finite
HTTP/token retries/time/body bounds. Existing HTTP timeout is ten seconds with
redirect refusal (`:2142`), token body is bounded, and SCIM JSON is capped at
2 MiB (`:2718`). A first-page check must validate list/page shape without
requiring a complete crawl: normal `totalResults > returned rows` is not an
error. It must not use complete filtered-lookup validation to pretend all
accounts were read. A successful probe proves that configured authentication
and one Users read worked then; it proves neither Groups/filter/write support,
full mapping correctness, secret revocation elsewhere nor delivery completion.

No caller-supplied network destination, credential/path, remote mutation, raw
resource response, new schema/bucket, durable connection receipt, plan/job/link
progress or identity/removal writer is proposed. Existing OAuth acquisition can
update accepted cache/freshness metadata; document that allowed effect instead
of calling the whole operation storage-write-free. Keep raw peer bodies, tokens,
private paths, usernames and subject identifiers out of results/error messages.
Authorization refusal must remain a refusal even if a peer reply just arrived.
Do not change delivery retries, leases, managed fields or settlement semantics.

No CLI/browser/configuration/approval/source/assembly/process edit is needed to
make this an intentional API operator path. Root must coordinate the two shared
production hunks with their owners. All accepted M03/M07/P06/P08/O03/O06
permissions, review/receipt/removal/audit behavior, client-creation receipt-secret
contract, route-specific optional/required headers and PAM fallback stay intact.

## One proposed validation, not executed

After explicit source reservation and runtime/Cargo queue release only:

```sh
env CARGO_TARGET_DIR="$PWD/.target-wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support,fuzzing --test i10_scim_connection_probe -- --test-threads=1
```

The one target should exercise exact scope denial before secret/network access;
static and supported OAuth authentication, credential content replacement;
revocation while upstream is in flight; 401 retry bounds, redirect/unavailable/
malformed/oversized reply refusal; fixed redaction including adversarial secret
and identity-bearing bodies; and an authenticated one-page success. A request
spy must observe only GET at the SCIM peer (OAuth token POST is legitimate),
with no SCIM POST/PATCH/PUT/DELETE. Compare full snapshots while allowing only
the accepted OAuth metadata, proving no new plans/jobs/links/identity/credential
effects or review/removal bypass. Check released target admission on both
success and failure. No broad campaign, actual tenant or released-artifact
claim follows. Private jobs=1/incremental=0/debug=0 and disk floor remain required.

## Accepted historical evidence, credited at its actual scope

These records were read at the fixed pin; none was rerun or independently
recreated by this audit. Current test definitions are source coverage only.

| Accepted record | Evidence credited and limit |
| --- | --- |
| [LDAP import recipe](../recipes/ldap-import.md), lines 3,14 | Accepted ignored test passed 1/1 against disposable OpenLDAP 2.7.1/Darwin: STARTTLS, mapping/import, 205-entry reviewed departure, login/MFA and fail-closed source behavior. No AD/customer peer or fresh current-pin run. |
| [Outbound SCIM recipe](../recipes/platform-outbound-scim.md), line 14 | Exact identity `policy_tests::outbound_scim_plans_provision_groups_preserve_remote_attributes_disable_departures_and_stop_stale_jobs` passed at its recorded recipe revision. Two-router loopback peer; no external SaaS. |
| [M03 connector port](local-management-connector-status-port-report.md), lines 71–88; accepted `1df219b72f55bad25079e2bd642563d3bf189a20` | Three export mocks and one real-binary loopback stored-LDAP/restart/import fixture passed on that source. Export status allowlist and loaded-state evidence, not a live-directory probe/tenant claim. |
| [Cloud boundary report](local-wave27-module-boundaries-report.md), lines 175–188 | Seven focused cloud functions passed on the earlier boundary source, including two probe scope/revocation functions and credential preflight/write rechecks. Existing bodies retained; not a new run or every cloud case passing here. |
| [Wave28 original-row evidence](local-wave28-task-closure-audit.json), I10 accepted `e4ad254a317cc53980220f8817657303d654ab8e`, `de95a49fb6b197c7dfafdb4e5cfd3d1775bad3c8` | Reviewed controller token/readiness and cloud credential API/browser CSRF/replay/revocation/private-file/redaction checks. Real tenant lifecycle remained separate. The older held M07 and scheduling holds do not reopen now-accepted decisions. |
| Same audit, P08 `113177300342ea32fad7b24cf64a998bb66d8a35`, `2b76c8763e8485e4f49e47e2b02c91fdd9b20515` and its cumulative accepted slices | Historical focused delivery/ambiguous-outcome/managed-field/settlement evidence. At-least-once and unresolved remote Create remain explicit. No exactly-once remote proof inferred. |
| Same audit, P06 `7b9253117e51a0676988de5ea26fb43dc2c627e5`, `221b0585423d09504a611582e3a1e04773a7b92c` | Accepted paged ownership/resource behavior and malformed-member refusal. Preserve these contracts; no new inbound schema/vendor demand. |
| Current source `tests/scim_oauth.rs:1465,1548,2603,2697`; `tests/reconciliation_jobs.rs:16,143`; `tests/cloud_directory.rs:3716,3775,4106,4643,4776` | Definitions assert token/static rotation, refresh-file behavior, exact review/freshness, controller restart/budget, cloud probe/auth/credential/controller/schedule behavior. Existence is not a newly observed passing result. |

Seven listed accepted commit ancestors (two I10, two P08, two P06 and the M03
connector port) were verified reachable from fixed main. That is provenance,
not seven fresh behavior checks. The broader historical matrix remains in
[wave30 actionability](local-wave30-remaining-task-actionability.md).

## Immutable source/evidence blobs and audit checks

All blobs in this table resolve under fixed `88790deb`:

| Path | Git blob |
| --- | --- |
| `src/directory.rs` | `646127686a6a746185d6fcee1ab4ebcfb794764c` |
| `src/assembly/directory.rs` | `9f2dd8989e5e894923b568cb69c00a52da129576` |
| `src/provisioning.rs` | `e745095151194b23531516b540d2c9d5df94b435` |
| `src/provisioning/token_freshness.rs` | `565ca1882932e549fe19008e492d5590bff0fc03` |
| `src/reconciliation.rs` | `f15ef16a97091817418ce95b14f3d59583caadc8` |
| `src/connector_definitions.rs` | `debf30cb8711c8ab25afa900c87970fd963de04a` |
| `src/config.rs` | `0fc0b9a530440c66555eae9d9ee976d1c03549c3` |
| `src/api.rs` | `b2097b0aeeb5a5e12d6586628fe2d84a6dd9ed72` |
| `src/assembly/cloud_operations.rs` | `8461d8d2556454cc9f401de5c6158dcd644ff38c` |
| `src/cloud_operations.rs` | `79043b1b4b39b08c522a14ef9c005eb31b0d5727` |
| `src/cli.rs` | `581b8ee69ff73b1512fa583cc6be5dbc9f0745ee` |
| `src/cli/transport.rs` | `28a1d2238c67b9fcb5d5d30a6217c0c0d099485e` |
| `crates/riauthctl/src/directory.rs` | `a14f26c7eeabf106f7bc42b447f820f7149c6c12` |
| `crates/riauthctl/src/provision.rs` | `b6d216f324b25d4ef2106dbb3dea3e1c6341ff99` |
| `tests/scim_oauth.rs` | `eea5290b7c4cb49ec60c76059919a5318f05c1f1` |
| `tests/cloud_directory.rs` | `1756a3ce6115691160503c6e02fabc19dbc1188e` |
| `tests/ldap.rs` | `cd387ac1eb16619ecda612498c0d1da38b6953b7` |
| `tests/reconciliation_jobs.rs` | `aa1c3bdc70b232796dfb0c150179b8165384b85a` |

The initial exact source claim/prospective three-file reservation and one target
command were sent to the selected project orchestrator before committing this
report. Root owns any subsequent implementation reservation, integration,
publication and task disposition. Only this new report is written.

Read commands, original-row detail hash, source/blob resolution, ancestor and
scope comparisons were performed. Initial `riwork task --help` returned exit 2
because that subcommand does not accept the flag; the advertised read-only
`task list --project ... --json` then succeeded and located the exact live row.
No mutation resulted. `python3 scripts/check-docs.py` exited 0. In-memory
fixed-blob, local-only SCIM plan trace, report-only scope, unchanged own HEAD/
product and Markdown structure assertions passed. `git diff --check` exited 0;
the new report is also checked in the staged diff before its commit. No product
test, helper execution or source alignment is implied by these document/Git
checks.

**Recommendation: keep I10 in_progress pending root's probe-scope decision.**
Do not close it from this report alone. The seven facets have concrete existing
paths for the catalog; the one identified SCIM connection gap is local and
actionable. Authorized tenant/peer fixtures, deployment controller credentials,
multi-node configuration/authority coordination and applicable release evidence
remain separately unmeasured prerequisites. No provider call, private credential
inspection, build, test, service, Cargo, desktop, external message, new worker/
task/worktree, alignment merge, main/accepted edit, push or board/status mutation
was performed. Desktop preference remains RiWork Cua.ai Driver only; none was
needed for this audit.

## Approved scoped SCIM probe implementation — runtime held

This section appends implementation evidence to the preceding read-only audit;
the original audit text and its historical-evidence limits are retained. Project
`891e7443-8dac-4c1b-897f-9e53cb59c7ee`, original I10 task
`2d472504-3063-4554-b546-674e31d06661`, existing worktree
`a1303b57-4a34-487e-9c63-a841f05b51a0`, branch
`roadmap/local-workflow-safety-wave27`. Root approved ledger reservation
`wave30_I10_scim_connection_probe`; source implementation was authorized while
Cargo, test/runtime execution and provider calls remained held.

History-preserving merge `b025c6af868d96fd613c75f0538323995de1dc55` aligned this
branch with exactly reviewed main `88790deb62d32c84fa17dceb12cd93a727224e94`.
It completed without conflicts, reset or stale-file replacement and retained
the original report commit `8d32b60523b531b52d1d75a171a370b5042f53d6` and earlier
source/private evidence. Implementation commit
`683e81a5e5423af651570551f90d094b6747cfa9` contains exactly the five reserved
source/test/documentation files below; this report delta is a separate commit.

| Implementation file | Immutable Git blob |
| --- | --- |
| `src/provisioning.rs` | `e572dfb0462f36302e672951b735a634b45fddbf` |
| `src/api.rs` | `cd552144cfe23397f65fd97d3cbd3466805dd9cc` |
| `tests/i10_scim_connection_probe.rs` | `d2d718154b3cd3359a8d77ee45fb19567ddc60af` |
| `docs/api.md` | `a22fdc85e4b47f85d5a557934ca7ac416936ea94` |
| `docs/scim.md` | `0ecc8cbe7996f54703aa52fa99e15af775be44b6` |

### Exact public seam and authority ordering

The new public Core seam is
`Core::provisioning_test_connection(&self, token: &str, target_id: &str) -> Result<Value>`.
The thin API seam is authenticated empty-body
`POST /api/provisioning/targets/{id}/test-connection`. It uses the existing
`run_connector(ConnectorWork::target("scim", &id), ...)` admission path. The
method accepts a configured target identifier and bearer authority only; it
accepts no caller URL, path, credential or mapping. There is no new mutation
header gate, plan confirmation, idempotency receipt or connection receipt.
The API documentation gains one route row; the SCIM documentation gains one
adjacent scoped request/status paragraph.

`provisioner.sync` on exact resource `provisioner/{target_id}` is checked before
target lookup, validation, HTTP/CA setup or credential reads. The configured
Target, its existing HTTP client and `authorized_fenced` supply static-token
and supported OAuth authentication. The existing callback rechecks live caller
authority before token acquisition, GET sends and retry sends. A final live
check runs after the completed outcome, including configuration, token,
transport and malformed-body failures. An observed intermediate
`access_denied`/`invalid_token` is also propagated, so a refusal cannot become
a diagnostic success when authority later changes. In-flight revocation must
remain an authorization refusal regardless of the peer's returned status.
These properties are source traces and queued test assertions, not observed
runtime results here.

The SCIM request is fixed to GET of the configured `/Users` endpoint with
`startIndex=1`, `count=1`, and SCIM JSON acceptance. The isolated first-page
validator requires the ListResponse schema, an explicit Resources array of at
most one object with a nonempty bounded id, a consistent nonnegative
totalResults, and matching optional startIndex/itemsPerPage. It rejects error
envelopes and malformed page shapes. A valid first page with totalResults
greater than the returned one resource remains legitimate. The probe does
not follow more pages or claim Groups, filter semantics or remote writes.

Existing finite protections are reused: redirects disabled, ten-second HTTP
request timeout, two-MiB SCIM response cap, one SCIM 401 refresh/retry, and the
existing at-most-two token acquisition attempts for retryable failure. No
existing timeout, retry, delivery, review, lease or credential helper changes.

### Fixed output and explicit metadata exceptions

Success returns only `connected`, `checked_at`, `component`, `safety`, and
`next_action`. Failure adds only `error`. Status components are fixed
`configuration`, `authentication_or_users`, or `users_page`; the shared
authentication/request helper is not misrepresented as a narrower cause.
Safety is `no_scim_writes`. Remedy is `none`, `check_scim_configuration`, or
`check_scim_credential_and_users_access`. Diagnostic errors are fixed
`invalid_configuration` or `connection_failed`; authorization errors retain
the existing refusal behavior. No raw peer body, URL, private path, credential,
target id or identity-bearing field is echoed.

There is no new persisted bucket or probe receipt and no new plan, job, link,
identity, credential or removal mutation. Existing OAuth acquisition can
update its accepted non-secret `scim_oauth_cache/{id}` and
`scim_oauth_freshness/{id}` metadata. The API also uses existing connector
admission bookkeeping and release behavior. Thus `no_scim_writes` does not
claim that every local store operation is read-only. Credential file handling,
controller/admission semantics, Optional/Required headers, receipt/review/
removal authority, PAM fallback and the accepted writer contracts remain
unchanged.

### New focused target definitions, not executed

The one new target defines seven synchronous tests and one Tokio API test.
Its private loopback request spy, bounded sockets/channels and worker cleanup
are test-local; no service or peer was launched in this source-only phase.

| Test function | Concrete queued evidence |
| --- | --- |
| `exact_scope_precedes_configuration_and_private_credential_access` | Invalid, wrong-scope and read-only callers fail before configured/unknown target, secret or network lookup; durable snapshot unchanged. |
| `partial_and_empty_first_pages_static_rotation_leave_pending_delivery_untouched` | Legitimate partial/empty pages, private static-file replacement, GET-only spy, and whole snapshot preservation with genuine pending delivery state. |
| `both_oauth_grants_reread_rotated_private_material_without_delivery_effects` | Client-credentials and refresh-token grants, cached reuse, private secret replacement, unchanged refresh file, and exactly the two accepted metadata-key exceptions. |
| `revoked_inflight_users_success_or_failure_remains_an_authorization_refusal` | Paused GET replies 200/401/503 cannot escape revoked-caller refusal or trigger a retry after revocation. |
| `revoked_token_acquisition_never_sends_the_users_request` | Revocation during token acquisition refuses before any SCIM GET, with explicit accepted metadata exceptions. |
| `bounded_refusals_and_malformed_pages_are_fixed_redacted_failures` | Static 401 bound, redirect refusal, unavailable transport, non-JSON/oversized/malformed pages, invalid/private-file failures, fixed fields and adversarial secret/identity/path redaction. |
| `failed_token_acquisition_is_bounded_and_never_calls_scim` | Token 401/503 attempt bounds, no SCIM request on acquisition failure, and bounded reacquisition after SCIM 401. |
| `api_needs_no_mutation_headers_and_releases_admission_on_both_outcomes` | Exact API scope refusal, no new mutation header gate, stale optional headers, overlapping target admission refusal, and release after both successful and failed outcomes. |

These definitions have not been compiled or run. Static snapshot assertions
allow only the two named OAuth keys; the static-token redb API case requires
the entire snapshot unchanged after admission release. The spy separately
permits OAuth token POST while requiring all requests at the SCIM peer to be
GET. No observed provider, tenant or all-catalog success is inferred.

### Actual static checks and corrections

`rustfmt --edition 2024 --config skip_children=true src/provisioning.rs src/api.rs tests/i10_scim_connection_probe.rs`
completed with exit 0. The corresponding final
`rustfmt --edition 2024 --config skip_children=true --check src/provisioning.rs src/api.rs tests/i10_scim_connection_probe.rs`
also exited 0. This proves Rust parsing/formatting only, not type-checking.
`python3 scripts/check-docs.py` exited 0 with Markdown links/build-directory
layout checked. `git diff --check` and `git diff --cached --check` exited 0.

An in-memory source reconstruction removed exactly the new Core method and
first-page helper, then compared all remaining `src/provisioning.rs` bytes to
fixed `88790deb`. Removing exactly the route and handler reproduced its full
`src/api.rs` blob. Removing exactly one API row and the adjacent SCIM paragraph
reproduced both original document blobs. All other production files under
`src` matched the fixed pin. Source assertions checked the first and final
authorization ordering, preservation of intermediate scope/token refusal,
the single fixed GET builder, exact first-page bounds, absence of new writer/
audit/mutation-header calls, eight test definitions and the five-file scope.
All passed. The immutable source hash/static proof were sent to the explicit
project orchestrator before this separate report delta.

Source reading corrected an initial test assumption that the existing digest
was a 64-character hex string: it is unpadded base64url SHA-256, so the metadata
assertion now requires 43 characters. This was a static fixture correction,
not a failed or repeated runtime test. An initial zsh search used an unmatched
assembly glob and a separate search named a nonexistent background test file;
both read-only searches were corrected to existing literal paths. There was
no compile/test failure, no simulated passing result and no alignment conflict.

### Runtime queue and remaining disposition

The sole prospective command remains queued, never executed in this phase:

```sh
env CARGO_TARGET_DIR="$PWD/.target-wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support,fuzzing --test i10_scim_connection_probe -- --test-threads=1
```

Root source review and explicit runtime/Cargo release are still required.
Use only the existing private cache with the eight-GiB free-disk floor; no
other target or campaign is proposed. No Cargo, build, test, service, provider
call, browser/desktop operation, external message, new worker/task/worktree,
main/accepted edit, push or board/status mutation occurred. Internal handoff
used the explicit project orchestrator. The reviewed source commit and this
append-only evidence are independently reviewable.

I10 remains in_progress. The bounded probe closes one source gap provisionally
subject to focused runtime review; root still owns the original all-advertised
integration gate and final disposition. Full crawl/Groups/filter/write support,
actual external SaaS/LDAP/cloud peers, deployment/controller credentials,
multi-node operation and release evidence are outside this connection result.
Historical catalog evidence above remains historical. W02/W05/R05 stay closed.
