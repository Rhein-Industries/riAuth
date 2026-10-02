# D03 original-scope independent review

Date: 2026-10-02. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Original task `9961ab66-f582-4e70-a4eb-da62e51a9acf` remains **in_progress**,
assigned to primary worktree `ed9ac424-59f4-4520-905b-919aea3521eb`.
This report-only support used existing worktree
`42bb51c6-c198-4adb-bd92-0a5222853231`, branch
`roadmap/local-module-boundaries-wave27`, parent
`33d4b76fa0a036cd7701663810ee138eccd9d6a3`. No assignment/status changed.

**Recommendation: original-scope DONE candidate for root interpretation and
review.** No concrete remaining local configuration, printed-step, permission,
failure-check or API/CLI-equivalence defect was identified in the bounded
inventory below. No existing-document hunk or new input is proposed to fix a
local defect. This is not all-recipe replay, all-provider interoperability,
release certification or completion inferred from this report alone.

## Original wording and fixed evidence

The live row was reread using `riwork task list --project
891e7443-8dac-4c1b-897f-9e53cb59c7ee --json`. Its exact outcome is:
“Include reusable configuration, expected results, permissions, failure checks,
and API/CLI equivalents.” Its goal is “Document completed user and operator
tasks, not merely available settings.” Its gate is “A new user and a new
operator can independently complete the documented workflows.” The evidence
clause asks for relevant implementation/tests/docs/artifacts as applicable,
actual checks and gaps, and prohibits completion from docs or a worker report
alone. Proposed prerequisites are M02, I10 and Q04. The live row's old
scheduling hold is superseded by this explicit support assignment; primary
ownership and root's status authority are unchanged.

All source/document reads used fixed published
`9cefe7a56425bb73c17753e8766d92320b77da3b` through Git objects, without a
merge or stale-file replacement. Read the initial D03 report
`0dc0a504cca3e5030495865b88c2dd93957a1cc0`, applied recipe correction
`dd896380ba5588cc0ff3fd45cdda5aea4a4d8e13`, and published follow-up
`65251df0c27296da27c03c85c1afa567a32a316a` in
`docs/roadmap/local-wave30-d03-tested-recipe-plan.md`. Root's integration
review is `98c2937cb22b72c71c075b76620620a954993713`, published as
`docs/roadmap/local-wave30-ci-88790de-integration-review.md`.

The fixed forward-auth recipe equals applied blob
`bc7eb471b051016c8b888b983cea448a7e39e68a`: the already-published rate
paragraph distinguishes shared PostgreSQL counters from process-local redb,
effective agreed defaults/overrides, recorded-value startup diagnostics,
explicit stopped-node upgrade with both confirmations, and missing-row-only
adoption. It retains 429 versus two-second admission-queue 503 and does not
claim binary-fixture rate exhaustion or deployed multinode proof. No second
rate correction is needed or made.

## Recipe inventory and direct body review

`docs/README.md` indexes nine `docs/recipes` pages: LDAP import/provider,
inbound/outbound SCIM, relying-party/upstream OIDC, forward-auth, SAML
IdP/source. Workspace and Entra have reusable operator guides in
`docs/enterprise/ENT-03.md` and `ENT-04.md`; a different directory/name is not
missing recipe content. Other profiles listed as absent from `recipes` are
inventory limits, not an original-row requirement to invent a page for every
capability. SAML/PG selected results are bounded corroboration below; this
support's detailed configuration/request/assertion review targets LDAP,
SCIM, OIDC, proxy and cloud, rather than every SAML or PG fixture body.

The table records actual bodies read, not just test names or metadata. Paths
and line coordinates refer to the fixed pin. Their definitions are source
inspection; execution credit is separately recorded in the next section.

