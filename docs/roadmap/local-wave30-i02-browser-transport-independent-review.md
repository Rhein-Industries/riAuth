# I02 browser transport: independent immutable source review

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; 2026-10-02. Independent
read-only support in existing worktree `ed9ac424-59f4-4520-905b-919aea3521eb`,
branch `roadmap/local-revisions-coordination-wave27`, report parent
`4d9e39767c11bfbb4190348a16eb15a15212d6b5`. Sole write is this new report.
Original I02 task `cbe1e83d-b58c-4be7-ba6c-5fb8a73c9839` and primary
`a2dff16a-c4b0-47fc-96de-ac85b1fb6d9e` are unchanged. D03 DONE and D05
in_progress remain as instructed; this review changes no assignment or status.

Review source **`36ccc64e4c4044491f55b68cf7fdcaeff992391c`**, parent
`a59809f47e0a1928271a9d6ecec4e207e17ef8c7`, against fixed published baseline
**`9b8956f7b2a9961b14e313fa57c0f5214a136772`** and accepted design
**`0569450afc67d1de01223bf42f39742193347722`**, `docs/roadmap/local-wave30-i02-federation-plan.md`.
All product/test references below are immutable Git-object reads, not moving
source-worker diffs. No alignment, source import or other-worker contact occurred.

## Recommendation and concrete finding

**Hold the new focused fixture's readiness for F1.** The production transport
hunks agree with the accepted cookie-bound design; I found no concrete new
production security, compilation or API-semantic defect in those inspected bodies.
That is a source-review conclusion, not a compiler or runtime result.
The complete candidate and its one appended test are **UNCOMPILED / UNRUN** by
this reviewer. Root owns correction ownership, runtime release and integration;
this report neither accepts the source nor closes I02.

**F1 — the non-root form_post fixture cannot satisfy its reused upstream mock.**
At source `tests/source_stage.rs:3907`, the new table pairs form_post with
`/identity`; the local fixture at3815–3819 initializes issuer
`http://localhost:9000/identity`. Its first upstream callback is at4332.
The protected, unchanged `Upstream::new` helper at1775–1778 instead asserts
that the submitted token request's redirect_uri is exactly
`http://localhost:9000/oauth/sources/upstream/callback`.

The actual unchanged production `src/assembly/source_runtime.rs:433–437`
builds source_callback_url from the configured issuer, and511–515 sends that
exact callback in the upstream token exchange. For the new second table entry it
therefore sends `http://localhost:9000/identity/oauth/sources/upstream/callback`.
These strings are unequal. The mock handler's assertion will panic before it
returns tokens. The existing callback path records a failed exchange; even if
its outer callback helper returns a recorded result, the subsequent stage resume
cannot produce the successful local-factor status required at4346.
Thus the form_post/non-root assertions will not establish the proposed evidence.
This is a deterministic source-derived expected failure, **not an observed test
failure, count, exit code or log**. The root-path iteration is not implicated by
this specific mismatch.

Smallest prospective correction: keep the mock's exact redirect_uri equality,
but bind it to this fixture's configured callback rather than the root literal.
In `Upstream::new`, after the existing records clone at1759, capture
`let callback = f.core.source_callback_url("upstream");`; clone it beside
`let records = records.clone();` at1765 before the async move; compare
`form["redirect_uri"]` exactly with that callback at1775–1778. The per-handler
clone retains the reusable closure's ownership semantics. Do not accept any
callback, change the issuer to the root, or remove the non-root test.

That minimal hunk changes the protected original test prefix, so **it requires
root's separate reservation**. No prefix/helper/test edit was made here. A
suffix-only duplicated configurable mock is a larger alternative if root keeps
the prefix absolute; this review does not reserve or implement either choice.
The early finding was sent to root with the explicit project UUID; send exited0.
No source worker was contacted.

## Production transport and security audit

Line numbers for the new module mean the candidate; unchanged assembly/guard/
renderer references mean fixed9b and have identical blobs at the candidate.

