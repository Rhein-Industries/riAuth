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
