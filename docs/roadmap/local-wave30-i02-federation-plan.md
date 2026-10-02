# I02 original federation scope: read-only review and one proposed continuation

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original I02 task
`cbe1e83d-b58c-4be7-ba6c-5fb8a73c9839`. Primary worktree remains
`a2dff16a-c4b0-47fc-96de-ac85b1fb6d9e`. This support review used existing
worktree `42bb51c6-c198-4adb-bd92-0a5222853231`, branch
`roadmap/local-module-boundaries-wave27`, parent
`c55a7d2a028ae2ec542749f45f00aa3249f68613`. Sole write: this report.

All current source references below mean fixed published
`9a819317efb3a13fa27cd86f884be2be00898fc0`, read through Git objects.
No merge, source/doc-recipe edit, runtime, build, test, desktop, external service,
worker contact, board mutation or push was performed. A03 remains DONE;
D01's separately planned confidential application journey is not duplicated.

## Original row and recommendation

The live RiWork row was reread using `task list --project
891e7443-8dac-4c1b-897f-9e53cb59c7ee --json`, filtering the exact UUID. It was
`in_progress`, assigned to the original primary worktree. Requested outcome:

> Cover source setup, verified linking, assurance mapping, browser completion, and credential/key rotation.

Workstream goal:

> Turn supported protocols into complete, tested integrations.

Original completion gate:

> Every advertised integration has a working setup, lifecycle, and failure-handling path—not just an endpoint or protocol module.

Prerequisites named by the row are A06, Q01 and Q02. Its evidence clause requires
review of implementation, tests, documentation and released artifacts as
applicable, with actual verification, remaining gaps and external prerequisites;
documentation or a worker report alone cannot establish implementation completion.
The old scheduling paragraph is retained in the row; the current instruction
authorizes this support report only. Root owns subsequent starts and statuses.

**Recommend retaining I02 in_progress for one concrete local browser continuation
gap.** An advertised Platform interactive `source_stage` that requires local TOTP
returns JSON after its upstream callback, without a page that can submit the factor
and finish the original browser authorization. The ordinary source-review page
already has an OTP field; it is a different, credential-bound transaction and is
not the missing adapter. This finding applies the original browser-completion and
working-lifecycle clauses. It does not make a named tenant, physical authenticator,
every-host rerun, human study or release certification an additional I02 gate.

## Current implementation against the five requested parts

