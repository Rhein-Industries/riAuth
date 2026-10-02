# D01 printed-step audit and bounded walkthrough proposal

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Task: `a96a1977-3210-4284-8f7d-645793369301` (D01).
Worktree: `42bb51c6-c198-4adb-bd92-0a5222853231`.
Audited published source: `60437b59933cadd40a1f5fbbb91ba153aee56456`.
Own starting HEAD: `6e1e1a9a9f206bc98bb03da38bbee3f27ee32425`.
Date: 2026-10-02.

Reading note: the initial audit below describes the report-only `46a3311`
phase. The later approval/build-proposal section records `6a5879f` / `551a3be`
checks as performed then. Applied proposals are now mapped to their immutable
commits rather than embedded; the final checker-correction section records
this update. Earlier unchanged-prefix claims describe the earlier report,
not this replacement. Runtime and original acceptance evidence remain pending.

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

## Applied guide hunks and immutable proposal provenance

The exact proposals, including their original guide-relative links and then
unqualified older-binary statement, remain immutable in
`46a3311f1773920a102d22dfb7098d13263f72ac:docs/roadmap/local-wave30-d01-walkthrough-report.md`.
The applied guide commit is `6a5879f3877b1d780052f91fcafcf0a8933e38a7`;
the embedded patch blocks are replaced here because the docs checker scans
links inside fences relative to this report's directory.

The proposals were generated against these exact published blobs:

- [Essentials guide](../essentials-guide.md): `2b8408f90d82bb35d49fc1ebd3512d9a1a853962`.
- [Platform guide](../platform-guide.md): `46f41f11fe211343be9059966253118401ffb4a0`.
- Historical walkthrough: `ef367ba40fdec9b26e6bef462d9a07de830e0d3b`.

The table accounts for every applied zero-context hunk in `6a5879f`:
nine Essentials hunks and ten Platform hunks. Coordinates are the exact
old/new ranges from `git show --format= --unified=0 6a5879f`, not current
guide line numbers after later accepted changes.

| Applied change | Essentials old/new hunk | Platform old/new hunk |
| --- | --- | --- |
| Small-install route and optional external facets | `-5,0 +6,6` | `-5,0 +6,6` |
| Fresh-store distinction; offline agreement recording with both confirmations and no adoption bypass | `-55,0 +62,12` | `-132,0 +139,12` |
| Renew expired or revoked standalone-client sessions | `-84 +102` | `-163 +181` |
| Direct-create exact retry refuses secret recovery; inspect/rotate; separate reviewed-creation recovery receipt | `-265,3 +283,7` | `-406,3 +424,7` |
| Prompt refers to recorded device/browser limits | `-332 +354` | `-497 +519` |
| Fresh server-CLI administrator login before backup after epoch revocation | `-367,0 +390,7` | `-572,0 +595,7` |
| Restart original lab/readiness; restored store stays pending; renew client session | `-444,2 +473,17` | `-701,2 +730,17` |
| Device/browser limits heading | `-905 +949` | `-2299 +2343` |
| Recorded gaps are not required manual steps; keep unexecuted evidence | `-918 +962,2` | `-2312 +2356,2` |
| Explicitly unexercised physical/synced/mobile/screen-reader coverage | No additional hunk | `-2318 +2363` |

Root's subsequent accepted qualification is
`fd6d8c8af6b65c2dbe47afb385289b26e2c1e643`, inspected directly: **binaries
that support only agreement formats 1 or 2 refuse format 3; rollback to
those binaries needs a compatible pre-command backup**. The historical
proposal's blanket older-binary assertion is superseded, not current guidance.
This correction does not edit either guide or replace that accepted wording.

The first client-create command remains simple: riauthctl fills its required
revision/key envelope. Group writes retain their printed exact revision/key
pairs. Plan/removal confirmation semantics are unchanged. The reviewed-creation
secret receipt, per-route optional/required headers and PAM revision fallback
remain accepted contracts. No credential, Group or remote-IO behavior is changed.

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

## Docs-checker correction — 2026-10-02

