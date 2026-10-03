# G05 original application-cutover scope: source audit and next input

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original task
`eee83438-ed75-4709-adf9-6807514a6d18`. Reservation:
`wave30_G05_application_cutover_original_scope_source_audit`. Supporting worktree
`42bb51c6-c198-4adb-bd92-0a5222853231`, existing shell
`f1575610-c6f0-4dbf-9f33-d01ed057a194`. Date: 2026-10-03.

## Disposition and one next step

Keep G05 open for one owner-named application's genuine cutover and route-back
rehearsal. The reviewed implementation already supplies migration classification,
target-bound plan/apply, supported identity and factor continuity, and the eight
local reference-RP cases. No concrete missing generic production facet was found
in the bodies reviewed below. Another synthetic fixture or converter move would
not supply the remaining application result.

The smallest next input is the previously requested target application's exact
RP/source/rollback inventory, with permission to use its isolated rehearsal
environment. No customer or application name is invented here, and this audit
does not repeat the request or authorize remote writes. Bind that input to one
rehearsal before proposing any implementation. An actual incompatibility would
then yield a specific source-first claim; none is established by this audit.

G01–G04 and R05 retain their accepted dispositions. This is not a new gate for
every SaaS, host, physical factor, tenant, released artifact or inexperienced
human study. The named application's own protocol and recovery contract determine
its exercised cases. D01's confidential-browser continuation and helpers are
separately owned and were neither executed nor changed.

## Original row and immutable basis

The current project export was read at
`planning/current-tasks.json` under the project's RiWork orchestrator directory.
It is 244,354 bytes, SHA-256
`0b38ca3eb8810a4628436eb5083f6329de848f3f4ddaa972c850601ad45c0835`.
The exact row is `[P2] G05 — Rehearse staged cutover per application`, status
`todo`, primary worktree `d1486050-a105-45b3-b199-b7e55ea8525c`. Neither assignment
nor status was changed. Its prerequisites are G01, G02, G03, G04 and R05.

Original requested outcome, verbatim:

> Test successful and denied access, claims, MFA, refresh, logout, recovery, and rollback.

Original workstream goal, verbatim:

> Make cutover predictable without pretending every credential or custom policy is portable.

Original completion gate, verbatim:

> The migration report tells an administrator exactly what transfers, what changes, and what still needs work.

The row requires relevant implementation, tests, documentation and artifacts as
applicable, actual verification and external prerequisites; documentation or a
worker recommendation alone cannot complete it. Its old scheduling sentence
does not authorize execution. This reservation authorizes only the new report.

All current product/guide reads used immutable published
`544d1340b80cd3e040dc13142cdcbc1d75fea4cb`, without checkout alignment. Own clean
entry HEAD was `74e9f7818cb7d5533120ddfcdb4940c4a04106fe` on
`roadmap/local-module-boundaries-wave27`. Full pinned `CONTRIBUTING.md` and
`SECURITY.md` were read; no applicable ancestor `AGENTS.md` was found. The
explicit static/report-only reservation excludes the contributing guide's broad
build/test campaign and documentation-index edit.

The same export reports these original prerequisites DONE:

| Row | Exact UUID | Accepted responsibility retained |
| --- | --- | --- |
| G01 | `2a53c3de-903c-4392-aea3-c1b97d5ec39e` | Exact/convertible/manual/unsupported preflight with reasons, actions and blockers. |
| G02 | `c3b6b41c-4472-4a16-9773-6e6f4ab6a8c4` | Deliberate subjects, issuers, groups, applications, credentials and verified source links. |
| G03 | `aa616ed9-2acf-43ed-bb51-bc6355378e3d` | Supported mappings/workflows convert; custom behavior requiring redesign is reported. |
| G04 | `e68a97cb-99d9-41e7-93f2-f209fcb55612` | Re-enrollment, factor/recovery/session and rollback communication. Draft notices are not observed delivery. |

## Source responsibilities and protections

[Migration](../migration.md) defines complete export collections, explicit
resolutions, the printed import/validate/plan/apply sequence, RP-specific
rehearsal and route-back. `ready_for_plan` means a usable manifest can be
reviewed, not that an application accepted it. Non-Authentik source inventories
do not acquire a converter by being classified. No private export or credential
file was opened in this audit.

