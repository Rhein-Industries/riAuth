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

## Approved transport-design appendix: wave30_I02_browser_transport_design

Report-only continuation of `3d5855623ff5cc1310a889d41768771bb8533aad`.
Root accepted the embedded-stage gap for design, not implementation or execution.
All design anchors still use fixed `9a819317efb3a13fa27cd86f884be2be00898fc0`.
The original report, failures, limits and in_progress recommendation above remain.
This appendix resolves the earlier transport question; source/runtime stay HELD.

### Chosen transport: existing cookie-bound browser delivery

Use the original browser authorization's existing return cookie and delivery
route, not a new completion credential or a fetch to the relying party:

| Step | Exact proposed behavior |
| --- | --- |
| Provider callback | Leave `source_stage_redirect` and both OIDC/SAML callback bodies unchanged. They still redirect to the existing stage GET with authorization_id. |
| Existing stage GET, JSON client | Keep StageReference extraction, deny_unknown_fields and the current source_stage_resume(None) → stage_result behavior. Missing Accept, wildcard, explicit application/json or disabled browser_ui stay on that branch. Existing JSON POST/cancel are byte-unchanged. |
| Existing stage GET, explicit HTML navigation | After query extraction, perform a read-only load_source_stage(stage_id, authorization_id). Only a stage whose stored request.request_binding is Some(authorization_id) gets a303 to the same-origin path `{base}oauth/resume/{authorization_id}/source-stage/{stage_id}`. A non-browser stage (binding None) keeps the legacy result. A mismatched stored binding fails; it does not fall back to another authorization. No factor or stage use is consumed by this handoff. |
| New page GET | Read-only: validate the exact stage pair and original browser with the preflight below and render the empty page. Do not call Core.source_stage_resume or cancel from this new GET. Its script makes one guarded JSON POST without otp to learn pending/local_factor_required or finish a proof needing no local factor. An already-ready, cookie-authenticated browser result can hand off to the original native delivery route without spending a factor again. An error keeps its status and cannot become a completion. |
| New guarded resume POST | Accept JSON containing only optional otp, reject every query parameter, and use unchanged browser_write_guard before preflight/Core. Call the existing public Core.source_stage_resume exactly once. Return200 JSON for waiting or for a local continuation; never emit an HTTP redirect or relying-party HTML to the fetch. A Core error remains an error response, including the already-committed bad-factor charge. |
| New guarded cancel POST | Same cookie/pair/query/Origin/header preflight; empty JSON only. Call unchanged Core.source_stage_cancel once. Return local-continuation JSON with outcome cancelled and code_issued false, not a successful authentication label. |
| Native final navigation | After an accepted continuation response, JavaScript calls location.assign only on the exact server-built original `{base}oauth/resume/{authorization_id}`. That existing GET checks the return cookie again and uses browser_resume_with → browser_response → unchanged response::callback. Query uses its normal302. form_post uses its existing escaped form, nonce script and destination-specific CSP. This is a full navigation, never a fetch following a redirect. |

This also restores the existing browser-owned SSO delivery: stage success already
records the session on the browser authorization, and browser_resume_with points
that browser at it while removing the pending row/terminal user-code index and clearing
the return cookie. The transport itself creates no bearer, credential, account,
authorization request, storage row or extra callback capability. Normal existing
Core source completion remains responsible for its current internal session/code
writes. Use an already-linked non-administrator in the proposed fixture; do not
require auto-provisioning or Group changes to demonstrate this flow.

The loopback cookie-path detail is essential, not a proposed cookie change:
browser_start sets the HTTP return cookie on `/oauth/resume/{id}` (plus issuer
base), whereas HTTPS uses the existing __Host cookie on `/`. A page or POST under
`/oauth/source-stages/` would miss the loopback return cookie. Placing all new page
and mutation routes below the original resume path works with both unchanged
cookie schemes. No SSO/return-cookie cloning or widening of its Path is proposed.

### Exact prospective paths and minimal hunks

No hunk below has been applied, compiled or executed. Implementation would require
root's separate reservation and source alignment. The proposed product paths are
only `src/api.rs`, additive `src/portal.rs`, new `src/portal/source_stage.rs`,
`src/portal/source-stage.html`, `src/portal/source-stage.js`, and one appended test
in `tests/source_stage.rs`. No existing source recipe/guide edit belongs to this
design approval. The previously proposed oidc-profiles limitation correction is
held with implementation; it is not part of this appendix's write scope.

1. `src/portal.rs:8`: add only a Platform-gated crate-private module declaration:

   ```rust
   #[cfg(feature = "platform")]
   pub(crate) mod source_stage;
   ```

2. `src/api.rs:810`, in the existing Platform router, add
   `.merge(crate::portal::source_stage::routes())` after the stage cancel route.
   In the existing source_callback rate arm at1076, add exactly the condition
   for `/oauth/resume/` paths containing `/source-stage/`. Those routes use the
   existing source_callback bucket; no new rate/agreement entry, limiter, permit,
   workflow label, header policy or admission rule. The source-totp/source-passkey
   source_start arm remains unchanged.

3. `src/api.rs:2005`, add HeaderMap extraction only to the stage GET. StageReference
   and existing JSON POST/cancel bodies at1993–2034 remain intact. The intended
   handler hunk is:

   ```rust
   // After the unchanged Query<StageReference> extractor:
   let html = app.core.config.browser_ui
       && crate::portal::source_stage::wants_html(&headers);
   let page = app.clone();
   let response = app.run(move |core| {
       if html && let Some(path) = crate::portal::source_stage::page_path(
           core, &id, &query.authorization_id,
       )? {
           return see_other(&page, &path, vec![]);
       }
       stage_result(core.source_stage_resume(&id, &query.authorization_id, None)?)
   }).await?;
   Ok(vary_accept(response))
   ```

   page_path is read-only: use the existing crate-visible
   `assembly::load_source_stage` in core.store.read; check the stored request
   binding as specified above. Build local paths with encoded path segments from
   the matched stored identifiers and Core.cookie_path; no user-supplied next,
   callback, redirect_uri or issuer. Vary:Accept is added to GET variants for the
   newly negotiated representation; legacy JSON status/body and execution order
   otherwise stay unchanged. JSON POST/cancel receive no representation change.
   wants_html is a small route-local selector: require a positive text/html media
   range; explicit positive application/json wins; q=0 or malformed quality does
   not opt into HTML. Do not broaden or rewrite the global accepts_html helper.

