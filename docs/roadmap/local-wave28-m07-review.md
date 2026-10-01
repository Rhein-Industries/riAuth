# Wave28 pinned M07 security and integration review

Date: 2026-10-02. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Reviewer worktree `ed9ac424-59f4-4520-905b-919aea3521eb`, branch
`roadmap/local-revisions-coordination-wave27`. Only this report was changed.
No source implementation was imported, edited, built or tested in this review.

## Recommendation

Accept the **cumulative connector feature through `b05c828`**, subject to narrow
integration preserving current main. Do not accept an earlier security prefix
as a finished feature. Hold `32f9aed` and the workflow portion of `ee2b2ab`
until the bearer activation path preserves the accepted live activation replay
and stale-run sealing contract. M07 as a whole remains **in progress**.
This is a static source review, not new runtime or release-artifact evidence.

S04 `38886cbe26cee7fe2deb6f89809e79ce4252a88c` remains held as instructed.
Root decides integration and board reconciliation; no task status was changed.

## Fixed inputs and original acceptance

- Management source worktree: `/Users/dominik/orca/projects/riAuth-public-preview-local-management-wave27`.
  Reviewed Git objects end at **`e5faa33ad6a8ce82abffd2bac0df9e1ddf21a817`**;
  source files were read with `git show`, never from its moving worktree diff.
- Current accepted main: **`32770ab73270901ec94a2d1249cbd8bb6052a415`**.
  Common base with the source: `4cc1c8bf82f48d9561f1f61c7fb13487b8d610b0`.
- Management report was read as supporting, mutable worker evidence, not as
  pinned implementation: `docs/roadmap/local-wave27-management-report.md` in
  that source worktree; observed SHA-256
  `d3b97b34c39bccb09ad5924663f90f45dc33b953cc18f321d900bb8f2babcf76`.
- Original M07 task `0da684c3-b5cd-45e5-b190-1d0ce97f2c80`: manage workflows,
  roles, connectors and supported resources through versioned APIs/manifests;
  the same change must have the same permission checks and outcome across
  interfaces, without bypassing review. Implementation, tests and applicable
  released-artifact evidence must support closure.
- Original S04 task `43b4ad2e-5b7c-46db-98ff-148be042d6ac`: avoid unrelated
  conflicts while invalidating relevant policy/authorization dependencies,
  retaining concurrency and identity guarantees.
- Original O03 task `85240c6b-8c87-4a62-a07e-68c7ed1a5d5a`: detect capability
  and security-setting mismatches; define shared jobs, rate limits and cache
  freshness; operators can identify failure, safe behavior and correction.

RiWork task details and `local-wave28-file-ownership.json` were read using the
explicit project UUID. The ownership file retains W02/W05 production ownership
of `src/workflow/approval.rs`, with M07 only its three visibility changes and
the connector guard at integration. No new implementation ownership is assumed.
Applicable repository guidance is `CONTRIBUTING.md`; the user's static-review
and narrow-check instructions take precedence over its general full-suite list.

## Per-source-commit verdicts

“Accept in bundle” means apply the reviewed changes only together with their
listed security follow-ups. It does not authorize a partially protected feature
or replacing accepted files with their old source versions.

