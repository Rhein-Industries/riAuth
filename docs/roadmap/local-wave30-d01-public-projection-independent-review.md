# D01 public decision projection — independent source review

Date: 2026-10-03, Europe/Vaduz. Reservation
`wave30_D01_public_projection_independent_review`; project
`891e7443-8dac-4c1b-897f-9e53cb59c7ee`, existing isolated worktree
`ed9ac424-59f4-4520-905b-919aea3521eb` only. Entry parent
`571822b2e72d16455ba8948f86ea93d674beb049` was clean. This new report is
the sole write. No source alignment, import or runtime reservation occurred.

## Recommendation and exact boundary

Accept the exact archived projection candidate for source composition review.
I found no concrete new defect requiring a correction within this observational
seam. Its complete bytes and AST reverse to the pinned 9f candidate. The new
output associates fixed public role/name/action labels with a typed ref from
the same returned snapshot; it does not choose or dispatch an action.

This recommendation does **not** assert that action associations are unique.
The projector deliberately preserves multiple matching provider refs. The
author's explicit requirement that root refuse an ambiguous association before
input remains necessary. Neither array order nor the first matching ref is an
admission rule. Root must select a unique association from the latest successful
projection of the same owned session/target/tab and pin its expected public
observation. The unchanged dispatcher checks freshness, not that external
role/name/action decision. Treating this output as an automatic action
authorizer would exceed the reviewed design.

The candidate, its projector, archived checker and composed browser fixture
remain UNEXECUTED here. There is no nonblank Driver observation, GUI execution,
confidential application journey, or whole D01/D05 closure from this report.
Root owns composition, any later memory/runtime reservation and disposition.

## Full reads and immutable source witnesses

I read all 767 lines of
`48288c1f10f24d47bb86b57edd0682014e9b64ee:docs/roadmap/local-wave30-d01-browser-decision-projection-plan.md`,
including the complete 508-line JavaScript candidate, both complete diffs,
all preservation tables, the 54-line archived checker and limitations. The
report is 60522 bytes, SHA-256
`16f0e6c4b77469d3057489e73bb1bea93cfe2668b76171c244594827a75a8321`.
The canonical candidate is document lines 154–661, including its final LF:
33701 bytes, SHA-256
`505b1c0fec96e248dc67db46379bd801fa06a2678462aa094fb2fc1bc5ea398f`.

The complete 485-line 9f candidate was previously read in my
[continuation review](local-wave30-d01-continuation-correction-independent-review.md).
This audit extracted and reparsed its entire immutable source again and
compared it with the entire new candidate in both directions:
`9f3a4a372b53fc6d73d5df45ad1a8d217ad60879:docs/roadmap/local-wave30-d01-continuation-correction-plan.md`,
document lines 227–711, 32502 bytes, SHA-256
`7ab23019e838512a84600d4069fef2e419b9bb7f07ff5aaa2c7477af956f9ca8`.
The two changed old function spans are also present in the read inverse.
This is whole-source comparison, not acceptance based only on named hashes.

I fully read the pinned c01 signin HTML, signin.js, shared auth.js and
interaction.rs. A combined source read was truncated at the tool output
boundary; the remaining JS and full auth.js were read separately before this
conclusion. Selected browser_runtime.rs bodies were read at lines 385–398 and
1130–1180; I did not claim a fresh full read of its 1740 lines.

All five source identities below were independently checked from
`c01c39ab4e092423d5522bedc50fff87656d8c0a`:

| Source | Git blob | Bytes / LF lines | SHA-256 |
| --- | --- | --- | --- |
| `src/portal/signin.html` | `e3e120898822bdfe331e4dc2d3fdeb59be048b08` | 6470 / 80 | `af5c163944bb397d6649c932fa91e749381772d307c685a95f53cd3f6dee5f44` |
| `src/portal/signin.js` | `af602f2449f83a7489f5c8f6aa47d737dfc523c8` | 23197 / 373 | `3d5ee4cfc1b3a88b3d6fa46d064df36b7f1a227d03a1a557830eac4528ccff22` |
| `src/portal/auth.js` | `e07260923ebb34995ef4d31a1b4605980626df7c` | 9235 / 175 | `c1a1aa440ca0144183019959b94bf950095309cbc414ec7bcffefbf48cf63ae3` |
| `src/api/interaction.rs` | `d4714c7590b251e8d94589a6b0b4f6f1e7bc32bd` | 10192 / 311 | `d4ed262fac0bbd167c3dd999f260ce7b26e9c61bcb5072bb0b380028cc2b38b1` |
| `src/assembly/browser_runtime.rs` | `f5ba027be118057b8ff491d168205f283caec4ac` | 72437 / 1740 | `d0548f68ef985fc2b59f72015bca4fe957e89895cc8c870ff48f07b6a5f3c16f` |