| Original part | Directly inspected implementation and assertions | Disposition / limit |
| --- | --- | --- |
| Source setup | `src/source.rs:19` defines canonical Source/OAuthProfile/SourceInput/SourceSpec records; `assembly/source_runtime.rs:132` validates exact endpoints, auth methods, pinned keys, bounded scopes/groups/ACRs and OAuth mappings. `source_catalog.rs:104` runs the shared management writer; `management.rs:2850` checks live authority before secret/profile writes, revocation and audit. `tests/identity/sources.rs:228` asserts missing-secret atomic refusal, versioned-secret apply, a subsequent no-op plan, redacted export/audit, scoped listing and agent refusal for administrator source login. | Implemented setup and failure paths. OIDC/OAuth sources are shared; SAML is Platform-gated. No fresh Essentials execution is asserted here. Source-group validation/writer calls are preserved; this review does not resolve the held Group-materialization work. |
| Verified linking | `source_runtime.rs:326` requires fresh local identity plus the enrolled factor for a link start. `source_finish.rs:79` rechecks live target/source, never chooses a user by email, and calls `management/source_links.rs:39`, whose Plan/VerifiedLogin authorities re-read source/account, validate exact subject and refuse reassignment. `portal_sources.rs:284` also binds the original browser account/session and completes delivery in the same transaction. `tests/identity/sources.rs:76,129,541` assert account collision refusal, planned/verified ownership, exact audits, fresh-session checks, delivery rollback and one-use completion. | Implemented, with actual historical passing function lines described below. Linking/unlinking is not permanent membership; only the link's source sessions are revoked. |
| Assurance mapping | `Source.trusted_mfa_acr` is an explicit bounded configuration, contrary to the older coverage-inventory statement that no mapping exists. OIDC `verify_identity` (`source_runtime.rs:935`) validates signature/issuer/audience/azp/nonce/time/at_hash before consulting that set. SAML `verified_identity`/`identity` (`source_saml_runtime.rs:394,513`) bind response/assertion to issuer, ACS, request, audience, signed stable NameID and fresh authentication, then map the signed context at line709. OAuth `oauth_identity` at1082 deliberately sets auth_time0/MFAfalse. `source_finish.rs` retains federated/mfa/otp provenance. | Implemented explicit MFA trust, not an arbitrary assurance-expression engine. OIDC/SAML fixtures assert accepted trusted MFA; OAuth fixture asserts no invented MFA/freshness or email link. Embedded-stage local OTP asserts no hardware/phishing-resistant AMR. |
| Browser completion | `portal/sources.rs` routes start/review/finish/unlink; `portal/source-login.js` navigates to the provider, and `portal/sources.html`/`sources.js` show the explicit account review, local OTP, confirmation, cancellation and safe errors. `assembly/portal_sources.rs` keeps the one-use credential in HttpOnly cookie state and removes the internal bearer before browser delivery. OIDC callback binding is checked before exchange; SAML uses ACS then same-site return with both cookies. The inspected HTTP/Core fixtures cover foreign/missing cookie, replay, rollback and success. | Standalone sign-in/link browser adapter exists. These HTTP/Core assertions are not a fresh rendered upstream-browser run. The separate embedded stage's local-factor continuation is the one proposed gap below. |
| Credential/key rotation | `management::write_source` distinguishes profile/pin changes from secret-only replacement. Profile changes revoke source sessions; only its own certificate-pin revocations get pin_retired. `source_callback_claim` retires presented stale/disabled state before exchange; `source_callback_record` rechecks the fingerprint after I/O. OIDC and SAML rotation fixtures inspect overlap, cutover, stable exact links, stale assertions, old-state replay after restoration and unaffected local administrator sessions. OIDC also tests replacement during exchange and secret-only replacement preserving the current session. | Implemented trust lifecycle with recorded executions; the last secret-only assertion does not perform a fresh exchange against an independently rotated tenant credential. No such tenant execution is credited. SP certificate/key matching and metadata/start are separate from IdP pin retirement. |

CLI counterparts were inspected, not executed: `src/cli.rs:1184,1855` exposes
source put/list/metadata/start/review/finish/links/unlink/stage-cancel. Start writes
the credential privately; finish checks the transaction issuer, accepts OTP via
private input, saves the session privately and removes it from printed output.
`crates/riauthctl/src/source.rs` has source list/put only: bounded regular input,
owner-only secret files, zeroization and the shared revision/idempotency writer.
It does not claim source-login or stage-resume commands. `docs/oidc-profiles.md`,
`docs/saml.md`, `docs/enterprise/ENT-11.md` and both upstream source recipes were
read as the advertised contract, not as proof of a new run.

## One prospective slice: embedded-stage local-factor browser continuation

Exact static dataflow at the fixed source:

1. `src/api.rs:1977`, `source_stage_redirect`, sends a successful stage callback
   to GET `/oauth/source-stages/{id}/resume?authorization_id=…`.
2. GET at2005 accepts only StageReference (deny_unknown_fields) and calls the
   existing `Core::source_stage_resume(..., None)`. POST at2016 accepts JSON
   authorization_id plus optional otp. `stage_result` at2036 produces redirects
   or the existing form_post callback only on completion; a waiting result is JSON.
3. `assembly/source_runtime.rs:676`, `resume_stage`, checks stage use/expiry,
   immutable request hashes/binding, source and exact linked account. At726–735
   a linked user with local TOTP and an upstream non-MFA proof returns
   `local_factor_required`, `code_issued:false`, before login/factor consumption.
