# Wave 30 I04 original-scope independent review

Date: 2026-10-02. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Original I04 task: `dae9c528-9e32-462c-947f-661a571f136b`.
Supporting WT: `ed9ac424-59f4-4520-905b-919aea3521eb`, branch
`roadmap/local-revisions-coordination-wave27`, starting clean HEAD
`9475ed0e91002d2539ab20abdf79fd2fa5a63f00`.
Reservation: `wave30_I04_current_original_scope_independent_review`.
Only this new report is changed. Original primary WT remains
`a2dff16a-c4b0-47fc-96de-ac85b1fb6d9e`.

## Recommendation against the original row

**Recommend original I04 as a DONE candidate for root's acceptance of the new
native IdP slice and actual receipt. No further concrete local correction or
runtime reservation is proposed.** The missing local peer role identified by
`ea24cb485f82534eac5298d308801d103c183619` is now exercised: real GNU Lasso
IdP construction and signing drive riAuth's source/SP callback, explicit account
link, reviewed one-use finish, unlink, trust withdrawal and terminal denial.
This conclusion uses implemented, executed peer fixtures and accepted historical
protocol evidence; this report itself is not implementation or completion proof.

The live exact-project row was read first using `riwork task list --project
891e7443-8dac-4c1b-897f-9e53cb59c7ee --json`, selecting only the original UUID.
It remains `in_progress`, `[P2] I04 — Verify SAML, LDAP, and RADIUS profiles`, with
A02/A06/Q02 prerequisites and the original primary assignment. Requested outcome:

> Test exact supported roles, bindings, methods, mappings, certificates, and revocation behavior against real peers.

Goal: turn supported protocols into complete, tested integrations. The original
workstream gate is:

> Every advertised integration has a working setup, lifecycle, and failure-handling path—not just an endpoint or protocol module.

The evidence instruction requires actual relevant implementation, tests, docs
and artifacts as applicable, plus gaps/prerequisites; documentation or a worker
report alone is insufficient. The row's old scheduling text was left intact.
Root alone interprets acceptance, integrates, publishes and changes status.

This recommendation concerns the documented local protocol profiles. It does
not certify all combinations of NameID, mappings, encryption and transport, or
a hosted vendor deployment. Those limits remain explicit below. The previously
identified reachable local independent-IdP role is no longer missing; an
all-provider, device, browser, human-study, release or full-CI campaign is not
substituted for the original outcome.

## Immutable inputs and actual read coverage

Published reference: `69f46cf75390cde70a992eb528c7ee5f769aeeca`.
All source reads were from Git objects, not a moving source worktree. Repository
guidance was read; no applicable AGENTS file was found in the worktree/ancestors.
No merge, alignment, source port, worker contact or runtime occurred here.

| Input | What was actually read |
| --- | --- |
| `fd89dc76be2d2eab92fd252925d0cbb7b4e3b18b` | Complete 864-line `scripts/lasso-saml-sp.c`, including all 224 additions, unchanged private-file/resource guards, SP/SLO modes and dispatch; complete new Rust body through its corrected descendant. |
| `561022078b500466712a67a2e9a0e1144977858c` | Entire six-line deadline delta; its resulting test is the parent of the bounded certificate correction. |
| `24c0dc921703432ff04a5cab7cc771c0c9d7df3e` | Complete 1,071-line `tests/saml_source_peer.rs`, including all helpers, five native calls, success/negative operations and final assertions; full certificate correction diff. |
| `4ed5b494454da33bd27bc046084522c05a55e2a5` | Certificate-correction evidence appendix and prior compilation-failure attribution; whole report identity, not a claim to reread its entire 215,211-byte history. |
| Published SAML hooks | Whole `source_saml_runtime.rs` (731 lines), `source_catalog.rs`, `source_finish.rs`, `source_saml_claim.rs`, `source_saml_record.rs`, `management/source_links.rs`; `source_runtime.rs:299–435` reservation body and `management.rs:2765–2933` unlink/source-writer bodies. |
| Published older fixtures | Complete three SAML `exercise` bodies; IdP helpers `saml.rs:1–194`; `saml_source.rs:1–600`, including `Upstream` construction/signing and the entire source exercise; whole `tests/ldap.rs`, `tests/ldap_provider_peer.rs`, `tests/radius_peer.rs`; both complete LDAP/PAP-RadSec functions in `network.rs:89–771`; EAP lifecycle `radius_eap.rs:972–1198` and native TLS/framing helpers `135–347`; whole `scripts/test-ldap.sh`. Remaining older file suffixes were not fully reread. |
| Published contracts/evidence | Whole `docs/saml.md`, `ldap.md`, `ldap-provider.md`, `radius.md`; relevant limitations and CI protocol command spans; full ea24 original-disposition append and selected accepted historical receipts; complete published `local-wave30-i04-lasso-idp-independent-review.md`. Large historical-report reads initially truncated; bounded subsequent reads supplied the selected original-disposition/provenance spans. |
| Later root cleanup `238618f0a8e916647682f6212ae18384469a80ff` | Full one-line deletion and whole-file reverse comparison, independently separated from the earlier PASS. |