4. `src/portal/source_stage.rs`: implement only the following routes and adapters:

   | Route (issuer base omitted) | Input / handler |
   | --- | --- |
   | GET `/oauth/resume/{authorization}/source-stage/{stage}` | NoQuery with deny_unknown_fields; read-only preflight, empty HTML page or native local handoff for an already-ready result. Browser_ui and positive HTML required for the page; no Core resume/cancel call. |
   | POST same path + `/resume` | NoQuery; JSON BrowserResume { otp:Option<String> }, deny_unknown_fields; unchanged browser_write_guard, preflight, one Core call, JSON-only outcome. |
   | POST same path + `/cancel` | NoQuery; empty deny_unknown_fields JSON object; same guard/preflight, one Core cancel, JSON-only outcome. |
   | GET `/portal/assets/source-stage.js` | Serve the static script, with browser_ui gate. |

   The new POST routes require browser_ui and positive application/json Accept;
   they do not negotiate a redirect. Query extraction rejects empty, encoded or
   ordinary otp, authorization_id and every other query field before any Core
   call. The pair comes exclusively from the route, not the JSON body. New request
   records are not accepted. Invalid/duplicate Origin, missing/duplicate custom
   header or non-same-origin Fetch Metadata are refused by the unchanged guard.

   Preflight, before a factor/cancel write: (a) get the original return cookie via
   api::binding_cookie(core, headers, "return", authorization), retaining duplicate
   cookie rejection; (b) call Core.authorize_state(authorization, binding, sso)
   to authenticate the private pending browser row and its expiry, without
   exposing that returned state (passing None for sso is sufficient; do not treat
   an SSO cookie as the return binding); (c) load the exact SourceStage pair and
   require request.request_binding == Some(authorization) and the existing
   expires_at > now, expires_at <= now+600 bounds. In the undecided case require
   status unavailable, error source_stage, continue null and unused/uncancelled
   stage for both the page and a new POST; Core still rechecks its complete binding
   and expiry in the write. A page in that state is an empty read-only prompt:
   it does not duplicate private login/identity/factor logic to guess readiness. Require callback-ready
   state to have status complete and continue exactly the original resume path.
   A ready result is recoverable only by native GET delivery, not another factor
   or cancel invocation. Do not deserialize a private Pending projection, widen
   fields, add a Core helper or call browser_resume during this preflight.

   Add Cache-Control:no-store to new private HTML/JSON/handoff/error responses
   and the old GET's new HTML handoff, retaining no-referrer. The static asset
   follows the existing asset policy. The new module builds its own303 only from
   the matched local path and configured issuer origin, using the same local-path
   checks as api::see_other1579–1593; that helper is private and stays unchanged.
   Do not widen its visibility or route through a client-supplied target.

   Result classification is closed: pending/local_factor_required with code_issued
   false → JSON retaining that exact status, so the script can distinguish waiting
   from a factor prompt; complete with code_issued true and redirect_uri → local
   continuation with outcome complete; rejected/cancelled with code_issued false
   and redirect_uri → local continuation retaining that failure outcome. Unknown
   or internally inconsistent results fail closed. A Result::Err is never fed
   to that classifier as a success. A GET recovery requires the exact pair plus
   authenticated Core state with continue equal to the original local resume path;
   after delivery removes Pending, recovery/replay fails normally. Do not extend
   expiry, reset used/cancelled, infer acceptance from an error or add fallback
   login. Missing stage/browser state yields an ended/error result.

5. `src/portal/source-stage.html`: use portal_html with the current global CSP,
   no-referrer policy and static external scripts. Render only response::escape'd
   verified stage/authorization identifiers in data attributes, the server-built
   local paths and an empty authenticator/recovery-code field (maxlength128,
   autocomplete one-time-code). No upstream authorization URL, nonce/state,
   source poll credential, return/SSO cookie, session token, RP callback/code or
   entered OTP is rendered into URLs/attributes/logs. The UI form is prevented
   from native submission and the input has no name for URL serialization; action
   buttons are type button with RiAuth.guard handlers. A submit event only prevents
   default, never adds another unguarded write path. Browser writes are JSON calls.

6. `src/portal/source-stage.js`: use existing RiAuth.post, inFlight and guard,
   with one page-level busy flag shared by initial check, factor, recheck and
   cancel (inFlight alone keys individual buttons). On initial load make exactly
   one guarded post(resume,{}) to learn pending/local_factor_required or complete
   a proof needing no factor; show the factor field only for local_factor_required.
   Submit only that field to the page's own resume endpoint, clear it on completion
   or refusal, and submit an empty object to cancel. Default retry remains false.
   RiAuth.post already uses same-origin credentials/mode, redirect:error, no-store,
   explicit JSON Accept and X-Riauth-Portal:1. Accept continue only when it exactly
   equals the page's server-built original resume path; navigate normally. Never
   fetch that path/RP, inspect opaque redirect headers, insert returned HTML,
   document.write, create an RP form, or construct a code/error callback. On
   ambiguous network failure, clear the input and offer reload of this original
   page to recover any already-decided result; do not silently repeat the OTP or
   claim cancellation. pending offers one explicit guarded post(resume,{}) recheck,
   not timed credential polling or a new request. Native page reload first reads
   the original browser state and recovers an already-decided callback; it never
   repeats the entered factor. Refusals keep their safe error text/status and do not
   manufacture a continuation. No passkey/application/CLI/browser-compatibility
   feature or new generic UX framework is proposed.

**No route-local CSP is needed.** The factor page never posts an RP form. Its
guarded JSON exchange has no redirect to follow. The existing browser-resume GET
is a separate document navigation, so response::callback's existing form_post
response installs its own correct nonce/destination CSP. portal_html, its global
form-action none, browser_write_guard and response::callback/form_post stay
byte-unchanged. Query state/code and form_post hidden protocol fields appear only
where the unchanged callback renderer already delivers them to the validated RP.

### Why these reads and calls retain the original binding and failure order

* `browser_start` (browser_runtime73–243) constructs the browser id, sets that
  request.request_binding, and passes the same id as StageStart.authorization_id
  and browser_id. It stores the identical original request in private Pending,
  with stage_id and the digest of its independently generated return cookie.
  Git caller inspection finds only browser_start and authorization_prepare
  (assembly/oidc752–760) constructing StageStart. The latter uses a newly generated
  ri_az authorization id with browser_id None, not an arbitrary existing browser
  id. Neither new route accepts StageStart or an Authorization record; the new
  preflight requires the browser constructor's stored binding. No caller can
  substitute its own request by posting an authorization_id in the new body.
* `authorize_state` (385–395) uses a read transaction and private interaction
  (1622–1637), which requires a live Pending and constant-time equality between
  digest(presented return cookie) and its browser_hash. It invokes no workflow
  continuation, factor, authorization or callback write. New transport cannot
  exchange an A cookie for B's browser id, and load_source_stage (source_stage13–26)
  separately requires the exact stage id and authorization id. Existing SourceStage
  public-to-crate request binding is sufficient; no private browser_id access is
  added. Core rechecks that private browser_id == request.request_binding itself.
* `resume_stage` (source_runtime676–808) rechecks used/cancelled, expiry including
  its upper bound, suspension_hash, request_hash and private browser binding.
  stage_resume_login (source_stage140–151) binds login to stage/source/nonce and
  its live expiry/failure status. It reads the enabled source, exact subject link,
  bound expected account, live client and local/upstream factor requirement;
  complete_source_login rechecks the fingerprint, account and authority. No
  caller-supplied request, subject, identity or assurance flag reaches this path.
* The browser guard read precedes the existing Core write; it is not falsely
  described as one combined transaction. At this source, browser_hash is assigned
  only at construction and never rotated in Pending; the private stage/request
  association is constructed together and cannot be changed by a caller's API
  input. The final existing stage_browser_callback (1498–1514) rechecks the same
  browser row's presence/expiry within the stage write. Removal/expiry after
  preflight fails that write and rolls it back; concurrent stage use/cancel is
  rechecked under the existing writer. There is no new mutation of either binding.
* Bad OTP returns outer Ok(inner Err) after attempts is persisted in
  complete_source_login (assembly/source_finish174–198). source_stage_resume201–210 commits the outer
  transaction before exposing that error. New transport calls this wrapper and
  translates its already-returned error afterwards; it must not wrap it in a
  different store.write that would roll back the charge. Store::write1047–1075
  commits only after the closure's outer Result succeeds. Outer failures roll
  back partial factor/link/session/code/delivery writes; inner charged failure
  commits the attempt while stage.used stays false, as the existing fixture shows.