Root requested this report-only correction after reviewing the guide commits.
The two applied embedded diff blocks are replaced by the exact commit/hunk
mapping above; original proposal text remains at immutable `46a3311`, not
silently corrected or lost. The original acceptance, findings, historical
executed results, refusals and unexecuted limits are unchanged. Every byte from
the bounded-plan section through the former end of report `551a3be` is retained
unchanged before this appended section (SHA-256
`63d14495e0cbfc161ea67d88ae5861a66196059a2d4ccae5b6a73524ee2a9f0d`).
Root's subsequent `fd6d8c8` formats-1/2 refusal/rollback qualification is recorded
above; neither guide nor the global checker is modified by this correction.

Actual checks performed:

- Before correction, `python3 scripts/check-docs.py` exited **1**: eleven
  guide-relative missing-link entries in the embedded patches, plus five
  existing build-directory layout errors.
- After correction, the same checker exited **1** with **no Markdown link
  errors**. Its only errors are the pre-existing `target-wave29-source`,
  `target-wave28-scim`, `target-wave28-portal`, `target-wave28` and
  `target-wave27` directories. They are preserved; no artifact/evidence was
  removed or moved to obtain a passing result. This is not a full-check pass.
- Static comparison against `git show --format= --unified=0 6a5879f` verified
  **19/19** exact applied hunk coordinates in the map. All report-local Markdown
  link destinations exist. Original acceptance/findings/historical evidence
  and all later evidence were compared byte-for-byte as described above.
- `git diff --check` passed. Changed scope is this report only; staged
  whitespace/scope and final commit/clean-tree results accompany the handoff.

Build and runtime remain held until root releases the signer/budget slot.
No Cargo, executable walkthrough, installation, service, browser, desktop,
source/guide edit, merge, push or task-status mutation occurred in this phase.
D01 remains in_progress; its original independent-user/operator gate is not
replaced by the link check. Root owns integration and closure.

## Released matching Essentials build — 2026-10-02

Root released the sole Cargo slot after the six budget cases exited 0. The
one authorized server/maintenance build **exited 0**; the slot was released
through the explicit project orchestrator immediately after receiving its
exit/hash result. No second Cargo build, client build or walkthrough ran.
Root also reported the accepted-worktree docs checker passed after porting
`b8f2784`; that is root's result, distinct from this tree's earlier recorded
five layout failures.

Actual source: `c01c39ab4e092423d5522bedc50fff87656d8c0a`-equivalent production
inputs, built from own HEAD `b8f27841bc4085303dc2642a72fc327e68f90308`.
Its only differences from that pin are the two guides and this report. Source
tree object: `3adc2b59c3547d22bff202daccfd8ad97f1e78ab`. Server manifests,
lock and toolchain file match the earlier exact input-hash table byte-for-byte.
Additional config/policy input hashes:

| Input | SHA-256 |
| --- | --- |
| src/config.rs | `6bf9a9fc6e4274a87fe175752edee4ac8b8995011af71e7756bf64a5cdf740d5` |
| src/node_security.rs | `979f1d22a4835bce97866e302dc2a8a21df4f0d383d4bd1e6960b6a5d7a3bcb7` |

Actual toolchain: `rustc 1.98.1 (48a229cea 2026-09-01)`, full compiler commit
`48a229ceaefd4985c50990b14116b6d856af0985`, host `aarch64-apple-darwin`, LLVM
22.1.8; `cargo 1.98.1 (797e8a9bc 2026-08-05)`. No Cargo config file exists at
working-directory ancestors or active Cargo home. The checked Rust flags,
compiler/wrapper, toolchain and build-target override variables were unset.

A local Python process launched and monitored exactly this Cargo argv in the
assigned worktree, using the released environment below:

```sh
cargo build --locked --no-default-features --features essentials --target aarch64-apple-darwin --bin riauth --bin riauth-maintenance
```

`CARGO_TARGET_DIR` was the absolute own-worktree path
`target/d01-essentials-c01c39a`; `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`. Both produced compiler
fingerprints contain exactly `["essentials"]` and empty Rust flags: no default,
Platform, test-support, fuzzing or USB feature. The new target was absent at
preflight; no accepted target, existing cache deletion or artifact substitution
was used.

