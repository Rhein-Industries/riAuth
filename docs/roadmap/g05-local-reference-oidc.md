# G05 local reference OIDC cutover rehearsal

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task
`eee83438-ed75-4709-adf9-6807514a6d18`. This is a **bounded local source
rehearsal**, not acceptance of G05. The requested real target application has
not yet been supplied. No production Authentik export, relying party, routing
change, or rollback was used.

## Prior artifact audit

| Artifact | Usable local basis | Boundary still open |
| --- | --- | --- |
| G01 preflight | [migration.md](../migration.md) defines four-way findings and `ready_for_plan`; the converter and fixtures in `tests/identity/operations.rs` cover Authentik imports and non-Authentik inventories. The [D05 audit](d05-acceptance-evidence.md#migration) says G01 integration commits are ancestors, while two source commits cited by its ledger were not ancestors of the D05 read head. | A classification is not an RP result. No real export is recorded. |
| G02 identity continuity | Migration docs and reimport fixtures bind issuer, subject, account ID, and source links, rejecting changed ownership. | No real Authentik export or live RP subject comparison. |
| G03 supported conversions | The migration fixtures exercise exact group bindings and supported scope mappings, and block unsupported conditions or unreviewed flows. | No real application's mapping or browser flow has been compared. |
| G04 re-enrollment | [reenrollment.md](../reenrollment.md) records which passwords and TOTP secrets can move by private reference and which passkeys, sessions, tokens, and recovery codes cannot. | Notices were drafts; no real enrollment, mail, or Authentik rollback sign-in was run. |
| R05 recovery | [R05's two local reports](recovery-drill-r05.md) each record `result: passed` and 16 checks for disposable redb or loopback PostgreSQL. Their JSON names real OIDC/SAML RP sign-in, external dependencies, PITR/failover, and key escrow as external gates. | R05 remains in progress. This test does not rerun restore or close its deployment gates. |

## Reproduce

From this worktree, with enough free disk for a private test build:

```sh
df -h .
CARGO_TARGET_DIR=/tmp/riauth-g05-reference-target \
  CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 \
  cargo test --locked --test g05_reference_oidc -- --nocapture
```

The test uses a disposable redb `Fixture`. It converts a synthetic Authentik
bundle with the same fields and `plan_state`/`apply_state` path as the existing
migration fixtures. The in-process reference RP checks an exact callback,
state, issuer, PKCE code exchange, JWT signature against public JWKS, audience,
nonce, subject, groups, and authentication method. No browser, network listener,
or external Authentik service is started. Passwords and codes stay in the
temporary fixture and are never printed.

## Case evidence

The test emits these case IDs only after their assertions pass. A failed assertion
stops the case sequence and fails the command. The evidence here is the source
assertion and local test output, scoped to the recorded source hash. It is not a claim
about a target application's behavior.

| Case | Assertion and expected observation |
| --- | --- |
| G05-01 import | Report `ready_for_plan=true`, zero blockers, no old sessions or tokens imported; plan and apply accept both private password references; provider discovery publishes the reviewed per-provider issuer. |
| G05-02 denied access | Bob lacks the imported engineering group; authorization fails and authorization-code count stays unchanged. |
| G05-03 successful access and claims | Alice's callback retains `existing=1`, state and issuer match, code redeems with PKCE, and the reference RP validates RS256 ID token against JWKS. `sub=authentik-alice-sub`, `groups=[engineering]`, `amr=[pwd]`; UserInfo has the same subject. |
| G05-04 refresh | A refresh rotates the token. Replaying the spent token returns `invalid_grant`, and access from the rotated family is rejected after replay. |
| G05-05 MFA | A reviewed `require_mfa` policy update denies Alice's password-only session. After local TOTP enrollment revokes that session, password plus TOTP reaches the RP with the same subject and `amr=[pwd,otp]`. This is new riAuth enrollment, not an imported Authentik factor. |
| G05-06 recovery | A newly issued riAuth recovery code signs Alice in once; reuse fails; RP subject stays the same. It is not an Authentik recovery token. |
| G05-07 logout | Logout revokes its riAuth session and access; a separate recovered session survives. No front- or back-channel RP logout is claimed. |
| G05-08 rollback boundary | The reference RP restores its saved local JSON route file byte-for-byte to the Authentik backend while keeping the issuer byte-identical. This checks a disposable config file only. It does not change a real route or prove Authentik credentials, sessions, or tokens still work. |

Observed on macOS 26.2 with `rustc 1.98.1`, from base commit `a3ccbf1`
plus this worktree's test source (SHA-256
`64e43530f6c6e69c2fe4f7f9d45fabb2cfb5b1843a6b022eec2a418008602ba6`):

```text
G05-01 preflight ready=true blockers=0; plan/apply accepted; discovery issuer matched
G05-02 denied bob; authorization-code count unchanged
G05-03 callback state/issuer/PKCE and JWKS JWT accepted; sub=authentik-alice-sub groups=engineering amr=pwd
G05-04 refresh rotated; replay invalid_grant and family access rejected
G05-05 password-only denied after require_mfa; TOTP sign-in accepted with stable sub and pwd+otp
G05-06 riAuth recovery code accepted once; reused code denied; RP sub stable
G05-07 logout revoked session and access; separate recovered session survived
G05-08 saved RP route file restored to Authentik backend; issuer unchanged; peer sign-in untested
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Real application runbook and acceptance gaps

Before an actual cutover, record the target RP's product/version, owner, current
issuer, redirect and logout URLs, subject and claim expectations, MFA policy,
refresh behavior, routing switch, and rollback owner. Export a complete
Authentik bundle privately and resolve every preflight blocker. Apply a reviewed
plan in isolation, then run each case above through that RP's real browser and
callback. Record request IDs, redacted responses, the RP's accepted issuer and
subject, and timestamps without saving tokens or secrets in this repository.
Keep Authentik and its configuration available. Rehearse restoring the saved RP
configuration and route, signing in with an Authentik credential that stayed
there, and invalidating or accounting for riAuth and RP sessions created during
the trial. Run R05's deployment restore and external gates separately. G05 can
be judged only from those peer-specific observations; this local case does not
mark it complete.