I reread the complete 225-line controller command at
`2e29c30d01ccd41a2aac3914e3ac20afa38d324f:docs/roadmap/local-wave30-d01-user-browser-review.md`,
document lines 7698–7922. Canonical bytes exclude the delimiter LF after final
`PY`: 17326 bytes, SHA-256
`5eb6ab2d3d23b49fd431c41f1932082fe0c6779ad724ac77b7fded10926b26b0`.
Its client setup explicitly creates `local-demo` with name `Local demo`,
confidential mode, fixed callback, `openid,profile` and private secret file.
No generated private input was inspected or substituted.

For the unchanged helper I read the relevant complete cookie, get, reply and
main setup/outcome spans at lines 531–682 from
`f4ef05d8428b235511e81277ba6b4b72d5ec08ba:scripts/d01-confidential-browser-demo.py`.
Its whole-source identity was checked and Python AST parsed only: 35749 bytes,
757 LF lines, blob `a01b1f3f81f0978eb0a02ed339c7248cae485350`, SHA-256
`75bfb8a63d68aee6ee6b2e500959c0e7d80a6331f1c690022c4300134c7d34f1`.
This is a selected-body reread and whole-object check, not a claim of fresh
full-helper semantic review or execution.

## Twelve public pairs and their actual source meaning

The projector's declaration has exactly twelve distinct literal pairs. Output
role/name strings are copied from this declaration after equality checks,
not copied from arbitrary snapshot fields.

| Canonical role | Canonical name | Source witness / permitted projected action |
| --- | --- | --- |
| heading | Local demo | Helper reply lines 575–582; observation only |
| heading | Sign in to continue to Local demo | signin.js lines 10, 41, 158–160 and fixed client name; observation only |
| heading | Local demo wants to use your riAuth account | signin.js line 216, required consent; observation only |
| heading | Continue to Local demo? | signin.js line 216, optional consent; observation only |
| heading | Protected application access | Helper reply lines 575–582; observation only |
| button | Sign in | HTML line 40 and helper line 578; `click` only |
| button | Allow | HTML line 59 and signin.js line 229; `click` only |
| button | Continue | signin.js line 229; `click` only |
| textbox | Username | HTML line 37; `type` only |
| textbox | Password | HTML line 38; `type` only |
| textbox | Authenticator or recovery code (if enabled) | HTML line 39 and signin.js line 186; observation only |
| statictext | Signed in. Protected application access is available. | Helper lines 558–560 and 580–582; observation only |

These are source-backed expected semantic roles, not an executed accessibility
or nonblank Driver measurement. The metadata survey below observed only an
inert rootwebarea. No actual password textbox role or `type` capability is
inferred from that blank page. Missing roles/capabilities fail to produce an
action association; there is no field-index or geometry repair.

The fixed client name is resolved through browser_runtime.rs lines 1144–1146
and returned in `application.name` at line 1162; signin.js derives `app()` at
line 41. Required and optional consent labels are both legitimate source
possibilities, not assertions that this fixture reaches both. The optional
OTP label differs from configured-TOTP `Current authenticator code`; the latter
is not added. Password/OTP visibility and required flags remain product-owned.
No passkey, OTP, terminal approval, cancellation or logout action is added.

signin.js sets application/account/host/error text in other elements. Those
elements are outside the canonical pairs. The protected helper renders a
fixed success sentence, without its subject; it still requires the fresh
application cookie and all existing protocol checks. These source predicates
are not replaced by a heading or a projected ref.

## Same-snapshot association, ownership and ambiguity

Candidate `page` starts at line 311, continuation at 387, projector at 435 and
publicPredicate at 462. The new assignment at line 329 takes the same raw result
that supplied the unchanged page flags and fresh-ref extraction. The projector
does not fetch another page, retain the raw result or join a label from an
earlier snapshot to a later ref.

For observed pairs it matches `role` and `name` together in this result's
`content_refs`, then emits each canonical pair once in declaration order.
For action associations it reads the role, name, ref and advertised actions
from one element of this result's `refs`. It requires the exact canonical
pair, a string matching `^p[0-9]+:[0-9]+$`, and the matching `click` or `type`
advertisement. Emitted action is a fixed literal; extra provider action names
are not copied. OTP has no actionable association.

The observed and actionable lists are separate projections of the provider's
two lists. This does not prove that every action pair also appears in
`content_refs`, or that every matching pair is unique. It does not independently
validate a ref prefix against `snapshot.id`. Those are not added claims of the
proposal: same-snapshot origin is established by using one returned object and
the existing Driver ref contract, rather than by a new token verifier.

