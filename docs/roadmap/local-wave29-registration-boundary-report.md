# Wave29 A03 registration runtime boundary

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; existing A03 task
`4467345d-7a4f-4bb9-bd5a-586c775b0b44`, worktree
`42bb51c6-c198-4adb-bd92-0a5222853231`, branch
`roadmap/local-module-boundaries-wave27`. Root approved this one bounded
registration move. A03 remains **in_progress**; root owns review, board changes,
integration and push.

## Baseline and history

Published main `3a46e983ae24530661c42f73e8d6e739574448f4` entered this branch
through clean history-preserving merge
`a214351355e8b99e33d89f2b35147dc67190c728`. The prior acceptance audit
`a89b69b18c4a4edbdd7b890569b421b62a013d4d` and all previous lane history remain.
There were no conflicts or resets. Immediately after alignment, every product
file equaled the published main pin; only the retained audit report differed.

Implementation commit `8ef7efc01ce4f4e3057c12bca7b96b35e2a2c1bc` changes exactly
four claimed files (145 additions, 134 removals). This report is a separate
commit. The original task wording/status was read through RiWork with the
explicit project ID. Existing `RIWORK_HOME` was retained; no environment-home
or model configuration was changed.

## Exact implementation scope

| File | Change |
| --- | --- |
| [registration.rs](../../src/registration.rs) | Retain the canonical registration records and InitialAccess view; replace runtime imports/bodies with the existing crate-private authority path's compatibility re-export |
| [assembly/registration_runtime.rs](../../src/assembly/registration_runtime.rs) | Complete four Core methods, whole RegistrationAuthority/private field/five methods and check_creator moved intact |
| [assembly.rs](../../src/assembly.rs) | Two unconditional lines: private module declaration and crate-private authority re-export |
| [check-module-boundaries.py](../../scripts/check-module-boundaries.py) | One five-line registration guard, independent of the existing management graph classification |

No management, API, Core, source, workflow, connector, storage, other production,
test or manifest file was edited by this slice. Changes inherited from the main
merge are accepted equivalents, not this implementation's edits. W02's workflow,
password/TOTP, approval work and the shared activation/source-factory hooks are
untouched. No broader ownership or private-field seam was needed.

## Source, record and wiring equivalence

The accepted original `src/registration.rs` SHA-256 is
`7790397e048a71025fa8f9867c49f514ee5825c344e61b3255954e298b6352f1`.
Baseline copies and the exact comparison script/output are preserved in ignored
own-worktree `target/wave29-registration/evidence/`.

The comparison locates braces using masked comments/literals, then compares
the original, unnormalized signature/body bytes. All ten runtime functions
match exactly:

- Core: `registration_template`, `registration_templates`,
  `revoke_registration`, `dynamic_register`.
- Authority: `for_token`, `template`, `id`, `require_available`, `consume`.
- Private helper: `check_creator`.

The complete authority type/documentation/private field/impl, complete Core
impl and creator-helper tail also match their original chunks byte for byte.
The entire new runtime file reconstructs exactly from its assembly imports/
header plus those two original chunks. This checks the complete helper set,
including constructors, literal registry keys, clock/digest operations, caller
arguments and transaction order rather than just ten isolated counters.

The canonical `RegistrationTemplate`, `InitialAccess` and its `view`, and
`RegistrationRequest` spans match exactly, including derives, field order,
Serde defaults/unknown-field handling and schema attributes. Only separators
outside those spans are excluded from the record comparison. No persisted
record or API input shape changed. Protocol reconstruction proves that its
only changes are removal of the complete runtime set, adjusted imports and the
crate-private compatibility alias; its only remaining function is `view`.

`RegistrationAuthority` remains `pub(crate)` and its `record` field remains
private. Its five method signatures/visibility remain unchanged. The assembly
module is private and its re-export is crate-private; the original
`crate::registration::RegistrationAuthority` path continues to name the one
implementation. `InitialAccess` fields already had the visibility required by
that runtime. No visibility was widened and no second authority/record type was
introduced. Registration remains available through the same unconditional
module wiring in both editions, with the same four public Core signatures.

Removing only the two inserted assembly lines reproduces the accepted
`src/assembly.rs` exactly. Removing only the new guard reproduces the accepted
checker exactly: no old policy expression, reader, counter or classification
was changed. Scope checks also compare the unchanged management/API/Core/source/
workflow files against the published pin. All other existing production files
are excluded from the four-file delta.

## Preserved security and transaction contracts

`for_token` still hashes the presented token, resolves the same registry key,
checks template/token/enabled/expiry, then checks current creator authority.
Agent creators still need live parent/agent authority and both existing client
and registration permissions; human creators still need an enabled
administrator record. No registration token gains management `client.write`.

The unchanged `management::register_client` still obtains that live authority
before receipt replay. An exact completed request may replay after final use;
a new request must pass `require_available`. Request/template validation still
precedes `consume`, which rechecks the limit and increments the same record in
the caller's Tx. Consumption, the existing shared client writer, audit and
receipt save remain in that same write transaction. Their caller bodies and
arguments are unchanged; no nested write or authorization cache was introduced.