| Contract | Complete dataflow inspected and conclusion |
| --- | --- |
| Return-cookie path | Browser construction in browser_runtime73–245 assigns the request binding and stage/browser authorization together and sets the existing return cookie on resume_path. binding_cookie1549–1581 uses the resume Path on HTTP loopback, and a per-interaction __Host name with Path=/ and Secure on HTTPS. The three new routes in portal/source_stage35–51 descend from that original resume path. No cookie is cloned, widened or minted by this adapter. |
| Exact browser and pair | preflight185–230 calls authorize_state with binding_cookie("return", authorization) and no SSO substitute. api1523–1546 refuses duplicate cookie values; browser interaction1622–1637 requires a live Pending and constant-time digest equality. load_source_stage13–26 separately requires both stored identifiers. preflight199–205 additionally requires stored request_binding to equal that authorization. Caller inspection found browser_start and authorization_prepare as the two StageStart constructors; the latter generates a different authorization id with browser_id None. |
| Stage/authentication expiry and use | preflight207–220 checks stage expiry, its now+600 upper bound and unused/uncancelled status before a write. It accepts only the expected unavailable/source_stage/continue-null state. Core resume_stage676–808 rechecks use, expiry, suspension/request hashes, private browser binding, login stage/source/nonce, linked account, live client and factor requirement inside its existing transaction. No caller-supplied request, subject, redirect or assurance flag enters the new Resume input. |
| Explicit HTML and legacy API | wants_html65–131 requires exactly one valid Accept field with positive text/html and no positive application/json; wildcard/no Accept/invalid quality do not opt in. Changed api2007–2031 performs only a read-only stage handoff for a stored browser-bound stage when browser_ui is enabled; unbound stages and the JSON branch retain Core resume plus stage_result. StageReference, original POST/cancel and stage_result are byte-preserved. Vary:Accept is added to successful negotiated GET results. The new dedicated page requires HTML; new writes require positive JSON and never negotiate an RP redirect. |
| Origin/header/fetch guard | write_preflight278–289 retains the exact shared browser_write_guard before Core invocation. http468–491 checks one exact Origin, one X-Riauth-Portal:1 and, if present, one same-origin Sec-Fetch-Site. No credentialed CORS is added; protect's existing CORS endpoint list excludes these routes. NoQuery and deny_unknown_fields inputs reject factor/query/body substitutions. Shared malformed If-Match/idempotency handling remains unchanged. |
| Existing rates, admission and privacy | The sole new category condition is api1076–1080: these child routes use source_callback before the general interaction arms. Effective agreed rate resolution, PostgreSQL shared counters, redb local RateTable, bounded App.run admission and outer privacy headers are untouched. Middleware still supplies no-store/nosniff and a default CSP; new private responses additionally receive no-referrer. The fixture's fresh router per request resets redb's in-memory rate table and therefore does not test cumulative rate/admission capacity. |
| No new writer or mint | The new module has read-only store.read calls; its only mutation entrypoints are the existing Core source_stage_resume/cancel wrappers, exactly once per accepted POST. It never calls standalone source_finish, browser_start, a direct authorization mint, a new StageStart or management writer. Normal existing Core completion still creates its internal session/code, removes the unreturned bearer mapping and records the browser callback; this is not a claim that the existing completion has no writes. |
| Bad factor versus rollback | source_finish174–198 persists attempts and returns outer Ok(inner Err) for an invalid local factor. resume_stage763–765 propagates that inner error; source_stage wrapper201–210 commits the writer before exposing it. The adapter's outcome translation occurs after that wrapper, with no surrounding store.write, so it cannot roll back the charge. Outer writer Err still rolls back partial changes under Store.write1047–1070. A committed rejected/cancelled result is a different case from an outer error and is not described as rollback. |
| Linked local account and assurance | Core reads the exact source/subject link, enforces any expected account, source fingerprint, user enabled/admin policy and authentication transaction in source_finish79–289. Local recovery removes exactly the presented digest; local TOTP advances its one-use step. AMR remains federated plus otp for local factor, without hardware/phishing-resistant inflation. The new fixture uses an explicit linked nonadministrator and auto_provision=false; no email selection or Group redesign is involved. |
| Authorization and consent boundaries | The adapter neither sets a decision nor implements a new consent path. Existing resume_stage773–792 sets the same original request's approve/transaction and invokes unchanged authorize_session. oidc854–1075 retains configured-reservation rejection, exact proof/hash/use, live identity/claims/assurance policy, request consumption and audit. Browser constructor103–111 rejects the combination with configured browser consent; reject_configured_pending1642–1654 remains intact. The existing stage's automatic completion contract is preserved; this fixture does not demonstrate a separate consent interaction. |
| Native query/form_post delivery | outcome291–302 drops redirect_uri and returns only status/code_issued plus the original local continuation. JavaScript never fetches that continuation: source-stage.js22–27 uses location.assign after exact local-path equality. Existing browser_resume_with254–312 checks the return cookie again, consumes Pending/user-code/proof, clears the cookie and points the browser at the saved session. api browser_response and response::callback83–140 retain query302 or the escaped form_post renderer, registered target query retention, nonce script, no-store/no-referrer and destination-specific form-action. Global portal form-action 'none' is not relaxed. |
| No factor/private URL or new logging | Only verified ids and server-built local paths enter escaped HTML attributes. HTML input has no name and buttons are type=button; submit is prevented. OTP/recovery input travels only in JSON and is cleared after a result/error. New code has no logging call or private credential rendering. auth.js47–53 supplies same-origin credentials/mode, explicit JSON/header, no-store and redirect:error without retry by default. Malformed JSON is mapped to a fixed message rather than echoed input; outcome errors use a fixed message. Existing observe113–167 labels the route pattern/method/status rather than raw URLs or credentials. Protocol code/state are emitted only at the existing validated native callback. |
| Encoding and issuer base | Paths148–163 encodes each id as a path segment, replaces space '+' with %20, prefixes page/continuation with cookie_path once and leaves JSON write paths relative for RiAuth.base. Page252–258 escapes attributes; portal_html replaces the base with its escaped configured path. local_handoff263–276 requires a local absolute path and configured issuer origin. Non-root production construction is coherent by inspection; F1 prevents the new non-root fixture from currently proving execution. |
| Cancel, reject and readiness recovery | A successful/rejected/cancelled Core result stays distinct in outcome and in JS. Cancel cannot be represented as authentication success. Ready GET requires authenticated complete state with the exact original continuation; it calls no Core resume and can deliver a result lost at the network boundary. Ready POST/cancel refuses. Missing/expired browser state or consumed delivery fails normally; stage expiry is not extended. JS shares busy across factor/recheck/cancel, uses guarded controls, one initial no-factor POST, no timed polling or automatic OTP/cancel retry; network/5xx ambiguity stops writes and offers reload. |

