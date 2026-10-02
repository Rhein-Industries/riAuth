# D03 tested integration recipes — bounded audit and proposal

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; existing D03 task
`9961ab66-f582-4e70-a4eb-da62e51a9acf`; worktree
`ed9ac424-59f4-4520-905b-919aea3521eb`, branch
`roadmap/local-revisions-coordination-wave27`. Audit date: 2026-10-02.

Read-only source pin: **`88790deb62d32c84fa17dceb12cd93a727224e94`**.
Initial audit parent: `eb1412aa6799ab959b90bdf71f29c983e15b67aa`.
Initial source and existing-document reads used immutable Git objects at the
published pin, not a moving worktree. No merge or source change was needed.
Initial audit commit `0dc0a504cca3e5030495865b88c2dd93957a1cc0` reserved and
added only this report. The separately approved paragraph correction and
report follow-up are mapped below.

## Original outcome and disposition

The live original row was reread with `riwork task list --project
891e7443-8dac-4c1b-897f-9e53cb59c7ee --json`: D03 is **in_progress**, assigned
to this worktree. Its outcome is reusable configuration, expected results,
permissions, failure checks, and API/CLI equivalents. Its workstream goal is
completed user/operator tasks, and its gate requires a new user and a new
operator independently completing the documented workflows. Recorded
prerequisites are M02, I10 and Q04. The latest assignment authorizes this audit;
the original row's historical scheduling text does not supersede it.

The existing recipes and accepted execution evidence satisfy substantial
parts of that outcome. They do not document completion of the independent
journey gate. Recommend retaining **in_progress**. One local documentation
correction is proposed below; fixing it alone would not establish D03 DONE.
S04, O03 and O06 remain DONE; D05 remains in progress. I10's separate source
facet audit remains root-owned.

## Published recipe coverage and actual operations

The recipe index is [docs/README.md](../README.md), lines 61–78, with cloud
guides under its directory/lifecycle section. There are nine files in
`docs/recipes`: LDAP import/provider, inbound/outbound SCIM, OIDC relying
party/upstream, forward-auth, SAML IdP/source. The last two are inventory
context here, not additional execution claims. Workspace and Entra already
have operator guides; absence of a dedicated `recipes` filename does not mean
their accepted implementations or reusable configuration are missing.

### LDAP import and LDAP provider

[LDAP import](../recipes/ldap-import.md) supplies the fixture directory mapping:
`directories.staff`, STARTTLS, private password file and CA, `entryUUID`, `uid`,
`cn`, `mail`, and `staff = (description=staff)`. It distinguishes this from
Active Directory mappings. Local `staff` must exist first. The agent has
`directory.read`/`directory.sync` on `directory/staff`, `user.write` on `*`,
and `group.members` on `group/staff`.

The actual [tests/ldap.rs](../../tests/ldap.rs) named body, lines 38–350,
checks actor-bound apply, secret omission, id-stable rename, password/MFA
login, untrusted-CA failure, changed-directory conflict, failure without
disabling accounts, explicit removal confirmation, and the 205-entry plan
whose previously missing user still requires review. The
[harness](../../scripts/test-ldap.sh) creates disposable loopback OpenLDAP and
selects `cargo test --locked --test ldap -- --ignored --nocapture`.
The printed CLI sequence is:

```sh
riauth directory list
riauth directory plan staff --out deployment-private/ldap-plan.json
riauth --if-revision "$(jq -r .revision deployment-private/ldap-plan.json)" directory apply --plan deployment-private/ldap-plan.json
```

Removal apply adds `--confirm-removals` for that exact reviewed plan. The
corresponding API is `/api/directories/{id}/plan` and
`/api/directory-plans/{id}/apply`; the CLI checks the saved server plan and
resumes bounded `snapshot_in_progress` responses. Fixture `Core` calls are
not executions of those CLI commands.

[LDAP provider](../recipes/platform-ldap-provider.md) supplies LDAPS/STARTTLS
listener configuration, exact allowed peers, certificate/key files, policy
client settings and `ldap.search` on `client/ldap`. Client creation is an
administrator/scoped `client.write` operation with explicit client-write
revision/idempotency inputs; service-agent creation is human-admin only.
There is no LDAP search HTTP equivalent: search/bind is the LDAP protocol,
while record administration has API/CLI equivalents.

