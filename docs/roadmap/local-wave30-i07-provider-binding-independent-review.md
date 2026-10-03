# I07 retained-provider binding: independent source and actual-evidence review

Date: 2026-10-03. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original I07 `34688b10-fa4b-4b83-8070-adfb3e55dc41`. Reservation `wave30_I07_provider_binding_independent_design`; existing WT `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2` / shell `2173637e-bdb3-4eba-a128-178f57b41a34`. Entry HEAD `1f04ceb2244e872411684b8f235170bdcef94488` was clean. This review owns only this new report.

**Disposition for root:** the successful-writer canonical provider tag plus an exact current-kind comparison is a supported smallest correction for the reached `require_device_trust` authorization gap. No source blocker was found for that narrow seam. Reserve three production files and `tests/device_trust.rs` only, as specified below, before implementation. The existing failed negative is actual evidence; proposed corrective bytes and additional cases are DATA, uncompiled and unrun. No whole-I07 completion, real Google tenant/device, claims-wide current-provider contract, configuration-rotation fingerprint or current CI result follows.

## Immutable source and evidence roles

Published production pin is `1a517a1a461b7017c353d37a5e498d2c7cfa7985`. Regression fixture pin is `6ba2e95c42c271804d1715eb05380953b008130d`. Actual invocation source HEAD is `be6d4afe932bf03454d6b0dedfa482c8a2e33b4f`. Whole actual test bytes equal immutable6ba:64683 bytes/1829 lines/SHA256 `ee9c1676e8fb3aa6f6d2092a343e70bb0beae703d563d3e410ad611093e340dd`. All1829 fixture lines were read in five bounded chunks, including the entire Google module and final regression;11 test definitions are present. Published1a and own WT have the earlier10-test62736-byte fixture, SHA256 `2287bfff16bac53e096652ac9760342577f0a46aaf3f5e714d064a6947559e8f`. This review neither imported the extra regression nor aligned the branch.

The actual be6 and fixed1a blobs are equal for the record type, Platform policy, Essentials stub, both assembly writers/claims reader, claims engine, Google adapter, model predicates, common fixture and Cargo manifest. `src/core.rs` is not whole-byte equal: actual be6 has the older `list_groups` body whereas fixed1a has the accepted paged body. The full two-version diff is confined to that function; the open/session/authorization witnesses reviewed here are unchanged. This comparison does not reopen accepted S02 or present old source as current production. Source identity comparisons are independent of body-read coverage below.

`CONTRIBUTING.md`48 lines/SHA256 `7e7dd7b756f734a8105cad5b96dffa8a51977c64f3ecd70b8182fa3de6ba6737` and `SECURITY.md`15 lines/SHA256 `2556771d58d09a2a4754e7484645b7e948b84286ef0d21cc66169b920a31c4d8` equal the previously completely read instructions, fixed1a and own files. No applicable AGENTS.md exists in this WT/ancestors or relevant src/assembly/tests/docs/roadmap directories. Findings remain local; no external contact or disclosure occurred.

The complete original I07 row was read from the local project `planning/current-tasks.json` export, **not a live board query**. Observation:244347 bytes,94 rows, SHA256 `5cfd882213745a86dcdeeb7e4debccf24fd22299c0fc7be73d6d988512739752`; canonical sorted compact UTF-8 complete row SHA256 `a1bbe7e4b68fdb42a400e9dfea996acb5ee8594737d9edd21741301153f1519d`. Exact UUID row still records `todo` and original WT `d0cebec9-5552-486a-88c6-9a1eb10e4a45`; no field was changed. Its acceptance is signal provenance, freshness and account/session/device binding against supported managed devices, with actual integration setup/lifecycle/failure evidence. This report addresses the one local retained-proof gap; it does not supply a managed-device enrollment, authorized project/account/customer/domain/trust-class or real challenge responder. The export's historical scheduling text is not a new delegation/runtime authorization.

## Actual negative, including the parser limit

All four permitted redacted JSON bodies were read completely, parsed as data and independently rehashed. Their parent is `deployment-private/i07-provider-transition-negative-20261003-3a19d4ccecd6` in the management WT; directory0700 and each bounded regular file0600. No credentials, raw Cargo log, generated account/private key, raw response/debug value, launch-source JSON or executable supervisor body was read. Additional directory entries were metadata-only discovery, not private-content review.

| Complete redacted receipt read | Bytes | SHA256 |
| --- | --- | --- |
| `actual-status.redacted.json` | 4008 | `a6da69b426fe23f1aa7d1a8888336d6db83ad1e1db68f4de97e13ae7f0cd24e2` |
| `grade.redacted.json` | 1189 | `4f05c6fee69f98743c0a1f6b43c17f719f363a3357e783a5eb927a73b3a9afb6` |
| `panic-location.redacted.json` | 412 | `f5f3d9a6ceeec2b8955d5abccc05dff518eda3e4c7996cef81f984865c7ab79d` |
| `cleanup-release.redacted.json` | 341 | `99c9e0e46e36a9398105e39fe8c46f707bb6f45a96fa9ea872be31b9bfd29ae7` |

`actual-status.redacted.json` records Cargo exit101 after62.32615s, child/ownedPG55144 reaped and verified group absent, complete stream EOF, output retained before grading, no internal failure/stop reason/signals/cleanup errors, no observed dependency download. The original command selected only `google::retained_local_proof_requires_reverification_after_provider_change`, `--locked --features test-support`, `--exact --test-threads=1`, jobs1/incremental0/dev+testdebug0, private existing `target/wave30-o06-readiness`. It was not rerun here.