The two explicitly authorized public evidence files were read in full from
supporting WT `e1b4399a-8c0d-46b8-880c-a71a4ebf53e7` at
`/Users/dominik/orca/projects/riAuth-public-preview-local-extension-isolation-wave27`.
No private protocol, input, temporary fixture, compiler capture, helper binary,
native library or archive was opened. Native implementation/header inspection
described in the prior independent report is historical reviewed evidence, not
an archive or library inspection performed by this review.

| Exact public file under that WT | Bytes | Independently matched SHA-256 |
| --- | ---: | --- |
| `target/i04-lasso-idp-4ed5b49-owned-asn1-filter.log` | 1055 | `f0807e70fc5ab811d18aa2daac6df649be29b47b5f69dc8975834c0958769602` |
| `target/i04-lasso-idp-4ed5b49-owned-asn1-filter-evidence.json` | 12603 | `316302890bdc93373572c01dcbf5918d0a687b20146a0ba47b7d404f977a0288` |

## Actual new native result and attribution boundary

The public log and JSON agree on the single default-Platform macOS invocation:

```sh
cargo test --locked --test saml_source_peer \
  lasso_idp_redirect_post_source_lifecycle_replays_and_live_trust \
  -- --exact --ignored --test-threads=1
```

Recorded start/end: `2026-10-02T19:50:10.426059+00:00` /
`2026-10-02T19:50:16.472821+00:00`. Cargo EXIT0; **1 passed, 0 failed,
0 ignored, 0 measured, 0 filtered**; build2.59s/test2.63s;
wrapper6.0465591247193515s. No retry or alternate filter is credited. The
wrapper used the own `target`, jobs1/incremental0/dev+test debug0 and explicitly
selected IdP helper. Cargo group39829 was joined/reaped, final group empty,
pipe EOF, no stop reason/supervisor error; minimum sampled free disk
22,505,332,736 bytes, above the 8 GiB floor. Two-second process samples are
not an exhaustive enumeration of short helpers or an independent TempDir
cleanup observation; host disk delta is not own build peak.

Recorded source HEAD is `4ed5b49`; actual Rust source is corrected `24c0dc9`,
39,539 bytes/blob `bc127dbe6b6060f1e490cb59bea63fa9e7ee201d`/SHA-256
`b196cba1eea40a67c7d70c6b463ae50ec6897d68c5d81052e33eb3bec2ae5894`.
Native C is34,120 bytes/blob `5fdcd972e4aee7dc2cd5ea9d8c96e3da23f7f76b`/SHA-256
`1b23f51314038ff15caa3aeb8c31acccb8bcd26d6fb702115767d0f68e4dc186`.
The receipt's before/after maps preserve selected helper SHA-256
`951465d744c1bf99e7ed91fc414337d00e960c24a1715977cce0e14535794bff`
and Lasso2.9 dylib SHA-256
`0af7c7ccfda4fe8d20c6ccdf5974a006b2b59a2d50244be95ea197c2d1f72cde`.
Those artifact identities are recorded provenance; they were not rehashed or
loaded independently here.

The warm production pin is `ae8937800254a1ad4296ea257de1eccc4780e45b`,
not whole published69. The relevant SAML/source/writer files and manifests are
byte-equal as detailed below. The five later API/portal/source-stage files
are not credited as executed by this Core/native filter. It starts no HTTP
listener, browser, hosted provider or real IdP user-authentication UI. Lasso
is a real independent protocol library; the C application supplies TRUE
authentication/consent for its synthetic principal and fixture metadata.

## Independent review of the new protocol and security behavior

