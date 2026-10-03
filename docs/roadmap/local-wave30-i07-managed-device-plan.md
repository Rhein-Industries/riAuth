# I07 managed-device provenance: source audit and one bounded proposal

Date: 2026-10-03. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original I07 task `34688b10-fa4b-4b83-8070-adfb3e55dc41`. Reservation `wave30_I07_managed_device_provenance_source_audit`, existing WT `7c85f5ef-3fac-4f72-aaed-08474d7fb454`, shell `7a798ec7-bad2-4d52-8bea-afbe1f8e8d2a`.

**Recommendation to root:** reserve one local regression for an already documented missing rotation contract: retain a fresh locally verified session, close Core, reopen the same instance with the valid Google provider, and require fresh Google verification before a device-protected client can authorize. The source currently has no stored provider identity to compare. This is a source-derived reachable risk and an unexecuted proposed negative oracle, not an observed failing test, an implemented correction, or physical-device evidence. Root must adjudicate the expectation before assigning the test/product slice. I07's original managed-device acceptance remains unproved; task/assignment/status stay unchanged.

## Authority, immutable inputs and boundaries

Audit source is exactly published `544d1340b80cd3e040dc13142cdcbc1d75fea4cb`, tree `a995971ec6829d41086f236b595c1493a2be0de0`, parents `6a4066c72ac58225cd444a806032eef8515e646b` and `e2e1cde2c4daaeb758b21d4208a5e9381caaa245`. Own clean starting HEAD is `66d8082ea95d2c7dc232efa0661e8dc7438a92bd`. There was no alignment, source merge, or main update.

Before the source audit, the complete original UUID row was read from project `planning/current-tasks.json`: 244354 bytes, SHA256 `0b38ca3eb8810a4628436eb5083f6329de848f3f4ddaa972c850601ad45c0835`, 94 rows, exact row index47. The end-of-read resample had the same hash. Canonical sorted compact UTF-8 JSON of the complete row is SHA256 `a1bbe7e4b68fdb42a400e9dfea996acb5ee8594737d9edd21741301153f1519d`. It records `todo`, original WT `d0cebec9-5552-486a-88c6-9a1eb10e4a45`, created1790534564/updated1790894718, P2/phase3 and A06/Q01/Q02 prerequisites. No task field was changed.

The exact requested outcome is: “Verify signal provenance, freshness, and account/session/device binding against supported managed devices.” Its workstream requires each advertised integration's setup, lifecycle and failure handling. The record also requires relevant source/tests/docs/artifact evidence and actual verification, rather than closure from a report. Its old scheduling text grants no worker launch; the current explicit bounded support assignment authorizes this report only and leaves original ownership/status alone. Model-effort guidance does not change a running worker.

`CONTRIBUTING.md` and `SECURITY.md` were read at the fixed pin before audit. Project coordination guidance `/.agents/skills/riwork-orchestrator/SKILL.md` under the same project directory was read in full (7324 bytes, SHA256 `9ac80913316a4cde11ca60f2c97876b94f3b02db267c5ab273c8492744a36d58`); no orchestrator action or delegation workflow was invoked. No applicable AGENTS.md was found in this WT/ancestors. Security findings remain in this local report; no external disclosure/query occurred.

Only this new report is writable. No product/helper/test/config/existing-document change or resource acquisition/release occurs. A09 ARM37101183416 owns validation/Cargo; runtime remains HELD. Future desktop work remains RiWork Cua.ai Driver MCP only, with fresh descriptions/state and explicit release. No D01 browser or source work is duplicated.

## Accepted evidence, dated rather than rerun

The fixed `docs/roadmap/local-wave28-task-closure-audit.json` I07 object and its complete ledger430/432 entries were read, as were the fixed coverage-inventory I07 object and current actionability I07 object. Large JSON containers were parsed as data to select these objects; their other row bodies were not reviewed. Current project `planning/accepted-commits.json` was independently read as data for the matching public entries: 637887 bytes, SHA256 `47e7622cd684ccaa2c5de794a2e69b4d71b0f736afa5f64949cd6b9132b27d55`. `local-wave28-integration-progress.json` was parsed/searched (76565 bytes, SHA256 `92c080c481c326a16d6eb6c934db5e198339a47c15264905877b62086e2ca551`); it added no named I07 execution body. No worker was contacted.

| Accepted source witness | Actual historical result recorded by accepted ledger | Limit |
| --- | --- | --- |
| `cf6592783d9be7aec1e14ffb33e2bfab9bc60f52`, `67b892d78d60bf7c3e60bbf50ba67c7005ee4bd9`, `09db56d9bf86007716aedcec783b5ebeafa33b87` | `device_trust`10/10; Essentials check passed with warnings | No live Google tenant/managed Chrome; later fixture correction cleared hygiene |
| `33b996ae41429417ed9da3fd2be499d09d0cc274` | focused wipe unit1/1; docs/hygiene/module/diff checks passed | Wipe fixture only; no live provider/device |
| Windows adjacent accepted ledger203/205/241 | dated Windows suites8/8,9/9,9/9 and identity boundary6/6, edition checks | Protocol/assembly proofs, no signed VM/LogonUI/hardware |
| Windows adjacent ledger399/422 | credential-exposure exact1/1; later Windows11/11 and focused CLI1/1 | Broader historical CLI16/17 failed on removal confirmation; preserve that failure, not a current rerun |
| I08 adjacent ledger122/137/160 | DeviceHost/CP/signed-bundle source accepted | Real Windows installation, DPAPI/ACL, signed artifacts and secure-desktop/LSA remained outstanding |

The four full I07 hashes above resolve to `commit` objects and are ancestors of fixed544 (each ancestry check exit0). Their commit messages/statistics were read for provenance. Current full implementation/fixture bodies were read separately below. Historical source shorts `f5bbf6b`, `93feb1d`, `9b479ea`, `a4f3b46` are ledger provenance only; no claim of full historical patch/log review is made.

The inventory's recorded named executions are the seven local `tests/device_trust.rs` tests plus three `google::` tests: `fresh_local_verification_allows_policy_and_issuance`, `forged_replayed_expired_and_mismatched_signals_are_rejected`, `missing_verifier_fails_closed_and_default_flag_does_not`, `jwks_verifier_rejects_an_unknown_kid`, `trust_is_bound_to_one_session_device_and_proof_lifetime`, `legacy_unbound_trust_and_mismatched_epoch_fail_closed`, `portal_and_proxy_require_the_originating_sessions_live_verification`, and Google's `google_verified_access_v2_binds_the_issued_challenge_to_session_epoch_and_device`, `google_verified_access_v2_fails_closed_on_replay_identity_and_endpoint_contracts`, `google_verified_access_v2_config_rejects_untrusted_endpoints_and_weak_trust`. It also records `verified_access_service_account_path_is_resolved_beside_the_config` and VA unit definitions `verify_body_requires_customer_device_and_allowed_trust`, `cleartext_and_unpinned_urls_are_refused`, `embedded_signed_data_binds_the_response_to_the_issued_challenge`, `raw_service_account_json_and_request_credentials_are_wiped`. These names/bodies and accepted aggregate results are evidence records, not tests run by this audit. Original stdout, full historical command flags and exit receipts were not present in the selected accepted records, so they are not reconstructed or invented.

`tests/m03_windows_device_e2e.rs::riauthctl_windows_device_is_one_service_with_the_server_cli` is an ignored real-local-server/client definition with private credential output and enroll/revoke/login/refusal/audit assertions. Its CI invocation is a source definition. Reading it supplies no fresh result or Windows enrollment/OS logon. The same distinction applies to Windows `SelfTest.RunAsync` and `windows/tests/test_manifest_binding.py`.

The fixed actionability disposition still requests an authorized managed Chrome/ChromeOS enrollment and Verified Access project/policy inputs. The dated inventory's old release/tag observations are not automatically refreshed to fixed544, and no current release/CI/dependency/build/hardware success follows from object identity.

## Client/server provenance and enforcement map

| Boundary | Source behavior read | Evidence class and practical limit |
| --- | --- | --- |
| Local default provider/key | Explicit/default `local`, bounded pinned PEM/JWKS, supported alg/kid, reject token jku/jwk/x5u/critical headers, configured riAuth audience/exp/nbf | Software-key JWT stand-in. Tests generate ES256/P-256 and RSA software keys; no device enrollment or attestation guarantee |
| Local challenge and binding | Logged-in bearer; hashed nonce, account/session/epoch, outstanding cap8, challenge<=120s/session expiry; successful transaction consumes nonce and binds one device to that session | Wrong session/epoch/key/audience, replay and expiry definitions cover refusals. Trust lifetime=min freshTTL/proof exp/session expiry. Same session cannot switch device, including after freshness expiry |
| Google provider credentials | Explicit provider rejects local verifier settings; private bounded regular secret descriptor/mode check; PKCS8 RSA service account; fixed RS256 assertion/scope/audience and token-cache fingerprint | `read_private_secret` checks regular/size and Unix owner-only mode; it does not establish physical custody/Windows ACL/TPM or add a nofollow/UID guarantee |
| Remote transport | Only pinned HTTPS Google token/generate/verify URLs; no ambient proxy or redirects, bounded bodies, 5s connect/10s request; request bearer/body/raw key JSON wipe boundaries | Production network code, unexecuted here. Memory/HTTP-client copies and allocator history remain documented limits |
| ChallengeResponse | Canonical bounded base64/SignedData parser; exact embedded issued data+signature comparison, matching proto2 duplicate semantics | Device signature/public-key provenance delegated to real Google verify; encrypted key info is not locally decrypted/verified. Test outer signatures are repeated bytes and replies are queued JSON, not attestation |
| Google policy evidence | Successful response requires `devicePermanentId`, configured `customerId` and operator allowlist within fixed acceptable trust levels; expected enrolled-device domain sent; profile-only/unmanaged shape/refused levels fail | `deviceSignals` are deliberately not evaluated. Allowed `CHROME_BROWSER_OS_KEY` is not a universal physical TPM requirement; configured HW level needs its own actual provider evidence |
| Google freshness/one-use | Issued_at nonzero and <60s, current session/account/epoch, unused issued challenge, response replay hash retained1min, transaction rechecks after remote result | Different-device response is consumed/replay-retained without changing prior verification; new session required. Final trust TTL is min configured freshness/session; remote minute is distinct |
| Access/refusal | `authorize_identity` calls trust gate; token issuance/live grant snippets reach same gate; protected portal/proxy definitions use originating session; username-only explanation cannot assume trusted session | Missing/unloadable verifier, inactive/epoch-mismatched session, missing/expired/unbound record refuse. Self-contained JWTs already issued elsewhere and copied bearer sessions have documented different limits |
| Browser boundary | Device-code page and shared decision writer require live SSO/review ref, same account/code, fresh authentication and current client policy; explicit POST decision only | OAuth device grant approval is separate from managed-device attestation/enrollment. No Chrome enterprise challenge responder was located in searched src/crates/windows/tests; no D01 browser claim borrowed |
| Windows enrollment | Exact `device.enroll` authority, admin-target agent refusal, shared revision/idempotency/credential-once receipt, device/user cap32 transaction, hashed secret, re-enroll invalidates prior tickets | Operator enrollment returns a random shared secret. It is not remotely attested TPM/provider provenance. Accepted receipt/header/exposure/PAM protections remain unchanged |
| Windows online proof | Device secret plus bound username/password+OTP or <=300s same-user reauthentication, shared lockout, one-use hashed ticket300s, live device/user/epoch at redeem; failed redeem still consumes | API protocol tests prove these synthetic paths, not physical Windows enrollment, local account custody, or secure-desktop logon |
| Windows client/provider | HTTPS/no redirects/cookies, response identity/expiry/epoch validation, no offline fallback; DPAPI machine secret and restrictive ACL source; enabled local SAM SID pin/re-resolution; CP invokes absolute host, exact bounded approval record, then separate local Windows password serialization to Negotiate | Full native/managed source read. Fake HTTP/MemoryStore self-test never exercises DPAPI/ACL/Windows SAM/LSA. No Windows toolchain/provider/VM executed |
| Windows signed delivery/removal | Same-byte signed manifest decode, selected signer/hash/version/architecture pin, four-file bundle, pending-update journal/rollback/floor and explicit downgrade audit; remote revoke/local purge before uninstall | Full scripts read; manifest-source definition parsed only. Authenticode, installer update/recovery, CP interaction and artifact validation remain external Windows inputs, not I07 completion |
| Certificate adjacency | ENT-05 separates explicit user certificate binding and CA verification from device health; header authority needs trusted immediate peer, CRL/OCSP limits explicit | Guide read only; mTLS/RADIUS production modules were not fully audited. Certificate possession or a source-defined certificate fixture cannot be credited as managed-device provenance |

Relevant source line witnesses: `src/device_trust.rs:195` policy gate; `src/device_trust_types.rs:86` stored record; `src/assembly/device_trust.rs:193` local verify and `:266` Google verify; `src/verified_access.rs:235` response policy, `:610` protobuf parser; `src/core.rs:207` reopen and `:1096` identity authorization; `src/assembly/claims.rs:42` approval-age fact; `src/management.rs:920` enrollment and `:3011` credential-once envelope; `src/assembly/windows_login.rs:225` online transaction; `windows/RiAuth.CredentialProvider/CredentialProvider.cpp:513` serialization.

## One proposed local seam: stored-provider transition

`DeviceVerification` stores only device_id/user_id/session_id/epoch/verified_at/expires_at. Both provider writers emit that same type, without origin/provider/policy evidence. `policy_reason` checks current provider readiness, then those record/session fields; it cannot distinguish a still-fresh local-software proof from a Google-verified proof. `approved_device_at` also returns the stored age without provider identity. `Core::open_store` preserves the instance session store; its startup agreement covers issuer/active capability names/token lifetimes/password history/effective HTTP rates, rather than device-provider settings. The device dependency check validates that the new verifier is usable, rather than clearing/rebinding each prior proof. Current valid local→Google configuration change leaves the compiled/enabled capability set unchanged.

This establishes the concrete source path to test: a valid locally verified session is retained through an ordinary same-issuer reopen configured for Google; before its <=300s freshness expires, a protected authorization finds the old record and current Google readiness, without running Google verification. There is no direct row planting, weakened verifier, changed auth policy, HTTP bypass or live-tenant assumption in the proposed case. Its predicted acceptance is a source inference, not a measured result. The accepted `docs/security/invariants.md` RI-DEV-001 explicitly says: “Verifier rotation semantics for already stored proofs need an explicit test.” Root can decide the appropriate re-verification/cache-lifetime contract; this report does not silently add all-setting agreement or runtime policy reload as a gate.

**Exact prospective reservation:** only `tests/device_trust.rs`, one new function in the existing feature-guarded `google` module, reusing its existing software account/key/Boom helpers. No production/schema/config/helper/browser change is proposed at this stage. Expected refused result is the existing `unmet_authentication_requirements`; the unprotected control must remain usable. Future source authorship and runtime both require separate root reservation. Existing10 tests are not rerun as a campaign.

The following is the complete unmaterialized, uncompiled, unexecuted candidate hunk. Old fixed test: 62736 bytes/1777 lines, SHA256 `2287bfff16bac53e096652ac9760342577f0a46aaf3f5e714d064a6947559e8f`. In-memory candidate: 64695 bytes/1830 lines, SHA256 `149795ac2fdcc3f0888ba2974089cfe47a48a92b165d9fe0cffefea110ca76c4`. Added bytes SHA256 `9a7bbe4f55dcb66c8175553f0005ae31791adac08943a64235da3082470aec63`. Removing exactly those 1959 inserted bytes before the final module brace reconstructs the entire old file byte-exact. No Rust compiler/parser/test/function was invoked; byte inversion is not compile or semantic proof.

```diff
--- a/tests/device_trust.rs
+++ b/tests/device_trust.rs
@@ -1776,0 +1777,53 @@
+
+    #[test]
+    fn retained_local_proof_requires_reverification_after_provider_change() {
+        let account_dir = tempfile::tempdir().unwrap();
+        let (pkcs8, _) = materials();
+        let account = write_account(account_dir.path(), &pkcs8, |_| {});
+        let mut f = Fixture::new();
+        let (private_pem, public_pem) = super::es256_pair();
+        f.core.config.device_trust =
+            Some(super::trust(f._dir.path(), &public_pem, "local-device"));
+        f.client("app", false);
+        f.client("plain", false);
+        enable(&f, "app");
+        let session = f.user("alice");
+        let issued = f.core.device_challenge(&session).unwrap();
+        let proof = super::sign(
+            &private_pem,
+            "local-device",
+            &super::device_claims(
+                &f.core.config.issuer,
+                issued["challenge"].as_str().unwrap(),
+                now() + 600,
+            ),
+        );
+        f.core.device_verify(&session, &proof).unwrap();
+        assert!(
+            f.core
+                .authorize(&session, f.request("app", &crypto::random_token("")))
+                .is_ok()
+        );
+
+        let Fixture { _dir, core, admin } = f;
+        let mut config = core.config.clone();
+        drop(core);
+        config.device_trust = Some(account.config);
+        let f = Fixture {
+            _dir,
+            core: riauth::core::Core::open(config).unwrap(),
+            admin,
+        };
+        let transport: Arc<dyn VerifiedAccessTransport> = Arc::new(Boom);
+        f.core.install_verified_access_transport(transport);
+        let error = f
+            .core
+            .authorize(&session, f.request("app", &crypto::random_token("")))
+            .unwrap_err();
+        assert_eq!(error.code, "unmet_authentication_requirements");
+        assert!(
+            f.core
+                .authorize(&session, f.request("plain", &crypto::random_token("")))
+                .is_ok()
+        );
+    }
```

After root confirms this negative oracle, reserves that file and later releases the sole validation lane, the proposed single command is:

```sh
env CARGO_TARGET_DIR="$PWD/target/wave30-i07-provider-binding" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support --test device_trust google::retained_local_proof_requires_reverification_after_provider_change -- --exact --test-threads=1
```

Future resources: one private target/job, existing pinned lockfile/platform+test-support definitions, no listener/service/desktop or live Google. Existing dev OpenSSL software-key primitives are reused, not custom crypto. Root sets a finite build/test allowance (proposed<=20min), fresh capacity>=8.5GiB with8GiB floor and own-process cancellation/join; retain actual numeric exit/first assertion failure before grading, stop with no auto-rerun. Fixture TempDirs own store and software account files; drop all Core handles before reopen and remove only owned fixture resources. No shared cache deletion. A failing negative oracle proves only this retained-provider mismatch; remediation would need a separately reviewed production/state slice. A passing result would need explanation from actual output/source, not fabricated retrospective attribution.

## External managed-device input that remains regardless of local result

Real I07 provenance still needs ONE explicitly authorized supported managed Chrome/ChromeOS device in an enrolled organization and an authorized Google Verified Access project/service account whose domain, customer and chosen trust level agree with private configuration. It also needs the actual supported client mechanism to obtain the issued challenge response, the enrollment/policy evidence for that exact device/provider/key class, and an authorized bounded environment for live generation/verification. None was supplied or inspected. Raw service-account/device identifiers/responses/bearers belong in private evidence, with public output limited to pinned source/artifact/provider identity and fixed success/refusal/freshness/binding booleans/counts. This is an exact missing input, not a new worker/runtime request or a universal requirement to test every device/vendor.

The supported client responder/tool/version/source is a required named input: this repository's OAuth device page is not that responder, and handframed SignedData cannot substitute. A real HW/TPM claim needs its actual platform/provider evidence if that selected trust class is advertised; the source search found no separate TPM enrollment/attestation implementation/fixture under src/crates/windows/tests. Windows x64/.NET9/MSVC/Windows SDK/PowerShell7.4+ and authorized signer/signed bundle/isolated VM are distinct I08 physical-host inputs; no API denial or local macOS protocol test supplies them. No external provider/network/toolchain query, install, artifact hash refresh or physical-device action occurred.

## Review coverage and object identities

Full-body reads below mean the entire listed fixed file body was actually displayed/read, with clipped portions re-read in bounded chunks. They do not imply that a separate historical commit body, compiled dependency, released artifact or execution log was reviewed. All Git object identities below are blob identities at fixed544; the fixed revision is a commit/tree identity above. Total full-file body coverage: 39 files. Metadata hashing was performed separately from body review.