| Recipe / reusable configuration and permission | Request and assertion bodies inspected | API/CLI mapping and bound |
| --- | --- | --- |
| LDAP import, `docs/recipes/ldap-import.md` and `docs/ldap.md` operator references: `directories.staff`, STARTTLS/CA/private bind file, entryUUID/uid/cn/mail, description-filtered staff. Create local staff first. Agent directory.read/sync on directory/staff, user.write and group.members on group/staff. | `tests/ldap.rs:22–361` builds the directory and agent, plans/applies as the same actor, refuses admin takeover of the agent plan, checks password omission, password/TOTP/one-use recovery, stable-id rename, stale source, untrusted CA, failed search without disable, revoked actor, reviewed departure and 205-entry create page still requiring removal review. `scripts/test-ldap.sh` builds a disposable loopback slapd/cert/private-file fixture and selects the ignored test. | CLI directory list/plan/apply maps to GET directories, POST directories/{id}/plan, GET directory-plans/{id}, POST apply. CLI reads saved server plan and resumes snapshot_in_progress; exact-plan removal confirmation survives transport. Optional If-Match is not made a universal required header. This is not AD mapping or a fresh Essentials execution. |
| LDAP provider, `docs/recipes/platform-ldap-provider.md`: explicit scopes, client LDAP settings/base/search groups, durable members, LDAPS/STARTTLS listener, exact allowed peers/certs. Live human admin creates service agent; ldap.search is scoped to client/ldap. | `tests/ldap_provider_peer.rs:122–156,179–end` builds public client/agent/listeners and invokes real ldapsearch with -x, STARTTLS -ZZ where appropriate, private-token -y, critical one-entry paging and hard CA verification. Bodies require two selected users, outsider/admin exclusion, wrong-token exit 49, crossed transport/unrelated CA exit 1, disable filtering and revoked-agent refusal. Read `scripts/test-ldap-provider.sh` selection. | Client/group/user/agent configuration has API/server-CLI counterparts with client/agent revision/key inputs and first-only private credential handling. Listener setup is configuration plus serve; search/bind is the application's LDAP protocol, not an invented HTTP/CLI search counterpart. Recipe distinguishes the separate ldap3 MFA test from the recorded third-party peer run. |
| Inbound SCIM, `docs/recipes/platform-inbound-scim.md`, `docs/scim.md` counterpart: 600-second agent with user/group read/write and group.members on *. JSON User includes schemas/userName/externalId/displayName/active/email; create uses idempotency key. | Complete `tests/identity/http.rs:245–399`: unauthenticated GET 401/scim+json; two identical POSTs/key return 201, ETag/Location, equal JSON without password. Subsequent Core calls require owned records, atomic rejection of active+unsupported roles patch, reciprocal group membership, unknown-member rollback, no admin adoption and deprovisioned-session refusal. | `src/cli.rs:1608–1622` maps resource/id/method/input/filter/pagination to /scim/v2, refuses global revision for resource writes and passes --if-version as ETag. HTTP create coverage is distinct from later Core PATCH/DELETE assertions; no named SCIM vendor, bulk/enterprise extension or new HTTP-status coverage inferred. |
| Outbound SCIM, `docs/recipes/platform-outbound-scim.md`, `docs/scim.md` target/CLI counterpart: private target token, staff selection, export_groups, agent provisioner.read/sync on provisioner/directory. | Complete `tests/identity/policy.rs:600–794` helper/test: second loopback riAuth router; retained unserved source router holds executor; eight-step bounded helper reads job state. Actor-bound apply and plan redaction; remote user/Group delivery; managed displayName changes preserve remote-owned name.givenName; exact reviewed departure disables/removes membership; revoked plan authority makes work stale. | CLI targets/plan/apply/jobs maps to provisioning target/plan/job routes; saved-plan checks and exact removal header remain. Apply creates durable work, not confirmed delivery. Those bodies do not prove every retry/ambiguity/deactivation/settlement mode or named SaaS delivery. Retained executor does not strengthen the nonrenewed lease contract. |
| OIDC RP, `docs/recipes/oidc-relying-party.md`: public rp, exact callback, openid/profile/offline_access, S256, backchannel_logout_uri. Administrator or permitted client.write configures client; end user approves. | Complete `tests/browser.rs:32–309`: loopback axum /start/callback/logout, random state/nonce/verifier, actual code exchange without client secret; RS256 JWKS issuer/audience/nonce checks; actual riauth request approve --yes --remember process; signed logout event/JTI/sid and absent nonce, revoked userinfo. Direct Core initialization/client setup/logout are explicitly distinguished. | Printed revision/key/client settings maps to POST clients; login/authorization inspection+decision/logout/userinfo have API/CLI equivalents. This fixture uses real selected Chrome plus terminal approval, not a confidential app, discovered provider, successful pre-logout userinfo or a customer RP. |
| Upstream OIDC, `docs/recipes/upstream-oidc.md`: source input, pinned JWKS, client/scopes/callback and ClientSecretBasic. Source.write plus user.write for auto-provision; administrator used. | Complete `tests/identity/sources.rs:4–74` and `tests/identity.rs:132–322` Upstream helper: token POST checks Basic/grant/exact redirect/S256 verifier; simulated authorize/code and riAuth-generated upstream signing key. Pending→review→one-use approval, same-email nonlinking/stable subject, trusted MFA, seven invalid claim overlays, source disable/session refusal with local Alice unaffected. | Read actual CLI source put/start/finish mapping to POST sources, sources/{id}/start and source-login/finish; private transaction file/issuer check and explicit --yes stay intact. No actual upstream /authorize endpoint, browser-cookie-bound source journey, vendor tenant or other client-auth mode is credited. |
| Forward-auth, `docs/recipes/platform-forward-auth.md`, both deploy templates: exact callback/public proxy client, trusted exact IPs, nginx auth_request, Traefik internal address/static aliasHeadersStrategy delete and trustForwardHeader:false. Client.write configuration and end-user-only approval; host deployment is infrastructure ownership. | `tests/outpost.rs:53–end` renders nginx template and asserts redirects/binding, binary approval, trusted identity/filtered cookies, missing-provenance/cross-site refusal, explicit API intent, logout and next-request revocation. Builtin-only branches are excluded from nginx credit. `tests/outpost_traefik.rs:757–end` renders both routing templates, asserts background 401, forged URL/header rejection/filtering, cross-origin 403, WebSocket origins/echo, hidden issuer auth endpoints and revoked parent. | Client writes and session approval/revoke have API/CLI equivalents; forwardAuth is protocol/template configuration, not a nonexistent CLI command. Traefik approval is Core::authorize, unlike nginx's CLI. Binary result does not prove every separately filtered negative fixture, deployed TLS/topology or socket closure after Traefik logout. |
| Workspace/Entra, ENT-03/04: actual TOML, owner-only credential modes, Workspace numeric-client-ID domain delegation/dedicated subject/read-only scopes; Entra tenant/cloud-bound Graph credentials/read-only admin-consented permissions. Correct resource is workspace/corp or entra/corp, with user.write/group.members, local groups created first. | `tests/cloud_directory.rs:376–493,754–793,880–1261,1482–1544,3031–3152`: fake-peer token request/signature/claims; configure/exercise with wrong LDAP scope and foreign actor refusal, redaction, replay without new fetch, stable identity/membership/removal review, suspension revocation; direct key reload/short-lifetime refusal; Graph transitive page failure preserves membership; failed token/budget/private-file checks. | CLI directory workspace/entra list/plan/apply and exact API collections/plans agree with `src/cli.rs:3047–3137`. Five-page progress/resume, actor/revision/config binding and removal review documented. Fake-peer fixture bodies support local contracts; numeric-ID consent/delegation, production TLS and live tenant timing remain named deployment inputs. |

