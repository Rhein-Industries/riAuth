# U10 missing-credential feedback: root source review

Root reviewed the complete three production hunks and additive focused test in authored `6800973556f45ed2136467bde1e7d74cd98aa633`, integrated as `1406de9e5349d3bb9ef06f55e638938129318a34`. Both base files equaled published 544d1340. [Exact identities and limits](evidence/wave30-u10-missing-credential-source-root-review.json) accompany the [append-only author evidence](local-wave30-u10-accessible-journey-plan.md#source-only-materialization--wave30_u10_missing_credential_field_association).

The existing missing-credential predicate still returns before a POST. Missing username/required password fields now reference the focused error and carry aria-invalid. Input or error clearing removes stale credential associations; the OTP hint is preserved. Error text, focus behavior and all other authentication/native/decision guards remain unchanged. Root read the relevant HTML, keyboard/axe helpers and complete focused test. Actual node syntax checks pass on both integrated files.

The focused test checks both missing fields, each singly missing field, input clearing, focus, small viewport, axe, no POST and unauthenticated portal status. It has not run. Its existing fixture requires a binary with the changed embedded portal, pinned browser dependencies, and finite owned-process supervision before any separately released execution. The ARM cohort currently owns validation. No physical authenticator, assistive reader, mobile host, recruited human, complete journey or original U10 completion is credited. Original task and primary assignment remain unchanged.

## Focused CI selection correction, 2026-10-03

Root inspected the complete published Public CI workflow and found that
`signin.spec.js` is excluded from its setup and eight-file headless commands.
The earlier root expectation that source a6d's CI would exercise the new missing-
credential test was incorrect. That source's still-running job supplies no
result for the new test. Static source review remains valid and unexecuted.

Only one focused Playwright invocation is added after the unchanged headless
allowlist in the existing25-minute step. It selects Chromium, one worker, zero
retries and the exact new test title. The step already installs pinned browser
dependencies and builds the current embedded portal fixture. Every pre-existing
workflow byte is restored by removing that invocation. No previous test is
removed, no timeout/action/permission/environment setting changes.

Ruby Psych parsed the resulting workflow, and all shell run blocks passed
`bash -n`; neither parser executed workflow code. Documentation, hygiene and
whitespace/scope checks are performed before commit. No local Cargo, browser,
dependency setup or test was invoked. The newly selected test remains UNRUN
until a later exact-source CI result or separately authorized fixture proves it.
This correction establishes test selection, not full U10/hardware/accessibility
or journey completion.

## Actual selector failure and full-title correction, 2026-10-03

Public CI run37106315679 at source1a517a1 failed integration job111159428293 after the existing setup completed9 passes and the authenticator allowlist completed22 passes with2 skips. The subsequent focused U10 invocation reported **No tests found**, so it supplies no execution or assertion result for the missing-credential case. Root retained the complete private log and checked the checkout, command and result boundaries. [Exact log/source identities and limits](evidence/wave30-u10-full-title-selector-root-review.json) accompany this observation.

My previous static selection check missed that Playwright applies grep to the full project/file/describe/test title string. The beginning anchor incorrectly required the bare test name at the beginning of that string. The [official grep documentation](https://playwright.dev/docs/api/class-testconfig#test-config-grep) describes this full-title matching behavior.

Source-only commit5ecf4710a403f7b59306b08ea9f67ae3f7475594 removes only that beginning anchor. The ending anchor, selected file, Chromium project, one worker, zero retries, all test assertions and every previous workflow byte remain unchanged. Reversing that one-character change reconstructs the full previous workflow. Ruby Psych parsed the corrected YAML; all27 shell run blocks passed bash syntax parsing; whitespace checks passed. These checks executed no workflow, Playwright or browser code.

The corrected selector remains unverified by runtime. The earlier a6 completed CI check is separate evidence and did not execute this focused U10 case. No local browser/Cargo invocation, original U10 completion, assistive-reader, hardware or complete user-journey result follows from this correction. The original task and primary assignment remain unchanged.

## Required-code feedback port and closed CI selection, 2026-10-03

Root read the complete independent U10 review, all three authored OTP hunks and the complete additive regression in25b4bdfe, plus its separate df845a80 evidence. The author's source base predates the accepted missing-credential feedback. Root therefore ported only the OTP reset/input/required-empty-code changes onto the current accepted source, retaining both existing credential associations and their entire regression. [Exact author/accepted mapping and inverse proofs](evidence/wave30-u10-required-otp-port-root-review.json) retain this distinction; no complete older worker source file was imported.

The required-empty-code predicate, exact messages, alert focus and return before POST remain unchanged. The OTP field gains invalid/error association for that refusal; error clear and OTP input restore its original hint. The additive keyboard/small-viewport/axe/no-POST/portal401 case checks clearing and hint restoration without submitting a valid factor or granting an application/session. Its3367-byte authored addition follows the entire unchanged current test file. Node syntax and exact whole-byte inverse checks passed; no JS or browser test ran.

A separate source-only CI hunk selects exactly the two focused feedback titles through a closed suffix alternation, preserving file, Chromium, one worker, zero retries and all other workflow bytes. YAML parsing and27 shell syntax blocks passed. This selection and the new required-code case remain UNRUN. The earlier No-tests-found failure remains failed; no original U10, real assistive-reader, physical authenticator or complete user journey result is inferred. Original status/primary remain unchanged.
