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
