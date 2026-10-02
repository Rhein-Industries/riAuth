# D01 printed-step audit and bounded walkthrough proposal

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Task: `a96a1977-3210-4284-8f7d-645793369301` (D01).
Worktree: `42bb51c6-c198-4adb-bd92-0a5222853231`.
Audited published source: `60437b59933cadd40a1f5fbbb91ba153aee56456`.
Own starting HEAD: `6e1e1a9a9f206bc98bb03da38bbee3f27ee32425`.
Date: 2026-10-02.

**Recommend retaining D01 in_progress.** The separate guides and the small
edition-specific setup exist. Three concrete printed-step corrections are
needed, and the printed Essentials chain still lacks walkthrough evidence.
This report supplies exact proposed hunks and one operator checkpoint for root
to reserve. It supplies no new runtime or user acceptance result.

Only this report is edited. The guide hunks below are proposals, not applied
changes. Fixed-main objects were read directly; guide and historical-walkthrough
blobs are identical to this branch, so no merge or file replacement was needed.
A03 and the six independently reviewed rows remain done. Root owns integration,
runtime reservation and task status.

## Original live acceptance

The live RiWork row was reread with `task list --project
891e7443-8dac-4c1b-897f-9e53cb59c7ee --json`. It is assigned to this worktree
and is `in_progress`. Its title is “Write separate Essentials and Platform guides.”

- Requested outcome: “Keep the small-install experience free of unnecessary advanced setup.”
- Workstream goal: “Document completed user and operator tasks, not merely available settings.”
- Completion gate: “A new user and a new operator can independently complete the documented workflows.”
- Evidence clause: review relevant implementation, tests, documentation and released artifacts
  as applicable; report actual verification, gaps and external prerequisites; do
  not complete implementation from documentation or a worker report alone.

The original prerequisite list is A02, A09, U10 and M02. The current assignment
supersedes the old scheduling hold. The user expressly canceled U10/accessibility
as a gate; physical-device matrices and accessibility scans are not added here.
The outcome does not require every Platform integration, every verifier graph,
official release certification or customer rollout.

The allowed guide paths are `docs/essentials-guide.md` and
`docs/platform-guide.md`; this report is the only file written in this phase.
No source/test ownership claim is proposed.

## Concrete printed-step findings at the fixed pin

Line numbers refer to the unmodified published guides.

| Finding | Printed location and consequence | Source-backed narrow disposition |
| --- | --- | --- |
| Direct creation retry advice | Essentials 265–267; Platform 406–408 say repeating client creation needs the same key and a new secret-file path. That suggests recovering a generated secret from an ordinary receipt. | Replace the advice with first-response disclosure and exact-retry `409 credential_already_issued`; inspect and rotate after lost delivery. `management.rs::ClientIssuanceReceipt::{check,save}` stores only a marker, and `create_client_issuing` / `rotate_client_secret_issuing` use it. `riauthctl/admin.rs::protect_secret` writes only the private destination. Preserve reviewed creation's separate recovery receipt, explicitly documented in the current client README and retained by `management/client_creation.rs::execute_client_creation_change`. |
| Revoked CLI session before backup | Essentials 314–316 / Platform 480–482 explicitly revoke all sessions on adding/removing a passkey. Essentials 359–360 / Platform 563–564 then use the server-CLI session saved in section 2. Following that chain as the administrator can produce an unauthorized backup request. | Print a fresh server-CLI login before backup. `assembly/passkey.rs::VerifiedRegistration::apply` increments the account epoch and `passkey_remove_in` does likewise. `identity::validate_user` rejects old epochs. `Core::password_login` plus `authenticator::consume_password_factor` permits the initialized password account to sign in again; the passkey task does not remove its password or enroll TOTP. Do not copy a browser cookie, weaken MFA or recover/reset credentials. |
| Stopped server before later remote tasks | Essentials 379 / Platform 584 stop serve before restore. Section 6 then says serve is “still running” without a restart (Essentials 444 / Platform 701). | Print restart/readiness of the **original** lab configuration, then renew the standalone-client session if missing/expired/revoked. Do not start the pending restored configuration to continue group tasks. `operations.rs::restore_into` and `recovery` retain the separate serving-closed recovery policy. |
| Small-install task selection | The introductions enumerate LDAP, SCIM and Platform configuration alongside the first tasks, without a short route through the basic install. | Add a short starting route and identify optional external-system procedures. Both section-1 install commands already select the right Cargo feature; `Config::default` leaves advanced tables empty and `reviewed_client_creation = false`. No mandatory workflow design, database cluster, mail or USB setup is added. |
| Existing-store safety | Historical binary observations predate current startup agreement enforcement. The guides discuss edition provenance but lack a nearby current-format upgrade distinction. | State that the tutorial creates a fresh store. Point existing-store readers to the explicit offline upgrade procedure, with both confirmation flags, deliberate missing-row adoption only, no conflicting format-3 rewrite and backup-based rollback. `Core::initialize_with_administrator` stamps the agreement; `Core::open_store` enforces it before startup writes; `node_security::record_security_agreement` preserves incompatible rows. This is an upgrade branch, not another fresh-install step. |
| Canceled manual gates | Both guides label unexecuted physical/synced/mobile/screen-reader coverage “manual gates” (Essentials 905–923; Platform 2299–2323). | Retain the historical coverage limits, remove their status as required walkthrough steps. No such testing is scheduled. |