| Full body read | Bytes/lines | Git blob | SHA256 |
| --- | --- | --- | --- |
| `CONTRIBUTING.md` | 3603/48 | `64708527ea0d85b741c5d8ffe88d4955a734bb1f` | `7e7dd7b756f734a8105cad5b96dffa8a51977c64f3ecd70b8182fa3de6ba6737` |
| `SECURITY.md` | 981/15 | `047208fb72e97fc942fe5d4d988b162c28f80d5c` | `2556771d58d09a2a4754e7484645b7e948b84286ef0d21cc66169b920a31c4d8` |
| `docs/enterprise/ENT-05.md` | 7835/87 | `fd03682c4835042ec0ad11bb580e8a8ca5ccdcc5` | `8bab3b344e1e910f9ac2d4b8645ebd09c72fa1b3940918e534e4d63b413f3b18` |
| `docs/enterprise/ENT-06.md` | 13373/92 | `f71d5fdbc33591d02a1c5f9daa5bf6621c45a6fc` | `16ddfb51e8579a4dab74d49dc6c4b486eb42e5f633271daa71afae29cf5fee02` |
| `docs/enterprise/ENT-13.md` | 12478/147 | `683f9622ccde30d59169f3f80c5afde3f20faa57` | `2baa25692482d0b1ffa33843ac059b543f25ad16fffc4fd91f673c9be71570be` |
| `windows/README.md` | 7961/136 | `a1bfd672184b447c94cdd61b1204520176b010e3` | `9b24f6ef2aad62fbc19515a9f1ca494f838739d35583f848af667c17fb61909c` |
| `windows/RECOVERY.md` | 5491/86 | `69ca6c34c29f777f93ac83fe27b00f2db14a8922` | `f6eb039b1ab73d73502cb5c26e470c70f173a445582ced590a3a8f3b5f4d9f9e` |
| `src/device_trust.rs` | 12579/350 | `6c1ff333817a07b341101f006664bcde4f2243ab` | `8524103a534cde6e785f3b272f4b69d28becf462c4b005232058d20fb37b51a0` |
| `src/device_trust_types.rs` | 3188/96 | `0a41009553bf5c2a0e30901c3a6d5ab4b6b3658f` | `2940b6fa74757e5e25a3761c1bb9cae4303a2f54c0c8c58cad550b66a2803273` |
| `src/device_trust_essentials.rs` | 1076/33 | `f6cd697fcab0d782bfc949a3faa109977624f08e` | `c5607cf028fe5fcce5acedd5eacdb7d305c64355a2bac59b70122b71b89dc68f` |
| `src/assembly/device_trust.rs` | 16633/418 | `47f092dc99d0a0529b37effd440eb756dd068a04` | `2ad918244800eb6f6c5ce3ab2cdeb8790dc60bbd96e0fcb8de5191819eee688b` |
| `src/verified_access.rs` | 36904/1068 | `a4f87182c2112d63c44caac96c5052f3ed3393b6` | `f79611690c3c28ce6fd0abec4c41252afa417241dfe7b6df46e5dee151d0f973` |
| `src/windows_login.rs` | 8468/278 | `56c3383289e5ba9c2687a9d270f9476c95b9181f` | `8ae057d1fa833abb08d1dfdc8f2ca23444bb62ded891c4f5322eafbb069f9d1f` |
| `src/assembly/windows_login.rs` | 12290/342 | `73e44452184a8f8029f85917289eb5652748606d` | `cacb4fba0e395af7d4cb5bf46786110c1d5969039e3f9e0e7f29f03cf12ef72e` |
| `src/identity/windows_credentials.rs` | 3164/95 | `63607f30471c2ac64f68fb3ad71187a9448ed4a2` | `999518427de03cf6223c02911473d466e799baaf384521f944c27733a447fc88` |
| `src/management/devices.rs` | 7496/238 | `8fb7191c8ade0f47827f96bed95f06a506449242` | `ba2fc1806f91be5b36b4455d29d7ef8b072d06b62a50f22352c1a99a8193f474` |
| `crates/riauthctl/src/windows_device.rs` | 6007/164 | `0f1f2bc13196bd01e03ba3d4c18661be32ac9022` | `5f372b1f02113f5e269b63757ec785f0e1648b52817da2baa7d9b7ce76d72351` |
| `src/portal/device.html` | 6282/87 | `691172e565fc45ada693bad4902880fffc7fb543` | `d00237a12a5dd66fc4b451feb8fb87fa0aefc4ebf5b647f0ee5c6e25742d9063` |
| `src/portal/device.js` | 14557/316 | `cdd807a6d6e454819f69632e5f5f57d5dbf10316` | `e86c105ef05cc11e7eaa66b207a34ffd664adf4259e43522f80c00e5848b2fd8` |
| `tests/device_trust.rs` | 62736/1777 | `25f6bbba250a66ba664e391905d379d1f24e7d43` | `2287bfff16bac53e096652ac9760342577f0a46aaf3f5e714d064a6947559e8f` |
| `tests/windows_login.rs` | 36240/1254 | `207b4ecd16d894a16acf701de0f468aea316a6f0` | `11cf6497412d3ffa7a2689acbb50ea39878435fef8153819ef0b0d4d49d2fe8d` |
| `tests/common/mod.rs` | 7847/230 | `9a40464e4ca311c7c37fe7cee7d5616aeefb73c7` | `b9672e8e13e956df123489c6962ae6306016415af7c9f9f9e325ea8f6f5367d6` |
| `tests/common/security.rs` | 5718/171 | `4c97703230f65fd649c361e2243feb61752981e5` | `ff5c3edf363380c7323d7026a22eb63c50b1e480cf4e3f7996d0de0986164674` |
| `tests/browser_device.rs` | 11068/349 | `309242fd2ede691c2cbfc359a76a05f2f747bd1c` | `3fd494af6f9e4704128f7b023d3b4bc7bcbfe5bf61fd9a4218e5853d6369caae` |
| `tests/device_decision_management.rs` | 8762/296 | `66f169f5bd6b49c66f1961978f914f296ebc794d` | `79bd6750db33c285f9777a7033a45f9416eae185c9764ef6be966cb500cdd395` |
| `tests/m03_windows_device_e2e.rs` | 11560/344 | `9de5757409e5eb93a2163d797b2743522210d0cd` | `e6609616ea4a4155171bd2f1059bc48000b65c1d72ab825d5cfb41dc8d027336` |
| `windows/Install-DeviceHost.ps1` | 82469/1524 | `bbea45b8e67baaf4514300d2c0cdb16170f1ec16` | `b600c2b45a3530de0ea0f34758ea2f4d25dfc0b99182e483aadbbcad75a252ce` |
| `windows/New-DeviceHostBundle.ps1` | 9373/223 | `374b87549c993b6a8163d7b3918c4a594f2eb2c1` | `0794280737cd7e5ea8b9317dfaf3b8ab393892c362554024a0ce4fd595ba04e7` |
| `windows/RiAuth.CredentialProvider/CredentialProvider.cpp` | 29863/729 | `e37f150408f42e4f1335983f9b038a72088e09f3` | `864e7e6bd97244ebe4d2ce807975b217829db53209971c0ad6b998525e037fa2` |
| `windows/RiAuth.CredentialProvider/CMakeLists.txt` | 867/16 | `9cca50b1f41ee03bf6dcc71096dcbb073ec81cdc` | `f580aabe585f5a6454336b941618419ad631e14052186219a56ea0dbb61b246e` |
| `windows/RiAuth.CredentialProvider/README.md` | 2187/36 | `fb58204a6e439c918cf32736e8efa4652bc994bc` | `c3c3334cd2d3a42fe207e910c6dcae8c30114233872ca0180ae328db262e91b1` |
| `windows/RiAuth.DeviceHost/RiAuth.DeviceHost.csproj` | 795/19 | `d27795f758fe39bad7e8723f1ce63914c8d57afb` | `e5af8331053b1167d6af93ea9ad657a0983e694fa5bb71f8f003370fbed499ea` |
| `windows/RiAuth.DeviceHost/DeviceHost.cs` | 11791/230 | `1b9adc281aa2414a698558663e8b78a2f15a4f5b` | `b060bb09bf9fda05e1497d70053d621227dcce6adb0aef1195f0bf0e5f9121ba` |
| `windows/RiAuth.DeviceHost/Program.cs` | 11073/232 | `6e518365a220f7ea3aceb260d12ca3f37f2d0a80` | `59e065d324dd684615aea08f4c7f19551e178208a99fcf691f4d85e4381639b2` |
| `windows/RiAuth.DeviceHost/WindowsStateStore.cs` | 12772/288 | `77938e8ec0383126b7e4bbb18da3259318e1df7d` | `9999747d0183650a109368feb0d11644ff90ad30c23b0d0be6b0ca3a930d9ce0` |
| `windows/RiAuth.DeviceHost/WindowsLocalAccount.cs` | 7223/157 | `34582f8cae84fe6f0552c686e4b3c7d1efe30e1b` | `85641bd7baef27b419899ec76d9665c8d5cfb90b59641bafe5908524763f0d0c` |
| `windows/RiAuth.DeviceHost/SelfTest.cs` | 6105/108 | `16529c0e757bf29d4986ba2d7ad33b59aa7f7969` | `c78dd4496222c58a36e575ddb0bb21be8e8d8904748aeebb8280e12479f08156` |
| `windows/tests/test_manifest_binding.py` | 2559/56 | `326ae280ee5246b4ce413b079e28c7d0dbec91f3` | `faef3f08fc3df94ef67d529dfc279351083b29f199873522ec21d0224fa6ea9d` |
| `Cargo.toml` | 4020/91 | `5660d4bb922fcdc5bfe05d7502585980f4720d06` | `58e5ef824ed96290179c9f76fea208dc37173caeee21b6ce8d37f8a6dd1abcb8` |

Partial-body reads are explicitly bounded; hashing the whole blob does not promote them to full-file review.

| Partial file | Body ranges read | Whole Git blob (identity only) | Whole SHA256 (identity only) |
| --- | --- | --- | --- |
| `src/core.rs` | 1–310; 930–1135 (complete open/open_store, session, identity and authorize_identity bodies; partial adjacent login/audit bodies) | `f206276b9c9e73167f2ac5010e3d2e628b8c0824` | `686e7e732f256e7b435ab69991b71fb26d7467214877d2e0f0c840741446caea` |
| `src/node_security.rs` | 1–322 production agreement/parse bodies; 343–381 and portions of tests displayed by the original bounded read (not the whole test module) | `da029997ad9c1a805cc3c6a1f6a50954f79a16c3` | `979f1d22a4835bce97866e302dc2a8a21df4f0d383d4bd1e6960b6a5d7a3bcb7` |
| `src/capability.rs` | 149–270 (device dependency validation and complete device_trust_usable bodies; partial adjacent client-policy body) | `eed33336bc59c6fcc9baa4379557421d9197e67c` | `6f265647ef12f2d0bb4c6483e251c5b850be50f849b1f1039380886f01532cad` |
| `src/config.rs` | 700–800; 935–1015; 1050–1126; 1240–1267 (device validation/path resolution, complete private-reader/writer and VA-path test bodies) | `0fc0b9a530440c66555eae9d9ee976d1c03549c3` | `4155154a81c70721c3e195733bf17ca1e416e99e2c2e11a69862632c275e8353` |
| `src/api.rs` | 4080–4145, including complete device_trust_challenge/device_trust_verify handlers; route locations searched | `a19ed8e0e875acec096ce55e665f90997a18760c` | `1bbb387dd627f1fd56b7dfe408ea17eef48131f927597da1b81b21ed5f64b8ca` |
| `src/management.rs` | 860–1100; 2998–3060 (complete Windows enrollment, issuing, revoke and credential-once bodies; adjacent snippets only) | `8ff588258fd859949015604518003c5950bcd2ad` | `c3e0fde68ae8bec0706ac306133997272d8602c0319ad54498bba462d25c3b70` |
| `src/jose.rs` | 1–280 (complete production pinned-JWK validation/verifier bodies; partial test module) | `198c20a60bd4bc9b05f8c95ebea3668bdb42a839` | `107f85b6dd9335f8dde1184fe97135d8eb98f5bece1a700a174196b1ed9bf979` |
| `src/assembly/claims.rs` | 1–110 (complete approved_device_at and explain bodies; partial simulation) | `9f91b907cd61b1562b419c0e7bc538c5510ef3f1` | `48f9b01cc05eed9f1accb0dc94b3a8990d882dde509fe8c8c71bdb5703b7f3b0` |
| `src/assembly/oidc.rs` | 1140–1255; 1520–1570; 1690–1730 (complete device review/decision bodies; partial issue/live-grant bodies) | `a62722e78e45f786e3901bd155395c7afc30be7c` | `6747207d82683c9d3844798ac350a80fa8058ad3a52c5aa6be7691e9b2e20dde` |
| `docs/security/invariants.md` | 841–900; 1100–1140 (RI-DEV-001/002 and adjacent capability contract; remainder not reviewed) | `a2b9d3e486136ba05dc52dd037bb6ecd3dc87d2e` | `fc2c12da0bd318a54b0ad23d8b4f16d36cdf4e172257a44ee84121a8ad850f14` |
| `.github/workflows/ci.yml` | 45–112 (test/check definitions only; no workflow execution or current CI claim) | `7c724fd7f4dc210a2268f5a702bdc09b1a10d76b` | `fc5245ef15f0ac8d0a460b71ded533ad96ad3dcbd795b31f38b3e84221940b5a` |

Selected data-object review (not whole-container prose review): fixed coverage I07 `/items/47`, blob `62fb8ec4e9fde65f05f4dc2e3827c286726c8a20`, SHA256 `033965d791d54dbacdbdf329d67534ebd1a045409ed0621ee5a8152b5febfa3b`; closure `/tasks/15`, blob `2b4bb9e53eae492a14ffe3ac9af3238816f64829`, SHA256 `051a159610f904a6e91645bf06a059d3db8bcaec85f95e0e6c0551b233fe058b`; actionability I07 object, blob `b50eb721b7e01f02283dec62a756feaabc9cccc7`, SHA256 `38eb08fc718875846414cadc7085c2c2ed3f01b3f5fe5c194caf2331c6e15310`. Adjacent Windows ledger entries listed above were read in full. Other source hits/ls-tree entries, such as mtls, RADIUS, upgrade, release workflow and unselected test files, are search/identity-only and not full body review.

Case-insensitive substring `tpm` initially also matched `HttpMethod`; that is not TPM evidence. A later bounded word/provider-method search under src/crates/windows/tests returned no TPM/Chrome responder body. Two unquoted pathspec searches failed locally with zsh no-matches; quoted follow-ups used the Git index. One guessed `src/management/windows_devices.rs` read was absent; source search located the real complete Windows writer bodies in `src/management.rs`. Initial ignored-path file discovery returned no result before explicit hidden/no-ignore discovery. Clipped source output was re-read; none of these read/discovery failures is a product/runtime outcome.

## Actual checks and held limits

Actual source/data-only checks: exact row uniqueness/hash resample; four accepted commit object/ancestry checks exit0; whole-test in-memory insertion/inverse byte proof; Python AST parse and in-memory code-object compile of the existing manifest-source fixture, without evaluation/import. No candidate Rust function, existing fixture case, mocked harness, provider/native command, server/CLI, compiler/build/Cargo, browser/Driver or network ran. No existing dependency or all-current-CI/release/HA/device/customer outcome is inferred. Docs/hygiene/whitespace and final scope results are appended after their actual checks below.

Protected own D01 report remains1488800 bytes, SHA256 `ac7b088408854cf77303e10b01e114c9e4bc336ec2811b02217733a176fc47a3`; current D01 helper remains36884 bytes, SHA256 `37d32db6fbd8c2c9b676f691d115ecfd1af893724144361971e86f73f573b406`. No historical private capture was opened or rewritten. Prior refused/failed/unknown cases, unknown sender/cause/lost-value correction and unproved whole60s cleanup remain intact; old memory passes are not borrowed. Authority/review/receipt/header/PAM/held Group/nonrenewed60s/paused IO protections and all closed rows stay unchanged. Root owns source interpretation, exact future reservation/runtime/integration/publication and task disposition. This report recommends no whole-row closure.


### Actual static handoff checks

- `python3 scripts/check-docs.py`: exit0, “Markdown links and build-directory layout checked”.
- `python3 scripts/check-repo-hygiene.py`: exit0, “Tracked-file hygiene checked (1085 files)”.
- `git diff --cached --check`: exit0; staged scope was exactly one added report, no other file. `git diff --name-only` was empty after staging.
- Re-decoded the exact report diff as data:1959 added bytes, full candidate64695 bytes, SHA256 `149795ac2fdcc3f0888ba2974089cfe47a48a92b165d9fe0cffefea110ca76c4`, zero removed source lines; exact whole-file byte inversion passed. This is static reconstruction, not Rust parsing/compilation/execution.
- Both protected D01 files compared byte-exact to starting66d8082; hashes/lengths above unchanged. Existing source/test/scripts/manifests/config are unchanged by the staged tree, and no historical private file was read or written.
- Source line-location resample corrected the new report's approximate locations to the exact function starts before commit. No code or outcome changed.

Final rerun after the report-only additions: docs exit0, hygiene exit0 (1085 files), whitespace exit0; staged scope exactly A for this report and no unstaged diff. The immutable handoff carries the commit/tree/report hash. No test, harness, provider, desktop, Cargo or validation slot was acquired or released. I07 remains the original unchanged task; root retains gate interpretation and future source/runtime ownership.


## 2026-10-03: exact provider-transition negative regression materialized, runtime held

Reservation `wave30_I07_provider_transition_negative_regression`, project891e7443-8dac-4c1b-897f-9e53cb59c7ee, existing WT7c85f5ef only. This is source/static evidence for original I07 task34688b10-fa4b-4b83-8070-adfb3e55dc41. Original assignment/primary/status remain unchanged. Root accepted the narrow contract after the prior dated source audit: a local-software proof is not Google verification; switching to valid configured Google requires fresh verification for protected authorization, while an unprotected client stays usable. That decision does not turn this unexecuted negative into a product pass or a production fix. The entire prior36339-byte report is preserved unchanged; its SHA256 is `975f7be87db7f6316d3e906ace72fc00d916b35cf12f92f2e01af09551f7a59b`.

### Exact source identity and preservation

Source commit `bc2019a2237bad9a894b25b10ec9a273716931b8`, parent `6bc8cf8a958e5875f07a298e6fbb8e66f9845292`, tree `c8395fcbc44c3cc866abce917ad6b043719c39ea`, changes ONLY `tests/device_trust.rs`:53 added lines, no removed lines. The complete1959-byte addition was decoded from the sole archived diff in immutable6bc8cf8; SHA256 `9a7bbe4f55dcb66c8175553f0005ae31791adac08943a64235da3082470aec63`. It was inserted immediately before the final closing brace of the existing `google` module, with no variation or automatic formatting.

| Object | Complete bytes/lines | SHA256 |
| --- | --- | --- |
| Fixed544 test at `544d1340b80cd3e040dc13142cdcbc1d75fea4cb` | 62736/1777 | `2287bfff16bac53e096652ac9760342577f0a46aaf3f5e714d064a6947559e8f` |
| Materialized test | 64695/1830 | `149795ac2fdcc3f0888ba2974089cfe47a48a92b165d9fe0cffefea110ca76c4` |

Materialized Git blob: `488dd0c41b4bfe8dc201c4902595ada7a29f29f5`. Whole-file reversal removed only the exact1959 bytes and reconstructed all62736 fixed544 bytes, not selected function fragments. The pre-edit working test equaled fixed544. The materialized addition, staged index and committed object all matched the approved candidate hash. The strict assertion remains `assert_eq!(error.code, "unmet_authentication_requirements");`; no generic denial or weaker equality substituted. Existing test bodies, imports, features and fixtures are byte-exact outside the insertion. This turn reviewed the entire new function and its exact diff; whole-file identity/inverse checks do not claim a new independent full-body audit of every old test.

`google::retained_local_proof_requires_reverification_after_provider_change` first establishes a local-software proof and protected authorization, preserves that store/session across a Core reopen with valid Google configuration, installs the existing failing in-process transport, then demands the exact protected-authorization refusal and unprotected-client success. This defines an intentionally negative regression. Current source lacks persisted provider identity, as the dated audit and root review already record. The actual test may fail under current product; no outcome is predicted as observed. If protected authorization unexpectedly succeeds, `.unwrap_err()` stops the test before the later unprotected-client assertion: that control would then remain unexecuted, not passed. This test supplies no physical enrollment, live Google/Chrome/ChromeOS/Windows/TPM provenance, HTTP or browser evidence.

### Actual authorized static check and formatting failure

Ran exactly `rustfmt --edition 2024 --check --config skip_children=true tests/device_trust.rs`: exit1. Rustfmt parsed the file and reported a formatting difference at line1782: it would replace the approved two-line device-trust assignment with `f.core.config.device_trust = Some(super::trust(f._dir.path(), &public_pem, "local-device"));`. No parse diagnostic appeared. Because root reserved exact candidate bytes, no formatter mutation/correction or second invocation occurred. The failure remains a formatting-check failure and must not be represented as an all-checks pass; any future formatting change needs root coordination.

`git diff --check` and source-stage `git diff --cached --check`: exit0. Source index scope was exactly one modified test and no unstaged diff; byte/inverse checks passed again after rustfmt. This was Rust parsing/format checking only, without type checking, compiling, running a test or generating a key. No Cargo, existing/candidate case, provider/native/version command, helper/CLI product, server, network, browser/Driver or new target ran. A09 ARM37101183416 retains the validation/Cargo lane; this phase acquired/released no resource slot.

### Existing private-cache inventory and exact later command

Read ONLY own-WT cache JSON and artifact file metadata, without executing any artifact, Cargo or compiler. Own `target/` contains two non-symlink private directories, `wave30-o06-controller` and `wave30-o06-readiness`. Each contains441 fingerprint JSON files and zero `device_trust` test fingerprints. Their cached compiler record says `rustc 1.98.1 (48a229cea 2026-09-01)`, aarch64-apple-darwin; that is retained metadata, not a fresh native/version invocation. Current `rust-toolchain.toml` pins1.98.1. Manifest, lock and toolchain bytes equal the prior readiness source pin `b7bc6efe0f10419e419cdbc16e7d2792b72777cb`:

| Current/prior identical input | Bytes | SHA256 |
| --- | --- | --- |
| `Cargo.toml` | 4020 | `58e5ef824ed96290179c9f76fea208dc37173caeee21b6ce8d37f8a6dd1abcb8` |
| `Cargo.lock` | 109243 | `b5c9d11c001244b8017303ce8c20516e02946483845eee40720b0910758d4426` |
| `rust-toolchain.toml` | 86 | `887f9be066a15585a2c583578e84b0fcb541126d81546276bad3d2ff00d61167` |

Both library fingerprints have features `["default", "essentials", "platform", "test-support"]`, empty rustflags, compile_kind0, rustc identity17329007180185699724, config9396254390672932401 and profile12672335563272108896. Both existing integration-test fingerprints have the same features/config/compiler, profile11094973624911973823 and59 dependency entries; canonical dependency-array SHA256 `93c41b91c846b1633d094745c99c3a30af81f7b349c75a7a55889f87902d46b8`. The default Platform plus test-support matches the proposed focused invocation, including the `google` module's two feature guards. The previous readiness report records jobs1/inc0/dev+testdebug0; current cache metadata does not independently prove a future build's environment.

| Cache record read in full as JSON data | SHA256 |
| --- | --- |
| controller `.rustc_info.json` | `dc8d2a963ab369c5be3389ec5f05cee9d9287f8daa70c31c82dea748e73aa09c` |
| readiness `.rustc_info.json` | `27df402be20083ab5b4835c05762e2b77beed67288686dd8fda9193b24cdd7c4` |
| identical `riauth-0d65ce4b80b71950/lib-riauth.json` | `ac6f00d7f80260b8f443564cbbdd8e64f91d91aa42f8d001c0a23e13778f1c10` |
| controller `riauth-b27819467adb3ed5/test-integration-test-o06_unscheduled_controller_diagnostics.json` | `cb7d02fae0baa52504d37605768d782d529167a9d3a4dacfa853821361e6e6cc` |
| readiness `riauth-e8defbc0b9ac7734/test-integration-test-o06_readiness_cause_signal.json` | `55b3ff104dc15b5942747a65a438013207ceb7f2d1904cb8b84c9023339096af` |

The existing regular, non-symlink metadata-only artifacts in both caches include `libopenssl-43b098cf0be13958.rlib`7629408 bytes and `libtempfile-3606bf3ae9042636.rlib`586488 bytes. `libriauth-0d65ce4b80b71950.rlib` is408115912 bytes in controller and408148624 bytes in readiness. No artifact contents or native code was executed/read as evidence of current correctness. Initial metadata output was clipped by the tool; a concise repeat read captured all relevant identities and fields above. Fingerprints are build metadata, not execution receipts.

Recommend root reserve the existing own readiness cache for ONE later focused command, avoiding the cold `wave30-i07-provider-binding` directory proposed in the dated prior phase. It has matching toolchain/dependency/feature metadata and a previous targeted-test cache, but product-source changes require rebuild and there is no compiled new test. Actual future cache validity, disk capacity, compile success and result remain unproved. Exact proposed command, NOT executed:

```sh
env CARGO_TARGET_DIR="$PWD/target/wave30-o06-readiness" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support --test device_trust google::retained_local_proof_requires_reverification_after_provider_change -- --exact --test-threads=1
```

Root must separately review/reserve this target and runtime after the sole lane is available; fresh disk-floor monitoring and root resource coordination remain prerequisites. No new target was created, cache deleted, artifact/provider pin refreshed or process inspected/signalled. Expected negative failure would be retained with the exact assertion stage and unreached-control limitation before any root-owned production fix.

### Protected scope and candid handoff

The source commit changed no production/schema/config/helper/other test or D01 file. I07 evidence append is a separate report commit. Protected D01 report remains1488800 bytes SHA256 `ac7b088408854cf77303e10b01e114c9e4bc336ec2811b02217733a176fc47a3`; helper remains36884 bytes SHA256 `37d32db6fbd8c2c9b676f691d115ecfd1af893724144361971e86f73f573b406`, compared byte-exact to6bc8cf8. No historical private capture was opened or rewritten. All historical failures, unknown sender/cause/lost values, prior memory limitations and unproved whole60 cleanup stay intact. Authority/review/receipt/header/PAM/held Group/nonrenewed60s/paused IO and closed rows remain unchanged. Root alone owns contract adjudication, integration/publication/runtime and original-row disposition. This source-only regression recommends no I07 closure and supplies no new D01/D05 outcome.


### Actual report-phase checks

After the main appendix was written: `python3 scripts/check-docs.py` exit0 (“Markdown links and build-directory layout checked”); `python3 scripts/check-repo-hygiene.py` exit0 (“Tracked-file hygiene checked (1085 files)”); `git diff --check` exit0. Actual source-tree delta was exactly one test; actual report-phase unstaged scope was exactly the existing I07 report. Complete prior report-prefix equality, committed-test equality/whole fixed544 inverse, and all protected D01/manifest/lock/toolchain bytes passed again. These passing document/scope checks coexist with the retained rustfmt exit1 above; no Cargo/test result is supplied. The final staged whitespace/prefix/scope check and immutable commit identifiers are returned in the handoff. No validation/Cargo/desktop slot was acquired or released.


## 2026-10-03: reserved one-line rustfmt correction, source/static only

Reservation `wave30_I07_regression_rustfmt_one_line`, same project891e7443-8dac-4c1b-897f-9e53cb59c7ee and existing WT7c only. Root authorized ONLY joining the newly added device-trust assignment to the installed rustfmt shape. Source commit `6ba2e95c42c271804d1715eb05380953b008130d`, parent `b05ee4c1f24061fdd22c6bebc42b3dc4e9b950cd`, tree `7357ad7e5a5d6b48c12c4e7abe242861fcd3627b`, changes ONLY `tests/device_trust.rs`:1 added line/2 removed lines. No other assignment, operation, assertion, test body or file changed. Exact test blob is `5c9d9401961033805745335ac353309237e9ba65`.

The entire47036-byte b05 I07 report prefix remains byte-exact, SHA256 `9837c83cb379fc642d716c752f393ebbf59060fa1f4ab215d765903509e70c90`. Its original rustfmt exit1 remains the actual dated failure. This appendix records a separately authorized source correction and new static result; it does not revise that prior outcome.

### Exact correction and qualified reversal

Only this source text changed:

```diff
-        f.core.config.device_trust =
-            Some(super::trust(f._dir.path(), &public_pem, "local-device"));
+        f.core.config.device_trust = Some(super::trust(f._dir.path(), &public_pem, "local-device"));
```

| Complete object | Bytes/lines | SHA256 |
| --- | --- | --- |
| Original materialized bc2019 test | 64695/1830 | `149795ac2fdcc3f0888ba2974089cfe47a48a92b165d9fe0cffefea110ca76c4` |
| Approved formatted test | 64683/1829 | `ee9c1676e8fb3aa6f6d2092a343e70bb0beae703d563d3e410ad611093e340dd` |
| Original proposal addition before formatting | 1959/53 | `9a7bbe4f55dcb66c8175553f0005ae31791adac08943a64235da3082470aec63` |
| Current formatted addition | 1947/52 | `26ec44ca91328e1338d70e2be9394ac2ed153aec4da39725e7c66ad5a31542d2` |
| Fixed544 original test | 62736/1777 | `2287bfff16bac53e096652ac9760342577f0a46aaf3f5e714d064a6947559e8f` |

Both source strings occur exactly once in their respective complete files. Replacing the single joined assignment with its exact prior two-line text reconstructs the entire bc2019 object byte-for-byte. Replacing the same text inside the1947-byte addition reconstructs the original1959-byte proposal. Every other test byte is unchanged. Separately, removal of the complete current1947-byte addition reconstructs the entire fixed544 test directly; no protected old function is excluded from this proof.

The original1959-byte proposal reversal is now explicitly qualified as a proof of the pre-format bc2019 object. It is not the byte length/hash of the current formatted addition. To reverse the original proposal from the current test, first undo only the reviewed formatting change, then remove the original1959 bytes. Either that path or direct removal of the current1947-byte addition restores all62736 fixed544 bytes. The strict `assert_eq!(error.code, "unmet_authentication_requirements");`, protected/unprotected authorization sequence, existing transport and all test operations remain exact.

### Actual static checks and held result

Ran `rustfmt --edition 2024 --check --config skip_children=true tests/device_trust.rs` once in this correction phase: actual exit0, empty stdout. Source `git diff --check` and staged `git diff --cached --check`: exit0. Source-index scope was exactly one modified test; staged hash matched the formatted candidate, and no unstaged diff remained before its commit. These are parsing/formatting/whitespace/source checks, with no Rust type-checking, compilation, test execution, key generation or provider observation. The earlier exit1 remains preserved above. The regression remains intentionally negative and unrun; no product fix/pass or successful unprotected control is claimed. If future protected authorization unexpectedly succeeds, the test still stops at `.unwrap_err()` before the unprotected assertion, preserving the earlier unreached-control limitation.

The prospective focused command and own private cache remain exactly the prior proposal; no new cache/target was created, cache deleted or runtime probe made. The previous static inventory is dated evidence, not a new build/cache execution result. The existing `wave30-o06-readiness` cache had matching toolchain/manifest/lock/features and no compiled `device_trust` target. Root alone must release/reserve the runtime and fresh disk-floor monitoring after independent source review and sole-lane availability. Exact command, NOT executed:

```sh
env CARGO_TARGET_DIR="$PWD/target/wave30-o06-readiness" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support --test device_trust google::retained_local_proof_requires_reverification_after_provider_change -- --exact --test-threads=1
```

A09 ARM37101183416 still owns validation/Cargo. No Cargo/test/provider/native artifact/version/key generation/HTTP/browser/Driver/runtime ran; no slot was acquired/released. No production/schema/config/helper/other test/D01 file changed, no alignment/main/push/status action occurred, and no other worker was contacted. The protected D01 report/helper remain byte-exact to b05, with the prior recorded hashes. Historical private files were not opened or rewritten. Original I07 task/assignment/status, all historical failures/unknowns/lost-value corrections and unproved whole60 cleanup remain unchanged. Root alone owns source review, runtime, integration/publication and original acceptance; no closure or physical managed-device provenance is inferred.


### Actual correction-report checks

After this appendix was written: `python3 scripts/check-docs.py` exit0 (“Markdown links and build-directory layout checked”); `git diff --check` exit0. Exact source-commit scope and report-only current scope passed. Entire b05 report-prefix equality, committed formatted-test equality and protected D01/manifest/lock/toolchain byte equality passed. Final staged whitespace/scope/prefix and clean-tree proofs are returned with the separate report commit. These checks add no compiled/type/runtime result and do not erase the original fmt failure. Runtime remains held; no lane was acquired/released.


## 2026-10-03: ONE released provider-transition negative regression, actual failure

Reservation `wave30_I07_provider_transition_negative_once`, project891e7443-8dac-4c1b-897f-9e53cb59c7ee, original I07 `34688b10-fa4b-4b83-8070-adfb3e55dc41`, existing WT7/shell7a only. Root explicitly released the sole Cargo/validation lane for ONE exact focused command after the A09 lane exited. The dated source-only proposals, initial rustfmt exit1, later formatting exit0 and all earlier unknown/physical-provider limits above remain intact. This appendix preserves the entire53246-byte f605 report prefix, SHA256 `9c6b2d1083c33d0ac2a6bf052961099611b77f9734072e1ca2fff5b9e741fc0d`.

Actual Cargo exit101:0 passed,1 failed,0 ignored,0 measured,10 filtered out. The expected negative oracle was reached at `tests/device_trust.rs:1821:14`, the protected authorization `.unwrap_err()` after switching configuration to Google. The fixed panic classifier recognizes the unwrap-error-on-Ok condition without copying its response values. The later strict error-code equality and unprotected-client assertion were NOT reached. This supplies a concrete failing regression under the accepted provider-transition contract, not a passing production fix or physical Google/Windows/managed-device result. No assertion was weakened, product code changed or second command run.

### Exact source and launch prerequisites

The clean reviewed branch before launch was `be6d4afe932bf03454d6b0dedfa482c8a2e33b4f`, tree `503b86e60d29c5e8b5349f5139620603b718bdd1`. The current test was byte-exact to source commit `6ba2e95c42c271804d1715eb05380953b008130d` and retained blob `5c9d9401961033805745335ac353309237e9ba65`:64683 bytes/1829 lines, SHA256 `ee9c1676e8fb3aa6f6d2092a343e70bb0beae703d563d3e410ad611093e340dd`. Removing the complete formatted1947-byte/52-line addition again restored ALL62736 bytes of fixed `544d1340b80cd3e040dc13142cdcbc1d75fea4cb`, SHA256 `2287bfff16bac53e096652ac9760342577f0a46aaf3f5e714d064a6947559e8f`. The original1959-byte pre-format proposal and its qualified reversal remain dated above.

All tracked files since6ba differed only in the existing D01 and I07 reports. Entire production/test/scripts/manifests/config/toolchain bytes were otherwise unchanged; the working tree was clean. Direct current/reviewed equality also covered:

| Build input | Bytes | SHA256 |
| --- | --- | --- |
| `Cargo.toml` | 4020 | `58e5ef824ed96290179c9f76fea208dc37173caeee21b6ce8d37f8a6dd1abcb8` |
| `Cargo.lock` | 109243 | `b5c9d11c001244b8017303ce8c20516e02946483845eee40720b0910758d4426` |
| `rust-toolchain.toml` | 86 | `887f9be066a15585a2c583578e84b0fcb541126d81546276bad3d2ff00d61167` |

Before spawn, exact-name `pgrep -x cargo` and `pgrep -x rustc` each returned1 with empty stdout/stderr, confirming no observed competing process. No unrelated arguments/environment were read or process signalled. The existing own, regular non-symlink `target/wave30-o06-readiness` cache had the reviewed `riauth-0d65ce4b80b71950/lib-riauth.json` SHA256 `ac6f00d7f80260b8f443564cbbdd8e64f91d91aa42f8d001c0a23e13778f1c10`, features `["default", "essentials", "platform", "test-support"]`, and its existing rlib. This is a warm-cache source prerequisite; the actual result below separately supplies compilation/test evidence. No duplicate cold target, cache prune, artifact/provider refresh or toolchain/version invocation was performed.

The only Cargo invocation was exactly:

```sh
env CARGO_TARGET_DIR="$PWD/target/wave30-o06-readiness" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support --test device_trust google::retained_local_proof_requires_reverification_after_provider_change -- --exact --test-threads=1
```

The private supervisor archived the exact displayed command and equivalent expanded `env` argument vector BEFORE launching it directly, with the current WT as cwd. No extra test filter/target, feature, offline flag or timeout flag was added to Cargo. It installed `SIGCHLD` default before any subprocess so the actual Cargo child status could be consumed. Fresh capacity was remeasured immediately before spawn:16775020544 bytes/15.622955GiB, above the12GiB launch requirement.

### Finite runtime, retained output and actual failure classification

A NEW absent directory `deployment-private/i07-provider-transition-negative-20261003-3a19d4ccecd6` was created0700; every contained file is owned, regular and0600. Source archive, combined output, numeric status, grading and later classification use exclusive creation; no historical evidence was overwritten. The supervisor's source was parsed and compiled to an unexecuted code object before its single runtime invocation. Its actual Cargo child used `start_new_session=True`; PID55144/PGID55144 was independently checked before collection. Supervisor PID55106 subsequently exited0.

The inclusive bound was1800s, with an active stop at1765s reserving cleanup time. It used nonblocking bounded output collection, a16MiB combined-log cap, nominal2s disk samples, own-stop below9GiB and8GiB floor. Stops for capacity, output cap, deadline or observed dependency download request would send TERM only to the proven own group, then KILL/reap if needed. The actual run completed in62.326150s with no stop reason, no internal failure, no cleanup error, no signal and complete output EOF. No dependency-download announcement was observed; this is not an independently monitored network claim.

There were32 retained disk samples including initial/final readings. Minimum free16485629952 bytes/15.353439GiB; final16697425920 bytes/15.550690GiB. Maximum observed sampling gap2.180825s is retained rather than claiming exact two-second scheduling. Neither9GiB stop nor8GiB floor was reached. The full1549-byte combined Cargo output and exit101 were flushed/fsynced before any grading, below the16MiB cap with no truncation. Its content remains private; no response debug values, keys, identifiers, request values or arbitrary panic text are copied into this report.

The initial private `grade.redacted.json` correctly recorded counts/exit/reaping but its strict thread-prefix regex did not match the observed panic rendering, so it initially recorded a null location and `expected_oracle_reached:false`. That reporting error is preserved byte-exact. A separate read-only bounded classifier, run AFTER output/status retention and resource release, matched only the fixed `tests/device_trust.rs` panic location and the fixed unwrap-error-on-Ok condition. New `panic-location.redacted.json` records1821:14 and `expected_oracle_reached:true`. No test or supervisor rerun/correction occurred. This is a correction to reporting interpretation, not a revised runtime outcome.

The actual failure stops at the newly reserved protected-provider-transition oracle. Source control flow establishes that local proof creation/verification, initial protected authorization, Google configuration reopening and installation of the existing synthetic `Boom` transport precede that line. The later `assert_eq!(error.code, "unmet_authentication_requirements")` and unprotected authorization follow it and remain unreached. This run does not demonstrate an unprotected-client pass, a production repair, actual Google verification, supported physical Windows enrollment, TPM/hardware provenance, or complete I07 acceptance. The existing fixture uses software-generated synthetic keys/private files and the in-process `Boom` transport; no real device/provider credential or Google response was supplied.

### Private evidence identities and release

All paths in this table are relative to the new0700 directory above. Hashes identify complete files; public receipt fields and fixed failure extraction were read, while raw response values were not published.

| Retained file | Bytes | SHA256 |
| --- | --- | --- |
| `supervisor.py` | 14537 | `d2ef5042864f3d5f85248edb894a7472e497048ae18045fe3e58de2079e8ac70` |
| `launch-source.redacted.json` | 2688 | `2de54e3e1f188cddbe3e584760fba70549efa11d6220cbe61887e7adc6f87608` |
| `owned-child.redacted.json` | 112 | `b6385f5f7adc18dc042eb89a20f3673161d7c8a356169b1433739f6749af7084` |
| `cargo-combined.private.log` | 1549 | `b87625e6e5fdcd355641a2f104fd7ecbb8d4fb126ca38749ba6cfc81ab43db48` |
| `actual-status.redacted.json` | 4008 | `a6da69b426fe23f1aa7d1a8888336d6db83ad1e1db68f4de97e13ae7f0cd24e2` |
| Initial `grade.redacted.json`, preserved reporting error | 1189 | `4f05c6fee69f98743c0a1f6b43c17f719f363a3357e783a5eb927a73b3a9afb6` |
| Separate `panic-location.redacted.json` | 412 | `f5f3d9a6ceeec2b8955d5abccc05dff518eda3e4c7996cef81f984865c7ab79d` |
| `cleanup-release.redacted.json` | 341 | `99c9e0e46e36a9398105e39fe8c46f707bb6f45a96fa9ea872be31b9bfd29ae7` |

Cargo exit101 was consumed and the child reaped; fresh exact PID55144 and PGID55144 absence were read back. The supervisor's tool result was exit0, and fresh PID55106 absence was also confirmed. There were no remaining owned children/group, no TERM/KILL, and no cleanup error. SOLE CARGO/VALIDATION RELEASE was sent immediately after these joined-exit/absence proofs, BEFORE the report appendix. The supervisor exit0 indicates successful retention/cleanup, not a passing Cargo test. Retained private evidence stays available for root review. No browser, Driver/session, service/listener or operator fixture was created by this assignment.

### Protected history and next authority boundary

Only this I07 report is appended. Current `tests/device_trust.rs`, every tracked production/script/config/manifest/toolchain file and D01 report/helper remain byte-exact to pre-run be6. Protected D01 report1512584 bytes SHA256 `7fe8d1ee394268ccedc49947c3b35422656068dd044049941b73b5611e0e60aa`; helper36884 bytes SHA256 `37d32db6fbd8c2c9b676f691d115ecfd1af893724144361971e86f73f573b406`. The be6 D01 refusal stays failed/unused, D01/browser remains HELD, and no old functions-store value was inspected/reset. All earlier source/formatting failures, historical private captures, unknown sender/cause/lost-value corrections, failed envelopes and unproved whole60 cleanup remain preserved. No source alignment, product/test/helper correction, second Cargo command, worker contact, task/worktree/shell creation, main/push/status or cache deletion occurred.

Original I07 todo/primaryd0ce and every prior authority/review/receipt/header/PAM/held Group/nonrenewed60s/paused IO contract remain unchanged. Root alone reviews this actual negative result and may separately reserve the exact production correction; this worker claims no whole-row closure, current physical-device/provider success or D01/D05 gate completion. Final document, whitespace, full-prefix, report-only scope and clean-tree checks are returned with the separate immutable evidence commit.