`grade.redacted.json` records0 passed/1 failed/10 filtered/0 ignored/0 measured. Its original strict thread-prefix parser left path/line/column null and expected-oracle/fixed-Ok flags false. **Retain those original failed parser fields.** The separate later `panic-location.redacted.json` records that original parser miss, exactly `tests/device_trust.rs:1821:14`, fixed-pattern `unwrap_err` receiving `Ok`, and the expected refusal-oracle location reached; it confirms that raw values were not emitted and actual result had already been retained. This is a supplied redacted static classification, not this worker rereading/reparsing private panic output. These receipts differ because of parser coverage, not because the test passed or the location changed.

Complete regression source lines1779–1828 establishes its control flow: valid local verification and protected authorization precede closing Core; the same instance/session is reopened with usable Google configuration and a `Boom` transport; protected authorize is expected to refuse at1821. Actual `Ok` contradicts that negative expectation. The exact error-code assertion after `unwrap_err` and later unprotected control were **unreached** in this run. No actual error message, callback/token contents or live provider invocation is inferred.

Resource DATA independently recomputed:32 samples, minimum16485629952 bytes (15.353439331GiB), initial16775020544 bytes, maximum observed sample interval2.180825s. Observations describe that invocation only; no current free-space check, future peak or hard sampling/write bound is claimed. `cleanup-release.redacted.json` records Cargo101, supervisor55106 exit0/absent, child55144 reaped/absent, owned group absent, no signals/errors and Cargo validation released. Supervisor0 is supervision status, **not Cargo success**. Root's release is a historical receipt; this review acquired or released no lane and made no process probe.

Recorded raw-log identity is1549 bytes/mode0600/SHA256 `b87625e6e5fdcd355641a2f104fd7ecbb8d4fb126ca38749ba6cfc81ab43db48`; source-archive identity2688 bytes/SHA256 `2de54e3e1f188cddbe3e584760fba70549efa11d6220cbe61887e7adc6f87608`. Those are **receipt-reported identities only**, not independent raw-file rehash/body reads. No raw values are copied into this report.

## Complete retained-record reader/writer audit

The fixed recursive search for `DeviceVerification`, `device_verifications`, `approved_device_at` and the trust gate found these interpretations; search results were followed with bodies as specified.

| Reader or writer | Exact source behavior | Consequence for the proposed seam |
| --- | --- | --- |
| `src/device_trust_types.rs:87` | Current retained row has device/user/session/epoch/verified_at/expires_at; session and epoch already default for legacy rows | Add only serde-default `Option<String>`; absence is unknown, never silently stamped local |
| `src/device_trust.rs:74` | `provider_kind`: missing config.provider or explicit local means Local; Google literal means Google; mixed provider settings/unknown names reject | Compare a fixed canonical name derived from this classifier, rather than raw optional config.provider |
| `src/device_trust.rs:104,195` | Readiness validates the usable current verifier; unprotected client returns before config/record access; protected identity/session/epoch/expiry/device check lacks proof origin | Add one conjunction to the existing fresh predicate; preserve all existing reasons/guards and unprotected early return |
| `src/assembly/device_trust.rs:28,36` | Typed exact session lookup and maintenance-page reader | Deserialization/default compatibility remains; no new store key/index/list/scan or generic writer |
| Local writer193–264 | Validate pinned JWT/audience/nonce/device/exp; live session/user and challenge session/epoch/unused/expiry; reject device change; consume and persist/audit atomically | Write fixed local tag only inside this already successful record construction245; do not trust JWT/body fields for provider identity |
| Google writer266–326 | Bounded canonical material, issued challenge/session/replay check, embedded response equality, remote accepted identity/customer/trust response, final transaction recheck | Write fixed Google tag only at successful record309. Prior different-device path consumes/retains response replay and returns error **before** new verification |
| `google_bound_challenge`328–367 | Same account/session/user+session epoch, unused, nonzero issued time and <60s absolute challenge window, stored response-hash retention | No change to challenge schema, nonce, response cache or transaction boundary |
| `src/device_trust.rs:333` cleanup | Expired freshness does not remove a still-live session's device binding; dead/revoked session does; legacy unbound row is removed | Keep provider-missing rows until ordinary lifetime cleanup so a tag migration cannot erase sticky device ID and allow replacement |
| `src/assembly/claims.rs:42` | Independent timestamp reader checks user/session/epoch/nonempty ID/verified_at>0 and<=now/unexpired freshness; returns verified_at | Keep this current historical approval-age fact unchanged. It is **not** a current-provider policy check; see explicit boundary below |
| Claims engine46–138,168–205,317–342 | Rebuilds actual current session identity and authority, then ApprovedDevice tests timestamp age; conditional rules narrow access/mappings | No claims API/trait/approval algorithm rewrite is needed for the literal require-device-trust regression |
| `src/edition.rs:307` and `src/recovery.rs:62` | Platform bucket/dependency registration and restored authority invalidation include the collection | Registry/invalidated-list spans are identity/classification witnesses, not new domain writers; no edition/recovery change |
| Existing fixture329–340 | Sole direct test `DeviceVerification` literal is deliberately planted legacy user-keyed/unbound proof under missing-verifier test | Add `provider: None` there, preserving its original wrong-session/unconfigured scenario; do not manufacture canonical origin for it |