The actual [OpenLDAP peer fixture](../../tests/ldap_provider_peer.rs) invokes
`ldapsearch -x`, adds `-ZZ` for STARTTLS, uses `-y <private-token-file>`,
`-E '!pr=1/noprompt'`, the registered base DN and CA verification `hard`.
Its assertions require two one-user pages, exclude outsiders/admin, then
exclude disabled Bob and reject a revoked agent. Wrong credentials exit 49;
crossed transports and an unrelated CA exit 1 without directory data. These
are separate from the ldap3 fixture's TOTP/rebind checks; neither proves
Active Directory compatibility.

### SCIM inbound and outbound

[Inbound SCIM](../recipes/platform-inbound-scim.md) names
`http_tests::scim_http_provisioning_is_owned_atomic_retriable_and_deprovisions_sessions`.
The actual [body](../../tests/identity/http.rs), lines 245–400, uses a
600-second agent with user/group read/write and group membership permissions.
Two HTTP POSTs to `/scim/v2/Users` carry the same
`Idempotency-Key: scim-create-42` and JSON body: both return 201 with ETag and
Location, equal JSON, and no password bytes. Unauthenticated GET returns
401 SCIM JSON. Later direct `Core` calls check ownership, atomic invalid
patch refusal, reciprocal group membership, no adoption of `admin`, and
deprovisioning invalidating the user's session. Those later calls do not
establish HTTP PATCH/DELETE status or header coverage.

The reusable [SCIM guide](../scim.md) prints administrator agent creation with
`--if-revision`/`--idempotency-key`, then `--agent-file ... scim Users`:
POST uses an idempotency key and input file; PATCH additionally uses
`--if-version '"RESOURCE_ETAG"'`. Keep these route-specific inputs distinct
from client writes and connector review confirmation.

[Outbound SCIM](../recipes/platform-outbound-scim.md) describes
`policy_tests::outbound_scim_plans_provision_groups_preserve_remote_attributes_disable_departures_and_stop_stale_jobs`.
The actual [body](../../tests/identity/policy.rs), lines 633–795, uses a second
riAuth loopback router, a private destination token file, source group `staff`,
`export_groups = true`, and agent `provisioner.read`/`provisioner.sync` on
`provisioner/directory`. The accepted source now retains an **unserved source
router** after target setup so queued owner/generation releases survive
direct controller steps. The existing eight-step helper is unchanged.
Assertions require actor-bound apply, no token in plans, remote user/Group
delivery, preservation of remote-owned `name.givenName`, confirmed departure
disable/Group removal, and stale work after agent revocation.

The CLI counterpart in the SCIM guide is `riauth provision targets`,
`provision plan payroll --out ...`, `provision apply --plan ...` and
`provision jobs`. The fixture target is `directory`, not that guide's
`payroll` target. Plan/apply routes are
`/api/provisioning/targets/{target}/plan` and
`/api/provisioning/plans/{id}/apply`; removal confirmation uses the exact
plan ID. Pending durable work is not delivery. The recipe fixture does not
prove named SaaS interoperability, every controller mode, or offboarding's
separate ambiguity/settlement operations.

### OIDC relying party and upstream issuer

[OIDC relying party](../recipes/oidc-relying-party.md) supplies a public `rp`
client with exact callback, `openid,profile,offline_access`, S256 PKCE and
back-channel logout URI. The actual [browser fixture](../../tests/browser.rs)
hosts an axum RP and riAuth on loopback. It verifies callback state/issuer,
ID-token signature/audience/nonce, and signed logout event, no nonce, JTI and
session ID. It actually invokes the binary's
`--server ... --session-file ... --non-interactive --json request approve
CODE --yes --remember`; initialization/client creation and logout are
direct `Core` calls. The recipe's reusable `client create rp` counterpart
includes `--settings-file`, `--idempotency-key` and `--if-revision`.
Login/approval/logout equivalents are `/api/login`,
`/api/authorization/decision` and `/api/logout`; a public client receives no
creation secret and sends no token-endpoint client secret.