Read relevant `src/cli.rs` branches at 1608–1622,1709–1800,1854–1879 and
3047–3137, plus `src/cli/transport.rs:209–233`: optional revision/key headers,
resource ETag, bearer/run envelope and both exact-plan removal headers stay
route-specific. SCIM PATCH is not a global revision mutation; connector apply
is not a client-issuance secret replay. Existing receipt-secret, PAM fallback
and live authority requirements remain unchanged.

Whole-file Git comparisons confirmed **16 explicitly inspected fixture,
template, CLI and harness paths** identical between fixed `9cefe7a` and
executed `88790deb62d32c84fa17dceb12cd93a727224e94`: tests/ldap.rs,
scripts/test-ldap.sh, tests/browser.rs, tests/portal_browser.rs,
tests/outpost.rs, tests/outpost_traefik.rs, both deploy templates,
tests/identity/{http,policy,sources}.rs, tests/identity.rs,
tests/cloud_directory.rs, tests/ldap_provider_peer.rs, src/cli.rs and
src/cli/transport.rs. This binds current selected fixture review to that job;
it does not establish execution of unselected tests or equivalence of an
entire historical cloud suite to current production.

## Recorded executions, failures and skips

The specified raw integration log was actually rehashed and its checkout,
command/result blocks inspected locally; no network lookup or replay occurred.
`/tmp/riauth-wave30-ci-37006213470-110834995871.log`: **265774 bytes**,
SHA-256 `0d327b38cb4999bedc20df3f739eafac161baa6d571aa3a81a4d139189c67859`.
Checkout is exact `88790deb62d32c84fa17dceb12cd93a727224e94`, run
`37006213470`, successful selected integration job `110834995871`, default
Platform build. Root's earlier 98c review observed the separate check still
running; latest root follow-up reports **full check FAIL, 11 passed/1 failed,
7.06 seconds**. That later failure is retained as root-supplied evidence; this
support did not receive/inspect a separate raw check log or infer its failed
test identity. No whole-run/all-green claim follows from integration success.