* Successful final authorization uses the stored request and transaction; stage
  use, code, source audit and stage_browser_callback share the original Core
  write. authorize_session failure goes through reject_stage/stage_denial,
  consumes that original request and stores an error callback with code_issued
  false. Cancel likewise marks used/cancelled and stores an access_denied callback.
  The adapter preserves these outcomes instead of changing an error into code.
* Native browser_resume_with254–310 checks the return cookie again, delivers only
  Pending.callback, points the browser at its recorded session if any, removes
  the one-use pending/user-code/proof state and clears its cookie. browser_response
  (api1547–1567) calls the untouched response::callback. A foreign cookie, mixed pair,
  replay or deleted/expired browser row cannot acquire a different authorization
  or a new code. Ordinary source finish and W02 configured/bearer factories are
  never called by this adapter.

The source-level reasoning assumes the existing private persisted-record contracts;
it does not claim new concurrency, PostgreSQL, tenant or browser execution. The
focused fixture below includes a removed/expired pending-row refusal and corrupted
stage bindings to make those refusal expectations explicit. If implementation
uncovers a writer that can change the browser hash/request association on a live
source stage, stop and report that concrete seam rather than weakening preflight
or changing held Core/storage logic.

### One deterministic appended HTTP/rendering test and one future command

Append exactly one function to `tests/source_stage.rs`:

```rust
#[cfg(all(feature = "platform", feature = "test-support"))]
#[tokio::test]
async fn browser_source_stage_transport_binds_factor_and_renders_both_response_modes() {
    // One table-driven function; local closures only, no shared helper edits.
    // Body follows the assertions enumerated below for query and form_post.
}
```

Use existing Fixture/Upstream/stage_client/authorization/codes/query helpers and
in-process api::router.oneshot, with fresh independent cases. Create an explicit
existing Alice link with source auto_provision false. Establish the current local
TOTP and obtain one-use recovery codes using synchronous existing Core factor
calls with a thread-local with_test_time scope only around that fixture setup:
set T to the current30-second boundary minus60 seconds, begin/confirm with the
code generated at T, then fresh local login at T+30 and recovery_codes in that
synchronous scope. Both steps are before real now, without waiting at a clock edge.
Do not carry that clock scope over await or assume it propagates
through App.run's blocking workers. Send the fixed invalid text bad-code for the
charged OTP refusal; use a recorded recovery code for deterministic success through
the same local-factor branch. Existing local_totp_is_still_required_when_upstream_is_not_mfa
remains the separate live-TOTP definition, not a newly executed pass in this design.
No sleep, clock-edge loop, reset of attempts or production test-clock change.
Advance expiry by fixture-only persisted expiry0, not a timed wait. Assertions
compare generated bindings rather than depending on fixed random identifiers.
Use the existing assert_http_mutation_snapshot oracle where shared middleware can
persist http_rates; do not exempt receipts, audit, account/factor, source login,
stage, session or code state. Fresh case fixtures keep this transport test within
the existing admission budgets without changing their implementation/config.

| Cases within that single function | Required assertions |
| --- | --- |
| Legacy JSON versus HTML | On equivalent fresh stages, legacy GET and JSON POST still produce the existing JSON/redirect outcomes. Accept application/json/no Accept/wildcard never render the new page; explicit text/html hands off to the child page; text/html q=0 does not. HTML contains escaped verified identifiers, empty input/static assets, unchanged CSP and no private provider/poll/cookie/token/code values. Use a fixture-only identifier containing quotes/angle brackets to assert escaped data attributes and encoded path segments rather than testing only random safe ids; it is not a new accepted identifier syntax. Handoff has Vary:Accept/no-referrer; new private responses have no-store. Compare snapshots before/after both new GETs: no factor/stage use, attempts, code or audit. Readiness is obtained only with a subsequently guarded JSON POST. |
| Exact bound browser/pair | Start two browser authorizations via Core.browser_start, retain their actual return cookies and distinct generated request bindings. Missing/duplicate/foreign cookie, A browser with B stage, changed body authorization_id/request/redirect_uri, or invalid Origin/custom-header/Fetch Metadata is refused before a factor attempt. Assert snapshots and zero new codes. The HTTP loopback cookie Path covers every new endpoint; no cookie is copied or reissued. |
| Factor in query / shared malformed headers | Original GET otp, empty otp and encoded %6ftp still return400 before effect. Repeat on every new route, including new POST query strings. Unexpected JSON fields and malformed shared If-Match/Idempotency-Key are400; valid header handling is left to the existing shared middleware, with no new receipt behavior. No charge, success or redirect follows these refusals. |
| Pending and wrong OTP | Callback-pending stage's guarded POST{} returns pending, with no code/attempt. After a verified non-MFA callback for the exact linked subject, guarded POST{} returns local_factor_required and the served script reveals the factor prompt; read-only GET never determines this by writing. bad-code POST returns the unchanged Core401; attempts increments exactly1, used remains false, no code/source completion audit or recovery-code consumption. Input is not reflected in HTML/JSON/URL. Include a proof already satisfying MFA: its guarded POST{} completes once, rather than a new GET performing that write. |
| Exact successful factor, both modes | Valid one-use recovery input returns200 JSON whose only handoff is the original local resume path; fetch sees no Location/302/RP form, bearer or protocol code. Exactly1 code has Alice's id, exact source/link, original PKCE challenge, state/issuer/client and federated+otp MFA, with no hardware/passkey assurance. Recovery digest is removed once; source.stage_resume audit is exactly1. |
| Native query and form_post delivery | GET that local path with the original cookie. Query yields the unchanged validated callback302. form_post yields the unchanged renderer's escaped hidden code/state/iss fields, original non-protocol query retention, nonce script, no-store/no-referrer and form-action pinned to the registered RP origin. Both clear the original return cookie, remove Pending/user-code state and make the recorded browser session usable; no bearer token is returned by this browser transport. Assert exact corresponding stored code, not merely an HTML field count. Redeem that delivered code with the fixture's original verifier/registered redirect/client using existing Core.token(TokenRequest); verify signed sub is exactly Alice, then repeat the identical redemption and assert invalid_grant/no second family. This is fixture-only RP redemption, not a new browser token flow. |
| Used/replay and ambiguous-result recovery | Repeat resume/cancel POST after success: refusal with no second charge/code/audit/consumption. Before delivery, native page reload may recover the already-ready bound browser result without another Core resume. After delivery, replay GET/POST and foreign-cookie delivery fail and snapshots retain exactly the existing outcome. No API used-state rule is relaxed. |
| Cancel / expiry / deleted browser | Separate fresh stages: guarded cancel returns cancelled/code_issued false and local continuation; its native query/form_post error callback has access_denied, exact state/issuer and no code. Subsequent resume/cancel refuses. Setting stage expiry0 or deleting/expiring Pending refuses without code or factor consumption; stage used/cancelled flags are not resurrected. |
| Corrupted or conflicting proof/request | On fresh fixture-only copies, wrong nonce/login-stage association, suspension/request hash, stored request binding, or upstream subject bound to another account is refused; restore fixture copies only to distinguish a fresh case. No success/code is inferred from the error/rejected result. A rejected Core callback is delivered as an error through the original browser route, never converted to accepted authentication. |

The test's served HTML/JS assertions also pin redirect:error, no automatic retry,
same-origin/custom-header JSON submission and exact local-path-only continuation.
These are source/HTTP rendering assertions, not simulated desktop execution or a
claim of physical TOTP/hardware, full browser compatibility or the D01 app journey.

**ONE proposed command, not run or released by this design approval:**