Started `2026-10-02T11:32:08.766800+00:00`; ended
`2026-10-02T11:35:27.497222+00:00`; elapsed **198.729 seconds**. Initial free
space was **14237978624 bytes (13.260 GiB)**, above the 12 GiB start threshold.
The two-second monitor recorded 100 observations (maximum actual sample gap
2.007 seconds). Minimum/final observed free space was **12958531584 bytes
(12.069 GiB)**; no 9 GiB stop was triggered and the 8 GiB floor was preserved.
The target occupied `1500880 KiB` by `du -sk` after completion.

| Built executable under `target/d01-essentials-c01c39a/aarch64-apple-darwin/debug/` | Bytes | SHA-256 |
| --- | --- | --- |
| riauth | 183038592 | `7abf745c10691a012a1918c83089168d0e0d88b42764dbf8ed538b90c828b606` |
| riauth-maintenance | 56499296 | `86490c7f71de6b7ae9d4dabdf9060a8757100f3aedbdaa670284d9e1206a2a95` |

Private evidence in that target: `build-provenance.json` (SHA-256
`39c9167129193aa9e4c861992a44696c614bcf689ecf99b27334a66df7b59684`),
`build-result.json` (`a2da665f407dba364de074fe693900142f24b2a43161b03281a30e1b486795b7`),
`build.stdout.log` (`ab67b2837d4a775bf8ac364147abf135c3fc41fe8876f5f586e73b4829cbe369`)
and `disk-observations.jsonl`
(`883b388645bea4cc361998ab94a77dffbf8389e18549f979b35abd4d07908af7`).
The build emitted three existing dead-code warnings: passkey workflow discard,
workflow registration start/verify, and `PostLogoutReturn::allowed_by`. No
warning-driven source change or extra build was attempted.

Post-build static checks freshly recomputed both executable hashes and verified
their sizes/exact Essentials fingerprints, unchanged pinned production inputs
and a clean tracked tree before this report appendix. The executables were not
run, installed or described as official released artifacts. The matching pair
now exists; standalone-client provenance and the disposable port-9000 checkpoint
remain separately scheduled by root. D01 stays in_progress against its original
independent-user/operator gate; no task status was changed.

## Released base-client build — 2026-10-02

Root released exactly one base-client build in ledger
`wave30_D01_base_client_build` (`approved: true`, `runtime_released: true`
for this build only). The entry specifies reuse of the warm private target,
the 9 GiB stop and 8 GiB floor. Operator/service/binary execution remains held.
The one build **exited 0** in **27.764 seconds**; exit, binary hash and result
were sent through the explicit project orchestrator and the Cargo slot released
immediately after receiving them. This appendix was added after that release.

Client source/manifests were verified against reviewed
`c01c39ab4e092423d5522bedc50fff87656d8c0a`, built from own HEAD
`d87eb1522d08a6836992b6b003f63ef36058f8c1`. Entire client tree:
`27344f623abd1829a9eb408af6b3e2d5ed16ab31`; client source tree:
`f909fcccab7f9e39234df7df9a25b4816efe3569`. Exact SHA-256 inputs:

| Input | SHA-256 |
| --- | --- |
| crates/riauthctl/Cargo.toml | `af385d4d989c53987396edfdfa684cd3902a603d1cf65dd31ac72da7f08de9ea` |
| crates/riauthctl/Cargo.lock | `2998555ddb2d00e8130a970fcacfed19dfee7a0b2e3305011ebd40746f67b4db` |
| rust-toolchain.toml | `887f9be066a15585a2c583578e84b0fcb541126d81546276bad3d2ff00d61167` |

Actual toolchain remained Rust/Cargo 1.98.1, compiler full commit
`48a229ceaefd4985c50990b14116b6d856af0985`, Cargo commit `797e8a9bc`, native
`aarch64-apple-darwin`, LLVM 22.1.8. No Cargo config files were found in working
directory ancestors or active Cargo home; checked Rust/compiler/wrapper/
toolchain/target overrides were unset. Actual Cargo argv used the absolute
own-worktree client manifest path:

```sh
cargo build --locked --manifest-path "$PWD/crates/riauthctl/Cargo.toml" --no-default-features --target aarch64-apple-darwin --bin riauthctl
```

Environment: the same absolute `target/d01-essentials-c01c39a` private target,
`CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`,
`CARGO_PROFILE_TEST_DEBUG=0`. Client fingerprint features are exactly `[]`,
with empty Rust flags: base client, no terminal-usb. No new target, server/test/
other build or cache deletion was performed. The log has no warnings or server
package compilation line; the independent client workspace was used.

Started `2026-10-02T11:53:05.903140+00:00`; ended
`2026-10-02T11:53:33.667980+00:00`. Initial free space was **12280758272 bytes
(11.437 GiB)**. The two-second monitor recorded 14 observations, maximum actual
sample gap 2.005 seconds. Minimum/final free space was **12090331136 bytes
(11.260 GiB)**; no stop was triggered and the 8 GiB floor was preserved.
The shared private target occupied `1703824 KiB` by `du -sk` afterward.

Produced `target/d01-essentials-c01c39a/aarch64-apple-darwin/debug/riauthctl`:
**20406784 bytes**, SHA-256
`bfbbb322f1a66d0ac9998beb9fb5838097cea0442cea3fd3a1d057fcb3f600cf`.
Compiler fingerprint SHA-256:
`7657f171d9e3a951215d9e5995f6667eed5e02cce9fec52b467bfc1de072442f`.
The prior Essentials server and maintenance SHA-256 values were freshly
verified unchanged both before and after this build.

Private evidence retained in the same target:

| File | SHA-256 |
| --- | --- |
| client-build-provenance.json | `19a19fe12e23ec465b2f5434b3ce50c2e525e0326eaa71c20fde52e4927319f0` |
| client-build-result.json | `4337fa7f7341b6977b880e8b73720873b7b0ba5b9fdfb90e997e51630c1b4919` |
| client-build.stdout.log | `52997132a8df9e6ad675d87357f94c5b17984114f91606d292b7c8937ef0b872` |
| client-disk-observations.jsonl | `4d1fb489f60648685711e097f9ab182f27131a457f2a7bd328e244811a13bc8b` |

Post-build static checks freshly verified client hash/size/empty features,
unchanged Essentials pair, pinned client inputs and clean tracked state before
this appendix. No produced binary was executed, installed or claimed as an
official release. The complete local matching artifact set is now available
for root review; the disposable port-9000 checkpoint remains separately held.
No guide/product edit, merge, push, desktop, worker/task creation or status
mutation occurred. D01 remains in_progress against the original gate.

## Released printed Essentials operator checkpoint — 2026-10-02

Root released ONE up-to-20-minute checkpoint in
`wave30_D01_operator_checkpoint`, for D01's exact task/worktree IDs.
All **27 CLI/probe commands exited 0**, plus both owned original-configuration
`serve` lifecycles exited 0. Cleanup ended at
`2026-10-02T12:11:32.327769+00:00`; wall elapsed **607.862 seconds**
(about 10 minutes 8 seconds), within the bound. No printed-command failure,
silent retry, alternate API or bypass occurred.

Execution source was the reviewed `c01c39a` production input recorded in the
build sections, own HEAD `950fc6ecb14fcbafd4587f95987b9231984994c2`.
All three exact artifact hashes above were recomputed before use. Printed
commands came from Essentials guide blob
`be68580333ad5f2238ac1966d164bf0fbf80aca1`; root's later `fd6d8c8`
qualification changes the unexecuted older-format paragraph only and remains
the current refusal/rollback guidance.

The only substitutions were absolute own-target executable paths, a fresh
private lab for config/data/settings/key/archive/credential/CSV paths, isolated
XDG directories for both CLI sessions, and `--password-stdin` /
`--non-interactive` for the private synthetic password input. No `HOME`
change or existing home credential/session/store read occurred. The lab was
0700; retained redacted evidence is 0600. Raw program stdout/stderr were
captured transiently for public-result selection, never logged; server output
was discarded. No secret/token/key/cookie value appears in the evidence.

### Actual redacted command outcomes