[Upstream OIDC](../recipes/upstream-oidc.md) supplies source input, pinned
JWKS, scopes, exact callback and `ClientSecretBasic`, then prints
`riauth source put --file ...`, `source start upstream --out ...`, and
`source finish --file ...` first for review, then with `--yes`.
These map to `/api/sources`, `/api/sources/{id}/start` and
`/api/source-login/finish`. A source writer needs `source.write` and, for
auto-provision, `user.write`; this fixture uses the administrator.

The actual [named body](../../tests/identity/sources.rs), lines 4–74, and
[Upstream helper](../../tests/identity.rs) inspect Basic authentication,
authorization-code grant, callback and S256 verifier. It has no real
`/authorize` endpoint: the helper synthesizes that part. It checks pending,
review, explicit one-use completion, no same-email linking, accepted MFA
claims, seven invalid token overlays, source-disable refusal and local
Alice remaining valid. Okta, Google and Entra issuer journeys are unverified;
this helper uses riAuth's own generated signing key as its pinned upstream key.

### Forward-auth and cloud directory guides

[Forward-auth](../recipes/platform-forward-auth.md) supplies nginx/Traefik
templates, exact public proxy clients/callbacks, trusted peers, permissions,
expected redirects/headers/cookies and refusal checks. The actual
[nginx exercise](../../tests/outpost.rs) distinguishes `if builtin` Rust-only
assertions from nginx results. The actual
[Traefik body](../../tests/outpost_traefik.rs), lines 757–1066, checks
forged URL/identity/intent headers, background 401, authenticated headers,
filtered cookies, cross-origin 403, WebSocket origin checks, concealed
issuer auth endpoints and parent-session revocation. Its approval is direct
`Core::authorize`; nginx runs `riauth request approve`. Client setup and
session revocation have API/CLI equivalents; there is no forward-auth CLI
subcommand. The rate paragraph has the concrete omission proposed below.

[Workspace](../enterprise/ENT-03.md) supplies direct service-account JSON
and delegated subject configuration, private file modes, numeric-client-ID
domain delegation and read-only scopes. [Entra](../enterprise/ENT-04.md)
supplies tenant-bound endpoints, secret/certificate choices, Graph `.default`
scope and read-only application permissions/admin consent. Both print
`riauth directory {workspace|entra} list`, `plan corp --out ...`, inspect
`{changes, removal_impact}`, then `--if-revision <plan.revision> ... apply`;
removals additionally require `--confirm-removals`. Both document exact
list/plan/get-plan/apply API routes and actor-bound permissions on
`workspace/corp` or `entra/corp`, not LDAP's `directory/corp`.

The actual [cloud fixture](../../tests/cloud_directory.rs) uses a local fake
peer. `exercise` refuses LDAP-only permissions and another actor's apply,
checks secret redaction, replay without another upstream fetch, and reviewed
membership removal. Workspace direct token handling verifies RS256,
issuer/audience/sub/kid/read-only scopes and one-hour assertion expiry;
`workspace_direct_service_account_assertion_and_expiry` checks fresh-key
rotation at apply and a 10-second token refusal before directory IO. Entra
fixtures check Graph scope/transitive membership and complete snapshot
failure handling. None connects a real tenant. CLI mapping and review-header
transport were inspected in [cli.rs](../../src/cli.rs), lines 1709–1746 and
3050–3130, and [transport.rs](../../src/cli/transport.rs), lines 209–233.

## Historical execution evidence credited precisely

These are existing results, not commands run during this audit or results at
`88790de`.

The downloaded integration log for run **36950097067**, job
**110661000640**, includes checkout
`2d05c00deed3a6dafcd458a5f1a1a9beb17d0dd8`.
`/tmp/riauth-wave29-integration-110661000640.log` SHA-256:
`458a30895b5987c0bebe2fb7368a1a32969fa0d41ce90acbfa56f43299ccd186`.
The raw command/test/result blocks were reread:

| Historical command, default Platform; no test-support feature added | Exact result in that log |
| --- | --- |
| `scripts/test-ldap.sh` → `cargo test --locked --test ldap -- --ignored --nocapture` | `openldap_plans_stable_ids_tls_login_mfa_and_fail_closed_sync`: 1 passed, 0 filtered, 8.05 s |
| `RIAUTH_TEST_BROWSER="$(command -v riauth-chrome)" cargo test --test browser --locked -- --ignored` | `browser_terminal_login_callback_and_signed_backchannel_logout`: 1 passed, 0 filtered, 19.93 s |
| `RIAUTH_TEST_NGINX="$(command -v nginx)" RIAUTH_TEST_BROWSER="$(command -v riauth-chrome)" cargo test --test outpost --locked -- --ignored` | `nginx_forward_auth_terminal_sso_headers_and_revocation`: 1 passed, 1 filtered, 18.16 s |
| `RIAUTH_TEST_TRAEFIK="$RUNNER_TEMP/traefik" cargo test --test outpost_traefik --locked -- --ignored` | `traefik_forward_auth_real`: 1 passed, 13 filtered, 3.15 s |

The log records Traefik v3.7.13 download and checksum success for
`52cd039a34258dd61c617a95d69252bc6bcae27c520f338186c31c7fef8f6394`.
nginx/OpenLDAP distro packages are not pinned by this evidence. The integration
job succeeded; that run's separate check failure is not relabeled green.
Its 91 PostgreSQL contracts are not 91 D03 recipe executions.

The separate failed-step log, run **36951643190**, job **110666025654**,
`/tmp/riauth-wave29-failed-36951643190.log`, SHA-256
`5a94f6f07d83ae7f4565d8fb032c3a629f31b44cff99ac84e67f7c0078d376f8`,
shows the inbound SCIM and upstream OIDC names above **passing**, and the
outbound SCIM name **failing** at then-line 618. Its command was
`cargo test --all-targets --features test-support,fuzzing --locked`;
the identity target had **171 passed, 7 failed, 3 ignored**, 0 filtered,
140.21 s. This downloaded failing-step artifact has no checkout block, so
this audit does not invent an exact checkout SHA for those two positives.

The accepted [identity fixture report](local-wave29-identity-policy-ci-report.md)
identifies baseline `d31778f066d3bc79f94829cf6295e1eb8df2a84f`, correction
`893be4fc0f80bd9da86e98c04a888ca4e7c06601`, and the retained source router.
It records the outbound name with
`cargo test --locked --features test-support,fuzzing --test identity
policy_tests::outbound_scim_plans_provision_groups_preserve_remote_attributes_disable_departures_and_stop_stale_jobs
-- --exact --test-threads=1`: **1 passed, 180 filtered, 2.72 s**.
That command used private `target/wave27`, jobs 1, incremental 0, dev/test
debug 0. It is historical local evidence, not a new Linux run at this pin.

The LDAP-provider recipe records local OpenLDAP `ldapsearch` **2.7.1**,
dual-transport successful paged searches, negative exit codes and
disable/revoke results at `fcba8a5e81037edb365c02c291b512576817e920`.
Its script selects `cargo test --locked --test ldap_provider_peer -- --ignored
--nocapture`. This audit inspected the body/recorded commands but did not
recover a new raw native log or execute the script.

The accepted `docs/roadmap/local-wave30-six-task-completion-review.md`
and `docs/roadmap/local-wave30-six-task-root-disposition.md` at the published
pin record
`03606ed7cd42ea3f716e3f8dcbebf001da9c3470`:
`cargo test --locked --test cloud_directory -- --quiet`, **33/33**, and
the same target with `--features test-support`, **35/35**. At
`4df94b61edf9161cac5217b00691e55fdb6ee6ed`,
`cargo test --locked --features test-support --test cloud_directory -- --quiet`
passed **36/36**. These are aggregate cloud-fixture counts, not 36 operator
recipes or live-tenant results. Current fixture bodies and the original
review's source-equivalence limits were inspected; no new cloud run or
whole-file equivalence to the old executed suite is claimed.
Those two root documents were read with `git show` at the immutable pin;
they are absent from this unmerged own branch and were not copied into it.

## Approved and applied slice: forward-auth rate paragraph

Root approved ledger entry `wave30_D03_forward_auth_rate_paragraph`, scoped to
this original task/worktree; its runtime release remains false. Applied only
[docs/recipes/platform-forward-auth.md](../recipes/platform-forward-auth.md),
the single paragraph at pinned line 164 immediately before `Closest API and
CLI`, in guide commit `dd896380ba5588cc0ff3fd45cdda5aea4a4d8e13`.
No index, other recipe, script, test or product hunk changed.