| Actual selected command/stage in that integration log | Returned result / precise credit |
| --- | --- |
| scripts/test-ldap.sh → cargo test --locked --test ldap -- --ignored --nocapture | Named LDAP import/password/MFA/fail-closed body above: 1 pass, 8.15s. |
| RIAUTH_TEST_BROWSER=<selected Chrome> cargo test --test browser --locked -- --ignored | Named public-RP callback/signed backchannel logout: 1 pass, 11.83s. |
| Same browser selection, --test portal_browser --locked -- --ignored | 1 pass, 10.40s. Reviewed setup/client policy and assertions at tests/portal_browser.rs:122–163,270–310,388–442,470–569 include selected catalogue, favorites, group removal, plain-text rendering, sign-out and account isolation. Terminal approval differs from independent password browser. |
| RIAUTH_TEST_NGINX=<nginx> plus selected Chrome, cargo test --test outpost --locked -- --ignored | Named nginx fixture: 1 pass, 1 filtered, 17.77s. Builtin-only assertions are not this result. |
| RIAUTH_TEST_TRAEFIK=<selected binary> cargo test --test outpost_traefik --locked -- --ignored | Named real Traefik: 1 pass, 13 filtered, 3.15s. Recorded v3.7.13 download checksum is 52cd039a34258dd61c617a95d69252bc6bcae27c520f338186c31c7fef8f6394; this worker did not download it. |
| Three locked ignored identity XMLsec filters | SAML sign/verify/decrypt, source responses, logout: one pass each, 177 filtered each, respectively 7.07/5.81/7.09s. Not three full SAML suites or independent named IdP/SP installs. |
| Default scripts/test-postgres.sh; shared-contract script; selected Q05 schedule | 2 passes/17.33s; 91 passes/90 filtered/211.58s; 1 pass/1 filtered/14.15s. Not 91 recipe journeys or every PG/HA target. |
| Real-browser setup; portal authenticator Playwright stage | 9 passes/53.5s; 22 passes and 2 skips/6.5min. Firefox/WebKit invitation-passkey skips remain; virtual/shim factors do not prove physical keys/phones. |