### Actual evidence-report checks

After the actual-evidence appendix was written: `python3 scripts/check-docs.py` exit0 (Markdown links and build-directory layout checked); `python3 scripts/check-repo-hygiene.py` exit0 (1085 tracked files); `git diff --check` exit0. Full53246-byte prior-prefix equality, report-only current scope, protected current/HEAD source equality and entire fixed544 test inverse all passed. These passing document/scope checks do not change the actual Cargo exit101 or supply the unreached error-code/unprotected control assertions. Final staged whitespace/prefix/scope and clean-tree proofs are returned with the separate immutable evidence commit. No further Cargo/test/native/provider/Driver execution occurred after lane release.


## 2026-10-03: reserved retained-provider binding correction, source/static only

Reservation `wave30_I07_retained_provider_binding_correction`, same project891e7443-8dac-4c1b-897f-9e53cb59c7ee and existing WT7/shell only. Root explicitly authorized the additive retained provider field, canonical kind identity, policy freshness equality, both successful writers, literal compatibility and focused regressions. Original I07 task/primary/todo and root gate authority remain unchanged. This appendix preserves the ENTIRE64870-byte84d3 report prefix, SHA256 `0e92ef3c37dde6d154dd8ef53fa7188c89dabb5ea33b51c01836e73ca55ed99c`. The previous actual negative exit101, unwrap-error-on-Ok1821, initial classifier error and separate correction, strict-error/plain assertions unreached, private captures and release remain dated evidence, not new outcomes.

Source commit `19805cb9250f3b3d77da4c15b52c313794450143`, parent `84d3d8a46df6cf9635bd00123592d2d67b573fa9`, tree `31d713bb4405c5c5372cf0e5b9fe508d64365ead`: ONLY four reserved source/test files,309 added lines and zero removed lines. The tree was clean after that source commit and before this separate report phase. Documentation path coordination remains the concrete outstanding scope input described below; no outside guide path was edited or new stub guide created.

### Implemented contract and exact source identities

`DeviceVerification` gains ONLY `#[serde(default)] pub provider: Option<String>` with its explanatory comment. A row lacking the field deserializes toNone, not local. `ProviderKind::identity()` returns only fixed `local` and `google_verified_access_v2`. The local and Google successful record writers each stamp their corresponding fixed enum identity, independently of a caller value or a configurable string. Neither writer manufactures trust before its existing verification checks or weakens the device-id binding.

In `policy_reason`, the existing unprotected-client early return, missing-config/readiness gates and identity-required gate remain byte-exact. Only after provider readiness and identity succeed does a pure `provider_kind(config)?.identity()` resolve the configured kind. Its underlying `provider_kind` body is unchanged: omitted `TrustConfig.provider` and explicit `local` map toLocal; unsupported configured kinds are already refused by readiness. The freshness predicate gains one conjunction: `record.provider.as_deref() == Some(provider)`. Missing, null, unknown or different retained kind cannot satisfy that equality. Missing WHOLE `device_trust` configuration remains unconfigured; it is not the same as an omitted provider within a valid local configuration.

This binds a retained proof to the kind that accepted it. It does not fingerprint keys, customer identity, policy settings or every same-kind configuration change. Returning to a kind matching a still-fresh retained proof is governed by the same current-kind comparison and existing session/freshness checks, not a new configuration-change epoch. No broader invalidation or physical-provider provenance promise is introduced.

| Complete committed file | Bytes/lines | Git blob | SHA256 |
| --- | --- | --- | --- |
| `src/device_trust_types.rs` | 3338/99 | `d62bbc5e57a01a7548cceac2c28ffef1e706895e` | `9b8a931b68b2a453350904b32f5e6f7a98f3cecb88a0cebde41f7ed3f9803825` |
| `src/device_trust.rs` | 12917/361 | `af21281b8c484e6ca46d2571e379c30c867ca363` | `5b2960a196a678e13fc767d6e662d7a099e8a5da1d3bdb82be01dc6b1878993e` |
| `src/assembly/device_trust.rs` | 16909/424 | `0ed73bb89732933ded0407877e571476eaba427e` | `95c0a41470d4bf6cd68a6017e308ef090cf282e7f999eaaa4476fca8c2afea7b` |
| `tests/device_trust.rs` | 74157/2118 | `54bc47d08b5562ed6d836df55144c93b5ac8dd2d` | `f5fc6b3d42f97bf7c4601d8228e713066af1e3a95eeee3aca60dfeeec4c7f5df` |

Full bodies of CONTRIBUTING/SECURITY, shared retained/config types, device-trust policy/verifier source, assembly entry points and the actual ENT-06 guide were read. Relevant existing synthetic Google transport/account/challenge/response helpers, existing successful Google verification test, the old transition test and all new/changed source bodies were read. Other old test bodies were compared as complete source spans; their body-content identity is not a claim of new execution or a fresh manual review of every unchanged body. The full candidate bodies are concrete Git source objects at19805cb, not merely an unmaterialized diff.

### Whole-file reconstruction and protected spans

For each of the four source files, a line-preserving comparison against84d3 had only `equal` and `insert` operations. Removing ALL declared insertion spans reconstructed the ENTIRE corresponding prior object byte-for-byte, including every unchanged old function and surrounding source. No exclusions, snapshot normalization or changed old assertions were used.

| Prior complete file | Bytes | SHA256 | New inserted lines |
| --- | --- | --- | --- |
| `src/device_trust_types.rs` | 3188 | `2940b6fa74757e5e25a3761c1bb9cae4303a2f54c0c8c58cad550b66a2803273` | 3 retained-field lines |
| `src/device_trust.rs` | 12579 | `8524103a534cde6e785f3b272f4b69d28becf462c4b005232058d20fb37b51a0` | 9 identity-helper lines,1 kind resolution,1 equality |
| `src/assembly/device_trust.rs` | 16633 | `2ad918244800eb6f6c5ce3ab2cdeb8790dc60bbd96e0fcb8de5191819eee688b` | 1 local writer line,5 formatted Google writer lines |
| `tests/device_trust.rs` | 64683 | `ee9c1676e8fb3aa6f6d2092a343e70bb0beae703d563d3e410ad611093e340dd` | 1 existing-literal compatibility line,288 new regression lines |

A separate comment/string-masked lexical function-span comparison found68 prior full function spans byte-exact. Exactly FOUR old spans have additions: `policy_reason`, `device_verify_local`, `device_verify_google`, and the existing missing-verifier test's sole `DeviceVerification` literal. Exactly FOUR new function spans exist: `ProviderKind::identity` and the three regressions listed below. This is static lexical span evidence, not Rust typechecking or AST execution. Whole-file byte reconstruction supplies the stronger no-other-byte-change proof.

The existing strict `google::retained_local_proof_requires_reverification_after_provider_change` complete attributed/function block remains1945 bytes/51 lines, SHA256 `4c16bd1ff495e791878972a7841463959b95e5f62e72457bf912a23241209354`, present exactly once and byte-exact to84d3/6ba. Its previously recorded1947-byte addition includes two surrounding newline bytes, so these lengths are deliberately distinguished. After removing this phase's additions, removing the original1947-byte6ba addition still reconstructs ALL62736 fixed544 test bytes. Source line numbers shift by the earlier literal addition; the old actual1821 failure remains attributed to its executed pre-fix source, not this unrun candidate.

The sole existing test literal adds `provider: None` to preserve its legacy/planted-row purpose. Explicit path-bounded static lookup found exactly FOUR type/literal sites: the shared struct, two assembly writers and this one test literal. Source namespace lookup showed the only production `tx.put("device_verifications",...)` sites are those two successful writers. Beyond the reserved retained-field addition, no new writer, arbitrary provider, storage schema version/migration, config/challenge/epoch/token/public route contract/workflow change is added.

All original identity/user/session/epoch, live-session, expiry and nonempty-device predicates remain exact; provider equality is an additional requirement. Challenge/replay/transaction/device-id-change/audit behavior, verifier and remote HTTP/private credential bounds, local JWT limits and public error/response bodies remain exact. Both writer `verified_at` calculations and expiry minima are untouched. `src/assembly/claims.rs` and approved_device/timestamp projection were not edited. Unprotected callers still return before reading provider/config/proof state. Cleanup still preserves the session's device-id binding until its original expiry/revocation behavior; it does not discard that binding to permit another device after a kind change.

### Focused new regression definitions, UNRUN

The original strict negative remains unchanged. New definitions add:

| New test | Intended source-defined oracle; no observed pass |
| --- | --- |
| `google::retained_google_proof_requires_reverification_and_can_rebind_both_providers` | Existing synthetic Google helpers perform initial acceptance and stampGoogle; reopened local configuration rejects protected access with exact `unmet_authentication_requirements` while plain remains usable; a new signed local challenge proof stampsLocal and restores protected access. Reopening Google rejects the retained local proof again; existing synthetic Google verification stampsGoogle and restores access. All three accepted signals use the same device id, preserving the existing different-device refusal. |
| `implicit_and_explicit_local_provider_keep_the_same_retained_binding` | Both omitted-to-explicitLocal and explicitLocal-to-omitted transitions reopen Core on the same fixture store. A real local verification writer stamps canonicalLocal in each source-defined branch, and that retained proof is expected to remain usable without another verification merely for the equivalent spelling. |
| `legacy_and_unknown_provider_rows_require_actual_reverification` | Starting from a successful local writer, only the retained provider is removed, setnull or changed to a fixed unknown synthetic string. Each negative requires exact protected refusal and preserves plain access. A NEW signed challenge proof through the actual local writer, not a forged success row, replaces that legacy/unknown provider and is expected to restore access. |

These are three meaningful regression definitions covering the requested kind/compatibility/writer seams, not three observed passes. Source-only raw-row alterations are used solely to model negative legacy/unknown inputs; every positive result is reached through the existing cryptographic verification path. The accepted in-process Google helpers handframe a synthetic challenge/response and script a synthetic accepted response. No actual Google/device attestation, managed Windows enrollment, hardware key or physical-provider success is claimed. No fixture, key generation, helper function, service, HTTP request or test case ran in this phase.

### Exact documentation mismatch; paragraph reserved for root coordination

The authorized path `docs/device-trust.md` DOES NOT EXIST. Initial path-specific `rg` returned2 with that missing-file diagnostic. The current published/own guide is `docs/enterprise/ENT-06.md`,13373 bytes, blob `f71d5fdbc33591d02a1c5f9daa5bf6621c45a6fc`, SHA256 `16ddfb51e8579a4dab74d49dc6c4b486eb42e5f633271daa71afae29cf5fee02`. Its complete body was read and stays byte-exact. Before touching any outside path, an asynchronous root coordination question requested either reservation of that guide or root ownership of the paragraph. No answer or outside-path authorization has arrived as this appendix is prepared; neither the real guide nor a misleading new stub path was written.

Concrete proposed ONE paragraph, immediately after ENT-06's `## Shared behavior` heading, pending exact root reservation:

> Each retained verification records the provider kind that accepted it (`local` or `google_verified_access_v2`). Protected clients require verification again when that kind differs from the configured provider, or the retained provider is missing or unknown. Omitted `provider` and explicit `local` are equivalent. This kind binding does not fingerprint every configuration, key or customer setting. Clients without `require_device_trust` keep their existing behavior.

This is the single remaining documentation scope input; it is not an invented product/runtime blocker or a completed guide edit. Root can review this concrete paragraph and reserve the actual path, or integrate it itself. No broader guide/API/release-note expansion is proposed.

### Actual static checks, retained failures and held runtime

Ran direct formatter ONLY on the four reserved files: `rustfmt --edition 2024 --config skip_children=true src/device_trust_types.rs src/device_trust.rs src/assembly/device_trust.rs tests/device_trust.rs` exit0; subsequent same-files `rustfmt --edition 2024 --check --config skip_children=true ...` exit0 with empty stdout. Whole-file reconstruction confirmed the formatter altered no prior source byte. `python3 scripts/check-docs.py` exit0; `python3 scripts/check-repo-hygiene.py` exit0 (1085 tracked files); source `git diff --check` and staged `git diff --cached --check` exit0. These are formatting/parser/document/source checks, with NO Cargo/Rust typecheck/compiler/test/native/provider/browser execution.

Two static lookup-witness attempts FAILED before completing their later receipt/source checks: the first Python-spawned `rg` had an overescaped regex and no explicit search paths; it returned1 with no matches. The corrected regex still returned1 because that child inherited the exhausted heredoc stdin and had no explicit search paths. A trailing separate namespace lookup caused the second shell's aggregate exit0 despite the Python subcheck's traceback; that aggregate is NOT treated as a passing witness check. The final corrected read-only witness explicitly supplied `src`/`tests` and DEVNULL stdin, returned0, found all four intended sites, and completed exact scope/protected-source/private-capture checks. No product/test assertion failed or runtime rerun occurred in this source phase. Earlier actual negative and formatting/reporting failures remain preserved separately above.

All EIGHT private files from84d3's actual negative directory were rehashed without publishing contents: complete hashes and0600 modes still match the prior table, including raw output, initial incorrect grade, separate panic classifier, child/status/cleanup receipts and supervisor source. No historical file was overwritten. D01 report1512584 bytes SHA256 `7fe8d1ee394268ccedc49947c3b35422656068dd044049941b73b5611e0e60aa`, helper36884 bytes SHA256 `37d32db6fbd8c2c9b676f691d115ecfd1af893724144361971e86f73f573b406`, existing ENT-06 guide, build manifests/lock/toolchain and initial I07 prefix remain byte-exact. No D01 store/controller/private-input/browser action or source alignment occurred. Unknown historic sender/cause/lost values, old failures and unproved whole60 cleanup remain unchanged.

Prospective ONE whole device_trust target command, NOT EXECUTED and requiring separate root runtime release after full source/independent review:

```sh
env CARGO_TARGET_DIR="$PWD/target/wave30-o06-readiness" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support --test device_trust -- --test-threads=1
```

The default Platform plus test-support source defines14 tests after these additions, but there is no observed new discovery/count/result. The existing own warm target and previous exact-command artifact evidence stay dated; no new target, cold duplicate, cache deletion, artifact/provider pin refresh or download occurred. Proposed future resources remain the root-coordinated sole lane, fresh12GiB launch/9GiB own-stop/8GiB floor,2s samples, one1800s bounded invocation, capped private16MiB output and exact owned group TERM/KILL/reap if required. Root must review/reserve those actual runtime details; this source phase acquires/releases no validation/Cargo/desktop slot. Runtime remains FREE/unused by this worker and D01 HELD. No current passing negative, type safety, whole-target pass, physical provenance or whole-I07 closure is asserted. Root alone owns production/source/doc integration, later execution and original acceptance/status.


### Actual source-report checks

After this source-phase appendix was written: `python3 scripts/check-docs.py` exit0; `python3 scripts/check-repo-hygiene.py` exit0 (1085 tracked files); `git diff --check` exit0. Entire64870-byte84d3 prefix equality, report-only current scope, exact committed four-file source equality and unchanged actual guide/D01/build-input proofs passed. The two earlier failed lookup witnesses remain explicitly retained above. Final staged whitespace/prefix/scope and clean immutable handoff are returned separately. The guide paragraph remains an unapplied concrete proposal pending exact root path coordination; these checks confer no type/test/runtime pass or I07 gate completion.


## 2026-10-03: exact ENT-06 documentation scope coordinated, guide-only commit

Root separately reserved ONLY `docs/enterprise/ENT-06.md` for the verbatim concrete paragraph froma140, plus this append-only report. The earlier missing `docs/device-trust.md`, discovery/coordination history, source-only limits, static lookup failures and every actual negative/reporting failure remain byte-exact in the complete81449-byte a140 report prefix, SHA256 `13372e42b1fa62a0212dba36f5d9379c25f1112dcb968e76e86b975ec216683d`. No stub guide was created at that absent path. This appendix records the new explicit reservation and completed paragraph, without retroactively rewriting the earlier pending scope input.

Guide commit `ed5f59fbfad2539a158ac5b08db133ed58f15c9a`, parent `a140d1b37f3489991ef204b452417b4fcc436473`, tree `f0d567d5b0eb11804c0ed0294ebbc6dd8c97d5b6`, changes ONLY `docs/enterprise/ENT-06.md`:ONE additive hunk,2 added lines/zero removed lines, immediately after `## Shared behavior`. Its complete467-byte paragraph was extracted from the immutable a140 blockquote by removing ONLY the two quote-prefix bytes; no word, punctuation or whitespace inside the paragraph changed. The added paragraph plus its two newline bytes is469 bytes, SHA256 `ffe8602e7260dbf3b0b4d7ef902cf2272d77655460b52d4cecbcd79bf36738d0`.

| Complete guide object | Bytes/lines | Blob | SHA256 |
| --- | --- | --- | --- |
| Baseline, byte-equal to immutable published758c43a | 13373/92 | `f71d5fdbc33591d02a1c5f9daa5bf6621c45a6fc` | `16ddfb51e8579a4dab74d49dc6c4b486eb42e5f633271daa71afae29cf5fee02` |
| Committed guide | 13842/94 | `4f19f9bcfde2d6ae5b325f8eb5c1218be9829875` | `cd002bfcbef02aaa2c4f344ef74fe2b419b3b89c33faac3f8a0e9ac30149a531` |

Removing that ONE unique exact469-byte addition reconstructs ALL13373 baseline bytes. The complete published/own baseline equality, unique heading/paragraph, exact a140 wording, one-hunk diff, staged guide-only scope and clean tree before this separate report phase were proved. No existing guide byte or code/test/manifests/config/provider behavior changed. The paragraph states provider-KIND binding, implicit/explicitLocal equivalence, missing/unknown retained-provider re-verification, existing unprotected behavior, and the absence of a complete key/customer/configuration fingerprint promise. The reserved serde-default Option field maps JSONnull toNone; null therefore falls within the missing retained-provider case, without altering the approved verbatim paragraph to add another word. This is source interpretation, not a fresh null-case execution. No approved_device/claim timestamp contract changed.

Actual guide-phase checks: `python3 scripts/check-docs.py` exit0; `git diff --check` and staged `git diff --cached --check` exit0. Source/test/code equality and the full-byte inverse passed. No formatter/compiler/Cargo/provider/version/helper/CLI service/HTTP/listener/browser/Driver/native/runtime executed. The original four-file source commit remains `19805cb9250f3b3d77da4c15b52c313794450143`; all of its source/test objects and every other tracked file were preserved. Root's accepted design and ongoing independent19805 review are coordination inputs, not an independent verdict observed by this worker. Runtime remains HELD; no slot was acquired or released.

### Fresh read-only warm-cache and capacity prerequisites

At `2026-10-03T08:28:11.358751+00:00`, a fresh statvfs sample reported16614924288 bytes/15.473854GiB usable free capacity, above the proposed12GiB launch floor. An earlier same-phase sample was16621158400 bytes/15.479660GiB. These are unreserved, time-specific observations: root must remeasure at any later approved launch and retain the9GiB own-stop/8GiB floor with2s samples. `pgrep -x cargo` and `pgrep -x rustc` each returned1/count0; no process arguments/environment were read and no signal was sent. This does not acquire a runtime lane or guarantee future process absence.

The existing own `target/wave30-o06-readiness` is an owned regular non-symlink directory. Bounded owned regular metadata reads found:

| Existing metadata file, relative to target | Bytes | SHA256 |
| --- | --- | --- |
| `.rustc_info.json` | 1965 | `27df402be20083ab5b4835c05762e2b77beed67288686dd8fda9193b24cdd7c4` |
| `debug/.fingerprint/riauth-0d65ce4b80b71950/lib-riauth.json` | 3611 | `ac6f00d7f80260b8f443564cbbdd8e64f91d91aa42f8d001c0a23e13778f1c10` |
| `debug/.fingerprint/riauth-8df5dbea6d9f15f4/test-integration-test-device_trust.json` | 4019 | `f078dcd7d024389e415d7624eac3f0cedd05994bc36619bfd551a526795c4ee7` |

Both lib/test fingerprints report features `["default", "essentials", "platform", "test-support"]`, rustc identity17329007180185699724, compile_kind0 and empty rustflags. Lib profile12672335563272108896 and test profile11094973624911973823 remain their respective recorded cache metadata. File metadata, WITHOUT reading/executing binary contents, found the regular owned `debug/deps/libriauth-0d65ce4b80b71950.rlib`411892376 bytes and prior `debug/deps/device_trust-8df5dbea6d9f15f4`68686848 bytes. These are reusable-cache prerequisites and old artifacts, not compiled evidence for19805 or the newly added tests.

An initial guessed fingerprint `riauth-3c65d1e515a3f965/test-integration-test-device_trust.json` was absent and explicitly recorded as such. A path-only `rg --files --hidden --no-ignore` inventory then found the exact existing8df5 fingerprint; that immutable metadata was read above. The missing guess is not hidden or converted into an execution/build failure. No cache/target creation, deletion/prune, artifact content hash refresh, toolchain/native invocation or download occurred.

Proposed ONE whole target command, STILL NOT RUN:

```sh
env CARGO_TARGET_DIR="$PWD/target/wave30-o06-readiness" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support --test device_trust -- --test-threads=1
```

