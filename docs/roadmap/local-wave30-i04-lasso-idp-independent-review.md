# Wave 30 I04 independent Lasso IdP source review

Date: 2026-10-02. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`;
existing WT `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`.
Reservation: `wave30_I04_lasso_idp_independent_source_review`.
This new report is the only owned change. Review starts from existing branch
HEAD `1c93a3ee564c69cc1cf8bd6e7eb9f1ecb4331250`; no alignment or source port.

## Finding and release boundary

No certain source-derived blocker was found in the reviewed additive IdP mode
and final new test. The selected Lasso source supports FORCE verification of
the original signed Redirect query, persistent native federation, both native
signatures, and assignment of the public assertion SessionIndex before native
serialization/signing. The test defines one bounded, explicit-link lifecycle
through the existing public Core APIs, with exact terminal-state and denial
checks. No corrective source hunk is proposed.

This is independent source review only. The candidate C has not been compiled,
the new Rust target has not been type-checked, and neither helper nor lifecycle
has been invoked here. Static source consistency is not native wire or runtime
acceptance. No runtime slot was acquired, no hosted receipt was queried, and no
current capacity or artifact success is inferred. Compiler and focused Cargo
phases remain HELD for separate root review and releases. The original I04
primary assignment `a2d`, original I04 disposition, I02 and all closed rows are
unchanged; root owns integration, release and status.

## Immutable source and read method

| Role | Immutable object | Body read or identity comparison |
| --- | --- | --- |
| Additive source | `fd89dc76be2d2eab92fd252925d0cbb7b4e3b18b` | Full 224 added C lines and all 1,066 new Rust lines read; full old C read supplies every unchanged C/main line. |
| Test refinement | `561022078b500466712a67a2e9a0e1144977858c` | Full final 1,072-line test read, including all six added deadline lines. |
| Initial author evidence | `40d4c565e079e22ac1a1611da2730f06a1fb186f` | Entire 277-line source-phase appendix (2297–2573) read, with earlier design, selected-library and protection witnesses consulted. |
| Final author evidence | `3330d50c05232cf926639e54d934b755f1f47e70` | Entire 36-line final refinement appendix (2574–2609) read; complete report bytes hashed and prefix compared. |
| Old C | `e028106c3d7c9a236056730ad500aad876616540` | All 640 lines read and exact whole-byte reversal checked. |
| Fixed production | `ae8937800254a1ad4296ea257de1eccc4780e45b`, `b619fe25269ccc150e473bbcde47cdb3623ef810` | Relevant source bodies read at b619; nine protected production files and three manifests compared byte-for-byte across both pins and both candidate commits. |

Source reads use immutable Git objects, never the source worker's working tree.
Witness line numbers below refer to final `5610220` for the test, `fd89dc7` for
C, `b619fe2` for production and the selected Lasso 2.9.0 tar member for native
source. The author's much larger historical report prefix was compared as an
identity/provenance boundary; this review does not claim an independent
line-by-line reread or rerun of every prior historical receipt.

| File / role | Bytes | SHA-256 |
| --- | ---: | --- |
| Final `scripts/lasso-saml-sp.c` | 34120 | `1b23f51314038ff15caa3aeb8c31acccb8bcd26d6fb702115767d0f68e4dc186` |
| Final `tests/saml_source_peer.rs` | 39498 | `0a5991d7c17cc921b3a13fdc0e49a2df10d45568aebbafc86e93cf27a2a9f85b` |
| Initial new test, fd89dc7 | 39321 | `200b3da8ab2dca094e16a5ddf54db24394f5f8f2793667ffe0dd0e0bd782093e` |
| Exact old C / b619 C | 23856 | `c3d3a8f7d1d2d472e8b877a89ea2807d534921d88638661e8b251e2365906563` |
| Author report before new source phase | 166178 | `9e1d7adfa8d97278f0fd8802909f45f185cb85df008940c91ae72f2e5562bdb8` |
| Author report at 40d4c565 | 184158 | `0b6638183f6e7ff87821c3355ab2c6d1a96a76a39bda0f241297d485b0d61e19` |
| Author report at 3330d50c | 186493 | `6cff38ba929d515f934ece93ddb588fa92474738f23efe7b3eff5c6d61589740` |

The prior native SP/SLO PASS, two failed receipts and first historical unknown
cause remain the author's historical evidence. This new source review neither
reclassifies those failures nor credits them as an IdP-to-riAuth lifecycle pass.
The new IdP target is a separate, currently unrun candidate.

## Primary native implementation and installed identity

Read-only selected tar:
`/tmp/riauth-i04-saml-sp/lasso-2.9.0.tar.gz`, 4053813 bytes, SHA-256
`63816c8219df48cdefeccb1acb35e04014ca6395b5263c70aacd5470ea95c351`.
Members were read in memory without extraction or native execution.

Installed selected dylib:
`/opt/homebrew/Cellar/lasso/2.9.0_4/lib/liblasso.3.dylib`, 501600 bytes,
SHA-256 `0af7c7ccfda4fe8d20c6ccdf5974a006b2b59a2d50244be95ea197c2d1f72cde`.
A bounded Python byte reader inspected its Mach-O arm64 dylib header, 26 load
commands and export trie (573 exported names). All five required exports are
present: `lasso_provider_get_metadata_one_for_role`,
`lasso_login_get_assertion`, `lasso_saml2_authn_statement_get_type`,
`lasso_server_new_from_buffers`, `lasso_profile_get_signature_status`.
This is symbol/identity inspection, not loading the library or proving its
transitive dynamic linkage at execution.

The installed 359-byte `lib/pkgconfig/lasso.pc` was read as text, SHA-256
`b33a6d16197865beda424287def683acd372026239b1725f08cbc06f3b6a452e`.
It declares the selected prefix and version 2.9.0; pkg-config was not invoked.
Seven complete public headers matched their whole tar-member bytes:
`id-ff/login.h`, `profile.h`, `server.h`, `provider.h`,
`xml/saml-2.0/saml2_assertion.h`, `saml2_subject.h` and
`saml2_authn_statement.h`. Public AuthnStatement contains `SessionIndex` and
`SessionNotOnOrAfter`; Assertion contains ID and its AuthnStatement list.
The assertion getter returns a new reference (native `id-ff/login.c:2649–2657`);
the metadata getter returns an owned duplicate (`id-ff/provider.c:260–278`).
The candidate frees those owned values and retains borrowed profile fields.

Native source witnesses read to resolve the actual contract:

- `saml-2.0/login.c:181–204,279–427,616–731`: FORCE requires verification;
  original Redirect signature status is checked; native ACS URL is matched
  against SP metadata and POST binding; authentication/consent and persistent
  federation are native profile operations. `xml/tools.c:975–1070` verifies
  the original encoded query components. Wrong pinned SP verification can
  return `LASSO_DS_ERROR_INVALID_SIGNATURE`, exactly 102 (`errors.h:135`).
- `saml-2.0/login.c:747–924`: native assertion creation sets issuer, audience,
  bearer subject and request correlation. Its automatic SessionIndex depends
  on an SLO endpoint (875–879), so the reviewed login-only fixture supplies
  the public AuthnStatement fields before response export. This does not
  claim SLO coverage or manipulate serialized signed XML.
- `saml-2.0/login.c:1547–1611`, `profile.c:1615–1642` and
  `saml2_helper.c:645–690`: FORCE installs the response signing context;
  assertion signing context is separately installed; same assertion is
  retained in the response/session. `id-ff/server.c:782–812` consumes the
  supplied private key and certificate buffers.
- `xml/xml.c:804–834,2820–2892` and `xml/tools.c:1228–1269`: child nodes are
  built recursively and signed during native export. The public SessionIndex
  assignment therefore precedes both signatures. `xml/tools.c:3099–3208`
  selects exclusive canonicalization, RSA-SHA256/SHA256 and enveloped/reference
  transforms. Native StatusResponse/Assertion snippet order puts each
  signature after Issuer. `xml/xml.c:157–190` emits single-line base64.

These body reads support the intended wire assertions. Their satisfaction by
an actual compiled peer remains a future runtime oracle, including exact
certificate-only KeyInfo and signature layout acceptance.

## C mode and lifecycle witnesses

| Contract | Exact witnesses and source conclusion |
| --- | --- |
| Original request and binding | C616–711 frames only four signed Redirect fields, rejects duplicates/controls and wrong algorithm, preserves the original query, then requires issuer, destination, ForceAuthn, nonpassive persistent NameID policy, POST binding and native ACS. Test447–488 obtains an actual public Core start/reservation and original query. |
| Authentication and signatures | C745–832 sets both FORCE hints, verifies signature status, builds a native authenticated assertion with real UTC/300-second expiry, sets native public session fields, then builds the response. Test514–687 requires exactly one Response/Assertion and two properly parented native signatures, exact reference IDs/algorithms/certificates, correlation, audience and times. Untouched native base64 enters the production verifier. |
| Stable identity and narrow scopes | Test759–911 creates fresh local identities, real fixture RSA certificates, configures the actual SP public key and one persistent-name login-only source with empty scopes/groups/mappings, no automatic provisioning and no trusted MFA ACR. Native persistent NameID remains stable across restored IdP identity, while assertion/session IDs change. No email adoption or unrelated identity linking is asserted. |
| Explicit local link and finish | Test704–753,945–978 drives callback, read-only finish review, explicit approval, one-use finish and issued source token for Alice. Production `source_finish.rs:1–290` binds the local fresh session and native subject; `source_runtime.rs:299–435` preserves local/factor freshness and explicit link semantics. |
| Unlink and token effects | Test945–978 refuses unrelated-local and source-token unlink, then fresh local Alice unlinks, audits, revokes the source token and preserves both local identities. Repeat unlink is denied; native identity restores/relinks without adopting another local identity. Production `management.rs:2765–2920` supports the expected owner/session and revocation behavior. |
| Trust withdrawal and restoration | Test983–1035 constructs native output before live public certificate withdrawal, checks token revocation plus sticky pin retirement, and refuses the stale pending callback. Restoring old pins does not revive the request, finish token or source token; local identities survive. |
| Signature denial, replay and audit | Test913–935 demands native wrong-SP-pin refusal at the closed stage/code102 with no outputs or Core changes. Test1037–1072 requires a structurally valid native response under an untrusted IdP key to produce completed:false plus sealed denial; no finish or retry revives it. Production claim/record files (57/78 lines, fully read) support terminal failure and the exact audit path. |
| Receiver pins and times | Entire production `source_saml_runtime.rs` (731 lines) read: independent pinned Response and Assertion verification; named persistent identity/qualifiers; Destination, Recipient, InResponseTo, audience and real time checks; required SessionIndex even without SLO; source-request fingerprint and replay constraints remain unchanged. |

The sole test is ignored and macOS-specific, using public Core APIs rather than
an HTTP/browser fixture. No Linux, browser, TLS, other NameID/profile, external
tenant, universal SAML or deployed/released acceptance is claimed. Empty source
groups do not redefine Group behavior; production Group, credential, receipt,
route-header, PAM and authorization contracts have no source edits here.

## Private files, deadlines and failure disclosure

C calls the existing strict `slo_limits`, `slo_read` and `slo_write` helpers,
not the older request-mode path-printing I/O. Those complete bodies were read.
Input/output files are bounded at 128 KiB, owned regular single-link 0600
files; no-follow opens and exclusive output creation protect the fresh private
fixture. C uses a 15-second alarm and file-size limit; new diagnostics disclose
only a fixed stage and integer code. Three output paths are distinct from all
inputs. Identity/session XML and POST contents are private file data.

Test206–258,318–438 requires fresh 0700 private siblings, no-follow/0600/capped
captures, input byte preservation, at most five direct native children and
min(whole-budget, 20 seconds) per child. Final six-line refinement keeps that
child deadline through all output acceptance. `OwnedChild` kills/reaps its
owned direct child on refusal/unwind; the fixture does not create pipe-reader
threads, join reader threads or escape the outer Cargo process group. Whole
60 seconds is an acceptance budget checked through the lifecycle, including
setup; it is not an independent asynchronous timer around every Core/OS call.
A future root wrapper still owns outer process-group timeout/cleanup and
resource monitoring. No runtime process ownership or cleanup was observed here.

Test55–106 discards private error values. Failure messages use fixed labels;
no assert_eq!/assert_ne! or raw error/debug dump is used for private objects.
Test260–303 projects stderr into at most one of eight fixed native stage names
plus a canonical i32; malformed/unknown/duplicate diagnostics become None.
Failure displays only exit code/signal/that projection. It never prints XML,
NameID, session, password, tokens, assertion contents or the raw native stderr.
Native library diagnostics remain capped private files, not public output.

Test108–145 compares the complete stored snapshot. Sealed denial removes only
the exact affected pending key and one specifically identified new denial audit
after proving their expected contents; every remaining key/value must match.
It does not broadly omit audit, sessions, counters, indexes, receipts or domains.
`core.rs:445–472,1108–1175` and `store.rs:1960–2020` were read to verify that
me/snapshot reads and this denial audit do not imply hidden stored revisions.

## Whole-byte preservation and actual static checks

In-memory reversal removed only C insertions at final lines18–20 (headers),
616–832 (217-line mode) and858–861 (dispatch): total224. All remaining23856
bytes, including whole old main, equal both e028 and fixed b619. C is identical
between fd89 and561. Earlier ae893 C is the historical373-line pre-SLO helper,
not the exact640-line base for this addition.

Final test reversal removed only inserted line261, line409 and431–434,
restoring all39321 fd89 bytes exactly. fd89 has precisely two changed paths:
C and new test;561 changes only the new test. Existing
`tests/saml_sp_peer.rs` remains39541 bytes/SHA-256
`897e778c2c53d6bc5ea943aefde46812a80cb73bafad82d1702a7529779ea20f`
at b619/fd89/561. All three existing `tests/identity/saml*.rs` files also match
b619 across the candidate commits; those are identity comparisons, not claims
of new full-body review or execution of the older test suites.

All nine relevant protected production files match byte-for-byte across
ae893/b619/fd89/561: `source_saml_runtime`, `source_catalog`, `source_finish`,
`source_saml_claim`, `source_saml_record`, `source_saml_keys`, assembly keyring,
keyring and management. All three manifests match: Cargo.toml, Cargo.lock and
rust-toolchain.toml. Full runtime/catalog/finish/claim/record bodies were read;
relevant management spans were read; remaining protection entries are exact
identity comparisons. No production algorithms or dependency changes exist in
the two source deltas.

Inspection errors retained: an initial comparison incorrectly required the
old SP/SLO test to equal the earlier ae893 test and failed that assertion.
The corrected comparison uses fixed b619 and candidate parents: the accepted
historical SLO diagnostic refinement accounts for that difference, not this
IdP addition. One rg invocation also returned2 for guessed hygiene/report
paths absent in this WT; paths were rediscovered and author report read from
immutable Git. Large combined read outputs were truncated; the full new test,
new C addition and relevant native/production/report spans were subsequently
read in bounded chunks. No compiler, helper or runtime failure occurred here.

Repository docs, staged hygiene, whitespace and report-only scope results are
recorded below after authoring. They validate this report and repository layout,
not C/Rust type correctness or a native protocol result.

## Prospective single filtered runtime command — not executed

After root's separate compiler-only reservation and actual accepted helper
artifact/hash/dependency result, then a separate Cargo reservation:

```sh
env CARGO_TARGET_DIR="$PWD/target" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  RIAUTH_TEST_LASSO_IDP="$PWD/target/i04-lasso-idp-source-design-v1/lasso-saml-sp" \
  cargo test --locked --test saml_source_peer \
  lasso_idp_redirect_post_source_lifecycle_replays_and_live_trust \
  -- --exact --ignored --test-threads=1
