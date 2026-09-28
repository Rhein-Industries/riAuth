# Workflow definition model

Status: **W01 model slice.** This page describes the canonical data model and
static validation in [workflow.rs](../src/workflow.rs). No request path, storage
record, management endpoint, executor or editor uses it yet. The existing
sign-in, enrollment, recovery, consent and source-stage code paths are unchanged.
Nothing here enables configurable workflows, and it is not evidence of the P01
Platform capability or of an Essentials/Platform build split.

## Scope

One typed, versioned representation covers five categories: `authentication`,
`enrollment`, `recovery`, `consent` and `sensitive_action`. The module imports no
Core, storage or protocol module. It defines data, parsing, validation, a
deterministic fingerprint and a pure transition lookup. A later executor (W02)
and invariant suite (W03) are expected to consume only `Validated` definitions.

The JSON Schema is published as `workflow` through `riauth schema workflow` and
`GET /api/schema/workflow`, alongside the existing schemas. Publishing the schema
does not mean any API accepts workflow definitions.

## Shape

```json
{
  "format": "riauth.workflow/v1",
  "id": "essentials-consent",
  "revision": 1,
  "category": "consent",
  "origin": "builtin",
  "entry": "session",
  "limits": {"max_duration_seconds": 600, "max_executions": 4},
  "steps": [
    {"id": "session", "action": {"type": "resume_session"}, "max_attempts": 1,
     "timeout_seconds": 60, "cancellable": true,
     "transitions": [{"on": "verified", "to": "consent"}, {"on": "failed", "to": "denied"}]},
    {"id": "consent", "action": {"type": "request_consent"}, "max_attempts": 1,
     "timeout_seconds": 300, "cancellable": true,
     "transitions": [{"on": "granted", "to": "success"}, {"on": "denied", "to": "denied"}]}
  ],
  "terminals": [
    {"id": "success", "outcome": "consent_granted", "requires": []},
    {"id": "denied", "outcome": "denied", "requires": []}
  ]
}
```

* **Steps** run one built-in action. The action set is closed: `identify`,
  `resume_session`, `verify_password`, `verify_passkey`, `verify_totp`,
  `verify_recovery_code`, `verify_email` (`reset` or `invitation` purpose),
  `verify_source`, `request_consent`, `enroll_credential`, `reset_password` and,
  for Platform only, `custom`. Each category permits a fixed subset.
* **Transitions** route a step's signals (`verified`/`failed`,
  `completed`/`failed`, `granted`/`denied`, or a custom stage's declared outputs
  plus `failed`). Every signal needs a route. For each signal, the first matching
  transition wins and the last must be unconditional, so there is never a default.
* **Conditions** are `account_has`, `has_proof`, `request_requires_mfa`, and bounded
  `all`/`any`/`not` combinations of them (depth 4, 16 nodes).
* **Terminals** end the run with the category's success outcome or `denied`.
  Cancellation and expiry are not transitions; an executor ends the run as
  `cancelled` or `expired` and cannot route them elsewhere. `RunState` and
  `RunBinding` are the storage-neutral state and definition-binding types.

All objects reject unknown fields and unknown `type` values. Every string is a
lowercase identifier of at most 64 bytes (`[a-z][a-z0-9_-]*`). The model has no
display strings, scripts, URLs, free-form maps or credential fields. An
identifier could still hold a short token if an author put one there, so the
model gives no field for secrets but cannot detect one. Parse errors give the
position and expected names. They leave out unknown names and values taken from
the input.

## Proofs and success

Only built-in verifiers produce proofs, and only on their success signal:
`session`, `password`, `passkey`, `totp`, `recovery_code`, `reset_email`,
`invitation`, `source`, `consent`, `enrolled` and `password_reset`. A proof's
required account, request and run bindings are declared by its type
(`Proof::binding`), not a definition field. The later executor must enforce
those bindings on actual evidence. A custom stage can route by its
outputs but never produces a proof, cannot reuse built-in signal names and only
receives permissions that its registration grants.