Online backup is **not** a finding: `cli.rs::Command::Backup`,
`cli/backup.rs::receive` and `api/backup.rs::stream` still implement an
authenticated export from the running service. Archive `riauth.backup/v3`
and startup security-agreement format 3 are separate formats. Backup stays
online; restore, agreement recording and store recovery stay offline.

The Platform section-11 fresh configured password example still has a supported
path: `approval::configured_definition_in` selects a live approval when one
exists, otherwise an active configured definition; `executor/version::review_pin`
enforces retained approval floors and exact configured bytes. No extra
three-administrator approval prerequisite is invented for the fresh example.
The historical run started the workflow but did not submit its password step.
No workflow production change is proposed.

The existing application prerequisite also stays honest: section 3 needs an
OIDC application serving the exact callback at `http://localhost:3000/callback`
to finish an application login. Client creation and policy explanation alone
are not that login. LDAP/SCIM/mail and SAML/LDAPS procedures similarly need their
explicitly named peers/materials when chosen. They are optional tasks, not
prerequisites of local Essentials sign-in.

## Historical executed evidence, without relabeling it

The kept [D01 walkthrough](d01-platform-cli-walkthrough.md) contains commands,
exit results, returned public fields, refusals and cleanup observations. The
table below reads those results, rather than inferring execution from paths.
All Platform snapshot rows used server source supplied as
`58357fde77211e62dc51c14fb3fc216bdf143ceb`, SHA-256
`de06f9b46ce3e4a929d4d065681325d664b9aedb6485f649ec098a57c22a6069`.
They are historical source-built debug snapshots, not official released
artifacts and not binaries for `60437b5`. No old binary or private credential
was opened or executed in this audit; recorded binary hashes were not freshly
recomputed.

| Kept record / documentation tree pin | Actual execution credited | Important unexecuted part |
| --- | --- | --- |
| Essentials catalog, `12165852700b23f54943d094f61c180819523a52` | Catalog Essentials; legacy server init/serve/readiness/login/doctor; unbound server-client write refused, revision-bound write succeeded with a private output file. Binary SHA `264ed196ec6366cd96aac6c2c058fa605af15541efd7160496e05f0a6d115e2f`; its compile-source pin is **not established** by the documentation-tree pin. | Printed Essentials guide, install, maintenance, riauthctl, groups/claims/audit and backup not executed. |
| Platform catalog, `58357fde77211e62dc51c14fb3fc216bdf143ceb` | Legacy init/readiness/login/doctor, confidential client create, group create/add-member, claims/explain and audit; revision advanced to 4. | Printed riauthctl chain, group get/has-member, browser and backup skipped. |
| Remote administration, `9c374beae00e99fbc7f922ef44c23243d6fbc28a` | Printed maintenance init; server startup/login/doctor; riauthctl login/client create/discovery/whoami/group get/has-member/update; explain and audit CSV. Secret absent from stdout, destination mode 0600. | No install, browser or recovery. `riauthctl doctor` actually refused locally, exit 2. |
| Backup entry points, `f0d714c96652dcb2c6a9898b2e0c32cfea729b73` | Legacy server keygen/online v3 backup, stop, restore into new redb, recovery status: verified archive and serving closed. | Printed maintenance executable names, recovery complete/recover-admin and second server unrun. |
| Passkey prompt, `080452a45c97535fa858ddfd2d72148fbe4760ff` | Browser password sign-in, security page, started chooser and canceled; server passkey list remained empty. | No stored credential, rename/remove/passkey sign-in, hardware or Essentials guide execution. |
| Invitation, `0791deb4737326c1bc2157a48a636decc1117a69` | Loopback SMTP invitation issued with bindings; password accepted; browser remained signed out; replay refused; invited person then signed in. | Passkey invitation and Essentials walkthrough unrun; no external mailbox. |
| Configured password workflow, `0add90f9febc5b00bbc37aed7099023ac1743737` | Schema/validate/plan/apply, restart with local-password config, configured start active on password step; export persisted definition. | Password continuation, TOTP and browser workflow execution unrun. |
| LDAP import, `11f1f8eaeaf23008b94767bcdd310bd46501c182` | StartTLS disposable OpenLDAP; directory list/plan/apply imported alice and staff membership; revision 1 before plan, 4 after apply. | No printed example host, directory-backed password login or Essentials guide execution. |
| Outbound SCIM, `a5769bb4e94e6adc8847c6d30d92782f3a3133b2` | Private loopback target; plan/apply and three job reads ended succeeded, processed 2/2; Users and Groups create requests observed. | No named SaaS target, later PATCH/deactivation/removals or Essentials guide execution; no exactly-once claim. |
| LDAP provider, `1377732a9be531729789852cd6efca3d50bf4256` | Scoped agent/public client, LDAPS listener and ldapsearch service/user bind paths; wrong credentials/nonmember/untrusted CA refused. | No customer directory, STARTTLS provider or Essentials runtime. |
| SAML preparation, `298cbec18eeccc3235c64368df50e505f5946418` | Key imports, metadata conversion, validate/plan/apply, source put/list/metadata/start; initial metadata destination refused, new destination worked; xmlsec signatures and local Lasso response acceptance. | Source finish/ACS/browser source login and named IdP/SP exchange unrun. The published command now uses the corrected new metadata filename. |