| Owner/path and complete bodies reviewed | Dataflow and retained refusal |
| --- | --- |
| `src/migration.rs`: `subject_source`, `accounts`, `continuity`, `connection`, `verify_links`, `classify_issuer`, `proven_target` | Exported account and literal issuer/subject evidence determine continuity. Source links must match exported connections and one owner; email equality cannot prove a link. Missing connections on an auto-provisioning source block rather than create a second identity. Per-provider/global issuer differences and ambiguity are reported. |
| `src/migration.rs`: `subject_safe_dictionary`, `scope_claims`, `application_access`; selected credential/source/report branches of `convert` | Only proven mapping values convert. A missing definition or an unprovable `sub` key cannot be waived by acknowledging an ID. Fixed safe keys with custom values require explicit manual translation; expressions are never executed. Group ancestry can change a claim. Application bindings preserve supported admission conditions; unsupported or unsafe alternatives block instead of widening access. The entire `convert` body was byte-compared, not reread in full. |
| `src/state.rs`: `require_target_state`, `target_identity`, `fingerprint_identity`, `live_target_identity`, `preserve_proven_bindings`, `plan_state_before_persist`, `apply_state_confirmed` | Planning previews then aborts; live actor/revision/authority and target identity are checked again before persistence. Apply binds the stored actor, issuer, exact plan, current dependencies/target and review/removal gate before reconciliation. Reimport cannot steal an account, subject or source link. Changes and result/audit commit through the shared writer; unresolved private references cannot be treated as imported credentials. |
| `src/core.rs`: `authorize_identity`, `logout`; `src/assembly/oidc.rs`: `refresh` | Current identity, enabled client, group/MFA/claim/device requirements govern admission. Logout uses the shared session-revocation writer. Refresh validates the client/live grant, rotates the token and commits family revocation on spent-token replay. These source reads do not prove target-RP logout delivery. |
| `src/authenticator.rs`: `import`, `consume_recovery_code`, `consume_password_factor` | Imported TOTP validates encoding/settings/size, marks the current step spent and clears old recovery codes. Consumption changes the same account's replay state. A riAuth recovery verifier is single-use and needs enrolled TOTP; it is not an Authentik recovery token. |
| `src/recovery.rs`: `require_serving`, `invalidate_restored`, `complete` | Restored/unreconciled or changed-lineage state refuses serving. In-place invalidation requires other PG clients to be absent. Completion requires an explicit persistent-credential attestation and the exact pending recovery ID; a stale ID cannot clear another gate. This restores riAuth state, not Authentik or an RP route. |

The complete conversion/preflight/reimport/target-claim/source-link assertions
read in `tests/identity/operations.rs` include
`authentik_import_preserves_exported_subjects_and_blocks_incomplete_translation`,
`authentik_preflight_classifies_every_exported_item`,
`authentik_preflight_fails_closed_on_missing_or_mismatched_resolutions`,
`authentik_reimport_keeps_verified_accounts_and_never_moves_identities`,
`authentik_plan_rejects_target_identity_claimed_after_export`, and
`authentik_import_links_only_exported_source_connections`. The last fixture
drives a local upstream through the exact migrated accounts; it refuses forged,
unexported and duplicate connections. Its source definition is not a new run.

Complete imported-password and imported-TOTP tests in `tests/identity/factors.rs`
were read: successful legacy-hash verification upgrades to Argon2id without an
identity change; wrong/excessive-cost inputs refuse; supported TOTP settings,
spent-step replay, session invalidation, idempotent reconciliation and secret
nondisclosure are asserted. No fresh execution or physical authenticator result
is attributed to those definitions. Complete common fixture construction,
authorization-request construction and reviewed client-policy replacement were
read, including distinct proposer/reviewer/executor roles. Complete recovery
tests `serve_refuses_unreconciled_restored_state_before_listening` and
`in_place_recovery_matches_restore_policy_and_keeps_history` were also read.

## Original eight facets: actual evidence and remaining application observation

