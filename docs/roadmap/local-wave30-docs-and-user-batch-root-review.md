# Wave30 incident, recipe and independent user evidence — root batch review

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
This documentation-only batch follows published main
`88790deb62d32c84fa17dceb12cd93a727224e94` / accepted
`c0fd403d25ded476857a4a24f428f6c44775fd51`.
No production, existing test, script, client, manifest, dependency or CI workflow
file changes are included.

## Concrete accepted corrections and observations

- [D04 review](local-wave30-d04-root-review.md): offline security-agreement
  upgrade before initial service start; shared PostgreSQL forward-auth counters;
  current CLI retryable exit class without assuming rollback; authenticator
  prerequisite for the relief administrator's printed sign-in command.
  Only those source-backed paragraphs change.
- [D03 recipe audit](local-wave30-d03-tested-recipe-plan.md): the one forward-auth
  rate paragraph now describes both backends, effective agreed rates, both
  offline confirmations and reviewed missing-row-only adoption. Root reversed
  just line 164 and reconstructed the entire baseline recipe byte-for-byte;
  applied SHA-256 is
  `08429f7380a397220563a06754126c62dce084078e42e117df7fe0740733e6c5`.
  Root read the complete audit and separate guide/report deltas, preserving its
  historical commands, failures and exact raw-log provenance. The two historical
  raw logs were rehashed; they are not newly executed recipes.
- [D01 independent browser](local-wave30-d01-user-browser-review.md) and
  [root user review](local-wave30-d01-root-user-review.md): accepted bounded
  password sign-in/navigation/logout/relogin with separate c01-binary,
  published-guide and Driver provenance. No physical factor or application
  browser journey is inferred.
- [Completed integration job](local-wave30-ci-88790de-integration-review.md):
  published-source selected PostgreSQL, LDAP, browser and proxy fixtures passed;
  portal journeys retain two skips. The separate check job was still running
  at the recorded observation; whole CI is not claimed green.
- [I10 audit](local-wave30-i10-operational-interface-plan.md): confirmed one
  configured outbound SCIM pre-delivery connection-check gap. A bounded
  Core/API implementation is separately reserved; no product code or test
  result for it appears in this batch.
- [D04 checkpoint plan](local-wave30-d04-lockout-walkthrough.md): one exact
  factor-free second-administrator incident is released separately. This plan
  is not an execution pass. No all-eight-incident gate is closed here.

## Original gate handling

D01, D03, D04 and D05 remain in progress while their documented task coverage
is adjudicated using actual current and accepted historical results. This batch
adds no recruited-human study, every-host rerun or universal full-feature gate.
It also does not substitute one password form for every advertised workflow.
I10 remains in progress pending its scoped implementation and original inventory
review. A09's existing task is now assigned for a precise artifact-production
proposal; no new artifact, official release or dispatch has been performed.
O07 retains its observed ownership-postcondition blocker; no host or permission
bypass was introduced. R05, O06, S04, O03 and prior accepted closures stay closed.

Root checks passed: precise diff/source review, original recipe reconstruction,
artifact and raw-log hash comparisons, documentation links/build layout and
aggregate Git whitespace. No root Cargo, build, service, container, browser or
incident runtime was executed. Worker reports remain observations at their
explicit scope, not independent root replays. Root owns final publication and
status decisions.