Validation computes every set of proofs that can arrive at each node, treating
every conditional branch as possible. It rejects the definition unless each
success terminal satisfies, for every arriving set:

| Outcome | Minimum proofs (category floor) |
| --- | --- |
| `authenticated` | `password`, `passkey` or `source` |
| `enrolled` | `enrolled`. `enroll_credential` needs `session` plus `password` or `passkey`, or `invitation` for a first passkey or password |
| `recovered` | `password_reset`, which needs `reset_email` first |
| `consent_granted` | `session` and `consent` |
| `action_authorized` | `session` and `password` or `passkey` |

`totp`, `recovery_code` and `request_consent` also need an account-binding proof
on every incoming path. A `failed` or `denied` signal cannot lead directly to a
success terminal. An `account_has` condition can only be used where every path
already holds an account-binding proof, so routing cannot reveal whether an
identifier names an account. A terminal's `requires` list can add proof
alternatives but cannot remove the floor. Success in enrollment, recovery and
sensitive actions must set `max_proof_age_seconds` to 300 or less. No terminal
declares assurance values such as ACR/AMR. The executor will derive them from
recorded proofs.

Static validation cannot know an account's factors or a request's requirements.
`Validated::complete` therefore rechecks a finishing run against its recorded
proofs and supplied `Facts`. It applies the floor and `requires` again. It also
requires a passkey, TOTP or recovery code for authentication when the account has
TOTP or the request requires MFA, and for sensitive actions and session-based
enrollment when the account has TOTP. Definitions cannot relax these rules. An
executor must call it before recording success. It does not yet model a trusted
upstream MFA assertion, so an upstream-authenticated account with local TOTP
still needs the local factor. W02 must reconcile that rule with today's
[source stage](../src/source.rs) behavior.

## Bounds

Loops are rejected, including self-loops; retry is expressed only by
`max_attempts` (1–5). Every node must be reachable. The longest path, weighted by
attempts, must fit `limits.max_executions` (at most 64). Runs last at most
86,400 s, steps at most 3,600 s (custom stages 30 s, 4,096 output bytes), with at
most 32 steps, 8 terminals, 12 transitions per step and 64 KiB per document.

## Essentials and Platform

`workflow::defaults()` returns seven `builtin` definitions: passkey sign-in,
password sign-in with TOTP and recovery-code fallback, passkey enrollment after
fresh verification, invitation passkey enrollment, password reset by mail proof,
consent, and sensitive-action re-verification. They reference no configured
source or stage. An Essentials profile accepts only these definitions. A
`builtin` definition must equal a shipped default after parsing. Whitespace, key
order and explicit `null` optional fields do not matter. Configured
definitions cannot use the `essentials-` prefix. A Platform profile additionally
accepts `configured` definitions, `verify_source` steps naming a supplied active
source, and registered custom stages. The caller supplies those references in an
`Environment`; the model performs no lookups.

[workflow_model.rs](../tests/workflow_model.rs) pins the defaults'
fingerprints, so a change to a default shows up in review. The model itself does
not require a revision increase.

## Left to later work

These are not implemented or established by this slice:

* Running definitions, persisting runs, or connecting to today's sign-in,
  passkey, lifecycle, consent and source-stage paths (W02). The defaults describe
  target journeys. Parity with existing endpoint behavior has not been checked.
* Runtime enforcement: one account per run, exact request/session binding, proof
  age, single use, attempt counters, cancellation and expiry, source and stage
  state at run time, invitation acceptance never adding a credential to an
  existing identity, and rejecting passwords for upstream-only accounts.
* Binding runs to their security dependencies. `RunBinding` covers the
  definition's ID, revision and fingerprint. It does not cover the `Environment`
  (active sources, stage registrations and permissions) or any approval record
  that RI-WF-002 requires.
* Invariant and race tests for RI-WF-001/002 and Q02-C07/Q05-R01 (W03).
* Management API, desired-state, storage, versioned approval, editor, templates,
  and product capability reporting or gating.