The two production construction sites and this sole test literal are all construction literals found in the indexed src/crates/tests search. Other existing test modifications are typed clones or raw JSON; they continue to deserialize missing-provider rows. Correctly typed provider `None`/JSON null and unknown/empty/differently cased strings will refuse the new protected predicate. A malformed non-string JSON provider cannot deserialize `Option<String>` and remains an error, rather than a healthy/local fallback; no promise that this storage-corruption error has the normal OAuth code is made.

Current local security remains pinned PEM/JWKS, supported algorithm/kid/audience/exp/nbf, no token-supplied remote keys; current Google transport remains fixed HTTPS endpoints, no redirects/proxy, bounded bodies and existing connection/request timeouts. Google expected enrolled domain/customer/trust-class, canonical embedded SignedData equality, response replay retention, local-JWT/cross-shape refusal, audit and sticky device identity are unchanged. This review read those adapter production ranges and all existing test definitions, without calling any adapter/key parser/HTTP/mock function. Queued JSON/repeated-byte test signatures remain synthetic provider evidence, not Google attestation or hardware evidence.

## Narrow semantics and counterexamples

1. **Default versus explicit local:** both classify Local and compare with the same stored literal. Using `record.provider == config.provider` would falsely reject a valid default-local writer's tag; the proposed canonical classifier avoids that. Existing local and Google successes must store their corresponding fixed tag, independent of request fields.
2. **Legacy:** missing or null provider refuses a protected request despite matching all other fields. Reverification uses the normal successful corresponding provider path; no migration backfill or inferred origin. Unknown string likewise refuses. A different device still requires a new session, even for a legacy or mismatched-provider row.
3. **Both directions:** local-tag proof under usable Google, and Google-tag proof under usable local, refuse. A new successfully verified proof for the **same session/device** can replace the tag through the existing writer. A failed Google reply, wrong nonce/session/epoch or different device cannot relabel the row.
4. **Current-kind, not transition history:** local→Google→local without successful Google replacement can reuse the still-fresh original local proof once Local is current again; this proposal records proof provenance, not a counter of intervening configuration changes. No epoch bump or irreversible transition-revocation claim is supplied.
5. **Same-kind settings:** local key/kid/JWKS/audience-related changes and Google key/account/domain/customer/allowlist changes retain the same canonical kind. Current readiness validates usability but the tag does not compare old acceptance settings. Freshness/identity still apply; policy on these rotations remains separate and unmeasured. Google bearer-cache fingerprint is a cache-key mechanism, not a retained DeviceVerification configuration fingerprint.
6. **Claims boundary, source certain:** a client with `require_device_trust=false` but an ApprovedDevice conditional rule/mapping reads the still-fresh timestamp through `approved_device_at`, even if its row tag is missing or differs from current provider. `authorize_identity` runs conditional claims enforcement and then the trust gate's unchanged unprotected early return; no implicit provider comparison exists there. The documented [conditional policy](../oidc-profiles.md) defines this predicate by current user/session/epoch/unexpired approval age, and does not promise current-provider matching. Thus the narrow correction cannot be described as every device-related consumer failing closed on provider change. No actual such transition case was run. This is an explicit bounded semantic counterexample/limit, **not an automatic new claims reservation or a demand to change the documented timestamp predicate**. If root requires current-provider binding there too, it needs a separately reviewed context-aware claims seam; changing timestamps/epoch/storage globally would not be this proposal.
7. **Mixed deployment/rollback:** old readers ignore the new unknown serde field and retain their old provider-blind behavior; only changed binaries enforce it. An old writer can replace a tagged proof with a missing-provider row, which a changed protected-policy reader then refuses. No mixed-version node agreement/fencing guarantee, old-binary security fix or simultaneous same-provider configuration consensus follows.

These limits do not prevent the exact new tag comparison from refusing the observed local-proof/Google-current protected authorization. They do prevent universal verifier-rotation, authorization-consumer, physical custody or same-version artifact equivalence claims. I07's original external managed-device gate remains open; I08/native Windows lifecycle and accepted shared contracts remain separate.

## Exact smallest prospective source reservation

Root may reserve **only** `src/device_trust_types.rs`, `src/device_trust.rs`, `src/assembly/device_trust.rs` and `tests/device_trust.rs`, with an author report handled separately. The first three use fixed1a production bytes; tests use full immutable6ba fixture after root chooses how to preserve that regression. Own test is older and is not overwritten in this review. No api/claims/config/Core/epoch/Store/migration/node-security/approval/credential writer, dependency or guide ownership is proposed.

This complete zero-context DATA diff adds exactly two schema lines, five canonical-mapping lines, one predicate conjunction, two trusted writer fields, and the sole existing test literal field. It does not include the new test cases below and is not a compiled/materialized corrective tree.

```diff
--- a/src/device_trust_types.rs
+++ b/src/device_trust_types.rs
@@ -93,0 +94,2 @@
+    #[serde(default)]
+    pub provider: Option<String>,
--- a/src/device_trust.rs
+++ b/src/device_trust.rs
@@ -209,0 +210,5 @@
+    let provider = match provider_kind(config) {
+        Ok(ProviderKind::Local) => "local",
+        Ok(ProviderKind::GoogleVerifiedAccessV2) => "google_verified_access_v2",
+        Err(_) => return Ok(Some("device_trust_verifier_unconfigured")),
+    };
@@ -227,0 +233 @@
+                && record.provider.as_deref() == Some(provider)
--- a/src/assembly/device_trust.rs
+++ b/src/assembly/device_trust.rs
@@ -249,0 +250 @@
+                provider: Some("local".into()),
@@ -313,0 +315 @@
+                provider: Some("google_verified_access_v2".into()),
--- a/tests/device_trust.rs
+++ b/tests/device_trust.rs
@@ -333,0 +334 @@
+                    provider: None,
```