## Race reasoning and limits

Preflight consists of separate reads and precedes the existing Core write; it is
**not one atomic authentication-and-completion transaction**. The ordinary
Pending browser_hash has one construction assignment and is not rotated by a
new adapter writer. The exact stage/request/browser association comes from the
existing constructors, not a new caller-controlled input.

Two concurrent resume/cancel calls can both pass an early read. Core then
rechecks used/cancelled in the existing writer, so only the winning decision can
commit; final browser delivery independently consumes Pending once. Removing or
expiring Pending between preflight and a successful factor's callback write makes
stage_browser_callback1498–1514 fail and rolls back that outer stage transaction.
If an invalid factor instead returns its charged inner error before callback
delivery, it can still commit its attempt; I do **not** extend the design's
successful-completion rollback claim to every charged failure after a read race.
Neither outcome creates a new adapter bypass or extra code.

The suffix checks readiness/replay and fixture-corrupted/deleted/expired records
sequentially; it contains no two-thread scheduling/interleaving assertion. The
JavaScript busy flag is a client control, not proof of server concurrency. No
thread race, PostgreSQL isolation, browser click timing, fresh peer, hardware,
deployed HA or paused-process external-I/O exclusion was executed or inferred.

## Appended fixture: actual assertions read, not passes

The full suffix is one 1047-line addition,3799–4845, containing one
`#[tokio::test]` named
`browser_source_stage_transport_binds_factor_and_renders_both_response_modes`,
gated on Platform plus test-support. Its complete body was read in consecutive
ranges. All oracles below are **UNCOMPILED / UNRUN** here.

| Candidate span | Assertion/oracle inspected |
| --- | --- |
| 3812–3961 | Local fixture per response mode, explicit source links, nonadmin account, configured issuer base, root/body helper closures. Synchronous thread-local time only for enrollment/fresh login/recovery issuance; no clock scope crosses await. Recovery input avoids a live TOTP-edge dependency in HTTP. |
| 3963–4090 | Disabled browser behavior; legacy Accept variants; read-only HTML handoff/page snapshots; no-store/no-referrer/CSP, base and relative paths; exclusion of cookie/recovery/provider/private transaction/nonce/state values; static JS/helper fetch-policy strings. These are rendering/source assertions, not a DOM execution. |
| 4092–4147 | Fixture-only unsafe stage id proves intended attribute escaping and encoded path construction, then restores original records; snapshot remains strict. |
| 4149–4330 | Foreign/missing/duplicate return cookies, mixed pair, Origin/header/Fetch Metadata failures, unacceptable/duplicate Accept, empty/encoded factor query and body substitutions, malformed optional shared headers, pending result. These refusals compare all state except the existing operational HTTP-rate ledger. |
| 4332–4470 | Verified upstream callback, read-only prompt, local-factor status, wrong-factor401 with exactly one attempt, no use/code/completion audit, corrupted hashes/bindings/expiry/login associations and deleted/expired Pending with state preservation. F1 affects the /identity iteration at the callback boundary. |
| 4471–4532 | Recovery completion returns JSON without Location/bearer/protocol code, exact account/client/PKCE/redirect/nonce/source/link/fingerprint and federated+otp MFA, one consumed recovery digest and one source.stage_resume audit; repeat writes refuse; ready page303 and foreign-cookie delivery refuse. |
| 4534–4671 | Native cookie clearing/session access/Pending and user-code removal; exact query callback or form_post action/escaped state/issuer/nonce CSP; delivered code linked to stored grant; original verifier redemption, signed subject, invalid_grant replay, same single family/code/audit. |
| 4673–4845 | Separate fresh fixture: cancel, conflicting bound upstream account rejection and no-factor completion; native error callbacks with original state/issuer and no code; sequential repeat refusal; unbound legacy HTML GET and JSON POST/cancel query/form behavior. |