| Mechanism | Source witnesses and reached test oracles |
| --- | --- |
| Exact signed request | C616–711 bounds four unique Redirect fields, exact RSA-SHA256 and 43-character relay, verifies the original encoded query using FORCE, and requires native SP issuer/provider, exact destination, ForceAuthn/nonpassive persistent AllowCreate and POST ACS. Wrong SP metadata pin reaches the fixed native request stage/code102, exit1, no output files or Core changes. Correct pins then use the same unclaimed reservation. |
| Native response, no handbuilt success XML | C745–832 calls native request processing/validation, assertion construction and response construction. It sets the public AuthnStatement SessionIndex to native assertion ID and expiry before native serialization/signing because this login-only profile has no SLO endpoint. The test checks two native signature parents/references, certificates, RSA-SHA256/SHA256/exclusive transforms, unique IDs, issuer/audience/Recipient/InResponseTo, persistent qualifiers, fresh actual timestamps and300-second bounds. Untouched native base64 enters riAuth's independently pinned verifier. |
| Explicit local identity and one-use finish | `complete_link` checks authenticated audit/replay increments, callback retry refusal with full snapshot equality, read-only review with exact Alice target and mfa:false, explicit finish, exactly one link/session/upstream-session, single link audit, spent finish refusal and correct issued identity/SessionIndex. Two successful native completions preserve the same federation after native identity restoration while request/index change. No email adoption, auto-provision, administrator login, attribute or trusted-MFA configuration is smuggled into this narrow peer. |
| Unlink and live authority | Normal fresh local unlink refuses the unrelated user and the source-issued token without writes; owner unlink audits once, removes the link, revokes the issued token and leaves both local accounts alive. Normal `source_put` withdraws the IdP pin, revokes the second source session with pin_retired:true and preserves local accounts. Native output made before withdrawal cannot complete the changed request. Restoring pins cannot revive its relay/finish or revoked session. |
| Wrong IdP trust and terminal denial | A structurally valid fifth native response under another real fixture key is rejected by the current pins. Callback returns completed:false, the exact pending record becomes claimed+failed and exactly one source.login_failed audit is added. `sealed_denial` then compares every other stored key/value. Repeated callback/finish cannot issue identity, retry the proof or duplicate writes. |
| Product checks remain operative | Claim commits retirement outside the returned refusal; record rechecks the live source fingerprint and assertion replay key; verifier independently verifies Response and Assertion with preserved namespace context; finish checks live enabled source/fingerprint, target session/account/factors, explicit approval, link ownership and upstream expiry in its normal writer. Unlink retains owner/freshness/factor, receipt replay/save, revocation and audit behavior. No bypass writer or new mint exists in the peer delta. |
| Bounded private lifecycle | Shared C guards impose15-second alarm,128 KiB owned0600 regular single-link no-follow inputs and exclusive outputs. Rust uses a fresh0700 fixture, capped private captures, env-cleared helper, at most five native calls and min(whole budget,20 seconds) child deadlines. The561 refinement retains that deadline through output validation. Child Drop kills/reaps on unwind. Whole60-second budget is checked at checkpoints, not an asynchronous bound around every Core call; the outer wrapper owns its separate group timeout. Fixed-label failure projection excludes protocol/private values. |

No certain compile/runtime/API-semantic or security defect was found in these
complete bodies. Actual PASS satisfies the test-defined native wire checks;
the source review alone is not a native oracle. The test does not execute
HTTP header contracts, GUI/JS, mapping/encryption variants, local OTP, deployed
HA or paused external-I/O exclusion. Older relevant evidence supplies its
own precisely attributed coverage rather than being relabeled as this filter.

## Original profile mapping and historical executions

Contracts at69 are [SAML](../saml.md), [LDAP import](../ldap.md),
[LDAP provider](../ldap-provider.md), [RADIUS](../radius.md) and
[limitations](../limitations.md). Source paths in this section refer to69;
historical executions retain their original pins. No old raw log or native
artifact beyond the two authorized files was reopened here.