```sh
env CARGO_TARGET_DIR="$PWD/target/i02-browser-transport-wave30" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support --test source_stage browser_source_stage_transport_binds_factor_and_renders_both_response_modes -- --exact --test-threads=1
```

Default Platform plus test-support only; no fuzzing, standalone-client/USB build,
second target, broad suite or provider/desktop command. Root must separately release
the implementation/check slot and align the owned branch to the reviewed source
before this command can run. Private target, jobs1/incremental0/dev+test debug0 and
the existing8GiB free-disk floor remain requirements. This appendix executed zero
Cargo, HTTP/provider/binary/service/browser/desktop commands and adds no extra gate.

### Additional immutable anchors and static verification

| Read source | Exact fixed Git blob / reviewed LF slice SHA256 |
| --- | --- |
| `src/assembly/browser_runtime.rs` | blob `f5ba027be118057b8ff491d168205f283caec4ac`; browser_resume_with254–310 `b7e6f50f524004c1eb8586a0278ae5e80637f8a990b372bdbdff514c2e2dedd1`; authorize_state385–395 `7eb2d0a1eabef1dbf159a95cd3cb85d35a6e4949631cbadfb00c6ba22c4a8624`; interaction1622–1637 `f02e99fb1da4674389da6e86c51467395c0aede38be6709e59329c42d06c3a7f`; stage_browser_callback1498–1514 `ae1a7b623f2d012ffd1a1907081515a1a9af55c3fa0870b29d1bfe637f45bcb7` |
| `src/assembly/source_stage.rs` | blob `d12ffffc0ea60de399decf3f089c66d9f42a6837`; load_source_stage13–26 `dc60f7b38d8db9368140808ef6e7d7e4c95fcd7628be390cf52115de84f77a55`; wrapper201–215 `a6817c8618e3a2be7db14de02f8b1c05854dee2100a4a997eb59595bfacffb90` |
| `src/assembly/source_runtime.rs` | prior pinned blob and resume hash above; cancel/rejection/denial809–891 `0a3a47053bdd02681e87196b19984ca75406e524363ec4a3f7ddd07a902d1d17` |
| `src/assembly/source_finish.rs` | blob `339e57f895b7203452f4ad6f384ac04e348fda19`; local factor/charged failure174–198 `a6fa65509b861f125e5088fbcd0576f6dc98e45f0ee363adb0272a10f65f7ffa` |
| `src/assembly/oidc.rs` | blob `a62722e78e45f786e3901bd155395c7afc30be7c`; other StageStart caller752–760 `62f0cea0ed47e267183e708ce886b3c3c75b1673e584b1716c6966269ce62923` |
| `src/assembly/browser_runtime.rs` read-only status | status_for1134–1183 `bec8c682480c7001880ded2a40890f64a93444b8bbd01a4baa81c503e2e9edad`; browser constructor73–243 `0c30a2db72cfdf739ccbfb4fcacbed70d0c480756e22cb4814291e4b353f2780` |
| `src/crypto.rs` | blob `f503b883ae7af5259ae159cd87d5da2dd629b1cc`; thread-local test-clock scope17–29 `fb02d5aa48fc23c51dea209c7615e6078e68794a452e65d03f864aba5887c3f8` |
| `src/store.rs` | blob `8e78d889220c9e956fe00a4f7a69b603063d3ebf`; redb write1047–1075 `eb712fb9794e5906088f88ca3756940485d5e7b754d9f2f6177890e0ffe6995d` |
| `src/portal/http.rs` / `src/portal/auth.js` / `src/response.rs` | blobs `0cd94a79b271c8026dc227b7c16229a1975937a3` / `e07260923ebb34995ef4d31a1b4605980626df7c` / `5ce29e8f70b10e049c93bc98afe02b2ae02e4f39`; CSP/guard, fetch policy and callback renderer inspected; all stay unchanged |
| `src/portal.rs` | blob `7d993ca92604c89a4a0f68c1c418767b210bbcf1`; proposed additive declaration only |

These are additional source/body/hash reads, not new execution evidence. Static
append/scope/path/hash/Markdown/whitespace checks and docs-checker results are
recorded at commit preparation below. All earlier mixed failures and missing
checkout provenance remain intact. Primary a2dff16a remains assigned; no board
change, reservation, other-worker contact or duplication of active D01 runtime.
W02 DONE and the bearer SourceTOTP API retain their exact semantics; held Group,
receipt-secret, route-header, PAM and remote-I/O/60s boundaries remain dependencies.
Any separately authorized desktop work must use RiWork Cua.ai Driver only, after
descriptions/current state; this design used no desktop tool or provider.

### Checks actually performed for this design appendix

* Read the fixed Git source and inspected the handler, cookie/CSP/guard, constructor,
  callback, Core binding/charge/rollback and fixture bodies described above. Git
  caller searches found the two StageStart constructors and the sole Pending
  browser_hash assignment; these are static reads, not executed authentication.
* The scoped Python assertions passed: the original20546 report bytes from
  `3d5855623ff5cc1310a889d41768771bb8533aad` remain an exact prefix; this report is
  the sole modified file; LF/trailing whitespace/fence balance are valid;
  all34 distinct referenced Git objects and25 existing paths at the fixed source
  resolve. Three new portal paths are explicitly prospective, not falsely
  claimed to exist. All13 additional LF-slice SHA256 values above recomputed
  exactly from the fixed source. `git diff --check` exited0.
* `python3 scripts/check-docs.py` exited1 solely for the five existing output
  directories `target-wave29-source`, `target-wave28-scim`, `target-wave28-portal`,
  `target-wave28`, `target-wave27`; it reported zero missing Markdown links.
  No directory or checker was changed. Staged scope/whitespace and append-only
  checks are repeated before the report-only commit.

No proposed hunk/test function was compiled or executed; the one future Cargo
command remains HELD. This is a transport design for root's review, not evidence
that the original browser gap has been implemented or that I02 can close.

## Approved source-only implementation: wave30_I02_browser_transport_implementation

The subsequent root instruction released exactly the six source/test paths from
the design, plus this append-only evidence. It did not release compilation,
execution or integration. The original live I02 row was reread: `in_progress`,
primary `a2dff16a-c4b0-47fc-96de-ac85b1fb6d9e`, with the original five-part outcome
and working setup/lifecycle/failure-handling gate quoted above. This supporting
worktree does not change that assignment or status. A03 and W02 remain DONE;
D01's isolated active runtime was neither contacted nor duplicated.

### History alignment and exact commits

* History-preserving merge `a59809f47e0a1928271a9d6ecec4e207e17ef8c7` has parents
  `0569450afc67d1de01223bf42f39742193347722` and exact published
  `9b8956f7b2a9961b14e313fa57c0f5214a136772`. No reset or replacement from a stale
  tree. Three conflicts were resolved narrowly: each guide's old blanket
  format-3 sentence was replaced only by the published formats1/2 qualifier;
  the report's empty published conflict side was removed while retaining the
  whole own design appendix. Both guides byte-match the published pin. Before
  implementation, the entire merged tree differs from that pin only in this
  report, which byte-matches `0569450`; the published report is its exact prefix.
  All published root notes and accepted equivalents were retained.
* Source/test commit `36ccc64e4c4044491f55b68cf7fdcaeff992391c`, parent that merge:
  exactly `src/api.rs`, `src/portal.rs`, `src/portal/source_stage.rs`,
  `src/portal/source-stage.html`, `src/portal/source-stage.js`,
  `tests/source_stage.rs`. Six files,1524 insertions/5 deletions. The one appended
  test function is
  `browser_source_stage_transport_binds_factor_and_renders_both_response_modes`.
  This report appendix is committed separately after static checks.