The no-factor case uses an account without enrolled TOTP and client MFA false;
it is **not** a new HTTP assertion that a trusted upstream MFA proof satisfies a
required factor. Trusted-MFA logic and existing Core assertions are preserved;
no historical pass is assigned to the new suffix. The suffix does not independently
assert every Accept/media extension, cookie transport or consent configuration,
and I do not invent such universal completion requirements.

## Actual body reads versus object/preservation checks

**Bodies actually read:** accepted design report1–570 including its complete
approved transport appendix; candidate api platform_routes676–871, its complete
changed stage GET2007–2031 with unchanged legacy references/result context, and
the changed rate classifier with full fixed9b protect872–1202 context; portal.rs
1–82; all353 lines of source_stage.rs, all35 HTML lines, all65 JavaScript lines;
the entire test suffix3799–4845. Existing test Upstream1741–1875 and
stage_client/authorization/codes1877–1915, charged-factor2818–2942 and local-TOTP
3376–3480 were read as relevant original fixture bodies.

**Baseline bodies actually read:** api App admission/run/blocking60–215,
router root/prefix/middleware267–340 and549–673, cookie/privacy/native delivery
1514–1663, OIDC callback/stage redirect1946–2047 and SAML ACS3936–3965;
browser_runtime constructor/one-use native delivery73–312, authorize_state385–395,
status1134–1190, browser session/path/cookies1460–1581, interaction and configured
consent refusal1622–1672; source_stage load/login/binding/wrappers13–215;
source_runtime source callback URL/form378–460 and500–594, complete
begin/resume/cancel/reject/denial595–892; source_finish1–289;
oidc other StageStart730–774 and authorization/proof/policy854–1075;
portal/http394–414 and468–491; complete auth.js1–175;
response callback/escaping83–140; store writer1047–1070;
error1–109, crypto test clock17–29 and TOTP257–323;
common snapshot35–75 and client/user132–179; standalone portal source review/
finish245–378 and callback claim/result source_callback1–103;
observability113–170. Caller searches confirmed the two StageStart constructors
and the ordinary Pending hash construction. Unrelated api handlers and all of the
original3798 test lines were **not** represented as fully reread merely because
their whole-file bytes were checked.

A route-name concern was also checked against the already-local, lock-selected
matchit0.8.4 source: tree.rs55–64 and658–715 normalizes parameters with per-value
remapping, supporting different parameter names on different descendant paths.
That source inspection is not a router-construction run or dependency validation.

**Separate object and byte checks actually passed:**

* Candidate parent diff has exactly the six claimed paths:1524 insertions and5
  deletions. Fixed9b comparison has those six product/test paths plus the design
  report inherited through the source branch. No older shared source stack is
  credited or imported.
* The original test blob `e124d964f3cb4c0ed680e98eebbd320d2a11c5ee` is an exact
  124619-byte/3798-line prefix of the candidate. Prefix SHA256
  `a65e2dd309b1260c3249f45dec14205b95eec142c6db72bb2dc743c2a9626e9c`; suffix 41399bytes/1047lines,
  SHA256 `5520824e6447fe176d3a1cc7344cf20c0c17a3dd964c22261cb52bb6a21c802c`.
  Exactly one new test attribute and named function appear in that suffix.
* Reverse just the route merge, rate condition and stage GET hunk in memory:
  the entire candidate api.rs equals fixed9b. Reverse the additive module
  declaration: the entire portal.rs equals fixed9b. Legacy POST/cancel,
  extraction/result, source cookies, global guard/CSP and other API routes are
  consequently byte-preserved.
* 512 existing tracked src/tests/Cargo objects outside the six claim paths
  match fixed9b at the candidate. The13 separately enumerated context objects
  below also match the source parent. Those equality checks are not body-reading
  or runtime evidence.