```

This is the unchanged author's one-filter default-Platform proposal, with no
new features or broader campaign. Exact final C/test/library/toolchain/helper
pins, a fresh private target, fresh capacity and owned cleanup must be reviewed
before release. Author planning estimates4 GiB incremental need/13 GiB start;
these are not independently measured resources, current host capacity or a
reserved slot. Floor8 GiB, private jobs1/incremental0/dev+testdebug0 and the
reviewed outer monitoring/deadline controls remain prerequisites. The reported
future compiler/metadata steps are held too; no cc/pkg-config/tool-version,
provider/library/protocol/HTTP/browser or native command was executed here.

Only this independent report is to be integrated. No source, test, manifest,
workflow, existing report or accepted branch is edited. No source worker or
reviewer contact, new worker/task/worktree, desktop, push or status mutation.

## Report validation receipt

Actual static commands: `python3 scripts/check-docs.py` EXIT0 (Markdown links
and build-directory layout checked); `python3 scripts/check-repo-hygiene.py`
EXIT0 (965 tracked files after staging this sole new report); `git diff --check
HEAD` and `git diff --cached --check` EXIT0. A Python source/byte check verified
all whole C/test reversals and protected equalities above, complete author
prefix preservation, newline/trailing-whitespace hygiene and exactly one new
reserved report with no tracked source changes. Staged scope is this one file.
No pre-existing docs/layout error remained in these actual checks.

The inspection assertion failure and absent-path rg error described above are
retained as review-method errors, with corrected comparisons and reads. They
are not erased, attributed to a repository checker defect or claimed as runtime
failures. No source correction, candidate compilation, test execution or
protocol outcome was observed. Root may consider the source review complete
for its bounded next decision; native compatibility/lifecycle and original I04
acceptance remain open pending separately authorized actual checks.