Historical evidence was separately inspected, without relabeling its pin or
failed job as current success:

- Rehashed old integration raw log run `36950097067`, job `110661000640`,
  checkout `2d05c00deed3a6dafcd458a5f1a1a9beb17d0dd8`, SHA-256
  `458a30895b5987c0bebe2fb7368a1a32969fa0d41ce90acbfa56f43299ccd186`.
  It records previous LDAP/Rust RP/nginx/Traefik passes, not new runs here.
- Rehashed failed-step raw log run `36951643190`, job `110666025654`,
  256279 bytes, SHA-256
  `5a94f6f07d83ae7f4565d8fb032c3a629f31b44cff99ac84e67f7c0078d376f8`.
  Read exact success lines for inbound SCIM and upstream OIDC and failure for
  outbound SCIM at then policy.rs:618: identity **171 passed/7 failed/3 ignored,
  140.21s**. This artifact lacks checkout; no exact SHA is invented for its
  positives and no newer Linux outbound pass is inferred from selected XMLsec.
- Accepted identity fixture correction/report at fixed pin identifies baseline
  `d31778f066d3bc79f94829cf6295e1eb8df2a84f`, correction
  `893be4fc0f80bd9da86e98c04a888ca4e7c06601`. Actual historical local command
  `cargo test --locked --features test-support,fuzzing --test identity
  policy_tests::outbound_scim_plans_provision_groups_preserve_remote_attributes_disable_departures_and_stop_stale_jobs
  -- --exact --test-threads=1` passed 1/180 filtered/2.72s. Retained source
  executor and unchanged eight-step helper address that fixture's pending
  admission release; old failure and compact-unwind warning remain recorded.
- Provider recipe records native OpenLDAP ldapsearch 2.7.1 dual-transport
  successful pages and refusal/disable/revoke results at
  `fcba8a5e81037edb365c02c291b512576817e920`; script selects locked ignored
  ldap_provider_peer. Body and recorded argv/results were read; no newly
  recovered native raw log or provider execution is claimed.
- Accepted six-task review/root disposition at fixed pin records cloud suite
  runs at `03606ed7cd42ea3f716e3f8dcbebf001da9c3470`: default 33/33 and
  test-support 35/35; later test-support 36/36 at
  `4df94b61edf9161cac5217b00691e55fdb6ee6ed`. Read I05's exact grant/request,
  rotation and short-token evidence, not just suite counts. These are historical
  local fake-peer results, not 36 recipes or any live tenant execution.

Prior TOTP rollover/diagnostic expiry failures and two bounded final feature-mode
passes remain as recorded in the accepted D01/W02 evidence; they do not fix or
identify the latest full-check failure. Earlier manual passkey cancellation,
source-finish/tenant gaps and canceled accessibility scope remain unchanged.

## Original independent-journey gate and remaining inputs

Independent basic D01 evidence is relevant common setup support, not a blanket
pass for each recipe. Read complete independent Sol report
`6837b745576552b0917b3bd1f1424515621a0a85:docs/roadmap/local-wave30-d01-user-browser-review.md`
and this lane's accepted operator report
`9cde81dd93ff171fa54194b2ed514142451484d9:docs/roadmap/local-wave30-d01-walkthrough-report.md`.
The separate worker used a fresh c01-source Essentials lab, printed interactive
hidden-password init/serve/ready and actual password sign-in/security
navigation/sign-out/relogin; this lane authored guide corrections and executed
27 CLI/probe commands plus two clean server lifecycles for init/login/client
creation/discovery/identity, backup/offline closed restore, original restart,
groups/claims preview/audit. That operator run is not independent of its guide
author. Their exact hashes/provenance/skips remain in those reports; neither
is claimed to use a binary built at 9cef or an official released artifact.

