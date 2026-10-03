# Q06 original browser and authenticator scope — source audit and next prerequisite

2026-10-03. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, original Q06
`393d3429-f554-497c-a543-f07727d9d039`. Reservation
`wave30_Q06_browser_authenticator_original_scope_source_audit` owns only this
new report. Existing WT `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`, shell
`2173637e-bdb3-4eba-a128-178f57b41a34`, entry
`7f04a72f750c7c011f68dee3369ab3c14e9c7e91`, clean tracked/index/untracked.
All current source claims here are pinned to published
`544d1340b80cd3e040dc13142cdcbc1d75fea4cb`; no alignment occurred.

The smallest remaining proposed slice is **one real native passkey sign-in
prompt dismissal followed by a fresh-gesture retry on one root-selected
browser/OS/authenticator profile**. This checks an explicit documented behavior
that simulated credentials cannot establish. It requires the pending root-owned
device/release inputs and a separately authorized private fixture. No certain new
product defect was established, so no product/test/guide hunk is proposed here.
This proposal is not a complete Q06 or U10 journey, runtime release or device pass.

Only this report is written. No source, existing report, test, guide, workflow,
helper, task, assignment, status, main or accepted tree changes. No browser,
Driver, native authenticator, service, HTTP, provider, Node case/VM, Cargo,
compiler, setup/download, remote query or dispatch ran. A09 ARM37101183416 owns
validation/Cargo according to the assignment; its state was not queried and no
slot was acquired/released. Root owns all integration/publication/disposition.

## Complete original row and state provenance

Read the full exact-UUID row from the existing local project export
`planning/current-tasks.json` under the project's RiWork orchestrator directory.
This is a dated local export, not a fresh project query: 244354 bytes, mode 0644,
mtime `2026-10-03T05:51:05.061711Z`, SHA-256
`0b38ca3eb8810a4628436eb5083f6329de848f3f4ddaa972c850601ad45c0835`.
Exactly one row matched. Its details SHA-256 is
`f1a59b7ec491dce241c6dbcca755dade8c22cb871f2a223658433fd936f5ab3b`;
whole row, sorted compact UTF-8 JSON canonicalization, SHA-256
`3daa222ab92eba19d3a986ef694f102b71ab308a77ed91f04050ec224f6a994a`.

Title: **[P1] Q06 — Test complete browser and authenticator journeys**.
Exact acceptance: **Include supported mobile/desktop environments, accessibility,
and physical hardware where promised.** Prerequisites U10, Q02; workstream
Security, testing, benchmarks, and releases. Exact gate: **Claims about security,
compatibility, speed, and recovery each have the appropriate evidence. Passing
one category does not establish the others.** The details require implementation,
tests, documentation and released-artifact review as applicable; reports or
documentation alone do not establish completion.

The exported row remains `todo`, original primary WT
`a11406f5-8478-4deb-9cf2-9a6148bc6c96`, created 1790534567, updated 1790894718.
The older scheduling text grants no runtime. This current reservation authorizes
this supporting static audit only, without changing the original primary or row.
Both editions retain shared identity, authorization, revocation and credential
protection. The optional riauthctl terminal USB client is separated from server
builds. The entire original product-boundary/scheduling/Driver text was read.

Also read the complete Q06 entry in pinned
[closure audit](local-wave28-task-closure-audit.json), source lines 2995–3068.
That older entry says `in_progress`, recommends `todo`, and expressly records
unmet physical/mobile/assistive evidence and canceled accessibility testing.
Those are dated observations; this report does not reconcile or mutate status,
restart accessibility testing or duplicate the pending user input questions.

## Verified reuse and source/body coverage

Before reuse, independently checked every one of the 45 immutable path/byte/LF/
SHA-256 identities in the complete e62a U10 source-audit report against fixed
544d. All 45 matched. Thus its fully read browser account/setup/sign-in/device/
security assets, 13 journey spec bodies, shared fixture/config and selected
Rust delivery/guard witnesses can be reused at the same object bytes.
[U10 audit and source-only receipt](local-wave30-u10-accessible-journey-plan.md)
keeps the full coverage distinction. This is not reuse of old screenshots or
unexecuted declarations as current runtime evidence.