Template issuance still calls
`management::create_registration_template_issuing` through the same explicit
write and first-only credential envelope, keeping the generated initial token
out of retry receipts. Listing retains its live principal/read filter;
revocation retains the existing mutation envelope and shared writer; dynamic
registration retains the same explicit write and management writer. Client
credential disclosure/receipt handling and edition validation remain with
their existing management owner. This pure move changes no authorization,
idempotency, revision, ownership, audit, secret protection or runtime behavior.

## Focused checks actually performed

| Check | Actual result |
| --- | --- |
| `python3 target/wave29-registration/evidence/check-equivalence.py` | Passed: all ten signatures/bodies, whole runtime chunks, canonical record/view spans, private/cfg/compatibility wiring, unchanged callers, four-file scope and retained history; output `source-equivalence.txt` |
| `python3 scripts/check-module-boundaries.py --json target-wave29-registration/evidence/graph-before.json` | Passed on the aligned accepted baseline; `graph-before.json` and `boundary-before.log` preserved at the final evidence location |
| `python3 scripts/check-module-boundaries.py --json target/wave29-registration/evidence/graph-after.json` | Passed after the move; `boundary-after.log` |
| `python3 target/wave29-registration/evidence/check-registration-guard.py` | Passed: clean and comment/ordinary/raw-string cases accepted; twelve negatives rejected by the new guard; `registration-guard-check.txt` |
| `rustfmt --edition 2024 --config skip_children=true --check src/registration.rs src/assembly/registration_runtime.rs src/assembly.rs` | Passed; excluded child files were not formatted |
| `git diff --check`, implementation `git diff --cached --check` and staged scope | Passed; exactly the four claimed implementation files |
| `python3 target/wave29-registration/evidence/check-report-links.py` and report `git diff --cached --check` | Passed: five local Markdown links; only this separate report is staged for the documentation commit |

The initial before-scan evidence tree was moved without loss from ignored
`target-wave29-registration/evidence/` to the repository's standard own-worktree
`target/wave29-registration/evidence/`; old evidence/backups were preserved.

Guard negatives cover the original mixed runtime, qualified/grouped aliased
Core and Tx imports, PostgreSQL references, Core/Tx compatibility facades,
unqualified Core/Tx usage, and `.store`/`.config` runtime access. They run only
the source checker against an isolated disposable copy under this evidence
directory; no production file is injected or executed. The checker continues
to mask comments/literals, so textual mentions do not create false failures.

No Cargo compile was needed: the complete private helper/type set moves together,
its canonical inputs are already accessible at their unchanged visibility, and
the crate-private type/re-export chain is explicit. The checks above prove
syntax/source/scope/wiring equivalence, not a new Rust compile or runtime run.
No runtime tests, broad tests/lints, benchmarks, cloud mutation, external service,
desktop/accessibility interaction, new task/worktree/worker/managed shell,
accepted/main edit, push or task-status mutation occurred. No Cargo target or
build process was used.

## Honest graph and original acceptance residuals

Counts are distinct Rust source files explicitly naming crate-root modules,
including grouped imports, tests and edition-excluded code. Parent imports,
macros, trait dispatch and runtime calls are outside this inventory. All
existing classifications and boundary counters are unchanged; registration
is still classified management, not protocol.

| Explicit source edge | Accepted base | Registration cut |
| --- | ---: | ---: |
| Protocol → Core | 0 | 0 |
| Protocol → storage | 1 | 1 |
| Management → Core | 13 | 12 |
| Management → storage | 12 | 11 |
| Management → assembly | 1 | 2 |
| Assembly → Core | 51 | 52 |
| Assembly → storage | 45 | 46 |
| Assembly → management | 22 | 23 |
| Rust files scanned | 264 | 265 |

Only registration's direct Core/storage edges disappear; the new runtime adds
the corresponding assembly edges and the protocol compatibility alias adds
one management→assembly edge. The sole protocol→storage reference remains the
existing cloud cleanup unit test. Neither that edge nor the unchanged zero
protocol→Core count establishes full architectural closure.

The original A03 outcome is: “Separate identity core, management, storage,
protocol adapters, connectors, API types, server assembly, and client
responsibilities.” Its workstream goal avoids a second identity implementation;
its distribution gate preserves users and authorization and refuses silently
ignored configuration, with shared revocation/credential-protection semantics.

The retained [original-acceptance audit](local-wave29-a03-closure-audit.md)
inventories all eight responsibilities and credits native A08 plain/encrypted
redb/PostgreSQL transition evidence at its historical product/validator pins.
This slice closes the concrete registration runtime mixture identified there;
it does not invalidate or rerun that evidence. It supplies no new current-main
cross-build/security-parity or official released-artifact result.

Management's Core/Tx helper dependencies, connector/provisioning/controller
runtime composition, API data/transport ownership and Core startup/listener/
background assembly still need root's disposition against the full outcome.
Some are intentional service composition; graph counts alone do not settle
them. Official release provenance, exact architecture assets, installed
previous-version rollback and deployment gates remain distinct from these
local boundaries. No other implementation slice is proposed or claimed here.
Recommend accepting this bounded move after review while keeping A03
`4467345d-7a4f-4bb9-bd5a-586c775b0b44` **in_progress**. Root owns closure and
integration; this report does not mark the task done.