| Reconstruction input | Baseline bytes / SHA256 | In-memory proposed bytes / SHA256 |
| --- | --- | --- |
| `src/device_trust_types.rs` | 3188 / `2940b6fa74757e5e25a3761c1bb9cae4303a2f54c0c8c58cad550b66a2803273` | 3244 / `e7499decea169e5fd58d8b1819567e072993021cd9834756e36b3ceaf94a68ee` |
| `src/device_trust.rs` | 12579 / `8524103a534cde6e785f3b272f4b69d28becf462c4b005232058d20fb37b51a0` | 12897 / `dafaf3f28e909ab4df3a421827cb7da2fbbaeef12a60fca4c393a9caf6f4852d` |
| `src/assembly/device_trust.rs` | 16633 / `2ad918244800eb6f6c5ce3ab2cdeb8790dc60bbd96e0fcb8de5191819eee688b` | 16749 / `25d451d550ecf96748651574edd4d845cbc6beb8c077f0bd2b6959d62a9c47b7` |
| `tests/device_trust.rs` | 64683 / `ee9c1676e8fb3aa6f6d2092a343e70bb0beae703d563d3e410ad611093e340dd` | 64719 / `672c4e0eaaa7ac1495e2fb0d89234f2ef29439d89a3422d0d20cb5483b2261de` |

Each reviewed old anchor occurred exactly once. Reversing every insertion in reverse order reproduced the complete corresponding immutable input byte-for-byte. The three production candidates add490 bytes total; sole existing test literal adds36. There are no removals/changed old lines, all other function bodies remain identical and every old fixture byte, including the failed regression, survives. This is byte/body preservation, **not a Rust AST parse, compilation, type-check, serde execution or passing test**. No proposal function was executed.

## Exact focused test cases to author after reservation

Keep all11 current6ba tests and their challenge/replay/session/device/epoch/expiry/portal/proxy/refusal/config/audit behavior. The sole legacy literal receives only the provider field above. Add direct stored canonical-tag assertions after the already successful local verification in `fresh_local_verification_allows_policy_and_issuance` and Google verification in `google_verified_access_v2_binds_the_issued_challenge_to_session_epoch_and_device`; do not turn their network mocks into a live-provider claim.

Add only these two meaningful functions inside the same integration target:

- `retained_provider_tag_requires_canonical_local_and_reverification`: issue/verify a real existing local JWT, verify its retained canonical tag and protected success, close/reopen with explicit local and then default local using unchanged other configuration; both must accept the same tagged proof. With the otherwise valid session/user/epoch/device/freshness held fixed, replace just the stored provider with four bounded variants: remove field, JSON null, one fixed unknown string, and the other canonical Google kind. Typed deserialization for the first two must yield None; every variant must produce the existing protected unmet-authentication code while the plain control remains usable. Restore/reverify with a new valid same-device local challenge, prove writer-created local tag and protected success. Keep original different-device/replay/TTL cases intact; do not reset device binding/session/epoch to make the case pass.
- `retained_provider_transitions_require_corresponding_successful_verification` inside `google`: one local→Google→local lifecycle using normal close/reopen and existing synthetic Script helpers. Use the same fixed synthetic device ID for successful proofs so the unchanged sticky-device contract permits re-verification. Before successful Google verification, protected authorization must refuse and plain control must work; one malformed/refused response must leave prior retained proof/provider unchanged. A valid issued/answered/accepted Google transaction must set Google tag and allow protected access, without letting another session use the proof. Reopen valid local; Google-tag proof now refuses; a new same-device signed local JWT sets local tag and allows access. Preserve the original Boom negative unchanged as a separate regression, and existing tests' consumed/replay-retained different-device semantics. No caller-supplied provider field/remote tenant/device is introduced.

This yields a planned13 test functions, not an actual pass count. New failure branches should use fixed assertion/panic labels that do not debug-print successful authorization/token values. Storage/permission/audit oracles must name exact prescribed changes and must never dump snapshot values. Claims timestamps and existing conditional policy are intentionally outside this test target; no changed-claims coverage is inferred.

One future **whole existing integration target** after source/independent review, exact production+test composition, fresh measured headroom and separate root lane release:

```sh
env CARGO_TARGET_DIR="$PWD/target/wave30-o06-readiness" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support --test device_trust -- --test-threads=1
```

This deliberately names the already recorded private target rather than claiming a new cold cache. Root must verify the chosen performer/cache/toolchain/native prerequisites; this WT's production/tree is not declared aligned to current root. Default features include Platform; test-support enables the existing synthetic transport and all Google test bodies. No fuzzing/alltargets/edition/PG/remote/hardware target is added. Plan a finite owned supervisor, capped0600 private log, closed redacted numeric actual/count/refusal/cleanup evidence persisted before grading, jobs1/inc0/debug0 and >=8GiB floor with proactive stop/reap before it. Exact outer deadline/log/sample/process implementation and fresh capacity are root's later reservation; historical62s/minimum does not guarantee this whole target's future resource use. Nothing here reserves, acquires or releases a runtime slot.

