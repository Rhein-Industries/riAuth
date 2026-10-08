# Shared security invariants and test contracts

This is the Q01 catalog for the source revision and evidence labels in the
[threat model](threat-model.md). Invariant IDs are stable: retain an ID when
enforcement moves, and add an ID when a distinct contract is introduced. A
contract is required policy; its evidence paragraph states how much of it is
observed today. Existing tests below were read, not run. Q02/Q05 recipes are
future authorized work, not implemented tests or claims of passing coverage.

## Coverage map

| Domain | Stable IDs | Current evidence boundary |
| --- | --- | --- |
| Accounts | RI-ACC-001–002 | Stable identity/link checks and administrator protections exist. |
| Credentials and recovery | RI-CRED-001–003 | Factor/recovery checks exist; full browser self-service is not established. |
| Sessions, proofs, consumption and revocation | RI-SES-001–005 | Browser, bearer, grant and replay paths exist; expiry must remain distinct from revocation. |
| Authorization and protocol binding | RI-AUTH-001–002 | Shared online checks and protocol-specific binding exist. |
| Agents, management approval, retries and audit | RI-MGT-001–006 | Current management core exists; exact secret-byte review, cached-result authority and future multi-party review need explicit contracts. |
| Connectors, offboarding and signals | RI-CON-001–004 | Local transactions and reviewed remote work exist. A disable commits per-target deactivation intent, which is delivered later under a scoped controller. |
| Workflows | RI-WF-001–002 | Embedded source stages exist; general configurable workflows are intended policy. |
| Proxies | RI-PROXY-001–002 | Forward auth and embedded proxy exist; deployment header trust remains an external dependency. |
| Devices and certificates | RI-DEV-001–003 | Server protocols exist; real device agents/peers are not established here. |
| Storage, secret custody, backups and rollback | RI-STORE-001–004 | Store/backup checks and restored-state invalidation with an operator-attested serving gate exist; no external freshness witness. |
| Shared core and compiled/runtime boundaries | RI-DIST-001–003 | Distribution separation and transitions remain intended policy. |

There are 35 IDs. The named regressions provide partial evidence, not a count of
fully proven invariants. TB-01–TB-11 and G-01–G-06 in the threat model map attack
inputs and gaps to these contracts.

## Accounts

### RI-ACC-001: stable identity and explicit account binding

**Contract.** Account authority is bound to immutable local identity, canonical
issuer, and explicit external identity links. Username, email, display name or a
caller-supplied subject must not substitute for another account. Shared builds,
rehashing, migration and restore preserve the supported issuer/subject contract.

**Observed enforcement.** [Core](../../src/core.rs) `open` checks stored issuer;
[claims](../../src/claims.rs) `subject`/`validate_user` enforce subject mappings;
[state](../../src/state.rs) `reconcile` rejects mismatched user IDs;
[sources](../../src/source.rs) `link_key`/`validate_identity` bind source,
issuer/subject link and local user. Directory binding is separately checked.

**Existing regressions.** [Operations](../../tests/identity/operations.rs)
`storage_survives_restart_and_issuer_cannot_be_changed_silently` and
`claim_mappings_scope_policies_and_subject_import_are_enforced`;
[sources](../../tests/identity/sources.rs)
`oauth_only_sources_use_pinned_userinfo_and_do_not_invent_oidc_assurance_or_link_by_email`.

**Missing coverage / later contract.** Q02-C01 substitutes another user ID,
same-email external account, issuer, sector and subject in each relevant adapter;
expect rejection without identity replacement. Q05-R03 races unlink/source
change with authentication. Cross-build continuity is pending RI-DIST-001.

### RI-ACC-002: no implicit administrator authority or last-admin loss

**Contract.** Ordinary authentication, parent ownership and agent permissions do
not confer human administrator/delegation authority. An administrator-owned
agent keeps its approved list as its ceiling and every agent restriction. Remote mutations must not
create, edit or enroll credentials for an administrator through a scoped-agent
path, or remove the last enabled administrator. Offline administrator recovery
is a separate operator-controlled capability.

**Observed enforcement.** [Core](../../src/core.rs) `admin`, `update_user`,
`ensure_remaining_admin`, and `recover_admin`; [state](../../src/state.rs)
`reconcile`; [agents](../../src/agent.rs) `create_agent`/`rotate_agent`;
[devices](../../src/windows_login.rs) `windows_device_enroll`;
[offboarding](../../src/offboarding.rs) `guard_schedule`/`apply_local`.

**Existing regressions.** [Management](../../tests/identity/operations.rs)
`agent_credentials_enforce_action_resource_and_identity_boundaries`;
[parents](../../tests/agent_parent.rs) `parent_user_ownership_constrains_agents`;
[owner authority](../../tests/agent_authority.rs)
`administrator_ownership_keeps_the_approved_list_as_the_ceiling`;
[devices](../../tests/windows_login.rs) `agent_cannot_enroll_an_administrator`;
[offboarding](../../tests/offboarding.rs)
`agent_without_permission_is_forbidden_and_last_admin_is_protected`.

**Missing coverage / later contract.** Q02-C04 enumerates every administrator
target and delegation path, including future browser administration. Q05-R03
promotes the target or disables another administrator between review and apply;
the committing writer must enforce the current protection.

## Credentials and recovery

### RI-CRED-001: credential verification and factor replay state are shared

**Contract.** A credential is accepted only by its verifier under the account's
current state. TOTP steps and recovery codes cannot authenticate twice; passkeys
require verified WebAuthn challenge/origin/RP/user verification and the stored
credential/counter rules. Transparent password rehash preserves identity and
does not masquerade as a credential replacement.

**Observed enforcement.** [Core](../../src/core.rs) `password_login` uses dummy
hashes, lockout, TOTP replay state and recovery-code digests; nested results commit
spent factors even after a rejected transaction. [Passkeys](../../src/passkey.rs)
`browser_passkey_finish` verifies current credential ownership and increasing
nonzero counters (zero-counter credentials remain supported).
[Crypto](../../src/crypto.rs) and [history](../../src/identity/password_history.rs)
handle hashing, imported hashes, upgrades and reuse.

**Existing regressions.** [Factors](../../tests/identity/factors.rs)
`totp_requires_confirmation_prevents_replay_and_satisfies_policy`,
`recovery_codes_are_single_use_and_password_change_revokes_old_sessions`,
`passkeys_require_user_verification_origin_nonce_and_live_counter_and_revoke_cleanly`;
[signin](../../tests/signin_core.rs)
`transaction_rejection_after_valid_totp_commits_the_step` and
`argon2id_non_default_parameters_are_rehashed_on_login`;
[history](../../tests/password_history.rs) `reuse_is_rejected_on_every_password_write`.

**Missing coverage / later contract.** Q02-C02 compares browser/CLI/directory/
device credential entrypoints and generic browser failures. Q05-R02 races the
same step/code/assertion and credential rotation; exactly one allowed use commits,
with no session from a stale credential or rejected request binding.

### RI-CRED-002: sensitive credential changes require fresh sufficient proof

**Contract.** Factor changes require the account's own live session, recent
authentication and existing-factor sufficiency. Password change verifies the
current credential again. A terminal-approved shared browser must establish its
own sign-in before managing passkeys. Request freshness/step-up must never be
inferred solely from another interaction's recent session.

**Observed enforcement.** [Core](../../src/core.rs) `mfa_begin`,
`recovery_codes`, `change_password`, `require_factor_session`;
[passkeys](../../src/passkey.rs) `passkey_register_start_in`,
`passkey_register_finish_in`, `passkey_remove_in`; [portal](../../src/portal.rs)
factor paths. Freshness is 300 seconds for the named begin/remove paths.
Enrollment finish instead binds its stored pending ceremony/session/epoch and
expiry; do not claim every finish repeats the same begin-time freshness check.

**Existing regressions.** [Portal](../../tests/portal.rs)
`portal_passkey_registration_requires_fresh_mfa_rule_and_signs_out`,
`portal_passkey_remove_requires_fresh_mfa_session`,
`terminal_shared_browser_signs_in_before_changing_passkeys`;
[factors](../../tests/identity/factors.rs) `passkey_removal_requires_mfa_session`.

**Missing coverage / later contract.** Q02-C02 tests just-before/at/after the
freshness/ceremony deadline, wrong account/session and missing existing factor
for begin and finish separately. Q05-R02 rotates/removes a factor after begin and
before finish. Terminal zero-`auth_time` behavior is explicitly scoped in RI-WF-001.

### RI-CRED-003: recovery proofs bind purpose, account, email and epoch

**Contract.** Mail proofs are expiring, single-use, purpose-specific and bound
to current account/email/epoch. Reset preserves enrolled factors and revokes old
authority. Invitation acceptance requires live creator authority for the user and
each group and cannot replace an existing identity. Recovery requests do not
reveal whether an eligible account exists.

**Observed enforcement.** [Lifecycle](../../src/lifecycle.rs) `enqueue`
supersedes a prior purpose proof; `account_reset_request` has a generic result;
`account_complete` revalidates account and `creator`, deletes the proof and updates
epoch/groups/audit atomically. [Core](../../src/core.rs) `recover_admin` is an offline operator path, not a
mail proof or remote-agent grant.

**Existing regressions.** [Factors](../../tests/identity/factors.rs)
`account_verification_is_delivered_through_smtp_and_bound_to_purpose_email_and_epoch`,
`email_password_reset_preserves_factors_revokes_grants_and_retries_delivery_without_exposing_tokens`,
`invitations_require_live_scoped_authority_are_single_use_and_cannot_replace_existing_users`.

**Missing coverage / later contract.** Q02-C02 includes every recovery writer
and future browser completion; assert factors survive reset, old sessions/grants
fail, and no account enumeration. Q05-R01/R03 race completion/supersession/email
change/creator or parent disable; one valid completion, no partial groups or user.

## Sessions, proofs and revocation

### RI-SES-001: credential classes and browser custody stay separate

**Contract.** Browser-owned sessions authenticate through protected SSO cookies
and have no bearer-token row. Staged logins, interaction/binding codes, browser
tombstones, agent credentials, Windows tickets and OAuth grants are not
interchangeable with a human session token. An agent's application access
token stands for its owner without any session: it never authenticates,
authorizes or consents as the owner, and only the owner's own session approves it.

**Observed enforcement.** [Signin](../../src/signin.rs) `stage_browser_login`,
`attach_browser_login`, `bearer_backed`; [browser](../../src/browser.rs)
`browser_session` rejects rotated mappings and `sso_cookies`/`browser_cookie`
provide HttpOnly/SameSite cookies and Secure/host prefixes on HTTPS.
[Core](../../src/core.rs) `session` resolves only the session-token digest index.
[CLI transport](../../src/cli/transport.rs) pins saved credentials to the issuer.
[Core](../../src/core.rs) `identity_user` replaces the session check only for the
reserved application identity, with
[agent applications](../../src/management/agent_applications.rs) `validate_live`;
`check_token_grant` keeps that identity on the grant that names its approval, and
[exchange](../../src/assembly/exchange.rs) refuses it as an exchange subject.