| Candidate file | Git blob | Bytes / lines |
| --- | --- | --- |
| `src/api.rs` | `a19ed8e0e875acec096ce55e665f90997a18760c` | 147317 / 4310 |
| `src/portal.rs` | `2b03aee0f2d453c1e5b396cfb2d7dc747a3b345d` | 2946 / 82 |
| `src/portal/source_stage.rs` | `042f13e2fdd1dfbf62e5646bf6c2b5054740c25a` | 11495 / 353 |
| `src/portal/source-stage.html` | `79493196cc921363deec6db4c1fdaceba1fa2661` | 1877 / 35 |
| `src/portal/source-stage.js` | `d94a1483c8c45491f951a4129c83a25fd9f69972` | 2897 / 65 |
| `tests/source_stage.rs` | `8b93c140b7c445f000a97fcc7f161da95af4b160` | 166018 / 4845 |

| Unchanged context file | Fixed9b and candidate Git blob |
| --- | --- |
| `src/assembly/browser_runtime.rs` | `f5ba027be118057b8ff491d168205f283caec4ac` |
| `src/assembly/source_stage.rs` | `d12ffffc0ea60de399decf3f089c66d9f42a6837` |
| `src/assembly/source_runtime.rs` | `51a46db7a7ad3e46914146461c79c6ae42ab124a` |
| `src/assembly/source_finish.rs` | `339e57f895b7203452f4ad6f384ac04e348fda19` |
| `src/assembly/oidc.rs` | `a62722e78e45f786e3901bd155395c7afc30be7c` |
| `src/portal/http.rs` | `0cd94a79b271c8026dc227b7c16229a1975937a3` |
| `src/portal/auth.js` | `e07260923ebb34995ef4d31a1b4605980626df7c` |
| `src/response.rs` | `5ce29e8f70b10e049c93bc98afe02b2ae02e4f39` |
| `src/store.rs` | `8e78d889220c9e956fe00a4f7a69b603063d3ebf` |
| `src/error.rs` | `5154328d29ee832ddc1ea24f029bfe47028e6211` |
| `src/crypto.rs` | `f503b883ae7af5259ae159cd87d5da2dd629b1cc` |
| `tests/common/mod.rs` | `9a40464e4ca311c7c37fe7cee7d5616aeefb73c7` |
| `Cargo.toml` | `5660d4bb922fcdc5bfe05d7502585980f4720d06` |

The source_callback and standalone portal_sources objects separately compared
equal as well: `c9644367d1e7e8919d761d7d5e81c3d4c70be220` and
`1587611ee4355f776d7cd0b7c0a818c5969ba9af`.

## Checks and remaining scope

Actual local checks: clean own branch/HEAD; ancestor/docs guidance and CONTRIBUTING
read (no applicable AGENTS.md found); immutable object/path reads, complete new
body/suffix inspection, whole-file/prefix reversal and the equality/count/SHA
checks above. `python3 scripts/check-docs.py` exited0 **before this report**
with “Markdown links and build-directory layout checked”; no pre-existing output
directory or missing-link flag appeared in this worktree. No checker, build
directory or cache was changed or cleaned. New-report reference/scope/whitespace
and documentation checks are recorded below at commit preparation.

Read corrections were source-only: an initial combined design output was truncated
and its missing spans were reread in bounded ranges; a guessed
`src/api/telemetry.rs` object did not exist (exit1), corrected by the actual
`src/api/observability.rs` module declaration and body. Neither is a candidate
compile/runtime result. No private state or credential contents were printed.

Root's smallest remaining local seam is the exact F1 mock callback correction,
with protected-prefix ownership resolved explicitly, followed by whatever bounded
runtime release root chooses. The accepted design's one future exact filter
remains unexecuted here; no additional benchmark, provider campaign, release gate
or universal browser requirement is proposed. No Cargo/native helper/server/HTTP/
provider/browser/desktop operation ran. Existing receipt-secret behavior,
route-specific headers, permission/review/removal/audit order, PAM fallback,
held Group and60s/nonrenewed admission limits remain preserved source contracts,
not newly executed evidence.

Commit-preparation checks: all27 distinct referenced Git objects resolved;
source body/suffix LF-slice SHA256 values recomputed, including stage GET2007–2031
`c72ddd12aa3385a1aa0372cc980629ea8c96bb19765be82f5932c4c07d48028f`.
The explicit F1 source-fact assertions, sole-new-report status, final LF,
fence balance and trailing-whitespace checks passed. The documentation checker
also exited0 with this report present. `git diff --check` exited0; staged
whitespace and exactly-one-added-path checks are repeated immediately before
commit. These checks execute no product/test code and do not change any oracle's
UNCOMPILED / UNRUN classification.