### Implemented transport, with execution still held

The implementation follows the reviewed cookie-bound design. The old GET retains
its exact StageReference extraction and Core call on the JSON/non-browser branch;
only positive HTML with browser_ui and a stored matching browser binding receives
the read-only local handoff. GET responses vary on Accept. Legacy JSON POST,
cancel, factor-in-query refusals, records and callback bodies remain byte-intact.

The new page GET authenticates the original return cookie through authorize_state,
loads the exact stored pair, checks binding/expiry and renders an empty prompt.
It invokes no resume/cancel/write. An already-ready callback hands off locally;
it does not infer a successful grant from the state name. Resume/cancel POSTs use
the unchanged browser_write_guard and explicit positive JSON, reject query and
extra JSON fields, and preflight the same browser/pair/undecided state. Each calls
its existing Core wrapper exactly once. Bad-code errors are translated only after
that wrapper returns, preserving its inner committed charge; there is no new
store.write to change outer rollback. Closed status/code_issued/redirect-presence
classification strips the RP URL and returns only a waiting status or the exact
original local continuation, retaining rejected/cancelled outcomes.

Private route responses add no-store/no-referrer; the global guard/CSP and callback
renderer stay unchanged. Shared-middleware refusals retain their existing policy.
Identifiers are encoded for path segments (spaces use%20) and then escaped in HTML
attributes. RiAuth.post receives issuer-base-relative paths, while native continue
uses the original base-prefixed path: no duplicate non-root issuer prefix. The
page has an unnamed empty factor input, guarded type-button actions and external
scripts. One page-level busy flag covers initial check, factor, recheck and cancel.
Network/5xx ambiguity clears the input and requires reload; factor/cancel are never
automatically retried. No RP fetch, returned HTML insertion, new credential,
account/request factory, standalone finish, private Pending projection or visibility
widening. The original browser-resume GET still owns query302/form_post rendering,
one-use delivery and browser SSO establishment. No Core/source/factor/workflow/
storage/authority/admission implementation was edited; only the approved API route
classification joins the existing source_callback bucket.

The single table-driven HTTP/rendering definition covers root query and non-root
form_post, disabled browser_ui, explicit media negotiation and malformed quality,
empty/escaped HTML and base-relative script paths, foreign/missing/duplicate
cookies and mixed pairs, the unchanged Origin/header/Fetch Metadata refusals,
query-factor and malformed shared-header/extra-body refusals, pending/local factor,
one committed bad-code attempt, fixture-only broken bindings/expiry/deleted browser,
one recovery-code consumption and exact Alice/source/link/fingerprint/PKCE/code,
native delivery and signed subject/one-use redemption, replay/reload, cancel and
an upstream Bob proof rejected by an actual Alice-bound browser request. Separate
legacy GET/POST/cancel cases retain their original query/form_post outcomes.
Setup uses existing admitted source-link writers and synchronous test-clock factor
setup; no standalone finish or shared helper/test edit. Fresh routers give the
independent transport cases fresh rate tables without changing a budget. Assertions
use the existing snapshot oracle, exempting only its operational http_rates keys;
they do not dump private snapshots, cookies, factors, codes or tokens.

These are definitions inspected statically, not passing runtime assertions. The
broken-proof cases exercise early Core/transport refusals; no post-factor fault
injector or concurrency test was added. Late outer rollback remains supported by
the unchanged Core/Tx bodies described in the design, not a new executed claim.

### Exact source hashes and actual static checks

Full-file SHA256 at source commit `36ccc64`:

| File | SHA256 |
| --- | --- |
| `src/api.rs` | `1bbb387dd627f1fd56b7dfe408ea17eef48131f927597da1b81b21ed5f64b8ca` |
| `src/portal.rs` | `ec3dd0321fe996caf8c8b4d8747c6c8f5c1b332bcd13f02c9bef7681f0897344` |
| `src/portal/source_stage.rs` | `a4fed0542f3393f3d09aa518d2c0cf74f7f90fcb992288cdb9dd9640a52239c2` |
| `src/portal/source-stage.html` | `1c8fe8280cd0fca1cc90084e772c366e7f75d82b0115faa21115f0b5b76255c0` |
| `src/portal/source-stage.js` | `e540a4910ee39dfaf9810a28259d32b41ee29787ab1e008692bb3d85d8e2ba1b` |
| `tests/source_stage.rs` | `d3d44e96f461bb88626d73caf0a30ff1610187ca6f6338a9e8e1e5a1b28fb9a6` |

Checks actually performed, without Cargo/type checking:

* Local guidance/clean state, fixed Git object and live exact-task read; merge and
  the conflict-resolution/equivalence assertions above.
* Python full API reconstruction: replace only the new GET span with its original
  span, remove the route merge and added rate condition, then match the entire
  published API byte-for-byte (original SHA256
  `ffcb8590457eda37a4fe0434c40d328eecdb1718953b8b3607405007a13451b3`).
  Thus all other handlers/records/helpers, including native browser/callback and
  old stage POST/cancel/result, are protected. Removing just the additive portal
  declaration similarly reconstructs the entire published portal module.
* Original `tests/source_stage.rs`124619 bytes remain an exact prefix, SHA256
  `a65e2dd309b1260c3249f45dec14205b95eec142c6db72bb2dc743c2a9626e9c`;
  the append contains exactly one tokio test/async function, with local closures.
  The prior52789 report bytes remain an exact prefix of this appendix.
* Sixteen held files byte-match the published pin: portal/http.rs, portal/auth.js,
  response.rs, assembly/browser_runtime.rs, assembly/source_stage.rs,
  assembly/source_runtime.rs, assembly/source_finish.rs, source.rs, core.rs,
  store.rs, workflow.rs, workflow/executor.rs, workflow/approval.rs,
  api/interaction.rs, config.rs and assembly/oidc.rs (all under src/).
* Static negative guards: new GET body has no resume/cancel/store.write/native
  resume; module has exactly one Core resume and one cancel call, no standalone
  finish, new store.write, bearer response or global CSP/CORS change.
* Installed1.98.1 formatter `rustfmt 1.9.0-stable (48a229ceae 2026-09-01)` parsed
  and formatted only the appended test via stdin, preserving all old bytes.
  Direct `rustfmt --edition 2024 --config skip_children=true --check` on api.rs,
  portal.rs and the new module exited0; children were not reformatted.
  `node --check src/portal/source-stage.js` exited0 (syntax only, no script run).
  Python HTMLParser/DOM-id/static-JS assertions passed: empty unnamed input,
  type-button guards, external scripts, relative POST/exact local continuation,
  no credential logging/storage, RP fetch/HTML insertion or automatic retry.
* `git diff --check` and staged whitespace/exact six-file source scope passed.
  `python3 scripts/check-docs.py` still reports only the five pre-existing
  target-wave directories listed above, zero missing Markdown links; no checker
  or directory change. Final append/scope/Markdown/whitespace checks are repeated
  for the separate report commit.

### One concrete held compile/runtime command and resource plan

The same focused command from the design would compile the matching Platform
library and source_stage test target, then run only the named function:

```sh
env CARGO_TARGET_DIR="$PWD/target/i02-browser-transport-wave30" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support --test source_stage browser_source_stage_transport_binds_factor_and_renders_both_response_modes -- --exact --test-threads=1
```