## Body-read coverage and actual static limits

| Fixed1a file | Body-read coverage | Whole bytes / SHA256 (identity independent of coverage) |
| --- | --- | --- |
| `src/device_trust_types.rs` | complete 1–96 | 3188 / `2940b6fa74757e5e25a3761c1bb9cae4303a2f54c0c8c58cad550b66a2803273` |
| `src/device_trust.rs` | complete 1–350 | 12579 / `8524103a534cde6e785f3b272f4b69d28becf462c4b005232058d20fb37b51a0` |
| `src/device_trust_essentials.rs` | complete 1–33 | 1076 / `c5607cf028fe5fcce5acedd5eacdb7d305c64355a2bac59b70122b71b89dc68f` |
| `src/assembly/device_trust.rs` | complete 1–418 | 16633 / `2ad918244800eb6f6c5ce3ab2cdeb8790dc60bbd96e0fcb8de5191819eee688b` |
| `src/assembly/claims.rs` | complete 1–206 | 8461 / `48f9b01cc05eed9f1accb0dc94b3a8990d882dde509fe8c8c71bdb5703b7f3b0` |
| `src/verified_access.rs` | 1–534 and 656–746; parser535–655 and unit tests not full-body reviewed | 36904 / `f79611690c3c28ce6fd0abec4c41252afa417241dfe7b6df46e5dee151d0f973` |
| `src/claims.rs` | 1–207 and 282–343: complete fact-load, fact evaluation, enforcement and identity mapping bodies | 29484 / `b5d5fb73cca939a4489d3a503c672b075c4d185b16ba4d716302ce814ee569ab` |
| `src/model/claims.rs` | complete 1–159 in two reads | 5062 / `1d3cb9156fcbfaf54bfd597aa69fd7c1944fe160840b39c5380dd33775602464` |
| `src/core.rs` | 190–310 and 1070–1175: complete open/open_store/authorize_identity; adjacent partial bodies | 57913 / `686e7e732f256e7b435ab69991b71fb26d7467214877d2e0f0c840741446caea` |
| `src/assembly/oidc.rs` | 1515–1590 and 1680–1740: issue/live-grant gate and claims call spans; not whole file | 78806 / `6747207d82683c9d3844798ac350a80fa8058ad3a52c5aa6be7691e9b2e20dde` |
| `tests/common/mod.rs` | complete 1–230 | 7847 / `b9672e8e13e956df123489c6962ae6306016415af7c9f9f9e325ea8f6f5367d6` |
| `Cargo.toml` | complete 1–91 | 4020 / `58e5ef824ed96290179c9f76fea208dc37173caeee21b6ce8d37f8a6dd1abcb8` |

Full6ba `tests/device_trust.rs` was separately read1–1829. `tests/identity/policy.rs`1–172 includes the complete conditional-policy fixture and only the next test opening; `docs/oidc-profiles.md`77–116, `src/node_security.rs`1–172, `src/config.rs`1040–1115 (complete bounded private-secret reader), `src/recovery.rs`1–89, `src/edition.rs`288–326, and `src/identity.rs`46–105 are selected context spans, not full-file audits. The earlier I07 author plan's source/negative/historical-evidence/context and coverage sections were read selectively; one large display was clipped, so this report does not claim full old-report review or independently reexecute its accepted historical10/10/Essentials/wipe results. Source search/comparison is not a historical log, SDK, artifact, live-provider or execution-body result.

A first unquoted `tests/claims*` pathspec caused a zsh no-matches read/search error; the quoted follow-up succeeded and the relevant fixture was read. One combined source/old-plan output was clipped; decisive complete assembly, record, policy, Google fixture and regression bodies were read in bounded calls. These are local source-discovery limitations, not product failures. No raw private panic output was read to compensate for the old parser limitation.

Actual checks so far: JSON syntax/full allowed bodies and four regular-file modes/hashes; exact original export row selection/hash; actual-vs-fixed/test equality and Core confined diff; all proposed anchors unique and four whole-byte inverses; no source/helper/config file materialization/execution. Check-docs/hygiene/whitespace/scope results are appended after their actual invocations. Source checkers are byte-equal to fixed1a (docs SHA256 `925418bb155b59554666603efce4af16160ee5e00df9c409d46a3381b390ae82`, hygiene SHA256 `a3b6855473e0d68308d0f7691bb2ccff766b49f21abd68d6c766e75725d31ece`). No Cargo/compiler/SDK/native/version/key parser/provider/HTTP/test/harness/browser/Driver/network/import ran. All previous reports and failures remain untouched; no current allgreen, corrective pass, slot release or whole-I07 closure is claimed. Root alone owns exact source reservation, release, integration/publication and disposition; RiWork Cua.ai Driver MCP-only preference remains for any separately authorized desktop action.

## Actual report-only handoff checks

- `python3 scripts/check-docs.py`: exit0, Markdown links and build-directory layout checked.
- `python3 scripts/check-repo-hygiene.py`: exit0, tracked-file hygiene checked (1065 indexed files before adding this report).
- Initial `git diff --check`: exit0; tracked unstaged and staged changes were empty; exactly this new report was untracked.
- Independently decoded the report's entire zero-context diff as DATA (SHA256 `14a2a9f80afc60b33412cd70244daf4e2455f26fc7c6f9e4b3743b8a8235636c`), reapplied its insertion coordinates to each immutable input in memory and matched all four candidate hashes. Removing the exact candidate coordinates reproduced every complete input again. No Rust source was written, parsed by a Rust tool, compiled or executed.
- Source witness line-number resample corrected readiness104 and complete Google writer266–326 / bound-challenge328–367 locations in this new report; source/proposal bytes did not change.