**Existing regressions.** [Signin](../../tests/signin_core.rs)
`attach_creates_tokenless_session_without_bearer_row`,
`rotation_tombstone_never_authenticates`, `attach_never_modifies_terminal_backed_sessions`;
[portal](../../tests/portal.rs) `portal_password_sign_in_sets_only_an_httponly_cookie`;
[Windows](../../tests/windows_login.rs) `enroll_returns_the_secret_once_and_login_succeeds`;
[agent application access](../../tests/agent_application_access.rs)
`agents_still_cannot_authorize_consent_or_launder_application_tokens`,
`approval_needs_ownership_a_live_agent_an_opted_in_application_and_fresh_sign_in`.

**Missing coverage / later contract.** Q02-C01 tries every credential class at
all other authentication endpoints and checks raw secrets stay out of browser
storage/views. Q05-R02 races browser attach/account switch and preserves the
correct CLI session while revoking displaced browser-owned authority.

### RI-SES-002: proof binds account, session and exact request instance

**Contract.** Fresh authentication proves a particular request instance for a
particular account/session, not every identical request in another tab. Approval
binds the account shown on the page; callback delivery belongs to the initiating
browser and, for page approval, the approving session.

**Observed enforcement.** [OIDC](../../src/oidc.rs) `Authorization::request_hash`
includes request binding; [signin](../../src/signin.rs) `bind_proof`,
`proof_valid`, `session_ref`; [browser](../../src/browser.rs) `interaction`,
`authorize_attach`, `authorize_decision`, `browser_resume_with`;
[SAML](../../src/saml.rs) interaction/request hashes and `saml_browser_decide`.
Proof use is paired with live identity validation, not a caller's asserted user ID.
[Source](../../src/source.rs) `source_callback` redeems a browser-started upstream
login only when the callback presents the binding cookie set at start. A missing
or different cookie ends that login before any token request, and replaying the
callback with the original cookie does not complete it. CLI, embedded-stage and
workflow logins have no browser cookie. A browser-started SAML assertion is accepted
at the ACS without that cookie, because a cross-site POST does not send it. Finish
stays closed until a same-site return presents both the start cookie and a one-time
cookie set on the ACS response. A return that has the one-time cookie but not the
start cookie ends the login before a session or link is written. CLI, embedded-stage
and workflow SAML logins are not cookie-bound.

**Existing regressions.** [Signin](../../tests/signin_core.rs)
`proof_is_single_use_and_bound_to_request_and_session`;
[browser](../../tests/browser_signin.rs) `proof_cannot_be_moved_between_identical_requests`,
`decision_rejects_changed_session_ref`, `browser_approval_is_delivered_only_to_the_approving_browser`;
[SAML](../../tests/identity/saml.rs) `saml_force_authn_requires_proof_from_interaction`;
[sources](../../tests/identity/sources.rs)
`browser_source_callback_is_redeemed_only_by_the_starting_browser`;
[SAML sources](../../tests/identity/saml_source.rs)
`saml_browser_acs_handoff_checks_success_foreign_browser_missing_cookie_replay_and_cli`.

**Missing coverage / later contract.** Q02-C03 swaps request hash/instance,
account, session, interaction ID, cookie and callback collector independently.
Q05-R01 races proof use, approval/deny and collection; one decision/delivery,
no successful cross-tab or cross-account transplant.

### RI-SES-003: one-time state is consumed at its defined commit boundary

**Contract.** Authorization codes, refresh rotation, device grants, client
assertions, PAR/JAR references, factor ceremonies and recovery proofs have
explicit one-use transitions. A rejected attempt must not undo consumption that
the path commits, or revoke another actor's family merely by guessing a code.

**Observed enforcement.** [OIDC](../../src/oidc.rs) `exchange_code`, `refresh`,
`exchange_device`; [authorization](../../src/authorization.rs)
`validate_reference`/`consume`; [JOSE](../../src/jose.rs) `consume_assertion`;
[passkeys](../../src/passkey.rs) finish paths; [Windows](../../src/windows_login.rs)
`windows_ticket_redeem`; [SAML logout](../../src/saml/logout.rs) `source_logout`
records a request only after its signature verifies and every session index
matches a source session whose fingerprint is current, or a session the same
pinned-certificate write revoked. A session revoked for another reason stays
unmatched after a later trust change, so that signed request stays unrecorded.
Nested `Ok(Err(...))` differs from a transaction error:
the former can commit failure/replay state, the latter rolls the writer back.

**Existing regressions.** [OIDC](../../tests/identity/oidc.rs)
`concurrent_code_redemption_has_exactly_one_winner`,
`refresh_rotation_replay_revokes_the_entire_family`,
`code_replay_revocation_requires_original_client_and_verifier`,
`signed_and_pushed_requests_are_bound_immutable_expiring_and_one_use`;
[factors](../../tests/identity/factors.rs)
`unknown_discoverable_credential_is_rejected_and_ceremony_consumed`;
[SAML logout](../../tests/identity/saml_logout.rs)
`saml_source_logout_after_certificate_rollover_is_one_time_and_replay_spares_the_next_login`,
`saml_source_logout_unrelated_revocation_does_not_consume_old_request_after_trust_replacement`.

**Missing coverage / later contract.** Q02-C03 records each path's success,
client-error consumption, server-error rollback and replay-family effect.
Q05-R01 repeats and concurrently submits each artifact, including across backend
processes; never infer that every rejected proof must be consumed identically.

### RI-SES-004: revocation is live and re-enable never revives dependents

**Contract.** Disabled/deleted accounts, epoch mismatch, revoked sessions or
families, disabled clients and revoked source/credential bindings lose local
online authority. Re-enabling an account/client does not revive previously
revoked dependents. The disable, owned-agent/device revocation and required
notification enqueue share the local transaction.

**Observed enforcement.** [Core](../../src/core.rs) `identity_user_unbound`,
`identity_user`, `user_security_transition`, `revoke_client_grants`;
[store](../../src/store.rs) `put` centralizes enabled-state epoch changes;
[agents](../../src/agent.rs) `authority_active`/`revoke_owned`;
[Windows](../../src/windows_login.rs) `revoke_user`;
[OIDC](../../src/oidc.rs) `validate_grant_local` and exchange-chain validation.

**Existing regressions.** [SSF](../../tests/ssf.rs)
`disable_entry_points_revoke_dependents_and_enqueue_exactly_once`,
`reenabling_legacy_disabled_accounts_never_restores_child_credentials`;
[factors](../../tests/identity/factors.rs) `password_reset_revokes_sessions_and_grants`;
[OIDC](../../tests/identity/oidc.rs)
`token_exchange_requires_bilateral_trust_actor_proof_and_revokes_with_parent`;
[agent application access](../../tests/agent_application_access.rs)
`revoking_the_approval_or_the_agent_ends_outstanding_tokens_at_once`,
`disabling_or_promoting_the_owner_ends_tokens_and_revives_nothing`,
`the_application_policy_and_opt_in_are_checked_on_every_use`,
`expiry_ends_tokens_and_the_approval`.

**Missing coverage / later contract.** Q02-C01/C05 covers all disable/reset/
unlink/import/offboarding writers and every online consumer. Q05-R03 revokes
during issuance, retries and queued jobs, then re-enables; old authority still
fails. Cross-snapshot rollback is a separate RI-STORE-004 gap.

### RI-SES-005: lifetime, offline grant and revocation semantics are explicit

**Contract.** Expired browser/CLI credentials cannot log in. An explicit offline
grant may outlive CLI expiry up to its absolute family lifetime; refresh cannot
extend that lifetime or expand scopes. Revocation/epoch/current policy still
apply online. Offline JWT/RP/device behavior must not be represented as immediate
online revocation.

**Observed enforcement.** [Core](../../src/core.rs) `session` checks bearer
expiry; `identity_user` intentionally does not impose CLI expiry on grants.
[OIDC](../../src/oidc.rs) `new_grant`, `refresh`, `validate_grant_local` check
grant/family lifetime. [Cleanup](../../src/core.rs) retains needed session rows.

**Existing regressions.** [OIDC](../../tests/identity/oidc.rs)
`cli_expiry_does_not_end_an_explicit_offline_grant`,
`refresh_scopes_can_only_shrink_and_expiry_is_absolute`;
[operations](../../tests/identity/operations.rs)
`cleanup_retains_sessions_when_instance_refresh_ttl_drops_after_issuance`.

**Missing coverage / later contract.** Q02-C01/C05 tests independent CLI,
browser, grant, family and device deadlines. Q05-R04 advances time during signing
and maintenance, including changed TTL defaults; assert absolute expiry and
explicit revocation independently. Device trust has an additional live-session
expiry requirement (RI-DEV-001).

## Authorization and protocol binding

### RI-AUTH-001: current authorization is revalidated where authority is used

**Contract.** Prior login, consent, preview or an old claim is insufficient to
authorize a new online operation. Current account/client, groups including live
temporary grants, scope rules, assurance, source and device requirements govern
issuance, refresh, online token use, portal launch and proxy checks. A service
client's own token does not acquire user identity/MFA.

**Observed enforcement.** [Core](../../src/core.rs) `authorize_identity` and
`groups_for`; [claims](../../src/claims.rs) `enforce`;
[assurance](../../src/assurance.rs) `needs_step_up`/`enforce`;
[OIDC](../../src/oidc.rs) `authorization_policy`/`validate_grant_local`;
[portal](../../src/portal.rs), [PAM](../../src/pam.rs), and
[outpost](../../src/outpost.rs) call shared policy.

**Existing regressions.** [Policy](../../tests/identity/policy.rs)
`group_policy_is_checked_again_at_refresh_userinfo_and_proxy`,
`updated_default_acr_rejects_userinfo_introspection_and_proxy_consistently`;
[portal](../../tests/portal.rs) `portal_inherits_live_access_without_admin_bypass_or_information_leaks`;
[PAM](../../tests/pam.rs) `approval_grants_group_policy_until_expiry_or_revocation`.

**Missing coverage / later contract.** Q02-C05 replays one policy mutation
through all online interfaces, including LDAP/RADIUS and future administration.
Q05-R03/R04 removes a group, expires temporary access, changes ACR/device/source
policy during verification. JWT claims at an offline RP have RI-SES-005 limits.

### RI-AUTH-002: protocol bindings and required protections cannot downgrade

**Contract.** Grants bind the authenticated client, registered redirect, S256
PKCE, requested resource/scopes and required proof key. Signed/pushed requests
bind exact supported semantics and expiry. Exchange requires bilateral trust and
the actor/parent chain. An agent's application exchange instead requires the
agent credential alone (client credentials are refused), its owner's live
approval for that exact client and resource, and scopes within the approval; it
issues one access token with `act`, never a login or refresh credential, and no
management permission substitutes for the approval. Unknown or unsupported security settings, proof
algorithms, authentication methods and required protections reject explicitly.

**Observed enforcement.** [OIDC](../../src/oidc.rs) `validate_authorization`,
`authenticate_client`, `exchange_code`, `refresh`;
[provider](../../src/provider.rs) settings/grant/redirect validation;
[authorization](../../src/authorization.rs) reference validation;
[resource](../../src/resource.rs), [DPoP](../../src/dpop.rs),
[JOSE](../../src/jose.rs), [exchange](../../src/exchange.rs),
[agent exchange](../../src/assembly/agent_exchange.rs).