**Not run; requires a separate root release of the sole Cargo slot and this
fixture runtime.** Default Platform plus test-support, installed pinned1.98.1;
no fuzzing, USB, standalone-client target or other test invocation. Source commit
and manifests must be re-pinned before that release. Current immutable manifest
SHA256: Cargo.toml `58e5ef824ed96290179c9f76fea208dc37173caeee21b6ce8d37f8a6dd1abcb8`,
Cargo.lock `b5c9d11c001244b8017303ce8c20516e02946483845eee40720b0910758d4426`,
rust-toolchain.toml `887f9be066a15585a2c583578e84b0fcb541126d81546276bad3d2ff00d61167`.

Read-only `df -k .` during static review reported14219380KiB available
(about13.56GiB). That is a historical capacity measurement, not a reserved budget.
Immediately remeasure before any released build; require at least12GiB, monitor
every2 seconds, stop only this Cargo process at9GiB to preserve the8GiB floor.
Private target in this worktree only; no accepted/cache deletion. Use private0600
logs and preserve redacted exit/elapsed/disk/source/feature evidence. Only this
selected function's disposable in-process HTTP/signed-peer fixtures may run;
bound its post-build runtime to20 minutes, stop its own process on failure/timeout
and release the slot immediately. No desktop, provider tenant or production service.

### Residual original acceptance and ownership limits

The proposed local transport is now implemented in an uncompiled source slice.
Concrete remaining verification is the held type/HTTP-rendering check and root's
independent review. No fresh browser/provider execution, released artifact or
original-I02 completion is claimed. The previously identified existing
oidc-profiles limitation paragraph is untouched and remains reserved for a separate
docs decision after reviewed implementation, not silently rewritten in this lane.
All historical passes/failures/log-provenance limits above remain historical.
Any wiring failure that needs a held writer/private browser contract change must
be reported to root before broadening. Receipt-secret, optional/required headers,
PAM, Group/source materialization, live authority/one-use/audit/credential and
remote-I/O/60s boundaries remain intact. Root alone integrates/pushes/statuses;
primary a2dff16a is unchanged. Driver-only desktop preference remains, and no
desktop/provider inspection or delegated work was performed.

Final report preparation passed: exact prior52789-byte prefix, sole report file,
LF/fence/whitespace checks,38 referenced Git objects and28 source/report paths,
and all six source-commit SHA256 values recomputed exactly. Repeated rustfmt
checks and Node syntax check exited0. The docs checker's actual captured exit
was1, with exactly the same five directory-layout errors and zero missing links.
These checks execute no product, test, provider or browser flow.

## Approved non-root upstream fixture correction — source only

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, original I02
`cbe1e83d-b58c-4be7-ba6c-5fb8a73c9839`, supporting WT
`42bb51c6-c198-4adb-bd92-0a5222853231`; primary a2dff16a is unchanged.
Root reserved `wave30_I02_nonroot_upstream_fixture_correction` after identifying
an **unrun fixture blocker**, not an observed test failure. Reviewed source is
`36ccc64e4c4044491f55b68cf7fdcaeff992391c`; preceding static appendix is
`130e1111d24a46b8d8fee97979b4f788b84d2410`. History is retained without another
merge/reset. This correction does not authorize Cargo, type checking or runtime.

### Concrete mismatch and exact bounded change

The appended browser test selects `/identity` for its form_post issuer. Existing
`Upstream::new` nevertheless asserted the root callback at its token endpoint.
Unchanged `src/assembly/source_runtime.rs:433–438` derives the callback from
`self.config.issuer.trim_end_matches('/')`; authorization at416 and token
exchange at511–515 both use that same `source_callback_url` value. Source-runtime
Git blob is `51a46db7a7ad3e46914146461c79c6ae42ab124a`. Thus the non-root fixture
must expect `http://localhost:9000/identity/oauth/sources/upstream/callback`,
while the root-issuer fixture still expects its original root callback.

Correction commit `196ac2096f79e5cc3c7d53ffa990ccdeba11a439` changes only
`tests/source_stage.rs`, three insertions/four deletions in `Upstream::new`:

* Capture `f.core.source_callback_url("upstream")` before the token-route closure.
* Clone that captured String beside the existing per-request records clone.
* Compare `form["redirect_uri"]` to it with exact `assert_eq!`, replacing only the
  hardcoded root value. No prefix matching or assertion removal.

The fixture reads its configured issuer through the existing Core getter; no
new global state, rate/clock/guard change or production edit. Authentication,
grant-type, code-record removal, PKCE-verifier and token-response assertions and
all other helper code remain byte-equivalent. The complete appended browser
test remains unchanged; there is no new test body or runtime result.

### Fresh immutable scope and reconstruction evidence

Python byte assertions passed before the correction commit: each of the three
reverse substitutions occurs exactly once, and their reversal reconstructs the
**entire** reviewed166018-byte test file, SHA256
`d3d44e96f461bb88626d73caf0a30ff1610187ca6f6338a9e8e1e5a1b28fb9a6`.
Corrected test is166005 bytes, SHA256
`0c5efa341f72922b3ef16cd92a6157200ab439b1860ad339c42d7b5ee83250b8`,
Git blob `34ba37484ba3f9e4be13e27764eb77da45ffd1a3`. Its41399-byte appended test
suffix exactly matches36ccc64 after accounting for the13-byte helper reduction.

The earlier124619-byte original-prefix protection describes the **preceding
source phase**. This expressly authorized correction changes that prefix's
Upstream helper; it is no longer literally unchanged in the corrected file.
Reversal restores both that full original prefix and the entire appended test.
All five production implementation files, the sixteen held files listed above
and Cargo.toml/Cargo.lock/rust-toolchain.toml still byte-match36ccc64. The sole
pre-commit working-tree diff and the correction commit contain only the fixture
file. No global checker, other helper, existing guide or source-profile doc edit.

Direct installed1.98.1 `rustfmt --check --edition 2024 --config skip_children=true
tests/source_stage.rs` exited0: parser/format check only. `git diff --check` and
staged whitespace check exited0. The Python proof also verified single-file
scope, exact appended-body bytes and all protected files/manifests. No compiler,
test, JavaScript, browser, binary, provider or service was executed this phase.

### Residual verification and original-row disposition

No form_post pass is inferred from correcting the source mismatch. The named
HTTP/rendering test and any type check remain unrun; independent full security
review continues under root, with no reviewer/other-worker contact here. The
previous exact held command/resource plan remains a proposal requiring root's
separate release after immutable source review, even though the Cargo slot is
reported free. All prior failures, historical execution pins and gate limitations
remain intact. There is no original-I02 closure recommendation from this static
correction, no release certification and no task-status change. Root alone
integrates/publishes/statuses; Driver-only desktop preference is retained.

Report preparation verified the correction commit's exact parent/single-file
scope and immutable corrected-test hash, five referenced Git objects, and the
entire65356-byte130e111 report prefix unchanged, with LF/fence/whitespace checks
passing. `python3 scripts/check-docs.py` actually exited1, reporting only the
same five pre-existing target-wave directories: target-wave29-source,
target-wave28-scim, target-wave28-portal, target-wave28 and target-wave27.
It reported zero missing Markdown links. No directory or checker was changed.

## Focused resource preparation — read-only, execution still held

Root authorized `wave30_I02_focused_resource_preparation` for this same project,
I02 and supporting WT, allowing only this report append. Preparation starts from
clean HEAD `b906c7b76fe20291e148abfb6ceca4ff6bb7f6ef`; product/test source remains
`196ac2096f79e5cc3c7d53ffa990ccdeba11a439`. Fresh read-only hashing reproduces all
five36ccc64 production SHA256 values above and the corrected test SHA256
`0c5efa341f72922b3ef16cd92a6157200ab439b1860ad339c42d7b5ee83250b8`.
No merge, configuration change, artifact invocation, Cargo or type check occurred.
Original primary a2dff16a and root's ownership of review/status remain unchanged.