Future prerequisites remain root full source/independent review and an explicit sole-lane release; fresh source/test equality, capacity and competitor samples; the same own warm target/jobs1/inc0/dev+testdebug0; one bounded invocation with private0700 evidence/exclusive0600 capped16MiB output, retained numeric status before grading, and exact owned process-group cleanup. The prior1800s/12GiB launch/9GiB stop/8GiB floor resource proposal is unchanged, but no execution or resource ownership is assumed. The new source-defined14-test count and anticipated negative correction are UNOBSERVED. No old failed11-test result, passing memory envelope or metadata artifact is borrowed as a current whole-target pass. I07 original task/status and D01 held state remain root-owned/unchanged; the guide paragraph resolves only the documented path-coordination gap.


### Actual documentation-report checks

After the coordination/cache appendix was written: `python3 scripts/check-docs.py` exit0; `git diff --check` exit0; complete a140 report-prefix, committed guide equality/verbatim whole-byte inverse to758c43a, exact guide/report-only scope and unchanged code/test/D01/manifest/toolchain checks passed. Final read-only capacity sample at 2026-10-03T08:32:56.569058+00:00 was16526512128 bytes/15.391514GiB; it is unreserved and must be sampled again before any separately approved launch. No resource slot was acquired/released, no runtime executed, and no new pass or gate disposition is supplied. Final staged whitespace/prefix/scope/clean proofs accompany the immutable report handoff.


## 2026-10-03: exact accepted-tree alignment and whole-target supervisor preparation, UNRUN

Reservation `wave30_I07_whole_target_exact_preparation`. The entire89006-byte41b/fixed5c I07 report prefix remains byte-exact, SHA256 `a8bf6f8eb4ae8c45787d3e3c370683a1299a59e1d32926a55594762759271f1e`. The original negative failure, initial panic-parser miss/separate correction, all formatting/static lookup failures, documentation absence/coordination and all prior raw/private captures remain dated and unchanged. Root has not released runtime.

ONE authorized history-preserving merge: `715f6e9c37cf3e6a92f13f284fa54e4cb6d8d0fd`, parents `41b4ec68d70b30c9c891aa098e05443c283b4209` and EXACT fixed accepted unpublished `5c20ca13effb28501976aef69e50c6738ab560b6`. Automatic merge had ZERO conflicts and required no resolution or source edit. Merge tree `75ffffe23b4d24f7b09256c3670a44ab693b1120` equals the ENTIRE fixed accepted tree. Complete recursive NUL-delimited Git tree entries matched all paths/modes/blobs, including every src/crates/script/test/manifest/toolchain/build configuration/guide and report. The own I07/D01/D05/A09 report complete prefixes were also compared directly. Working tree was clean after the merge. No reset/rebase/stale copy/import or main/push/status operation occurred. Alignment carries accepted workflow/source changes exactly; it does not independently validate their bodies or runtime.

### Exact controller manifest and allowed adaptation

Original root-reviewed negative supervisor was read fully AS DATA from its retained exclusive0600 private file; complete14537-byte SHA256 `d2ef5042864f3d5f85248edb894a7472e497048ae18045fe3e58de2079e8ac70` matched. The candidate below is concrete complete executable source archived ONLY in this report, NOT a materialized controller/runtime file. Neither original nor candidate was imported, evaluated, executed or case-tested. Only stdlib AST parsing, source hashing and text/AST reconstruction were performed.

Manifest:

```json
{
  "candidate_bytes": 15555,
  "candidate_lines": 258,
  "candidate_not_imported_or_evaluated": true,
  "candidate_sha256": "e88102b020534c334d6452cd8b2a2edb45e4c54fd6a395d82ff5416cd4e275bc",
  "child_loop_drain_cleanup_caps_deadlines_exact_except_retained_source_pin": true,
  "declared_transform_count": 8,
  "diff_bytes": 12460,
  "diff_sha256": "335f4ec5880e0c1a15c68cab1dfd2c1079be13620b4a579b03c805e88bb9a265",
  "five_helper_functions_byte_exact": true,
  "no_runtime_file_materialized": true,
  "original_bytes": 14537,
  "original_sha256": "d2ef5042864f3d5f85248edb894a7472e497048ae18045fe3e58de2079e8ac70",
  "runtime_leaf_absent": "deployment-private/i07-device-trust-whole-5c20ca-20261003-once",
  "sigchld_first_main_statement_exact": true,
  "transforms": [
    "new fixed absent private evidence leaf",
    "alignment/source pins",
    "display whole-target command",
    "argv whole-target command",
    "accepted source equality and report-only descendant preflight",
    "truthful accepted-source report and whole-target archive metadata",
    "truthful current retained source head",
    "single-summary whole-target numeric grade and fixed panic locations"
  ],
  "whole_ast_inverse": true,
  "whole_byte_inverse": true
}
```

The eight exact text transforms affect ONLY the fixed private output leaf; source/accepted-test/report pins and truthful equality metadata; removal of the test name/--exact from the displayed command and argv; and whole-target numeric grading. `HEAD` denotes immutable merge715f, whose entire tree equals fixed5c; it does NOT guess a self-referential future report commit. Future launch requires that exact merge as ancestor, clean tracked state, fixed current accepted test blob/hash/bytes, and only the I07 report changed since that merge. The complete fixed89006-byte historical report prefix must remain exact. These checks pin every other tracked path/mode/blob to the accepted merge via the report-only tree diff; they do not allow other production, test, buildconfig or guide deltas. Actual current launch HEAD/report hash are retained truthfully only if later execution is separately released.

The old fixed544 whole-addition check is not falsely reused on the new74157-byte candidate test. Preflight now requires exact fixed5c test equality/blob `54bc47d08b5562ed6d836df55144c93b5ac8dd2d`/SHA256 `f5fc6b3d42f97bf7c4601d8228e713066af1e3a95eeee3aca60dfeeec4c7f5df`; earlier full544 byte inverses remain dated above. The canonical current test/source/report pins and accepted manifest bytes must match before Popen. No product or test code is altered by this controller preparation.

### Exact complete proposed controller (UNRUN)

```python
import hashlib
import json
import os
from pathlib import Path
import re
import selectors
import signal
import stat
import subprocess
import sys
import time

ROOT = Path('/Users/dominik/orca/projects/riAuth-public-preview-sol-management-wave30')
RUN = ROOT / 'deployment-private/i07-device-trust-whole-5c20ca-20261003-once'
HEAD = '715f6e9c37cf3e6a92f13f284fa54e4cb6d8d0fd'
REVIEW = '5c20ca13effb28501976aef69e50c6738ab560b6'
FIXED = '5c20ca13effb28501976aef69e50c6738ab560b6'
TEST_SHA = 'f5fc6b3d42f97bf7c4601d8228e713066af1e3a95eeee3aca60dfeeec4c7f5df'
GIB = 1024 ** 3
LOG_CAP = 16 * 1024 ** 2
DISPLAY_COMMAND = 'env CARGO_TARGET_DIR="$PWD/target/wave30-o06-readiness" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support --test device_trust -- --test-threads=1'
ARGV = ['env', 'CARGO_TARGET_DIR='+str(ROOT/'target/wave30-o06-readiness'), 'CARGO_BUILD_JOBS=1', 'CARGO_INCREMENTAL=0', 'CARGO_PROFILE_DEV_DEBUG=0', 'CARGO_PROFILE_TEST_DEBUG=0', 'cargo', 'test', '--locked', '--features', 'test-support', '--test', 'device_trust', '--', '--test-threads=1']


def digest(data):
    return hashlib.sha256(data).hexdigest()


def exclusive_json(name, obj):
    data=(json.dumps(obj,sort_keys=True,indent=2)+'\n').encode('ascii')
    assert len(data)<LOG_CAP
    fd=os.open(RUN/name,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
    try:
        with os.fdopen(fd,'wb') as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        parent=os.open(RUN,os.O_RDONLY)
        try:
            os.fsync(parent)
        finally:
            os.close(parent)
    except BaseException:
        raise
    return {'path':str((RUN/name).relative_to(ROOT)), 'bytes':len(data), 'sha256':digest(data), 'mode':'0600'}


def read_git(args):
    return subprocess.check_output(['git',*args],cwd=ROOT,timeout=5)


def free_bytes():
    v=os.statvfs(ROOT)
    return v.f_bavail*v.f_frsize


def group_absent(pgid):
    try:
        os.killpg(pgid,0)
        return False
    except ProcessLookupError:
        return True
    except PermissionError:
        return False


def main():
    signal.signal(signal.SIGCHLD, signal.SIG_DFL)
    os.umask(0o077)
    assert RUN.stat().st_uid==os.getuid() and stat.S_IMODE(RUN.stat().st_mode)==0o700
    assert not RUN.is_symlink()
    current_head=read_git(['rev-parse','HEAD']).decode().strip()
    assert read_git(['merge-base',HEAD,current_head]).decode().strip()==HEAD
    assert read_git(['rev-parse',HEAD+'^{tree}'])==read_git(['rev-parse',FIXED+'^{tree}'])
    assert read_git(['status','--porcelain'])==b''
    test=(ROOT/'tests/device_trust.rs').read_bytes()
    fixed=read_git(['show',FIXED+':tests/device_trust.rs'])
    assert digest(test)==TEST_SHA
    assert read_git(['hash-object','tests/device_trust.rs']).decode().strip()=='54bc47d08b5562ed6d836df55144c93b5ac8dd2d'
    assert test==read_git(['show',REVIEW+':tests/device_trust.rs'])
    assert test==fixed
    changed=set(read_git(['diff','--name-only',HEAD,current_head]).decode().splitlines())
    assert changed<={'docs/roadmap/local-wave30-i07-managed-device-plan.md'}
    report=(ROOT/'docs/roadmap/local-wave30-i07-managed-device-plan.md').read_bytes()
    report_base=read_git(['show',FIXED+':docs/roadmap/local-wave30-i07-managed-device-plan.md'])
    assert report[:len(report_base)]==report_base
    inputs={}
    for path in ['Cargo.toml','Cargo.lock','rust-toolchain.toml']:
        data=(ROOT/path).read_bytes()
        assert data==read_git(['show',REVIEW+':'+path])
        inputs[path]={'bytes':len(data),'sha256':digest(data)}
    target=ROOT/'target/wave30-o06-readiness'
    assert target.is_dir() and not target.is_symlink() and target.stat().st_uid==os.getuid()
    fingerprint_path=target/'debug/.fingerprint/riauth-0d65ce4b80b71950/lib-riauth.json'
    fingerprint=fingerprint_path.read_bytes()
    assert digest(fingerprint)=='ac6f00d7f80260b8f443564cbbdd8e64f91d91aa42f8d001c0a23e13778f1c10'
    features=json.loads(fingerprint)['features']
    assert features=='["default", "essentials", "platform", "test-support"]'
    assert (target/'debug/deps/libriauth-0d65ce4b80b71950.rlib').is_file()
    competitors={}
    for name in ['cargo','rustc']:
        observed=subprocess.run(['pgrep','-x',name],capture_output=True,timeout=3)
        assert observed.returncode==1 and observed.stdout==b'' and observed.stderr==b''
        competitors[name]={'returncode':observed.returncode,'count':0}
    initial_free=free_bytes()
    assert initial_free>=12*GIB
    archive={'project':'891e7443-8dac-4c1b-897f-9e53cb59c7ee','reservation':'wave30_I07_whole_target_exact_preparation','source_head':current_head,'alignment_merge':HEAD,'accepted_source_pin':FIXED,'source_tree':read_git(['rev-parse','HEAD^{tree}']).decode().strip(),'reviewed_test_commit':REVIEW,'test_blob':'54bc47d08b5562ed6d836df55144c93b5ac8dd2d','test_sha256':TEST_SHA,'test_bytes':len(test),'accepted_test_sha256':digest(fixed),'accepted_test_equality':True,'report_baseline_sha256':digest(report_base),'current_report_sha256':digest(report),'expected_test_count':14,'tracked_production_manifests_toolchain_unchanged':True,'inputs':inputs,'command':DISPLAY_COMMAND,'argv':ARGV,'features':features,'warm_target_fingerprint_sha256':digest(fingerprint),'initial_free_bytes':initial_free,'minimum_launch_bytes':12*GIB,'owned_stop_bytes':9*GIB,'floor_bytes':8*GIB,'sample_interval_seconds':2,'inclusive_bound_seconds':1800,'active_stop_seconds':1765,'log_cap_bytes':LOG_CAP,'competitors':competitors,'supervisor_sha256':digest(Path(__file__).read_bytes()),'supervisor_pid':os.getpid(),'sigchld_default_before_spawn':signal.getsignal(signal.SIGCHLD)==signal.SIG_DFL}
    archive_id=exclusive_json('launch-source.redacted.json',archive)
    launch_free=free_bytes()
    assert launch_free>=12*GIB
    log_path=RUN/'cargo-combined.private.log'
    log_fd=os.open(log_path,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
    started=time.monotonic()
    samples=[{'elapsed_seconds':0.0,'free_bytes':launch_free}]
    stop_reason=None
    signals=[]
    child=None
    child_reaped=False
    owned_group=False
    group_gone=False
    bytes_written=0
    downloaded=False
    log_capped=False
    cleanup_errors=[]
    carry=b''
    sel=selectors.DefaultSelector()
    stream_eof=False
    first_stop=None
    returncode=None
    internal_failure=None
    with os.fdopen(log_fd,'wb') as log:
        try:
            child=subprocess.Popen(ARGV,cwd=ROOT,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,start_new_session=True)
            pgid=child.pid
            owned_group=os.getpgid(child.pid)==pgid
            assert owned_group
            exclusive_json('owned-child.redacted.json',{'pid':child.pid,'pgid':pgid,'owned_group_verified':True,'elapsed_seconds':time.monotonic()-started})
            os.set_blocking(child.stdout.fileno(),False)
            sel.register(child.stdout,selectors.EVENT_READ)
            next_sample=started+2
            term_at=None
            kill_at=None
            while True:
                now=time.monotonic()
                if now>=next_sample:
                    current_free=free_bytes()
                    samples.append({'elapsed_seconds':round(now-started,6),'free_bytes':current_free})
                    next_sample=now+2
                    if current_free<9*GIB and stop_reason is None:
                        stop_reason='disk_stop_threshold'
                if now-started>=1765 and stop_reason is None:
                    stop_reason='active_deadline'
                observed=child.poll()
                if observed is not None:
                    returncode=observed
                    child_reaped=True
                if stop_reason is not None and first_stop is None:
                    first_stop=now
                if child_reaped and not group_absent(pgid) and stop_reason is None:
                    stop_reason='owned_group_remained_after_child_exit'
                    first_stop=now
                if stop_reason is not None and not group_absent(pgid):
                    if term_at is None:
                        os.killpg(pgid,signal.SIGTERM)
                        term_at=now
                        signals.append({'signal':'TERM','elapsed_seconds':round(now-started,6)})
                    elif now-term_at>=5 and kill_at is None:
                        os.killpg(pgid,signal.SIGKILL)
                        kill_at=now
                        signals.append({'signal':'KILL','elapsed_seconds':round(now-started,6)})
                if child_reaped and group_absent(pgid) and stream_eof:
                    group_gone=True
                    break
                if first_stop is not None and now-first_stop>=20:
                    cleanup_errors.append('owned_cleanup_deadline')
                    break
                if now-started>=1790:
                    cleanup_errors.append('outer_cleanup_deadline')
                    break
                for key,_ in sel.select(0.2):
                    block=os.read(key.fileobj.fileno(),65536)
                    if not block:
                        sel.unregister(key.fileobj)
                        stream_eof=True
                        continue
                    remaining=LOG_CAP-bytes_written
                    saved=block[:max(0,remaining)]
                    log.write(saved)
                    bytes_written+=len(saved)
                    if len(saved)!=len(block):
                        log_capped=True
                        if stop_reason is None:
                            stop_reason='log_cap'
                    scanned=(carry+block).lower()
                    if any(label in scanned for label in [b'downloading ',b'downloaded ',b'updating crates.io index',b'updating git repository',b'fetching ']):
                        downloaded=True
                        if stop_reason is None:
                            stop_reason='unexpected_dependency_download'
                    carry=scanned[-128:]
        except BaseException:
            internal_failure='supervisor_internal_failure'
        finally:
            if child is not None and owned_group:
                if not group_absent(child.pid):
                    try:
                        os.killpg(child.pid,signal.SIGTERM)
                        signals.append({'signal':'TERM','elapsed_seconds':round(time.monotonic()-started,6)})
                    except ProcessLookupError:
                        pass
                    except BaseException:
                        cleanup_errors.append('term_failed')
                    try:
                        child.wait(timeout=min(5,max(0.01,1798-(time.monotonic()-started))))
                        child_reaped=True
                    except subprocess.TimeoutExpired:
                        pass
                    except BaseException:
                        cleanup_errors.append('wait_failed')
                    if not group_absent(child.pid):
                        try:
                            os.killpg(child.pid,signal.SIGKILL)
                            signals.append({'signal':'KILL','elapsed_seconds':round(time.monotonic()-started,6)})
                        except ProcessLookupError:
                            pass
                        except BaseException:
                            cleanup_errors.append('kill_failed')
                try:
                    returncode=child.wait(timeout=min(5,max(0.01,1798-(time.monotonic()-started))))
                    child_reaped=True
                except BaseException:
                    cleanup_errors.append('final_reap_failed')
                group_gone=group_absent(child.pid)
                child.stdout.close()
            sel.close()
            log.flush()
            os.fsync(log.fileno())
    samples.append({'elapsed_seconds':round(time.monotonic()-started,6),'free_bytes':free_bytes()})
    raw=log_path.read_bytes()
    retained={'source_archive':archive_id,'source_head':current_head,'command':DISPLAY_COMMAND,'launch_free_bytes':launch_free,'disk_samples':samples,'disk_minimum_bytes':min(s['free_bytes'] for s in samples),'elapsed_seconds':round(time.monotonic()-started,6),'child_pid':None if child is None else child.pid,'child_pgid':None if child is None else child.pid,'owned_group_verified':owned_group,'child_returncode':returncode,'child_reaped':child_reaped,'owned_group_absent':group_gone,'signals':signals,'stop_reason':stop_reason,'internal_failure':internal_failure,'cleanup_errors':cleanup_errors,'unexpected_dependency_download_observed':downloaded,'log_capped':log_capped,'stream_eof':stream_eof,'log':{'path':str(log_path.relative_to(ROOT)),'bytes':len(raw),'sha256':digest(raw),'mode':'0600'},'output_retained_before_grade':True}
    result_id=exclusive_json('actual-status.redacted.json',retained)
    # Grading starts only after the complete bounded output and numeric status are fsynced.
    counts_matches=re.findall(rb'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',raw)
    counts_match=counts_matches[0] if len(counts_matches)==1 else None
    counts=None if counts_match is None else {'passed':int(counts_match[1]),'failed':int(counts_match[2]),'ignored':int(counts_match[3]),'measured':int(counts_match[4]),'filtered_out':int(counts_match[5])}
    panic_matches=re.findall(rb'panicked at (tests/device_trust\.rs):(\d{1,6}):(\d{1,6}):',raw)
    panic_locations=[{'path':'tests/device_trust.rs','line':int(match[1]),'column':int(match[2])} for match in panic_matches[:14]]
    unexpected_ok_panic_count=raw.count(b'called `Result::unwrap_err()` on an `Ok` value')
    whole_target_complete=bool(counts and counts['passed']+counts['failed']==14 and counts['ignored']==0 and counts['measured']==0 and counts['filtered_out']==0 and child_reaped and group_gone and stream_eof and not log_capped and not downloaded and stop_reason is None and internal_failure is None and not cleanup_errors)
    whole_target_pass=bool(whole_target_complete and returncode==0 and counts_match[0]==b'ok' and counts['passed']==14 and counts['failed']==0 and not panic_matches and unexpected_ok_panic_count==0)
    classification='pass' if whole_target_pass else 'test_failure' if whole_target_complete and counts_match[0]==b'FAILED' and returncode==101 and counts['failed']>0 else 'incomplete'
    grade={'actual_status':result_id,'test_counts':counts,'summary_count':len(counts_matches),'expected_test_count':14,'panic_locations':panic_locations,'panic_location_count':len(panic_matches),'panic_locations_capped':len(panic_matches)>14,'unexpected_ok_panic_count':unexpected_ok_panic_count,'whole_target_complete':whole_target_complete,'whole_target_pass':whole_target_pass,'whole_target_classification':classification,'resource_release_proven':bool(child_reaped and group_gone),'child_returncode':returncode,'stop_reason':stop_reason,'internal_failure':internal_failure,'cleanup_errors':cleanup_errors,'elapsed_seconds':retained['elapsed_seconds'],'disk_minimum_bytes':retained['disk_minimum_bytes'],'log_bytes':len(raw),'log_sha256':digest(raw),'source_archive':archive_id}
    grade_id=exclusive_json('grade.redacted.json',grade)
    print(json.dumps({'grade':grade,'grade_file':grade_id},sort_keys=True),flush=True)
    return 0 if child_reaped and group_gone and internal_failure is None and not cleanup_errors else 125


if __name__ == '__main__':
    sys.exit(main())
```