Multiplicity is retained in provider order with `flatMap`; no rank, coordinate,
array index, nearest element, first-ref selection or automatic action follows.
Root must reject a duplicate/ambiguous intended association or inconsistent
token association before constructing a decision. Absence is not permission
to use another label, old ref, geometry or a terminal/API substitute. This is
the explicit source-plan boundary, not a new runtime result or an assertion
that the cell itself implements the root decision check.

I read the actual enabled tool descriptions for `get_browser_state`,
`browser_click` and `browser_type` from tool metadata, without invoking any
of them. Their contract describes exact owned session/target/tab binding,
semantic_v2 action/content refs, `p<snapshot>:<index>` scope, and invalidation
on navigation or a newer snapshot for that tab. Type requires an editable
latest-snapshot ref. Click's explicitly requested `dom_event` route is synthetic
and dispatch alone does not prove activation. Metadata exposes no complete
typed-ref field/action-enum JSON schema; this review does not invent one.

I also fully read the retained public blank survey at
`8a2c4a192d04a43cd930141506a5215cca343542:docs/roadmap/evidence/wave30-d01-root-blank-driver-contract.json`:
3893 bytes, SHA-256
`b8145f029796a2063dfe167f026fe299d85afcaedbec5a7abacbeb5bf1a5ea27`.
It records an exact Driver-owned binding, complete viewport semantic_v2
snapshot, one inert rootwebarea, no actionable refs and a seven-field omitted
count map. Cooperative close was unverifiable; exact-owned kill and session
end left no owned window/PID. It was already cleaned. It establishes this
observed blank shape only, not a live session for this candidate, nonblank
label/action support, all failure variants, or a hard MCP timeout.

The unchanged snapshotArgs use the previously owned session/target/tab.
Continuation requires the supplied ref in `own.fresh_refs`, clears that list
before input and obtains another snapshot afterwards. Root must refresh its
association after each single-use input. A cached `public_decision` is not a
second permit: its display does not bypass the fresh-ref check or provider
invalidation. Navigation/input code, decision kinds and private password
handoff are whole-byte unchanged.

## Privacy, refusal and product protection

Projection output has only `observed` and `refs`, and their fixed role/name
plus fixed action and namespace-filtered provider ref. It reads no node value,
account, username value, password, OTP, subject, host, URL, query, cookie,
sender, header, HTML, state ID, exception text or provider error detail.
The twelve name constants are fixed fixture labels. The allowed ref is the
existing provider handle, not a credential or a fetched URL.

Tool error, non-ok status, incomplete snapshot or the fixed RP error label
produces empty projection lists. publicPredicate then requires a canonical
observed exact pair; it no longer accepts the old role-by-label Cartesian
product or obsolete OTP wording. The existing checked wrapper retains a fixed
failure and enters cleanup when a predicate refuses or throws. No raw snapshot
or exception text is added to that failure output.

The extra successful-output property is observational and keeps
`journey_credit:false`. It adds no new writer, credential mint, HTTP request,
helper invocation, terminal approval, dispatcher kind or product permission.
Specifying an expected post-input label remains distinct from proving the
input succeeded: existing `confirmed`/`unverifiable` dispatch handling still
requires the subsequent snapshot and helper outcome.

The read product context retains browser binding cookies, browser-write guard,
credential floor, strict input structures and live account/session_ref checks
on consent. auth.js retains same-origin cookies/Origin behavior, the portal
header, redirect refusal, guarded consent activation and non-retry credential
POST; signin.js retains original resume-path validation and credential/OTP
clearing. Nothing in this archive edits those sources or relaxes them.

## Independent byte and AST checks actually performed

I used an independent Python text extractor and zero-context patch applicator
over immutable Git objects. Each diff has five hunks; all coordinates, removed
lines and added-line counts were checked. Applying forward reconstructs all
33701 candidate bytes; applying inverse reconstructs all 32502 baseline bytes.
No surrounding source was replaced or materialized as an executable file.

| Static object | Bytes / LF lines | SHA-256 |
| --- | --- | --- |
| Forward diff | 2405 / 46 | `b6f884994720949a44bfb417ff68de56e71e04f04d7357686ffc71452a873ed2` |
| Inverse diff | 2405 / 46 | `3a9053840f82174e4a761cd319b04e82bf2c62c2cc31adccb4a320804e968ae2` |
| New projector function span | Source span only | `414fa7865ac064f9c907af5badf4294ebaa43df319ad56facccd6033b7047feb` |

An independent parser-only Node program used bundled Acorn with latest module
syntax. It parsed both full cells and both full pinned product JS sources.
It normalized away location fields and normalized AST BigInt data; it did not
evaluate any source under review or invoke the archived checker. Structural
inverse removed only the added page assignment, the new projector, the new
successful-output property and restored only the old predicate node. Entire
normalized AST equality then held.