New full reads: exact Q06 export/audit rows; 226-line Public CI and 186-line draft
release workflows; complete limitations and passkeys guides; 79-line four-case
passkey-flow unit source; complete 260-line passkey policy/support module; the
417-line b71 terminal CI receipt; both relevant CI diagnosis reports. Re-read
selected complete native delivery/sign-in/management functions and portal
validation documentation at the fixed object. Testing/bootstrap/release-note
claims were read in relevant bounded spans. Broad initial combined displays were
truncated; full workflows/diagnosis bodies and relevant omitted claim spans were
followed up, while other already-read U10 bodies use the checked identity reuse.
No unrelated compiler log, private credential, protocol capture, screenshot or
private native fixture was inspected.

Concrete preserved interfaces:

- `auth.js:103–121` calls actual `navigator.credentials.get/create` with the
  server options and AbortSignal; `:126–169` preserves unused options on native
  NotAllowedError, cancels explicit pending ceremonies, rejects stale generations
  and refuses cancellation after finish submission. `:171` shows availability
  only with PublicKeyCredential and secure context. A platform/browser test is
  needed to establish its real gesture/prompt behavior.
- `app.js:369–432` retains focused canceled/timed-out feedback, fresh button
  activation, no credential POST retry, explicit cancel and account-generation
  checks. `:499–608` retains bound management, recent own-browser MFA, visible
  names and explicit rename/remove decisions. Labels/alerts/keyboard controls
  exist in the pinned HTML. Source presence is not a spoken announcement.
- The reused portal/interaction HTTP bodies preserve exact Origin,
  X-Riauth-Portal and same-origin Fetch Metadata guards, browser binding, CSP,
  frame denial and credential dispatch. `passkey.rs:125–137` derives RP ID from
  the issuer hostname and exact origin and refuses an IP-address issuer;
  `:204–209` retains fresh-factor policy. None is bypassed by this proposal.
- Invitation completion requires a later sign-in and creates no session;
  reset links, credential epochs, recovery-code replay and other-browser live
  authority are materially asserted by the existing fixtures. Loopback mail and
  software TOTP still do not establish external delivery or a phone app.

## What is claimed, configured and actually executed