### Exact forward diff; its inverse restores all original bytes/AST

```diff
--- original-negative-supervisor.py
+++ proposed-whole-target-supervisor.py
@@ -11,15 +11,15 @@
 import time
␠
 ROOT = Path('/Users/dominik/orca/projects/riAuth-public-preview-sol-management-wave30')
-RUN = Path(__file__).resolve().parent
-HEAD = 'be6d4afe932bf03454d6b0dedfa482c8a2e33b4f'
-REVIEW = '6ba2e95c42c271804d1715eb05380953b008130d'
-FIXED = '544d1340b80cd3e040dc13142cdcbc1d75fea4cb'
-TEST_SHA = 'ee9c1676e8fb3aa6f6d2092a343e70bb0beae703d563d3e410ad611093e340dd'
+RUN = ROOT / 'deployment-private/i07-device-trust-whole-5c20ca-20261003-once'
+HEAD = '715f6e9c37cf3e6a92f13f284fa54e4cb6d8d0fd'
+REVIEW = '5c20ca13effb28501976aef69e50c6738ab560b6'
+FIXED = '5c20ca13effb28501976aef69e50c6738ab560b6'
+TEST_SHA = 'f5fc6b3d42f97bf7c4601d8228e713066af1e3a95eeee3aca60dfeeec4c7f5df'
 GIB = 1024 ** 3
 LOG_CAP = 16 * 1024 ** 2
-DISPLAY_COMMAND = 'env CARGO_TARGET_DIR="$PWD/target/wave30-o06-readiness" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support --test device_trust google::retained_local_proof_requires_reverification_after_provider_change -- --exact --test-threads=1'
-ARGV = ['env', 'CARGO_TARGET_DIR='+str(ROOT/'target/wave30-o06-readiness'), 'CARGO_BUILD_JOBS=1', 'CARGO_INCREMENTAL=0', 'CARGO_PROFILE_DEV_DEBUG=0', 'CARGO_PROFILE_TEST_DEBUG=0', 'cargo', 'test', '--locked', '--features', 'test-support', '--test', 'device_trust', 'google::retained_local_proof_requires_reverification_after_provider_change', '--', '--exact', '--test-threads=1']
+DISPLAY_COMMAND = 'env CARGO_TARGET_DIR="$PWD/target/wave30-o06-readiness" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support --test device_trust -- --test-threads=1'
+ARGV = ['env', 'CARGO_TARGET_DIR='+str(ROOT/'target/wave30-o06-readiness'), 'CARGO_BUILD_JOBS=1', 'CARGO_INCREMENTAL=0', 'CARGO_PROFILE_DEV_DEBUG=0', 'CARGO_PROFILE_TEST_DEBUG=0', 'cargo', 'test', '--locked', '--features', 'test-support', '--test', 'device_trust', '--', '--test-threads=1']
␠
␠
 def digest(data):
@@ -69,16 +69,21 @@
     os.umask(0o077)
     assert RUN.stat().st_uid==os.getuid() and stat.S_IMODE(RUN.stat().st_mode)==0o700
     assert not RUN.is_symlink()
-    assert read_git(['rev-parse','HEAD']).decode().strip()==HEAD
+    current_head=read_git(['rev-parse','HEAD']).decode().strip()
+    assert read_git(['merge-base',HEAD,current_head]).decode().strip()==HEAD
+    assert read_git(['rev-parse',HEAD+'^{tree}'])==read_git(['rev-parse',FIXED+'^{tree}'])
     assert read_git(['status','--porcelain'])==b''
     test=(ROOT/'tests/device_trust.rs').read_bytes()
     fixed=read_git(['show',FIXED+':tests/device_trust.rs'])
     assert digest(test)==TEST_SHA
-    assert read_git(['hash-object','tests/device_trust.rs']).decode().strip()=='5c9d9401961033805745335ac353309237e9ba65'
+    assert read_git(['hash-object','tests/device_trust.rs']).decode().strip()=='54bc47d08b5562ed6d836df55144c93b5ac8dd2d'
     assert test==read_git(['show',REVIEW+':tests/device_trust.rs'])
-    assert test[:len(fixed)-2]+test[-2:]==fixed
-    changed=set(read_git(['diff','--name-only',REVIEW,'HEAD']).decode().splitlines())
-    assert changed=={'docs/roadmap/local-wave30-d01-user-browser-review.md','docs/roadmap/local-wave30-i07-managed-device-plan.md'}
+    assert test==fixed
+    changed=set(read_git(['diff','--name-only',HEAD,current_head]).decode().splitlines())
+    assert changed<={'docs/roadmap/local-wave30-i07-managed-device-plan.md'}
+    report=(ROOT/'docs/roadmap/local-wave30-i07-managed-device-plan.md').read_bytes()
+    report_base=read_git(['show',FIXED+':docs/roadmap/local-wave30-i07-managed-device-plan.md'])
+    assert report[:len(report_base)]==report_base
     inputs={}
     for path in ['Cargo.toml','Cargo.lock','rust-toolchain.toml']:
         data=(ROOT/path).read_bytes()
@@ -99,7 +104,7 @@
         competitors[name]={'returncode':observed.returncode,'count':0}
     initial_free=free_bytes()
     assert initial_free>=12*GIB
-    archive={'project':'891e7443-8dac-4c1b-897f-9e53cb59c7ee','reservation':'wave30_I07_provider_transition_negative_once','source_head':HEAD,'source_tree':read_git(['rev-parse','HEAD^{tree}']).decode().strip(),'reviewed_test_commit':REVIEW,'test_blob':'5c9d9401961033805745335ac353309237e9ba65','test_sha256':TEST_SHA,'test_bytes':len(test),'fixed544_test_sha256':digest(fixed),'whole_test_reversal':True,'tracked_production_manifests_toolchain_unchanged':True,'inputs':inputs,'command':DISPLAY_COMMAND,'argv':ARGV,'features':features,'warm_target_fingerprint_sha256':digest(fingerprint),'initial_free_bytes':initial_free,'minimum_launch_bytes':12*GIB,'owned_stop_bytes':9*GIB,'floor_bytes':8*GIB,'sample_interval_seconds':2,'inclusive_bound_seconds':1800,'active_stop_seconds':1765,'log_cap_bytes':LOG_CAP,'competitors':competitors,'supervisor_sha256':digest(Path(__file__).read_bytes()),'supervisor_pid':os.getpid(),'sigchld_default_before_spawn':signal.getsignal(signal.SIGCHLD)==signal.SIG_DFL}
+    archive={'project':'891e7443-8dac-4c1b-897f-9e53cb59c7ee','reservation':'wave30_I07_whole_target_exact_preparation','source_head':current_head,'alignment_merge':HEAD,'accepted_source_pin':FIXED,'source_tree':read_git(['rev-parse','HEAD^{tree}']).decode().strip(),'reviewed_test_commit':REVIEW,'test_blob':'54bc47d08b5562ed6d836df55144c93b5ac8dd2d','test_sha256':TEST_SHA,'test_bytes':len(test),'accepted_test_sha256':digest(fixed),'accepted_test_equality':True,'report_baseline_sha256':digest(report_base),'current_report_sha256':digest(report),'expected_test_count':14,'tracked_production_manifests_toolchain_unchanged':True,'inputs':inputs,'command':DISPLAY_COMMAND,'argv':ARGV,'features':features,'warm_target_fingerprint_sha256':digest(fingerprint),'initial_free_bytes':initial_free,'minimum_launch_bytes':12*GIB,'owned_stop_bytes':9*GIB,'floor_bytes':8*GIB,'sample_interval_seconds':2,'inclusive_bound_seconds':1800,'active_stop_seconds':1765,'log_cap_bytes':LOG_CAP,'competitors':competitors,'supervisor_sha256':digest(Path(__file__).read_bytes()),'supervisor_pid':os.getpid(),'sigchld_default_before_spawn':signal.getsignal(signal.SIGCHLD)==signal.SIG_DFL}
     archive_id=exclusive_json('launch-source.redacted.json',archive)
     launch_free=free_bytes()
     assert launch_free>=12*GIB
@@ -231,14 +236,19 @@
             os.fsync(log.fileno())
     samples.append({'elapsed_seconds':round(time.monotonic()-started,6),'free_bytes':free_bytes()})
     raw=log_path.read_bytes()
-    retained={'source_archive':archive_id,'source_head':HEAD,'command':DISPLAY_COMMAND,'launch_free_bytes':launch_free,'disk_samples':samples,'disk_minimum_bytes':min(s['free_bytes'] for s in samples),'elapsed_seconds':round(time.monotonic()-started,6),'child_pid':None if child is None else child.pid,'child_pgid':None if child is None else child.pid,'owned_group_verified':owned_group,'child_returncode':returncode,'child_reaped':child_reaped,'owned_group_absent':group_gone,'signals':signals,'stop_reason':stop_reason,'internal_failure':internal_failure,'cleanup_errors':cleanup_errors,'unexpected_dependency_download_observed':downloaded,'log_capped':log_capped,'stream_eof':stream_eof,'log':{'path':str(log_path.relative_to(ROOT)),'bytes':len(raw),'sha256':digest(raw),'mode':'0600'},'output_retained_before_grade':True}
+    retained={'source_archive':archive_id,'source_head':current_head,'command':DISPLAY_COMMAND,'launch_free_bytes':launch_free,'disk_samples':samples,'disk_minimum_bytes':min(s['free_bytes'] for s in samples),'elapsed_seconds':round(time.monotonic()-started,6),'child_pid':None if child is None else child.pid,'child_pgid':None if child is None else child.pid,'owned_group_verified':owned_group,'child_returncode':returncode,'child_reaped':child_reaped,'owned_group_absent':group_gone,'signals':signals,'stop_reason':stop_reason,'internal_failure':internal_failure,'cleanup_errors':cleanup_errors,'unexpected_dependency_download_observed':downloaded,'log_capped':log_capped,'stream_eof':stream_eof,'log':{'path':str(log_path.relative_to(ROOT)),'bytes':len(raw),'sha256':digest(raw),'mode':'0600'},'output_retained_before_grade':True}
     result_id=exclusive_json('actual-status.redacted.json',retained)
     # Grading starts only after the complete bounded output and numeric status are fsynced.
-    counts_match=re.search(rb'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',raw)
-    panic_match=re.search(rb"thread 'google::retained_local_proof_requires_reverification_after_provider_change' panicked at (tests/device_trust\.rs):(\d+):(\d+):",raw)
-    oracle=bool(panic_match and int(panic_match[2])==1821 and b'called `Result::unwrap_err()` on an `Ok` value' in raw)
-    counts=None if counts_match is None else {'passed':int(counts_match[2]),'failed':int(counts_match[3]),'ignored':int(counts_match[4]),'measured':int(counts_match[5]),'filtered_out':int(counts_match[6])}
-    grade={'actual_status':result_id,'test_counts':counts,'panic_path':None if panic_match is None else 'tests/device_trust.rs','panic_line':None if panic_match is None else int(panic_match[2]),'panic_column':None if panic_match is None else int(panic_match[3]),'expected_oracle_reached':oracle or bool(counts and counts['passed']==1 and counts['failed']==0),'expected_unwrap_err_received_ok':oracle,'strict_error_assertion_reached':bool(counts and counts['passed']==1 and counts['failed']==0),'unprotected_control_reached':bool(counts and counts['passed']==1 and counts['failed']==0),'resource_release_proven':bool(child_reaped and group_gone),'child_returncode':returncode,'stop_reason':stop_reason,'internal_failure':internal_failure,'cleanup_errors':cleanup_errors,'elapsed_seconds':retained['elapsed_seconds'],'disk_minimum_bytes':retained['disk_minimum_bytes'],'log_bytes':len(raw),'log_sha256':digest(raw),'source_archive':archive_id}
+    counts_matches=re.findall(rb'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',raw)
+    counts_match=counts_matches[0] if len(counts_matches)==1 else None
+    counts=None if counts_match is None else {'passed':int(counts_match[1]),'failed':int(counts_match[2]),'ignored':int(counts_match[3]),'measured':int(counts_match[4]),'filtered_out':int(counts_match[5])}
+    panic_matches=re.findall(rb'panicked at (tests/device_trust\.rs):(\d{1,6}):(\d{1,6}):',raw)
+    panic_locations=[{'path':'tests/device_trust.rs','line':int(match[1]),'column':int(match[2])} for match in panic_matches[:14]]
+    unexpected_ok_panic_count=raw.count(b'called `Result::unwrap_err()` on an `Ok` value')
+    whole_target_complete=bool(counts and counts['passed']+counts['failed']==14 and counts['ignored']==0 and counts['measured']==0 and counts['filtered_out']==0 and child_reaped and group_gone and stream_eof and not log_capped and not downloaded and stop_reason is None and internal_failure is None and not cleanup_errors)
+    whole_target_pass=bool(whole_target_complete and returncode==0 and counts_match[0]==b'ok' and counts['passed']==14 and counts['failed']==0 and not panic_matches and unexpected_ok_panic_count==0)
+    classification='pass' if whole_target_pass else 'test_failure' if whole_target_complete and counts_match[0]==b'FAILED' and returncode==101 and counts['failed']>0 else 'incomplete'
+    grade={'actual_status':result_id,'test_counts':counts,'summary_count':len(counts_matches),'expected_test_count':14,'panic_locations':panic_locations,'panic_location_count':len(panic_matches),'panic_locations_capped':len(panic_matches)>14,'unexpected_ok_panic_count':unexpected_ok_panic_count,'whole_target_complete':whole_target_complete,'whole_target_pass':whole_target_pass,'whole_target_classification':classification,'resource_release_proven':bool(child_reaped and group_gone),'child_returncode':returncode,'stop_reason':stop_reason,'internal_failure':internal_failure,'cleanup_errors':cleanup_errors,'elapsed_seconds':retained['elapsed_seconds'],'disk_minimum_bytes':retained['disk_minimum_bytes'],'log_bytes':len(raw),'log_sha256':digest(raw),'source_archive':archive_id}
     grade_id=exclusive_json('grade.redacted.json',grade)
     print(json.dumps({'grade':grade,'grade_file':grade_id},sort_keys=True),flush=True)
     return 0 if child_reaped and group_gone and internal_failure is None and not cleanup_errors else 125
```

The three standalone `␠` lines in this diff archive each encode exactly ONE ASCII space followed by LF (U+0020 U+000A), the unified-diff blank-context marker. Decode only those three complete lines before applying or inverting the diff. The decoded canonical diff is exactly12460 bytes, SHA256 `335f4ec5880e0c1a15c68cab1dfd2c1079be13620b4a579b03c805e88bb9a265`; the executable candidate archive is unchanged. This explicit rendering avoids report trailing whitespace while retaining every canonical diff byte.

Apply the decoded canonical diff once to the hash-checked complete original source to reconstruct the candidate. Reversing these exact hunks restores ALL14537 original bytes and its complete location-independent AST; no unchanged body/snapshot exclusions are used. All FIVE helper functions (`digest`, `exclusive_json`, `read_git`, `free_bytes`, `group_absent`) are byte-exact. SIGCHLD-default installation remains the first main statement before ANY Popen. The complete child-loop/read/drain/TERM/KILL/reap/final-status retention source span remains byte-exact, except the declared current-head metadata substitution. Thresholds12GiB/9GiB/8GiB, nominal2s sampling,16MiB log cap,1765s active stop/1790s loop cleanup limit/1798s bounded final waits inside the declared1800s budget are unchanged. No clock reset, retry, fallback, cold target, guard or output-cap widening is added. This is a source preservation proof, not fresh timeout/cleanup enforcement or runtime validation.

All runtime paths would be beneath the declared NEW `deployment-private/i07-device-trust-whole-5c20ca-20261003-once`, verified ABSENT during preparation. It was NOT created. A later root release must authorize private0700 creation, exclusive0600 supervisor/source/archive/log/child/status/grade files there and the exact candidate hash. Existing eight negative-run files and all older outputs remain separate. The original exclusive file helpers and numeric owned PID/PGID/readbacks are unchanged. No arbitrary exception, response, key/ID, panic value or body is exposed publicly.

The whole-target classifier requires exactlyONE fixed test-summary match,14 executed passed+failed, zero ignored/measured/filtered, complete EOF/nontruncated retained output, no download/stop/internal/cleanup failure and joined absent ownership. A pass additionally requires actual Cargo0, summaryok,14 passed/0 failed and zero fixed panic/unwrap-error-on-Ok signals. FAILED summary plus Cargo101/nonzero failed count yields only fixed `test_failure`; anything partial/inconsistent is `incomplete`. Fixed target panic path/line/column and finite counts are public; raw unexpected-Ok response debug data remains only in capped0600 private output. No strict error/plain journey assertions are retroactively credited from a failed/partial run, and resource release remains separate from a passing test result. Source-defined14 is an expectation, not observed discovery.

The single command remains `env CARGO_TARGET_DIR="$PWD/target/wave30-o06-readiness" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support --test device_trust -- --test-threads=1`. Display and expanded env argv differ from the negative command ONLY by removing the name and --exact. Root must fully review this immutable source before any separately released ONE invocation. Sole validation/Cargo runtime remains HELD/free and unused by this phase; no native/provider/keygen/version/compiler/Cargo/test/browser/Driver/D01 store/real-fixture action ran, no output leaf or child/process group was created, and no slot was acquired/released. Root alone owns original gate/status/integration/publication/runtime decisions.

### Fresh read-only prerequisites and static check disclosure

At `2026-10-03T08:54:45.245184+00:00`, free capacity was14808350720 bytes (13.791351GiB), above the proposed12GiB launch minimum. It is a shared observation and is not reserved. Exact-name cargo and rustc observations each returned exit1/count0; no process arguments/environment or unrelated process signals were read or used. Root must recheck capacity, absence of competing Cargo/rustc and source/cache identity at any future separately released launch; stop instead of lowering thresholds, downloading unexpectedly, pruning or duplicating a cold target.

The existing owned `target/wave30-o06-readiness` cache was read as metadata only. `.rustc_info.json` is1965 bytes/SHA256 `27df402be20083ab5b4835c05762e2b77beed67288686dd8fda9193b24cdd7c4`; the library fingerprint is3611 bytes/SHA256 `ac6f00d7f80260b8f443564cbbdd8e64f91d91aa42f8d001c0a23e13778f1c10`, and the device_trust fingerprint is4019 bytes/SHA256 `f078dcd7d024389e415d7624eac3f0cedd05994bc36619bfd551a526795c4ee7`. Both record features `["default", "essentials", "platform", "test-support"]`, rustc identity17329007180185699724, compile_kind0 and empty rustflags; library/test profile identities are12672335563272108896/11094973624911973823. This is an existing same-feature warm-cache prerequisite, not a build/discovery/result on corrected source. No compiler or toolchain-version invocation occurred.

The first report whitespace check emitted three trailing-whitespace diagnostics at then-lines854,872,873, all the canonical diff's blank context lines. The compound shell's exit0 came from its later static Python check and did NOT establish that the whitespace command passed; its individual numeric exit was not separately retained. Those three archive-rendering lines are now explicitly encoded as described above; the candidate, decoded canonical diff and original report prefix remain exact. The failure is preserved here, rather than treated as a source or runtime result. `python3 scripts/check-docs.py` already returned actual exit0. Final isolated whitespace/docs/scope/hash/inverse checks follow this rendering correction and remain static-only.

Final isolated checks after the rendering correction returned actual exit0 individually: `git diff --check`, `python3 scripts/check-docs.py`, and the static alignment/scope/source/archive checker. The latter verified both merge parents and the complete equal accepted tree, the only changed report path, the whole prior89006-byte prefix, candidate15555B/258lines, decoded four-hunk diff12460B, exact forward/reverse whole-source bytes and AST, all five unchanged helper function bodies, SIGCHLD-default as the first main statement, six current source/guide/helper witnesses and the absent future runtime leaf. Candidate/controller/harness/product code was not imported or evaluated; no child or runtime output was created. These checks establish source preparation only, not compilation, discovered test count or a target result.

Fresh final read-only prerequisite sample (unreserved; no process arguments/environment or signals):

```json
{
  "capacity_reserved": false,
  "exact_name_observations": {
    "cargo": {
      "exact_name_count": 0,
      "exit": 1
    },
    "rustc": {
      "exact_name_count": 0,
      "exit": 1
    }
  },
  "free_bytes": 13581897728,
  "free_gib": 12.649128,
  "launch_12gib_met_at_sample": true,
  "runtime_released": false,
  "utc": "2026-10-03T09:02:22.736836+00:00"
}
```

## 2026-10-03: unreaped-leader whole-target supervisor correction, SOURCE ONLY / UNRUN

Root's new source review found a concrete generation-ownership counterexample in the dated64e4 controller: `child.poll()` consumed the leader at old lines153–156 before nonzero group signals at162–170 and204–222. A numeric `killpg(..., 0)` absence check cannot retain generation identity after that reap. This is a source finding only: no reused PID, unrelated signal or new target execution is observed, and it is not an attribution for the earlier failed negative run. The complete129064-byte64e4 report prefix, original dated source/archive/canonical diff, formatting/whitespace failures, negative runtime/parser miss and prior captures remain unchanged.

