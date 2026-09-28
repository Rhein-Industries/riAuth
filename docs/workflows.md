# Workflow definition model

Status: **W01 model, W03 proof provenance, and bounded W02 verifier paths.**
The Platform server persists bounded runs, attempts, requests and evidence, and exposes
password, passkey and OIDC/SAML source reauthentication for a live bearer session.
Password and source paths require local TOTP when enrolled. All use the existing
verifiers and finalize through the W03 store boundary. They do not complete a
downstream OIDC sign-in transaction or issue a new session. Other built-in
verifier actions remain unconnected. The existing
sign-in, enrollment, recovery, consent and source-stage paths are unchanged.
The W04 Platform conditional application policy now narrows existing client
authorization and projects scoped claims from verified session signals; see
[OIDC profiles](oidc-profiles.md#platform-conditional-application-policy).
It does not yet enable configured workflow execution. The W02/W03 workflow
proof receipts remain bound to their account,
session, request and run, and this client policy cannot produce a workflow proof
or success outcome.
The source paths use server-defined workflows and do not accept arbitrary
configured definitions.

## Scope

One typed, versioned representation covers five categories: `authentication`,
`enrollment`, `recovery`, `consent` and `sensitive_action`. The model defines data, parsing, validation, a
deterministic fingerprint, a pure transition lookup and a completion boundary.
The executor consumes only `Validated` definitions and connects the completion
boundary to trusted verification and durable storage.

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

Source receipts additionally retain the source fingerprint, exact linked
subject/account record and consumed upstream transaction reference. The
production adapter checks the current enabled source and explicit account link
inside the completion transaction. Receipt expiry is capped by the original
signed upstream assertion expiry as well as the request and receipt lifetimes.

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
receipts together through the store boundary. The password, passkey, source and TOTP adapters
make this operation atomic and recheck live account/session authority, run
expiry and success-proof freshness inside the writer transaction. This seam does not yet
model a trusted upstream MFA assertion, so an upstream-authenticated account
with local TOTP still needs the local factor. W02 must reconcile that rule with
today's [source stage](../src/source.rs) behavior.

## Local password reauthentication

On Platform, `POST /api/workflows/password` pins a workflow to the live bearer
session's account. Accounts with a local password and enrolled TOTP use the
server-owned `platform-password-totp-reauthentication` definition, whose success
requires both proofs. Accounts without TOTP retain the shipped password-only
path. Neither path accepts a directory-managed password or a caller-selected
definition. Essentials ordinary sign-in is unchanged.

`POST /api/workflows/{id}/password` accepts a `password`. The executor reserves
one step attempt, calls sign-in's password hash verifier outside the writer,
then rechecks the exact run, reservation, account epoch, live session, request,
credential hash and account lockout before storing evidence. The receipt's
freshness starts at verification time. Password proof never creates a staged
browser login or session, so an incomplete MFA chain cannot be attached elsewhere.

A correct password advances an MFA run to TOTP and leaves its account-wide
failed-credential count intact. The full chain must succeed before that count
is cleared. Cancellation, correct-password retries and new runs cannot restore
the guessing budget. The existing bound TOTP endpoints below consume both
receipts and complete the run in one transaction. The two-factor definition
allows three attempts per factor within ten minutes; exhaustion denies instead
of entering the model's unconnected recovery-code fallback.

## Local passkey reauthentication

On Platform, `POST /api/workflows/passkey` starts the shipped passkey workflow
for the exact live bearer session. `POST /api/workflows/{id}/passkey/start`
reserves the current attempt and returns its `workflow` view and `public_key`
WebAuthn options. `POST /api/workflows/{id}/passkey` accepts only a `response`
containing the signed WebAuthn credential. Ceremony references and the binding
nonce remain server-side. The verifier binding covers the account, epoch,
session, request, run, definition, step, attempt and step start time.

The existing local passkey verifier checks the challenge, origin, RP ID,
signature, user verification, current account, live credential and counter.
Challenge consumption, counter update, removal of its temporary staged login,
W03 receipt creation and consumption, and W02 finalization share one writer
transaction. No session is issued or upgraded. Passkey evidence satisfies the
existing MFA floor, including a request requiring MFA or an account with TOTP.
Invalid credentials spend the reserved challenge and attempt; cancellation and
expiry discard that attempt's ceremony. A response from a cancelled run or
earlier attempt cannot verify a new challenge. The workflow permits three
attempts of at most five minutes within its ten-minute run limit.

## Source reauthentication

On Platform, `POST /api/workflows/sources/{source}` with a live bearer session
returns a `workflow` view and `authorization_url`. The executor pins the exact
account, account epoch, session, request, definition, run, step, attempt and
reservation nonce into the existing source login before redirecting upstream.
It accepts enabled OIDC or SAML sources and existing explicit account links; it cannot
provision or reattach an account. Only one workflow can be active per session.

The existing OIDC callback checks the signed ID token, issuer, audience, nonce
and authentication time. The SAML ACS verifies its signed assertion, recipient,
audience, request correlation, authentication time and replay state. SAML
workflow receipts expire at the earliest of the assertion Conditions,
SubjectConfirmationData and optional SessionNotOnOrAfter bounds, without clock
skew extending any lifetime. The pinned source protocol must match its verifier
result. `POST /api/workflows/{id}/source` with the original
bearer session polls that reserved result; the caller supplies no proof, claims,
signal or receipt reference. Verification consumes the source login and creates
W03 evidence. For accounts without TOTP, that writer also consumes the evidence
and finalizes the W02 run. Accounts with TOTP instead enter the local factor step;
the upstream proof alone cannot finish that workflow.
Source configuration changes, unlinking, mismatched accounts or sessions,
revoked/expired authority, old assertions and replay fail closed. Cancellation
and run expiry discard the source reservation. The ordinary source-finish API
cannot use workflow-bound logins, and this path issues no session or OAuth code.

This bounded workflow has one source attempt and lasts at most ten minutes.
Authentication must occur at or after that attempt starts; its proof must be
at most 120 seconds old at completion. Workflow SAML validation uses this same
strict lower bound for `AuthnInstant`; ordinary source logins retain their
five-second tolerance. Protocol clock-skew allowances do not extend workflow
proof freshness or signed assertion expiry. OAuth-only sources remain unavailable
on this path. Upstream
MFA assertions are not converted into local-factor proofs.

## Local TOTP after primary verification

An account with TOTP uses the distinct, pinned
`platform-source-totp-reauthentication` definition. It requires both source and
TOTP evidence. The original source-only definition remains available for its
existing pinned runs. Password-plus-TOTP uses the same local-factor adapter and
requires a fresh password receipt instead. After the primary step,
`POST /api/workflows/{id}/totp/start`
returns a `workflow` view, an opaque `challenge` and its `expires_at`.
`POST /api/workflows/{id}/totp` accepts that `challenge` and the authenticator
`code`, using the original bearer session. The handle binds the account, epoch,
session, request, run, definition, primary proof, current step and attempt.

The executor uses the ordinary TOTP verifier, enrolled algorithm/digits/period,
and account-wide `totp_last_step`. It rechecks the primary proof's freshness and
authority before verification and again at completion, including local-password
eligibility or the source's live account link. The factor
counter, both evidence consumptions and the final run state commit in one
transaction. A completion error rolls back factor consumption. This path issues
no session, changes no session assurance, and stores no submitted code.

Invalid codes spend the reserved handle and one of three attempts. Retrying
requires a new handle; delayed submissions and handles from cancelled runs
cannot complete it. The ordinary account lockout also applies, so cancellation
or a new workflow does not reset failed-code counts. The factor deadline cannot
extend the primary proof's expiry, its 120-second freshness bound, or the
run/request deadline. A source proof also retains its signed assertion expiry.
Epoch changes invalidate pending verification. Recovery-code fallback remains
unconnected.

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
  lifecycle, consent and embedded source-stage paths. Current W02 endpoints
  cover password, passkey and OIDC/SAML source reauthentication for a live bearer
  session. They do not consume an OIDC request or issue a session. Endpoint parity
  has not been checked.
* An atomic credential-mutation receipt/finalization protocol for enrollment and
  password reset across account epoch E to E+1, including passkey enrollment's
  session revocation. Until then these success outcomes remain blocked. A
  denial after an epoch change also remains blocked by current-facts binding;
  W02 must resolve such runs with its expiry/cancellation or mutation protocol.
* Arbitrary configured workflows, custom stage execution, trusted upstream MFA
  assertion adapters, recovery-code fallback, invitation
  acceptance, and the other built-in verifiers. The password
  path stores attempt timing, enforces retry and run
  bounds, cancellation and expiry, rejects upstream-only accounts, and rechecks
  account, session, request and receipt authority in its final transaction.
* Binding runs to their security dependencies. `RunBinding` covers the
  definition's ID, revision and fingerprint. It does not cover the `Environment`
  (stage registrations and permissions) or any approval record
  that RI-WF-002 requires.
  The source reauthentication request now pins and rechecks its source fingerprint
  and the receipt's explicit account link; broader dependency/approval binding remains.
* End-to-end invariant and race tests for the remaining verifier integrations
  and both durable backends. The focused source regression exercises the signed
  OIDC callback, binding and authority changes, rollback on stale evidence, and
  competing completion writers on the local store. The focused passkey regression
  uses signed WebAuthn credentials to exercise session/request/run/attempt
  isolation, revoked authority, retries and competing completion writers.
  The focused TOTP regression uses signed upstream evidence and real enrolled
  codes to exercise bound retries, cancellation, stale/consumed source evidence,
  account-wide replay, epoch invalidation and atomic competing completion.
  The password-plus-TOTP regression checks both real verifiers, retained lockout
  across correct passwords and cancelled runs, proof substitution/expiry,
  ordinary sign-in replay rejection and competing finalization writers.
  PostgreSQL has not been exercised for this executor slice.
* Management API, desired-state, storage, versioned approval, editor, templates,
  and product capability reporting or gating.