The whole 277-line `tests/g05_reference_oidc.rs` was read. Its exact current
11,366-byte source equals executed accepted
`cf827a9ee103672b329907a433fb96ac868fe178`. It uses synthetic Authentik records,
a disposable redb fixture and an in-process public reference RP. It starts no
browser, HTTP RP listener or Authentik service. The eight case markers appear
only after their assertions. The complete [G05 evidence page](g05-local-reference-oidc.md)
and its recorded output were read; no original runtime log was rehashed here.

| Original facet | Recorded local assertion/result | What remains for the named target |
| --- | --- | --- |
| Successful access | G05-01/03: zero blockers, plan/apply, provider discovery, exact callback including existing query, state/issuer, S256 code exchange, independent RS256/JWKS verification, audience and nonce. | Actual RP accepts its callback/token and grants application access to the intended existing account. |
| Denied access | G05-02: nonmember authorization fails; authorization-code count is unchanged. R05's local HTTP RP separately returns 403 without its application cookie. | Denied target identity gets no application access or accidental account/privilege; local cookie absence is not a target group-policy result. |
| Claims | G05-03: exported subject, engineering group and password `amr`; UserInfo subject agrees. Migration tests cover ancestry and incomplete/unsafe mappings. | Compare the target's exact issuer/subject, requested scopes, role/group/custom claim expectations and verified-email behavior. |
| MFA | G05-05: reviewed `require_mfa` denies password-only authorization; new riAuth TOTP enrollment revokes that session; password+TOTP yields stable subject and `pwd,otp`. | Target's chosen supported factor/assurance policy succeeds and insufficient assurance refuses. No migrated or physical passkey is claimed. |
| Refresh | G05-04: rotation succeeds; spent replay gives `invalid_grant` and invalidates rotated-family access. | Target's configured refresh/grant/session behavior, including reauthentication after refusal. Old Authentik refresh tokens are not migrated. |
| Logout | G05-07: riAuth session/access are revoked; a separate recovered session survives. | Target's configured logout endpoint/channel and application cookie/session behavior. No front/back-channel delivery follows from the fixture. |
| Recovery | G05-06: newly issued riAuth recovery code works once, reuse refuses, RP subject is stable. Accepted R05 separately proves application login after a closed-gate backup restore. | Use the target's selected account-recovery route; distinguish it from operator store recovery and observe application access afterward. |
| Rollback | G05-08: saved disposable JSON route file is restored byte-for-byte to an Authentik marker, with unchanged issuer. | Restore actual approved RP/proxy configuration and reach the retained original IdP with a credential that stayed there; account for both RP/IdP sessions. A route-file assertion is insufficient. |

### Executed historical cohorts, kept at their actual pins

| Cohort | Exact source/evidence and result | Limits |
| --- | --- | --- |
| G05 local reference | Page observation from base `a3ccbf1` plus test SHA-256 `64e43530f6c6e69c2fe4f7f9d45fabb2cfb5b1843a6b022eec2a418008602ba6`; accepted ledger run head `cf827a9ee103672b329907a433fb96ac868fe178`: `cargo test --locked --test g05_reference_oidc -- --nocapture`, 1 passed / 0 failed, eight case IDs. Historical environment used jobs=2 / threads=4. | Source-built in-process fixture, not a released artifact, browser or customer cutover. Historical jobs/settings are not a proposed build command. |
| G02 ownership/reimport | Accepted review records `679f2927885d9dc4dcc1c881fd972bcade70e07a`: reimport filter 2/2, target-claim 1/1, shared desired-state/verified-login ownership 1/1. `3ddcab4d43cfa1ce41222b6253dedbed573c0c64`: reimport 3/3 and target-claim 1/1, plus Essentials library check. | Current complete `convert` and `classify_issuer` bodies byte-match `3ddcab4`; this is source equivalence, not execution at `544d`. |
| Corrected preflight fixtures | Source `e08ef8cef4d103e578aec0537fe86bb592187b06`, accepted `b6f1e130ec58ba706b5085c9e0302fa642bda2ce`: the three conversion/classification/fail-closed functions named above each passed once, 180 filtered. Full execution report read. Exact command form: `cargo test --locked --features test-support,fuzzing --test identity operations_tests::<named_function> -- --exact --test-threads=1`. | Their complete current bodies byte-match the executed source. Original incomplete mapping definitions caused CI failures; the accepted corrections retain blockers and nondisclosure, rather than relaxing conversion. No all-CI-green inference. |
| R05 real local HTTP RP | Complete `evidence/r05-local-rp-2026-10-02.json` and root disposition read: one invocation, 19 checks passed, 2026-10-02 11:55:03–11:55:10 UTC. Production `c01c39ab4e092423d5522bedc50fff87656d8c0a`, runtime source `b5cea614c4f46d82aff2380c052bd2dffc760f9e`, binary SHA-256 `0f137475af5a8040d96a794b1ad331e7430be4467046b81b1312fb974b7e8a6a`. | Synthetic public loopback RP, real HTTP callback/token/cookie and native OpenSSL 3.6.4 verification. Authorization is scripted with a fixture bearer, not an independent browser/confidential flow or Authentik migration. |