| Original facet | Actual accepted execution and source-backed coverage | Precise boundary |
| --- | --- | --- |
| SAML IdP roles/bindings/mapping/certificates | Accepted Lasso2.9 SP signed Redirect→POST login validates signature/audience/time; Recipient is a separate helper comparison. Historical tampered NameID, replaced metadata pin, re-signed expired Conditions and wrong Recipient refuse. CI XMLsec IdP exercise signs POST AuthnRequest, verifies signed metadata/Response/Assertion and decrypts optional encrypted assertion; its full body checks request/ACS index, escaped mapped name, fresh authentication, browser binding, consent/group withdrawal. | Lasso is an independent SP library receiver with persistent NameID and empty attributes. XMLsec is a real crypto tool, not a complete SP; groups mapping is configured but not separately asserted at an independent receiver. Transient/email/unspecified and arbitrary deployed mapping combinations are not all native receiver passes. |
| SAML source/SP role and lifecycle | New actual Lasso IdP PASS closes ea24's missing independent protocol-generator role. Existing XMLsec source exercise additionally drives mapped display/email/verified values, explicit link/no email adoption, issuer/audience/recipient/request checks, stale authentication/tamper/namespace negatives, encrypted assertion and trusted MFA context, unlink/live trust paths. | Older `Upstream::response` is handwritten XML and risaml encryption; only its signer is independent XMLsec. New Lasso success messages are native, but its configured profile is persistent, unencrypted, empty mappings/groups/ACRs, login-only. Optional browser-stage delivery has source tests, not a new real-browser or SAML-stage run. |
| SAML revocation/SLO | Accepted real Lasso SP/SLO receiver restores native identity/session, verifies exact persistent subject/index, removes the matching peer session, reloads its empty state and returns a correlated signed Redirect response. XMLsec logout exercise covers SP Redirect, SP POST and upstream POST fan-out, local revoke first, loop prevention, wrong peer/request/relay/configuration, replay, unrelated account and Success/PartialLogout. | Lasso proves one local-library Redirect receiver profile. XMLsec peer protocol is test-authored, including direct durable-deadline advance for late response; no natural remote timeout or every-origin browser delivery is inferred. New IdP mode itself has no SLO endpoint. |
| LDAP import/mappings/password/revocation | Real OpenLDAP slapd STARTTLS named pass maps entryUUID/uid/cn/mail and description=staff, applies205 entries, retains ID/factor on rename, checks real user password/MFA/recovery, untrusted CA, stale content/collision/revoked agent, reviewed removal and session invalidation. Whole script/fixture were read. | Fixture preparation uses a disposable directory; production service/user binds use STARTTLS. Actual import LDAPS, AD objectGUID/sAMAccountName/memberOf and directory-specific overlays are unexecuted deployment variants. Provider LDAPS is not credited to import. Errors are not empty/removal authorization and no local-password fallback is claimed. |
| LDAP provider methods/certificates/revocation | Accepted external OpenLDAP ldapsearch2.7.1 uses actual LDAPS and STARTTLS, page size1/two scoped users, disabled omission, wrong/revoked token rc49, wrong CA/crossed scheme rc1. Named ldap3 socket test actually covers root DSE/Who Am I, user/factor/simple bind, scopes/private-attribute exclusion, bound paging/cookie, failed rebind and live agent revoke. | LDAP provider is read-only LDAPv3; ldap3 is an independent client library, ldapsearch an executable. AD/POSIX/write/SASL/password-modify emulation is outside scope. The external-client local result is accepted historical attribution, not a newly read original terminal log. |
| RADIUS methods/mappings/certificates | Accepted FreeRADIUS radclient3.2.10 UDP PAP accept/reject/exact replay has one accept audit. Named OpenSSL UDP/RadSec socket test covers independent packet authentication/Proxy-State, OTP replay/live group withdrawal, standard integer/VLAN and vendor encoding, CA+exact NAS leaf pin and rogue/anonymous refusal. | FreeRADIUS evidence is radclient, not a FreeRADIUS server/hardware NAS, and does not prove RadSec. RadSec TLS is real OpenSSL; RADIUS packet framing is handwritten fixture code. NAS enforcement of reply values/established sessions remains the NAS's responsibility. |
| RADIUS EAP-TLS assurance/revocation | Named independent OpenSSL method13 fixture negotiates TLS1.3 and optional TLS1.2, fragments/retries, protected success and exported/decrypted MPPE keys; explicit user-bound certificate rather than CN/outer-identity adoption; missing/rogue cert/MFA refusal, unrelated enrollment, live group/binding/CRL withdrawal including cached accepts, strict-listener TLS1.2 refusal. | TLS engine is real OpenSSL inside test-owned NasPeer/EAP framing over UDP. No physical supplicant, OS matrix or combined EAP-over-RadSec execution is inferred. No CoA/termination of established NAS sessions. |

Historical attribution was checked against the immutable ea24/current accepted
report records and actual fixture bodies, rather than fresh execution here:

- CI36950097067/job110661000640 at `2d05c00deed3a6dafcd458a5f1a1a9beb17d0dd8`:
  three `RIAUTH_TEST_XMLSEC="$(command -v xmlsec1)" cargo test --test identity
  --locked FILTER -- --ignored` filters `saml_independent_xmlsec`,
  `saml_source_independent_xmlsec`, `saml_logout_independent_xmlsec` record
  respectively1/0 in5.94s/5.63s/7.78s; `scripts/test-ldap.sh` records
  `openldap_plans_stable_ids_tls_login_mfa_and_fail_closed_sync`1/0 in8.05s.
  Recorded raw265770-byte log SHA-256
  `458a30895b5987c0bebe2fb7368a1a32969fa0d41ce90acbfa56f43299ccd186`.
  CI XMLsec/OpenLDAP versions are not inferred from native macOS versions.
- Failed aggregate CI36951643190/job110666025654 at
  `d31778f066d3bc79f94829cf6295e1eb8df2a84f` nevertheless records named
  `network_tests::ldap_provider_tls_scoped_search_paging_rebind_mfa_and_revocation`,
  `network_tests::radius_udp_radsec_authenticators_mfa_duplicates_pinning_and_live_policy`,
  `radius_eap_tests::openssl_eap_tls_versions_fragments_keys_enrollment_policy_and_revocation`,
  source browser ACS and signed/encrypted source passes. These are historical
  named passes, not a green aggregate or fresh whole-current-tree execution.
  Recorded raw256279 bytes/SHA-256
  `5a94f6f07d83ae7f4565d8fb032c3a629f31b44cff99ac84e67f7c0078d376f8`.
- Lasso SP login/lifetime/Recipient accepted pins:
  `62ce3d83980c52f5242353952a3c4a0c7fdc6b7a`,
  `8d389d4ca919918e8a9ddf5f4531effc55fb3757`,
  `25cd9b517a932fa72251291433ec9ef6f2ac890d`, each recorded1/1.
  Later native SLO receipt in `743dfedfc1c70b0b2b08b0bd4e7318b064eb949e`
  records Cargo0/1pass0fail1filtered/build2.59s/test2.09s, helper SHA-256
  `18f148c0a3119d4a1268597c829c680a743c4978ccc96924dac337368d3a891c`.
  Its native production is also protectedae; it is not the new IdP result.
- OpenLDAP provider accepted `fcba8a5e81037edb365c02c291b512576817e920`,
  source `d00437fc534faef0b1e2a0bb07a9891d1f018083`, recorded1/1 using
  `scripts/test-ldap-provider.sh` and `/opt/homebrew/opt/openldap/bin/ldapsearch`.
  FreeRADIUS accepted `38f82fe7fce02c0bd70db4a4cd22bb15edcea7a6`, source
  `c51a0ef9bdb1dc03eddee5244871345c1f5a7456`, recorded named1/0 in2.47s
  using `scripts/test-radius.sh`; raw12298-byte SHA-256
  `f5fff90c6e750382a9ccbac1e44c817df36490c7fb0e96bb8befb3939a28bb86`
  has no checkout marker, so source linkage remains accepted ledger attribution.
  Earlier network/EAP accepted pins are `6b201b163dab33384b6c92bda6d8d28876163baa`,
  `3c09fc398872057f13cda62f718513fc859590f3`,
  `c077d41c4960c7c42eb17edb045d10682350a1f1`.

Unsupported SOAP/artifact/ECP/encrypted NameID, transient/unsolicited upstream
login, LDAP mutation/schema emulation and unsupported RADIUS methods remain
outside the advertised bounded profiles. Actual hosted peers, AD/LDAPS-import
variants, physical NAS/supplicants, Windows, browser navigation/JS and release
certification remain unmeasured. They are explicit profile/deployment limits,
not newly invented mandatory local closure campaigns. Existing SP/NAS sessions
cannot be universally terminated by riAuth, and no HA or paused-I/O claim is made.

## Failures, warning cleanup and preservation proofs

Both earlier native SLO attempts remain failed. The first inner cause is still
unknown. The diagnostic second attempt observed native request processing
code102/exit1 but failed its stale expected `(-111)` assertion; later positives
were unreached. The later SLO pass does not retroactively change either result.
The IdP compilation attempt at ac869/9ec96 failed Cargo101 with two E0308
temporary-owned-Asn1Time borrowing locations; zero native test oracles ran then.
Its recorded1735-byte log SHA-256 remains
`380ebe9e2f4ef43d173a8e1affa929a77235dd024b125fbe18c7b11e8d5bbe5f`.
The corrected24c block binds the existing0/1-day validity objects to owned
locals before borrowing; certificate validity/security semantics are unchanged.