Full root ownership receipt was read as public DATA from `planning/evidence/wave30-i07-whole-target-controller-root-ownership-review.json` in the project orchestrator workspace. `CONTRIBUTING.md` and `SECURITY.md` were read fully; no local AGENTS file was listed by the scoped inventory. From immutable `9fdc157b7ef09d2998854cf92bea31ea26ce0bf5:docs/roadmap/local-wave30-i08-windows-lifecycle-plan.md`, only the relevant wait-observer preflight, ended/signal_group/finish_group and bounded cleanup caller/preservation commentary at lines963–977,1160–1310,1373–1411,1557–1574,1657–1665 were body-read. The complete I08 report was hashed as98172B/SHA256 `58d568b61f3589e0643d8d6d4f591b485b3f574e7f5b41ceac82d2434df83f42`; its broader observer/resource/toolchain scope was not borrowed. Revision `3ae` is not available in this worktree (Git lookup exit128), so no body or execution is credited from it. An initial optional-path inventory lookup also returned exit2 when searching absent `planning`; subsequent exact source reads above succeeded. Neither lookup is a runtime/product finding.

### Exact source manifest

```json
{
  "blank_context_lines_encoded": 1,
  "candidate_bytes": 20011,
  "candidate_lines": 354,
  "candidate_sha256": "7f880d6ac9324d08b3a9022346d42836bdaf3258adeba34474a56daa8744d236",
  "canonical_diff_bytes": 16589,
  "canonical_diff_sha256": "592e2864966216b59a217f1138bb516bf8a833f06c90295f3cf14be1089cba8b",
  "exact_text_transforms": 6,
  "old_controller_bytes": 15555,
  "old_controller_sha256": "e88102b020534c334d6452cd8b2a2edb45e4c54fd6a395d82ff5416cd4e275bc",
  "prior_report_bytes": 129064,
  "prior_report_commit": "64e4f9636aa60fa8f28fdb527d35f6c76e62bf26",
  "prior_report_sha256": "ce8bd2c1ee9ef6d2c42d53f02003770ece5273921d4999075e799de55fcc1d55",
  "project": "891e7443-8dac-4c1b-897f-9e53cb59c7ee",
  "reservation": "wave30_I07_unreaped_leader_supervisor_correction",
  "resource_acquired_or_released": false,
  "root_review_receipt_bytes": 1025,
  "root_review_receipt_sha256": "04f3dcc0c4d18ab4082b91decca8622e939b52af243508473506aa81bb5f810a",
  "runtime_executed": false,
  "runtime_leaf_created": false
}
```

The correction has six exact text substitutions: waitability/default-handler capability preflight; unreaped-state/finite helper definitions; immediate pre-spawn default-handler recheck; nonconsuming active/terminal cleanup loop; bounded cleanup followed by direct consuming wait/readback; and finite retained status fields. All top-level imports/constants/paths/pins/command/cache/feature/source checks and five original top-level helper bodies are byte-exact. The original complete log-reader AST is unchanged except `sel.select(0.2)` becomes its bounded caller-supplied timeout. All remaining bytes after final disk sampling, including private fsync-before-grade ordering and whole14 grader, are exact except the declared four added finite status fields.

At a future released invocation, SIGCHLD default is installed as the first main statement, verified with `getsignal`, and reverified immediately before the sole Cargo Popen. Native `waitid`, WNOWAIT/P_PID/WEXITED/WNOHANG, terminal CLD codes, direct waitpid and status conversion must exist; otherwise preflight refuses BEFORE Cargo spawn. Availability is a future conditional prerequisite, not an observed capability/result of this source-only phase. `observe_leader()` uses P_PID plus WEXITED/WNOHANG/WNOWAIT, accepts only no terminal result or the exact child's terminal CLD result, never consumes status, and poisons subsequent signal eligibility on any observation/guard exception. Both a live direct child and its reserved zombie remain waitable; a zombie is not labelled as a remaining live-child failure. No `child.poll`, `child.wait`, `child.kill`, `child.terminate` or `child.send_signal` call remains.

Every nonzero group signal occurs at the sole `signal_group` sink, after the original `os.getpgid(child.pid)==child.pid` group proof plus a fresh successful nonconsuming waitability/default-handler guard with unreaped child and open signal phase. Observation failure refuses all subsequent nonzero signals; there is no numeric-identity fallback. Each bounded TERM/KILL intent is recorded before the guarded call, with fixed outcome `ownership_refused`, `attempting`, `sent`, `group_absent` or `unconfirmed` and finite unreaped-verification bool. `unconfirmed` does not assert delivery. The conservative normal-terminal path also requests TERM, drains within five-second grace, then requests KILL before consuming status; future observed signals must be disclosed even when Cargo had already terminated. Each signal type is attempted at most once, deadlines are not renewed, and observation failure cannot be repaired by a second observation or guessed kill.

The original active1765s, loop/cleanup1790s and final1798s absolute clocks within declared1800s remain; initial12GiB, stop9GiB, floor8GiB, nominal2s observations,16MiB cap, download refusal and single fixed command/pins are unchanged. The same2s disk observation body is used during bounded cleanup/drain, never a reset of START. TERM grace is bounded by the earlier of term_at+5, first_stop+20 and START+1790; exceptions may remove remaining grace. `signal_phase_closed=True` occurs before the sole consuming `os.waitpid(child.pid, WNOHANG)` site; no later path can reopen it or call the signal sink. Reaping/draining uses one deadline bounded by START+1798 and five seconds from entry. The real matching waitpid PID/status alone sets child_reaped, retains raw numeric wait status and supplies `waitstatus_to_exitcode`; no ECHILD-to-zero synthesis or inferred numeric exit is used. A failed/missing wait leaves exit null and cannot pass. The Popen returncode cache is assigned only from that actual conversion. After consumption, only signal0 absence readbacks and bounded pipe draining occur; any remaining/reused/unknown numeric group is a failed cleanup observation, never a reason to signal again.

Original first stop_reason and internal failure are never overwritten by cleanup exceptions. Cleanup retains separate fixed labels, numeric wait status and finite observer/signal-phase fields; no raw exception/request/response/key/private panic value is emitted. Passing remains the unchanged fourteen-result/actual-Cargo0/complete-output/no-error classifier. Actual child/group absence is separate from test success. If unreaped ownership cannot be confirmed, no guessed signal is authorized; a still-live or unjoinable child is an incomplete cleanup with release unproven, not a fabricated pass.

### Complete corrected proposed supervisor (UNRUN)

```python
import hashlib
import json
import os
from pathlib import Path
import re
import selectors
import signal
import stat
import subprocess
import sys
import time

ROOT = Path('/Users/dominik/orca/projects/riAuth-public-preview-sol-management-wave30')
RUN = ROOT / 'deployment-private/i07-device-trust-whole-5c20ca-20261003-once'
HEAD = '715f6e9c37cf3e6a92f13f284fa54e4cb6d8d0fd'
REVIEW = '5c20ca13effb28501976aef69e50c6738ab560b6'
FIXED = '5c20ca13effb28501976aef69e50c6738ab560b6'
TEST_SHA = 'f5fc6b3d42f97bf7c4601d8228e713066af1e3a95eeee3aca60dfeeec4c7f5df'
GIB = 1024 ** 3
LOG_CAP = 16 * 1024 ** 2
DISPLAY_COMMAND = 'env CARGO_TARGET_DIR="$PWD/target/wave30-o06-readiness" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support --test device_trust -- --test-threads=1'
ARGV = ['env', 'CARGO_TARGET_DIR='+str(ROOT/'target/wave30-o06-readiness'), 'CARGO_BUILD_JOBS=1', 'CARGO_INCREMENTAL=0', 'CARGO_PROFILE_DEV_DEBUG=0', 'CARGO_PROFILE_TEST_DEBUG=0', 'cargo', 'test', '--locked', '--features', 'test-support', '--test', 'device_trust', '--', '--test-threads=1']


def digest(data):
    return hashlib.sha256(data).hexdigest()


def exclusive_json(name, obj):
    data=(json.dumps(obj,sort_keys=True,indent=2)+'\n').encode('ascii')
    assert len(data)<LOG_CAP
    fd=os.open(RUN/name,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
    try:
        with os.fdopen(fd,'wb') as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        parent=os.open(RUN,os.O_RDONLY)
        try:
            os.fsync(parent)
        finally:
            os.close(parent)
    except BaseException:
        raise
    return {'path':str((RUN/name).relative_to(ROOT)), 'bytes':len(data), 'sha256':digest(data), 'mode':'0600'}


def read_git(args):
    return subprocess.check_output(['git',*args],cwd=ROOT,timeout=5)


def free_bytes():
    v=os.statvfs(ROOT)
    return v.f_bavail*v.f_frsize


def group_absent(pgid):
    try:
        os.killpg(pgid,0)
        return False
    except ProcessLookupError:
        return True
    except PermissionError:
        return False


def main():
    signal.signal(signal.SIGCHLD, signal.SIG_DFL)
    assert signal.getsignal(signal.SIGCHLD)==signal.SIG_DFL
    assert all(hasattr(os,name) for name in ('waitid','P_PID','WEXITED','WNOHANG','WNOWAIT','CLD_EXITED','CLD_KILLED','CLD_DUMPED','waitpid','waitstatus_to_exitcode'))
    assert callable(os.waitid) and callable(os.waitpid) and callable(os.waitstatus_to_exitcode)
    os.umask(0o077)
    assert RUN.stat().st_uid==os.getuid() and stat.S_IMODE(RUN.stat().st_mode)==0o700
    assert not RUN.is_symlink()
    current_head=read_git(['rev-parse','HEAD']).decode().strip()
    assert read_git(['merge-base',HEAD,current_head]).decode().strip()==HEAD
    assert read_git(['rev-parse',HEAD+'^{tree}'])==read_git(['rev-parse',FIXED+'^{tree}'])
    assert read_git(['status','--porcelain'])==b''
    test=(ROOT/'tests/device_trust.rs').read_bytes()
    fixed=read_git(['show',FIXED+':tests/device_trust.rs'])
    assert digest(test)==TEST_SHA
    assert read_git(['hash-object','tests/device_trust.rs']).decode().strip()=='54bc47d08b5562ed6d836df55144c93b5ac8dd2d'
    assert test==read_git(['show',REVIEW+':tests/device_trust.rs'])
    assert test==fixed
    changed=set(read_git(['diff','--name-only',HEAD,current_head]).decode().splitlines())
    assert changed<={'docs/roadmap/local-wave30-i07-managed-device-plan.md'}
    report=(ROOT/'docs/roadmap/local-wave30-i07-managed-device-plan.md').read_bytes()
    report_base=read_git(['show',FIXED+':docs/roadmap/local-wave30-i07-managed-device-plan.md'])
    assert report[:len(report_base)]==report_base
    inputs={}
    for path in ['Cargo.toml','Cargo.lock','rust-toolchain.toml']:
        data=(ROOT/path).read_bytes()
        assert data==read_git(['show',REVIEW+':'+path])
        inputs[path]={'bytes':len(data),'sha256':digest(data)}
    target=ROOT/'target/wave30-o06-readiness'
    assert target.is_dir() and not target.is_symlink() and target.stat().st_uid==os.getuid()
    fingerprint_path=target/'debug/.fingerprint/riauth-0d65ce4b80b71950/lib-riauth.json'
    fingerprint=fingerprint_path.read_bytes()
    assert digest(fingerprint)=='ac6f00d7f80260b8f443564cbbdd8e64f91d91aa42f8d001c0a23e13778f1c10'
    features=json.loads(fingerprint)['features']
    assert features=='["default", "essentials", "platform", "test-support"]'
    assert (target/'debug/deps/libriauth-0d65ce4b80b71950.rlib').is_file()
    competitors={}
    for name in ['cargo','rustc']:
        observed=subprocess.run(['pgrep','-x',name],capture_output=True,timeout=3)
        assert observed.returncode==1 and observed.stdout==b'' and observed.stderr==b''
        competitors[name]={'returncode':observed.returncode,'count':0}
    initial_free=free_bytes()
    assert initial_free>=12*GIB
    archive={'project':'891e7443-8dac-4c1b-897f-9e53cb59c7ee','reservation':'wave30_I07_whole_target_exact_preparation','source_head':current_head,'alignment_merge':HEAD,'accepted_source_pin':FIXED,'source_tree':read_git(['rev-parse','HEAD^{tree}']).decode().strip(),'reviewed_test_commit':REVIEW,'test_blob':'54bc47d08b5562ed6d836df55144c93b5ac8dd2d','test_sha256':TEST_SHA,'test_bytes':len(test),'accepted_test_sha256':digest(fixed),'accepted_test_equality':True,'report_baseline_sha256':digest(report_base),'current_report_sha256':digest(report),'expected_test_count':14,'tracked_production_manifests_toolchain_unchanged':True,'inputs':inputs,'command':DISPLAY_COMMAND,'argv':ARGV,'features':features,'warm_target_fingerprint_sha256':digest(fingerprint),'initial_free_bytes':initial_free,'minimum_launch_bytes':12*GIB,'owned_stop_bytes':9*GIB,'floor_bytes':8*GIB,'sample_interval_seconds':2,'inclusive_bound_seconds':1800,'active_stop_seconds':1765,'log_cap_bytes':LOG_CAP,'competitors':competitors,'supervisor_sha256':digest(Path(__file__).read_bytes()),'supervisor_pid':os.getpid(),'sigchld_default_before_spawn':signal.getsignal(signal.SIGCHLD)==signal.SIG_DFL}
    archive_id=exclusive_json('launch-source.redacted.json',archive)
    launch_free=free_bytes()
    assert launch_free>=12*GIB
    log_path=RUN/'cargo-combined.private.log'
    log_fd=os.open(log_path,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
    started=time.monotonic()
    samples=[{'elapsed_seconds':0.0,'free_bytes':launch_free}]
    stop_reason=None
    signals=[]
    child=None
    child_reaped=False
    owned_group=False
    group_gone=False
    bytes_written=0
    downloaded=False
    log_capped=False
    cleanup_errors=[]
    carry=b''
    sel=selectors.DefaultSelector()
    stream_eof=False
    first_stop=None
    returncode=None
    internal_failure=None
    leader_exit_observed=False
    leader_observation_failed=False
    signal_phase_closed=False
    child_wait_status=None
    next_sample=started+2
    term_at=None
    kill_at=None

    def sample_disk(now):
        nonlocal next_sample,stop_reason
        if now>=next_sample:
            current_free=free_bytes()
            samples.append({'elapsed_seconds':round(now-started,6),'free_bytes':current_free})
            next_sample=now+2
            if current_free<9*GIB and stop_reason is None:
                stop_reason='disk_stop_threshold'

    def observe_leader():
        nonlocal leader_exit_observed,leader_observation_failed
        try:
            assert child is not None and owned_group and not child_reaped and not signal_phase_closed
            assert not leader_observation_failed and child.returncode is None
            assert signal.getsignal(signal.SIGCHLD)==signal.SIG_DFL
            observed=os.waitid(os.P_PID,child.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT)
            assert observed is None or (observed.si_pid==child.pid and observed.si_code in (os.CLD_EXITED,os.CLD_KILLED,os.CLD_DUMPED))
        except BaseException:
            leader_observation_failed=True
            raise
        if observed is not None:
            leader_exit_observed=True
        return observed is not None

    def signal_group(sig):
        nonlocal internal_failure
        label='TERM' if sig==signal.SIGTERM else 'KILL'
        event={'signal':label,'elapsed_seconds':round(time.monotonic()-started,6),'outcome':'ownership_refused','unreaped_leader_verified':False}
        signals.append(event)
        try:
            assert sig in (signal.SIGTERM,signal.SIGKILL)
            observe_leader()
        except BaseException:
            if internal_failure is None:
                internal_failure='supervisor_internal_failure'
            cleanup_errors.append('signal_ownership_unconfirmed')
            return False
        event['unreaped_leader_verified']=True
        event['outcome']='attempting'
        try:
            os.killpg(child.pid,sig)
            event['outcome']='sent'
        except ProcessLookupError:
            event['outcome']='group_absent'
        except BaseException:
            event['outcome']='unconfirmed'
            if internal_failure is None:
                internal_failure='supervisor_internal_failure'
            cleanup_errors.append('term_failed' if sig==signal.SIGTERM else 'kill_failed')
            return False
        return True

    def drain(timeout):
        nonlocal stream_eof,bytes_written,log_capped,stop_reason,downloaded,carry
        for key,_ in sel.select(timeout):
            block=os.read(key.fileobj.fileno(),65536)
            if not block:
                sel.unregister(key.fileobj)
                stream_eof=True
                continue
            remaining=LOG_CAP-bytes_written
            saved=block[:max(0,remaining)]
            log.write(saved)
            bytes_written+=len(saved)
            if len(saved)!=len(block):
                log_capped=True
                if stop_reason is None:
                    stop_reason='log_cap'
            scanned=(carry+block).lower()
            if any(label in scanned for label in [b'downloading ',b'downloaded ',b'updating crates.io index',b'updating git repository',b'fetching ']):
                downloaded=True
                if stop_reason is None:
                    stop_reason='unexpected_dependency_download'
            carry=scanned[-128:]

    with os.fdopen(log_fd,'wb') as log:
        try:
            assert signal.getsignal(signal.SIGCHLD)==signal.SIG_DFL
            child=subprocess.Popen(ARGV,cwd=ROOT,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,start_new_session=True)
            pgid=child.pid
            owned_group=os.getpgid(child.pid)==pgid
            assert owned_group
            exclusive_json('owned-child.redacted.json',{'pid':child.pid,'pgid':pgid,'owned_group_verified':True,'elapsed_seconds':time.monotonic()-started})
            os.set_blocking(child.stdout.fileno(),False)
            sel.register(child.stdout,selectors.EVENT_READ)
            while True:
                now=time.monotonic()
                sample_disk(now)
                if now-started>=1765 and stop_reason is None:
                    stop_reason='active_deadline'
                ended=observe_leader()
                if (stop_reason is not None or ended) and first_stop is None:
                    first_stop=now
                if first_stop is not None:
                    if term_at is None:
                        term_at=now
                        if not signal_group(signal.SIGTERM):
                            break
                    elif now-term_at>=5 and kill_at is None:
                        kill_at=now
                        if not signal_group(signal.SIGKILL):
                            break
                    if kill_at is not None:
                        break
                if first_stop is not None and now-first_stop>=20:
                    cleanup_errors.append('owned_cleanup_deadline')
                    break
                if now-started>=1790:
                    cleanup_errors.append('outer_cleanup_deadline')
                    break
                drain(0.2)
        except BaseException:
            if internal_failure is None:
                internal_failure='supervisor_internal_failure'
        finally:
            if child is not None:
                if first_stop is None:
                    first_stop=time.monotonic()
                if owned_group and not leader_observation_failed:
                    if term_at is None:
                        term_at=time.monotonic()
                        signal_group(signal.SIGTERM)
                    if kill_at is None and not leader_observation_failed:
                        grace_deadline=min(term_at+5,first_stop+20,started+1790)
                        try:
                            while time.monotonic()<grace_deadline:
                                sample_disk(time.monotonic())
                                drain(min(0.2,max(0.0,grace_deadline-time.monotonic())))
                        except BaseException:
                            if internal_failure is None:
                                internal_failure='supervisor_internal_failure'
                            cleanup_errors.append('cleanup_drain_failed')
                        if not leader_observation_failed:
                            kill_at=time.monotonic()
                            signal_group(signal.SIGKILL)
                else:
                    cleanup_errors.append('owned_leader_unconfirmed')
                # Close the signal phase before any consuming wait; never reopen it.
                signal_phase_closed=True
                reap_deadline=min(started+1798,time.monotonic()+5)
                try:
                    while True:
                        sample_disk(time.monotonic())
                        waited_pid,waited_status=os.waitpid(child.pid,os.WNOHANG)
                        if waited_pid==child.pid:
                            child_wait_status=waited_status
                            child_reaped=True
                            returncode=os.waitstatus_to_exitcode(waited_status)
                            child.returncode=returncode
                            break
                        assert waited_pid==0
                        remaining=reap_deadline-time.monotonic()
                        if remaining<=0:
                            cleanup_errors.append('final_reap_failed')
                            break
                        drain(min(0.2,remaining))
                except BaseException:
                    if internal_failure is None:
                        internal_failure='supervisor_internal_failure'
                    cleanup_errors.append('final_reap_failed')
                try:
                    while child_reaped and owned_group:
                        group_gone=group_absent(child.pid)
                        if group_gone and stream_eof:
                            break
                        remaining=reap_deadline-time.monotonic()
                        if remaining<=0:
                            break
                        sample_disk(time.monotonic())
                        drain(min(0.2,remaining))
                    if child_reaped and owned_group:
                        group_gone=group_absent(child.pid)
                    if not group_gone:
                        cleanup_errors.append('owned_group_remaining_or_unconfirmed')
                    if not stream_eof:
                        cleanup_errors.append('output_drain_incomplete')
                except BaseException:
                    if internal_failure is None:
                        internal_failure='supervisor_internal_failure'
                    cleanup_errors.append('final_absence_or_drain_failed')
                try:
                    child.stdout.close()
                except BaseException:
                    if internal_failure is None:
                        internal_failure='supervisor_internal_failure'
                    cleanup_errors.append('stdout_close_failed')
            else:
                signal_phase_closed=True
            sel.close()
            log.flush()
            os.fsync(log.fileno())
    samples.append({'elapsed_seconds':round(time.monotonic()-started,6),'free_bytes':free_bytes()})
    raw=log_path.read_bytes()
    retained={'source_archive':archive_id,'source_head':current_head,'command':DISPLAY_COMMAND,'launch_free_bytes':launch_free,'disk_samples':samples,'disk_minimum_bytes':min(s['free_bytes'] for s in samples),'elapsed_seconds':round(time.monotonic()-started,6),'child_pid':None if child is None else child.pid,'child_pgid':None if child is None else child.pid,'owned_group_verified':owned_group,'child_returncode':returncode,'child_wait_status':child_wait_status,'child_reaped':child_reaped,'owned_group_absent':group_gone,'leader_exit_observed':leader_exit_observed,'leader_observation_failed':leader_observation_failed,'signal_phase_closed':signal_phase_closed,'signals':signals,'stop_reason':stop_reason,'internal_failure':internal_failure,'cleanup_errors':cleanup_errors,'unexpected_dependency_download_observed':downloaded,'log_capped':log_capped,'stream_eof':stream_eof,'log':{'path':str(log_path.relative_to(ROOT)),'bytes':len(raw),'sha256':digest(raw),'mode':'0600'},'output_retained_before_grade':True}
    result_id=exclusive_json('actual-status.redacted.json',retained)
    # Grading starts only after the complete bounded output and numeric status are fsynced.
    counts_matches=re.findall(rb'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',raw)
    counts_match=counts_matches[0] if len(counts_matches)==1 else None
    counts=None if counts_match is None else {'passed':int(counts_match[1]),'failed':int(counts_match[2]),'ignored':int(counts_match[3]),'measured':int(counts_match[4]),'filtered_out':int(counts_match[5])}
    panic_matches=re.findall(rb'panicked at (tests/device_trust\.rs):(\d{1,6}):(\d{1,6}):',raw)
    panic_locations=[{'path':'tests/device_trust.rs','line':int(match[1]),'column':int(match[2])} for match in panic_matches[:14]]
    unexpected_ok_panic_count=raw.count(b'called `Result::unwrap_err()` on an `Ok` value')
    whole_target_complete=bool(counts and counts['passed']+counts['failed']==14 and counts['ignored']==0 and counts['measured']==0 and counts['filtered_out']==0 and child_reaped and group_gone and stream_eof and not log_capped and not downloaded and stop_reason is None and internal_failure is None and not cleanup_errors)
    whole_target_pass=bool(whole_target_complete and returncode==0 and counts_match[0]==b'ok' and counts['passed']==14 and counts['failed']==0 and not panic_matches and unexpected_ok_panic_count==0)
    classification='pass' if whole_target_pass else 'test_failure' if whole_target_complete and counts_match[0]==b'FAILED' and returncode==101 and counts['failed']>0 else 'incomplete'
    grade={'actual_status':result_id,'test_counts':counts,'summary_count':len(counts_matches),'expected_test_count':14,'panic_locations':panic_locations,'panic_location_count':len(panic_matches),'panic_locations_capped':len(panic_matches)>14,'unexpected_ok_panic_count':unexpected_ok_panic_count,'whole_target_complete':whole_target_complete,'whole_target_pass':whole_target_pass,'whole_target_classification':classification,'resource_release_proven':bool(child_reaped and group_gone),'child_returncode':returncode,'stop_reason':stop_reason,'internal_failure':internal_failure,'cleanup_errors':cleanup_errors,'elapsed_seconds':retained['elapsed_seconds'],'disk_minimum_bytes':retained['disk_minimum_bytes'],'log_bytes':len(raw),'log_sha256':digest(raw),'source_archive':archive_id}
    grade_id=exclusive_json('grade.redacted.json',grade)
    print(json.dumps({'grade':grade,'grade_file':grade_id},sort_keys=True),flush=True)
    return 0 if child_reaped and group_gone and internal_failure is None and not cleanup_errors else 125


if __name__ == '__main__':
    sys.exit(main())
```