### Matching cache and profile evidence

Inspected only this worktree's existing `target-wave27/cargo` cache. Both that
directory and its target-wave27 parent are nonsymlinks; its resolved path stays
inside this worktree, and the debug tree has no symlinks. `du -k -d 2` measured
2377044KiB allocated (2.267GiB): deps1978980KiB, build70520KiB,
fingerprints12272KiB, incremental0KiB. This is an existing private cache, not the
accepted target and not the previously proposed new cold directory.

Metadata `.rustc_info.json` records1.98.1, commit
`48a229ceaefd4985c50990b14116b6d856af0985`, host aarch64-apple-darwin and the
installed1.98.1 toolchain path. Rustc/Cargo files exist there by stat; neither was
executed, so this records cache provenance rather than a fresh version result.
The current rust-toolchain.toml still selects1.98.1. Fresh manifest hashes match
the source-phase values: Cargo.toml
`58e5ef824ed96290179c9f76fea208dc37173caeee21b6ce8d37f8a6dd1abcb8`, Cargo.lock
`b5c9d11c001244b8017303ce8c20516e02946483845eee40720b0910758d4426`, toolchain
`887f9be066a15585a2c583578e84b0fcb541126d81546276bad3d2ff00d61167`.

Relevant fingerprint records:

| Existing record | Features | Profile hash / artifact metadata |
| --- | --- | --- |
| riauth-0d65ce4b80b71950 lib | default, essentials, platform, test-support | 12672335563272108896; rlib400029936 bytes, rmeta44874859 |
| riauth-25aded663d5712f5 server and riauth-9fc8fea30bfac30b maintenance | Same four | Same profile; executables255814720 and67010608 bytes |
| riauth-2cea114daae82d64 cloud_directory integration test | Same four | 11094973624911973823; executable198904944 bytes |
| riauth-aeaedbe66486b343 / riauth-87bb46aa584f94f0 check libs | default, essentials, platform / essentials | 10509049656720700007; rmeta-only, not linkable product substitutes |
| source_stage integration test | Absent | No existing fingerprint or executable; it must compile/link |

The lib/server/maintenance records share rustc hash17329007180185699724,
config9396254390672932401, empty rustflags and native compile_kind0. Historical
wave27 report `7e87428567475d2a6e8589d58c4419c16c768a55` records actual cloud
tests using this exact target, locked/offline/default+test-support, jobs1,
incremental0, DEV_DEBUG=0 and TEST_DEBUG=0. The subsequent source-check report
`af4fd9e253b82c3094b21114f623952d968cf377` records the same profile settings.
These historical commands support the profile interpretation; the hash alone
does not decode debug settings or demonstrate current-source correctness.

Python read-only metadata traversal matched all59 cloud-test dependency hashes
to unique cached fingerprint records, including all53 direct lib dependencies,
riauth and the five additional dev crates. All59 have linkable rlib/dylib files.
The reachable graph contains439 records with no absent/ambiguous dependency
hashes, and697 linkable/metadata files totaling1253844890 bytes. Its360 registry
package/version paths match the current lockfile and have source directories
and package manifests. An initial version-splitting heuristic misclassified
`toml-1.1.6+spec-1.1.0`; exact whole name/version lookup resolved that inspection
false positive, with zero actual lock mismatches. No dependency code was run.

Compiled records match the rustc/config/flags/native-kind settings. The38
run-build-script records instead have Cargo's zero profile/config metadata,
with the same rustc/empty flags/native kind; these are not an alternate compile
profile. Existing recorded build-output paths checked were present. Comparison
of2518 allowlisted tracked compiler/SDK/OpenSSL/native-build setting records
against the current environment found no mismatch;64 other variable names were
not inspected. No applicable ancestor/home `.cargo/config[.toml]` exists.
CARGO_TARGET_DIR, jobs/incremental/debug overrides, RUSTFLAGS/encoded flags,
wrappers and RUSTUP_TOOLCHAIN are currently unset. The proposal reproduces the
historical overrides explicitly, without introducing flags or changing config.

Metadata SHA256: lib-riauth.json
`ac6f00d7f80260b8f443564cbbdd8e64f91d91aa42f8d001c0a23e13778f1c10`,
cloud_directory test JSON
`f74546fdaa6a5ab161af5eaf5ea1e798a0108eccbc51db0ff5b68ae5f963ed28`,
and .rustc_info.json
`27df402be20083ab5b4835c05762e2b77beed67288686dd8fda9193b24cdd7c4`.
No binary content hash/version/execution was substituted for source evidence.

### One proposed command, not executed

This supersedes only the earlier cold-target location proposal:

```sh
env CARGO_TARGET_DIR="$PWD/target-wave27/cargo" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --offline --features test-support --test source_stage browser_source_stage_transport_binds_factor_and_renders_both_response_modes -- --exact --test-threads=1
```

The current1.98.1 toolchain pin applies. Default selects Platform (which includes
Essentials), plus test-support, exactly matching the linkable dependency graph.
No explicit cross-target argument: the cache is native aarch64-apple-darwin;
adding a target triple would create a separate artifact layout. No USB, fuzzing,
standalone-client build or additional filter. Offline mode matches the recorded
cache-creation commands and fails instead of downloading a missing dependency.
Cargo still decides freshness and must rebuild all changed product sources and
the new source_stage test. The old lib dep-info omits all three new portal stage
assets, and old products date to2026-10-01; none is evidence of this I02 flow
passing. No old executable will be invoked directly.

### Actual capacity and prospective owned-group budget

`df -k .` initially reported13367980KiB free. Final measurement at
2026-10-02 16:11:08 UTC reported13360168KiB, about12.74GiB, leaving3.74GiB above
the unchanged9GiB own-stop threshold. This is a snapshot during the held queue,
not a reservation or permission to start before root releases the slot.