| Normalized AST | SHA-256 |
| --- | --- |
| Full 9f baseline | `786f743a839085292b57dc0c695a97c338d57eef9ab314168ff081da3e100a74` |
| Full proposed candidate | `cce3210177ab2e377a58153c62a2c50f047b031f6e979e0ac17d22b4a46235ce` |
| Complete structural inverse | `786f743a839085292b57dc0c695a97c338d57eef9ab314168ff081da3e100a74` |

Thirteen old functions remain; only `page` and `publicPredicate` differ. The
eleven byte-identical complete bodies are finiteObservation, diagnostic,
latch, observeController, pollController, timedDriver, cleanup, checked,
navigateAndSnapshot, entry and continuation. projectPublicDecision is the
fourteenth function. All five collector declarations are byte-identical; their
decoded literal bodies were Python AST parsed only:

| Collector | Bytes | SHA-256 |
| --- | ---: | --- |
| CLOCK | 114 | `cfb3f3154cb778388724728b6f7cb804bd7e142844ded7c044afc8dd3135ab4d` |
| STOP | 1600 | `5de5244440b134689bedfb1235e805c65a8b726e753cbdac1d0831ef1c83d071` |
| READBACK | 4424 | `b7d3504245ae94a584b455fa65826fbaa7759ec7a50a7b81c2cf8284c9f9051f` |
| MARKER | 626 | `13c6549e0328ad695769938cbc0be7eb47d7a10ea9f00c4d897655603437a949` |
| PERSIST | 805 | `819a7785625ce2f78a9cb03b5f242dbfa6728f9f507b242e3c048887ad01ea30` |

The exact controller Python body and helper were also AST parsed without
imports or main execution. Python and the independent Node parser exited 0.
There were no failed candidate executions or failed static comparisons in
this audit. The earlier author's two extraction errors and quoted-path Git
search correction remain dated source-read failures in its report.

## Historical results and remaining boundaries

I fully read and JSON/hash checked the published descriptor-memory receipt at
`35c3fd3007c52d8142c7bee1d69aee127cc42a95:docs/roadmap/evidence/wave30-d01-descriptor-memory-0ec651d.json`:
5638 bytes, SHA-256
`5df541995dd1f74bff9c91104400140040d986a98a9f13f8688fbcb82cc4782a`.
Its actual root-run payload
`b10cf8376edbb80b00c051305f31643b04714494260c4a49576a354361048779`
passed 128/128 memory cases: 50 observer and 78 legacy, child exit 0/reaped,
one spawn, zero traceback/str/repr traps. Child time was 0.101654 seconds and
controller time 0.201888 seconds. This proves the recorded helper-memory
scope only. It neither ran this projector nor supplied a browser journey.

The earlier 18-case cleanup memory result, older 78 legacy memory pass and
61/76 failed designs keep their original scopes and outcomes. The existing
[boundary review](local-wave30-d01-controller-boundary-independent-review.md)
and continuation review are preserved; no later projection recasts their
evidence. The c88 prepared-180 actual failure remains failed with
`unexpected_failure`; original sender, cause, lost values, true first-cleanup
clock and whole-cleanup-within60 proof remain UNKNOWN.

Entry phase fix, password retention until actual dispatch and clearing at
cleanup entry, exact final helper-zero allowance, same-owned-invocation
partial-line drain and 16384 cap all remain byte-identical to 9f. Receipt
before comparison, first-failure latch, owned joins and fixed cleanup output
also remain unchanged. The 840-second active/900-second inclusive limits and
individual Driver observations are not hard interruption guarantees. Whole
cleanup within60 remains false/unproved; no new lifetime IO, deployed HA or
paused-process claim is added. Existing security/receipt/header/PAM, held Group
and nonrenewed60s contracts and closed rows are untouched.

No candidate, case, function, VM, eval, memory stub, archived checker,
controller, collector, helper, module main, product CLI, native provider,
HTTP request, network query, browser, Driver method, Cargo test or runtime
executed. Static parser programs and documentation checks are the only
executions. No runtime slot was acquired or released, and no worker was
contacted. The preparation/private-input adapter and composed next-decision
review remain separately owned; this report introduces no schema for them.

## Documentation and handoff checks

`python3 scripts/check-docs.py` passed before the report
(`Markdown links and build-directory layout checked`), with no preexisting
flags. The report-aware checker, final LF/trailing-whitespace check,
`git diff --check`, immutable reference checks, staged sole-path scope and
post-commit cleanliness accompany the handoff. No report cache cleanup or
source edits were performed. Prior reports and every other tracked file
remain identical to parent 571822; the final commit contains only this report.

Root may accept this precise source seam and independently review any changed
memory/composed fixture before releasing it. This recommendation carries
no new execution evidence or authorization to run the real fixture.