Actual new PASS log includes unused `POST` warning at test50 and a linker
`__eh_frame` warning. Root derivative238 deletes only the unused69-byte constant
line. Its test is39,470 bytes/blob `adfc2b4a6d8362a7d84f9ea441e5be380783e292`/
SHA-256 `610e29c32d7eed456c33493aea2bb1eab52fc7a5791d36db5686c59532fc5512`.
Reinsertion restores all39,539 corrected24c bytes exactly. No later compilation,
lint, native pass or linker-warning resolution is inferred for that derivative.

Actual in-memory Git-byte checks passed:

- Whole C addition reversal reconstructs the exact old640-line/23,856-byte
  helper; initial fd89 scope is only helper plus new test.
- Reversing561's six deadline lines restores fd89's complete39,321-byte test;
  reversing24c's certificate block restores561's complete39,498-byte test.
  The24c commit itself changes only the test (4 added/5 removed lines); the
  wider561-to24c history also includes separate evidence reports.
- Fourteen protected paths are byte-equal across ae/published69/fd89/561/24c/
  4ed/238: runtime/catalog/finish/claim/record/source_saml_keys, assembly keyring,
  keyring, management, management/source_links, source_runtime, Cargo.toml,
  Cargo.lock and rust-toolchain.toml. Only the explicitly listed bodies above
  were semantically reread; remaining paths are identity checks.
- Whole `src`/`crates`/manifest/toolchain/build comparison ae→69 identifies
  only API, portal and three source-stage files as differences. No later
  browser/route execution is credited to the warm native production.
- Complete IdP/source/logout exercise spans at69 equal their actual2d05 CI
  spans:19,334/13,707/11,913 bytes, SHA-256 respectively
  `df10198089b8c55cd929dc7364e721b375efdadebbc74cc67e7f6855f58d2942`,
  `bd93c1c4247c95685c4e51b29a813c6f6924b9a634ba10b9a1c000e89ca2d511`,
  `0589daa0a8d5d78b55dffb717a146739088fa58e844cf14e0116d2c3b5b960dc`.
  Whole network/EAP/source fixture files at69 equal the failed aggregate's
  d317 pin; equal blobs preserve named historical attribution, not new runs.

Inspection limitations/errors retained: one guessed public evidence basename
was absent; the exact authorized `-filter-evidence.json` was then found and
matched before reading. An initial byte-reversal utility incorrectly split
unified hunk headers and failed its first assertion; corrected line-header
parsing completed all comparisons without writing candidate files. A read-only
`riwork orchestrator --help` lookup exited2 because that subcommand lacks that
flag; its printed usage confirmed explicit-project send syntax. These are
inspection errors, not candidate compilation or protocol failures. No private
values or unrecorded historical caller/cause is inferred.

## Report checks and handoff

This review acquires/releases no runtime slot. No source, test, guide, existing
report, manifest, board, main or accepted tree was changed. No helper/library/
archive/compiler/Cargo/HTTP/provider/browser/desktop was executed or imported,
and no worker was contacted. Backend authorization, reviewed receipt-secret
behavior, route-specific headers, PAM fallback, held Group, shared admissions,
SCIM stamps and the nonrenewed60-second/paused-I/O limitations stay intact.
Closed I02/I10/D03/D04/O06/R05/W02/W05 and all other dispositions are preserved.

Actual static checks: `python3 scripts/check-docs.py` EXIT0 (Markdown links and
build-directory layout); `git diff --check` EXIT0; report pin/original-row,
newline and trailing-whitespace assertions EXIT0. The initial Git status has
exactly this one untracked reserved report and no tracked diff. Whole-file
reversal, protected equality and historical-span comparison checks above
completed EXIT0 after the disclosed inspection-script correction. Staged
whitespace/scope checks passed: `git diff --cached --check` EXIT0, exactly one
new reserved path staged and no unstaged tracked changes.
`python3 scripts/check-repo-hygiene.py` EXIT0 (944 tracked files). Final clean
state and the immutable report-only commit hash are returned to root.
Root may use this independent DONE candidate
with the actual native receipt and immutable source; original status remains
unchanged here. No further product hunk or runtime campaign is requested.