| Source commit | Exact verdict and reason |
| --- | --- |
| `c8dd4f8cc0b75c7e5fb0d7bf23abd44d8a0e74f8` | **HOLD standalone; ACCEPT in connector bundle** with `3cf5de6`, `f0ac6e2`, `aafb125`, `b05c828`. Adds typed manifest definitions, shared writer, human-admin restrictions and connector exclusions. Its end-state-only ownership can release/reclaim a credential at a different destination; first-use destination is unrestricted; rows are inert at this prefix. |
| `4d2cb6912d82c048caea4b334369f52ef027a261` | **HOLD standalone; ACCEPT in connector bundle**. Startup activation and per-process status are useful, but this prefix activates the earlier credential weaknesses. Preserve main's surrounding startup gates and workers. |
| `3cf5de6d08b2b027c15c9e263538ca47f512de28` | **HOLD as final security state; ACCEPT in connector bundle**. Permanent name bindings close release/reclaim redirection, absolute lexical comparisons improve exclusion, and aggregate audit views redact definitions. First use is still unpinned and startup binding rechecks arrive in `f0ac6e2`. |
| `f0ac6e2f3c72eb981a8de1f0efe522cb66a25d3c` | **HOLD without follow-ups; ACCEPT in connector bundle** with `aafb125` and `b05c828`. Adds operator origin admission, all credential-field classification, startup binding checks, CA/certificate exclusion and confirmed retirement. This prefix's folded pin lookup can admit an unpinned case variant on a case-sensitive filesystem. |
| `aafb125ff4a13d91c382fba856d83d72d8030e78` | **HOLD without `b05c828`; ACCEPT in connector bundle**. Exact pin spelling closes the file-name issue and dormant pins stop affecting TOML-only operation. Its HTTP-style normalization of LDAP opaque hosts must receive the subsequent as-dialed fix. |
| `b05c8283f8a3ce4045c98e066d4778e9eae6dc26` | **ACCEPT** as the final connector follow-up. LDAP pins now compare the ASCII host the client dials and refuse percent-encoded/non-ASCII hosts. Server CLI export retains connector activation status. Includes focused test/clippy adjustments; no claim that this review reran them. |
| `32f9aed565ccf1c3f5dd242aa5df561d38185122` | **HOLD** for finding F1 below. Review/revoke envelopes and first activation have bounded input, transaction, human-admin, receipt and revision checks, but new bearer activation bypasses main's hardened replay service. Visibility-only reconciliation alone does not fix this. |
| `8bf11abfb314ef2e0c1119ac14cd4855686376a6` | **ACCEPT** the local PostgreSQL evidence source and additive script target, dependent on the complete connector bundle. Tests cover plain/encrypted values, stale/concurrent apply, replay, restart, binding mismatch and retirement. These are ignored local tests, not new reviewer execution or deployed HA evidence. Preserve other script targets. |
| `ee2b2ab3d74c0b523d5f281d1bfa19b99ed1737b` | **HOLD whole commit pending F1/M03 dependencies**. **ACCEPT separately** its connector export-status and ignored-header refusal behavior on the corresponding accepted CLI base. Workflow CLI clients correctly bind request/response identifiers and send retry headers, but depend on the held bearer service. Keep its other operations/test changes subject to the management review. |
| `4cf53d226d4b12d7fe6465c378f0295c9f47d879` | **ACCEPT** the ignored loopback LDAP end-to-end evidence source, dependent on the connector bundle and riauthctl operations base. It checks pin refusal, delayed activation, import and synthetic password non-disclosure. No live LDAP/TLS or released-binary claim. |
| `e5faa33ad6a8ce82abffd2bac0df9e1ddf21a817` | **ACCEPT** the independent `src/cli/grants.rs` `&PathBuf` to `&Path` lint fix. This has no connector security behavior. |

Other M03 commits in the ancestry were used only as interface context, not
re-reviewed or approved wholesale here. `git cherry` identifies accepted patch
equivalents for `eb738c0`, `d2eacb1`, `e06416f`, `8dc7b39`, `1625395`,
`069734c`, `95a1a6d`, `32d8bcb`, `a7af730`, `5efe978`, and `a57bdee`.
The hyphen-digest behavior also exists in main's `09c2ba0`; its source commit
`622fd94` is not reported patch-identical by `git cherry`. Preserve accepted
equivalents. Remaining registration/device/operations/backup/email M03 slices
need their own review; this report grants no new verdict on them.

## Findings requiring coordination

### F1 — Medium: bearer activation replay bypasses accepted live validation

At the pinned head, `src/api/workflow.rs:556–566` calls
`approval::activate_in` inside `approval_command`; that uses generic
`Core::mutation_checked` (`src/core.rs:69–108`). A matching receipt returns
at lines 87–95 before the callback, revision check or operation runs.
The pinned `activate_in` also returns an existing approval when only its
executor/plan/pointer match (`src/workflow/approval.rs:309–324`), without
`selection_holds` or stale-run sealing.

Current main's accepted `956fb35948c8266d1539e13cd449ad36a715f78d` deliberately
moved activation replay into `Core::activate_workflow` at
`src/workflow/approval.rs:140–167`: revalidate selection, environment and
author/reviewer/executor authority; if stale, commit run sealing and return
conflict without restoring the pointer. Its regression source is
`tests/workflow_approval.rs:820`,
`environment_binding_and_activation_replay_require_live_review`.

Concrete static trace: activate with a bearer receipt, disable the reviewer or
change a referenced source, then retry the same activation/key as the still-live
executor. Generic receipt replay returns the former
`selection: "approved-definition"`; it never invokes live validation/sealing.
The equivalent main Core call conflicts and retires stale runs. Even retaining
main's implementation and changing only function visibility leaves this receipt
bypass. A new-key replay calls the first-activation helper rather than main's
replay branch and typically fails its old plan revision, again omitting that
branch's sealing behavior.

This establishes an interface/security-contract defect. It does **not** establish
unauthorized token issuance or pointer revival: existing execution checks must
remain in force, and receipts are historical results. No dynamic reproduction
was run. Coordinate a transaction-level activation service with W02/W05 that
preserves live replay and commit-on-stale sealing for the bearer envelope;
avoid nested writers. Only that activation path needs special reconciliation;
the review/revoke adapters do not acquire a new execution authority here.