Estimate an additional peak of **2–3GiB**, reserving3GiB for this one Cargo group:
the old rlib+rmeta together are about0.414GiB; allow their full regenerated set
and compile objects, server/maintenance rebuilds and temporary copies (existing
two copies total about0.601GiB), a new source_stage executable (budget0.35GiB,
versus the existing cloud test's0.185GiB), linker scratch and limited dependency
revalidation/rebuild headroom. These are size-based estimates, not measured
peak guarantees. Existing reachable non-riauth link/metadata artifacts account
for about0.753GiB plus native build output; metadata supports reusing those
rather than cold-duplicating them. No claimed saving equals the whole2.267GiB
cache, which also contains old products and check-profile artifacts.

Retain the prior **at least12GiB immediately before start**, after current SCIM
exits and root grants this sole group the slot. At the observed12.74GiB, a3GiB
additional peak leaves about9.74GiB; at the12GiB minimum it reaches the9GiB
margin, so the monitor may stop a larger-than-estimated build. Recheck source,
toolchain/settings and disk then; do not alter profiles to chase reuse. Sample
free disk every2 seconds throughout Cargo/link/test, stop only the owned process
group at9GiB, preserve the8GiB floor and never kill another lane. No other cache
or evidence deletion/move. Any unexpected substantial dependency rebuild or
shared-disk drain invalidates the estimate; stop/report rather than force flags
or lower the threshold. Root owns aggregate budget and release scheduling.

Retain private0600 logs and redacted argv/source/features/exit/elapsed/disk
evidence. Bound only the selected disposable fixture runtime to20 minutes,
perform PID/group-scoped finally cleanup and immediate exit/slot-release handoff.
This preparation creates no log/target/helper, allocates no execution slot and
does not launch any artifact, service, provider, CLI product or browser.
Independent review and focused compile/test remain held; no form_post pass,
original-I02 closure, real tenant or release evidence is inferred. All historical
failures and receipt/header/PAM/Group/authority/one-use/60s limitations remain.

Preparation checks actually performed: Git state/object reads, manifest/settings
and fingerprint/dep-info metadata reads, stat/du/symlink inspection, df snapshots
and Python metadata/hash/scope assertions. Final static checks preserved the
entire70445-byte b906c7b report prefix, confirmed this is the sole changed file,
and matched all six source files/three manifests to196ac209 plus both cited
historical report objects. LF/fence and `git diff --check` passed. The docs
checker exited1 only for the same five pre-existing target-wave directory-layout
violations, with zero missing Markdown links. No formatter/compiler/test or
artifact was executed, and no cache/config/evidence was modified or removed.

## Released focused compile/test — compilation passed, selected test failed

After SCIM exited and released its slot, root separately authorized exactly the
one locked/offline/default+test-support source_stage filter printed above, on
clean committed `cd17c3bcffeba750d7827a54fdcf8d6a7470edba`. Production and test
bytes remained `196ac2096f79e5cc3c7d53ffa990ccdeba11a439`; the five production
hashes, corrected test and all three manifests were rechecked against that pin.
This is the first executed check of this transport slice. Earlier uncompiled
statements remain historical; this run is **not a passing transport result**.

### Preflight and exact execution

Preflight passed: correct worktree/clean HEAD, own nonsymlink warm cache and
matching features/profile/settings, no new applicable Cargo config/flag/wrapper
override, no competing Cargo/rustc process, and at least12GiB free. `rustc -vV`
actually reported1.98.1, commit48a229ceaefd4985c50990b14116b6d856af0985,
host aarch64-apple-darwin and LLVM22.1.8. Cargo resolved through the installed
`/Users/dominik/.cargo/bin/cargo` shim under the unchanged1.98.1 toolchain pin.
No extra Cargo command, target, profile or feature was introduced.

The exact released command ran once with jobs1/incremental0/dev+testdebug0 and
`CARGO_TARGET_DIR="$PWD/target-wave27/cargo"`. Supervisor launched it in a new
owned session/process group99685, with an1800s outer cap and1200s selected-fixture
cap starting at Cargo's Running marker. It sampled disk/process metadata on a
nominal2-second cadence, retained the9GiB own-stop/8GiB floor and3GiB transient
budget, and capped the exclusive private Cargo log at32MiB. No retry followed.

| Actual result | Evidence |
| --- | --- |
| Compilation completed | Cargo: `Finished test profile [unoptimized] target(s) in 59.03s`; only riauth compiled, no third-party rebuild |
| Selected named test ran | source_stage-51034130e831fdee; 0 passed, 1 failed, 0 ignored, 0 measured, 23 filtered out; harness3.25s |
| Cargo exit | 101; total supervised64.104s; Running-to-finally interval4.891s, distinct from harness timing |
| Failure location | tests/common/mod.rs:71:9, mutation snapshot assertion |
| Stop/exception flags | None: no timeout, disk/budget/shared-drain stop or supervisor exception |

The failure message identifies **one changed source_logins record**. The shared
oracle at tests/common/mod.rs:48–74 ignores only operational http_rates indexes
and never dumps record values. Safe review extracted only the collection name
and count; record IDs and values are withheld. The log has no caller backtrace
or row/step marker, so it does not establish which snapshot call, HTTP step or
response-mode row reached that assertion. Source review of the named test's
snapshot calls cannot substitute for that missing executed trace. No inference
of query/form_post completion, factor charge/consumption, one-use handoff,
cancel/replay/expiry or non-root callback execution is made from this failed run.
The earlier exact callback correction remains source-proved and now compiles;
this failure neither proves its runtime path passed nor identifies a production
security defect. Do not weaken the mutation oracle or silently correct source.

### Immutable private evidence and cleanup

All evidence is under this worktree's private0700 directory
`target-wave27/evidence/i02-focused-20261002T162348Z-2e255445/`. Each file was
created exclusively with mode0600; no raw credential/body values were printed
in progress or handoff output. Files are retained, not deleted or moved:

| File | Bytes | SHA256 |
| --- | --- | --- |
| cargo.log | 1322, not truncated | 9ed4e8a9a01a40f929ba51b343a0ee96cd26fe83972c475271319c0c4047e567 |
| disk-process-samples.jsonl | 7156 | 10f40aeadc811c5becd2fa49be96a481dae3cb98a1cb8478000dc1af08100db5 |
| result.json | 2147 | 78663035596a32f73d22eabc920378eda00ad986bfb8ca4d2e1315e753a3d82c |

Start/end UTC: 2026-10-02 16:23:48.247254 /16:24:52.351686. Free bytes were
13704462336 initially (12.763275GiB),13210279936 minimum (12.303032GiB), and
13470035968 finally (12.544949GiB).31 disk/process samples were recorded; maximum
observed sample interval was2.112s including metadata-inspection overhead.
Target allocation started2434093056 bytes and peaked2670579712, growth236486656
bytes (0.220245GiB), below the prospective3GiB budget. Neither stop margin nor
floor was approached.

The generated selected-test executable is207422528 bytes, SHA256
`74db7ec761b6860a52784215a9a4d949bf6e49bfd2464ecb350eccb547d7bba1`.
Its test fingerprint JSON SHA256 is
`e4d87ed9b2b6d2c151e8f9ba46bf23ed3206327cc140480f5172c7ed4beff63f`;
features are default/essentials/platform/test-support, profile11094973624911973823,
rustc17329007180185699724, config9396254390672932401, empty flags/native kind0.
The rebuilt lib retains profile12672335563272108896 and matching settings. These
are generated build/source provenance, not evidence that the failed test passed.

Finally cleanup waited/reaped Cargo. Group99685 had no members before cleanup
and none afterward; a separate post-run process-table check also found zero
members. No signal to another process/group was sent. The test's in-process
local HTTP fixtures ended with its process; no separate service/browser/provider
was launched. HEAD remained cd17c3bc and the tracked tree stayed clean after exit.
Explicit-project RiWork handoff exited0 and **released the sole Cargo slot before
this appendix was written**, reporting exit/count/location/log hash/disk/cleanup.
No automatic retry, source/helper/config correction or additional test occurred.

### Remaining verification and original I02 gate

This concrete local assertion failure is the next verification blocker. Root
must reserve any further diagnosis or correction separately; this lane makes no
new implementation claim or retry request by action. The entire chosen function
must eventually pass before its later assertions can be credited. The original
source setup/verified linking/assurance/browser completion/rotation outcome is
not certified by a successful compile and a failing execution. Historical passes,
mixed failures and source-only evidence above remain precisely scoped. Original
primary a2dff16a/status and root-only review/integration/publication ownership
remain unchanged; receipt/header/PAM/Group/live-authority/one-use/60s boundaries
and Driver-only preference are preserved.

Report-only static validation preserved the entire80009-byte cd17c3bc prefix,
matched all six source files/shared snapshot helper/three manifests to196ac209,
and independently checked the failed result, evidence hashes/modes and finally
cleanup. LF/fence/scope and whitespace checks passed. The docs checker again
exited1 solely for the same five pre-existing target-wave directory-layout
violations, zero missing Markdown links. These report checks performed no
second compile, test, service or artifact execution.