Aliases in the following exact argv rendering: `$RIAUTH`, `$MAINT` and
`$CTL` are the matching binaries under the assigned
`target/d01-essentials-c01c39a/aarch64-apple-darwin/debug/`; `$LAB` is
the one disposable private lab. All quoted text is a public argv value;
password input is omitted.

| Printed stage | Actual argv with private paths replaced by aliases | Exit | Observed public outcome |
| --- | --- | --- | --- |
| section2.init | `$MAINT --config $LAB/riauth.toml --non-interactive init --issuer http://localhost:9000 --listen 127.0.0.1:9000 --data-dir data --admin admin --password-stdin` | 0 | Initialized fresh config and redb data directory. |
| section2.ready | `curl --fail http://127.0.0.1:9000/readyz` | 0 | Probe passed; listener owned by PID 75420. |
| section2.server_login | `$RIAUTH --server http://localhost:9000 --non-interactive login admin --password-stdin` | 0 | Administrator identity; server session mode 0600. |
| section2.doctor | `$RIAUTH --server http://localhost:9000 doctor` | 0 | Exact issuer; returned public diagnostics. |
| section3.client_login | `$CTL --server http://localhost:9000 --non-interactive login admin --password-stdin` | 0 | Administrator identity; separate client session mode 0600. |
| section3.client_create | `$CTL --server http://localhost:9000 client create local-demo --name 'Local demo' --confidential --redirect-uri http://localhost:3000/callback --scope openid,profile --secret-file $LAB/local-demo-secret.json` | 0 | local-demo; openid/profile; exact callback; new nonempty credential file mode 0600; no top-level credential value fields in captured stdout. |
| section3.discovery | `$CTL --server http://localhost:9000 discovery` | 0 | Exact issuer and /oauth/token endpoint. |
| section3.whoami | `$CTL --server http://localhost:9000 whoami` | 0 | Administrator identity. |
| section5.fresh_server_login | `$RIAUTH --server http://localhost:9000 --non-interactive login admin --password-stdin` | 0 | Fresh password administrator session; §4 revocation itself skipped. |
| section5.keygen | `$MAINT keygen --out $LAB/backup.key` | 0 | New private key file mode 0600. |
| section5.online_backup | `$RIAUTH --server http://localhost:9000 backup --key-file $LAB/backup.key --out $LAB/backup.riauth` | 0 | New nonempty private archive mode 0600; server still running. |
| section5.offline_restore | `$MAINT restore --backup $LAB/backup.riauth --key-file $LAB/backup.key --out $LAB/restored` | 0 | Verified true; redb; serving_allowed false; new restored config. |
| section5.recovery_status | `$RIAUTH --config $LAB/restored/riauth.toml recovery status` | 0 | Pending record present; serving_allowed false; no attestation. |
| section6.ready | `curl --fail http://127.0.0.1:9000/readyz` | 0 | Original config restarted; probe passed. |
| section6.client_login | `$CTL --server http://localhost:9000 --non-interactive login admin --password-stdin` | 0 | Renewed isolated standalone-client session. |
| section6.revision_before_create | `$CTL --server http://localhost:9000 revision` | 0 | Revision 1. |
| section6.group_create | `$CTL --server http://localhost:9000 --run-id staff-group --idempotency-key staff-create --if-revision 1 group create staff` | 0 | staff-create / revision 1 / staff-group. |
| section6.revision_before_member | `$CTL --server http://localhost:9000 revision` | 0 | Revision 2. |
| section6.group_add_member | `$CTL --server http://localhost:9000 --run-id staff-group --idempotency-key staff-add-admin --if-revision 2 group add-member staff admin` | 0 | staff-add-admin / revision 2 / staff-group. |
| section6.group_get | `$CTL --server http://localhost:9000 group get staff` | 0 | Group record returned. |
| section6.group_has_member | `$CTL --server http://localhost:9000 group has-member staff admin` | 0 | member true. |
| section7.revision_before_claims | `$CTL --server http://localhost:9000 revision` | 0 | Revision 3. |
| section7.client_update | `$CTL --server http://localhost:9000 --run-id local-demo-claims --idempotency-key local-demo-claims --if-revision 3 client update local-demo --scope openid,profile,groups --settings-file $LAB/local-demo-claims.json` | 0 | Scopes groups/openid/profile; exact printed settings object. |
| section7.explain | `$RIAUTH --server http://localhost:9000 explain local-demo admin --scope 'openid profile groups'` | 0 | Simulation true; allowed true; token_issued false; staff and department=lab projected. |
| section8.audit | `$RIAUTH --server http://localhost:9000 audit --limit 100` | 0 | 11 rows; both correlated group actions observed. |
| section8.inventory | `$RIAUTH --server http://localhost:9000 inventory audit --limit 100 --filter staff-group` | 0 | 2 exact-run rows; revision 4; cursor absent. |
| section8.report | `$RIAUTH --server http://localhost:9000 report audit --run-id staff-group --out $LAB/audit-staff-group.csv` | 0 | 2 exact-run CSV rows; printed 7 columns; mode 0600. |