**Existing regressions.** [OIDC](../../tests/identity/oidc.rs)
`redirect_pkce_and_code_replay_are_enforced`,
`private_key_jwt_binds_issuer_subject_audience_and_consumes_assertions_atomically`,
`dpop_binds_code_refresh_resource_and_replay_revocation_to_the_key`,
`resource_indicators_bind_consent_code_refresh_audience_and_online_policy`,
`token_exchange_enforces_target_dpop_binding_for_impersonation_and_delegation`;
[agent application access](../../tests/agent_application_access.rs)
`exchange_issues_only_an_owner_access_token_naming_the_agent`,
`dpop_bound_applications_require_a_proof_for_agent_tokens`,
`other_agents_and_other_owners_agents_get_nothing`,
`management_permissions_confer_no_application_access`.

**Missing coverage / later contract.** Q02-C03 independently changes each
binding/algorithm/method, strips required proof/PAR/JAR/MFA, and tests duplicate
parameters. Q05-R01/R03 races replay, client key/policy rotation and parent
revocation. Build-time omission must also reject (RI-DIST-002/003).

## Agents and management

### RI-MGT-001: management permission is current, explicit and scoped

**Contract.** Every management effect needs the current principal's exact
action/resource permission; a user parent adds no authority. An owned agent
acts with its approved permissions limited to the owner's current authority:
the approved list for a full administrator, otherwise the owner's live
delegated grants plus personal actions on their own account. Issuance refuses
a permission outside that authority and, for a non-administrator owner, any
wildcard; promotion revokes owned agents; every credential use and delayed job
recomputes the limit. Agent rotation preserves scope/owner and invalidates the old
credential. Disabled/expired agents or disabled/deleted parents cannot execute
or delegate, including delayed work.

**Observed enforcement.** [Agents](../../src/agent.rs) `Principal::require`,
`principal`, `live_principal`, `OwnerAuthority::limit`, `management`,
`rotate_agent`; [management](../../src/management.rs) `create_agent`;
[reconciliation](../../src/reconciliation.rs) `scoped_agent`;
[registration](../../src/assembly/registration_runtime.rs) `check_creator`;
[lifecycle](../../src/lifecycle.rs) `creator`;
[provisioning](../../src/provisioning.rs) `actor`;
[offboarding](../../src/offboarding.rs) `authority_still_valid`.
Direct core calls lack an HTTP request context, so HTTP preconditions cannot be
inferred from a successful core-only test (RI-MGT-003).

**Existing regressions.** [Operations](../../tests/identity/operations.rs)
`agent_credentials_enforce_action_resource_and_identity_boundaries`,
`agent_credential_rotation_preserves_permissions_and_invalidates_the_old_token`;
[parents](../../tests/agent_parent.rs) `parent_user_ownership_constrains_agents`;
[owner authority](../../tests/agent_authority.rs)
`delegated_owner_authority_limits_the_agent_on_every_request`,
`existing_owned_agents_are_cut_over_to_the_owner_authority`,
`personal_actions_manage_only_the_named_account`;
[offboarding](../../tests/offboarding.rs)
`execution_revalidates_agent_parent_even_for_a_legacy_enabled_agent`,
`execution_applies_the_owner_authority_held_at_execution`.

**Missing coverage / later contract.** Q02-C04 builds the action/resource and
admin-target matrix with no inferred wildcard or delegation. Q05-R03 changes
principal/parent/target authority between preview, apply, job claim and commit.

### RI-MGT-002: approval binds exact operation and security dependencies

**Contract.** Applying a reviewed change must not silently alter its actor,
issuer, targets, content, credential references/versions, or dependency state.
Revalidate current authority and revision/config/trust dependencies before the
first effect; stale or changed approval requires a new review.

**Observed enforcement.** [State](../../src/state.rs) `plan_state` previews
without commit; `apply_state` compares the full received/stored plan, actor,
issuer, expiry, revision and recomputed changes. Secret references/versions are
reviewed; supplied secret bytes are resolved at apply and are not committed into
the preview hash. [LDAP](../../src/directory.rs) and
[cloud](../../src/cloud_directory.rs) additionally compare upstream snapshot and
config fingerprints; cloud destructive removals need the reviewed plan ID.

**Existing regressions.** [Operations](../../tests/identity/operations.rs)
`agent_plans_are_immutable_atomic_and_idempotent`;
[OIDC](../../tests/identity/oidc.rs)
`stale_plans_and_partial_secret_failures_do_not_change_configuration`;
[cloud](../../tests/cloud_directory.rs)
`malformed_success_pages_fail_plan_and_apply_without_deprovisioning` and
`group_wipe_requires_plan_id_header_on_http_apply`.

**Missing coverage / later contract.** Q02-C04 tampers with every reviewed
field and dependency separately; any rejection leaves no partial change/audit
success. G-04 is a suspected approval gap: choose an explicit secret-content
commitment or documented apply-time secret authority contract before claiming
byte-exact approval. Q05-R05 changes revision/trust/secret-file contents after
preview and races two applies. No change here weakens approval to accommodate it.

### RI-MGT-003: retries recover a result without repeating unauthorized work

**Contract.** A retry key is scoped to the principal and exact request. A
matching authenticated retry recovers the same result/credential without another
effect or success audit; a changed request/authority or expired receipt cannot
start fresh work under that key. Cached-result disclosure still needs an explicit
authorization contract.

**Observed enforcement.** [HTTP](../../src/api.rs) `protect` hashes method,
URI and raw bounded body and validates unique context headers/quoted `If-Match`.
[Context](../../src/context.rs) `mutation` authenticates, compares receipt
fingerprint/permission snapshot, and requires revision for new HTTP agent writes.
Receipts live 24 hours with tombstones retained to seven days; matching receipt
retrieval precedes revision checks. [State](../../src/state.rs) `apply_state`
returns an applied result after live principal/actor/plan checks, before rerunning
resource permissions. It does not compare a permission snapshot at that branch.

**Existing regressions.** [Operations](../../tests/identity/operations.rs)
`http_agent_mutations_are_atomic_retriable_conditional_and_attributed`,
`agent_plans_are_immutable_atomic_and_idempotent`, and
`external_vault_signing_keeps_private_keys_out_of_storage_pins_version_and_verifies_signatures`.

**Missing coverage / later contract.** Q02-C04 checks different principal,
URI/body, expired receipts, reduced scope and newly privileged target on result
retrieval. Reduced-authority plan-result/secret disclosure is G-04, not a
demonstrated finding. Q05-R05 interrupts after commit/before response, retries
concurrently and verifies one effect, one credential and one success audit.

### RI-MGT-004: local effects, revocation, receipts and audit commit together

**Contract.** An authorized local mutation's data, security transitions,
derived indexes, required queued work, retry result and audit evidence must
describe the same committed outcome. Failure cannot leave a partial credential
or successful audit. Equivalent interfaces need equivalent permissions,
security effects and audit facts, allowing transport-specific request IDs.

**Observed enforcement.** [Store](../../src/store.rs) `write`, `put`, `delete`
and index/security-transition hooks; [core](../../src/core.rs) `audit` adds
redacted changes/actor/parent/run/request attribution and selected revision bumps;
[context](../../src/context.rs) receipts; [state](../../src/state.rs) apply result
and audit. Revision is not a universal counter for every bucket; depend on the
actual checked state as well (RI-STORE-001).

**Existing regressions.** [Operations](../../tests/identity/operations.rs)
`http_agent_mutations_are_atomic_retriable_conditional_and_attributed`;
[SSF](../../tests/ssf.rs) `password_entry_points_enqueue_once_and_failed_changes_rollback`;
[storage](../../tests/storage.rs)
`maintenance_and_queue_work_is_bounded_and_indexes_commit_atomically`;
[reports](../../tests/reports.rs) `password_change_audit_has_no_hash_and_secrets_stay_out`.

**Missing coverage / later contract.** Q02-C04/C08 compares direct/HTTP/CLI/
future browser writes and rejection snapshots, audit and notifications. Q05-R05
forces rollback or lost response at each local commit boundary. External side
effects require the weaker, explicit RI-CON-002 delivery contract.

### RI-MGT-005: multi-party review cannot transfer or manufacture authority

**Contract (intended policy).** Platform's delegated administration and review
bind author, eligible reviewers, executor, exact content and dependency versions.
Required separation of roles cannot be satisfied by an unauthorized agent,
self-review or a stale role. Review does not add execution permission; current
author/reviewer/executor authority and the approval policy must still hold at the
commit boundary. Audit attributes each role and approved content.

**Current evidence boundary.** [User model](../../src/model.rs) has a human
`admin` flag; [agents](../../src/agent.rs) have scoped permissions. Current
[state plans](../../src/state.rs) bind one actor, with no multi-party human-role
approval record. [PAM](../../src/pam.rs) has a narrower approver gate;
[PAM tests](../../tests/pam.rs) `unauthorized_and_concurrent_decisions_do_not_grant`
are adjacent evidence, not enforcement of the future administration contract.
A02 P14 proposes this broader capability. No complete baseline regression exists.

**Missing coverage / later contract.** Q02-C04 tests disallowed role overlap,
scope/parent escape, changed content/dependencies and revoked reviewer/executor;
rejection leaves no mutation or success audit. Q05-R03/R05 races approvals,
permission/policy changes and execution; stale approval cannot authorize a new
operation and retries produce one attributable committed outcome.

### RI-MGT-006: agent-prepared sensitive changes need exact human approval

**Contract.** An agent cannot change an authentication factor, recovery
address, administrator role or delegated grant set by preparing it. An owned
agent with `changes.prepare` on the target records one exact change, bound by
digest to its content and to a fingerprint of the target's relevant state.
Only the owner applies it, by digest, with a sign-in from the last five minutes
and MFA when enrolled; agent credentials cannot approve and another person's
change is not found. At approval the change must be open and unexpired, the
preparing agent live and still authorized, the target state unchanged, and,
for role and grant changes, the owner a current full administrator who
authorized that agent themselves, so another administrator cannot author a
reviewed change through it and then review it. The change
then runs as the owner through its existing writer, so elevation provenance,
credential exposure, last-administrator and passkey rules still hold; a
reviewed grant role is only staged for the M05 author/reviewer/executor
workflow. Owner-approved changes to the owner's own account are not operator
exposure. Prepared changes carry no secret and are invalidated on restore.

**Observed enforcement.** [Prepared changes](../../src/management/prepared_changes.rs)
`prepare`, `approve`, `state`, `preparer_live`; [owner self-service](../../src/management/owner_agents.rs)
`owner_session`; [grants](../../src/management/grants.rs) `requires_review`,
`stage_grants`, `write_immediate_grants`; [management](../../src/management.rs)
`update_user`; [identity](../../src/identity.rs) `user_security_transition`;
[recovery](../../src/recovery.rs) `INVALIDATED`.