Staged/final scope checks and immutable handoff identities follow after their actual checks. No runtime slot was acquired or released.

Final staged checks: docs exit0; hygiene exit0 (1066 indexed files); staged whitespace exit0. Staged change is exactly A for this report, with no unstaged tracked diff or untracked files. Every other indexed mode/blob equals entry `1f04ceb2244e872411684b8f235170bdcef94488`; existing I08/X01/U10/Q06/D01/A09/S02 reports and source/test/config/helper files are untouched. The final commit/report hash is provided in the handoff; runtime and implementation remain HELD.

## 2026-10-03 — independent immutable source19805 review (runtime HELD)

Reservation `wave30_I07_provider_binding_immutable_source_independent_review`, same project/original I07/WT/shell. Entry HEAD remains `9d79f51ba623d1a226de0f83fb15f6d73b6cc375`, with clean tracked/index/untracked scope. The full accepted preceding30039-byte/168-line report, SHA256 `aed978eef04db578a8b45905478e0fb8587f00fa77c577ee6fa98e9bd715b6d3`, is preserved byte-for-byte, including the dated unmaterialized13-function design, failed negative and original parser limits. This appendix reviews the author's **different actual immutable implementation**, not a retroactive rewrite of that proposal.

**Source-only disposition:** no certain production or fixture blocker found for the narrow `require_device_trust` provider-kind correction in source `19805cb9250f3b3d77da4c15b52c313794450143`. The three additional test functions have meaningful separate oracles; source acceptance does not establish compilation, executed assertions or original managed-device completion. No corrective source/test/guide edit is proposed by this review. All runtime remains HELD for root's separate disposition/release.

### Exact parent, delta and immutable byte proof

Source commit `19805cb9250f3b3d77da4c15b52c313794450143`, exact parent `84d3d8a46df6cf9635bd00123592d2d67b573fa9`, source tree `31d713bb4405c5c5372cf0e5b9fe508d64365ead`. The complete parent→source diff contains exactly four modified100644 files, **309 added / zero deleted lines**:3 type lines,11 Platform policy/canonical-helper lines,6 successful-writer lines,289 fixture lines. Thus production additions total20; test additions are one field in an existing literal plus three new functions/attributes/separating lines. There are no guide/manifest/config/Core/claims/epoch/Store/approval/workflow/helper changes in this commit.

| Immutable file | Exact-parent bytes / SHA256 | Source19805 bytes / lines / SHA256 |
| --- | --- | --- |
| `src/device_trust_types.rs` | 3188 / `2940b6fa74757e5e25a3761c1bb9cae4303a2f54c0c8c58cad550b66a2803273` | 3338 / 99 / `9b8a931b68b2a453350904b32f5e6f7a98f3cecb88a0cebde41f7ed3f9803825` |
| `src/device_trust.rs` | 12579 / `8524103a534cde6e785f3b272f4b69d28becf462c4b005232058d20fb37b51a0` | 12917 / 361 / `5b2960a196a678e13fc767d6e662d7a099e8a5da1d3bdb82be01dc6b1878993e` |
| `src/assembly/device_trust.rs` | 16633 / `2ad918244800eb6f6c5ce3ab2cdeb8790dc60bbd96e0fcb8de5191819eee688b` | 16909 / 424 / `95c0a41470d4bf6cd68a6017e308ef090cf282e7f999eaaa4476fca8c2afea7b` |
| `tests/device_trust.rs` | 64683 / `ee9c1676e8fb3aa6f6d2092a343e70bb0beae703d563d3e410ad611093e340dd` | 74157 / 2118 / `f5fc6b3d42f97bf7c4601d8228e713066af1e3a95eeee3aca60dfeeec4c7f5df` |

All three production parent blobs equal fixed1a audited in the preserved report. The complete parent test equals immutable6ba:64683 bytes/1829 lines/SHA256 `ee9c1676e8fb3aa6f6d2092a343e70bb0beae703d563d3e410ad611093e340dd`. The complete source tests are74157 bytes/2118 lines. Reading the whole diff and all new bodies was separate from metadata/hash comparison. `git diff --no-ext-diff --unified=0` of these four exact pins is11522 bytes, SHA256 `ebd25feff10fad52a4f7e5625ea33b2126c170381b6240a818fe399e33dc0699`.

For each file, in-memory line comparison yielded only equal/insert operations; removing every insertion at its source coordinates reconstructed the entire exact parent. Production added764 bytes total, tests9474. Whole-file inversion proves every old line survives, including original predicate/nonce/expiry/device/identity/refusal/audit code and fixture assertions. This is DATA/byte proof and manual source review; no Rust AST parser, compiler, type checker, serde/key parser, test/helper or module import executed.

### All20 production additions and canonical semantics