R05's recorded parent command was
`python3 scripts/recovery-drill.py --binary "$PWD/.target-wave27/debug/riauth" --evidence "$PWD/.target-wave27/r05-wave30-local-rp.json"`.
It was not run by this audit. The complete 478-line RP helper was read, including
body/token limits, duplicate-JSON refusal, proxy/redirect avoidance, exact local
callback binding, state/nonce/PKCE, RS256/JWKS/audience/issuer/time/access-hash
checks, same-subject comparison, private cookie establishment and finally cleanup.
The callback carries no IdP bearer. Both source and restored app phases returned
callback/token/protected success; no-cookie access was 403. This helper is not
D01's confidential-browser helper.

The accepted 19-check receipt also records wrong-key/tamper exit 2 with no
restore target, occupied-target exit 5 with marker preservation, restored
sessions invalidated, serve denied before reconciliation, wrong ID 409 and
missing attestation 400 preserving the gate, then fresh service/application
login. The fixture's persistent credentials justified its own attestation;
a customer's snapshot cannot establish that review. Older redb and PG archive
16-check cohorts remain older executions. The physical PG cohort's same-lineage
blind spot and explicit offline invalidation are retained, not rewritten as
automatic restore detection. Its pending gate stayed closed. The earlier PG
restore CLI panic remains recorded in R05 history, not a current replay here.

The older G05 page's “R05 remains in progress” predates the accepted newer R05
root disposition. That stale status does not reopen R05 or turn its 19 checks
into external-RP evidence. Likewise G04's notices remain drafts; current G05 MFA
assertions do not prove notice delivery or physical enrollment.

## Credential portability and rollback input

[Re-enrollment](../reenrollment.md) was read alongside current credential bodies.
Supported authorized password-hash and TOTP references can transfer deliberately;
newly established passwords and newly issued riAuth recovery codes are distinct
from old credentials. Import marks TOTP's current step spent. Passkeys, old
sessions/cookies/tokens, recovery codes, administration and signing keys are not
automatically imported. Signing-domain preservation is an explicit key-custody
decision, not permission to read a private key during this audit. An unchanged
subject with a changed issuer still requires RP account-migration decisions.

Keep the original IdP/configuration available. A password changed only in riAuth
does not change the Authentik password; riAuth does not synchronize credentials,
accounts or sessions back. An old same-host passkey's post-rollback acceptance
was not observed. Restoring riAuth's format-3 backup is a separate closed-gate
procedure with explicit persistent-credential review; it is not route-back to
Authentik. Do not use an adoption bypass. Only older binaries supporting formats
1/2 are known to refuse format 3; rollback to those needs a compatible
pre-command backup, rather than a blanket assertion about all older binaries.

One input bundle is sufficient to reserve the next slice:

1. **Named application and accountable owner:** product/version, protocol and
   isolated target environment; exact current IdP/source and application/proxy
   route; scope of authorized export, rehearsal writes and route restoration.
   No production mutation authorization is inferred from “finish everything.”