**Existing regressions.** [Prepared changes](../../tests/agent_prepared_changes.rs)
`an_owner_approves_their_own_recovery_address_change`,
`an_owner_approves_removal_of_their_authenticator_app`,
`an_owner_approves_removal_of_one_passkey`,
`an_administrator_owner_approves_a_role_change_and_agent_credentials_stay_unelevated`,
`low_risk_grants_apply_and_reviewed_roles_are_only_staged`,
`approval_binds_the_exact_change_and_the_account_state`,
`approval_needs_fresh_authentication`,
`revoking_the_agent_voids_its_pending_changes`,
`only_a_current_administrator_approves_role_and_grant_changes`,
`an_ordinary_owner_agent_cannot_prepare_role_or_grant_changes`,
`change_requests_carry_no_secret_or_unknown_field`,
`prepared_changes_do_not_survive_restore_or_outlive_expiry`,
`http_routes_serve_the_agent_the_owner_and_the_bound_browser`.

**Missing coverage / later contract.** Q05-R03/R05 race approval with agent
revocation, owner demotion and concurrent target writes. No CLI or portal page
presents these routes yet.

## Connectors and offboarding

### RI-CON-001: imports require complete snapshots and stable bindings

**Contract.** Directory/SCIM ingestion cannot relink accounts from mutable
email/name alone, elevate imported users, or treat incomplete/malformed snapshots
as reviewed mass removal. Plans bind owner, stable external IDs, source identity,
complete content, local revision and source config; apply revalidates them.
Durable group projections exclude temporary entitlements.

**Observed enforcement.** [LDAP](../../src/directory.rs) `snapshot`,
`directory_plan_get`, `directory_apply`, `reconcile`, `validate_identity`;
[cloud](../../src/cloud_directory.rs) pagination/materialization/removal impact
and `cloud_apply_confirmed`; [inbound SCIM](../../src/scim.rs) `owned`,
`scim_write`, `scim_delete`. [Core](../../src/core.rs) separates durable/effective
groups. Connector snapshot checking does not freeze the upstream after the read.

**Existing regressions.** [LDAP](../../tests/ldap.rs)
`openldap_plans_stable_ids_tls_login_mfa_and_fail_closed_sync`;
[cloud](../../tests/cloud_directory.rs)
`workspace_links_membership_suspension_and_redaction`,
`malformed_success_pages_fail_plan_and_apply_without_deprovisioning`;
[HTTP](../../tests/identity/http.rs)
`scim_http_provisioning_is_owned_atomic_retriable_and_deprovisions_sessions`;
[PAM](../../tests/pam.rs) `directory_projections_stay_durable_through_grant_expiry_and_revocation`.

**Missing coverage / later contract.** Q02-C06 substitutes external ID/source,
cross-owner resource, pagination origin, incomplete/empty page, group snapshot
and removal confirmation. Q05-R03/R05 races imported disable, source change,
re-enable and local writes against apply; no unreviewed partial deprovisioning.

### RI-CON-002: downstream work is reviewed, durable and honest about uncertainty

**Contract.** Provisioning dispatch uses the reviewed target/resources and
current creator/parent authority, revision and target configuration. Durable
leases/results support safe retries and reconciliation. A timeout after remote
acceptance is uncertain, not proof of failure; partial/stale work must remain
visible. Local transactions cannot guarantee remote rollback or exactly-once
effects across arbitrary peers.

**Observed enforcement.** [Provisioning](../../src/provisioning.rs)
`provisioning_apply`, `claim_provisioning`, `provisioning_step`,
`finish_provisioning` bind actor/target and lease; dispatch uses remote lookup,
managed-field comparison and conditional ETags. Finish rechecks revision/actor
after the external effect. Revocation after claim can coincide with an in-flight
accepted write (G-05). OAuth secret/token acquisition rejects redirects and keeps
token metadata separate from raw cached access tokens.

**Existing regressions.** [Policy](../../tests/identity/policy.rs)
`outbound_scim_plans_provision_groups_preserve_remote_attributes_disable_departures_and_stop_stale_jobs`;
[SCIM OAuth](../../tests/scim_oauth.rs)
`uncertain_patch_response_is_reconciled_without_a_second_patch`,
`wrong_secret_server_errors_and_rejected_tokens_do_not_call_scim`,
`secret_rotation_and_static_token_rotation_apply_on_next_acquisition`.

**Missing coverage / later contract.** Q02-C06 models queued/accepted/partial/
stale outcomes and preserves remote unmanaged fields. Q05-R06 pauses before and
after peer acceptance, revokes authority/parent, changes target config, expires
lease or loses response; assert no subsequent unauthorized dispatch, no false
completion and reconciliation of uncertain work. Multi-process peer effects need
integration evidence; a lease-only test cannot establish that guarantee.

### RI-CON-003: delayed offboarding rechecks authority, identity, cancellation and lease

**Contract.** Only a valid due job's current lease owner can commit. Target
identity, creator/parent permissions, administrator protection and cancellation
are rechecked at execution. Successful local disable and its audit/result happen
once; remote completion is separately reported.

**Observed enforcement.** [Offboarding](../../src/offboarding.rs)
`offboard_claim`, `offboard_commit`, `authority_still_valid`, `apply_local`
validate owner/deadline/status/user ID/cancel/authority. `apply_local` bumps epoch
and disables through central store hooks. The shared transition records
per-target deactivation rows in that transaction; the job keeps their IDs and
its view reports each target's live outcome. Nothing calls SCIM inside the commit;
[deactivation delivery](../../src/provisioning/deactivation.rs) reports `delivered`
only after the target confirms the account inactive. Reschedule/cancel are separately authorized writes.

**Existing regressions.** [Offboarding](../../tests/offboarding.rs)
`second_worker_claims_once_and_epoch_increments_once`,
`expired_lease_is_reclaimed_by_one_owner`,
`running_cancel_is_observed_before_revocation`,
`execution_revalidates_agent_parent_even_for_a_legacy_enabled_agent`,
`offboarding_commits_downstream_intent_and_reports_each_target_only_after_delivery`.

**Missing coverage / later contract.** Q02-C06 verifies due-time/identity/
creator/admin/cancel outcomes and honest downstream status. Q05-R07 races claim,
lease expiry/reclaim, cancel/reschedule, retry/restart and target promotion; old
owners cannot commit, successful epoch/audit increments occur once.

### RI-CON-004: security signals bind trusted issuer, subject and event

**Contract.** An inbound signed security event affects only the configured
subject binding for its trusted issuer/format. Duplicate events do not repeat
revocation. Outbound disclosure respects current subject bindings, and required
events enqueue atomically with the local transition. Owning an OAuth client
secret does not authorize creation of inbound signing trust.

**Observed enforcement.** [SSF](../../src/ssf.rs) trust/subject validation,
`accept_set`, `enqueue` and `deliver_once`; [core](../../src/core.rs)
`user_security_transition` distinguishes credential changes from factor use.
Delivery is durable/retriable, not synchronous remote revocation.

**Existing regressions.** [SSF](../../tests/ssf.rs)
`inbound_sets_disable_only_the_linked_account`,
`ssf_subject_bindings_include_format_and_subject_issuer`,
`duplicate_set_acknowledges_without_reapplying_session_revocation`,
`removing_a_subject_binding_stops_queued_disclosure`,
`ordinary_client_secret_cannot_create_own_signing_trust`.

**Missing coverage / later contract.** Q02-C06 swaps issuer/subject format/key/
event and checks permitted event types, redaction and no unrelated account effect.
Q05-R01/R06 races duplicate SETs, binding removal and delivery leases; one local
transition and no new disclosure after authorization is lost.

## Workflows

### RI-WF-001: embedded source stages cannot skip request-bound verification

**Contract.** A required source stage binds the exact original authorization,
stage instance, source/link, browser and local account. Resume/cancel/replay and
local-factor completion must preserve that binding and current trust. OAuth-only
identity data cannot invent OIDC authentication time or MFA assurance.

**Observed enforcement.** [Sources](../../src/source.rs) `begin_source_stage`,
`source_stage_resume`, `load_stage`, `enforce_pending_stage`, `verify_identity`,
`validate_identity`; [SAML sources](../../src/source/saml.rs) verify upstream
responses. OIDC source nonce/authentication time and configured trusted MFA ACR
are checked. `source_callback` ends a login when its pinned source changed
before the code is exchanged, and drops an identity when the keys change before
that identity is stored. Restoring the previous keys does not finish that login.
`saml_source_callback` ends a login when its pinned source changed before the
ACS accepts the response. Restoring the previous IdP certificate leaves that
login unfinished. `saml_source_browser_return` ends a verified browser login
when its pinned source changed before the return. Restoring the previous
certificate does not confirm that return. A code, SAML response, or browser
return presented while its source is disabled ends that login. Enabling the
source again does not redeem it. A login that was not presented can still
complete after the source is enabled again. [SAML](../../src/saml.rs) `stale` deliberately treats zero
`auth_time` as exempt from the terminal freshness rule; request `max_age` and
factor operations are separate rules, not satisfied by that exemption.

**Existing regressions.** [Stages](../../tests/source_stage.rs)
`suspend_then_resume_completes_the_original_request_and_replay_fails`,
`upstream_account_must_match_the_bound_user_and_link_table`,
`prompt_and_max_age_still_require_a_fresh_transaction`,
`local_totp_is_still_required_when_upstream_is_not_mfa`,
`oauth_only_stage_does_not_invent_authentication_assurance`;
[sources](../../tests/identity/sources.rs)
`oidc_source_jwks_rotation_checks_old_and_new_keys_stale_assertions_and_rollback_replay`,
`oidc_callback_presented_while_source_disabled_cannot_redeem_after_reenable`;
[SAML sources](../../tests/identity/saml_source.rs)
`saml_source_certificate_rollover_checks_old_and_new_keys_stale_assertions_and_rollback_replay`,
`saml_browser_return_ends_when_pinned_source_changes_and_restore_cannot_confirm`,
`saml_login_presented_while_source_disabled_cannot_continue_after_reenable`;
[browser](../../tests/browser_signin.rs) `oauth_source_session_is_exempt_from_terminal_freshness`.

**Missing coverage / later contract.** Q02-C07 changes stage/request/account/
link/source trust independently and asserts no issued code. Q05-R01/R03 races
resume/cancel/source disable/unlink/expiry with local-factor completion.

### RI-WF-002: configurable workflows retain shared security gates

**Contract (intended policy).** Platform workflows cannot override account,
credential, authorization, revocation or proof requirements. Every run binds its
definition/version and security dependencies plus account/session/request.
Approval resumes only reviewed content with revalidated authority; changed or
unavailable required stages reject explicitly. Essentials has complete default
journeys without an operator-designed workflow.

**Current evidence boundary.** [Module exports](../../src/lib.rs) and
[config](../../src/config.rs) show embedded source stages, not a general
configurable-workflow engine. RI-WF-001 is adjacent enforcement, not proof of
arbitrary workflow safety. No baseline regression establishes this whole contract.

**Reviewed-version slice.** Configured runs loaded from `config.workflows` store
the active revision, fingerprint, and policy digest, and `workflow_reviewed`
retains the highest adopted pin. A missing retained pin, a policy change, a
disabled account, or a rolled-back revision seals the open run as denied, with
no new evidence and no grant. Browser OIDC password, TOTP, and passkey consent
commit that seal before a factor, ceremony, or decision write, including after
the store is reopened onto a changed policy. When the pin still matches, a
browser continuation whose consent selection changed, was removed, or no longer
names a usable adapter commits that denial before `Browser consent workflow changed`,
and restoring the selection does not resume the run. Reopening the store keeps that pin
and a denial already recorded for the run. A later compatible revision starts a
new run and leaves the sealed run denied. Code-owned revisions that are absent from `config.workflows` stay
unpinned. With no activation pointer, the persisted authoring store remains
outside the executor's selection.

