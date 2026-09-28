# Workflow definition model

Status: **W01 model, W03 proof provenance, bounded W02 verifier paths, and W06 Platform authoring.**
The Platform server persists bounded runs, attempts, requests and evidence, and exposes
password, passkey and OIDC/SAML source reauthentication for a live bearer session.
Password and source paths require TOTP or a one-time recovery code when enrolled. All use the existing
verifiers and finalize through the W03 store boundary. These canonical chains
can also complete an explicitly approved downstream OIDC request. No path
issues a new session. Connected credential mutations cover passkey enrollment
authorized by a fresh existing-passkey proof, mail-proven password recovery,
and first-password or first-passkey invitation acceptance. Their real verifiers
finalize the credential, proof consumption and epoch change atomically.
Ordinary sign-in, other enrollment, consent and source-stage paths are unchanged;
browser recovery and invitation acceptance
keep their existing responses and require a separate sign-in.
The W04 Platform conditional application policy now narrows existing client
authorization and projects scoped claims from verified session signals; see
[OIDC profiles](oidc-profiles.md#platform-conditional-application-policy).
Platform now starts four active configured authentication shapes: local password
alone for an account without TOTP, password followed by enrolled local TOTP,
password with a TOTP or recovery-code choice, or one user-verified passkey step
for an account with an enrolled passkey.
The W02/W03 workflow proof receipts remain bound to their account,
session, request and run, and this client policy cannot produce a workflow proof
or success outcome.
The source paths use server-defined workflows and do not accept arbitrary
configured definitions. Configured enrollment, recovery, custom stages and
other authentication chains remain unconnected.

## Scope

One typed, versioned representation covers five categories: `authentication`,
`enrollment`, `recovery`, `consent` and `sensitive_action`. The model defines data, parsing, validation, a
deterministic fingerprint, a pure transition lookup and a completion boundary.
The executor consumes only `Validated` definitions and connects the completion
boundary to trusted verification and durable storage.

The JSON Schema is published as `workflow` through `riauth schema workflow` and
`GET /api/schema/workflow`, alongside the existing schemas. Platform management
accepts configured definitions in desired-state manifests. The W02 executor
selects active definitions from `config.toml`, not this persisted authoring store.

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
when the account has TOTP. Definitions cannot relax these rules. Stored receipts
alone cannot authorize `enrolled` or `recovered` completion. Supported adapters
also supply a private in-memory capability from the real verifier: existing-passkey
authorized passkey enrollment, mail-proven password recovery, and invitation
first-password or first-passkey enrollment. The completion writer binds evidence
at epoch E to the actual credential mutation at E+1, rechecks live authority, and
commits revocation, receipt consumption and the final run state together.
Credential mutations without such an adapter remain blocked.

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
requires password and local-factor proofs. Accounts without TOTP retain the shipped password-only
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
receipts and complete the run in one transaction. Revision two allows three
attempts per verifier within ten minutes; exhausted TOTP leads to recovery code,
and exhausted recovery denies. Revision-one runs retain their original path.

### Active configured local verifier paths

Platform can load an operator-authored definition in `config.toml` and start it
with `POST /api/workflows/configured/{workflow}` using a live bearer token. The
entry must be active and canonical. The supported shapes have one
`verify_password` step, `verify_password` followed by `verify_totp`, the same
password/TOTP path with a recovery-code fallback, or one `verify_passkey` step.
The only factor fallback routes TOTP's failed signal to `verify_recovery_code`;
success requires the corresponding verifier receipts. The model validates the
revision, fingerprint, proof floor, run duration, step timeout and
retry/execution limits.
None of these shapes has a configured source or custom-stage dependency. The
password-only shape is:

```toml
[workflows.local-password]
active = true

[workflows.local-password.definition]
format = "riauth.workflow/v1"
id = "local-password"
revision = 1
category = "authentication"
origin = "configured"
entry = "password"

[workflows.local-password.definition.limits]
max_duration_seconds = 600
max_executions = 3

[[workflows.local-password.definition.steps]]
id = "password"
action = { type = "verify_password" }
max_attempts = 3
timeout_seconds = 120
cancellable = true
transitions = [
  { on = "verified", to = "success" },
  { on = "failed", to = "denied" },
]

[[workflows.local-password.definition.terminals]]
id = "success"
outcome = "authenticated"
requires = [["password"]]

[[workflows.local-password.definition.terminals]]
id = "denied"
outcome = "denied"
requires = []
```

For an account with enrolled TOTP, set `max_executions = 6`, change the
password step's `verified` target to `totp`, add this step before the terminals,
and set the success terminal's `requires = [["password", "totp"]]`:

```toml
[[workflows.local-password.definition.steps]]
id = "totp"
action = { type = "verify_totp" }
max_attempts = 3
timeout_seconds = 120
cancellable = true
transitions = [
  { on = "verified", to = "success" },
  { on = "failed", to = "denied" },
]
```

The TOTP step uses `POST /api/workflows/{id}/totp/start` to reserve a bound
attempt handle, then `POST /api/workflows/{id}/totp` to submit its challenge and
code. Each retry needs a new handle. The existing verifier consumes a successful
TOTP time step once across workflows and ordinary sign-in; both receipts and
finalization commit together. The two-step shape has no recovery-code fallback.

To allow a recovery code as the alternative factor, set `max_executions = 9`,
route the TOTP step's `failed` signal to `recovery-code`, add one
`verify_recovery_code` step with `verified → success` and `failed → denied`, and
set the terminal to
`requires = [["password", "totp"], ["password", "recovery_code"]]`.
The user chooses recovery with the current TOTP handle at
`POST /api/workflows/{id}/recovery-code/start`, as described below. This retires
that TOTP attempt without proof; only a verified recovery code can complete the
fallback path. Both factors use the same account lockout and one-time consumption
as ordinary sign-in. Their receipts and finalization commit in one transaction.

For a passkey-only definition, use one `verify_passkey` step and set the
authenticated terminal to `requires = [["passkey"]]`. The existing
`POST /api/workflows/{id}/passkey/start` and `/passkey` endpoints reserve and
consume its user-verified WebAuthn ceremony. The verifier binds the ceremony to
the run, attempt, account, session, request and definition; a response from a
different run or session cannot complete it. The passkey step can satisfy an
account's enrolled TOTP assurance floor. It also issues no session or code.

`POST /api/workflows/{id}/password` executes the verifier.
`GET /api/workflows/{id}` resumes and applies deadlines, and
`POST /api/workflows/{id}/cancel` closes a cancellable run. The existing writer pins
the exact definition, account, epoch, session and request, and consumes its
password receipt with terminal completion. A wrong password consumes one bounded
attempt and the account-wide lockout budget. A cancelled or expired attempt
cannot later complete. An inactive entry cannot start a run; a started run keeps
its pinned definition across server restarts. The password-only shape refuses
accounts with enrolled TOTP; both password/MFA shapes require it. The passkey
shape requires an enrolled passkey. All four are reauthentication only: they issue
neither a new session nor a downstream OIDC code.

## Downstream OIDC completion

On Platform, prepare a terminal authorization with its existing live bearer
session, review the request, and send the complete `Authorization` JSON, including
`transaction_id`, `decision: "approve"`, and `prompt: "login"` or `max_age: 0`, to
`POST /api/workflows/authorization` for the canonical password/MFA chain,
`POST /api/workflows/authorization/passkey` for a user-verified passkey, or
`POST /api/workflows/authorization/sources/{source}` for upstream reauthentication.
Each starts with the authorization already pinned, before any verifier challenge.
Requiring request-bound freshness also prevents ordinary approval from leaving a
reusable transaction before the workflow reserves it. It does not attach an older
reauthentication result or infer consent from verification.

The reservation covers the full OIDC request hash (client, redirect, PKCE, nonce,
scopes, resource, claims and request references), current client configuration,
prepared transaction, exact account/epoch/session, workflow request, run and
definition revision/fingerprint. Live checks reject request substitution,
consumed or expired references, client changes, revocation and stale receipts.
Ordinary OIDC completion rejects a reserved request even with a fresh session
or without its transaction id. Cancelled and completed reservations retain a
replay tombstone for the existing seven-day workflow retention window.

Successful verification uses the existing OIDC policy, assurance, claims and code
issuer. Factor consumption, all proof receipts, the prepared transaction,
PAR/signed-request consumption, code issuance and final run state share one
transaction. Policy failure rolls them all back. The final view's
`authorization_response` holds the existing issuer's callback URL; resuming the
same owned run returns that response without issuing another code.

The authorization grant gets only this run's verified primary authentication time
and assurance: `pwd`, `webauthn mfa`, or `federated` with `mfa` only when the
pinned source verifier trusts that assurance. Local factors add `otp` or
`recovery_code`. Upstream MFA never skips an enrolled local factor in the source
workflow. Raw upstream AMR and the bearer's previous assurance are not inherited.
Source grants retain the exact verified source fingerprint and account link.
SAML grants additionally require the bearer's existing source, NameID and
SessionIndex association to match the assertion; new or changed SAML session
associations remain unsupported. The stored bearer session is unchanged, so
workflow success does not open factor-management privileges. These entry points
require an account-bound prepared transaction and explicit approval; silent
requests, account selection and embedded-source clients are rejected. Browser
integration and arbitrary configured workflow selection remain unconnected.
Essentials sign-in and OIDC behavior are unchanged.

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

## Passkey enrollment finalization

On Platform, `POST /api/workflows/passkey-enrollment` starts the shipped
enrollment journey for an existing passkey owner and records the exact live
session proof. Complete its existing `/passkey/start` and `/passkey` steps with
user verification, then send `{"name":"Added key"}` to
`POST /api/workflows/{id}/passkey-enrollment/start`. This returns registration
options; submit the signed credential as `response` to
`POST /api/workflows/{id}/passkey-enrollment`.

The registration ceremony stays server-side and binds account, session, workflow
request, run/version, step/attempt, nonce and account epoch E. Ordinary enrollment
finish/cancel cannot use it. Starting and finishing recheck live authority and
the exact unspent session/passkey receipts, with a 120-second receipt lifetime.
Neither a recovered bearer nor a stored success assertion substitutes for the
existing passkey's real UV proof. The stored bearer's assurance is never upgraded.

The shared registration verifier returns a private in-memory capability. Only
the completion writer can use it after checking the full path and all receipts
at E. Registration consumption, the credential write, account epoch E to E+1,
existing logout/revocation effects, consumed receipts and final run state share
one transaction. A completion rejection rolls all of those writes back; invalid
WebAuthn responses spend their reserved attempt and deny without changing the
account. Concurrent completion can commit only once. The durable run records
the credential and epoch transition; its successful view reports `enrolled`
and `credential_epoch`. Previous sessions cannot resume or reuse the result.
Sign in again after enrollment.

This session-bound adapter supports adding a passkey after verification of an
existing passkey. Invitation first-passkey enrollment uses the separate mail-bound
adapter below. First-passkey enrollment through a password/TOTP session and
other credential mutations still require their own bound adapters. Persisted
mutation receipts alone cannot enable those success outcomes.

## Password recovery finalization

On Platform, the existing explicit browser/CLI reset submission executes the
shipped password-reset workflow inside its writer transaction. The real mail
verifier pins the proof digest, reset purpose, intended immutable account ID,
verified email, current account epoch and exact `account_latest` recovery request.
Factor or credential changes that advance the epoch invalidate an older link;
a replaced request or proof retargeted to another account cannot complete.
Assisted recovery also pins the current support-exposure record and requires
the address verified before the support change to match the mail proof.

Identification creates no proof. The verified reset-mail receipt and password
mutation receipt bind to the same request, run/version, account and epoch, with
no authenticated session. Only the in-memory mail-verifier capability can
authorize the password mutation. Password history policy, proof retirement,
receipt consumption, E-to-E+1, existing logout/revocation effects and the final
`recovered` state commit together. The epoch is written before revocation is
queued. Failure rolls back the entire transaction; concurrent submissions yield
one success and the existing used-link error for the loser.

This preserves scanner-safe GETs, explicit reset POSTs, the existing JSON response
and password-policy retry behavior. No session, staged login, authentication
success or authorization code is created. Ordinary reset retains enrolled
factors, which remain required at the next sign-in. Assisted recovery retains
the accepted policy of removing exposed passkeys, TOTP and recovery codes and
returning `factors_reset: true`; that removal and clearing the support-exposure
record commit with the same epoch transition. Essentials keeps its ordinary
reset path and also rejects a proof whose current recovery-request index no
longer matches.

## Invitation password enrollment

On Platform, the existing browser/CLI invitation acceptance enrolls a first
password through the server-owned `platform-invitation-password-enrollment`
workflow. Both editions share the invitation authority and initial-credential
writer. Only the existing mail verifier can create the in-memory mutation
capability; the caller cannot select an account, workflow signal, or receipt.

The capability pins the exact invitation proof, current request index, immutable
account ID, email, epoch, pending reservation and M04 support-exposure record.
Completion rechecks that the account is still disabled, unverified and without
credentials, that the username still resolves to that account, and that no
directory manages its password. The existing management writer revalidates the
inviter's live permissions and group assignments. Reissued, revoked, expired,
retargeted or stale-epoch invitations cannot activate an account.

The invitation and enrollment receipts bind to the request, account, run/version,
step/attempt and epoch with no authenticated session. Password policy, group
membership, activation, password storage, E-to-E+1, proof/index/reservation
retirement, revocation, evidence consumption and `enrolled` completion share one
transaction. The new epoch is stored before revocation is queued. A rejected
completion rolls back everything, and competing submissions consume the link
only once. Invitation acceptance preserves any M04 exposure marker; it cannot
replace independent recovery or clear that privilege boundary.

Scanner-safe GET/HEAD, explicit POST, used-link errors and the existing
`completed`/`login_required` response remain unchanged. No login, session, cookie
or OIDC code is minted. Essentials uses the same invitation checks and atomic
credential writer without persisting Platform workflow runs or receipts.

## Invitation passkey enrollment

Both editions expose the [invitation passkey API](passkeys.md#first-passkey-from-an-invitation).
`POST /api/account/accept/passkey/start` binds a WebAuthn registration to the
invitation token and a separate, server-generated ceremony. The matching
`/finish` requires both handles and a real authenticator response; `/cancel`
requires both handles. Neither accepts a caller-selected account or session.
The verifier checks challenge, origin, RP ID and user verification. Starting again
replaces the previous ceremony; invalid authenticator responses spend only their
correctly bound registration attempt. Cancellation leaves the invitation available
for another start. Each ceremony lasts at most two minutes.

The same invitation authority and credential writer described above recheck the
account, email, epoch, pending reservation, exposure marker and inviter's live
user/group permissions. Invitation revocation, reissue or acceptance invalidates
the pending registration. The passkey write, account activation, group membership,
E-to-E+1, proof/index/reservation retirement and revocation commit atomically.
M04 exposure and independent human elevation provenance remain protected.

On Platform this adapter completes the shipped `essentials-invitation` definition.
Its invitation and enrollment receipts bind to the immutable account at epoch E,
run/version and a request containing the digests of both the invitation token and
the registration ceremony, with no authenticated session. Receipt consumption and
`enrolled` completion share the credential writer transaction. Competing submissions
produce one success and one used-link error; replay cannot create a second
credential or run. Essentials enforces the same credential transaction without
workflow records. Success returns `{"completed":true,"login_required":true}`;
the recipient signs in separately. The existing browser invitation page still
offers password acceptance; the passkey endpoints are for authenticator clients.

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
`platform-source-totp-reauthentication` definition. It requires source evidence
and TOTP or recovery-code evidence. The original source-only definition remains available for its
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
Epoch changes invalidate pending verification.

## Recovery-code fallback

Revision two of both canonical MFA definitions permits a one-time recovery code
after a fresh password or source proof. It does not authorize factor replacement,
factor removal, password reset, or a session-assurance upgrade. Existing
factor-management restrictions still apply to the original session.

To choose recovery while TOTP is active, first obtain its current handle from
`POST /api/workflows/{id}/totp/start`, then send
`{"totp_challenge":"<handle>"}` to
`POST /api/workflows/{id}/recovery-code/start`. This retires that exact TOTP
attempt without producing evidence and returns a new `workflow`, `challenge`
and `expires_at`. Once TOTP attempts are exhausted, or to retry recovery, send
`{}` to the recovery start endpoint. Selection never clears failed-code counts.

`POST /api/workflows/{id}/recovery-code` accepts the recovery `challenge` and
`code`. Its handle has a separate factor binding and pins the exact account,
epoch, live session, request, run, definition, primary receipt, step and attempt.
Neither an old TOTP handle nor a recovery handle from another attempt can be
substituted. Recovery inherits the primary receipt's freshness and signed expiry;
switching factors cannot extend either deadline.

The verifier is shared with ordinary password sign-in and removes the digest
from the account's live recovery-code set. Code removal, primary and recovery
receipt consumption, lockout clearing, audit and completion share one write
transaction. A completion error rolls these changes back. A wrong or already-used code
spends an attempt and debits the same account lockout as password/TOTP sign-in;
a locked account cannot use an otherwise valid recovery code. Rotation takes
effect immediately, and no plaintext recovery code is persisted by the workflow.

## Bounds

Loops are rejected, including self-loops; retry is expressed only by
`max_attempts` (1–5). Every node must be reachable. The longest path, weighted by
attempts, must fit `limits.max_executions` (at most 64). Runs last at most
86,400 s, steps at most 3,600 s (custom stages 30 s, 4,096 output bytes), with at
most 32 steps, 8 terminals, 12 transitions per step and 64 KiB per document.

## Essentials and Platform

Platform administration has a **Workflows** section with authentication,
enrollment, recovery, consent, and sensitive-action starters. It edits existing
configured definitions in all five categories, including steps, permitted built-in
actions, attempt and time limits, entry, and route destinations. The consent
starter requires a resumed session before a consent decision; the sensitive-action
starter requires a session and password proof, with a TOTP route when enrolled.
The editor shows the canonical `riauth.workflow/v1` JSON and uses the existing
server model and manifest planner to validate the entire definition before saving.
Its graph preview is static: it never executes credentials, verifiers, or sessions.
Existing conditional routes and action fields remain in the canonical draft,
but the browser editor does not yet author new conditions, custom stages, or
source references. Changing an action rebuilds that step's signal routes, so
authors should review its destinations before validating.

The same definition can be supplied in a desired-state `riauth/v1` manifest's
`workflows` array. `POST /api/state/plan`, apply, and export use the shared
plan, revision, audit, and permission boundary. The browser routes delegate to
that path for one definition at a time. Definitions require Platform and
`workflow.write` for the exact `workflow/<id>` target; export and browser list
require `workflow.read`. A changed definition must increase its revision.
Omitting a workflow leaves it unchanged. Persisted configured definitions are
authoring data; the W02 configured executor uses active definitions from
`config.toml` for its supported password and Password→TOTP paths.
Essentials does not show workflow authoring and rejects configured definitions.

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
  session. Password/MFA, passkey and source/MFA chains consume a prepared terminal
  OIDC request atomically; browser integration and changing a SAML session's
  logout association remain unconnected.
  No workflow issues a session. Endpoint parity has not been checked.
* Extending atomic credential-mutation finalization beyond existing-passkey
  authorized passkey enrollment, mail-proven password reset and invitation
  first-password/first-passkey enrollment: session-authorized first-passkey
  enrollment through password/TOTP, TOTP enrollment and other initial credential
  paths remain blocked pending their real adapters.
  A denial after an epoch change also remains blocked by current-facts binding;
  W02 must resolve such runs with its expiry/cancellation or mutation protocol.
* Configured enrollment, recovery, other authentication chains,
  custom stage execution, and the other built-in verifiers. The configured local
  verifier paths store attempt timing, enforce retry and run bounds, cancellation
  and expiry, and recheck account, session, request and receipt authority in the
  final transaction. The password paths reject upstream-only accounts.
* Binding runs to their security dependencies. `RunBinding` covers the
  definition's ID, revision and fingerprint. It does not cover the `Environment`
  (stage registrations and permissions) or any approval record
  that RI-WF-002 requires.
  The source reauthentication request now pins and rechecks its source fingerprint
  and the receipt's explicit account link; broader dependency/approval binding remains.
* End-to-end invariant and race tests for the remaining verifier integrations
  across both durable backends. The shared
  [`invitation_passkey_bound_competing_completion` contract](../tests/contracts/shared.rs)
  exercises real WebAuthn registration, wrong-account and superseded-request
  rejection, competing completions, consumed bound receipts and replay on plaintext
  and encrypted redb/PostgreSQL. Its concurrent callers run within one process;
  it does not establish multi-process or failover behavior. The focused source
  regression exercises the signed OIDC callback, binding and authority changes,
  rollback on stale evidence, and
  competing completion writers on the local store. The focused passkey regression
  uses signed WebAuthn credentials to exercise session/request/run/attempt
  isolation, revoked authority, retries and competing completion writers.
  The focused TOTP regression uses signed upstream evidence and real enrolled
  codes to exercise bound retries, cancellation, stale/consumed source evidence,
  account-wide replay, epoch invalidation and atomic competing completion.
  The password-plus-TOTP regression checks both real verifiers, retained lockout
  across correct passwords and cancelled runs, proof substitution/expiry,
  ordinary sign-in replay rejection and competing finalization writers.
  The recovery-code contract exercises both real primary chains, handle
  substitution, cancellation, revoked/expired authority, competing consumption,
  shared sign-in replay/lockout and unchanged factor-management restrictions.
  The OIDC contract checks request/client/run/version/session isolation, replay,
  revoked and expired authority, rollback on a live membership-policy denial,
  concurrent finalization and real one-time authorization-code redemption.
  The passkey/source OIDC contract additionally checks actual UV/trusted-source
  assurance, rejection of untrusted upstream MFA, mandatory local factors and
  source-link revocation, with complete transaction rollback snapshots.
  Those other workflow executor paths still lack shared PostgreSQL evidence.
* Management API, desired-state, storage, versioned approval, editor, templates,
  and product capability reporting or gating.