4. `tests/source_stage.rs:3376` drives precisely that HTTP path: GET factor
   parameters (including empty/encoded otp) return400 without a code; clean GET
   returns200 JSON local_factor_required; a subsequent JSON POST with OTP redirects
   and its code has AMR federated+otp and no hardware/passkey assurance. This is a
   reviewed source assertion, not an execution performed by this support review.
5. `docs/oidc-profiles.md:179` explicitly records no browser OTP form or dedicated
   CLI stage-resume command. The callback reaches raw JSON in a normal browser;
   the documented JSON POST remains a functional API workaround.

**Propose only a browser continuation adapter for this existing stage**, with a
factor prompt, retry/cancel/expired result and final callback handling. Keep the
existing Core stage/factor/authorization implementation intact. Preserve current
JSON GET/POST clients and the rejection of every factor-in-query variant. Do not
call standalone source finish, mint a terminal credential, start another request,
accept an arbitrary account or relax configured-workflow admission.

Prospective file/ownership request, not a reservation: narrow stage-handler
delegation in `src/api.rs`; new `src/portal/source_stage.rs`,
`src/portal/source-stage.html` and `src/portal/source-stage.js`, with additive
module wiring in `src/portal.rs`; one focused fixture in `tests/source_stage.rs`;
the single limitation paragraph in `docs/oidc-profiles.md` only once corrected.
Root must coordinate the
API/portal ownership with the original I02 primary lane before source work.
No source/private-helper, management writer, workflow factory/executor, I10 Entra
diagnostic or D01 confidential-application implementation is part of this claim.
The accepted configured source→current-TOTP bearer adapter is session-only
reauthentication; it is not this initial interactive stage continuation.

The transport needs explicit design review before implementation: the ordinary
portal CSP has form-action none and its write guard requires same-origin plus the
custom portal header (`portal/http.rs:394,468`). A blind fetch of the existing
redirecting POST is not a safe browser handoff plan. Preserve both query and
form_post response behavior through the existing callback renderer
(`src/response.rs:83`), without weakening the global portal CSP/guard, following
an upstream redirect in a fetch, or logging/exposing OTP or completion credentials.
If that needs shared runtime/storage changes, report the exact seam before
broadening this prospective adapter claim.

One bounded completion path after root approval: add the adapter, then run only
its focused HTTP/rendering assertions for HTML factor prompt versus unchanged
JSON, wrong factor and charged retry, exact successful account/request/code,
cancel/expiry/replay refusal, query-factor rejection and query/form_post handoff.
Reuse the existing Core assertions in
`resume_stage_charges_bad_factor_before_one_use_completion` (2818–2942): bad OTP
charges one attempt while stage.used remains false; success consumes once, emits
one audit and one code; replay leaves the committed snapshot unchanged. A later
separately authorized Driver browser checkpoint may establish rendered completion;
none is authorized or claimed here. This is one local adapter proposal, not an
instruction to rerun every protocol suite or to close I02 after a graph count.

## Executed evidence reused, with limits