**Exact-content approval slice.** One workflow-only desired-state plan can be
reviewed by a second enabled administrator and activated by a third. Activation
commits the catalog row and an immutable approval in one store write and leaves
`config.toml` unchanged. While the activation pointer is current, configured
execution uses the definition embedded in that approval. A present
`config.workflows` entry must already be active and byte-equal. The pin binds
the definition id, revision, and fingerprint, the approval id, and a dependency
digest of the platform profile, the supported adapter, each referenced enabled
source record, and each used extension module. A stale, revoked, or refused
selection seals an open pinned run. The same bytes may be approved again; the
old run stays sealed and the next run is new, with fresh proofs. A lower
revision than the retained pin is refused before the pointer is written.
Cancellation of an open run stays cancelled. Unapproved `config.workflows`
entries still start through the reviewed pin. Browser selector usability still
reads `config.workflows`. The dependency digest leaves the rest of the
environment unbound. Activation holds the existing writer lock and leaves other
clients connected. This does not establish RI-WF-002.

**Controlled-extension slice.** [The host](../../src/workflow/extension.rs)
is a held in-process contract. A native registrant shares the server address
space, and this crate cannot preempt it or interpose on its syscalls. The
executor does not call it. The
[guest gate](../../src/workflow/extension_gate.rs) links Wasmi 0.40 on Platform
only. One configured graph runs: a custom `extension` step, then local
password. The guest has no imports, one 64 KiB page, no network and no
filesystem. Fuel is 1–10,000. Wasmi 0.40.0 has no epoch or interrupt API.
`Store::call_hook` and `call_resumable` pause only around host functions, and
this guest links none, so the engine cannot preempt `route`. The server
re-executes its own binary and the child parses, translates, instantiates, and
runs `route` inside the step timeout. The 1–30 second field is that many
thousands of fuel units and the parent's monotonic deadline, measured from
just before spawn. That interval includes process creation, observation, and
reap. It is not an exact kernel schedule. When the clock reaches the deadline,
including a child that has already exited, the parent kills a child that is
still running, reaps the process, discards its stdout, and reports `elapsed`.
The child is not detached. Fuel exhaustion stays `fuel`, or `timeout` when the
timeout fuel budget is strictly smaller than the manifest fuel. Admission
refuses a function body whose 7-fuel-per-byte translation charge exceeds
`min(manifest fuel, timeout_seconds × 1,000)` before `Module::new` and before
a process starts. At the 10,000 fuel cap the body can be at most 1,428 bytes.
`check` runs Wasmi `Module::new` in the helper under the manifest timeout and
does not keep the image. A validation observed at or after that deadline is
`elapsed` and is not admitted. Structural caps and the translation-budget
arithmetic still run on the caller before that spawn. Each `execute` is a new
process, so each call pays the translation charge again inside the deadline. A tighter step budget can still skip
translation. A charge that fits is translated in the child. Validation of a
fitting body is not fuel-metered. `route.call` holds `&mut Store` until it
returns, so another thread inside the child cannot drain that store's fuel.
Stopping a thread would require `unsafe`, which this crate forbids. The helper
is `current_exe` with only the internal argv token. The child environment is
cleared. The helper refuses to run when any variable remains, except macOS
`__CF_USER_TEXT_ENCODING` after exec when its value is three short `0x`
hexadecimal fields. The parent does not pass that variable. Its working
directory is private and removed after reap, and the
request has no bearer token, password, configuration path, or database URL.
File descriptors already open in the server can still be inherited. A missing
helper or a spawn failure is `failed` and grants nothing. Stdout is capped;
bytes past that cap are `output` and are not a label. Before
compilation the gate allows one `() -> i32` function, two exports, and at most
32 i32 locals, and it rejects a data segment. At the call, Wasmi reserves a
value stack of 64 `UntypedVal` slots (8 bytes each, at least 512 bytes) and
refuses a frame that would make the live length reach 64, before `Vec::reserve`.
That trap is `limit`. `ResourceLimiter` does not cover this stack; it allows
the one 65,536-byte linear memory and reports a further page as `failed`. A
frame that fits runs in the child until it returns, spends its fuel, or the
parent kills the process. Output is one declared
label or the built-in `failed` signal, and it is not a proof. The host reads at most
32 bytes of that label, and only when the returned length equals a declared label.
A longer return is `output` and is not read, including when the manifest output cap
is 4,096. A length inside that cap that equals no declared label is `undeclared_output`
and is not read. The run binding
stores the module hash beside the source-registration pin used by
source-verifier runs. A changed manifest seals the open run before the new
bytes can run. Essentials does not link Wasmi, rejects `workflow_extensions`,
and `execute` returns `external_runtime_required`. The capability
`workflow.controlled_extensions` is configured and usable only for that active
validated graph plus a matching admitted manifest. Default, inactive,
unsupported, and Essentials configurations leave both flags false. This is not
the Q02 engine adapter.

**Missing coverage / later contract.** Q02-C07 needs the future engine adapter:
missing/disabled stage, changed definition/dependency, forged next-stage state,
wrong account/request and weaker credential route all reject without issuance.
Q05-R01/R03 races definition change, resume, cancel and authority loss; completed
steps cannot be replayed into another run. Resolve approval/dependency versioning
with A02 and the workflow workstream before implementing the engine.

## Proxies

### RI-PROXY-001: only trusted forwarding context can assert application identity

**Contract.** Immediate-peer trust gates forwarded identity/context. Exact
allowed origin/target and supported header syntax constrain login/callback;
untrusted caller identity headers and riAuth cookies cannot pass through to the
application as authority. WebSocket/unsafe request origin checks precede forwarding.

**Observed enforcement.** [Outpost](../../src/outpost.rs) `proxy_peer`,
`forwarded_target`, `Settings::target`, `outpost_forward`, `outpost_callback`;
[proxy](../../src/proxy_server.rs) `clean`/`handle` strip/rebuild identity headers
and private cookies; [API attribution](../../src/api/rates.rs) only uses configured
trusted proxies. nginx/Traefik deployment must overwrite client headers and copy
the filtered application cookie as documented in [proxy SSO](../proxy.md).

**Existing regressions.** [Traefik](../../tests/outpost_traefik.rs)
`traefik_endpoint_requires_trusted_peer`,
`traefik_endpoint_validates_forwarded_headers_and_ignores_original_url`,
`authenticated_request_returns_identity_and_filtered_cookie`,
`websocket_handshake_requires_allowed_origin`,
`cross_site_unsafe_methods_are_refused_before_authentication`;
[HTTP](../../tests/identity/http.rs) `forwarding_headers_cannot_spoof_the_rate_limit_identity`.

**Missing coverage / later contract.** Q02-C05 tests duplicate/malformed
forwarding headers, unknown peers, wrong target origin, identity injection,
reserved cookies and return URLs. Q05-R03 changes proxy settings/trust while
callback is pending. Real deployment behavior needs real proxy evidence.

### RI-PROXY-002: proxy sessions inherit live parent authorization

**Contract.** A proxy cookie is bound to its client/policy fingerprint and
parent session; online forwarding rechecks account, session, groups/scopes,
assurance and device trust. Consent or parent revocation cannot leave an
independently valid proxy credential.

**Observed enforcement.** [Outpost](../../src/outpost.rs) `fingerprint`,
`outpost_auth`, `revoke_sessions`; [OIDC](../../src/oidc.rs)
`proxy_auth_with_proof`; [browser](../../src/browser.rs) `revoke_consent`;
[proxy](../../src/proxy_server.rs) retrieves current client profile per request.

**Existing regressions.** [Traefik](../../tests/outpost_traefik.rs)
`revoked_parent_session_is_unauthorized_next_time`;
[network](../../tests/identity/network.rs)
`consent_revocation_rejects_existing_proxy_sessions` and
`domain_proxy_shared_session_exact_origins_public_suffix_and_live_scope_policy`;
[device](../../tests/device_trust.rs) `portal_and_proxy_require_the_originating_sessions_live_verification`.

**Missing coverage / later contract.** Q02-C05 compares policy changes on all
proxy adapters. Q05-R03 revokes during callback or before the next forwarded
request. Established WebSockets and upstream application sessions are not proven
to terminate immediately; do not widen a next-request guarantee into that claim.

## Devices and certificates

### RI-DEV-001: verified device trust belongs to one session, account and device

**Contract.** Device trust requires a configured verifier with pinned key,
algorithm and the configured riAuth audience, an unspent nonce for this account/session/
epoch, and a fixed device ID. Expiry is capped by verification freshness, signed
proof and originating live session. Missing verifier or legacy unbound trust fails
closed; trust cannot be borrowed by another session.

**Observed enforcement.** [Device trust](../../src/device_trust.rs)
`verify_token`, `device_challenge`, `device_verify`, `policy_reason` verify pinned
algorithm/key and audience/expiry/nonce, consume the challenge, and bind
`DeviceVerification` to session/user/epoch/device. A different device needs a new
session. No device-attestation or enrollment-agent guarantee follows from this JWT
signal protocol. `google_verified_access_v2` is a separate adapter in
[verified_access.rs](../../src/verified_access.rs). Before verify it requires the
`challengeResponse` `SignedData` to embed the issued challenge's data and
signature bytes. The device signature over that embedding remains Google's
verify check. The adapter was not run against `verifiedaccess.googleapis.com`
or a managed Chrome device, and it does not evaluate `deviceSignals`.

**Existing regressions.** [Device trust](../../tests/device_trust.rs)
`forged_replayed_expired_and_mismatched_signals_are_rejected`,
`missing_verifier_fails_closed_and_default_flag_does_not`,
`trust_is_bound_to_one_session_device_and_proof_lifetime`,
`legacy_unbound_trust_and_mismatched_epoch_fail_closed`.

**Missing coverage / later contract.** Q02-C07 swaps nonce/session/epoch/device/
key/audience/algorithm and tests lifetime caps on token/portal/proxy paths.
Q05-R01/R03 races verification, nonce replay, account disable and session expiry.
Verifier rotation semantics for already stored proofs need an explicit test.

### RI-DEV-002: device credentials and online/offline tickets have distinct authority

**Contract.** Device enrollment/reassignment is authorized for a fixed user,
returns its secret only through protected credential output, and revokes old
secret/tickets. Online tickets are one-use short-lived logon assertions, not OAuth
or admin tokens. Offline claims bind device/user/epoch and bounded time; genuine
disconnected operation has an explicit revocation-delay limit.

**Observed enforcement.** [Windows](../../src/windows_login.rs)
`windows_device_enroll`, `windows_login`, `windows_ticket_redeem`,
`windows_offline_verify`, `revoke_user` check device digest/user/factors and
epoch. Redeem consumes even on failed live checks. Server offline verification
also reads live storage; it is not proof a disconnected client learned revocation.
The [protocol guide](../enterprise/ENT-13.md) limits offline tickets to 72 hours
and identifies the missing packaged credential provider.