The original text said `forward_auth` is “counted in memory on each node.”
That contradicts accepted [api.rs](../../src/api.rs), lines 1096–1131:
PostgreSQL uses `store.shared_rate_limit`, with `app.run_forward` for the
forward-auth category; redb uses `app.rates`. The
[effective defaults](../../src/config.rs), lines 188–205, are 6000 and 30;
[node_security.rs](../../src/node_security.rs), lines 211–223, compares the
effective category map and supplies recorded-value/remedy diagnostics.
[API](../api.md) line 397 and [operations](../operations.md) lines 453–488
already describe the accepted backend boundary and explicit upgrade policy.

Immutable proposal/applied-hunk mapping:

| Object | Exact reference |
| --- | --- |
| Original proposal | `0dc0a504cca3e5030495865b88c2dd93957a1cc0`, this report's original proposed-slice section |
| Original recipe | `88790deb62d32c84fa17dceb12cd93a727224e94:docs/recipes/platform-forward-auth.md`, paragraph at line 164; blob `64cea490b0d222f7157bbf00e983a1f836a670e3` |
| Applied recipe | `dd896380ba5588cc0ff3fd45cdda5aea4a4d8e13:docs/recipes/platform-forward-auth.md`, same line; blob `bc7eb471b051016c8b888b983cea448a7e39e68a` |
| Exact diff | `git diff dd896380ba5588cc0ff3fd45cdda5aea4a4d8e13^ dd896380ba5588cc0ff3fd45cdda5aea4a4d8e13 -- docs/recipes/platform-forward-auth.md`: one line removed, one added |