- `src/device_trust_types.rs:94–96`: one explanatory comment, serde default, `provider: Option<String>`. No change to TrustConfig, Challenge, existing defaults, timestamp fields or serialized record key. Missing/null deserialize as None under the proposed derived type and cannot equal a required Some canonical provider. Unknown string also fails the new predicate. Non-string storage corruption remains a deserialization error, with no promised normal OAuth error code or local fallback.
- `src/device_trust.rs:74–82`: `ProviderKind::identity(self)` is a closed mapping from the existing Copy enum to exactly `local` and `google_verified_access_v2`, both static strings. Unlike the old report's inline mapping, the author reuses this one helper in both writers and the gate. Neither a JWT claim, request body field, device ID, arbitrary stored string nor raw optional configuration chooses the written identity.
- `src/device_trust.rs:222,240`: derive current canonical kind and require exact string equality as an **additional** fresh-record conjunction. All user/session/epoch/expiry/nonempty-device and active-session checks remain. Current readiness still precedes identity/kind lookup and missing/unusable/mixed/unknown configuration keeps the existing unconfigured reason; missing identity keeps its existing session-required reason; unprotected client returns before any config/row read. The new `provider_kind(config)?` follows successful `provider_ready(config)`, whose first step already calls the same classifier on the same immutable TrustConfig borrow. Its classification is pure over ordinary String/Path/Vec fields; no intervening mutation/network/config reload occurs. Thus there is no certain new error-classification bypass from this second classifier call. No unwrap/default-to-local on an unknown current kind is added.
- `src/assembly/device_trust.rs:250`: Local.identity is stored only in the existing successful local verification transaction after pinned JWT/audience/device/nonce/session/user+session epoch/unused/expiry validation and same-device guard. Its two call sites remain provider-kind-gated public `device_verify` or submitted-Local dispatch. Token/JSON client input cannot assign the field.
- `src/assembly/device_trust.rs:315–319`: Google.identity is stored only after bounded material, issued challenge/session/replay, embedded issued SignedData comparison, successful accepted Google response/customer/trust check and the final transaction recheck. Submitted-Google is the sole call site. The preceding different-device rejection still commits consumed/replay-retained challenge state without replacing verification. This five-line formatting does not change control flow, audit or expiration arithmetic.

The full source Platform file1–361, shared types1–99 and assembly1–424 were read. A combined display clipped part of assembly; its entire424-line body was then reread alone. All20 new lines and their guard/writer context are fully reviewed. The writer call-site/record-construction search confirms only the existing two production construction sites and one existing test literal. No provider tag is minted by cleanup, config adoption, claims, epoch change, migration or generic maintenance.

Default config.provider None and explicit local classify to the same Local identity; no raw Option equality is used. Legacy/mismatched proof retains the sticky session/device record and needs successful same-device verification to satisfy the changed protected gate. Different-device verification still requires a new session. Provider origin is a kind, not a full old acceptance-settings fingerprint, transition counter or physical attestation. All earlier current-kind→same-kind settings, local→Google→local restoration-without-replacement, mixed old/new binary and claims timestamp limits remain.

### Three complete new functions, and original negative preservation

The actual fixture has **14 test definitions:11 unchanged original definitions plus3 additions**. The earlier13-function proposal was a source design with two suggested new functions; it was never an execution count or a mandatory count oracle. The author separates compatibility and legacy into different functions, with the third covering both writer directions. No count is inferred by resemblance and no14-pass outcome is claimed.

| Full new function body read | Exact source lines | Body bytes / SHA256 (from fn line through closing brace, LF retained) | Defined oracle and limit |
| --- | --- | --- | --- |
| `google::retained_google_proof_requires_reverification_and_can_rebind_both_providers` |1832–1983|5086 / `b5f9f9fbf72330593ac2e01638ca593e16d9060c683b51f17cc792a17041868c`| Successful synthetic Google writer/tag/protected acceptance; close/reopen current Local requires refusal/plain control; normal same-device local writer/tag restores acceptance; close/reopen Google refuses local row/plain control then normal successful Google writer/tag restores acceptance. All three writer points use fixed same synthetic device ID, preserving sticky-device semantics. No real Google response/device or failed-response mutation oracle is added here. |
| `implicit_and_explicit_local_provider_keep_the_same_retained_binding` |1987–2036|1700 / `67c835a1e9d515b28289c083ff5f1abdd5e7d22af348521ca1d2c28704843581`| Two finite variants: initial default-local proof, reopen explicit local; initial explicit local proof, reopen default local. Each uses real local signature/writer, asserts stored canonical local and protected acceptance after reopening. Same key/config/instance is retained; not key-rotation/config-fingerprint coverage. |
| `legacy_and_unknown_provider_rows_require_actual_reverification` |2039–2118|2621 / `74d99578c79a32139ecc8438baf3fc2a5941627c79adda1b8337b75d3ae54730`| Initial actual local proof, then3 variants changing only provider: missing field, JSON null, fixed unknown string. Each requires protected unmet-authentication error/plain authorization success, new signed same-device challenge verification, stored local tag and restored protected access. Identity/session/epoch/device/expiry are held unchanged; no row deletion/session reset or invented legacy origin. |

Complete original negative `google::retained_local_proof_requires_reverification_after_provider_change`, source1780–1829, body SHA256 `688d225757439f549105b938beefae392663d8f78a50af2cc70846a432a61880`, is byte-identical to its exact parent/immutable6ba body. It still proves local protected acceptance before close/reopen with valid Google config plus Boom transport, then expects protected refusal and unprotected control. The sole literal insertion earlier in the file shifts its source location by one line: source19805 unwrap_err is1822; the actual historic6ba failure at1821 is untouched, not relocated in the old receipt. The original11 test function bodies were independently compared; all are exact except the one extra `provider: None` line in the deliberate planted legacy/unconfigured literal. No old assertion, snapshot exclusion or fixture outcome was weakened/removed.