**Existing regressions.** [Windows](../../tests/windows_login.rs)
`bad_secret_disabled_user_and_revoke_fail`,
`offline_ticket_respects_epoch_expiry_revoke_and_device`,
`reenroll_replaces_the_mapping_and_the_secret`, `agent_cannot_enroll_an_administrator`.

**Missing coverage / later contract.** Q02-C07 includes cross-class ticket
rejection, wrong binding, caps and redaction. Q05-R01/R02 races redeem, re-enroll,
quota and epoch/disable changes. A future device agent needs separate local clock,
secret custody, known-epoch and reconnect contracts and real-device evidence.

### RI-DEV-003: certificate authentication requires explicit live binding

**Contract.** Valid CA membership alone cannot choose a local account. Native or
trusted-proxy certificate authentication verifies chain/purpose/time and configured
revocation checks plus explicit user binding; untrusted certificate headers reject.
Certificate assurance must not silently satisfy a stronger MFA policy.

**Observed enforcement.** [mTLS](../../src/mtls.rs) `verify_chain`,
`binding_matches`, `validate_identity`, binding/session invalidation;
[RADIUS EAP](../../src/radius/eap.rs) certificate enrollment/identity validation;
[assurance](../../src/assurance.rs) `actual` preserves certificate ACR.
Optional CRL configuration and both mTLS modes are documented in
[certificate trust](../enterprise/ENT-05.md); `required` does not require a client
certificate on every anonymous TLS handshake, only on certificate login.

**Existing regressions.** [mTLS](../../tests/mtls.rs)
`https_client_certificates_bind_chain_revocation_and_reject_forged_headers`;
[EAP](../../tests/identity/radius_eap.rs)
`openssl_eap_tls_versions_fragments_keys_enrollment_policy_and_revocation`.

**Missing coverage / later contract.** Q02-C07 tests unbound/wrong-account,
expired/wrong-purpose/revoked chain, forwarded peer/header and insufficient ACR.
Q05-R03 changes binding/revocation material while login is pending and records
whether existing sessions observe that change. No universal continuous CRL
revalidation or real-supplicant compatibility is claimed here.

## Storage, custody and recovery

### RI-STORE-001: verification cannot commit against changed or expired reads

**Contract.** Preparation outside the writer must track every security-relevant
read/dependency and revalidate it at commit, including deadline expiry without a
concurrent mutation. Conflicts retry or reject with no stale credential/effect.
Local transactions provide atomic one-time decisions and cannot include
irreversible remote writes inside retried preparation callbacks.

**Observed enforcement.** [Prepared store](../../src/store/prepared.rs)
`prepared_write` caches point/range reads and staged writes, tracks future
`expires_at` values, validates under `Store::write`, and retries up to four times.
[Store](../../src/store.rs) uses redb transactions or PostgreSQL advisory-locked
writers. Tracking `expires_at` is not proof that every caller's freshness,
derived entitlement deadline, file config or external dependency is tracked.

**Existing regressions.** [Storage](../../tests/storage.rs)
`prepared_authentication_does_not_hold_the_writer_and_rechecks_revocation`,
`concurrent_prepared_writes_retry_without_lost_updates`,
`prepared_ranges_detect_phantoms_and_include_staged_changes`,
`prepared_authority_expiring_during_signing_is_rechecked_before_commit`;
[PostgreSQL](../../tests/postgres.rs)
`postgres_atomicity_shared_sessions_replay_limits_migration_and_fenced_failover`.

**Missing coverage / later contract.** Q02-C08 applies contract adapters to
redb plaintext/encrypted and PostgreSQL. Q05-R03/R04 pauses actual hashing/signing
after reads, changes point/range authority or crosses a policy deadline, then
asserts no stale issuance. Existing helper tests do not exhaust caller dependencies.

### RI-STORE-002: recoverable secrets have explicit custody and public outputs are redacted

**Contract.** Passwords/opaque credentials use one-way verifiers where possible;
recoverable factor/key/delivery/retry secrets have explicit access and storage
custody. Public views, inventory, audit, metrics and ordinary CLI output cannot
disclose them. A configured encryption key cannot silently fall back to plaintext.
External signer pins/version/signature must be verified before issuance.

**Observed enforcement.** [Crypto](../../src/crypto.rs) password hashing,
digests and AEAD; [store](../../src/store.rs) record-key AAD, encoding checks,
public views and `redact_audit_value`; [config](../../src/config.rs)
`read_private_secret`/`write_private` bound private regular-file reads and Unix
permissions/atomic writes; [transport](../../src/cli/transport.rs) private saved
sessions, issuer pinning and no redirects; [KMS](../../src/kms.rs) verifies remote
signatures. Unix mode checks are not proof of Windows ACLs.

**Existing regressions.** [Policy](../../tests/identity/policy.rs)
`non_admins_cannot_manage_identities_and_views_never_return_secrets`;
[reports](../../tests/reports.rs) `password_change_audit_has_no_hash_and_secrets_stay_out`;
[operations](../../tests/identity/operations.rs)
`encrypted_backup_restore_preserves_identity_and_keys_and_invalidates_grants` and
`external_vault_signing_keeps_private_keys_out_of_storage_pins_version_and_verifies_signatures`;
[SCIM OAuth](../../tests/scim_oauth.rs) secret/cache regressions.

**Missing coverage / later contract.** Q02-C09 inserts distinct secret markers
and recursively scans every public/error/audit output; test private files, wrong/
missing keys and receipt/mail custody. Encryption is optional today; mail bodies
and credential-result receipts may contain usable secrets. Q05-R05/R06 tests
secret rotation during retries and signer failure/invalid signature before commit.

### RI-STORE-003: backup/restore authenticates complete supported snapshots

**Contract.** Backup reads one consistent snapshot and protects records,
configuration and ordering with authenticated encryption. Restore validates
format/schema/issuer/manifest and records, refuses overwrite and creates a private
new instance while preserving supported identities/keys. Missing external keys,
credentials/files/services remain visible recovery prerequisites.

**Observed enforcement.** [Operations](../../src/operations.rs) `backup`,
`load_chunked`, `restore_v1`, `restore_v2`, `commit_restore`, and the [v3 stream
reader](../../src/operations/stream.rs) authenticate v1/v2/v3 archives and validate
issuer/schema, signing keys, user/client state and an enabled administrator with
a username binding. They apply the RI-STORE-004 policy and rebuild indexes in
the import transaction. Restore targets a new redb directory or an empty isolated
PostgreSQL database, then reopens it and checks the recovery gate before writing
the usable configuration. Default v2 archives have a 64 MiB limit and new
plaintext pages have an 8 MiB limit; v3 streams have a 4 GiB quota. Restore does
not restore external Vault custody or secret files, or prove full application login.

**Existing regressions.** [Operations](../../tests/operations.rs)
`chunked_backup_restore_preserves_identity_and_invalidates_grants`,
`legacy_single_blob_backup_still_restores`,
`restore_rejects_oversized_archive_before_reading_or_creating_output`,
`schema_two_backup_restores_and_rebuilds_queue_and_retention_indexes`;
[identity operations](../../tests/identity/operations.rs)
`encrypted_backup_restore_preserves_identity_and_keys_and_invalidates_grants`;
[shared contracts](../../tests/contracts/shared.rs)
`direct_restore_selected_backend`, `direct_restore_failure_preserves_original`,
and `direct_restore_postgres_archive_into_redb`.

**Remaining coverage.** Q02-C10 still needs a complete external prerequisite
inventory and a deployment rehearsal with real administrator credentials and
applications. Q05-R08 covers interrupted restore and backup writes; broader
failure injection and operational cleanup need separate deployment evidence.
RTO/RPO and real application recovery need separate deployment evidence;
`verified` is a scoped local check.

### RI-STORE-004: restore rollback cannot be mistaken for current revocation state

**Contract (intended recovery policy, reconciled with A02 G11/E17).** Preserve
identity continuity and invalidate restored sessions, pending proofs and grants
by default before traffic resumes. Restored persistent credentials must reconcile
post-snapshot revocations or be rotated/re-enrolled before use. Stop/fence other
writers and reconcile replay state, receipts and pending/uncertain remote work.
An authenticated old archive must not be advertised as preserving revocations or
consumptions made afterward. A continuity exception requires an explicit reviewed
policy and equivalent evidence; it cannot silently restore pre-incident access.

**Observed enforcement (R04).** [Recovery](../../src/recovery.rs) `invalidate`
runs inside the restore import transaction (`commit_restore`), on a PostgreSQL
store whose recorded cluster/database/table lineage changed (`verify_lineage`),
and through offline `riauth recovery invalidate --database-restored`
(`invalidate_restored`, refused while other riAuth PostgreSQL clients are
connected). It deletes restored sessions, grants, families, one-time codes,
pending proofs, proof mail and consents; clears pending MFA enrollment secrets
(`users[*].totp_pending`); ends RP sessions (paged, rows kept for signing-key
retention) and queues back-channel logout; revokes temporary access and denies pending requests; advances every
account epoch and `meta/revision` by 2^32; keeps replay caches, identities,
subjects and keys. `meta/recovery` then blocks `serve` and readiness until
`recovery complete --recovery-id <id> --persistent-credentials-reconciled` records the operator's
attestation that listed persistent credentials were reconciled or rotated.
`recovery status` inspects without opening: it never creates, formats, migrates
or recovers a store and reports a missing one as not serving. The
policy and PostgreSQL boundary are in [restored-state recovery](../recovery.md).

**Observed limit.** No external monotonic epoch/revocation ledger exists.
Restored persistent credentials (disabled accounts, changed passwords, used
recovery codes, removed passkeys/bindings, rotated agent/client/source secrets,
retired signing keys) return to snapshot state; the gate lists them but cannot
verify reconciliation. Physical/PITR restores of the same PostgreSQL cluster,
asynchronous promotions and copied redb files are undetected and depend on the
operator command. Replay entries accepted only on the lost timeline, offline JWT
or RP/SP sessions, Windows offline tickets and remote job effects are outside the
policy.

**Existing evidence.** [Recovery](../../tests/recovery.rs)
`restore_invalidates_restored_sessions_proofs_and_grants_but_preserves_identity`,
`older_restore_cannot_resurrect_post_snapshot_revocations_or_consumption`,
`serve_refuses_unreconciled_restored_state_before_listening`,
`in_place_recovery_matches_restore_policy_and_keeps_history`,
`recovery_ends_every_rp_session_page_and_clears_pending_mfa_enrollment`,
`every_storage_collection_has_a_restore_classification`; shared contract
`database_native_restore_policy` on redb, encrypted redb and isolated PostgreSQL
(database clone lineage detection and connected-client refusal), and
`recovery_status_never_creates_or_writes_a_store` on the same four modes.
[Operations](../../tests/operations.rs)
`chunked_backup_restore_preserves_identity_and_invalidates_grants` compares the
restored snapshot against the policy.

**Missing coverage / later contract.** Q05-R08 still interrupts restore/policy
writes and races recovery with running nodes. Q02-C10 extends the restore matrix
to TOTP/recovery-code consumption, agents, devices, receipts and remote jobs, and
to R01/R02 readers. A continuity exception, automatic credential reconciliation
from an external audit ledger, and SAML SP logout fan-out are not implemented.