### Exact narrow forward diff (encoded blank-context lines)

```diff
--- 64e4-whole-target-supervisor.py
+++ unreaped-leader-whole-target-supervisor.py
@@ -66,6 +66,9 @@
␠
 def main():
     signal.signal(signal.SIGCHLD, signal.SIG_DFL)
+    assert signal.getsignal(signal.SIGCHLD)==signal.SIG_DFL
+    assert all(hasattr(os,name) for name in ('waitid','P_PID','WEXITED','WNOHANG','WNOWAIT','CLD_EXITED','CLD_KILLED','CLD_DUMPED','waitpid','waitstatus_to_exitcode'))
+    assert callable(os.waitid) and callable(os.waitpid) and callable(os.waitstatus_to_exitcode)
     os.umask(0o077)
     assert RUN.stat().st_uid==os.getuid() and stat.S_IMODE(RUN.stat().st_mode)==0o700
     assert not RUN.is_symlink()
@@ -128,8 +131,92 @@
     first_stop=None
     returncode=None
     internal_failure=None
+    leader_exit_observed=False
+    leader_observation_failed=False
+    signal_phase_closed=False
+    child_wait_status=None
+    next_sample=started+2
+    term_at=None
+    kill_at=None
+
+    def sample_disk(now):
+        nonlocal next_sample,stop_reason
+        if now>=next_sample:
+            current_free=free_bytes()
+            samples.append({'elapsed_seconds':round(now-started,6),'free_bytes':current_free})
+            next_sample=now+2
+            if current_free<9*GIB and stop_reason is None:
+                stop_reason='disk_stop_threshold'
+
+    def observe_leader():
+        nonlocal leader_exit_observed,leader_observation_failed
+        try:
+            assert child is not None and owned_group and not child_reaped and not signal_phase_closed
+            assert not leader_observation_failed and child.returncode is None
+            assert signal.getsignal(signal.SIGCHLD)==signal.SIG_DFL
+            observed=os.waitid(os.P_PID,child.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT)
+            assert observed is None or (observed.si_pid==child.pid and observed.si_code in (os.CLD_EXITED,os.CLD_KILLED,os.CLD_DUMPED))
+        except BaseException:
+            leader_observation_failed=True
+            raise
+        if observed is not None:
+            leader_exit_observed=True
+        return observed is not None
+
+    def signal_group(sig):
+        nonlocal internal_failure
+        label='TERM' if sig==signal.SIGTERM else 'KILL'
+        event={'signal':label,'elapsed_seconds':round(time.monotonic()-started,6),'outcome':'ownership_refused','unreaped_leader_verified':False}
+        signals.append(event)
+        try:
+            assert sig in (signal.SIGTERM,signal.SIGKILL)
+            observe_leader()
+        except BaseException:
+            if internal_failure is None:
+                internal_failure='supervisor_internal_failure'
+            cleanup_errors.append('signal_ownership_unconfirmed')
+            return False
+        event['unreaped_leader_verified']=True
+        event['outcome']='attempting'
+        try:
+            os.killpg(child.pid,sig)
+            event['outcome']='sent'
+        except ProcessLookupError:
+            event['outcome']='group_absent'
+        except BaseException:
+            event['outcome']='unconfirmed'
+            if internal_failure is None:
+                internal_failure='supervisor_internal_failure'
+            cleanup_errors.append('term_failed' if sig==signal.SIGTERM else 'kill_failed')
+            return False
+        return True
+
+    def drain(timeout):
+        nonlocal stream_eof,bytes_written,log_capped,stop_reason,downloaded,carry
+        for key,_ in sel.select(timeout):
+            block=os.read(key.fileobj.fileno(),65536)
+            if not block:
+                sel.unregister(key.fileobj)
+                stream_eof=True
+                continue
+            remaining=LOG_CAP-bytes_written
+            saved=block[:max(0,remaining)]
+            log.write(saved)
+            bytes_written+=len(saved)
+            if len(saved)!=len(block):
+                log_capped=True
+                if stop_reason is None:
+                    stop_reason='log_cap'
+            scanned=(carry+block).lower()
+            if any(label in scanned for label in [b'downloading ',b'downloaded ',b'updating crates.io index',b'updating git repository',b'fetching ']):
+                downloaded=True
+                if stop_reason is None:
+                    stop_reason='unexpected_dependency_download'
+            carry=scanned[-128:]
+
     with os.fdopen(log_fd,'wb') as log:
         try:
+            assert signal.getsignal(signal.SIGCHLD)==signal.SIG_DFL
             child=subprocess.Popen(ARGV,cwd=ROOT,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,start_new_session=True)
             pgid=child.pid
             owned_group=os.getpgid(child.pid)==pgid
@@ -137,106 +224,115 @@
             exclusive_json('owned-child.redacted.json',{'pid':child.pid,'pgid':pgid,'owned_group_verified':True,'elapsed_seconds':time.monotonic()-started})
             os.set_blocking(child.stdout.fileno(),False)
             sel.register(child.stdout,selectors.EVENT_READ)
-            next_sample=started+2
-            term_at=None
-            kill_at=None
             while True:
                 now=time.monotonic()
-                if now>=next_sample:
-                    current_free=free_bytes()
-                    samples.append({'elapsed_seconds':round(now-started,6),'free_bytes':current_free})
-                    next_sample=now+2
-                    if current_free<9*GIB and stop_reason is None:
-                        stop_reason='disk_stop_threshold'
+                sample_disk(now)
                 if now-started>=1765 and stop_reason is None:
                     stop_reason='active_deadline'
-                observed=child.poll()
-                if observed is not None:
-                    returncode=observed
-                    child_reaped=True
-                if stop_reason is not None and first_stop is None:
+                ended=observe_leader()
+                if (stop_reason is not None or ended) and first_stop is None:
                     first_stop=now
-                if child_reaped and not group_absent(pgid) and stop_reason is None:
-                    stop_reason='owned_group_remained_after_child_exit'
-                    first_stop=now
-                if stop_reason is not None and not group_absent(pgid):
+                if first_stop is not None:
                     if term_at is None:
-                        os.killpg(pgid,signal.SIGTERM)
                         term_at=now
-                        signals.append({'signal':'TERM','elapsed_seconds':round(now-started,6)})
+                        if not signal_group(signal.SIGTERM):
+                            break
                     elif now-term_at>=5 and kill_at is None:
-                        os.killpg(pgid,signal.SIGKILL)
                         kill_at=now
-                        signals.append({'signal':'KILL','elapsed_seconds':round(now-started,6)})
-                if child_reaped and group_absent(pgid) and stream_eof:
-                    group_gone=True
-                    break
+                        if not signal_group(signal.SIGKILL):
+                            break
+                    if kill_at is not None:
+                        break
                 if first_stop is not None and now-first_stop>=20:
                     cleanup_errors.append('owned_cleanup_deadline')
                     break
                 if now-started>=1790:
                     cleanup_errors.append('outer_cleanup_deadline')
                     break
-                for key,_ in sel.select(0.2):
-                    block=os.read(key.fileobj.fileno(),65536)
-                    if not block:
-                        sel.unregister(key.fileobj)
-                        stream_eof=True
-                        continue
-                    remaining=LOG_CAP-bytes_written
-                    saved=block[:max(0,remaining)]
-                    log.write(saved)
-                    bytes_written+=len(saved)
-                    if len(saved)!=len(block):
-                        log_capped=True
-                        if stop_reason is None:
-                            stop_reason='log_cap'
-                    scanned=(carry+block).lower()
-                    if any(label in scanned for label in [b'downloading ',b'downloaded ',b'updating crates.io index',b'updating git repository',b'fetching ']):
-                        downloaded=True
-                        if stop_reason is None:
-                            stop_reason='unexpected_dependency_download'
-                    carry=scanned[-128:]
+                drain(0.2)
         except BaseException:
-            internal_failure='supervisor_internal_failure'
+            if internal_failure is None:
+                internal_failure='supervisor_internal_failure'
         finally:
-            if child is not None and owned_group:
-                if not group_absent(child.pid):
-                    try:
-                        os.killpg(child.pid,signal.SIGTERM)
-                        signals.append({'signal':'TERM','elapsed_seconds':round(time.monotonic()-started,6)})
-                    except ProcessLookupError:
-                        pass
-                    except BaseException:
-                        cleanup_errors.append('term_failed')
-                    try:
-                        child.wait(timeout=min(5,max(0.01,1798-(time.monotonic()-started))))
-                        child_reaped=True
-                    except subprocess.TimeoutExpired:
-                        pass
-                    except BaseException:
-                        cleanup_errors.append('wait_failed')
-                    if not group_absent(child.pid):
+            if child is not None:
+                if first_stop is None:
+                    first_stop=time.monotonic()
+                if owned_group and not leader_observation_failed:
+                    if term_at is None:
+                        term_at=time.monotonic()
+                        signal_group(signal.SIGTERM)
+                    if kill_at is None and not leader_observation_failed:
+                        grace_deadline=min(term_at+5,first_stop+20,started+1790)
                         try:
-                            os.killpg(child.pid,signal.SIGKILL)
-                            signals.append({'signal':'KILL','elapsed_seconds':round(time.monotonic()-started,6)})
-                        except ProcessLookupError:
-                            pass
+                            while time.monotonic()<grace_deadline:
+                                sample_disk(time.monotonic())
+                                drain(min(0.2,max(0.0,grace_deadline-time.monotonic())))
                         except BaseException:
-                            cleanup_errors.append('kill_failed')
+                            if internal_failure is None:
+                                internal_failure='supervisor_internal_failure'
+                            cleanup_errors.append('cleanup_drain_failed')
+                        if not leader_observation_failed:
+                            kill_at=time.monotonic()
+                            signal_group(signal.SIGKILL)
+                else:
+                    cleanup_errors.append('owned_leader_unconfirmed')
+                # Close the signal phase before any consuming wait; never reopen it.
+                signal_phase_closed=True
+                reap_deadline=min(started+1798,time.monotonic()+5)
                 try:
-                    returncode=child.wait(timeout=min(5,max(0.01,1798-(time.monotonic()-started))))
-                    child_reaped=True
+                    while True:
+                        sample_disk(time.monotonic())
+                        waited_pid,waited_status=os.waitpid(child.pid,os.WNOHANG)
+                        if waited_pid==child.pid:
+                            child_wait_status=waited_status
+                            child_reaped=True
+                            returncode=os.waitstatus_to_exitcode(waited_status)
+                            child.returncode=returncode
+                            break
+                        assert waited_pid==0
+                        remaining=reap_deadline-time.monotonic()
+                        if remaining<=0:
+                            cleanup_errors.append('final_reap_failed')
+                            break
+                        drain(min(0.2,remaining))
                 except BaseException:
+                    if internal_failure is None:
+                        internal_failure='supervisor_internal_failure'
                     cleanup_errors.append('final_reap_failed')
-                group_gone=group_absent(child.pid)
-                child.stdout.close()
+                try:
+                    while child_reaped and owned_group:
+                        group_gone=group_absent(child.pid)
+                        if group_gone and stream_eof:
+                            break
+                        remaining=reap_deadline-time.monotonic()
+                        if remaining<=0:
+                            break
+                        sample_disk(time.monotonic())
+                        drain(min(0.2,remaining))
+                    if child_reaped and owned_group:
+                        group_gone=group_absent(child.pid)
+                    if not group_gone:
+                        cleanup_errors.append('owned_group_remaining_or_unconfirmed')
+                    if not stream_eof:
+                        cleanup_errors.append('output_drain_incomplete')
+                except BaseException:
+                    if internal_failure is None:
+                        internal_failure='supervisor_internal_failure'
+                    cleanup_errors.append('final_absence_or_drain_failed')
+                try:
+                    child.stdout.close()
+                except BaseException:
+                    if internal_failure is None:
+                        internal_failure='supervisor_internal_failure'
+                    cleanup_errors.append('stdout_close_failed')
+            else:
+                signal_phase_closed=True
             sel.close()
             log.flush()
             os.fsync(log.fileno())
     samples.append({'elapsed_seconds':round(time.monotonic()-started,6),'free_bytes':free_bytes()})
     raw=log_path.read_bytes()
-    retained={'source_archive':archive_id,'source_head':current_head,'command':DISPLAY_COMMAND,'launch_free_bytes':launch_free,'disk_samples':samples,'disk_minimum_bytes':min(s['free_bytes'] for s in samples),'elapsed_seconds':round(time.monotonic()-started,6),'child_pid':None if child is None else child.pid,'child_pgid':None if child is None else child.pid,'owned_group_verified':owned_group,'child_returncode':returncode,'child_reaped':child_reaped,'owned_group_absent':group_gone,'signals':signals,'stop_reason':stop_reason,'internal_failure':internal_failure,'cleanup_errors':cleanup_errors,'unexpected_dependency_download_observed':downloaded,'log_capped':log_capped,'stream_eof':stream_eof,'log':{'path':str(log_path.relative_to(ROOT)),'bytes':len(raw),'sha256':digest(raw),'mode':'0600'},'output_retained_before_grade':True}
+    retained={'source_archive':archive_id,'source_head':current_head,'command':DISPLAY_COMMAND,'launch_free_bytes':launch_free,'disk_samples':samples,'disk_minimum_bytes':min(s['free_bytes'] for s in samples),'elapsed_seconds':round(time.monotonic()-started,6),'child_pid':None if child is None else child.pid,'child_pgid':None if child is None else child.pid,'owned_group_verified':owned_group,'child_returncode':returncode,'child_wait_status':child_wait_status,'child_reaped':child_reaped,'owned_group_absent':group_gone,'leader_exit_observed':leader_exit_observed,'leader_observation_failed':leader_observation_failed,'signal_phase_closed':signal_phase_closed,'signals':signals,'stop_reason':stop_reason,'internal_failure':internal_failure,'cleanup_errors':cleanup_errors,'unexpected_dependency_download_observed':downloaded,'log_capped':log_capped,'stream_eof':stream_eof,'log':{'path':str(log_path.relative_to(ROOT)),'bytes':len(raw),'sha256':digest(raw),'mode':'0600'},'output_retained_before_grade':True}
     result_id=exclusive_json('actual-status.redacted.json',retained)
     # Grading starts only after the complete bounded output and numeric status are fsynced.
     counts_matches=re.findall(rb'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',raw)
```

Decode ONLY the1 complete standalone `␠` lines to exactly ONE ASCII space plus LF before applying the archived diff. Decoding reconstructs every canonical diff byte/hash in the manifest. Apply forward once to the complete hash-checked64e4 controller; reverse once to reconstruct all15555 old bytes, its full location-independent AST, and the original dated SHA256. No old controller file, private output leaf, source/test/guide/config/other report or historical capture was materialized or edited. The future leaf role/path remains `deployment-private/i07-device-trust-whole-5c20ca-20261003-once`; it is still absent and requires a distinct root runtime release and exclusive private-file authorization before use.

Static checks during construction: complete six-substitution byte/AST reversal; all source outside main and all five original helper spans exact; parameterized complete old log-reader AST exact; entire whole14 grading suffix exact; one WNOWAIT observer site, one direct consuming waitpid site and only two killpg sites (signal0 helper and guarded nonzero sink); no forbidden Popen consuming/signaling calls; old child_reaped-based remaining-live-group error removed. Candidate code was AST-parsed only: no imports, function/body/case/controller evaluation, wait/signal/probe/native/Cargo/test/provider/browser/Driver/HTTP/library or output-leaf operation occurred. These are source proofs, not executed lifecycle/ownership/timeout tests.

Root must read the full immutable candidate/diff and separately decide any exact future validation/release. Runtime remains HELD; no slot was acquired or released, and original I07/D01/D05 acceptance/status remains root-owned. All unknown historical sender/cause/lost-value and failed-run limits remain dated and unchanged.

Final static checks returned actual exit0 individually: report whitespace (`git diff --check`), Markdown/layout (`python3 scripts/check-docs.py`), complete archived forward/inverse AST/byte and ownership-order/scope checker, and staged tracked-file hygiene (`python3 scripts/check-repo-hygiene.py`,1143 files). The canonical diff has three hunks; the single standalone blank-context marker decodes exactly. Static ownership-order proof checked WNOWAIT options/strict observer assertions and poison latch, pre-spawn default-handler recheck, guarded nonzero sink, all signal calls before the irrevocable phase closure, sole matching actual waitpid/status-to-exit conversion and no forbidden Popen status/signaling calls. All five original top-level helper bodies and the complete whole-target grader are byte-exact.

Read-only hash/mode/owner checks verified all eight original I07 negative-run private captures against their retained SHA256 witnesses: regular, own UID,0600 and byte-hash exact, including the original supervisor, full private raw panic output and original parser-miss grade. Their contents were not emitted or rewritten. This correction performs no live capacity/process/ownership/waitability/protocol/tool probe; the prior capacity/cache observations remain dated in the preserved64e4 prefix. Current WNOWAIT behavior, bounded cleanup and real exit remain UNEXECUTED prerequisites. No claim is made that static proof establishes an actual cleanup, whole target pass, device/provider provenance, browser journey or original-row closure.