The applied paragraph is in the recipe, rather than duplicated as Markdown
or a diff in this report. It states both PostgreSQL categories use shared
counters and redb counters stay in the owning process. Defaults remain
6000/30; overrides resolve to effective agreed rates, and omitted/explicit
defaults agree. Recorded-value diagnostics and restart remedy remain explicit.
Root's approval clarification is included: existing old-format rows use the
stopped-node, backup and explicit upgrade procedure with both
`--confirm-authentication-policy` and `--confirm-rate-limits`; only a reviewed
missing-row adoption adds `--adopt-missing-agreement`. Existing policy is not
silently overwritten. The paragraph links the authoritative
[rate limits and admission procedure](../operations.md#rate-limits-and-admission)
and [API limits](../api.md#proxy-authorization-and-limits), with report-relative
targets. It retains 429/503 distinctions, the two-second queue timeout and
binary-fixture/deployed-peer limits.

This restored a truthful reusable configuration/failure/remedy statement to
an already tested recipe. It neither changes O03 nor adds a rate-exhaustion,
remote-peer, HA or paused-before-IO claim. The old statement of defaults,
binary-fixture limits and queue timeout remains represented.

Approved focused checks actually performed:

1. A focused Python byte comparison restored just this paragraph and required
   the entire recipe to equal its claimed immutable baseline. Numbers,
   backend branches and startup diagnostic were checked against pinned source.
2. `python3 scripts/check-docs.py` for Markdown/link validation.
3. `git diff --check` and exact changed-path/hunk inspection.

No Cargo command, benchmark, service or new runtime evidence is necessary for
this documentation-only correction. It cannot substitute for the independent
journey gate. The claim/proposal was reviewed before the existing-doc edit;
root alone owns the original independent new-user/operator gate.

## Separate remaining inputs and acceptance limits

Completed local correction: the backend/rate-agreement paragraph above.
Existing cloud guides already contain reusable configuration, permission,
result and failure instructions. Their lack of a dedicated recipe page, and the other profiles
listed in each recipe's inventory, merit owner review against original D03
scope; they do not justify silently adding every missing profile or reopening
accepted integration rows in this slice.

Independent journey evidence: [D01's walkthrough](d01-platform-cli-walkthrough.md)
records eleven partial disposable runs with different copied artifacts,
including the Platform snapshot
`58357fde77211e62dc51c14fb3fc216bdf143ceb`. It explicitly records unexecuted
steps. These are useful historical printed-command checks, not proof that
a new user and a new operator independently completed all documented
workflows. That gate needs a documented journey outcome and any concrete
failure corrections, without imposing a universal brand-new full-suite gate.

External inputs remain profile-specific: controlled Workspace/Entra tenant
delegation/consent and credentials; named OIDC applications/issuers and SCIM
products; real directory mappings; operator proxy TLS/topology; applicable
hardware and released artifacts. Historical disposable PostgreSQL executions
are credited where they occurred; no PG service was launched here, and live
replication/HA is not required or proved by this recipe audit. Deployment
availability, full-lifetime external-IO exclusion and old-binary rollback
proof are not inferred. The accepted nonrenewed 60-second connector admission
limitation and SCIM owner/generation freshness fences remain intact.

No Q03/Q09 campaign, held Group redesign, canceled accessibility work,
receipt-secret/header/PAM contract change, task mutation or worker contact
is proposed. Root owns review, integration/publication and status decisions.

## Checks actually performed for the initial audit

Read-only Git status/parent checks, explicit-project live D03 row read,
immutable recipe/index/source/fixture/CLI reads, historical raw-log filters
and SHA-256 verification, and existence checks for **34 pinned paths and
seven historical commit references**. An initial guessed
`docs/recipes/README.md` read failed because that path does not exist; it was
corrected to the actual `docs/README.md` index before drawing conclusions.

Initial report validation found two relative links to those pinned root
documents, which are absent from this own branch. The documentation checker
and the local-path assertion refused them. The references were changed to
explicit pinned repository paths, preserving the immutable evidence reads
without importing other files. This was a report-only correction.

Report-only validation: `python3 scripts/check-docs.py`, relative-link and
exact pinned-path/reference checks, raw-log hash checks, `git diff --check`,
and changed-path verification **passed** after that correction. The focused
validator checked 34 relative links to 31 unique pinned paths, both pinned-only
root documents, nine distinct commit references and both raw-log hashes.
The earlier 34-path/seven-reference inventory and these report-reference
counts describe different explicit sets. The staged whitespace/scope check
also passed before committing this single new report. No runtime,
Cargo, tests, benchmark, service, browser, network peer or desktop action was
performed; no existing evidence artifact or product file changed.

## Approved paragraph implementation evidence

The existing own branch was clean at initial proposal commit `0dc0a504`.
The recipe equaled the entire fixed `88790de` blob before editing. Guide
commit `dd896380ba5588cc0ff3fd45cdda5aea4a4d8e13` has that proposal as its
parent, changes only the recipe, and contains exactly **+1/-1 line** at 164.
Applied recipe SHA-256:
`08429f7380a397220563a06754126c62dce084078e42e117df7fe0740733e6c5`.
The report follow-up is a separate commit changing only this report.

Actual focused checks passed:

- `python3 scripts/check-docs.py`: Markdown links and build-directory layout.
- Python immutable-source/byte checks: the only changed line is 164;
  replacing it with the original paragraph reproduces the complete fixed
  recipe byte-for-byte. Current own `HEAD` before the edit had the same
  baseline bytes. Defaults, both backend branches, effective-rate lookup,
  recorded-value diagnostic, both maintenance confirmations, missing-row-only
  adoption, queue timeout and both new link anchors match the cited source.
- The entire authoritative `Shared policy and offline upgrade` procedure in
  own `docs/operations.md` equals fixed `88790de`, although that whole file
  differs elsewhere. No whole shared document was imported or edited.
- `git diff --check`, recipe-only staged scope/whitespace and guide commit
  path/hunk checks. All 34 report-relative links, 11 typed immutable commit
  references, both declared recipe blobs and final report-only
  scope/whitespace were also checked.

There was no new failed check or runtime correction in this implementation.
The initial audit's failed guessed index read and missing-own-branch link
correction remain recorded above; historical CI failures/passes and all
execution/source/provenance limits remain unchanged. No Cargo/test command,
service, peer, desktop action, worker/task/worktree creation, merge/reset,
main/accepted edit, push or status mutation occurred. D03 stays **in_progress**:
this paragraph does not establish the independent documented journey gate.
