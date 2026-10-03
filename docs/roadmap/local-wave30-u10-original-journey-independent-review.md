# U10 original browser journey: independent source and evidence review

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`. Original U10 task:
`934531ec-3a80-497c-b931-0d86b298ead2`; original primary worktree:
`cef37394-7d96-42ef-93d3-003ef3a4420f`. Supporting worktree:
`e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`, branch
`roadmap/local-extension-isolation-wave27`. Reservation:
`wave30_U10_original_journey_independent_audit`. Review date: 2026-10-03.

**Original U10 completion is not established by the inspected evidence.**
The accepted component journeys supply useful browser and virtual-factor evidence,
but do not establish the complete original journey with assistive technology and
supported physical authenticators. The current missing-credential correction is
present; its newly selected regression has no supplied terminal result yet.
There is one small, reachable source follow-up: an empty **required OTP** focuses
an error while leaving the OTP field without invalid state or an error description.
Reserve only that feedback and one focused no-request regression if root chooses
implementation. This is a source-derived omission, not an observed reader failure,
authentication bypass, or claim that its correction would complete U10.

Only this new report changes. No original task status/assignment or U01–U09 outcome
changes. No validation/Cargo/desktop slot was acquired or released. The review
uses fixed Git objects, not another worker's mutable source or private fixtures.

## Original row and prerequisite outcomes

The complete exact UUID row and complete U01–U09 rows were read from the local
project export at:
`/Users/dominik/.local/share/riwork/orchestrators/projects/891e7443-8dac-4c1b-897f-9e53cb59c7ee/planning/current-tasks.json`.
The export is 244347 bytes, SHA-256
`5cfd882213745a86dcdeeb7e4debccf24fd22299c0fc7be73d6d988512739752`,
mtime `2026-10-03T07:27:51.148464+00:00`, with 94 rows. This is dated local export
provenance, not a fresh RiWork API/status query. Its U10 row says `todo`, primary
`cef37394-7d96-42ef-93d3-003ef3a4420f`; root owns current board reconciliation.
The dated leave-todo/wait scheduling sentence does not cancel this authorized
source audit or authorize execution.

Exact requested outcome:

> Test complete journeys with keyboard navigation, assistive technology, small screens, and supported authenticators.

Exact workstream goal and completion gate:

> Ordinary users never need the terminal to manage their accounts.

> A nontechnical user can accept an invitation, enroll a passkey, access an application, manage factors and sessions, and recover access entirely in the browser.

The row also requires relevant implementation/tests/docs/artifacts as applicable,
actual verification and residual inputs; a report alone is not completion. Shared
identity, authorization, revocation and credential protections remain required.
Its desktop instruction is RiWork Cua.ai Driver only, with setup/permission failure
reported rather than a provider switch. No desktop was needed here.

All nine proposed prerequisites are `done` in that export. This audit does not
reopen their implementation or add a universal all-target gate.

| Prerequisite / exact task UUID | Accepted original outcome relevant to the journey |
| --- | --- |
| U01 `8546e10c-0eda-48d8-aa75-872019157b3c` | Ownership-verified, single-use first-administrator setup. |
| U02 `f6edc44f-720d-4c75-b137-f635922ccf67` | Browser invitation/email acceptance, expiry/revocation feedback and scanner-safe links. |
| U03 `3a8ddc0e-723c-45d0-8031-0730de5109a8` | Passkey enrollment/sign-in/additional keys/rename/removal/cancellation and fresh sensitive proof. |
| U04 `473b40e8-d611-457f-9584-9faac9db938f` | Browser password change/reset retains MFA and account-type boundaries; password reset is not lost-factor recovery. |
| U05 `4a04e19e-5732-4a9f-ab52-8c45b174af45` | TOTP/recovery-code enrollment, confirmation, replacement, removal and rotation. |
| U06 `4ca5c346-83c6-4ebe-8f80-23c712394537` | Sessions/consents listing, selected/all-session revocation and consent withdrawal. |
| U07 `e623b017-7e97-4ed9-82e5-37e104c7c7e3` | Browser review and approval/rejection of device application/access requests. |
| U08 `9f023192-3804-4043-b156-cc163a7ee33e` | Browser upstream login/link/unlink and local-factor handling without a terminal handoff. |
| U09 `9ebbbe0d-4b0f-4d12-a734-f4eae08d9e41` | Passkey-only administrator creation, ordinary administration, independent backup and emergency-recovery boundaries. |

These outcomes supply prerequisites; their `done` labels do not themselves prove
U10's separate complete-journey assessment.

## Fixed source, complete bodies and reused evidence

All current source statements below refer to published
`1a517a1a461b7017c353d37a5e498d2c7cfa7985`. Own report parent is
`d8b97a59bb07610353a8847227437fe97a658a86`; no source alignment/merge occurred.
The sign-in JS and test are whole-byte equal to authored
`6800973556f45ed2136467bde1e7d74cd98aa633`, integrated as
`1406de9e5349d3bb9ef06f55e638938129318a34`.

The following complete bodies were read in bounded slices in this audit. Hashes
are SHA-256 of exact fixed Git blob bytes, not hashes of rendered tool output.

| Complete body | Bytes | SHA-256 |
| --- | ---: | --- |
| `docs/roadmap/local-wave30-u10-accessible-journey-plan.md` | 45994 | `be6c6495d6a4ccd678aff9013715fb10be471cb72edffc7112856292dec07d5e` |
| `docs/roadmap/local-wave30-u10-root-source-review.md` | 3007 | `84631247cae03e311432e72ef8b9e0db88e42680f4bd5c4a9e9873b5be97a01f` |
| `docs/roadmap/evidence/wave30-u10-missing-credential-source-root-review.json` | 1024 | `ef1b07ebaa2de1de7fc5ca1e328649de342d43ea925760182398cdd4333f1a0e` |
| `src/portal/signin.js` | 23771 | `d42f6ead02c1c27f55778b75b00d7ede654bbe669004aa07c964088124e0797b` |
| `src/portal/signin.html` | 6470 | `af5c163944bb397d6649c932fa91e749381772d307c685a95f53cd3f6dee5f44` |
| `tools/browser/signin.spec.js` | 36605 | `2cc417e36f76e4455fa78488e2019f19abb05999323b8e7cd8193b0d1ec49c88` |
| `tools/browser/accessibility-journeys.spec.js` | 14959 | `01b41cb562b8ca41bc16de1c59ef7d6f8f51c743e82d6b684bc5144e869c8dec` |
| `tools/browser/invitation-passkey.spec.js` | 19053 | `93b5c5c9d488858d9f76e80c3af14a9c042122c963b724dd91904d67c8721ca5` |
| `tools/browser/authenticator-recovery.spec.js` | 16772 | `df954147a4f4d3505ca0957fa35253f21a83bf1bfe0cc3e25607eada716ff1d9` |
| `tools/browser/password-reset-replay.spec.js` | 18482 | `d9fb0048c935385c226900f690717e137d397824a282209c1199e5cabe2223cd` |
| `tools/browser/multi-authenticator.spec.js` | 31081 | `137953b2d18440fb41d3570ff9d6bc98bad03c8af2fc8ffc408ffd95a2fe5720` |
| `src/portal/self_service/security.js` | 14328 | `f4e9c56247a8f315ad5ec6b5c8931d61443e13f89d1e58e6ce32987b2596d38b` |
| `tools/browser/fixture.js` | 4521 | `b886dac64616015a791f48e2784a6adf84733e7f2f5a1b2dabb2e68e3172bc8b` |
| `tools/browser/playwright.config.js` | 1330 | `24c63e0563b0f7e1333a7bea03445297f69c87519bc8af44c313dd84d880b5a1` |
| `.github/workflows/ci.yml` | 12124 | `0f152ab75e490dd3f9f614714a776eaa7e2714fc518900b7f7ba8d7cb24a5949` |

The complete U10 plan's earlier review covers portal `index/app/auth/account/setup`,
self-service security HTML, device UI, HTTP guards and interaction handlers, CSS,
and the remaining invitation/rename/revocation/setup/admin/portal fixture bodies.
That is attributed prior body review, not a claim that this audit newly reread all
those production bodies. Whole-blob comparison to its fixed `544d1340b80cd3e040dc13142cdcbc1d75fea4cb`
established unchanged portal `index.html`, `app.js`, `auth.js`, `account.js/html`,
`setup.js/html`, `device.js`, self-service security JS/HTML, and the browser fixture,
config, accessibility, three invitation, factor-recovery, reset, multi-authenticator,
rename, revocation, setup and portal files. The two changed sign-in bodies were
newly read completely here. No source function or test import was evaluated.

## Journey coverage and the limits of each observation

| Original facet | Actual source/oracle and accepted evidence | Remaining attribution limit |
| --- | --- | --- |
| Invitation → passkey → later sign-in | CDP invitation checks expired/used refusal, empty name feedback, native WebAuthn enrollment, no session merely from accepting the invitation, later passkey login and logout. Password/shim invitation paths have reported three-engine execution. | Loopback SMTP supplies invitation links. Chromium CDP supplies automatic presence/UV; Firefox/WebKit skip that ceremony. Shim UV is set by the test. No external mailbox, physical key, synced or phone-hybrid passkey is proved. |
| Application access | `signin.spec.js` defines keyboard password/TOTP, consent, issuer/state/PKCE callback and code redemption, silent remembered-consent return, account switching and refusal. | Its RP callback is a local HTTP stub; token exchange uses test-side fetch. These definitions are not a new runtime or the D01 confidential application journey. Terminal-approval alternatives are not no-terminal user completion. |
| Factor management and recovery | Factor-recovery defines keyboard enrollment/error feedback, ten displayed-once codes, clear-on-close, one-use codes and replay refusal; another browser's session epoch is rejected. Multi-authenticator defines two independent shim contexts, name/registration preservation, first-key removal/refusal, remaining-key sign-in and retained TOTP. | Test-computed TOTP from the displayed setup key is not a phone app. Two shim stores are not two physical devices. Lost-all-factors recovery remains the printed operator boundary, not password-reset success. |
| Sessions and consents | Current security UI uses live snapshot/management authority, identity/session binding and recent reauthentication; selected/all-session revocation and consent withdrawal stay scoped. The accessibility session case defines reauth feedback; factor/removal cases check another context's live 401 even while its cookie remains. | Component checks are not an observed single end-user complete journey through every management screen. Revocation is not inferred solely from cookie clearing. RP-side logout remains separate. |
| Password recovery | Reset fixture checks unknown/unverified nonenumeration, no reset mail for those accounts, mismatch/current-password refusals, one-use verified-account link, no automatic login, other-session revocation, old-password refusal and new-password login. | That recovery account has no factor; page text about retaining factors is not a measured factor-retention journey. Fixture mail is not an external mailbox. Mail-unavailable error is not successful recovery. |
| Keyboard and small screens | Accessibility fixture defines eight focused cases; selected screens assert focus, outlines, target bounds, axe, 320/768/1440 widths and doubled DOM font size. Two-authenticator uses 390×844 CSS bounds, keyboard traversal and Enter for actions. | Some setup/navigation uses fill/click, scrolling or programmatic focus; WebKit/macOS uses Option-Tab, with recorded engine-specific backwards traversal. CSS viewport/font changes are not a physical phone, mobile keyboard or native browser zoom. |
| Assistive technology | Label/alert/live-region/description/focus source and axe definitions provide inspectable semantics. | No supplied accepted complete spoken screen-reader journey, reading order or announcement trace. Axe/static DOM cannot substitute for that original input. |
| Supported authenticators | Virtual native Chromium WebAuthn and shim paths have bounded reported executions. The prior U10 plan attributes U01's 2026-09-28 Cua.ai Driver observation to reaching a backup Touch ID prompt. | Backup completion was expressly unclaimed; prompt images/private fixture were not read here. No complete physical/synced/hybrid or real-mobile journey is credited. No invented every-device certification campaign follows. |

The config defines Chromium/Firefox/WebKit engine projects, one worker and zero
retries. Test dependencies in the read plan are Playwright 1.63.0 and axe 4.13.0;
no tool/version was invoked here. Its Firefox automation COOP preference affects
the test browser, not product headers. Fixture startup defaults to 120s with
beforeAll allowance; its stop helper awaits child exit without an outer deadline.
A future execution still needs separately reviewed finite own-process cleanup.

### Historical results, source identity and current CI

The complete current U10 plan reports the following historical accepted results.
This audit read that complete evidence body and compared the specified Git blobs;
it did not open historical private logs/traces, rerun tests, or independently grade
their old execution envelopes.

| Historical source / accepted commit | Reported actual result | Current-body qualification |
| --- | --- | --- |
| `0d590fa316a494307650a23751317c3e6bcb16c4` / `099c19047ecde2ab990f7c7e9dada604981b9ff8` | Empty administrator-password feedback, 320px, Chromium/Firefox/WebKit 3/3; alert/invalid/association/no cancelled mutation. | Later accessibility file includes workspace test. This is a component result, not AT. |
| `e3036220e37ad99140a5a681e0da4780a7308d74` / `5892563fddecaf5446c4d7219a0cacb4e236e551` | Empty-workspace feedback 3/3, associations and input clearing. | Prior plan reports current accessibility blob equality; no fresh browser credit follows. |
| `94764690429918a7a8189322c78d522817f0b544` / `bd35698b08394d712a9d71e44ca084407aa476e5` | Shim passkey-revocation 3/3, no skips; removed key and other session refused. | Exact current revocation blob equals historical source. |
| `3b2c2d0a663a47f45d4e1504bd47d11d8b9fb207` / `9359b80ff2ab2c27b9c8791b4db166d491174530` | Recovery 8.2/9.7/8.8s on the three engines, no skips; counts 10→9→8. | Exact current recovery blob equals historical source. |
| `6bceffb1122d282b7ef1fba327d0f025db21b003` / `d08f8e734cc22cfb3937b38910d4336ab5327677` | Reset 6.2/6.1/7.1s, no skips; separate mail-off refusal 829ms/1.2s/808ms. | Current reset is **not** whole-byte equal: later capture selection isolates that account's one reset while permitting fixture invitation messages, still excludes ineligible resets and requires exactly one reset message. Original result stays at original source. |
| `d1f675d7410e05475f9acf2a90ac27604dfb60de` / `bb55a50c7bc4fca79b9f4e34b2deb2efbc09d22c` | Rename 3.9/4.8/5.0s, no skips; keyboard errors/trim, credential/session preservation. | Exact current rename blob equals historical source; identity proof, not a rerun. |
| `ec5f0140388b4a5a4e92d08ea2bb9c4eb2e10532` / `d039306bae4a30f763102802686a85beb8b6c54a` | Two-authenticator 10.0/11.4/11.1s, no skips; first refused, second/TOTP survive. | Exact current multi-authenticator blob equals historical source. |
| `b77e6370504f13fd5dcf87f46aad6a8121299542` / `a72a08630570f6830f4987919d5173f0ce1f40e3` | Native virtual invitation Chromium 1/1, 2.4s; Firefox/WebKit skipped. | Exact diff to current changes introductory CI/limitations comments only. Neither skip becomes a pass. |
| `a4d0a1ff8e84e1a04322c1b5648b64998723a241` / `58e5ef3c0b15a8ab2e76e423bb6c592982041dc6` | Password invitation three passes, 2.0/3.2/2.4s, 26.6s total. | Prior complete plan reports introductory-comment-only difference; no native-factor credit. |
| `7187286fabd5b521dbd664be1d0e2c79f9c76dc5` / `03124272a6cfe14ac48227c504cc896c2f11d9aa` | Shim invitation three passes, 1.6/2.1/1.8s, 23.1s total. | Prior complete plan reports introductory-comment-only difference; shim limits remain. |

The current complete root review explicitly corrects its earlier expectation
that A6 CI would run the newly added sign-in test. User supplies terminal metadata
success for all three jobs of `37104118354` at
`a6d361600a03713fc1b687f367e9db84efe43463`; this audit did not query jobs or read
their logs. Its workflow omits `signin.spec.js` from the setup/eight-file commands.
That success supplies no result for the new empty-credentials regression.

Current `.github/workflows/ci.yml:187–190` adds only the exact Chromium/one-worker/
zero-retry empty-credentials selector after the unchanged eight-file command.
Removing that one literal 245-byte invocation restores the entire A6 workflow:
11879 bytes, SHA-256
`fc5245ef15f0ac8d0a460b71ded533ad96ad3dcbd795b31f38b3e84221940b5a`.
User states current `37106315679` at fixed `1a517a1` is **RUNNING**. No actual
pass/failure for the selected test was supplied; none is inferred or awaited here.
The broader sign-in/accessibility files are not thereby selected wholesale.

The source-reviewed missing-username/required-password fix at `6800973` keeps the
same predicate, focused error, return before POST and clearing guards. It adds
invalid/error associations for only missing credential fields and clears them on
input/reset while preserving the OTP hint. Its focused test covers both missing,
each singly missing, input clearing, focus, 320px/axe/no POST and portal 401. It
uses optional OTP and explicitly expects its hint-only state. Required empty OTP
is a different branch and is not covered by that newly selected case.

Historical cancelled accessibility work, previous failed/held journey attempts,
CDP skips and source-only results remain as dated in the untouched reports.
An older cancellation neither proves the new source broken nor authorizes new
accessibility execution in this reservation.

### D01 separation

The complete fixed public receipt
`docs/roadmap/evidence/wave30-d01-timeout-memory-94-root-review.json` was read:
3842 bytes, SHA-256
`5b1784564fe5d53968529df3959e906c02ed523bf3c739257cdfb694daa515e6`.
It records 94/94 synthetic memory cases, 871 assertions, child/outer exit 0 and
reaped groups, with `journey_credit: false` and `whole60_proven: false`. It states
the absence of browser/cryptographic/provider/HTTP/helper-main execution.
The actual D01 confidential journey remains unpassed with the supplied pre-START
seed refusal. A few matching passages in the large D01 report were searched;
its entire body was not reread. No raw capture or other worker was accessed.
Memory PASS and the local RP stub are not confidential application access credit.

## One smallest reachable local follow-up, not reserved by this report

**Witness at fixed `1a517a1`:** `src/portal/signin.html:32,36–39` defines a focused
alert, a `novalidate` form and labelled OTP with
`aria-describedby="signin-otp-hint"`. `signin.js:184–190` makes OTP required for
the MFA application or configured TOTP stage. With username and required password
supplied but normalized OTP empty, `signin.js:322` calls `clearError`, focuses
the exact required-code message, and returns before `act`/POST. `clearError:65–71`
removes every field's invalid flag, and only username/password descriptions.
The branch never sets OTP invalid or adds the error description. Its field
therefore retains only the hint when the user returns to it. Input handling at
331–334 preserves OTP's existing hint and has no association reset to perform.

This is directly reachable in the existing test
`an MFA-only application refuses a password-only sign-in with the right message`
at `tools/browser/signin.spec.js:254–281`: it requires OTP, submits without code,
asserts the message and zero password requests, then exercises a server refusal.
Its local empty-code segment does **not** assert OTP invalid/error description.
The alert is still focused and has `role="alert"`; this audit does not claim a
WCAG violation, spoken-reader failure or security regression from source alone.
The omission is the same bounded field-feedback mechanic already implemented for
missing credentials/workspace/administrator confirmation.

Prospective ownership is only two existing paths, with no fixture/helper changes:

1. `src/portal/signin.js`: three narrow attribute hunks. In the existing required-
   empty-code branch set OTP `aria-invalid="true"` and description
   `signin-otp-hint signin-error`; on error clear and OTP input restore precisely
   `signin-otp-hint` while clearing invalid state. Keep username/password cleanup,
   exact required predicate, normalized-code parser, both existing messages,
   alert/focus/early return, acting guard, secret clearing and all POST/passkey/
   consent/logout/native-cancellation paths unchanged. No HTML/CSS change needed.
2. `tools/browser/signin.spec.js`: one additive focused case,
   `empty required interaction code identifies its field without a request`,
   using existing `fixture.clients.mfa`, keyboard helpers and a 320×640 CSS viewport.
   Supply nonempty username/password locally, leave required OTP empty, submit
   with Enter: exact alert/focus, OTP-only invalid state and hint+error association,
   no password POST, same unauthenticated placeholder and portal 401. Return by
   keyboard and type an unsubmitted local value: invalid state clears and hint-only
   description returns. Check field association/control fit/axe as available in
   the existing test helpers. Never submit a valid factor, grant application access,
   weaken MFA, change fixture setup or spend a recovery code to ease this refusal.

One **prospective**, not existing or executed, command after exact source/root
reservation and an exact-source embedded fixture/dependency preflight, from
`tools/browser`:

```sh
CARGO_TARGET_DIR="$PWD/../../target" ./node_modules/.bin/playwright test signin.spec.js --project=chromium --workers=1 --retries=0 --grep '^empty required interaction code identifies its field without a request$' --reporter=line
```

There is no permission to build or run that command here. Root must independently
review the exact new test/source, prerequisite fixture identity, private output
handling, finite outer bound and owned-child cleanup before a single release.
The existing 60s per-test and 120s startup policy are source bounds, not guaranteed
wall-clock cleanup. Any separately necessary fixture build needs the own private
target, jobs=1, incremental=0, dev/test debug=0 and fresh capacity above the agreed
8GiB floor; no cache deletion, build estimate or runtime quota is claimed here.
Do not enlarge this slice into every form, engine, protocol or authenticator.

## Original disposition and unmeasured inputs

Do not recommend original U10 DONE from the current evidence. Keep the original
gate: a complete ordinary-user browser journey, assessed with keyboard, assistive
technology, small-screen behavior and the authenticator profile actually selected
for supported use. The source follow-up above is the one immediate local seam;
it is not a replacement for the missing journey observation.

Future original-scope evidence needs a named reader/OS/browser and usable supported
authenticator input, an eligible invited account and mail path, a functioning
application return/access path, and factor/session/recovery circumstances with
their printed security boundaries intact. Record actual announcements, keyboard
focus/actions, viewport/device mechanics and refusal/replay effects throughout the
same journey. Do not label a CDP/shim as a physical device, doubled fonts as native
zoom, a source/axe result as reader execution, or terminal approval as ordinary
browser completion. An eligible email-reset account cannot silently stand in for
lost-all-factors recovery. A nontechnical-user outcome is unobserved here; no
recruited human, physical device or real assistive input is invented.

This does not invent a mandatory every-host/every-device/full-suite certification
campaign or a new unrelated human study. Root can choose one bounded representative
original journey and explicit remaining profile limits. Current D01 seed/refusal
diagnosis is separately owned, so this audit proposes no duplicate application
implementation or runtime. U01–U09 and other closed rows remain closed.

## Actual checks and handoff limits

Actual work: read applicable `CONTRIBUTING.md` and check for applicable `AGENTS.md`
(none found); clean own branch/HEAD inspection; immutable `git show` complete-body
reads and bounded supplementary line searches; data-only JSON parsing; SHA-256 and
whole-byte comparisons; exact historical diffs; whole-workflow invocation inverse;
report path/reference, UTF-8/Markdown fence/hygiene, whitespace and sole-file diff
checks. The initial guessed read paths `tools/browser/account.spec.js` and
`.github/workflows/check.yml` were absent; actual indexed files/`ci.yml` were then
used. These read-only misses are not missing product capabilities.
The first report-only checker used an incorrect heading-count expectation and
refused; checking the six exact level-two headings corrected that audit check.
The report bytes were unchanged for that correction, and the repeated static
check passed. It was not a source test or runtime failure.

No Cargo, compiler, Node/browser parser, source function/import, fixture, test,
benchmark, product binary, helper, provider, service, network/job query/download,
desktop or assistive technology ran. No original/current CI result was regenerated.
Only the new report is staged/committed; production, manifests, prior reports and
all failed/skipped receipts remain byte-preserved. Source readiness/field semantics
are inspection findings. The root owns future file/runtime reservations, published
integration and original task disposition; accepted receipt-secret/header/PAM,
Group and 60s boundaries are unchanged.