[Portal validation](../PORTAL.md#validation) and [passkeys](../passkeys.md)
make the boundary unusually explicit. They do not name a certified universal
OS/browser/device matrix. They offer native WebAuthn where the browser has it in
a secure context and require testing the deployment's browsers/devices, expressly
including Safari's dismissal/retry. Particular browser/device compatibility still
needs that environment under [release limitations](../limitations.md).

| Environment or claim | Source / actual evidence | Precise limit |
| --- | --- | --- |
| Desktop browser engines | Playwright 1.63.0 and axe 4.13.0; Desktop Chrome, Firefox and Desktop Safari profiles; one worker/no retries. Actual Linux CI rows below reached Chromium/Firefox/WebKit test bodies. | WebKit's Desktop Safari profile is not Safari.app on macOS/iOS. Headless Linux is not native Windows/macOS browser, a human desktop journey or hardware evidence. |
| Narrow screens | 390×844 CSS phone-size views in eight Q06 bodies; U10 defines 320/768/1440 and 200% DOM text cases. Source includes reflow, labels and focus management. | CSS dimensions/font changes do not run Android/iOS, a virtual keyboard, OS zoom or touch. No particular phone/browser version is certified. |
| Accessibility | Existing eight U10 spec definitions check axe/focus/keyboard/selected layout; historical focused workspace/admin 3/3 claims stay at 5892563/099c190. | The Q06 CI allowlist does not include accessibility-journeys.spec.js. Axe and focused virtual keyboard assertions are not assistive-reader/user completion. Accessibility remains canceled/unproved, not restarted. |
| Browser-native passkeys | Actual get/create delivery, required UV policy, usernameless/pinned flows, cancellation/management source and software cryptographic tests. | Shims replace navigator.credentials and set UP/UV; CDP uses a virtual internal authenticator. Neither proves physical presence, real UV refusal, synced-device independence or phone hybrid. |
| Physical native historical observation | D05's U01 record at base 14de533, dated 2026-09-28, reports a Cua.ai Driver session reaching a backup Touch ID prompt. | Backup completion was expressly unclaimed. This audit did not inspect the private fixture/images; it is a bounded accepted report claim, not completed native enrollment or a Q06 hardware pass. |
| Terminal authenticator | Optional riauthctl terminal-usb requires supported CTAP2 USB PIN/touch; server USB dependency boundary is separate. | Platform keychains/Bluetooth/hybrid-phone CLI transports are expressly not implemented. A USB compile/dependency test does not exercise a physical key, and browser claims do not extend these CLI transports. |
| Release environments | Draft release config builds native Linux x86-64/ARM64 server/client/maintenance products, both editions; tag/current-main validation, hashes and artifact smoke precede a maintainer draft. | Package architecture is not browser/device compatibility. This source read neither proves a release ran nor invents a current official asset. Pending root release/device answers remain necessary; active A09 results are not borrowed. |

Public CI uses Ubuntu 24.04, Rust 1.98.1, pinned checkout/toolchain actions,
contents:read, normal Platform default features for the browser fixture. It
installs locked Playwright browsers once for setup, builds portal_fixture, then
runs exactly the eight Q06 files with one worker, zero retries and 25-minute step
bound. Ordinary interaction tests have 60s limits; startup defaults to 120s plus
5s setup allowance. The existing fixture stop awaits its owned child without an
outer deadline: a later local runtime needs separately reviewed finite cleanup.
Source configuration is not hard resource/output/cleanup proof.

Firefox's pinned test project disables COOP context swaps. Server headers are
unchanged and Rust header assertions remain; Chromium/WebKit retain their normal
settings. Successful Firefox rows therefore exclude default COOP isolation
coverage. The dated removal condition requires an actual verified tooling fix;
a new version number or old upstream issue label is not sufficient.

## Independently inspected historical actual Linux browser records

Read/hash the already-retained public integration log, with no download/query:
`/tmp/riauth-wave29-integration-110661000640.log`, 265770 bytes, mode 0644,
SHA-256 `458a30895b5987c0bebe2fb7368a1a32969fa0d41ce90acbfa56f43299ccd186`.
Its attribution is the accepted D05 E-CI record: run `36950097067`, job
`110661000640`, checkout `2d05c00deed3a6dafcd458a5f1a1a9beb17d0dd8`.
Only fixed browser installation/runner metadata and public test status/count
lines were exposed; no raw fixture/protocol/private payload was printed.

Observed runner image Ubuntu 24.04 `20260927.320.1`, provisioner `20260901.588`,
Rust `1.98.1 (48a229cea 2026-09-01)` x86_64 Linux. Installation lines identify
Chrome for Testing and Headless Shell `153.0.8010.12` / revision1243,
Firefox `155.0` / revision1543 and WebKit `26.6` / revision2359. These are that
run's versions, not current installed host measurements or browser binary hashes.

A static status parser recomputed exactly 24 Q06 records, 22 passed and two
skipped, plus nine setup passes. Exact source titles were read from the complete
spec bodies and parsed as DATA. This is actual reached test-body evidence at the
old run, not this audit executing the cases. Conditional/internal branches are
not automatically credited just because a test passes.

| Q06 body / fixed test line | Chromium | Firefox | WebKit | Material assertion and authenticating mechanism |
| --- | --- | --- | --- | --- |
| authenticator-recovery:178 | pass | pass | pass | Enrollment error focus, epoch/session revocation, one-use recovery and 10→9→8 counts; page key and software-computed TOTP. |
| invitation-passkey-shim:193 | pass | pass | pass | Expiry/blank-name/replay refusal, completion without session, later discoverable sign-in/out; P-256 shim supplies UP/UV. |
| invitation-passkey:202 | pass | skip | skip | First credential, expiry/replay and no-session completion plus later sign-in; Chromium CDP virtual authenticator only. |
| invitation-password:219 | pass | pass | pass | Keyboard password acceptance, expiry/replay/no-session/later sign-in/out and zero passkey POSTs; no authenticator installed. |
| multi-authenticator:313 | pass | pass | pass | Two distinct shim credentials, removed first refused, second/TOTP survive, both sessions lose authority; two simulated stores. |
| passkey-rename:228 | pass | pass | pass | Keyboard blank/space/name validation, trimmed name and credential/session preservation; one shim. |
| passkey-revocation:61 | pass | pass | pass | Enrollment/removal invalidate other live session, removed key refused, password fallback succeeds; one shim. |
| password-reset-replay:202 | pass | pass | pass | Nonenumeration, errors, one-use reset and old-password refusal/epoch revocation; loopback mail, account has no factor. |

Raw witnesses: lines2795–2807 contain nine setup passes / 59.1s; lines2835–2863
contain Q06's 22 passes/two skips / 6.5m. Firefox skip line2847 and WebKit skip
line2855 are precisely invitation-passkey. Other files' WebAuthn/headless skip
guards are source definitions; no extra skip was observed in these records.
Timing is whole fixture-test duration, not native prompt latency or a benchmark.

Independent complete-byte comparisons found all eight Q06 specs, setup.spec.js,
fixture.js, config, package/lock, auth.js, account.js, app.js, baseline signin.js,
portal HTTP and interaction HTTP identical between that actual checkout and fixed
544d (19 paths). Both CI/release workflows are also identical. This links actual
old reached bodies to the fixed definitions without claiming the entire current
product, core, binary or release was executed. Binary hashes/native private
credential records are not supplied by these log lines.

Also read the full accepted b71 terminal receipt at
[evidence/wave30-ci-b71-terminal-root-review.json](evidence/wave30-ci-b71-terminal-root-review.json):
run `37063876066`, source `b71b7b0041a549793233e8c7a81bbb61797e20f3`, integration
job111027210794 has setup and headless-journey steps marked SUCCESS. Its raw
inspection section is check-job-specific. No separate browser count, browser
binary pin or physical test is invented from those step statuses. This remains a
dated success at b71, not all-green evidence for 544d or our U10 correction.

## Accepted older results and failures stay dated

The original Q06 accepted ledger pins `03124272a6cfe14ac48227c504cc896c2f11d9aa`
and CI ports `4676f7419618d96389b9bdbbb3ccc00ccc74c837` /
`85dcd4a0c0e420871bf6b7f408263183dd255f77` were read against their author messages.
Source `7187286fabd5b521dbd664be1d0e2c79f9c76dc5` reports shim invitation 3/3.
Source `39c5b099271875b2d80f2216709f90dacde3f5f1` reports the local eight-file
allowlist 22 pass/two skips in 201s with revisions1243/1543/2359. Its source
fixture provenance is a private target plus mtime, not a cryptographic native
artifact/device attestation; the author expressly did not run GitHub Actions.
Later E-CI above is a separately observed hosted execution. D05's older
"source-only" classification means software/in-tree evidence; it must not erase
that those local browser fixtures actually reported passes.

Retain the historical Firefox failures: CI36937149832 recovery first navigation
and CI36936535723 setup navigation timed out. The accepted
[wave28 follow-up](local-wave28-ci-followup.md) records local default-pref hangs
and focused runs after the test-only preference, while its root integration note
keeps the first-navigation CI attribution inferred. 2beb587's original
version-only fix/removal claim was explicitly narrowed by accepted 5c1a5a7.
No present upstream/browser defect or shipped fix is inferred here.

Retain the separate Chrome/RP failure in
[wave29 diagnosis](local-wave29-browser-start-ci-diagnosis.md): run36954886983,
job110675857307, source93999d15f7681ff918f986a341c48b5556bee24f, timed out at20s
with Chrome alive and no pending authorization; stderr/stage timings were absent.
The next docs-only source passed. Actual cause remains unknown; successful later
runs do not retrospectively pass it. Its diagnostic proposal is historical,
unapplied by this report, and grants no raw URL/protocol logging authority.
The old CI56 CLI startup failure and D01 confidential-flow failures remain in
separate owned reports; neither is diagnosed, retried or overwritten here.

U10 code6800973/report7f04a72 are root-reviewed/integrated per this assignment,
with only static evidence and the new focused browser test held. Those later
candidate bytes differ from baseline544d signin.js/signin.spec.js. They gain no
runtime pass from the old body equality. D01/I02 and all closed rows remain
separate; this audit neither duplicates the confidential demo nor reopens them.

## One concrete next prerequisite — no source delta or runtime release

The next slice is exactly native portal sign-in dismissal/retry, not every
hardware vendor, browser, OS, tenant or recovery situation. Source witness:
`docs/passkeys.md:31` expressly requires testing canceled/timed-out prompt retry
on supported devices, including Safari; `auth.js:126–169` and
`app.js:418–432` are the unchanged implementation. Four passkey-flow unit cases
mock native creation and cancellation; the Q06 allowlist does not establish a
physical authentication prompt. The CDP cancellation U10 case is also virtual.
No new certain source bug was found and no change to those bodies is requested.

A future root reservation needs these exact inputs before executable design:

1. The answer to the pending device/release selection: one named browser with
   exact version, OS/version and real authenticator type/model/transport. If
   choosing Safari, it must be actual Safari.app or the named mobile Safari,
   not Playwright WebKit. A platform prompt/hardware-backed credential is labeled
   separately from an external USB key, synced passkey or hybrid phone. No device
   availability is asserted and no duplicate question is sent.
2. One root-reviewed product source/build or verified release selected by root,
   binary hash/edition/toolchain/features, configuration identity and privately
   prepared disposable password-backed ordinary account with a genuinely native
   enrolled discoverable UV credential plus retained fallback. No CDP/shim/VM
   injection. Missing enrollment/preparation is a prerequisite, not credited by
   the older Touch ID prompt. No official release ID is invented.
3. Exact issuer hostname/origin and secure-context delivery on that device.
   Same-host localhost evaluation may use the configured loopback hostname;
   a different physical device needs an actually reachable trusted HTTPS issuer,
   matching RP ID/origin and native TLS or the existing trusted proxy guard.
   Do not substitute an IP issuer, widen CORS/Origin, disable UV, relax freshness,
   or publish the private fixture. No transport/provider readiness is inferred.
4. Root separately reviews one finite launch/Driver budget, owned browser/profile/
   fixture cleanup, private bounded evidence, capacity monitoring/8GiB floor and
   permitted safe state observations. No peak allocation is measured here.
   Desktop uses only RiWork Cua.ai Driver after reading descriptions/current
   state, with snapshot-bound actions. No Playwright --headed conversion or
   concurrent shared-desktop allocation is proposed.

Planned action boundary: open the fixed portal signed out; activate its public
Sign in with a passkey button; dismiss the authentic native prompt before
verification; observe the existing focused canceled/timed-out feedback and
continued signed-out state; use one fresh activation to retry the same unexpired
flow and perform real verification; observe ordinary signed-in portal access;
then sign out and finish the separately approved owned cleanup. Prompt shown,
dismissal, real verification and final UI state must be recorded separately.
No enrollment/removal/backup/phone/assistive/full-invitation/recovery completion is
added to this small slice. Missed or unavailable native interaction remains a
failure/unreached result, never shim fallback or an automatically extended retry.

Public evidence would contain fixed phase/outcome booleans, exact source/build/
environment identity and cleanup verdicts. It must exclude passwords, PINs,
private subjects, credential IDs/assertions, session/cookie/token values, secret
URLs and protocol/biometric payloads. No hardware attestation or native UV claim
follows merely from a page result or prompt appearance. Native/physical evidence
requires the actual selected setup and its own bounded observation.

No source/test path ownership is requested beyond this report at this stage.
Once root has the pending inputs, it can reserve an exact source-first finite
native recipe/controller if one is necessary, then independently release it.
This report supplies neither a driver adapter nor a launched fixture. No new
browser command is substituted for the already-owned held U10 filter.

Q06 still has unrecorded real environment/physical/assistive claim evidence;
U10's independent nontechnical full browser journey remains open. No new
universal platform campaign, human recruitment claim or status decision is made.

## Immutable source identities and actual static limits

The table identifies compared bytes, not runtime coverage. All paths are at fixed
544d; the 19 actual-checkout equal paths are expressly listed above. Supporting
source reads are distinguished from full-body reuse and selected spans above.

| Path | Bytes / LF | SHA-256 |
| --- | --- | --- |
| `.github/workflows/ci.yml` | 11879 / 226 | `fc5245ef15f0ac8d0a460b71ded533ad96ad3dcbd795b31f38b3e84221940b5a` |
| `.github/workflows/release.yml` | 9864 / 186 | `da8607c5ea33a906185141e0caebec18c2282c22676981e0c659fd65eef4369f` |
| `tools/browser/passkey-revocation.spec.js` | 9049 / 169 | `b6061d4ac397eda9663d1dda69a3cef01f3280f9db1626da926c4e2ca0e345e6` |
| `tools/browser/authenticator-recovery.spec.js` | 16772 / 312 | `df954147a4f4d3505ca0957fa35253f21a83bf1bfe0cc3e25607eada716ff1d9` |
| `tools/browser/passkey-rename.spec.js` | 20303 / 394 | `57fc51a84f8b0fa6a48891c064fcb4ff9dd17d50798212cff4da1af802125db6` |
| `tools/browser/password-reset-replay.spec.js` | 18482 / 347 | `d9fb0048c935385c226900f690717e137d397824a282209c1199e5cabe2223cd` |
| `tools/browser/multi-authenticator.spec.js` | 31081 / 569 | `137953b2d18440fb41d3570ff9d6bc98bad03c8af2fc8ffc408ffd95a2fe5720` |
| `tools/browser/invitation-passkey.spec.js` | 19053 / 355 | `93b5c5c9d488858d9f76e80c3af14a9c042122c963b724dd91904d67c8721ca5` |
| `tools/browser/invitation-password.spec.js` | 16293 / 312 | `b6ef8ea308494760886cc13972865de5b91ca9a5d1de8eac5c44e6e5019ba216` |
| `tools/browser/invitation-passkey-shim.spec.js` | 17812 / 330 | `212b00ea4be9e85d05dc4d6c3c80b6559a6b0904323894cab3f20734d3c4be7c` |
| `tools/browser/setup.spec.js` | 11201 / 181 | `dd934c78a76b2f6d1ab7803df74eda4cb485ac87a5c1fdc5ca5ec8386023169f` |
| `tools/browser/accessibility-journeys.spec.js` | 14959 / 297 | `01b41cb562b8ca41bc16de1c59ef7d6f8f51c743e82d6b684bc5144e869c8dec` |
| `tools/browser/passkey-flow.test.js` | 4100 / 79 | `e5d4ac141f94d829d68ee5e853c8b01d412e49f9622b694f0b374d45253ecb78` |
| `tools/browser/fixture.js` | 4521 / 81 | `b886dac64616015a791f48e2784a6adf84733e7f2f5a1b2dabb2e68e3172bc8b` |
| `tools/browser/playwright.config.js` | 1330 / 19 | `24c63e0563b0f7e1333a7bea03445297f69c87519bc8af44c313dd84d880b5a1` |
| `tools/browser/package.json` | 210 / 7 | `105fbcbbde405cee41712021c586da014a41db28ba1c946820ffc6e7f4e6a78b` |
| `tools/browser/package-lock.json` | 2538 / 82 | `81db66e55e5570b4cadcbffa821a34820a0aedd14c6679d4e1c8ebb5c9e4f30e` |
| `src/portal/auth.js` | 9235 / 175 | `c1a1aa440ca0144183019959b94bf950095309cbc414ec7bcffefbf48cf63ae3` |
| `src/passkey.rs` | 9273 / 260 | `ff1b781e782eb171cde40e25507794eb851aa17b01ff41600a7d4f8102e5c4f4` |
| `docs/PORTAL.md` | 43482 / 371 | `386fa4ca53f4607133b3f30071b7e2714ff4504cb0879b514fe2b52d86ddc015` |
| `docs/passkeys.md` | 12903 / 99 | `4d96608a6c1121cee131bd74f27be78dd42ee7ecfb7e7a67330513e679806b96` |
| `docs/limitations.md` | 4116 / 18 | `5399dab49f0d533628b9b1318a2b0c1459c44f50ff505879cf7d903d89493011` |
| `docs/roadmap/local-wave28-task-closure-audit.json` | 219710 / 3734 | `051a159610f904a6e91645bf06a059d3db8bcaec85f95e0e6c0551b233fe058b` |
| `docs/roadmap/d05-acceptance-evidence.md` | 153623 / 1699 | `87dc6e145f8db722f3e4c2acfa7ddc027cba4c86516acfb2252c928a92ac52f6` |
| `docs/roadmap/evidence/wave30-ci-b71-terminal-root-review.json` | 15797 / 417 | `e5f732c902fd1324ce1718e4773ad8e754101e8cba1f45145d8b62fba56e4d7c` |

Actual static DATA parser: installed Node's bundled Acorn parsed 12 fixed JS
sources, including all eight Q06 specs, setup, accessibility, shared native
helper and config, exit0. It read source test names/skip guards without running
modules, functions, callbacks, VM, imports of reviewed code, browser or cases.
The separate read-only log/schema counter exit0 verified the exact eight-file
24/22/2 and setup9 counts. The 45-identity reuse and 19-body/two-workflow equality
checks exited0. No source restoration/AST inverse is required because no existing
source was edited. This static report does not certify all browsers, independent
conformance, security, recovery or speed. Historical failures/skips remain dated.

Report-present `python3 scripts/check-docs.py` exit0. Initial tracked-file
hygiene exit0 counted 1062 files; after staging this new report,
`python3 scripts/check-repo-hygiene.py` exit0 counted 1063, including it.
`git diff --cached --check` and the staged exact-scope/readback checks exited0;
scope is exactly one new mode100644 report, with no existing tracked change or
other index/untracked delta. No static check failure occurred. The final evidence
paragraph is checked before committing. Commit hooks/signing are disabled only
for that invocation, without a persistent Git setting change. No runtime credit,
slot acquisition/release or original Q06/U10 disposition follows.