For D03, selected CI independently executes reusable fixture/harness/template
configuration and the public-RP terminal/browser journey; this review checks
the expected outputs, refusals, permissions and operator counterparts against
the bodies. That is additional execution evidence beyond the initial D03
report's then-partial D01 record. The original outcome is supported by the
published recipes and accepted code/results, including the actual bounded
journeys above; a newly authored report or zero graph count is not its proof.
There is no original named-provider list, universal new-run requirement or
requirement to recruit an inexperienced human/every-host study. Not every
printed administrative counterpart or optional continuation was executed.
The report does not equate API/Core fixture calls with those CLI executions.

The **separately planned confidential D01 browser workflow in worktree 7** is
not duplicated or credited. Current public RP/terminal/proxy tests do not
complete that chosen confidential-app password/consent journey. Likewise,
root-owned I10 production probe
`683e81a5e5423af651570551f90d094b6747cfa9` is unrun and outside this review's
implementation scope; no new probe result or acceptance is inferred.

Named deployment inputs remain real directory mappings and service credentials,
application callbacks/upstream issuer keys, SCIM product credentials/settlement,
proxy host/TLS/trust topology, Workspace delegation/numeric client ID and
Entra tenant consent/cloud credentials. They are explicit recipe inputs and
deployment validation limits, not an invented reason to hold every local
recipe or mandate all providers. The retained **nonrenewed 60-second connector
admission lease** does not guarantee exclusion if admitted IO pauses beyond
expiry. At-least-once/ambiguous remote effects still need truthful inspection,
quiescence/settlement before operator recovery; no exactly-once or paused-IO
proof is claimed. Token minimum-lifetime and LDAP certificate reload bounds
do not strengthen that lease. Held Group representation/materialization,
receipt-secret/direct-first-only distinctions, route headers, PAM fallback,
removal review/revision/authority/audit protections stay intact.

Root can consider original-scope closure after reviewing this mapping and the
actual accepted executions. This candidate does not close I10/Q04 deployment
inputs, the chosen D01 confidential journey, D05 category acceptance, physical
devices, universal release/tenant compatibility or full CI. The primary lane
keeps its ownership; no other worker was contacted.

## Checks actually performed by this support

Read live original row; fixed CONTRIBUTING/SECURITY and ancestor guidance
(no AGENTS file found); initial/applied/published D03 reports and root review;
indexed recipe/operator configuration and the specific request/assertion/CLI
bodies above. Performed Git object/whole-file comparison and three existing raw
log SHA/result checks. No build, test, binary, helper, service, desktop/browser,
benchmark, network service or tenant ran. No historical pass was replayed.
Desktop preference remains **RiWork Cua.ai Driver only**, descriptions/state
before any separately authorized input; no desktop was needed here.

Read-command discovery had two local tooling failures: `riwork tasks --help`
returned 2, and parsing task-list output without --json failed; the explicit
task list --project --json read then succeeded. An initially guessed upstream
test name was absent; its actual named body and helper were subsequently read.
These are read/discovery corrections, not product failures or substituted
runtime sequences. New-report static Markdown/path/reference validation,
repository docs checker result, exact staged scope/whitespace and clean commit
state are recorded below after validation. Only this new report is written;
no source/helper/recipe edit, merge/reset/main/push/status or new
worker/task/worktree/managed shell occurred.

Static validation results:

- New-report Markdown/reference/path and reserved-file scope checks passed;
  immutable Git objects and declared pinned source paths resolve.
- `python3 scripts/check-docs.py` exited **1**, with no missing Markdown links:
  only pre-existing `target-wave29-source`, `target-wave28-scim`,
  `target-wave28-portal`, `target-wave28`, and `target-wave27` directory-layout
  findings. Those retained artifacts were not deleted, moved or reclassified,
  and the global checker was not altered. This is not a new recipe/link defect.
- `git diff --check` and staged scope/whitespace passed. The commit contains
  only this new report; the own branch is clean after commit. No main or
  accepted branch, other lane file or board row was changed.