## Product and runtime boundaries

### RI-DIST-001: shared capabilities use one semantic core

**Contract (intended policy).** Essentials and Platform share account IDs,
subjects, credential/factor verification, session/proof binding, authorization,
revocation, retry and audit effects for common capabilities. Compact browser
administration, API and riauthctl must not bypass shared checks. Product size or
headless operation does not permit weaker security.

**Current evidence.** [Core](../../src/core.rs), [policy](../../src/claims.rs),
[assurance](../../src/assurance.rs), [store](../../src/store.rs) are shared in one
crate today. [CLI](../../tests/cli.rs)
`binary_initializes_serves_and_manages_oidc_over_real_http` is one-binary journey
evidence, not two-distribution parity or a complete browser-admin journey.

**Missing coverage / later contract.** Q02-C01–C10 run identical common
contracts on each eventual build and browser/API/CLI adapter, comparing normalized
identity/authority/revocation/audit outcomes. Q05-R01–R09 run common race schedules
on both. Record actual artifact hash/features/target; compiled-out Platform features
do not excuse failure in the shared contracts.

### RI-DIST-002: compiled, enabled and configured are separate fail-closed states

**Contract (intended policy).** A compiled capability may be disabled; an enabled
capability may still lack required trust/configuration. Disabled/omitted required
security cannot become a silent bypass. Reject unsupported configuration and
stored policy references before exposing a partial service. Server/headless
operation must not depend on terminal USB libraries or suppress ordinary browser
WebAuthn; optional USB belongs to riauthctl.

**Current evidence.** [Config](../../src/config.rs) denies unknown fields,
defaults optional listeners/connectors to empty and verifiers to absent, and
validates configured entries. [Device trust](../../src/device_trust.rs) denies
required trust without a verifier. [Cargo.toml](../../Cargo.toml),
[lib](../../src/lib.rs), [router](../../src/api.rs) and
[server](../../src/api/server.rs) have no reviewed Essentials/Platform gating.
Local HTTP loopback and optional at-rest encryption are current defaults, not
newly established product choices.

A02's proposed default serves shared browser account/admin/protocol components.
Headless management may hide shells but retains the required interaction pages;
all-HTML-off is valid only for service/API-only configurations without interactive
dependencies. Public discovery must reflect usable configured profiles. Current
`agent::capabilities` and OIDC discovery are largely static, not proof of those
capability states. These are intended gates, not implemented switches.

**Existing adjacent regressions.** [Device trust](../../tests/device_trust.rs)
`missing_verifier_fails_closed_and_default_flag_does_not`;
[config](../../src/config.rs) configuration tests;
[CLI](../../tests/cli.rs) `cli_help_and_http_rejection_are_actionable`.
No baseline test establishes the complete build/runtime matrix.

**Missing coverage / later contract.** Q02-C11 checks compiled-in/out ×
enabled/disabled × configured/missing/invalid, including routes, listeners,
workers, discovery, headless UI conflicts and active/retained stored dependencies.
Q05-R09 switches state with pending proofs/jobs;
none completes through a weaker fallback. Reconcile defaults with A02.

### RI-DIST-003: incompatible product/schema transitions reject before weaker operation

**Contract (intended policy).** Version/build/config changes preserve supported
identity and security state or reject with a reviewable incompatibility. An
Essentials switch cannot ignore required Platform workflows, trust, protocols or
credentials referenced by existing state. Future schema, unknown protections or
encryption mismatch reject; recovery uses a compatible rehearsed reader.

**Observed enforcement and limits.** [Upgrade](../../src/upgrade.rs) migrates
schemas transactionally and rejects unsupported future versions;
[store](../../src/store.rs) rejects storage encoding/key mismatch;
[operations](../../src/operations.rs) rejects unsupported backups. Product
compatibility metadata/transition gates are not identified in the baseline.

**Existing regressions.** [Identity operations](../../tests/identity/operations.rs)
`schema_upgrade_is_atomic_preserves_credentials_and_rejects_future_versions`,
`encrypted_backup_restore_preserves_identity_and_keys_and_invalidates_grants`;
[operations](../../tests/operations.rs) legacy backup/schema tests. These cannot
establish two-build downgrade safety.

**Missing coverage / later contract.** Q02-C11 preflights both distribution
directions and compatible/incompatible version/schema/config/backup combinations;
failed startup does not mutate security state or begin serving. Q05-R09 changes
build/config with pending proof/plan/job/refresh state and fences mixed writers.
Reject unsupported dependencies explicitly; never drop required MFA/workflow or
decryption to make a transition succeed.

## Q02 reusable test contracts

Q02 remains todo. Build bounded adapters around actual interfaces; do not mirror
the helper implementation as the oracle. Start with existing common fixtures in
[tests/common](../../tests/common/mod.rs) and its
[security fixtures](../../tests/common/security.rs). Reuse the controlled clock
in [crypto](../../src/crypto.rs) under `test-support`. The result oracle inspects
public outcome **and** committed user/session/family/proof/receipt/job/audit/index
state, with per-path expected failure-consumption rules from RI-SES-003.

| Contract | Concrete reusable inputs and oracle | IDs / missing adapters |
| --- | --- | --- |
| Q02-C01: identity/session | Create users A/B, browser/CLI sessions and all opaque credential classes. Substitute ID/issuer/session/proof type; disable/re-enable or change epoch; compare identity and old-authority rejection. Separately expire CLI and offline family. | RI-ACC-001; RI-SES-001/004/005; future distributions and browser self-service. |
| Q02-C02: credential/recovery | Exercise password, TOTP, recovery, passkey enroll/remove/login, reset/invite/verify via each writer. Vary account/session/purpose/email/epoch/expiry and existing factor; assert no identity replacement, preserved reset factors and proper revocation/replay state. | RI-CRED-001–003; browser recovery/factor paths and device/directory parity. |
| Q02-C03: proof/grant binding | Issue real artifact, mutate one binding at a time: client/redirect/PKCE/resource/scope/key/request instance/account/session/nonce/algorithm. Assert rejection, unchanged unrelated family, prescribed consumption and no code/token on failure. | RI-SES-002/003; RI-AUTH-002; OAuth/SAML/DPoP/assertion adapters. |
| Q02-C04: management parity | Same mutation via direct core, HTTP, CLI and later browser; vary action/resource/parent/admin target/plan fields/dependencies/revision/retry key. Compare normalized committed effects and redacted audit; no partial result on missing secrets or denied last-admin change. Test cached-result authority separately from re-execution; future author/reviewer/executor separation and live roles bind exact approved content. | RI-ACC-002; RI-MGT-001–005; all management routes and future browser/delegated admin. |
| Q02-C05: policy consumers | Obtain authority, then change groups/PAM, client/scopes/ACR/device/source or revoke consent/session. Use token refresh, UserInfo/introspection, portal, LDAP/RADIUS and proxy adapters; assert current policy at each online boundary. Keep offline assertions separate. | RI-AUTH-001; RI-SES-004/005; RI-PROXY-001/002. |
| Q02-C06: connector/jobs | Feed complete/malformed/partial/empty snapshots, wrong external/subject identity, peer origins, config fingerprints, target IDs and ETags. Apply only reviewed resources; check leased progress, cancellation, authority, local disable and honest downstream partial/completion. | RI-CON-001–004; shared remote-state oracle needed. |
| Q02-C07: workflow/device | Vary required stage/definition/dependencies/link/account/request/proof; vary device nonce/key/session/epoch/ID/lifetime and certificate chain/binding/peer. Assert no invented assurance, wrong-account success or missing-required-verifier fallback. | RI-WF-001/002; RI-DEV-001–003; future workflow/device-agent adapters. |
| Q02-C08: transaction/audit | Run real contracts on plaintext/encrypted redb and PostgreSQL; inject rejected second write/invalid signer and compare before/after snapshots, index/queue/receipt/audit consistency. Equivalent transports retain equivalent actor/action/security effects. | RI-STORE-001; RI-MGT-004; PostgreSQL requires explicit integration environment. |
| Q02-C09: secrets | Use unique password/seed/key/bearer/mail/receipt markers; inspect views, plans, export/error/audit/metrics/ordinary CLI output recursively, plus private-file and configured-encryption behavior. Authorized credential output/recoverable storage is tested separately, not called a leak by definition. | RI-STORE-002; RI-MGT-003; error/log capture and platform-specific custody coverage. |
| Q02-C10: recovery | Restore complete/tampered/old archives with correct/wrong/missing keys and external dependencies; check identity/subjects, output isolation and supported format limits. Enforce A02's proposed default invalidation of restored sessions/proofs/grants and persistent-credential reconciliation/rotation; explicitly handle consumed factor/replay/receipt/job state. | RI-STORE-003/004; implementation and detailed recovery policy pending review. |
| Q02-C11: build/runtime compatibility | Once builds exist, record binary hash/target/features and traverse capability/config/stored-dependency states and switch directions. Assert explicit startup/reachability errors with no weaker auth, unwanted worker, identity fork or plaintext fallback. | RI-DIST-001–003; RI-WF-002; pending A01/A02 and architecture implementation. |

Record expected error class and state effect per path. Generic browser credential
errors may intentionally differ from CLI lockout errors. Normalize generated IDs,
timestamps and request IDs for parity comparisons; never normalize away changed
actor, target, credential, scope, epoch, revocation, notification or audit success.
Mark unavailable integrations as unverified, not passing skips. Named artifact
and real-peer gates remain separate from contract unit/HTTP checks.

## Q05 replay, concurrency and interruption schedules

Q05 remains todo. Use barriers/channels at actual read/verify/commit/remote
acceptance boundaries and the controlled clock; avoid timing-only sleeps as
correctness evidence. Do not add permanent production bypasses to pause checks.
Exercise process restart and separate PostgreSQL connections/processes where the
contract concerns shared state. Each schedule asserts final state and effect/audit
counts, not just returned status.