| Evidence | Actual result / provenance | What it establishes |
| --- | --- | --- |
| Current selected integration run37006213470 / job110834995871, checkout `88790deb62d32c84fa17dceb12cd93a727224e94` | Raw `/tmp/riauth-wave30-ci-37006213470-110834995871.log`, 265774bytes, SHA256 `0d327b38cb4999bedc20df3f739eafac161baa6d571aa3a81a4d139189c67859`, rehashed and relevant command/result lines read. `RIAUTH_TEST_XMLSEC="$(command -v xmlsec1)" cargo test --test identity --locked saml_source_independent_xmlsec -- --ignored`:1 passed/177filtered,5.81s. | Actual independent xmlsec signatures plus the full exercise body: metadata/signing, explicit linking, no email selection, signed context mapping, rejection cases, encryption, bound SAML-stage issuance/replay and unlink. In-process Core orchestration, default Platform; not a named IdP, rendered source page or physical factor. |
| Earlier selected integration run36950097067 / job110661000640, checkout `2d05c00deed3a6dafcd458a5f1a1a9beb17d0dd8` | Raw `/tmp/riauth-wave29-integration-110661000640.log`,265770bytes, SHA256 `458a30895b5987c0bebe2fb7368a1a32969fa0d41ce90acbfa56f43299ccd186`; same selected SAML-source function passed. Historical repeat, not another independently counted acceptance case. | Earlier independent-signer evidence; its separate check/TOTP failure is not erased by selected integration success. |
| Mixed earlier check run36951643190 | Raw `/tmp/riauth-wave29-failed-36951643190.log`,256279bytes, SHA256 `5a94f6f07d83ae7f4565d8fb032c3a629f31b44cff99ac84e67f7c0078d376f8`; actual command cargo test --all-targets --features test-support,fuzzing --locked. Contains11 sources_tests pass lines and10 saml_source_tests pass lines, including both rotation functions, browser cookie/ACS/rollback, source-manifest/ownership/OAuth checks and disabled-state retirement; also the two source-logout pin-retirement tests pass. Identity aggregate171passed/7failed/3ignored; outbound SCIM policy failure remains. This downloaded check log has no checkout block: its exact source commit is not reconstructed from a test name or timestamp. | Actual historical assertions executed in a failed larger job. Neither all-green CI nor a fresh pass at the fixed source. No claimed source equivalence to that unpinned checkout. |
| Accepted earlier I02 retirement slices | `local-wave28-task-closure-audit.json` records accepted `94ffa6062fc6982e985aa28f9b56c500271cfde8` (source `cfb85ed0182c0adadd92ada88d661d1c3bf2da1b`), `ab5743eef83d0e47bcbe671fa748563653056a1a`, `c2037b5e1d5d6d221db25ee50381eed81da60106`; focused OIDC/SAML disable, source-link/trust and logout tests passed. The source/authority behavior and assertion bodies were inspected in current assembly/writers; accepted commit diffs/provenance were read. | Historical reviewed retirement evidence, not freshly replayed tests or proof of a named peer. The narrower pin_retired predicate must remain, including unrelated prior revocations and old-request/index reuse refusal. |
| Accepted source-TOTP API report | `local-management-source-totp-api-report.md`: code `af3f0040df7de5a39ba264143c2e524a67694a91`, follow-up `bf1a152a5b9136344dbd462c97ae001912a6f064`; seven focused test-support cases passed at the latter, plus reported Clippy/fmt/docs checks. The report and current handler/wiring were inspected. | Existing session-only bearer start and HTTP continuations, refusal/rate/ignored-header semantics; not an embedded-stage page or fresh tenant/browser execution. |

The current selected XMLsec result can be tied to current reviewed assertions:
17 explicitly compared source/UI/CLI/test blobs are identical between fixed9a819317
and checkout88790deb. This includes the complete identity helper/source/SAML-source
files, source-stage fixture, ordinary browser source files, source runtime,
callback/finish/live-identity/SAML verifier, shared source-link writer and standalone
client source module. This equivalence does not make other functions selected by
that CI filter, nor does it prove whole-tree or all-backend equivalence.

The user-recorded subsequent full check FAIL (11pass/1fail,7.06s) remains a separate
limit; this review did not obtain its raw log or assign it a failed-test cause.
Selected integration success is not full-check success. Historical Platform SAML
guide preparation imported keys/metadata and performed source put/list/start, but
did not execute source finish/ACS/browser completion; the D01 report records that
limit. Own D01 operator commit `9cde81dd93ff171fa54194b2ed514142451484d9`
and independent password-browser report
`6837b745576552b0917b3bd1f1424515621a0a85` are not upstream federation,
application consent, physical passkey or source-key-rotation evidence. Real RP,
LDAP, PostgreSQL and proxy executions are also not substituted for those steps.