### F2 — Low: management report overstates frozen scope

The report says scope is selected at first use and then frozen by the binding.
`Definition::binding_value` (`src/connector_definitions.rs:297–330`) excludes
scope; `scopes` at 346–357 and `credential_change` at 1340 flag a scope edit
but allow it with the same credential. The regression source explicitly
permits it (`tests/connector_manifest.rs:1061–1108`). Correct the closure
description: destination, trust-reference and presented identity bindings persist;
scope changes remain human-admin credential changes. This does not defeat the
operator's origin pin or imply an unauthorized scope grant by the remote peer.

## Connector security trace at the final pin

**First use and release/reclaim.** `check_pins` compares an exact relative file
name and requires every reached origin to belong to the operator-written set
(906–925). Reach includes the static target, OAuth token endpoint plus target,
Workspace broker/Google token endpoint plus Admin SDK, and Entra token endpoint
plus Graph (375–434). HTTP clients disable redirects. LDAP uses the host as
dialed, with separate scheme/effective port. Pins choose origins, not first-use
bind DN/client ID/tenant/delegated subject or scope.

Bindings key the lowercased credential name to kind/destination/trust-reference/
identity digest. Reconcile overlays legacy names, checks the desired set against
bindings, and persists both released old names and new names in the same apply
writer (1084–1140, 1184–1207, 1290–1316, 1369–1376). Retirement preserves
tombstones, including parsable legacy rows (1214–1254). Same destination and
identity reclaim is allowed; a new destination/identity requires a new name.
Retire-and-reclaim in one manifest receives the same binding check.

**Every file field.** `Definition::visit` exhaustively destructures the four
connector types and `Oauth`/`WorkspaceDirectAuth` (156–266). Credentials are LDAP
password, Workspace broker secret/direct key, Entra shared secret/private key,
SCIM static token, OAuth client secret and refresh token. CA files and Entra's
public certificate are public fields. `check_non_secret` rejects public names
that are pinned or bound, and `check_set` rejects public paths owned by active
credentials (930–947, 998–1018). Public trust files can be shared. Unknown
inline-secret fields are refused by the typed `deny_unknown_fields` structures.

**Paths and directory exclusion.** Relative names must be nonempty canonical
ASCII components, with bounded length, no absolute/parent/current spelling,
repeated separators, backslash or trailing-dot component (525–556). Config loading
resolves the secret root relative to the TOML file (`src/config.rs:776–779`).
Cross-entry ownership/exclusion compares absolute, lexically normalized,
case-folded paths. The root may neither contain nor lie inside `data_dir` and
may not contain other configured database, mail, TLS, PostgreSQL, signer,
controller, webhook, device-trust, certificate/listener/NAS/EAP files
(694–774). TOML connector ownership is checked separately with the stored set.

This is **lexical normalization, not filesystem canonicalization**. It does not
detect symlinks, hard links, mount aliases or secret-file content substitution.
Those require a dedicated directory provisioned and controlled by the operator;
the worker report documents that boundary. No stronger confinement or immutable
credential-bytes guarantee is accepted here. Plan/apply/export do not read files.

