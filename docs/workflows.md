# Workflow definition model

Status: **W01 model, W03 proof provenance, bounded W02 verifier paths, a fail-closed reviewed pin for configured runs, W06 Platform authoring, and a held W07 extension contract.**
The Platform server persists bounded runs, attempts, requests and evidence, and exposes
password, passkey and OIDC/SAML source reauthentication, plus request-bound
configured OIDC consent, for a live bearer session.
Password and source paths require TOTP or a one-time recovery code when enrolled. All use the existing
verifiers and finalize through the W03 store boundary. These canonical chains
can also complete an explicitly approved downstream OIDC request. No path
issues a new session. Connected credential mutations cover passkey enrollment
authorized by a fresh existing-passkey, local-password or current-TOTP proof,
passkey removal, TOTP enrollment authorized by a fresh existing-passkey proof
or fresh local-password proof for a password-only account, TOTP replacement
authorized by an existing-passkey proof or both fresh password and current-TOTP
proofs for a local account without a passkey, mail-proven password recovery, and
first-password or first-passkey invitation acceptance. Their real verifiers
finalize the credential, proof consumption and epoch change atomically.
Ordinary sign-in, other enrollment and source-stage paths are unchanged;
browser recovery and invitation acceptance
keep their existing responses and require a separate sign-in.
The W04 Platform conditional application policy now narrows existing client
authorization and projects scoped claims from verified session signals; see
[OIDC profiles](oidc-profiles.md#platform-conditional-application-policy).
That policy does not itself select or execute workflow definitions.
A run loaded from an active `config.toml` workflow stores the reviewed revision,
fingerprint, and policy digest. A later policy change, a disabled account, or a
rolled-back revision seals that open run with no new evidence and no grant.
Restoring the previous policy starts a new run and leaves the sealed run denied.
Platform now starts four active configured authentication shapes: local password
alone for an account without TOTP, password followed by enrolled local TOTP,
password with a TOTP or recovery-code choice, or one user-verified passkey step
for an account with an enrolled passkey.
Platform also supports the three exact configured consent shapes described below.
It supports eight configured enrollment shapes: a live session and fresh verified
existing passkey followed by passkey registration, a new TOTP secret or TOTP
replacement; or a password-only account's live session and fresh local-password
proof followed by its first passkey or TOTP secret; or a local TOTP account
without a passkey using its current TOTP to enroll a first passkey, or fresh
password and current-TOTP proofs to replace TOTP; or a linked upstream-only
account using a fresh source assertion to enroll its first passkey.
It also supports one configured recovery shape: explicit reset-mail verification
and password reset in the same transaction, plus two exact configured sensitive
action paths for passkey removal: a live session with fresh verified passkey
proof, or an MFA session with fresh local password and current TOTP proofs.
The W02/W03 workflow proof receipts remain bound to their account,
session, request and run, and this client policy cannot produce a workflow proof
or success outcome.
Source reauthentication uses server-defined workflows; source-first passkey
enrollment accepts only the exact configured definition described below.
Other configured enrollment and recovery shapes and other authentication
chains or consent shapes remain unconnected. Custom stages are not part of
those executor paths.

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
  `verify_source`, `request_consent`, `enroll_credential`, `replace_totp`,
  `remove_passkey`,
  `reset_password` and,
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
`invitation`, `source`, `consent`, `enrolled`, `password_reset` and
`passkey_removed`. A proof's
required account, request and run bindings are declared by its type
(`Proof::binding`), not a definition field. W03 makes completion use stored,
typed evidence rather than a caller-supplied list of proof kinds. Each evidence
record names the built-in action and step attempt that produced it, its
verification and expiry times, consumption state, and the account, account
epoch, request and run to which it belongs. Its optional session must match
the run's exactly; `session`, `consent` and `passkey_removed` proofs require one. A custom stage
can route by its outputs but never
produces a proof, cannot reuse built-in signal names and only receives
permissions that its registration grants. The held host repeats that rule
for a direct call. Configuration and the executor do not reach it.

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
| `enrolled` | `enrolled`. `enroll_credential` needs `session` plus fresh `password`, `passkey` or `source`; current `totp` may authorize passkey enrollment only. `invitation` may authorize a first passkey or password only. `replace_totp` needs `session` plus `passkey`, or `session`, `password` and `totp` |
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
alone cannot authorize enrollment, recovery or passkey-removal completion.
Supported adapters supply a private in-memory capability from the real verifier:
existing-passkey authorized enrollment, fresh UV passkey removal, mail-proven
password recovery, and invitation first-password or first-passkey enrollment.
The completion writer binds evidence at epoch E to the actual credential
mutation at E+1, rechecks live authority, and commits revocation, receipt
consumption and the final run state together. A definition's static completed
transition cannot supply that capability.

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

## Configured OIDC consent

Platform accepts three exact active configured `consent` graphs. The existing
`session` (`resume_session`) → `consent` (`request_consent`) graph requires
`[["session", "consent"]]`, allows one attempt per step and lasts at most 120
seconds. It handles requests that need no reauthentication. The new
`session` → `passkey` (`verify_passkey`) → `consent` graph requires
`[["session", "passkey", "consent"]]` with proof age at most 120 seconds. It
handles requests that need reauthentication, such as `prompt=login` or
`max_age=0`, for accounts with an enrolled passkey. Its run lasts at most 120
seconds with at most five executions; the passkey step allows up to three
attempts and 120 seconds, and consent allows one attempt and 120 seconds.
The local TOTP graph is `session` → `password` (`verify_password`) → `totp`
(`verify_totp`) → `consent`. It requires
`[["session", "password", "totp", "consent"]]` with proof age at most 120
seconds. It needs an enrolled TOTP factor and fresh local password proof before
the current TOTP code; TOTP alone cannot refresh the OIDC authentication time.
The run lasts at most 120 seconds with at most eight executions, up to three
password and three TOTP attempts, and one consent attempt. Recovery codes are
not an alternative in this graph. All routes are unconditional, and no graph
can reach approval from a static transition. The live session, prepared
transaction or request reference can expire sooner. The generic
`POST /api/workflows/configured/{workflow}` does
not start consent. The built-in example above has different limits and is not
any configured path.

Prepare an OIDC `Authorization` for the same live bearer session, review its
client and scopes, then send the complete JSON with `transaction_id` and no
`decision` to `POST /api/workflows/configured/{workflow}/consent`. It rejects
the wrong reauthentication graph, silent or account-selection prompts, browser
bindings and embedded source stages. For the passkey graph, call
`POST /api/workflows/{id}/passkey/start` and then
`POST /api/workflows/{id}/passkey` with the signed WebAuthn assertion before
approving. For the TOTP graph, call `POST /api/workflows/{id}/password`, then
`POST /api/workflows/{id}/totp/start` and
`POST /api/workflows/{id}/totp` with its returned one-attempt challenge and a
current code. The account's spent TOTP step is updated when verification
succeeds, even if the user later denies consent. With the same bearer, send
`{"approve":true}` or `{"approve":false}` to
`POST /api/workflows/{id}/consent`. Explicit denial is also accepted while the
passkey, password or TOTP step is active and invalidates any pending handle.
Failed verifications consume bounded attempts; they never create a consent
proof.
`GET /api/workflows/{id}` resumes the run;
`POST /api/workflows/{id}/cancel` cancels only an active, cancellable step. The
final view's
`authorization_response` contains the existing issuer's code callback on
approval or denial callback without a code.

The executor pins the account and epoch, live session, OIDC request hash and
client, prepared transaction, workflow request, run and definition. It emits
session proof from the live session; only a verified WebAuthn assertion emits
passkey proof, only verified local password and current TOTP code emit their
respective proofs, and only explicit approval emits consent proof. Fresh
assurance applies only to the resulting OIDC grant, never to the stored bearer
session. The TOTP grant uses the fresh password proof's authentication time and
the existing password plus OTP assurance. Final approval marks the exact
prepared transaction reauthenticated and consumes it and the bound receipts
in the same write as
the existing issuer's code response. Denial spends that transaction and returns
the normal denial callback without a code. Ordinary authorization cannot use
that reserved transaction. An ordinary decision with its transaction ID spends
that exact preparation and prevents its later consent reservation. An ordinary
no-ID decision with a bearer requires an ID while that account has a bound
preparation; another account's bound preparation does not block it. For live
anonymous preparations, the issuer records the exact transaction digests passed
over by the deciding account. Later use of one of those IDs by that account
conflicts, while another account can still decide it. A later admitted
preparation remains usable. Expiry and cancellation discard only the pending
transaction; completed and closed runs retain replay protection for that
transaction. No new session or remembered consent grant is created. Other
configured consent graphs, including TOTP-only and recovery-code consent
reauthentication, are unsupported.

Platform can opt browser OIDC interactions for all clients into the exact
session-only graph by setting top-level
`browser_consent_workflow = "local-consent"` in the server configuration,
where the name identifies one active canonical `session` → `consent` definition,
the exact `session` → `passkey` → `consent` reauthentication definition, or the
exact `session` → `password` → `totp` → `consent` definition described above.
Configuration validation rejects a missing, inactive, or different graph;
Essentials rejects the setting. For this selected browser path, the server
prepares one cryptorandom OIDC transaction bound to the browser interaction ID
and the account already signed in, if any. The HttpOnly SSO cookie, interaction
binding cookie, live account/session, prepared transaction, request content,
client policy, and explicit page decision are rechecked before approval. The
session proof, required reauthentication proofs, and approval receipt are consumed
with code issuance in one write;
the finished run is retained for replay checks. A signed-out Deny spends the
same exact preparation and returns the normal denial callback without a code.
The original browser delivers either callback once. Terminal user-code approval
is unavailable for this selected path.

The browser page always asks for a decision for a selected configured request,
including when the client has implicit or previously remembered consent. Its
Remember control is hidden, and a submitted `remember` value creates no grant.
The session-only path refuses embedded source stages, account selection, prompt
login, `max_age=0`, or a session that still needs reauthentication. The passkey
path allows an OIDC request that needs reauthentication, including `prompt=login`
or `max_age=0`, after the browser has a live SSO session. Its interaction page
uses `/oauth/resume/{id}/passkey/start`, `/passkey/finish`, and `/passkey/cancel`
to operate a durable run-bound UV WebAuthn ceremony. A fresh proof advances the
run to the separate Allow/Deny decision; it does not upgrade the stored session
or issue a code. Approval consumes that proof with the exact prepared transaction
and client-policy check. Denial before or after passkey proof spends the same
request. The run and browser session expire independently within the configured
120-second maximum; a canceled ceremony spends one bounded attempt. A missing,
expired, or changed graph or session fails closed without ordinary-consent
fallback. The global browser selector is exact: choosing the passkey graph makes
requests that do not need reauthentication unavailable through this adapter.
The password/TOTP graph also requires reauthentication and a live browser SSO
session. Its page sends the password and current authenticator code in separate
submissions to `POST /oauth/resume/{id}/password`; the durable run moves from
password to TOTP to the explicit Allow/Deny decision. Each verifier checks the
HttpOnly SSO cookie, interaction binding, exact prepared transaction, client,
account, session and run. TOTP spends its current step transactionally and cannot
be replaced by a recovery code. Bounded failures or an explicit Deny close the
run and spend the prepared request; approval consumes both fresh proofs without
upgrading the stored session or issuing a new one. Browser reloads resume the
current stage, and a failed or changed authority has no ordinary-consent
fallback. The selected graph rejects requests that do not need reauthentication.
SAML configured consent and remembered-consent creation remain unsupported.

Standard terminal OIDC preparations, outside embedded source stages, admit at
most 64 live indexed attempts per request hash. At capacity, a new preparation
atomically retires one still-unclaimed anonymous attempt, selected by earliest
expiry, before admission. Account-bound and authenticated-session-bound attempts
are never displaced. A retired ID cannot authorize or deny; its holder must
prepare again. If every slot is claimed, admission returns a conflict. Each
attempt otherwise expires after 600 seconds. Sustained anonymous traffic can
still displace another signed-out caller's preparation before sign-in, so this
bounded admission rule does not guarantee completion during a flood.

Saturating the index cannot block a live bearer's direct no-ID decision for the
same static URL. Signed-out no-ID denial still requires an ID while any matching
preparation is live, because it has no account to bind a replay fence to.
Legacy upgrade overflow remains a conservative no-ID fence until its live rows
are gone.

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

The reservation is keyed by the cryptorandom prepared transaction digest and
checks the full OIDC request hash (client, redirect, PKCE, nonce, scopes,
resource, claims and request references), current client configuration,
exact account/epoch/session, workflow request, run and
definition revision/fingerprint. Live checks reject request substitution,
consumed or expired references, client changes, revocation and stale receipts.
Ordinary OIDC completion rejects a reserved transaction even with a fresh
session. A no-ID decision requires the ID of that account's live bound
preparation; the anonymous preparation replay rule above also applies.
Cancelled and completed transactions retain
token-specific replay state for the existing seven-day workflow retention
window; a fresh preparation of identical request content remains usable.

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

These session-bound adapters support adding a passkey after verification of an
existing passkey, a fresh local-password proof for a password-only account, or
a fresh current-TOTP proof for a local TOTP account as described below.
Invitation first-passkey enrollment uses the separate mail-bound adapter below.
Other credential mutations still require their own bound adapters. Persisted
mutation receipts alone cannot enable those success outcomes.

An active configured definition can start the same complete server path at
`POST /api/workflows/configured/{workflow}`. It must have exactly the
`session → passkey → enroll` steps (`resume_session`, `verify_passkey`, then
`enroll_credential: passkey`), unconditional verified/completed routes in that
order, failed routes to denial, and one enrolled terminal requiring session,
passkey and enrolled proofs. The run lasts at most 1,200 seconds, with at most
16 executions, passkey retries bounded to three, a 120-second maximum success
proof age, and cancellable steps. The existing `/passkey/start`, `/passkey`,
`/passkey-enrollment/start` and `/passkey-enrollment` run endpoints perform the
real ceremonies. The definition and request remain pinned across restart;
cancellation or expiry discards a pending ceremony, and completion uses the
same atomic epoch change and session revocation described above. Config,
start and resume reject other configured passkey enrollment shapes except the
three exact first-passkey paths below.

### Password-only first-passkey enrollment

Platform also accepts the exact configured `session → password → enroll` path
for a local account with no existing passkey, TOTP factor or pending TOTP
enrollment. The live session and request-bound password receipt precede a
workflow-owned WebAuthn registration. Success requires session, password and
enrolled proofs; the run lasts at most 600 seconds, allows at most eight
executions and three password attempts, and limits success proofs to 120
seconds. All steps are cancellable, with failed routes to denial.

Start at `POST /api/workflows/configured/{workflow}`, verify the password at
`POST /api/workflows/{id}/password`, then use the existing
`/passkey-enrollment/start` and `/passkey-enrollment` endpoints. The ceremony
is bound to the account, session, request, run and attempt; ordinary passkey
finish cannot consume it. Cancellation or expiry discards it. Final
registration, epoch change, revocation, audit, receipt consumption and run
completion share one writer. The bearer gains no MFA assurance and no session
is issued. This path does not accept an invitation proof or an upstream-only
account.

### Current-TOTP first-passkey enrollment

Platform accepts the exact configured `session → totp → enroll` path for a
local account with an enrolled TOTP factor, no passkey and no pending TOTP
change. A live MFA bearer and a fresh session receipt are bound to the run;
the current TOTP code is then spent once through the existing account-wide
verifier. Recovery codes cannot replace that step. Only afterward can the
same run start a WebAuthn registration through `/passkey-enrollment/start`
and confirm it through `/passkey-enrollment`.

Success requires session, TOTP and enrolled receipts, each at most 120 seconds
old. The run lasts at most 600 seconds with eight executions, at most three
TOTP attempts and cancellable steps. The workflow owns the WebAuthn ceremony;
ordinary registration cannot finish it, and cancellation or expiry removes
it. No passkey is stored before the final writer atomically stores the signed
credential, advances the account epoch, queues revocation, audits, consumes
receipts and finalizes the run. The existing TOTP and recovery codes remain;
the bearer gains no new assurance and no session is issued.

### Linked-source first-passkey enrollment

Platform accepts the exact configured `session → source → enroll` path for an
enabled account with no local or directory password, passkey, TOTP factor or
pending TOTP change. Start with the account's live bearer at
`POST /api/workflows/configured/{workflow}/source-passkey`. The bearer must
come from the same enabled OIDC or SAML source and account link named by the
definition. The response supplies a workflow view and upstream authorization
URL. Its signed callback is consumed at `POST /api/workflows/{id}/source` with
the original bearer; ordinary source completion cannot consume the reservation.
The fresh assertion must resolve to that same link. A source or link change
blocks completion.

Only after source verification may the same run use
`/api/workflows/{id}/passkey-enrollment/start` and
`/api/workflows/{id}/passkey-enrollment` for signed WebAuthn registration.
Success requires session, source and enrolled receipts no older than 120
seconds. The run lasts at most 600 seconds, with at most eight executions, one
source attempt, 300-second source and registration steps, and cancellable
steps. Cancellation or expiry discards the
upstream reservation or pending registration. The final writer atomically
stores the credential, advances the epoch, queues revocation, audits, consumes
the receipts and finalizes the run. It issues no session or consent grant and
does not change the stored bearer's assurance. Essentials rejects this
configured shape. An invitation cannot use this session-bound path.

New Platform source-verifier runs pin the selected source ID and live
registration fingerprint in `RunBinding`, in addition to the definition
fingerprint. The source reservation and evidence carry that binding. At verifier
use and credential finalization, the executor compares it with the request's
source pin and rechecks the enabled live registration. A changed registration
cannot finish the run. Cancellation still requires a live owner session; a
registration change can invalidate a source-backed bearer. Run expiry and
cleanup remain bounded. In-flight runs created before this field existed
retain the prior request-pin and live source checks until their 600-second limit.

## Configured TOTP enrollment

Platform accepts one exact configured `enrollment` shape for an account that
already has a passkey and has no TOTP enrollment in progress. It uses
`session → passkey → enroll` (`resume_session`, `verify_passkey`, then
`enroll_credential: totp`), unconditional success routes, failed routes to
denial, and a terminal requiring session, passkey and enrolled proofs. The run
lasts at most 600 seconds with eight executions; the enrollment step has one
attempt and a 120-second deadline. All steps are cancellable. Config, start
and resume reject other passkey-authorized TOTP shapes; Essentials excludes
configured enrollment.

Start with `POST /api/workflows/configured/{workflow}` and a live bearer, then
complete `/api/workflows/{id}/passkey/start` and `/api/workflows/{id}/passkey`
with a signed user-verified assertion.
`POST /api/workflows/{id}/totp-enrollment/start` returns the new secret and
`otpauth_uri` once. Submit `{"code":"..."}` to
`POST /api/workflows/{id}/totp-enrollment` with the same bearer to confirm. Resume
returns progress only. The run owns the secret: ordinary TOTP confirmation
cannot consume it, and cancellation, denial or expiry discards it.

The final writer rechecks the live account, session, request, run, fresh
passkey receipt and new TOTP code. It installs the secret, spends the confirming
time step, advances the account epoch, queues revocation, audits and consumes
the workflow receipts in the same transaction that finalizes the run. No new
session or recovery codes are issued. This shape does not replace an existing
TOTP app or enroll the first authenticator for an account without a passkey.

### Password-only TOTP enrollment

Platform also accepts the exact configured `session → password → enroll` shape
for a local account with no passkey, TOTP factor or ordinary TOTP enrollment
pending. Its actions are `resume_session`, `verify_password` and
`enroll_credential: totp`. Success requires session, password and enrolled
proofs; verified/completed transitions are unconditional and failures lead to
denial. The same 600-second run, eight-execution, one-attempt enrollment,
120-second enrollment deadline and proof-age limits apply. The password step
allows at most three attempts within 300 seconds, and all steps are
cancellable. Config, start and resume reject other shapes and Essentials
excludes configured enrollment.

Start with `POST /api/workflows/configured/{workflow}` using a live bearer,
verify the local password at `POST /api/workflows/{id}/password`, then use the
same `/totp-enrollment/start` and `/totp-enrollment` endpoints above. The
password attempt uses the ordinary account lockout and yields a receipt bound
to that account, session, request and run. It does not raise the bearer's MFA
assurance or authorize an application request. The final writer rechecks the
live local account and unspent password receipt with the new code, then
installs the factor, spends the confirming time step, advances the epoch,
queues revocation, audits, consumes receipts and finalizes in one transaction.
It issues no session or recovery codes. Cancellation, denial or expiry clears
the run-owned secret without changing the account's factors.

## Configured TOTP replacement

Platform accepts one separate exact `enrollment` shape for an account with an
existing TOTP factor and passkey. It has the same limits, proof requirements,
transitions and cancellable `session → passkey → enroll` steps as configured
TOTP enrollment, except the final action is `replace_totp`. Config, start and
resume reject other replacement shapes; Essentials excludes this action.

Start with `POST /api/workflows/configured/{workflow}` and a live bearer, then
complete the run's signed UV passkey endpoints. Call
`POST /api/workflows/{id}/totp-replacement/start` to receive the run-owned new
secret and `otpauth_uri` once. Confirm a new code with `{"code":"..."}` at
`POST /api/workflows/{id}/totp-replacement`. Resume reveals progress, never
the secret. The old TOTP factor and recovery codes remain valid while the run
is pending; cancellation, denial and expiry discard the new secret.

The final writer rechecks the bound account, live session, request, run, fresh
passkey receipt and new code. In one transaction it replaces the secret,
records the confirming time step as spent, clears old recovery codes, advances
the account epoch, queues revocation, audits the replacement, consumes receipts
and finalizes the run. It issues no session or new recovery codes. The
enrollment endpoint cannot confirm a replacement run, and the replacement
endpoint cannot confirm an enrollment run.

For a local account with TOTP and no passkey, Platform also accepts the exact
`session → password → totp → enroll` replacement shape from a live MFA bearer.
It requires a fresh password receipt and a separately consumed code from the
**current** TOTP factor in the same account, session, request and run before
`/totp-replacement/start` generates a new secret. Success requires session,
password, TOTP and enrolled receipts; each step is cancellable, exhausted
attempts deny, the run lasts at most 600 seconds and the final proof age is at
most 120 seconds. Recovery codes cannot substitute for the current TOTP step. The
same replacement endpoints confirm a new code and atomically rotate the
factor, spent step, recovery codes, account epoch, revocations and audit. The
old factor and recovery codes remain until final commit; resume never reveals
the new secret and no session assurance or session is issued. Other TOTP
replacement shapes remain unavailable.

## Configured passkey removal

Platform accepts two exact `sensitive_action` definitions. The first has
`session → passkey → remove`: `resume_session`, `verify_passkey`, then
`remove_passkey`. The second has `session → password → totp → remove` with
`verify_password` and `verify_totp` in place of the passkey verifier. Each
successful step routes unconditionally to the next, every failed step routes
to denial, and the success terminal requires exactly the session, selected
factor proofs and passkey-removed proof with a 120-second maximum proof age.
Runs last at most 600 seconds and allow at most eight executions. Session and
remove steps get one attempt each, factor steps at most three; all are
cancellable. Config, start and resume reject other configured removal shapes.

`POST /api/workflows/configured/{workflow}/passkey-removal` takes a bearer
token and `{"credential_id":"..."}`. The server pins that existing credential
to the account, session, request, run and definition before any challenge.
For the passkey path, use `POST /api/workflows/{id}/passkey/start` and
`POST /api/workflows/{id}/passkey` for a fresh signed user-verified WebAuthn
assertion. For the password and TOTP path, submit the password at
`POST /api/workflows/{id}/password`, reserve the current TOTP attempt at
`POST /api/workflows/{id}/totp/start`, then submit its challenge and code at
`POST /api/workflows/{id}/totp`. This path requires an existing MFA bearer
session, a local password and an enrolled TOTP factor; a recovery code cannot
replace the current code. Both paths commit through
`POST /api/workflows/{id}/passkey-removal` with the same bearer. The final
endpoint accepts no target ID or caller signal. Ordinary configured start
cannot enter either path.

The final writer rechecks the enabled account, live session, exact target
owner, unspent proofs, expiry and an independent local password or enough
other passkeys to avoid
removing the last usable authenticator. It deletes the credential, advances the
account epoch, queues session revocation, audits removal, consumes proofs and
finalizes the run atomically. Cancellation and expiry cannot commit removal;
replay cannot repeat it. The password and TOTP path does not raise the bearer
session's assurance. No new session is issued. Essentials does not accept
either configured definition. Directory-managed passwords are not treated as an
independent local recovery route for this guard.

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

Platform can select an active exact configured reset definition at
`POST /api/workflows/configured/{workflow}/password-reset` with a reset mail
`token` and new `password`. The definition has only `identify → email → reset`
steps (`identify`, `verify_email: reset`, `reset_password`), unconditional
completed/verified routes, failure routes to denial, and a recovered terminal
requiring reset-email and password-reset proofs no older than 120 seconds.
It allows at most four executions and a 2,400-second run bound. The mail token
chooses the account and current recovery request; no bearer session is required
or created. The existing mail verifier, password policy and atomic mutation
writer perform all three steps in one submission. A replaced, retired or expired
mail proof cannot start the run, and a failed password policy check rolls it
back for retry. Completion consumes the proof once and revokes old sessions.
The anonymous submission shares the account rate bucket (10 per minute by
default). A read-only proof check precedes password hashing; the writer checks
the proof again before consuming it.
There is no separately cancellable active run between requests; replacing or
retiring the mail proof cancels its authority before submission. The existing
browser/CLI reset endpoints continue to use the shipped workflow, and
Essentials does not expose this configured endpoint.

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
the recipient signs in separately. The browser invitation page still accepts a
password, and when WebAuthn is available it also offers this passkey enrollment
through the same three endpoints. That choice creates no session.

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
Its graph and validation plan are static previews: they never execute credentials,
verifiers, or sessions. Saving a definition does not activate it or make an unsupported
shape executable; the server selects only supported shapes from its runtime configuration.
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
`config.toml` for its four supported authentication paths. Applying a definition
in the authoring store does not activate it or make arbitrary graphs executable.
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

## Controlled extensions

Platform compiles the capability `workflow.controlled_extensions`. A running
instance reports that name configured and usable when one active workflow
validates as the extension-then-password shape below and an admitted manifest
covers its stage. A default configuration, an inactive workflow, an unsupported
graph, or a rejected or non-covering manifest reports the name not configured
and not usable. This build rejects an attempt to disable that name. Essentials
does not compile the capability, so configured and usable stay false there.
A custom stage with no checked manifest still fails configuration as an unknown
stage. The one executable shape is the Wasmi guest documented below.

[`workflow::extension::Host`](../src/workflow/extension.rs) is a held in-process
contract for a native registrant. It is not an isolation boundary and it is not
on the configured path. Registration names the stage, the
permissions it may grant (`read_profile`, `read_groups`, `read_request`,
`network`), and the resource caps. A call runs only when the host was opened
enabled on a Platform binary, the definition is a configured Platform workflow,
and every permission the step requests was granted. The view copies only those
requested identifiers. Each identifier is at most 64 bytes and uses lowercase
letters, digits, `.`, `_`, and `-`. Groups are capped at 32. Input, output, and
network response budgets are capped at 4,096 bytes. A stage timeout cannot
exceed 30 seconds, and the effective deadline is the tighter of the step and
the registration. At most 16 stages are registered and at most 4 calls run at
once.

`network` requires an allowlist of 1–8 exact hostnames and budgets of at most
4 requests and 4,096 response bytes. `localhost`, `.local`, numeric addresses,
wildcards, ports, and uppercase names are rejected. `fetch` returns a permit
for that declared size or a denial. It does not open a socket or return a body.
A step that does not request `network` is denied even when the registration
has an allowlist.

The accepted result is one declared routing label. Output bytes are measured
against the tighter cap and then dropped, so they cannot become evidence,
claims, or a stored audit payload. `Action::proof` is empty for that label.
Reserved signal names are rejected again even if a caller bypassed validation.
A result that arrives after the deadline is discarded. The worker is not
preempted; it occupies one in-flight slot until it returns. A panic in the
stage becomes a failure rather than a label.

This host does not load scripts, WebAssembly, or dynamic libraries. A native
registrant shares the server process: it can call any API it can name, open a
socket, and keep running after the host's deadline discards the label. Safe
Rust in this crate cannot preempt that thread or interpose on its syscalls.
`unsafe_code = forbid`. Platform links Wasmi for the gate below, not for this
host.

## Isolated guest

Platform links Wasmi 0.40 through `dep:wasmi` on the `platform` feature.
Essentials does not link it. [`extension_gate::execute`](../src/workflow/extension_gate.rs)
returns `external_runtime_required` on Essentials, and `RUNTIME_LINKED` is true
only on Platform. The executor calls this gate for one configured shape. It
does not call [`Host::invoke`](../src/workflow/extension.rs).

`config.toml` may contain `workflow_extensions`, at most 16 JSON manifests
keyed by stage id. A non-empty map on Essentials is rejected because it
requires the Platform build. On Platform, `check` must succeed and the map key
must equal the manifest's stage. The only graph that then runs is custom, then
local password:

* origin `configured`, category `authentication`, entry `extension`
* the custom step id is `extension`, outputs are exactly `allow` and `block`,
  and it does not request `network`
* `allow` goes to `password`, and both `block` and `failed` go to `denied`
* the password step id is `password`; `verified` goes to `success` and
  `failed` goes to `denied`
* `success` requires exactly `[["password"]]`; `denied` requires nothing
* the custom step allows one attempt; the password step allows 1–3
* run duration is 1–600 seconds and `max_executions` is 1–8, at least the
  worst path

Any other custom graph still has no executable adapter. A custom `allow` that
goes straight to success remains an implicit success and is rejected. The guest
label is not a proof. `Action::proof` stays empty for it, and success still
stores a password receipt.

`check` accepts one JSON manifest, `riauth.workflow-extension/v1`, of at most
128 KiB, with no unknown fields:

| Field | Bound |
| --- | --- |
| `stage` | Workflow id grammar, equal to the configuration key |
| `outputs` | 1–8 distinct labels, excluding `verified`, `failed`, `completed`, `granted`, `denied` |
| `permissions` | Distinct `read_profile`, `read_groups`, `read_request`. `network` is rejected |
| `network` | The string `deny` |
| `fuel` | 1–10,000 Wasmi fuel units |
| `memory_bytes` | Exactly 65,536, one WebAssembly page |
| `max_input_bytes` | 1–4,096 projected identifier bytes |
| `max_output_bytes` | 1–4,096. The host copies at most 32 bytes, the maximum label length |
| `timeout_seconds` | 1–30, as an instruction budget, not a wall-clock preemption |
| `module_base64` | Standard base64, 1 byte through 65,536 |
| `module_sha256` | 64 lowercase hex characters of the decoded module |

The hash is compared before the module is admitted. A mismatch is `integrity`.
Uppercase hex is `malformed`. Platform then parses the bytes with Wasmi and
rejects any other shape. Essentials stops after the hash and the numeric
bounds, so it does not parse the module. `Checked` keeps the admitted bytes so
execution uses the hashed image, and its `Debug` output does not print them.
`execute` admits that image again before it runs.

The guest exports `memory` and `route () -> i32` and has no imports. There is
no WASI, filesystem, socket, clock, randomness, or host callback. The engine
disables floats, multi-memory, bulk memory, reference types, tail calls, and
saturating float-to-int, and ignores custom sections. Translation is lazy:
`Module::new` validates a body that fits the translation budget, and the first
call builds Wasmi IR. The first `execute` reuses the module from `check`. A
later `execute` parses the bytes again. The structural check rejects a
non-empty import section, a start section, any
table, global, or element, and any memory other than one page with minimum and
maximum both 1. It also rejects more than one function, any type other than
`() -> i32`, more than two exports, more than 32 i32 locals, a non-i32 local,
or a data segment. Those caps are decided before `Module::new`, so a 64 KiB
module cannot ask Wasmi to compile thousands of functions or a million locals.
The store allows one instance and one memory, no tables, caps that memory at
65,536 bytes, and traps if it grows. At instantiation Wasmi calls
`ResourceLimiter::memory_growing` with current 0, desired 65,536 and maximum
65,536. `StoreLimits` allows that initial size because the desired size is not
greater than `memory_size`, and its `ByteBuffer` then builds a `Vec<u8>` of
length 65,536. A further page makes `memory_growing` return an error.
`trap_on_grow_failure` turns that error into `GrowthOperationLimited`, which
this gate reports as `failed`. `memory.grow` therefore ends the step as
`failed` instead of returning a label. `ResourceLimiter` does not account for
the interpreter value stack.

That stack is reserved when `route` is called. The guest sets Wasmi
`StackLimits` so the initial height and the maximum height are both 64
`UntypedVal` slots. `UntypedVal` is 8 bytes, so `ValueStack::new` reserves at
least 512 bytes. `extend_by` checks the live length before `Vec::reserve` and
returns `StackOverflow` when `additional >= 64 - len`. The host reports that
trap as `limit`. On an empty stack a compiled frame of 64 registers is refused
and a frame of 63 is accepted. In this pinned Wasmi 0.40.0, 32 i32 locals and
a constant result need 34 slots, so that module still runs. Sixty-three live
`i32.load` results need 65 slots and are refused before the buffer grows. A
frame that fits still runs on the caller until it returns or spends its fuel.
The same `StackLimits` value keeps the call depth at 16 frames.

Wasmi 0.40.0 cannot preempt that one function. Its `Config` has no epoch or
interrupt setting. `Store::call_hook` runs only when the host calls Wasm or
Wasm calls a host function. `call_resumable` pauses only when a host function
returns an error. This guest has no imports, so a loop in `route` does not
return to the host until the call traps or finishes. Fuel's base cost is 1.
Before the instance starts, the host installs
`min(manifest fuel, timeout_seconds × 1,000)`. Wasmi 0.40.0 charges 7 fuel
for each byte of the function body and subtracts that charge before it builds
IR. Admission applies that charge to
`min(manifest fuel, manifest timeout × 1,000)` and refuses a larger body
before `Module::new`. At the 10,000 fuel cap that limit is 1,428 body bytes.
The refusal is `timeout` when the timeout budget is strictly smaller than the
manifest fuel, and `fuel` otherwise. `check` validates a body that fits, once.
The first `execute` reuses that module. A later `execute` parses the bytes
again, so each call pays the translation charge. A tighter step timeout can
still exhaust on the first call and skips translation when the charge exceeds
the fuel in the store. A charge that fits is translated entirely on the
caller, then `route` runs until it returns or spends the fuel that remains.
Validation of a body that fits is not fuel-metered. None of these steps
preempts the caller at a wall-clock deadline. `route.call` holds `&mut Store`
until it returns, so another thread cannot drain that store's fuel. Validation
and translation run on the same caller. A detached guest thread would still be
running after the caller continued, and stopping it requires `unsafe`, which
this crate forbids. When the two budgets are equal,
or fuel is smaller,
execution exhaustion is `fuel`. A timeout of 10 seconds or more cannot be
tighter than the 10,000 fuel cap, so that execution exhaustion is `fuel`.

`route` writes its label at offset 0 and returns the length. A workflow label
is at most 32 bytes. The host reads guest memory only when that length equals
one declared label and is within `min(step max_output_bytes, manifest
max_output_bytes, 32)`. A negative length, or a length above that cap, is
`output` and is not read. A length inside the cap that equals no declared
label, including a prefix, is `undeclared_output` and is not read. The host
zeroizes the copy it did read. Bytes after the returned length are not read.
A manifest cap of 4,096 stays valid and does not raise the 32-byte copy.

The host writes input only at offset 4,096, inside a 4,096-byte window: a
little-endian record count, then `kind`, `length`, and bytes. Kinds are account
(1), request (2), client (3), and group (4). The executor copies a field only
when the step requested that permission and the manifest granted it.
`read_profile` is the account id. `read_groups` is at most 32 group names.
`read_request` is the workflow request id, and the live session id in the
client slot, because this start is a bearer session and has no OAuth client.
The token, password, email, display name, and password hash are not copied.
Each identifier is 1–64 characters of lowercase letters, digits, `.`, `_`, and
`-`, with no empty, leading, trailing, or repeated dots. The projected bytes
must fit `max_input_bytes` and the window. A missing required fact is
`input_limit`, and the guest does not run. A withheld permission is absent, not
an empty placeholder.

A gate denial becomes the built-in `failed` signal and routes to `denied`. It
does not become an attacker-chosen label. The run binding stores
`extension_sha256`, the lowercase hex of the module that started the run, with
the definition id, revision, and fingerprint. Resume, cancel, and password
verification compare the live manifest with that hash. A different module seals
the open run with `failed` and no evidence, under the stored hash, and does not
execute the new bytes for the old run. The next start uses the new module.
Rewriting both a stored definition and its fingerprint remains the existing
store trust model; this hash stops a configuration-side module swap.

Other custom graphs, other verifiers, and the Q02 engine adapter are still out
of scope. The capability is configured and usable for this active shape when
the admitted manifest covers the stage. Default, inactive, unsupported, and
Essentials configurations leave configured and usable false.

## Reviewed configuration pin

A start loaded from an active `config.workflows` entry, whose validated
definition is that entry, stores a reviewed pin beside `RunBinding`. The pin
holds the definition revision, the definition fingerprint, and a policy digest.
The digest is the SHA-256 hex of `riauth.workflow-reviewed/v1`, the active flag
(`true` or `false`), the workflow id, the revision, and the fingerprint, each
on its own line. `RunBinding` equality remains the definition id, revision,
fingerprint, optional source registration, and optional `extension_sha256`.

`workflow_reviewed` keeps the highest adopted pin for that workflow id and is
retained across restore, so a restored older definition cannot start below it.
The first start writes it. A higher live revision replaces it when that start
commits. The same revision, fingerprint, and policy is reused. The same
revision with a different fingerprint or policy is rejected with
`Workflow policy changed` and leaves the stored pin unchanged. A live revision
below the stored pin is rejected with `Workflow version was rolled back`.

Open pinned runs are listed in `workflow_account_runs`, keyed by account, with
at most 32 run ids. Clearing `workflow_active_sessions` for that run removes
the id. The account index is invalidated on restore. Disabling a user seals
every indexed open pinned run in the same user write.

While a pinned run is still active, the executor seals it before a verifier can
reserve or complete a step. Browser OIDC password, TOTP, and passkey consent
do that in a write that commits before the factor, ceremony, or decision
write. A grant write that returns an error rolls its own transaction back, so
the denial has to be committed first. The seal finishes at the snapshot's
denied terminal when that terminal exists, and as cancelled otherwise. It
writes no evidence and issues no grant. A browser continuation after that
seal adds no consent completion and leaves the authorization code unissued.
The stored failure is `policy_changed`
(`Workflow policy changed`) when the snapshot disagrees with the pin, the
configured entry is missing or inactive, the live revision is higher, the live
fingerprint or policy digest differs, the retained pin is missing or differs,
the request row is missing, or a bound authorization or consent client
fingerprint no longer matches. Reopening the store keeps the retained pin, and
a denial recorded before the reopen is still denied. It is `rolled_back`
(`Workflow version was rolled back`) when the live
revision is below the pin or below the retained pin. It is `user_disabled`
(`Workflow account is disabled`) when the user row is missing or disabled.
A disabled account fails the old bearer before a later resume, so the seal is
committed by the user update itself. A higher revision seals the old run as
`policy_changed` and can start a different run. Publishing a revision at or
above the retained pin can start a new run. The sealed run stays denied if the
old policy returns or the account is enabled again.

Code-owned revisions stay unpinned. That includes the shipped password, passkey,
source reauthentication, invitation, and password-reset workflows. The pin is
recorded only for configured start, configured passkey removal, configured
consent, browser OIDC consent starts, and configured source-first passkey
enrollment. The executor still
selects active definitions from `config.toml`, not from the persisted
`workflow_definitions` store.

This executor matches `roadmap/w02-configured-executor-wave15` at
`06b9793a943d2d0ab15c8990a009449f346adc33`: `RuntimeRun` and
`start_authorization_workflow` live in `src/workflow/executor.rs`, and
`workflow_password` lives in `src/workflow/executor/password.rs`. The separate
`roadmap/w02-runtime-executor` worktree at
`c03d1f8e831694819edf0adb02d129ea8ca62da9` is a smaller password-only executor.
Its `RuntimeRun` is `deny_unknown_fields` in `src/workflow/executor.rs`, its
only public methods are `workflow_start`, `workflow_resume`, `workflow_cancel`,
and `workflow_password` (all in that file), and it has no
`start_authorization_workflow`. Its password start is the code-owned essentials
workflow and stays unpinned. Those worktrees were not edited. They do not gain
this guard until this branch is merged.

A later W02 merge keeps `reviewed` and `reviewed_failure` optional, with serde
default and omission when absent, so an unpinned row stays readable by an older
`deny_unknown_fields` reader. A pinned row carries both fields, and that older
reader rejects it until it grows the same fields. Finalization that clears
`workflow_active_sessions` also drops the account index; a finalizer that
writes a final state without `close` or `clear_active_session` drops the index
itself. A new verifier entry point that loads a run by id calls
`reject_stale_reviewed` before reserving or completing a step, and commits a
seal with `Ok(Err)` or with an earlier write that returns `Ok`. Resume and
cancel call `reviewed_outcome` and return the sealed view. On the
runtime-executor tree, `close` untracks the account index if this slice is
ported, and `workflow_password`, `workflow_resume`, and `workflow_cancel` use
the same guards. Client fingerprints remain that tree's client-policy pins;
this slice also seals when those fingerprints change.

## Left to later work

These are not implemented or established by this slice:

* Connecting the remaining verifier actions and existing OIDC/browser sign-in,
  lifecycle, browser/remembered consent and embedded source-stage paths. Current
  W02 endpoints cover password, passkey and OIDC/SAML source reauthentication,
  plus the exact configured OIDC consent path, for a live bearer session.
  Password/MFA, passkey and source/MFA chains consume a prepared terminal OIDC
  request atomically; browser integration and changing a SAML session's logout
  association remain unconnected.
  No workflow issues a session. Endpoint parity has not been checked.
* Extending atomic credential-mutation finalization beyond existing-passkey
  authorized passkey enrollment and removal, password-only and current-TOTP
  first-passkey enrollment, existing-passkey or password-only TOTP enrollment,
  existing-passkey or password-and-current-TOTP replacement, mail-proven
  password reset, invitation first-password/first-passkey enrollment, and
  linked-source first-passkey enrollment: other upstream-only and first-passkey
  paths, other TOTP replacement and other initial credential paths remain
  blocked pending their real adapters.
  A denial after an epoch change also remains blocked by current-facts binding;
  W02 must resolve such runs with its expiry/cancellation or mutation protocol.
* Other configured enrollment, recovery, authentication or consent chains,
  and the other built-in verifiers. The configured local verifier paths store
  attempt timing, enforce retry and run bounds, cancellation and expiry, and
  recheck account, session, request and receipt authority in the final
  transaction. The password paths reject upstream-only accounts. One configured
  custom stage can run, and only as the Wasmi guest above: `extension` then
  local password, with the guest label choosing `password` or `denied`. The
  held native host stays unwired. Other custom graphs stay closed.
* Binding the remaining security dependencies. `RunBinding` covers the
  definition's ID, revision and fingerprint. New source-verifier runs also pin
  the live source registration, and a guest run pins `extension_sha256` of the
  module that started it. Configured runs loaded from `config.workflows` store
  a reviewed pin beside that binding, as described above. The rest of the
  `Environment`, and the approval record RI-WF-002 requires, remain unbound.
  The source receipt separately pins its explicit account link. Safe resume of
  changed content, code-owned revisions outside `config.workflows`, and broader
  dependency binding remain.
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
* Management API as the executor's selection source, desired-state and the
  persisted `workflow_definitions` store as that source, versioned multi-party
  approval and safe resume, the editor and templates, extensions beyond the
  existing guest gate, and product capability reporting or gating. The executor
  still reads active definitions from `config.toml`.