The exact section-7 JSON block was copied from the guide into a private settings
file and applied once. Membership/create and claims writes used their printed
distinct keys and freshly read revisions 1, 2 and 3. Audit inventory confirmed
revision 4. All audit views correlated `group.create` and `group.member.add`
with `staff-group`; the CSV columns were exactly `id, at, actor, action,
target, run_id, request_id`. Preview was a simulation, not token issuance or
application login.

### Ownership, recovery and cleanup evidence

Before launch, `lsof -nP -iTCP:9000 -sTCP:LISTEN` exited 1 with no listener;
an exclusive bind check on `127.0.0.1:9000` succeeded. Initial server PID
**75420** exclusively owned the listener, served only the original lab, and
was stopped after online backup with a PID-scoped interrupt: exit 0, no listener
remaining. Only then was the archive restored offline. Status returned
`serving_allowed: false` and a pending record. No restored-store server,
completion, attestation or factor recovery was attempted.

Original-config restart PID **84673** exclusively owned port 9000. Its
PID-scoped finally cleanup also exited 0 and left no listener. No unrelated
process was signaled or stopped. The only disposable lab, including synthetic
password, sessions, credential, key, archive, CSV and pending restored store,
was removed after redacted evidence was retained. The matching build artifacts
and all prior evidence remain intact.

Initial free space: **11859480576 bytes
(11.045 GiB)**; final snapshot:
**11883405312 bytes (11.067 GiB)**.
Each command checked the 8 GiB floor; each owned-server guard checked an
8.5 GiB stop margin every second and enforced the overall 20-minute deadline.
No guard stop or floor failure occurred. A numeric runtime minimum was not
logged and is not inferred from these two snapshots.

Retained 0600 redacted evidence:
`deployment-private/d01-wave30-operator-685d00b092a8.redacted.jsonl`,
34 records, SHA-256
`a476b366da75b7a7c52a9f5b3edd5b615ea72beeb527d7801289501334774f3e`.
Static validation confirmed all 27 exit records, both server exits, removed
fixture, unused final listener, evidence mode/hash and unchanged pinned
production source. No runtime or build was repeated for those checks.

The immediate explicit-project release handoff was attempted after cleanup,
but RiWork rejected it with exit **2**:
`leave copy mode and enable terminal input before submitting`.
The resource is already free; the rejection was reported in commentary.
The post-commit handoff records whether the receipt retry was delivered.
No terminal/desktop-provider workaround or input-mode change was made.

### Credited scope and remaining gate

This supplies current local **operator** evidence for §§2, 3, 5 and 6–8.
Install, browser first-administrator setup/user login, passkey §4, actual epoch
revocation, OIDC application login/consent, LDAP/SCIM/mail/invitations, recovery
attestation/break-glass, edition switching, peers and deployment were skipped.
No independent browser-user acceptance or full D01/release completion is
claimed. No failure-driven guide correction is needed for this executed path.

No Cargo/build/test, helper/product/guide edit, extra service/worker/shell,
desktop, merge, main edit, push or board/status mutation occurred. This report
is appended only after exit and cleanup. D01 stays in_progress against the
original independent-user/operator gate; root alone owns integration and
closure.