The remote-administration record used tool source
`f430c2f01b63cbd53ffd0f56ebd277e03724d822`, not the server pin:
riauthctl SHA `edccd38a893972b1d369a3743a0e069c698fab99ee3508f066aa639b6f6602a3`;
maintenance SHA `7b2293f9548f74d8e15a14e851bc0594c67c912fe4fbd2bf6e63e485e4cd71ab`.
It explicitly did not perform one same-revision installation of all three.

Supplementary accepted [Q08 native macOS evidence](q08-exact-edition-bundles.md)
and its matrix JSON at `4ca7558ee8563a0c5eedc1f13d4a9a6faa6a8d4a` record both
explicit-edition builds and fresh redb/PostgreSQL readiness/admin login.
[Native Linux ARM64 evidence](q08-native-arm64-local-6ca4779.md) at
`6ca4779b68cf41e29af490d2aca6ea3d59e1f2bd` records same-edition v3 backup,
maintenance restore, serving-closed recovery, explicit completion, old-session
refusal and new login for both editions. Its separate archive/image/installed
cross-edition checks failed; they are not silently turned into passes. These
local source-built results support primitives, not execution of the printed
Essentials tasks at the current pin or proof of an official release.
No `cargo install` walkthrough is established by those build records.

## Exact proposed guide hunks — not applied

These are generated against the exact published blobs:

- Essentials: `2b8408f90d82bb35d49fc1ebd3512d9a1a853962`.
- Platform: `46f41f11fe211343be9059966253118401ffb4a0`.
- Historical walkthrough: `ef367ba40fdec9b26e6bef462d9a07de830e0d3b`.

The first client-create command remains simple: riauthctl fills its required
revision/key envelope. Group writes retain their printed exact revision/key
pairs. Plan/removal confirmation semantics are unchanged. The reviewed-creation
secret receipt, per-route optional/required headers and PAM revision fallback
remain accepted contracts. No credential, Group or remote-IO behavior is changed.

````diff
--- a/docs/essentials-guide.md
+++ b/docs/essentials-guide.md
@@ -3,4 +3,10 @@
 Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task D01
 `a96a1977-3210-4284-8f7d-645793369301`.
+
+For a small install, start with sections 1 through 3. Section 4 is the
+user’s passkey task; section 5 is the operator’s backup task. Groups, claims
+and audit are sections 6 through 8. Add sections 9 through 11 only when you
+have an LDAP directory, a SCIM target or invitation mail to configure. None
+of those external systems is required for the first local sign-in.

 This is the Essentials task guide through its second slice, plus the
@@ -54,4 +60,16 @@
 instance stays Essentials. `keygen` does not open a store.