2. **Literal identity and RP contract:** issuer including path/trailing slash,
   client ID/type, exact redirect/logout URLs, discovery/JWKS and signing-key
   custody decision, stable subjects, scopes/claims/groups/roles and admission
   policy. Identify one allowed and one denied test identity by private reference,
   plus the selected supported MFA/recovery/refresh/logout expectations.
3. **Complete private source and target inventories:** export users/groups,
   providers/applications, policy bindings, all referenced mapping definitions,
   sources and exported user-source connections; target-state export when
   reimporting. Record chosen reviewed translations and credentials as private
   references/versions. Other source systems receive inventory classification
   until a supported converter exists; do not relabel them Authentik.
4. **Rollback and session ownership:** saved RP/proxy configuration, retained
   original IdP and an original usable credential, route-back operator, finite
   stop condition and disposition of sessions/refresh families created on each
   side. Include a verified compatible riAuth backup/key inventory only for the
   store-recovery route actually selected. Never promise sync-back or recovery
   of a lost key from a snapshot.

The user has already requested the target; its absence from the inspected row
and published evidence is the precise pending input, not a request for a new
roadmap item. Secret contents belong in a private operator environment. Public
evidence should carry source/artifact hashes, finite stage/status/counts and
reviewed redacted claim comparisons, never tokens, OTPs, cookies, passwords,
private keys, sensitive IDs or export contents.

### Prospective bounded path, held until root reserves it

First review this one bundle statically against `src/migration.rs` and the
selected RP contract. The existing printed CLI path is
`riauth-maintenance import-authentik --file <private-input> --out <new-private-directory>`,
then `riauth validate --file <manifest> --json`,
`riauth plan --file <manifest> --out <new-plan> --json`, and
`riauth apply --plan <reviewed-plan> --non-interactive --run-id <one-fixed-rehearsal-id> --json`.
These are prospective commands; `<...>` values are deliberately not executable
or a new CLI alias. A blocker yields no applicable manifest. Use current live
authority/review/removal floors and exact approved retry binding; do not
reinterpret a receipt as permission to expose a credential again. Reviewed
client-creation receipt-secret behavior and route-specific required/optional
headers remain unchanged.

After root approves matching binaries, exact inputs, one controlled fixture and
all required owner actions, reserve one finite application sequence for the eight
facets above. Use the actual RP's normal protocol flow and original IdP for
route-back. Reuse accepted local cases as control evidence; do not rerun every
provider or duplicate D01. If a printed/configured step fails, stop dependent
steps, retain its safe stage/status and propose the smallest precise hunk before
any repeat. No silent alternate API, membership bypass or assurance downgrade.

Any future local lab would use fresh UID-owned nonsymlink mode-0700 workspace,
0600 private evidence/session/reference files, matching artifact/source hashes,
an unused loopback listener, own PIDs/groups, finite command/body/log caps and
finally stop/join/read-back cleanup. Root must review the concrete resource plan;
a candidate 20-minute fixture bound is not authorization. Recheck disk, preserve
8 GiB floor with 2-second samples and an owned stop at 9 GiB; no cache deletion
or stopping another process. No Cargo is requested. Real route restoration is
performed only by its authorized owner. Any future desktop interaction uses
RiWork Cua.ai Driver MCP after reading its descriptions/state; unavailable setup
must be reported without switching providers.

PAM fallback, held source Group materialization, receipt/header contracts,
nonrenewed 60-second admission limits, paused-IO limitations and truthful
at-least-once provider settlement remain intact. This audit adds no observer,
lease extension, remote-write permission or full-release claim. A09 ARM run
`37101183416` owns the validation/Cargo lane; this work acquired no slot.

## Read coverage, hashes and static checks

Full bodies read: G05 fixture and evidence page; migration and re-enrollment
guides; R05 guide, RP helper, public 19-check JSON and root disposition;
identity-operations correction report; the individual production/test bodies
listed above. The G02 sections of the six-task review, complete root disposition
and E-MIG/G05 sections of D05 were read. Larger Rust modules and D05 were not
read end-to-end. Hashes of larger files are identity checks, not a claim of full
body review, execution or independent rehash of private runtime logs.

All entries below are at published `544d1340b80cd3e040dc13142cdcbc1d75fea4cb`:

| Path | Bytes | SHA-256 |
| --- | ---: | --- |
| `src/migration.rs` | 167857 | `5f923acc9befca63aebb97d04bc29462d36a3e8e573a79513f847dd1fbead152` |
| `src/state.rs` | 96050 | `ac7a222e1e70e2831f879b87a6f745c349ded13ddac7af88ae8f1d6f01a3d2b1` |
| `src/authenticator.rs` | 17353 | `dd22860ca897576bc7eadccb27644e0b2a74def8749d469e015c8f196d467b25` |
| `src/recovery.rs` | 28367 | `ff40bc88774658da7de79bfaabd845463ea338fe88847e3c570fbf690f3e1602` |
| `tests/g05_reference_oidc.rs` | 11366 | `64e43530f6c6e69c2fe4f7f9d45fabb2cfb5b1843a6b022eec2a418008602ba6` |
| `tests/identity/operations.rs` | 209252 | `92bc7cd3440f529d1bfe83e522f74eb37caa078d743185dbe9442c4a316a9a9b` |
| `tests/identity/factors.rs` | 47627 | `37d7d6abe19568f59a563808439a7286d4acddbedf6f3a5dd1b5663eefcbc4c4` |
| `docs/migration.md` | 48252 | `37329fc51e187be48d16cacccd7de739b93fd23b66b1b7c4e9215a82ad020a86` |
| `docs/reenrollment.md` | 30148 | `c7b3b7e07e18d9b0c15979f901b62cd1fd27da515e946d40298d94294c867fe5` |
| `docs/roadmap/g05-local-reference-oidc.md` | 6838 | `889ed8fe9decb35f8d4ce19130e996f1721eb89337e60233ef4b1f335cdb6713` |
| `scripts/recovery-drill-oidc.py` | 22540 | `f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d` |
| `docs/roadmap/evidence/r05-local-rp-2026-10-02.json` | 6090 | `5498bbf1f089224947ef3f0cbc7e163c4aa35683eb8d0256943100f49012df41` |

Data-only Git/source comparisons passed: whole G05 fixture equals `cf827a9`;
complete `convert` (65,032 bytes) and `classify_issuer` (3,718 bytes) equal
`3ddcab4`; the three complete corrected preflight fixture bodies equal `e08ef8c`.
Complete conversion-body SHA-256 is
`bd6bac1cb86548d5bc08841ecfcb6e6b21d368e5fd193f6d497044d005d7190d`;
issuer-body SHA-256 is
`73adface8e8e569d4e4ce5b233461adb13dc5b029c1815ed61c7c987fb387e5c`.
These comparisons read immutable bytes; no archived product/helper function ran.

Checks actually performed by this audit:

- `pwd`, `git status --short`, `git rev-parse HEAD`: assigned cwd/branch history,
  clean entry HEAD confirmed; no alignment or merge.
- Read-only `git show`, `git ls-tree`, `git rev-parse`, `rg` and bounded `sed`
  reads; Python data-only JSON/hash/span comparisons described above: PASS.
  All full 40-hex commit pins in this report resolve with `git cat-file -e`.
- Report-local Markdown link existence: PASS, three relative links. Report
  trailing whitespace/newline check and `git diff --cached --check`: PASS.
- `python3 scripts/check-repo-hygiene.py`: EXIT 0, 1,024 staged tracked files.
  No private key material or generated/private file was added.
- `python3 scripts/check-docs.py`: EXIT 1, only existing root directories
  `target-wave29-source`, `target-wave28-scim`, `target-wave28-portal`,
  `target-wave28`, `target-wave27`. No Markdown-link failure. No directory was
  deleted, moved or inspected for credentials, and no checker was changed.
- Report-only staged-name/stat and protected-file scope checks: PASS. The one
  new path is this report; existing files remain byte-identical to entry HEAD.
  Commit hooks configuration was unset and no active non-sample executable
  hook was present. Post-commit cleanliness and exact hash accompany handoff.

No runtime, Cargo, compiler, product/native tool, HTTP/provider/network,
browser/Driver, service, managed state, board change or other-worker contact was
performed. Root owns interpretation, integration and original-row status.
