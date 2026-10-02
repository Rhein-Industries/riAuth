# M03 and M07 original-acceptance disposition

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`. M03 task
`3b9fbfa6-716a-4ce8-842a-2c953bab3d0f` and M07 task
`0da684c3-b5cd-45e5-b190-1d0ce97f2c80`, worktree
`9c54c023-a95f-48e9-a908-6f97a5677c98`. Fixed main
`72259b4b40283727a2a9f6e2279b3ef15d4ea532`. Both tasks remain in progress
pending root review of this disposition.

## Method and evidence limits

The original rows were reread from the project's
`planning/current-tasks.json`. Cumulative published ports were read at the
fixed main from Git objects only. Claude Opus 5.5 led the work, with two
read-only Sonnet mappers (both reported `claude-sonnet-5-5`):

- an M03 interface mapper covering every management mutation across the
  bearer API, browser routes, server CLI and riauthctl;
- an M07 mapper for GUI-only state and resource coverage across APIs and
  manifests.

The lead re-checked each finding below in the source.

This is a static reconciliation. Nothing was built, tested or run. Runtime
evidence remains what the accepted reports and published CI record. The
`tests/identity/{oidc,operations,policy}.rs` CI fixes belong to other lanes
and were not reviewed or rerun.

## Original rows

- M03, "Use one management service everywhere": "GUI, CLI, and API must share
  authorization, validation, transactions, idempotency, and audit behavior."
- M07, "Extend desired-state coverage": "Manage workflows, roles, connectors,
  and other supported resources through versioned APIs and manifests—not
  GUI-only state."
- Shared workstream gate: "The same change has the same permission checks and
  outcome regardless of which interface submits it."
- Shared completion evidence: "Review the relevant implementation, tests,
  documentation, and released artifacts as applicable."

Explicitly accepted and not reopened:

- the credential-issuance receipt contract, including reviewed
  client-creation recovery;
- the per-route required versus optional header policy;
- the PAM fallback;
- operator-only `riauth.toml` choices: listeners, PAM approvers,
  reviewed-membership and reviewed-client settings, connector reconciliation
  modes and controllers;
- D1 and D2 from the retry-parity review.

A resource that an interface does not offer is not treated as a divergence.
No universal all-resource browser, release or HA gate is added.

## M03 mapping

The server CLI and riauthctl call only bearer HTTP routes for remote
mutations. Local-store access in `src/cli.rs` is limited to offline commands:
init, prepare-setup, recover-admin, restore, migrate-postgres, recovery and
keygen. Browser handlers in `src/portal/admin.rs`, `access_review.rs`,
`sources.rs` and the self-service handlers call the same `core.*` methods as
`src/api.rs`, using a cookie token. `Core::principal` and `management()` are
shared.

| Family | Offered on | Shared writer |
| --- | --- | --- |
| Users, groups and reviewed memberships, clients (create, update, rotate, four reviewed families), delegated grants (immediate and reviewed), invitations, device decisions | bearer, browser, server CLI, riauthctl | yes |
| Agents, registration templates, signing keys, Windows devices, client and RADIUS certificates, offboarding, source put, SSF administrator streams | bearer, server CLI, riauthctl | yes |
| Directory and SCIM operations | bearer, server CLI, riauthctl; the browser offers cloud test, verify and schedule plus deactivation dismissal | yes |
| Desired-state plan, apply and export (including connectors) | bearer, server CLI, riauthctl; the browser offers a one-workflow editor subset | yes (narrower browser editor) |
| Workflow review and targeted revocation | all four | yes; header policy accepted |
| Workflow activation | all four | **no**, see the seam below |
| PAM requests and decisions | all four | yes; accepted fallback |
| Session and consent revocation | bearer, server CLI, riauthctl; browser self-service | one writer with intent arms (finding 4) |
| Backup | bearer, server CLI, riauthctl (stream); legacy JSON on bearer only | different functions (finding 2) |

## The remaining local seam (M03; also M07 workflow management)

**Browser and Core workflow activation ignores headers that the caller
supplies.**

- `Core::activate_workflow` (`src/workflow/approval.rs:150-158`) calls
  `activate_or_replay_in` with a no-op first-activation guard. It stores no
  receipt and never compares a supplied `If-Match`.
- The browser route `/api/admin/workflows/activate` (`src/portal/admin.rs`
  `workflow_activate`) calls this method. Its `writer()` guard checks only
  same-origin and the cookie, so supplied headers pass through and are then
  ignored.
- The same browser and Core path honors supplied headers for review and
  revocation (`retry_command` with `RetryHeaders::Optional`,
  `approval.rs:216-306`). Bearer activation (`src/api/workflow.rs`
  `activate_command`) requires both headers, compares `If-Match` before a
  first activation and saves its receipt.
- `docs/workflows.md:1500-1501` states for all three services:
  "Browser/Core retain optional headers; supplied headers are honored."

Outcome difference for the same change: a browser or raw Core activation that
supplies a stale `If-Match` commits the activation (200), while bearer
returns 409. A browser key reused for a different activation request is not
detected. This is not covered by the accepted per-route header contract. That
contract makes headers optional on browser and Core; it does not make
supplied headers ignored, and the documentation says the opposite.

Smallest fix, which preserves accepted activation semantics:

- Move the body of the bearer `activate_command` into `src/workflow/approval.rs`
  as one shared envelope taking `RetryHeaders`.
- `Core::activate_workflow` calls it with `Optional`, so 428 is never raised.
- The envelope reads the receipt only when a key is supplied and compares
  `If-Match` only when one is supplied, and only before a first activation.
- It saves the receipt on `Activated` when a key is supplied.
- `Stale` keeps commit-then-409.
- `src/api/workflow.rs` calls the shared envelope with `Required` and drops
  its duplicate body.
- `activate_or_replay_in`, `activate_in`, `historical_floor`, raw activation
  semantics and every runtime handler stay unchanged.

Prospective file claim:

| File | Hunk |
| --- | --- |
| `src/workflow/approval.rs` (W02 owner coordination) | `Core::activate_workflow` wrapper; new shared activation envelope with `RetryHeaders` |
| `src/api/workflow.rs` | `activate_command` delegates with `Required`; duplicate removed |
| `tests/workflow_approval_api.rs` | Browser HTTP and raw Core cases: supplied stale `If-Match` gives 409 with no approval or pointer; supplied key saves a receipt; an exact same-key retry is revalidated and writes nothing; reusing the key with a different request gives 409; no headers behaves as today |
| `src/portal/admin.rs`, docs | None; the documentation already states the intended behavior |

## Other findings, not blocking the seam

1. **Legacy JSON backup has no audit** (low to medium).
   - `POST /api/operations/backup` (`src/api.rs:595`) calls `Core::backup`
     (`src/operations.rs:126-201`) in a read transaction with no audit. The
     stream route writes `operations.backup.started`, `completed`, `failed` and
     `cancelled` (`src/api/backup.rs`).
   - Both CLIs use the stream, and the legacy route is bearer-only. So this is
     an audit gap on a legacy API route, not a cross-interface difference for
     the same change.
   - Owner decision: audit it with the stream's actions, or retire the legacy
     route.
2. **riauthctl `user disable` sends a different change** (low).
   - riauthctl sends `revoke_sessions: true`, while the server CLI and the
     browser send only `enabled: false` (`crates/riauthctl/src/admin.rs:313-325`).
     Disabling already bumps the user's epoch, so session invalidation is the
     same.
   - The extra flag adds a second epoch bump and an SSF `session-revoked`
     event (`src/management.rs:1517-1524,1807-1809`).
   - The server treats identical requests identically. This is a client
     command-semantics difference.
3. **Self-service session and consent revocation is gated differently by
   channel** (policy).
   - The browser requires a fresh, factor-verified browser session.
     Withdrawal also requires a visible consent row.
   - The bearer arms do not (`src/management/sessions.rs:123-216`,
     `src/management/consents.rs:130-181`). The code calls this deliberate.
   - It is end-user self-service rather than administrator management, and is
     outside the three accepted contracts. Record it as a product decision, not
     a seam.
4. **PAM console pre-check** (low, by design). `/api/admin/access/*` requires a
   management principal first. An approver who is neither an administrator nor
   delegated uses `/api/portal/access/*` or the bearer API.
5. **CLI shortcut for an already-applied plan** (low). The server CLI and
   riauthctl return the server's plan status for an applied plan rather than
   calling apply again. The status endpoint authorizes on its own.

## M07 mapping

No declarative GUI-only state was found at the fixed main.

- **Browser writes reach bearer writers.** Every state-changing browser route
  calls a `core.*` method that a bearer route also calls: users, grants,
  invitations, groups and memberships, clients and their reviewed families,
  deactivation dismissal, PAM decisions, cloud test/verify/schedule, workflow
  plan/apply and workflow review/activate/revoke.
- **The one browser-only flow is a bound ceremony.** Administrator passkey-first
  user creation (`src/assembly/passkey.rs:317-433`) is deliberately bound to a
  browser WebAuthn session and documented (`docs/passkeys.md`,
  `docs/PORTAL.md`). The user it creates is also creatable by
  `POST /api/users`, a manifest `UserSpec` (including `password_disabled`) and
  an invitation with passkey acceptance.
- **Coverage by resource:**
  - workflows have versioned APIs and a `workflows` manifest; approvals are
    reviewed actions over the API;
  - roles have the delegated-grant API and `delegated_grants`;
  - connectors have the desired-state API and four tables plus
    `retired_connectors`, with operator pins and export status;
  - SSF administrator streams have an API and `ssf_streams`;
  - users, groups, clients, sources and links have both.
- **API-only resources** (agents, registration templates, signing keys,
  certificate bindings, Windows devices, invitations, PAM access, offboarding,
  cloud schedule) are not GUI-only. The one-time-credential and
  relative-lifetime kinds are documented as dedicated-API resources.
- **Operator-only settings** (listeners, PAM approvers, reviewed settings,
  reconciliation modes and controllers) live in `riauth.toml` by design.
- **Versioning:** manifest `api_version` `riauth/v1`; plans bound to hash,
  base revision and issuer; the schema catalog; capabilities
  `riauth.capabilities/v2`; connector format `riauth.connector/v1`; workflow
  approval schemas.

Stale orchestrator status text, not code:

- `docs/roadmap/coverage-inventory.md` (M07 row) still says workflows, roles
  and connectors live only in `riauth.toml`.
- `docs/roadmap/local-wave28-task-closure-audit.md` (M07) still says main
  rejects connector definitions.

Both contradict `src/state.rs` and `docs/agent.md` at the fixed main.

## Closure verdicts against the original rows

- **M03: not closable yet.** The workflow activation seam above is an actual
  locally implementable difference, and it falls outside the accepted
  contracts.
  - After that seam lands and its focused tests run, no further cross-interface
    divergence outside the accepted contracts was found.
  - Findings 1-5 need owner decisions or are low; none is required by the
    original row as a cross-interface difference for the same change.
  - Applicable evidence beyond local code remains: runtime results from
    published CI for the ported slices, and released-artifact evidence where
    the release process applies.
- **M07: original row met in local implementation.**
  - Workflows, roles, connectors and the other supported resources are
    manageable through versioned APIs and manifests, and no declarative state
    is GUI-only.
  - Because the workstream gate is shared, the same activation seam should
    land before M07 is reported closed for workflow management.
  - Remaining evidence is external or runtime: published CI for the connector
    and workflow ports, live LDAP, Workspace, Entra and SCIM peers, deployed
    multi-node PostgreSQL, and released artifacts as applicable.
  - The two stale status documents above need orchestrator correction.

Root reviews this disposition. Neither task status was changed here.