+This first-install path uses a new store and matching current binaries.
+Existing stores from older binaries need the separate [offline upgrade
+procedure](operations.md#rate-limits-and-admission). Startup refuses old or
+missing security agreements and never adopts them automatically. Stop every
+riAuth process and verify a pre-upgrade backup before running
+`security-agreement-record --confirm-authentication-policy --confirm-rate-limits`
+with the matching-edition maintenance binary. Add `--adopt-missing-agreement`
+only for a deliberately reviewed missing-row adoption; it cannot bypass a
+present incompatible row. A conflicting format 3 policy cannot be overwritten
+by this command. Older binaries refuse format 3; rollback requires the
+compatible pre-command backup. This is not a step for a fresh small install.
+
 The [product contracts](roadmap/product-contracts.md) describe the desired
 Essentials and Platform split. They are a target contract. They are not
@@ -82,5 +100,5 @@
 cookie. It does not write either CLI session file.

-Sections 6 and 7 sign the riauthctl session in again if it has expired, then
+Sections 6 and 7 sign the riauthctl session in again if it is expired or revoked, then
 change groups and the `local-demo` client. `explain`, `audit`, `report`,
 `directory`, and `provision` use the server CLI session from section 2.
@@ -263,7 +281,11 @@
 that already exists. The client reserves that file before the request and
 writes the one-time secret there. Standard output names `credential_file` and
-omits the secret. Repeating the command needs the same `--idempotency-key`
-and a new secret-file path; the details are in the
-[riauthctl README](../crates/riauthctl/README.md).
+omits the secret. Direct creation stores no secret in its retry receipt:
+an exact same-key, same-revision retry returns `409 credential_already_issued`.
+If creation committed but credential delivery failed, inspect `local-demo`
+and rotate its secret with a new key, the current revision and a new
+`--secret-file`; repeating creation does not recover it. Reviewed creation
+has a separate secret-recovery receipt contract. Both paths are described in
+the [riauthctl README](../crates/riauthctl/README.md).

 If `--scope` is omitted, a non-service client asks for
@@ -330,5 +352,5 @@
 The dialog text says to pick this device, another device, or a security key
 in the browser prompt. Synced passkeys, phones, and physical keys are part of
-the manual accessibility and authenticator gates below. This slice does not
+the recorded device and browser limits below. This slice does not
 record a result for them.

@@ -365,4 +387,11 @@
 backup key is still required. Keep `backup.key` outside the host you are
 willing to lose.
+
+Section 4’s passkey changes revoke this account’s CLI sessions too. Before
+backup, sign the server CLI in again as the password administrator from `init`:
+
+```sh
+riauth --server http://localhost:9000 login admin
+```

 ```sh
@@ -442,6 +471,21 @@
 ## 6. Create a group and add the administrator

-The operator does this with `riauthctl` while `riauth serve` is still running.
-Sign in again when the riauthctl session is missing:
+If you followed section 5, `serve` is stopped. Restart the **original** lab
+configuration and leave it running:
+
+```sh
+riauth --config deployment-private/essentials-lab/riauth.toml serve
+```
+
+In another terminal, check readiness:
+
+```sh
+curl --fail http://127.0.0.1:9000/readyz
+```
+
+Do not start the restored configuration merely to continue this guide; it is
+a separate recovery exercise and stays closed while reconciliation is pending.
+The operator now uses `riauthctl` against the original running server. Sign in
+again when that session is missing, expired or revoked by a passkey change:

 ```sh
@@ -903,5 +947,5 @@
 | `riauth capabilities` | Artifact catalog for the binary on `PATH`. | `usable` is null until a configured instance reports runtime state. The catalog is not a peer or authenticator test. |

-## Manual accessibility gates
+## Recorded device and browser limits

 [accessibility-journeys.spec.js](../tools/browser/accessibility-journeys.spec.js)
@@ -916,5 +960,6 @@
 - a spoken screen reader (VoiceOver, TalkBack, or NVDA)

-Those five are manual gates. This task did not run the Playwright spec, a
+These are limits of the recorded runs, not required steps for this local
+walkthrough. This task did not run the Playwright spec, a
 desktop browser, or a screen reader. [Passkeys](passkeys.md) also says
 physical hardware and platform compatibility still need testing on the
````

````diff
--- a/docs/platform-guide.md
+++ b/docs/platform-guide.md
@@ -3,4 +3,10 @@
 Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task D01
 `a96a1977-3210-4284-8f7d-645793369301`.
+
+Start with the shared local tasks in sections 1 through 8. Add directory
+import or outbound SCIM in sections 9 and 10 only when needed. Sections 11
+through 13 are optional configured-workflow, SAML and LDAP-listener tasks;
+section 14 needs invitation mail. None is required to start the small
+Platform instance or sign in to it.

 This is the Platform task guide through its third slice, plus the
@@ -131,4 +137,16 @@
 That handoff is not a step in this slice.

+This first-install path uses a new store and matching current binaries.
+Existing stores from older binaries need the separate [offline upgrade
+procedure](operations.md#rate-limits-and-admission). Startup refuses old or
+missing security agreements and never adopts them automatically. Stop every
+riAuth process and verify a pre-upgrade backup before running
+`security-agreement-record --confirm-authentication-policy --confirm-rate-limits`
+with the matching-edition maintenance binary. Add `--adopt-missing-agreement`
+only for a deliberately reviewed missing-row adoption; it cannot bypass a
+present incompatible row. A conflicting format 3 policy cannot be overwritten
+by this command. Older binaries refuse format 3; rollback requires the
+compatible pre-command backup. This is not a step for a fresh small install.
+
 The [product contracts](roadmap/product-contracts.md) describe the desired
 split. They are a target contract. They are not evidence that every Platform
@@ -161,5 +179,5 @@
 cookie. It does not write either CLI session file.

-Sections 6 and 7 sign the riauthctl session in again if it has expired, then
+Sections 6 and 7 sign the riauthctl session in again if it is expired or revoked, then
 change groups and the `local-demo` client. `explain`, `audit`, `report`,
 `directory`, and `provision` use the server CLI session from section 2.
@@ -404,7 +422,11 @@
 that already exists. The client reserves that file before the request and
 writes the one-time secret there. Standard output names `credential_file` and
-omits the secret. Repeating the command needs the same `--idempotency-key`
-and a new secret-file path; the details are in the
-[riauthctl README](../crates/riauthctl/README.md).
+omits the secret. Direct creation stores no secret in its retry receipt:
+an exact same-key, same-revision retry returns `409 credential_already_issued`.
+If creation committed but credential delivery failed, inspect `local-demo`
+and rotate its secret with a new key, the current revision and a new
+`--secret-file`; repeating creation does not recover it. Reviewed creation
+has a separate secret-recovery receipt contract. Both paths are described in
+the [riauthctl README](../crates/riauthctl/README.md).

 `riauth client create` is the server CLI, a separate command. Without both
@@ -495,5 +517,5 @@
 The dialog text says to pick this device, another device, or a security key
 in the browser prompt. Synced passkeys, phones, and physical keys are part of
-the manual accessibility and authenticator gates below. This slice does not
+the recorded device and browser limits below. This slice does not
 record a result for them.

@@ -571,4 +593,11 @@
 willing to lose.

+Section 4’s passkey changes revoke this account’s CLI sessions too. Before
+backup, sign the server CLI in again as the password administrator from `init`:
+
+```sh
+riauth --server http://localhost:9000 login admin
+```
+
 ```sh
 riauth-maintenance keygen --out deployment-private/platform-lab/backup.key
@@ -699,6 +728,21 @@
 ## 6. Create a group and add the administrator

-The operator does this with `riauthctl` while `riauth serve` is still running.
-Sign in again when the riauthctl session is missing:
+If you followed section 5, `serve` is stopped. Restart the **original** lab
+configuration and leave it running:
+
+```sh
+riauth --config deployment-private/platform-lab/riauth.toml serve
+```
+
+In another terminal, check readiness:
+
+```sh
+curl --fail http://127.0.0.1:9000/readyz
+```
+
+Do not start the restored configuration merely to continue this guide; it is
+a separate recovery exercise and stays closed while reconciliation is pending.
+The operator now uses `riauthctl` against the original running server. Sign in
+again when that session is missing, expired or revoked by a passkey change:

 ```sh
@@ -2297,5 +2341,5 @@
 | `riauth capabilities` | Artifact catalog for the binary on `PATH`. | The first loopback record's `edition` was `essentials`, with `usable` null on every entry. The Platform-catalog record's `edition` was `platform`, `build_features` was `["essentials", "platform"]`, and `usable` was null on every entry. A configured instance's runtime report is a different document. Compiled Platform features are not configured features. |

-## Manual accessibility gates
+## Recorded device and browser limits

 [accessibility-journeys.spec.js](../tools/browser/accessibility-journeys.spec.js)
@@ -2310,5 +2354,6 @@
 - a spoken screen reader (VoiceOver, TalkBack, or NVDA)

-Those five are manual gates. This task did not run the Playwright spec or a
+These are limits of the recorded runs, not required steps for this local
+walkthrough. This task did not run the Playwright spec or a
 spoken screen reader. Section 4 records one isolated desktop browser on
 loopback. That browser signed in at `/apps` and cancelled the passkey prompt
@@ -2316,5 +2361,5 @@
 browser that accepted an invitation password and did not start a passkey
 ceremony. Physical keys, synced passkeys, phones, and
-spoken screen readers remain manual gates. [Passkeys](passkeys.md) also says
+spoken screen readers were not exercised. [Passkeys](passkeys.md) also says
 physical hardware and platform compatibility still need testing on the
 intended devices. The cancelled prompt in section 4 is one loopback
````

## One bounded executable plan for root reservation

**Not authorized or executed in this audit.** One 20-minute Essentials CLI
operator checkpoint, performed as individual commands without a new harness.
Root reserves the runtime after the O06/CI queue and supplies an existing managed
shell/runtime slot, one loopback listener at `127.0.0.1:9000`, and prebuilt
Essentials server/maintenance plus base riauthctl with exact source/provenance
and SHA-256 hashes. Use `60437b5` binaries; if unavailable, report that concrete
prerequisite rather than substituting a historical binary or building one.
A later source pin requires root's explicit selection and a fresh narrow diff
review. No new shell, worker, build or installation is part of this plan.

| Order | Printed tasks, with only explicit isolation substitutions | Evidence / stopping rule |
| --- | --- | --- |
| 1 | Verify supplied binary hashes/catalog and root's port reservation. Create one fresh private lab under this worktree, mode 0700, umask 077; private synthetic password and separate server/ctl session files. | Record edition, feature set, source pin and public command summaries. Require at least 8 GiB free. Do not read home credentials, existing stores or old target outputs. |
| 2 | Section 2: maintenance init with the printed issuer/listen/data-dir/admin; isolated `--config`, `--password-stdin` and `--non-interactive`. Start one serve PID; readiness; server CLI login/doctor. | Fresh Essentials redb, no advanced setup; exact issuer and successful authorized doctor. Password/session values never enter the report or command transcript. |
| 3 | Section 3: riauthctl login; **first** printed confidential local-demo create to a new private file; discovery/whoami. | New destination mode 0600 and secret omitted from stdout; exact client/scopes/callback fields. No app on port 3000 is launched and no OIDC redirect or consent is claimed. On uncertain issuance stop and report; do not recreate or attempt secret recovery from the direct receipt. |
| 4 | Section 5: fresh server CLI login, maintenance keygen, authenticated online backup; stop and await the owned server; maintenance restore to a new directory; legacy recovery status on the restored config. | Verified v3 archive, separate key, new restored redb and serving closed/pending recovery. No `complete`, persistent-credential attestation, factor reset or restored server. |
| 5 | Proposed section-6 restart of the **original** config/readiness. Sections 6–8: ctl login, group revision/create/revision/add-member/get/has-member; save the exact claims object, revision/client update; server explain and audit/inventory/report. | Preserve each key/revision on any retry. Expect staff membership, simulated groups/department claims with `token_issued:false`, and correlated group audit rows/private CSV. Do not present explanation as token issuance. |
| 6 | Stop/await only owned serve PID, verify the reserved listener is gone, remove only this disposable lab and its synthetic credentials after keeping a redacted result record. | Record actual exit statuses, command substitutions, cleanup outcome and failures. No broad process kill, shared-file overwrite, external dependency or secret publication. |

Stop at the first concrete failure or the time bound. Report it without changing
production code, bypassing policy, refreshing a retry fingerprint or launching
extra services. No browser, SMTP, LDAP/SCIM fixture, automation, Cargo, benchmark,
edition switch, cloud call, restore attestation or accessibility scan is included.
Any later necessary desktop work must use **RiWork Cua.ai Driver only**, inspect
its descriptions/current state first, and report permission/setup failure
without changing providers.

This checkpoint supplies current operator CLI evidence only. It deliberately
does not claim a new user's browser/passkey task, OIDC application completion,
source installation, invitation, peer setup or deployment verification.

## Checks actually performed and remaining scope

Performed: live project/task read; clean own-tree/status and fixed Git-object
inspection; repository `CONTRIBUTING.md` / `SECURITY.md` review; current source
and kept historical command/result inspection; guide/walkthrough whole-blob
comparison with own HEAD; exact old-context cardinality checks and
`git apply --check -` dry validation of both proposed diffs. No guide was
modified by those dry checks. No incoming `manual-accessibility-gates` Markdown
anchor was found at the fixed pin. Report whitespace/scope checks are recorded
with the handoff after commit.

No repository automation, Cargo/build/test/benchmark, service, browser, desktop,
external lookup or tenant operation ran. No historical artifact was re-certified.
This report does not depend on official release availability to hold D01: the
local printed-step corrections and missing printed Essentials walkthrough
evidence are concrete. Source installation and the relevant user browser tasks
remain unexecuted here; the chosen optional application/peer tasks retain their
named prerequisites. Root should assess the original new-user/new-operator gate
after the bounded checkpoint and applicable user evidence, without adding
canceled U10, whole-graph, vendor-conformance or release gates.

The concrete hunk summary and plan were sent through `riwork orchestrator send
--project 891e7443-8dac-4c1b-897f-9e53cb59c7ee` before any guide edit/runtime.
Runtime reservation remains pending; no task status was changed.

## Approved guide corrections and build proposal — 2026-10-02

This section updates the initial read-only audit above. Root authorized exactly
its proposed guide hunks; the ledger `wave30_D01_guide_corrections` was read from
the project's `planning/local-wave29-ownership-approvals.json`: `approved: true`,
`runtime_released: false`, D01's exact UUID and these three allowed paths.
The live original D01 row was reread and remains `in_progress`. No task status
was changed. The original independent-user/operator gate and canceled U10 scope
remain unchanged.

**The concrete guide corrections are now committed; runtime is still held.**
Guide-only commit: `6a5879f3877b1d780052f91fcafcf0a8933e38a7`.
It changes only `docs/essentials-guide.md` and `docs/platform-guide.md`,
109 inserted / 19 removed lines. It applies the approved direct-create
credential-already-issued/inspect-and-rotate guidance and preserves the separate
reviewed-creation recovery-receipt exception; adds fresh server CLI login after
passkey epoch revocation and restart/readiness of the **original** configuration
after the offline restored-store checkpoint; clarifies optional task routing,
confirmed format-3 offline upgrade without adoption bypass, and historical
device/browser evidence limits. Online backup, restore commands, pending recovery
attestation, headers, PAM fallback and all other printed commands remain intact.

### History alignment

No current matching Essentials artifact was found in the inspected locations.
The branch lacked 24 accepted runtime/client paths at the published pin, so
alignment was necessary for the proposed current-source build. History-preserving
merge `1a2bdedca89c7b43690d05f3b018cde05946024e` has both parents:
`46a3311f1773920a102d22dfb7098d13263f72ac` and
`c01c39ab4e092423d5522bedc50fff87656d8c0a`.
There was no reset, rebase, main edit or push.

Exactly two old report add/add conflicts contained empty own-side sections
against already accepted **additive root notes**. Removing the conflict markers
retained those notes and every original report line:
`local-wave29-a03-final-disposition.md` (root closure decision) and
`local-wave30-remaining-task-actionability.md` (dated historical-inventory note).
The resolved files are byte-identical to published `c01c39a`. Original report
commits and `46a3311` remain ancestors. There was no production conflict or new
implementation in this merge. This alignment and its narrow resolutions were
reported to the explicit project orchestrator.

All `src/`, `crates/riauthctl/`, server manifests/lock and toolchain inputs now
match `c01c39a`; the D01 commits change documentation only. No stale source file
was substituted and no other lane's accepted source was rewritten.

### Exact static verification performed

The approved proposals were read from immutable report commit `46a3311`,
checked, then applied only to the two allowed guide paths. The final files were
independently reconstructed in memory from the published guide blobs and each
approved patch hunk, with exact old/new line counts and old-context assertions.

| Static check | Actual result |
| --- | --- |
| Both resulting guide files equal the bytes reconstructed from the approved proposal | Passed, exact equality. |
| Reverse `git apply --check` of each approved proposal on the resulting file | Passed for both. |
| Original fenced blocks as an unchanged ordered subsequence | Essentials **27/27**, Platform **42/42**. This includes original shell commands and JSON/TOML records. |
| Additional fenced shell blocks | Exactly three per guide: `riauth ... login admin`, original-lab `riauth --config ... serve`, and loopback `curl ... /readyz`. No other command block was inserted or modified. |
| New offline-upgrade Markdown target and fragment | `docs/operations.md` exists and contains `## Rate limits and admission`. |
| Reviewed-creation receipt exception reference | Existing client README and its explicit reviewed-creation exception text verified; direct marker/refusal remains in `src/management.rs`. |
| Canceled blanket device/manual gate language | `manual gates` absent from both resulting guides; historical unexecuted-device evidence remains. |
| Whitespace and commit scope | `git diff --check` and `git diff --cached --check` passed; staged guide commit contained exactly the two approved paths. |
| Accepted runtime preservation after merge | `git diff --quiet c01c39a HEAD -- src crates/riauthctl Cargo.toml Cargo.lock rust-toolchain.toml` passed. Both conflict-resolved reports match the accepted pin. |
| Five build manifest/lock/toolchain inputs | Byte equality to `c01c39a` verified; no build performed. |

Resulting guide object evidence:

| Guide | Git blob | SHA-256 |
| --- | --- | --- |
| Essentials | `be68580333ad5f2238ac1966d164bf0fbf80aca1` | `ac1b10ddf08158a5286c6de0c3233e51f5e0402d76579984c5b3595edacaa4c4` |
| Platform | `934d1bf1d7150a546101319661bab2a05ade4a11` | `2b8c1d222d1b2fb38090ecaed3cd725182729863446b9280a9bf83a8de0e7c4f` |

### Existing binary provenance inspection — no execution

The inspection covered executable filenames in this worktree's private targets,
known `/tmp/riauth-*` snapshots, the known `~/.cache/riauth-cargo` build caches,
project planning records and tracked historical artifact records. It excluded
deployment-private credentials. No `riauth`, maintenance or client binary was
executed, and no existing store/session/password was read. This is a bounded
inventory, not a claim to have searched every installed executable on the host.

| Candidate inspected | Actual metadata/hash result | Why it is not a current matching Essentials artifact |
| --- | --- | --- |
| Own `target-wave27/cargo/debug/riauth` | 255814720 bytes; SHA `694d314c0aeb8a0266e11c4123783789005693f500dd2b29bf0fb0e25374876f`. Associated `bin-riauth.json` records `default, essentials, platform, test-support`. | Platform/test-support feature set; no current Essentials-only provenance. |
| Own sibling maintenance | 67010608 bytes; SHA `83ccc6663221a148bf7d925929267d827a4ae05fedc2b7cad040e755fbde9e9f`. Associated maintenance fingerprint has the same four features. | Same feature mismatch. |
| `q09-essentials-6ca4779-kept/riauth` | 205063032 bytes; freshly read SHA `c6f0ba061663393c0d7874e1dac19ccd2bca48d781555e86a7717b4653a8a7dd`, matching the historical reference. | Kept historical candidate, not evidence of a current `c01c39a` build; its filename alone is not compiler/source provenance. |
| `q09-8x8-47aa248/kept/essentials-debug/riauth` | 202086936 bytes; freshly read SHA `e36a139ae67559dd221ca28798999e8ad55e0ab7868a573f363e44371aac5d25`, matching the tracked Q09 record at `47aa248ce1c68284773746bbb5fd59c6098afdea`. | Explicit historical source/build, not current source. |
| Temporary snapshots | Inventory includes the older Platform `58357fd`, tools `f430c2f` and other historical targets; no current source-pinned Essentials server/maintenance pair was identified. | Neither a directory name nor a version string establishes current edition/source equivalence. |

The new hash reads above are static artifact inspection in this approved
follow-up; they do not change the initial audit's statement that it had not
recomputed historical hashes. No accepted target was used for building.

### One exact private locked Essentials build proposal

**Proposal only — not run.** Root schedules this after the existing O06/CI slot.
Working directory: this assigned worktree. Source inputs: published
`c01c39ab4e092423d5522bedc50fff87656d8c0a`, retained by merge, with only D01
documentation changes on top. Host observed with `uname -sm`: Darwin arm64.
Toolchain is the repository's pinned Rust 1.98.1. Edition/features:
Essentials only, default features disabled, no Platform/test-support/fuzzing/USB.
Build precisely the server and matching maintenance executables:

```sh
env CARGO_TARGET_DIR='/Users/dominik/orca/projects/riAuth-public-preview-local-module-boundaries-wave27/target/d01-essentials-c01c39a' CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo build --locked --manifest-path '/Users/dominik/orca/projects/riAuth-public-preview-local-module-boundaries-wave27/Cargo.toml' --no-default-features --features essentials --target aarch64-apple-darwin --bin riauth --bin riauth-maintenance
```

This is one Cargo invocation and a new private target under the assigned
worktree. Its expected outputs are
`target/d01-essentials-c01c39a/aarch64-apple-darwin/debug/riauth` and sibling
`riauth-maintenance`. It does not install binaries, modify accepted/main,
start a service or run tests. It does not build the independent client workspace:
root's checkpoint reservation also identifies the base riauthctl artifact and
its source/hash evidence. Do not silently substitute the old `f430c2f` client.

Exact build-input SHA-256 values inspected:

| Input | SHA-256 |
| --- | --- |
| Server Cargo.toml | `58e5ef824ed96290179c9f76fea208dc37173caeee21b6ce8d37f8a6dd1abcb8` |
| Server Cargo.lock | `b5c9d11c001244b8017303ce8c20516e02946483845eee40720b0910758d4426` |
| rust-toolchain.toml | `887f9be066a15585a2c583578e84b0fcb541126d81546276bad3d2ff00d61167` |
| Client Cargo.toml | `af385d4d989c53987396edfdfa684cd3902a603d1cf65dd31ac72da7f08de9ea` |
| Client Cargo.lock | `2998555ddb2d00e8130a970fcacfed19dfee7a0b2e3305011ebd40746f67b4db` |

Disk snapshot at **2026-10-02T11:13:23Z**, `df -k .`: **15430836 KiB
available (about 14.72 GiB)**. Space declined while other authorized work ran,
so this measurement is not permission to start a build. Root should remeasure
immediately before the serialized slot and require at least **12 GiB** free,
allowing a planning budget of **4 GiB** for this no-debug/incremental-off target.
That budget is an estimate, not a measured build size. Monitor free space every
two seconds during the owned Cargo process; stop the owned process at **9 GiB**
to leave a margin above the mandatory **8 GiB** floor. If the budget/floor is
unavailable, defer the build; do not delete existing evidence/backups or take
another lane's target. Capture actual elapsed time, exit status, toolchain,
source/features, produced binary hashes and disk observations when root releases
the command. No monitor or build was launched here.

The existing one 20-minute disposable CLI checkpoint remains the next runtime
proposal, now against the root-selected current build rather than the historical
snapshots. It still leaves restored recovery pending and excludes browser,
application launch, extra fixtures, cloud operations and credential attestation.
Current printed Essentials execution and user/browser evidence remain uncredited;
D01 stays in_progress. The resolved docs gaps are no longer implementation
blockers. No new artifact request is sent to the user: root owns build/artifact
scheduling and the subsequent precise runtime release.

Report verification: the original `46a3311` report is an unchanged byte prefix;
there is one appended follow-up section, Markdown fences balance, both guide
SHA-256 values still match the committed files, and the guide commit's exact
scope is the two reserved paths. Report `git diff --check` passed. The separate
report commit and final clean-tree check are supplied with the explicit-project
handoff; no runtime/build result is claimed.