The full original11 fixture bodies were reused only after proving parent equals the already fully read6ba object and checking their function-body preservation. All3 new bodies, original negative and whole changed region1769–2118 were read from source19805 with line numbers; sole old-literal adjustment was fully read in the exact diff. The Google module retains its original Platform+test-support feature guard; the two new local cases are ordinary top-level tests. The retained helpers remain the same private synthetic ES256/service-account/queued-reply fixture bodies. TempDirs/account/config/key lifetimes survive each same-instance close/reopen; no source-visible lost fixture directory or mismatched device ID is used to manufacture a refusal.

Defined coverage is bounded: provider mismatch in both directions; successful re-binding through both actual assembly writers; default/explicit alias equality; None/null/unknown legacy failclosed and successful reverification; unprotected controls in the transition/legacy/old negative cases. The new lifecycle does **not** include a failed-response-before-rebinding record snapshot or a new cross-session variant; existing unchanged tests define challenge/session/epoch/replay/TTL/different-device/endpoint/audit boundaries, and this implementation leaves their code/guards intact. No fresh pass for any such assertion is inferred. The legacy/epoch raw-JSON test continues to omit provider, so its refusal now also has the new missing-provider condition; its old epoch guard remains byte-exact, but this unchanged fixture is not newly claimed as an isolated epoch oracle after schema addition.

The new negative branches retain unwrap_err, so an unexpected Ok could format synthetic authorization contents into Cargo's failure output, as the original negative did. The earlier report's suggested fixed-label error branching was not materialized. No production/public response leak is inferred from this test mechanism; any separately released runtime must keep raw output private/capped and publish only reviewed redacted fixed classification. This review did not open raw Cargo output or execute a failing case and supplies no runtime output-privacy pass.

### Claims, configuration, guide and security boundaries

Identity comparisons show source/parent/fixed1a equality for `src/assembly/claims.rs` (SHA256 `48f9b01cc05eed9f1accb0dc94b3a8990d882dde509fe8c8c71bdb5703b7f3b0`), claims engine/model, Google adapter (`f79611690c3c28ce6fd0abec4c41252afa417241dfe7b6df46e5dee151d0f973`), Essentials stub, common fixture, config, node-security and identity module. Those previously reviewed source bodies/selected spans and exact equality support reuse of their semantics; this appendix does not promote partial prior reads into whole-file audits. The new shared field alone changes typed deserialization; the timestamp reader still checks current user/session/epoch/verified_at>0 and<=now/unexpired record and returns approval time. It does not compare provider. ApprovedDevice conditional policies/mappings without the separate require-device-trust flag remain the explicit source-derived boundary in the preceding report, not a corrected or executed policy here.

Same-kind local key/kid/JWKS or Google account/key/domain/customer/allowlist changes remain separate acceptance policy, unmeasured. Current-kind equality does not fingerprint those settings or mutate shared agreement/epochs. Both success writers retain existing challenge/replay/expiration/session/device/audit checks and transaction timing. Missing/mixed/unusable configuration, signature/input constraints and unprotected policy are preserved. Claims/approval/config/Store/epoch writers were not broadened.

`docs/enterprise/ENT-06.md` is not among the four paths: source equals parent (and prior fixed1a), SHA256 `16ddfb51e8579a4dab74d49dc6c4b486eb42e5f633271daa71afae29cf5fee02`. Its coordinated guide work is still author-held and is neither integrated nor reviewed as an added19805 change. This review does not edit/import/approve that separate guide or call it already complete. CONTRIBUTING/SECURITY/checker objects equal the previously read/checked pins; applicable guidance is unchanged and no AGENTS.md was found for this report path.

### Actual checks and held limits for this appendix

Actual static/data checks: exact commit/parent/tree/four-path/mode/numstat identity; whole four-file inverse; old11 test-body preservation except exact literal field; complete new function bodies and source locations/hashes; original negative body identity; protected adapter/claims/config/guidance/checker comparisons; immutable source diff whitespace exit0. No product/test source was imported into this WT and no Rust parser/compiler/typecheck/import/serde/SDK/provider/native/helper/function/test/harness/browser/Driver/remote command ran. A clipped combined display was corrected by rereading assembly alone; no source failure was hidden or converted to runtime evidence.

The prior negative remains Cargo101/0pass1fail10filtered with later error/unprotected checks unreached; all original redacted receipt/parser/cleanup statements and bytes survive unchanged. No private raw panic, credentials, new actual receipt, artifact or source-worker communication was read/requested. No lane was acquired/released and no original I07 primary/assignment/status or closed row was touched. Root alone owns compilation/runtime release, review, integration/publication and original-scope disposition. Source acceptance is limited to this narrow provider-kind gate, with the guide and managed-device prerequisites separate.

Docs/link/hygiene/prefix/append-only scope checks are recorded below after their actual invocations. Source and runtime remain distinct; original external managed-device acceptance is not established by14 definitions or this report.

Actual appendix checks: `python3 scripts/check-docs.py` exit0 (Markdown links/build-directory layout); `python3 scripts/check-repo-hygiene.py` exit0 (1066 indexed files); report diff whitespace exit0. Exact accepted30039-byte prefix hash resampled unchanged. Only this existing report was modified, with empty index and no untracked files before staging. No source or guide was edited, and no runtime slot changed. Final staged/scope/clean evidence and immutable commit/report identities are supplied in the handoff.