**Startup and fingerprints.** `merge` is read-only, rechecks row key/format/content
digest, typed validation, pins, persisted bindings, same-ID TOML agreement,
combined ownership/caps and merged config validation (1404–1480).
`Core::open_store` calls it before node-security/capability gates and before
migration, lineage reconciliation, backfill and workers (216–241). Unset opt-in
leaves definitions dormant; stored cloud definitions still block Essentials via
edition preflight. Loaded status compares this process's captured `(revision,
digest)` with the current row; changed/retired definitions remain in the old
process until restart (1487–1536). Existing connector operation fingerprints
consume the merged configuration.

This is a per-process startup snapshot, not hot reload or a linearizable
activation fence. Rows and bindings are read in separate read transactions.
A concurrent definition edit/retirement can leave an opener using the prior
configuration; subsequent status exposes the mismatch. Pins/bindings are startup
guards, not a promise to stop an already-running process after operator changes.
Stored `updated_by` is provenance, not a requirement that its author stay an
administrator forever; this configuration persists like other administrator state.

**Plan/apply/export and rollback.** Plans use the existing principal, authority
digest and global revision. Apply requires the exact stored plan, recomputes
impact, rechecks authority/expiry/config admission, writes definitions/bindings,
compares changes and commits result/audit together (`src/state.rs:1568–1683`,
1694–1889). A refusal after writes rolls back the writer. Retirement always
requires exact confirmation via `retired_connectors` impact; automation refuses
connector changes. Applied replay reauthorizes access to the stored result and
does not reapply. Connector audit rows carry digests/IDs, not definitions/files.
Export returns unresolved references to full human administrators; agents and
delegated humans receive no connector definitions/status.

`connector_definitions` and `connector_credential_bindings` are retained recovery
buckets (`src/recovery.rs:171–183`). Restoration retains only the snapshot's
bindings; an older backup can roll back newer identity bindings. Operator origin
pins remain outside the store, but are not an identity/scope anti-rollback fence.
Retirement followed by recreation starts row revision 1. Older binaries do not
gain connector activation from these rows; no released downgrade/rollback proof
is claimed. Existing recovery invalidation/lineage gates must remain unchanged.

**Global fallback and S04.** `has_connectors` covers all four maps and retirement.
Group/client-name/client-description/user-display shapes exclude it, and apply
rechecks those shapes. Mixed connector manifests remain global. Both workflow
review and editor reject a mixed connector manifest. The S04 retained-plan lookup
hunk can follow this connector baseline; it must not broaden any shape or restore
an earlier source version of these guards.

## Integration dependencies and evidence limits

- Apply hunks to main, never whole stale files. Preserve accepted W02/W05
  environment/source digests, live replay/sealing and execution checks; W07
  extension isolation; CI cloud apply authorization ordering; A03 cloud assembly
  moves; O06 storage-allocation route/cache; all management route additions.
- Preserve main's `1e7fefc` shared admission callers/owner-generation release,
  `880583c` shared forward-auth counter and `3a3583a` SCIM stamps. M07 schema
  derives can be added to the current Target/Oauth types without replacing their
  implementation. Keep `connector_admissions` and `scim_oauth_freshness` recovery
  invalidation alongside M07's retained buckets. Preserve accepted mail/logout/
  SSF leases and dispatch pins. CI-owned test files remain untouched.
- The worker reports connector tests on redb/Essentials, two disposable local
  PostgreSQL modes, and one real-debug-binary loopback LDAP import. Their test
  sources were inspected here; those runs were **not repeated** or independently
  observed. Ignored targets are not current CI/release/deployed-provider evidence.
- No deployed HA, real cloud changes, external messages, browser/accessibility
  scan, desktop input, source-worker contact, new tasks/worktrees/shells, reset,
  merge or push occurred. No Cargo process ran. Disk was about 46 GiB free at the
  initial check; no accepted build output was used. Future approved builds must
  use this worktree's private `target/wave27`, jobs 1, incremental 0, and stop
  before free space falls to 8 GiB.

## Proposed next local work — awaiting ownership

**S04 first:** after root accepts the connector baseline, reconcile held
`38886cb` with all `has_connectors` exclusions and its focused retained-stale-plan
regression. That finishes the lookup slice, not the original whole task.
Next bounded gap: narrow families still fail plan persistence on any unrelated
global revision change between preview and the final writer, even when their
captured row dependencies and actor authority remain current (main
`src/state.rs:1596–1632`). Propose scoped commit-time validation for only the
existing narrow families, with a focused interleaving proof of unrelated success
and relevant-row/authority conflict. Connector/mixed/target-bound plans must retain
the global check; S02 Group representation stays held. Needs `src/state.rs`
and `tests/state_reconciliation.rs` ownership before edits.

**O03 next:** HTTP counters are shared on PostgreSQL, but each node supplies its
own threshold (`src/api.rs:1102–1118`; `src/store.rs:736–769`). Two compatible
nodes with limits 2 and 200 use the same counter yet the latter can admit after
the former refuses. `meta.node_security` compares only issuer, active
capabilities and four authentication integers; it does not detect this mismatch.
Propose an explicit agreement for the complete effective rate-limit categories,
including default values, with fail-closed mismatch diagnostics and an explicit
existing-store adoption/rollback contract. Do not silently canonize whichever
node starts first. This is a proposed implementation, not a new task or an
authorized format migration. Needs rate/config/agreement/diagnostics ownership;
API/core/recovery/provisioning work remains paused pending root assignment.

S04 and O03 both remain in progress against their original acceptance. Keep the
60-second, non-renewed shared admission limitation, no paused-process external-IO
fence, synthetic peer limits, and current shared SCIM stamp contracts explicit.

## Checks actually performed

Initial `pwd`, clean branch/status, pinned-object identity and `df -k` checks
succeeded. Read original RiWork tasks, ownership, applicable guidance, fixed Git
history/diffs, source/test code and the management report; compared accepted
patches with `git cherry`. Report verification uses `git diff --check` and
`python3 scripts/check-docs.py` only. No fresh build or test execution.
