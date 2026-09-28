# Workflow definition model

Status: **W01 model, W03 proof provenance, and a limited W02 executor path.**
The server now persists bounded runs, attempts, requests and evidence, and exposes
an Essentials password reauthentication workflow for a live bearer session. It
uses the existing local password verifier and finalizes through the W03 store
boundary. It does not yet complete an OIDC sign-in transaction or issue a new
session, and other built-in verifier actions remain unconnected. The existing
sign-in, enrollment, recovery, consent and source-stage paths are unchanged.
Nothing here enables configurable workflows or the P01 Platform capability.

## Scope

One typed, versioned representation covers five categories: `authentication`,
`enrollment`, `recovery`, `consent` and `sensitive_action`. The module imports no
Core, storage or protocol module. It defines data, parsing, validation, a
deterministic fingerprint, a pure transition lookup and a completion boundary.
A later executor (W02) must consume only `Validated` definitions and connect the
completion boundary to trusted verification and durable storage.

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

Only proof-producing built-in actions yield proof kinds, and only on their
success signal:
`session`, `password`, `passkey`, `totp`, `recovery_code`, `reset_email`,
`invitation`, `source`, `consent`, `enrolled` and `password_reset`. A proof's
required account, request and run bindings are declared by its type
(`Proof::binding`), not a definition field. W03 makes completion use stored,
typed evidence rather than a caller-supplied list of proof kinds. Each evidence
record names the built-in action and step attempt that produced it, its
verification and expiry times, consumption state, and the account, account
epoch, request and run to which it belongs. Its optional session must match
the run's exactly; `session` and `consent` proofs require one. A custom stage
can route by its outputs but never
produces a proof, cannot reuse built-in signal names and only receives
permissions that its registration grants.

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
`Validated::complete` therefore rechecks a finishing run through a crate-private
completion store. It loads the run, its recorded step trace and evidence, checks
the pinned definition, account epoch, request, run and session bindings, and
accepts success evidence only from a fresh, unconsumed built-in action on the
recorded path. It rejects a wrong or unreachable terminal, a forged custom-stage
proof, stale success evidence, and evidence reused from another run. Trusted
account and request facts then apply the floor and `requires` again. They also require a
passkey, TOTP or recovery code for authentication when the account has TOTP or
the request requires MFA, and for sensitive actions and session-based enrollment
when the account has TOTP. Definitions cannot relax these rules. The seam
currently **rejects `enrolled` and `recovered` completion**, even with otherwise
valid receipts. Those actions can increment the account epoch, and passkey
enrollment can revoke the session. W02 must provide one atomic protocol that
binds the pre-mutation evidence at epoch E to the actual mutation and final
state at E+1 before either outcome can succeed.

A `denied` terminal still needs an active run and a valid recorded path with
matching historical evidence provenance, bindings and step attempts. An earlier
receipt may have expired by denial time; its expiry does not make denial into
authentication success. A spent receipt, expired run, changed account epoch or
wrong path still fails. Supported completion finalizes the run and consumes
receipts together through the store boundary. W02 must make that operation
atomic in the durable store and recheck live account/session authority, run
expiry and success-proof freshness at commit time. This seam does not yet
model a trusted upstream MFA assertion, so an upstream-authenticated account
with local TOTP still needs the local factor. W02 must reconcile that rule with
today's [source stage](../src/source.rs) behavior.

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

* Connecting the remaining verifier actions and existing OIDC/browser sign-in,
  passkey, lifecycle, consent and source-stage paths. The current W02 endpoint
  covers only local-password reauthentication for a live bearer session; it
  neither consumes an OIDC request nor issues a session. Endpoint parity has not
  been checked.
* An atomic credential-mutation receipt/finalization protocol for enrollment and
  password reset across account epoch E to E+1, including passkey enrollment's
  session revocation. Until then these success outcomes remain blocked. A
  denial after an epoch change also remains blocked by current-facts binding;
  W02 must resolve such runs with its expiry/cancellation or mutation protocol.
* Runtime source and stage state, invitation acceptance, and the other built-in
  verifiers. The password path now stores attempt timing, enforces retry and run
  bounds, cancellation and expiry, rejects upstream-only accounts, and rechecks
  account, session, request and receipt authority in its final transaction.
* Binding runs to their security dependencies. `RunBinding` covers the
  definition's ID, revision and fingerprint. It does not cover the `Environment`
  (active sources, stage registrations and permissions) or any approval record
  that RI-WF-002 requires.
* End-to-end invariant and race tests for the remaining verifier integrations
  and both durable backends. The focused password path is covered on the local
  store; PostgreSQL has not been exercised for this executor slice.
* Management API, desired-state, storage, versioned approval, editor, templates,
  and product capability reporting or gating.