| Schedule | Required interleaving | Required oracle |
| --- | --- | --- |
| Q05-R01: replay/decision | Two consumers start with the same valid proof/code/refresh/assertion/mail/device/SET/stage; race approve versus deny/cancel and callback collection. Also replay with the wrong client/key/account. | At most one authorized one-time transition; documented family revocation only by a correctly bound replay; no transplanted callback or repeat local disable. Covers RI-SES-002/003, RI-CRED-003, RI-CON-004, RI-WF-001/002, RI-DEV-001/002. |
| Q05-R02: credential/browser race | Pause after credential verification/begin, consume the same factor or rotate/remove it elsewhere; race attach of same-user/other-user browser sessions and device re-enrollment/quota. | No stale credential issuance; factors are spent per contract; browser-only sessions stay tokenless, correct session survives or revokes once, replaced device secret/tickets fail. Covers RI-CRED-001/002, RI-SES-001, RI-DEV-002. |
| Q05-R03: authority revalidation | After read/preview/claim, disable user/parent/agent/client, change epoch/groups/source link/key/device binding, promote target, remove another admin or revoke a required reviewer/executor role; release verifier/apply/delivery. Re-enable afterward. | Each new local effect has current authority or rejects atomically; old dependent credentials remain revoked; no admin escalation or false remote rollback claim. Covers RI-ACC-001/002, RI-MGT-001/002/005, RI-SES-004, RI-AUTH-001/002, RI-CON-001/002/003, RI-WF-001/002, RI-PROXY-001/002, RI-DEV-001/003, RI-STORE-001. |
| Q05-R04: deadline/phantom | Pause prepared hashing/signing after authority/range reads; advance clock across grant/proof/session/family or temporary-entitlement deadline without a write; insert/change a range dependency; resume. | Revalidated expiry/derived policy prevents stale issuance; absolute offline lifetime holds; index cleanup does not revive or orphan authority. Covers RI-SES-005, RI-AUTH-001, RI-STORE-001. |
| Q05-R05: approval/retry interruption | Plan/review, then mutate dependency/secret reference/version or file bytes; race reviews/two applies or change approval policy. Commit a mutation then lose its response; retry with same/different request/principal or reduced authority, including expired receipts. Inject local error before commit. | One authorized committed effect/result/credential and success audit; rejected work leaves no partial effects; cached disclosure obeys reviewed policy. Covers RI-MGT-002/003/004/005, RI-STORE-002 and secret-content gap G-04. |
| Q05-R06: peer uncertainty/lease | Pause before and after remote acceptance, lose response, revoke authority/binding, rotate secret or config, expire/reclaim lease, restart and retry. Repeat for SCIM, logout/SSF and mail delivery as applicable. | Honest partial/stale/uncertain state and current authority before subsequent work; conditional reconciliation avoids duplicate mutation where supported; no false remote exactly-once/rollback assertion. Covers RI-CON-002/004, RI-STORE-002. |
| Q05-R07: offboard due work | Two workers claim, old lease expires, one reclaims, then cancel/reschedule/revoke creator/parent or promote target before commit; interrupt/restart before and after commit. | One final local disable/epoch/success audit; lost owner cannot commit; cancellation or inactive authority leaves target protected; the result records the local outcome and per-target downstream intent and claims no remote completion. Covers RI-CON-003 and RI-ACC-002. |
| Q05-R08: old/partial restore | Backup before revoke/rotate/consume/remote acceptance, perform the newer event, stop/fence writers, restore old snapshot; separately truncate/tamper/interrupt restore and rebuild indexes. | Characterize exactly which still-unexpired authority/replay/work returns; no false current-revocation claim; selected recovery policy prevents serving unreconciled state; malformed/partial restore is not served. Covers RI-STORE-003/004. |
| Q05-R09: product/config transition | Carry live proofs/plans/refresh/jobs through supported and unsupported build/config/schema switches; attempt mixed writers or removal of a required capability/verifier. | Preserve shared identity/security semantics or explicit preflight/startup rejection; no silent weakened stage, factor, proof, secret custody or background action. Covers RI-DIST-001/002/003 and RI-WF-002. |

Suggested implementation order:

1. Q02-C01–C05/C08/C09 around existing core/HTTP paths: identity/credential
   classes, permission matrix, proof binding, exact plans/receipts, atomic effects,
   redaction and live policy. Add build adapters when architecture work provides
   them; avoid waiting to preserve existing security behavior.
2. Q05-R01–R05: retain existing one-winner regressions, then test actual
   verification against revocation/expiry and interrupted management responses.
   These are the earliest guards for moving security work outside writers.
3. Q02-C06/C07 and Q05-R06/R07: delayed authority, complete import snapshots,
   external uncertainty, signal disclosure and device/workflow binding.
4. Q05-R08 characterization and Q02-C10 policy enforcement for recovery; implement
   reviewed default invalidation/reconciliation before advertising rollback safety. This is high risk
   and should run alongside the earlier work when recovery changes begin.
5. Q02-C11 and Q05-R09 against actual Essentials/Platform/riauthctl artifacts,
   required before accepting distribution/runtime transitions. Extend the same
   contracts to future browser/admin/workflow paths as they land.

## Dependency reconciliation and validation

Q01 is a reviewable draft, not a RiWork completion decision. Final acceptance
must reconcile A01's reviewed implementation/test/artifact inventory and A02's
reviewed capability/default/transition contracts. Specifically confirm:

- Every promised shared browser account/admin journey reuses these invariants;
  incomplete paths stay roadmap work without reducing Essentials scope.
- Compiled/enabled/configured gates, headless/browser behavior, server/client USB
  separation and Linux x86-64/ARM64 targets match A02. No target/artifact parity is
  claimed from source inspection.
- A01's observed limits and external prerequisites agree with the gap ledger;
  any stronger security claim has enforcement and relevant regression evidence.
- Recovery continuity versus invalidation, secret-content approval and
  cached-result disclosure authority have explicit acceptance decisions.

The dependency drafts became available and were read on 2026-09-27, in the A01
and A02 worktrees identified by the assignment. These SHA-256 digests identify the
read snapshots; later revisions require renewed reconciliation. Both dependency
drafts remain subject to orchestrator review.

| Dependency file | Read-snapshot SHA-256 |
| --- | --- |
| A01 `docs/roadmap/coverage-inventory.md` | `9c41501ebfd3e1d2bb3e6e2d942ae7439577cacf6d8cb3e66c18e1b0ad23fef6` |
| A01 `docs/roadmap/coverage-inventory.json` | `39e3da9b6009880d7c77d6d1f8ea780e73232fdba12efce3d580b23cc7142906` |
| A02 `docs/roadmap/product-contracts.md` | `71ee15c10dbe9bb9de4bf503e6d16d66613266456c95a76566f2bcdb92cdb181` |
| A02 `docs/roadmap/capability-matrix.json` | `a1a91cf64331d52c6555ccd0363f9d9540bca8adabee446936cf744a2b98a2b4` |

A01 maps 94 backlog items and 31 capabilities and confirms the current single
crate/client-server coupling, missing distribution/workflow/admin UI gates,
terminal-bound account paths, separate write engines, local-only offboarding,
redb-only restore and lack of reusable backend/build/interface contracts. Q01
preserves the ten A01 invariants with this explicit crosswalk:

| A01 draft invariant | Q01 stable coverage |
| --- | --- |
| INV-1: disable cascade/non-resurrection | RI-SES-004; RI-MGT-004; RI-CON-003/004 |
| INV-2: request/session proof | RI-SES-002/003 |
| INV-3: one winner/replay family | RI-SES-003; RI-AUTH-002; Q05-R01 |
| INV-4: prepared revalidation | RI-STORE-001; Q05-R03/R04 |
| INV-5: agent/admin and last-admin protections | RI-ACC-002; RI-MGT-001 |
| INV-6: explicit upstream links | RI-ACC-001; RI-CON-001; RI-WF-001 |
| INV-7: durable versus temporary groups | RI-CON-001; RI-AUTH-001 |
| INV-8: online policy revalidation | RI-AUTH-001; RI-PROXY-002 |
| INV-9: strict configuration | RI-DIST-002; RI-AUTH-002 |
| INV-10: schema safety | RI-DIST-003; RI-STORE-003 |

A02's 46 acceptance units and G01–G12 gates agree with this catalog when desired
policy is distinguished from baseline enforcement:

| A02 gate | Q01 stable coverage / unresolved evidence |
| --- | --- |
| G01: identity continuity | RI-ACC-001; RI-DIST-001; RI-STORE-003/004 |
| G02: shared authorization | RI-ACC-002; RI-MGT-001/005; RI-AUTH-001/002 |
| G03: revocation/ordering | RI-SES-004/005; RI-CON-003/004; Q05-R03/R04 |
| G04: credential protection | RI-CRED-001–003; RI-STORE-002/003 |
| G05: one-time/request binding | RI-SES-002/003; RI-WF-001/002; RI-DEV-001/002 |
| G06: sensitive/browser safety | RI-CRED-002/003; RI-SES-001/002; future scanner-safe lifecycle pages require a test that GET consumes no proof. |
| G07: management equivalence | RI-MGT-001–005; exact secret-byte approval and cached-result authority remain G-04 review questions. |
| G08: downstream honesty | RI-CON-002/003/004. Durable per-target deactivation intent commits with every disable. Automatic delivery needs a scoped controller in `automatic` mode, and other targets wait for review. Real-peer acceptance is still required. |
| G09: assembly/capability states | RI-DIST-001/002; actual builds and dynamic discovery are missing. |
| G10: strict configuration/transitions | RI-DIST-002/003; headless conflicts and active/retained dependencies need future preflight. |
| G11: backend/recovery safety | RI-STORE-001/003/004; default invalidation plus persistent-credential reconciliation/rotation is intended, not current restore behavior. |
| G12: native bundles/evidence | RI-DIST-001/003; Linux x86-64/ARM64 artifact and peer/hardware evidence unverified. |

A02 DEC01 remains open: its ordinary OIDC/OAuth source inclusion in Essentials
(E20) differs from A01's provisional all-sources Platform placement (C24). Q01
does not freeze that placement; source security contracts apply wherever supported.
External signing/certificate placement, headless configuration names, bootstrap
ownership, passkey-only lockout prevention, recovery detail and platform minima
also require the A02 DEC01–DEC06 review. A02's default restore invalidation is adopted
here as the proposed safer acceptance contract; it must not be confused with the
baseline's explicit grant-preservation tests.

Only the two security Markdown files are authorized changes. No runtime/security
test execution, new harness, commit, publication or task-status update is part of
Q01. Final documentation validation on 2026-09-27:

| Check | Exact result |
| --- | --- |
| `python3 scripts/check-docs.py` | Exit 0: `Markdown links and build-directory layout checked`. Includes the new untracked Markdown files. |
| `python3 scripts/check-repo-hygiene.py` | Exit 0: `Tracked-file hygiene checked (225 files)`. This checks index blobs; the two untracked additions were separately checked against the same filename/private-key policy. |
| Inline Python reference/coverage validation | Exit 0: 35 unique invariant headings; each has contract, observed/current evidence, missing coverage, Q02 and Q05 references. All expanded invariant references resolve. 134 named regression references resolve to 119 unique test functions in their linked test files. |
| Inline Python hygiene validation of new documents | Exit 0: both new file names allowed by `blocked_reason`; no private-key header matches in either file. No validation script or harness was added. |
| `git diff --check` | Exit 0, no diagnostics. Because the additions are untracked, this was supplemented by the two explicit comparisons below. |
| `git diff --no-index --check /dev/null docs/security/threat-model.md` | Exit 1 with no whitespace diagnostics: expected nonzero comparison result for a new nonempty file. |
| `git diff --no-index --check /dev/null docs/security/invariants.md` | Exit 1 with no whitespace diagnostics: expected nonzero comparison result for a new nonempty file. |
| `git status --short --untracked-files=all` | Only `?? docs/security/invariants.md` and `?? docs/security/threat-model.md`; no tracked changes and HEAD remains the dependency-updated base. |

Rust/security/browser tests, builds, released artifacts and real peers were not
run or verified for this documentation change. PostgreSQL, OpenLDAP,
nginx/Traefik, XML signature tools, real browsers/authenticators/devices, external
signer/connector credentials and deployment fencing/recovery remain prerequisites
for the affected future contracts. The named existing regressions and mocked
peers cannot substitute for those outcomes. Q02 and Q05 remain untouched and todo;
the orchestrator decides Q01 acceptance after reviewing these files and dependencies.