No live customer/Okta/Entra/Google tenant, hardware authenticator, Windows install,
official released binary or deployment evidence was created or inferred. Unsupported
encrypted upstream ID tokens/private_key_jwt and SAML transient/unsolicited/artifact/
SOAP/ECP profiles are not promoted to requirements by this report. The proposed
gap is local and reachable without tenant access.

## Exact source anchors and static checks

| Fixed source object | Git blob / exact bytes |
| --- | --- |
| `src/api.rs` | `cd552144cfe23397f65fd97d3cbd3466805dd9cc`; complete stage redirect/reference/resume/result section1977–2047 SHA256 `fe04c9c6ff7f3a6d9a4a864e53762e9aa9fe5ffac7509ac85f392493e7a53e32` |
| `src/assembly/source_runtime.rs` | `51a46db7a7ad3e46914146461c79c6ae42ab124a`; complete resume_stage676–808 SHA256 `97d1b2eebf081a12530065b0290875a7b4571523c5f297f982cbdc3223b2ae23` |
| `tests/source_stage.rs` | `e124d964f3cb4c0ed680e98eebbd320d2a11c5ee`; complete local_totp test3376–3480 SHA256 `23a3c0f0b5382b8ff92ec69ca197f053da09bf5a249c9de6d3c65b841a49fd29` |
| `tests/identity/sources.rs` / `tests/identity/saml_source.rs` | `0f38b0c0dff9921dd123b560c55a6c1cb5e1fcf6` / `e02ac9f9e79a9b9be026c779ec2343fffd1798a1` |
| `src/assembly/portal_sources.rs` / `src/portal/sources.js` | `1587611ee4355f776d7cd0b7c0a818c5969ba9af` / `c66478b771419b0adb359829e3c4afa015531ddf` |
| `src/management.rs` / `src/management/source_links.rs` | `8ff588258fd859949015604518003c5950bcd2ad` / `c4fc52071026f3acac5f5d040b906070ebec7aca` |
| `src/cli.rs` / `crates/riauthctl/src/source.rs` | `581b8ee69ff73b1512fa583cc6be5dbc9f0745ee` / `81f0e7c933d8caa4c9129c10f5ad79b6ab45819e` |

Body hashes cover inclusive line slices with original LF bytes, without line-number
decoration. Actual checks in this review: live task read; clean branch/HEAD read;
fixed Git object/path and relevant body inspection; historical accepted diff/report
inspection; the17 blob comparisons; three raw-log SHA256 recomputations and selected
command/test-result inspection; the three exact body hashes. Source definitions
and planned future checks are not new runtime passes.

The inline Python report check passed:23 pinned path references,23 existing Git
objects, three exact body hashes, final LF, balanced fences, no trailing whitespace
and sole-file worktree scope. `python3 scripts/check-docs.py` exited1: no Markdown
link error; only the five pre-existing output directories target-wave29-source,
target-wave28-scim, target-wave28-portal, target-wave28 and target-wave27. No checker
change, deletion or cache cleanup was made. Staged `git diff --cached --check`
passed, and the final staged name check contained this report alone. The report
checks were repeated after adding this evidence. These are static documentation
checks, not Cargo or protocol executions.

All accepted protections remain dependencies: exact source/subject/issuer and
token/nonce/state binding, authority-before-writes/replay, one-use proof, transaction
and audit order; reviewed client-creation receipt-secret recovery; route-specific
optional/required headers; PAM fallback; held Group behavior; at-least-once remote
I/O and existing60s lease limits. Canceled accessibility work is not a completion
gate. Desktop preference remains RiWork Cua.ai Driver only, descriptions/current
state before interaction; this review used no desktop provider.

Root should review this concrete interpretation and prospective adapter scope,
then decide whether to start work in the original I02 lane. This report neither
reserves implementation files nor changes the task's status.
