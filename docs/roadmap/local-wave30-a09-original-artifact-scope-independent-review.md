# Wave30 A09 original artifact scope independent review

2026-10-03. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, original A09
`506e3979-a590-4af3-8fa8-ee90d3a517f2`, reservation
`wave30_A09_original_artifact_scope_independent_review`. Supporting WT
`e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`, branch
`roadmap/local-extension-isolation-wave27`, parent
`3e7a0d34b2ae7740091c920714716ec6bc06b4b5`. Primaryf2 is unchanged.
Fixed published source is `a6d361600a03713fc1b687f367e9db84efe43463`.
This new report is the sole write; it records static review and attribution of
accepted receipts, with no new product or artifact execution.

## Conditional disposition

**No additional concrete source blocker found for the reserved native ARM
container cohort. Original A09 is a conditional local DONE candidate if root
verifies that run `37101183416` completes the reviewed cohort successfully and
proves cleanup. No DONE recommendation applies to the currently ACTIVE run.**
At this review's boundary its terminal result, output artifact and application
assertions are unknown. I did not query, download or wait for them.

The original outcome is production and testing of the supported server, client,
container and maintenance assemblies. The existing native archives cover both
architectures and both server/maintenance editions; the accepted x64 container
receipt supplies actual shared-state/refusal evidence using those exact tools.
The ARM PostgreSQL receipt adds native, current-format shared-state evidence.
The remaining reachable local facet is the **root-verified native ARM container
terminal receipt for the already reserved cohort**, including the same actual
identity/authorization/configuration oracles. A runner label, completed builds,
valid archives, or successful capabilities alone cannot substitute for it.

If all conditions below hold, I identify no further local implementation or
new fixture required by the original A09 row. This recommendation does not
convert the dated LOCAL cohorts into official shipped releases, claim current
publication source equivalence, certify every shared capability, or discharge
the separate Q08/Q10/Q11 gates. Those limitations remain explicit in the original
architecture outcome's evidence rather than disappearing when its local work
is disposed of. Root alone decides original acceptance and board status.

## Original row and scope boundaries

I read the entire matching project/task row in the supplied project export:
`planning/current-tasks.json`, 244,354 bytes, SHA-256
`0b38ca3eb8810a4628436eb5083f6329de848f3f4ddaa972c850601ad45c0835`.
Its observed file mtime was `2026-10-03T05:51:05.061711Z`. This is a dated local
export, not a new live API read. It says in_progress, primary WT
`f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`, prerequisites A04/A05/A08/Q08.
The dated leave-todo scheduling text does not override the current reservation.

Exact requested outcome:

> Produce and test server, client, container, and maintenance artifacts for explicitly supported platforms, including Linux x86-64 and ARM64.

Exact workstream completion gate:

> Switching distributions does not create different users, different authorization rules, or silently ignored configuration.

The row also requires the same identity, authorization, revocation and
credential-protection semantics for shared capabilities, relevant actual tests
and documentation/artifact evidence, and honest remaining prerequisites.
I read the complete related A04/A05/A08/Q08/Q10/Q11 export rows to distinguish
their outcomes, without changing any assignment or status.

| Original responsibility | Evidence/disposition boundary |
| --- | --- |
| A09 packaging and supported architectures | Native GNU Linux x86-64/ARM64 server, maintenance and remote-client archives; corresponding OCI variants; observed execution of their reviewed local checks and shared distribution behavior. |
| A04/A05/A08 | Separate executables, additive same-source editions, safe identifiers/configuration and incompatible downgrade refusal; accepted foundations retained, with actual artifact behavior checked here. |
| Q08 exact shipped bundles | Essentials/Platform, supported backends/architectures, exclusions and incompatible settings for the specifically selected shipped cohort. LOCAL hashes are inputs to evidence, not that cohort's release identity. |
| Q10 installed upgrade/recovery | Exact installed artifacts, schema/build transitions, restore, maintenance and supported rollback. Local transition samples do not certify this gate. |
| Q11 verifiable release/review | Signatures, inventory/provenance, review scope and vulnerability response. Checksums, action pins and internal independent source reviews do not supply a signature or external security assessment. |

[Product contracts](product-contracts.md), lines77–102,321–353,401–460 at the
fixed publication retain Linux `x86_64-unknown-linux-gnu` and
`aarch64-unknown-linux-gnu`, OCI `linux/amd64`/`linux/arm64`, same-source editions,
and native ARM execution. Its supported-platform **release gate** additionally
names installation/startup, HTTPS/discovery, management, passkey server
verification, revocation, configuration rejection, transitions, backup/restore,
dependency inventory and exact tested release provenance. These requirements
remain required for the corresponding support/release claim; this review grants
no HTTPS/passkey/installed-recovery/signing credit from a password-only LOCAL
cohort. The original row does not require creating a new universal browser,
protocol, tenant, hardware or full-suite campaign to recognize its produced and
tested local architecture assemblies. Optional USB-client hardware, undeclared
macOS/Windows releases and unrelated integration certification are not new A09
implementation demands. No requested shipped-source/device input is invented
or requested again here.

## Bodies read, reuse and pins

I read all current container helper lines1–2337 in bounded contiguous reads,
including both transport branches, archive validators, native/architecture
guards, build recipes, full snapshot/fixture/refusal oracles, resource ownership,
process joining, failure handling and cleanup. I read the entire current
109-line workflow, entire Dockerfile/.dockerignore, the complete 316-line native
archive checker, and complete 118-line store probe. No helper was imported and
no candidate function was invoked.

Previously completed Q08 full-body reads of the native workflow, exact-edition
matrix and installed gate are reused only after whole-byte equality checks.
The complete PostgreSQL helper body is covered by the earlier reviewed lines
90–450 and this review's remaining portions, including renewed inspection of
the complete `shared_probe`. Earlier full reads of all four native/PG/x64
terminal receipts are likewise reused after exact byte equality to `544d134`;
the x64 terminal receipt was additionally reread in full here. Production body
reads are scoped: complete relevant password/session/me/logout/user/group
adapter and principal functions, UserView/Group/Session definitions,
node-security agreement/enforcement/transition functions, and earlier reviewed
edition/transition bodies at unchanged identities. This is not a full audit of
every production file or every capability's behavior.

| Fixed object | Bytes / SHA-256 |
| --- | --- |
| Container helper | 138,392 / `6726dfd945445873214e934aa1f20815dc99ab24660b2205a45fd43c1486f221` |
| Container workflow | 4,640 / `63887b4f57c959ce5256865c2befc7cc568b95e9eb7d10eff0909cf3dc6747b8` |
| Native artifact workflow | 19,487 / `62c378975021066d62d5b32d0eee2c87de8501d01ded1ae0fae8147e920e58d7` |
| PostgreSQL transition helper | 31,339 / `32a2952e3b4c3ff2ab73a747238862d3446c31bf4896c869bc9efacb1ba8fe49` |
| Native archive checker | 13,118 / `2f477665ed584bc148fb230c0658f6d0cc66aceaa5af631c4dc467872fa9ecce` |
| Snapshot probe | 5,722 / `da486cb8cd7c9d6dfcd78da2704e27688b43a8dd40c7fa58e5b4574f678aed32` |
| Dockerfile | 1,512 / `458ebb247170c6d4af2ff45b22e5a36e0a5380612140a43448d2ddcebc5d83cb` |
| .dockerignore | 794 / `4fbd9472316442bdd8b72e268feb1140701e87ac28dd02295c687b6d379eb093` |

The four current workflow/helper objects in the first four rows are exactly
equal at `a6d361` and `544d134`. Large historical reports were read only in
relevant sections: original artifact-plan scope and prior native/shared phases,
and ARM completion-plan original scope/source materialization and limitations.
Initial overlarge output was truncated and is not full-body evidence for their
3,024/697 lines. I make no such claim. Current BuildKit receipts were hashed and
their selected metadata inspected; this is not a new registry or tool execution.

## Provenance and architecture witnesses

| Accepted observation | Exact source and native platform | Actual scope |
| --- | --- | --- |
| Native x64 `37046857550`, job `110970324302` | Workflow `0d090169f20f61f7cb59b685dccb13203526539f`; product `b619fe25269ccc150e473bbcde47cdb3623ef810`, tree `a627df2ce21a4d255b8c1914d4f543e32f40f4de`; ubuntu-24.04 Linux/x86_64/X64, ELF62 | Five LOCAL archives, both server/maintenance editions and base riauthctl; eleven commands exit0, edition/capability/route/agent/downgrade smoke checks. |
| Native ARM `37016520583`, job `110868629053` | Workflow `036a392656b4b5070cc86a11d5ca3258b7b868d2`; product `9a819317efb3a13fa27cd86f884be2be00898fc0`, tree `1528b61ba463d9262d6252d54174748a176f313b`; ubuntu-24.04-arm Linux/aarch64/ARM64, ELF183 | Five LOCAL archives and eleven successful commands, with matching native checks. |
| ARM PostgreSQL `37082572962`, job `111086077482` | Controller `d644e2c617501aca8efd43f6350c72b788c19d4d`; validator `04f79568c98d518613e2acf5e9700ba6f0ca5c4f`; exact prior product9a binaries; PostgreSQL16.15 | Current-format-3 plaintext PostgreSQL E→P→E ordinary identity/grant/refusal/configuration sample; helper exit0,20.219462s. |
| x64 encrypted-redb/container `37099059596`, job `111134825816` | Controller `e17f723c211b9cba4c8f07257f3e40560325c223`; exact productb619/prebuilt images/reader; helper6726 | Passed eleven fixed cohort checks,2700 joined command steps,18 reader observations,10 preservation packets; no fresh image build in this run. |
| ARM container `37101183416` | Reserved closed ARM selection product9a/reviewb71; terminal source/tool/artifact details still require root's actual receipt | ACTIVE per supplied user state. No terminal, UID, application, reader, transition or cleanup credit yet. |

Both native receipts record `rustc 1.98.1 (48a229cea 2026-09-01)` and package
version0.1.1. Version labels establish neither equal source nor a shipped
cohort. Each receipt independently binds archive/binary lengths and hashes,
exact license/notices inputs, manifest/lock inputs, native ELF machine and actual
command logs. The native root receipts do not independently rehash their outer
ZIPs; that limitation remains distinct from later transport validation.

Native transport artifact IDs/digests:

- x64 `11246575279`,52,428,080 bytes:
  `7c1bcad0f78d90eb611ab76d67943a32a0d691cd5bbd9e5588512bdde635e2a0`.
- ARM `11232871527`,49,177,062 bytes:
  `fc2a83dd286b70e455802af7713b60724d97b9e598240f6d9037c279601e4d30`.
- PG output `11259328231`,19,973 bytes, root-rehashed:
  `2fb5052a1ee04d9aafbda4cfbad334ea34d7d3065f6468d44d9bc1a26e28a46b`.
- x64 container output `11265212727`,115,361,104 bytes, root-rehashed:
  `4afb3c28549f918e032d201a1b58f86fd2660adaac4e0e7dc8764025d8735435`.

These are retained public metadata witnesses, not new downloads or hashes of
archive contents in this review. The complete individual binary/archive hashes
remain in the linked root receipts rather than being replaced by a version-only
identity summary.

Within each architecture Essentials and Platform use the same product revision.
Across architectures, 9a and b619 are different whole trees. Their root/client
manifests/locks/toolchain, Store, model, principal, delegation, grants/sessions
writers, config, node-security, edition/transition, Dockerfile, snapshot probe
and artifact/installed/matrix/encrypted helpers were compared byte-for-byte;
the inspected files match. The entire Core matches between 9a and b619. Their six
production-path differences add Platform source-stage browser handling and
correct a cloud certificate-credential diagnostic. I read the relevant complete
diff, not those entire new source-stage files. Fixed a6 additionally differs
from b619 in `Core::list_groups` and portal signin.js. Current Core hash
`686e7e732f256e7b435ab69991b71fb26d7467214877d2e0f0c840741446caea`
therefore cannot label either dated native artifact as this publication's build.

The Dockerfile pins Rust1.98.1-trixie index
`sha256:a8a5f0a1e5fe7dfe1d352591e4a1c7dd2c08fd70475cae872cf3458ba0df0546`
and Debian trixie-slim index
`sha256:a99cfc517144bc59b1978475ec53b46ecabec7e43635402ee5b77cc54cd1b20a`.
The helper selects architecture-specific BuildKit manifests98cc6a3f… for amd64
and 3ad6bb9b… for arm64, with pinned review receipts and actual image config/OS/
architecture checks. It permits exactly one jobs1/inc0/debug0 resource prefix
whose removal restores the entire original Dockerfile. It does not establish a
minimum libc/distro support matrix beyond the named observed runner/image
baselines, or bit-for-bit equality between separately built native/image servers.

## Exact common-state oracles and their limits

The current helper's complete `fixture_gate`, `snapshot`, `revoke_copy`,
`config_refusals`, `direct_refusal`, `plan`, `handoff`,
`platform_refusal_fixture` and reader/preservation methods were reviewed.

1. Fresh labelled volumes must copy up as UID/GID10001/mode0700 before any
   credentials. Native matching maintenance performs keygen/init; config, key
   and encrypted redb must be0600. Server config mounts are read-only and data
   writable; application containers stay nonroot/read-only/cap-drop/no-new-
   privileges. The reader/maintenance executables are separately pinned mounts.
2. Ordinary delegate creation, group membership and auditor grant use normal
   public writers, current revisions and idempotency keys. Snapshot equality
   compares all12 UserView fields, complete group and grants, stable me fields,
   exact issuer/JWKS URI and parsed JWKS across E→P→E. Only new session ID/expiry
   are excluded from cross-login equality; no claim of preserved bearer values.
3. The ordinary grant actually authorizes audit HTTP200. Ordinary user creation
   must refuse HTTP403/client exit4; before/after visible user arrays match.
   Two logged-out credentials remain refused through subsequent handoffs:
   HTTP401/client exit3, with session file removal. This is actual sampled
   authorization/revocation, not all actions, protocols or receipt replay cases.
4. Issuer, session-policy and login-rate candidates each change exactly one
   parsed setting. Read-only maintenance plan must report the precise blocker,
   exit5; direct server open must return the matching marker, exit2. Exact
   config/key/candidate bytes and the full logical row/metadata/count snapshot
   must stay equal. Unsupported active-edition direct opens and retained
   Platform agent downgrade similarly refuse instead of silently dropping state.
5. Plan returns an exact nonempty token; explicit offline activation alone changes
   the edition. Writers are stopped first. Source agreement functions require
   format3 and compare issuer, authentication policy and all16 effective rates;
   target-only active capabilities and coordinated activation metadata are the
   deliberate transition changes. The container fixture's rate oracle checks
   the complete effective map plus successful enforced startup; it does not
   individually mutate all16 categories.
6. The bound read-only snapshot mode of the native test probe calls
   `Store::inspect`, hashes every decoded row, and returns five metadata fields
   and four counts privately. Other probe modes exist but the controller selects
   only `snapshot`; no fixture/raw-ledger mutation is used by this cohort.
   Reader inode/modes/copy/read identity and its own before/after physical bytes
   must agree. Refused product opens may change physical redb representation;
   full logical equality is the meaningful refusal oracle, with config/key
   equality separately mandatory. No arbitrary changed-row exception is allowed.

Accepted x64 observations meet these eleven fixed checks. All ten packets
reported physical-redb equality false while exact logical preservation passed;
this is not evidence of unsafe identity-row mutation or byte-identical files.
The PG sample independently preserves57 nontransition rows on upgrade and68 on
downgrade, issuer/authentication/all16 rates, ordinary user/group/grant equality,
auditor200/admin403, exact permitted admission changes derived from BEFORE, and
other-connected-client refusal. It checks unchanged expires_at after a0.2s
sample and logout401; this is nonrenewal sampling, not waiting a full TTL,
testing the entire60-second lease, or a paused-IO/concurrency fence.

Both native archive checks additionally exercise shared/Platform route
boundaries, Essentials agent exclusions and Platform state downgrade refusal.
Native base-client archive smoke only checked its version; its ordinary remote
commands are exercised by the accepted container fixture. None is universal
client/protocol/browser/tenant evidence or the full installed recovery suite.

## Required ARM terminal disposition checks

Root's later sanitized receipt must establish all of the following before the
conditional original-row recommendation is applicable:

- Exact repository/run/attempt/controller/helper/workflow provenance; selected
  product9a/tree1528/reviewb71 and native artifact11232871527, with verified
  transport/source/archive/binary identities, not a0.1.1 label substitution.
- Actual native Linux/aarch64/ARM64 runner and daemon/OCI/ELF183 observations;
  both editions built, saved, strictly validated, removed/loaded with matching
  image IDs, source labels, public license/binary/capability parity and reader
  build/inputs. No emulated x64 or prebuilt-x64 branch as ARM evidence.
- All eleven fixed application checks reached and passed, including full
  E→P→E common-state comparisons, precise config/downgrade refusals and complete
  logical preservation. Empty checks or a failed/refused fixture remains open
  even after successful builds/archives/UID setup. Fresh reader/packet evidence
  must substantiate the assertions; do not prescribe x64's2700 step count as an
  ARM oracle.
- Numeric successful terminal/controller outcome with complete bounded captures
  and resource observations, all owned CLI children consumed/joined and groups
  absent, no cleanup errors, no remaining owned containers/volumes, builder
  children removed and owned image inventory empty. CLI process groups and
  daemon resource ownership are separate proofs; no global prune is cleanup.
- Preserve `official_release:false` and `shared_full_gate:not_run` as written.
  No full shared gate, HTTPS/passkey/USB device, registry publication, installed
  recovery, signature, current-a6 build or whole supported-baseline claim is
  inferred from a future pass. Actual unsupported engine/UID/setup failure stays
  a failure and requires a separately reserved diagnosis, not bypass/rerun here.

These conditions consume the already reserved work. I propose no new workflow,
test, command, hardware matrix or runtime reservation in this report. If its
terminal result fails or cleanup is unverified, the one smallest remaining
facet is that exact failed ARM cohort boundary, to be named from actual evidence
by root before any source change. There is no justification yet to invent a
second defect or assign a cause to unseen logs.

## Receipt bodies and historical failure preservation

All links refer to immutable objects read at the fixed publication, or byte-
equal bodies from the preceding audit; no present remote state was queried.

| Root receipt | SHA-256 / attribution |
| --- | --- |
| [Native x64](evidence/wave30-a09-native-x86-root-review.json) | `ccf7c3fb6c0bd85816097c32a24a1c1b0a67b9597346d571cd68975a4680b1af`; archive slice passed,full shared gate not_run. |
| [Native ARM](evidence/wave30-a09-native-arm64-37016520583.json) | `2cbf8ee46dbdd8b2ab43ad933913dd0a16c20b3183497c2d3b9cf3d1287da227`; archive slice passed,full shared gate not_run. |
| [Native ARM PG](https://github.com/Rhein-Industries/riAuth/blob/a6d361600a03713fc1b687f367e9db84efe43463/docs/roadmap/evidence/wave30-a09-native-pg-37082572962-root-review.json) | `750b4c95b2598c3cb53a8a1b477638c7ac43aab70762efb786797a0a3089189d`; sample passed,full_shared_gate not_certified/release_gate_result false; cleanup pidfile_verified false remains exact despite separately observed absence/removal. |
| [x64 encrypted-redb/container terminal](https://github.com/Rhein-Industries/riAuth/blob/a6d361600a03713fc1b687f367e9db84efe43463/docs/roadmap/evidence/wave30-a09-linux-bind-replay-actual-root-review.json) | `fa9de597e3a5b74e20a317097c9d0622db5c15880c94a6d802f87595345db1c8`; passed,2700 joined steps/zero remaining groups-containers-volumes/no cleanup errors/empty image inventory. |

The old DockerDesktop UID0 postcondition failures remain failed. The retained
archive42 pass tested a validator, not an application. Earlier archive,
physical-hash, unknown private-tuple/OSError and bind98 failures are not rewritten
by the later x64/PG passes. b619's stale source notices remain historically
retained and explicitly limited; they are not silently regenerated or certified
as current release inventories. Neither cleanup success nor source review gives
retrospective application credit to an unreached fixture.

## Checks actually performed and handoff limits

Read-only Git object/tree/hash comparisons and static Python AST parsing passed
for the inspected helper bodies. Entire production differences between 9a/b619
and b619/a6 were inspected at the named paths; all claimed identical files were
compared as full bytes. Current CONTRIBUTING.md was read; repository/ancestor
AGENTS.md searches found no applicable file. Its broad runtime recommendations
are superseded by this explicit static-only reservation.

Read-discovery failures were corrected without importing files: the guessed
container receipt name did not exist, and guessed `src/grants.rs`,
`src/api/agents.rs`, `src/management/users.rs` paths stopped early identity
scripts. Tree/function discovery located the actual receipt and shared modules;
no nonexistent path was counted as a successful read. A later optional
`src/management/user.rs` existence probe likewise supplied no body/equality
claim. Oversized historical-report output was treated as partial, as recorded
above. None was a product/check failure or runtime observation. An initial
multi-hunk report correction failed an exact text anchor before applying any
change; the bounded report edits were then applied against the actual text.
The first docs check exited1 on the two later root receipts absent from this
unaligned worktree. Those two links now use the exact published Git objects;
no receipt was imported and no source alignment was performed.

Final UTF-8/newline/whitespace/fence/five-link and fixed-object checks passed.
The corrected docs checker exited0; tracked-file hygiene exited0 (1,055 files).
The entire prior Q08/Q11/S02 reports matched their HEAD bytes and recorded
hashes, with zero tracked diff before this sole new report was staged. The
no-index whitespace command printed no violations and exited1 because the new
file differs from /dev/null; staged whitespace/scope and postcommit cleanliness
are verified separately in the handoff. No Cargo/compiler/native/helper,
Docker/archive extraction, product/service/provider/browser/desktop/network,
remote query/download, other-worker contact or source alignment occurred.
No validation/Cargo/desktop slot was acquired or released. No new task/worktree/
shell, board mutation, main edit, merge or push occurred. RiWork Cua.ai Driver
remains the sole desktop provider if a future authorized scope needs one.
All accepted receipt-secret/header/PAM/permission/review/removal/audit/credential/
Group and60-second contracts and already closed original rows are preserved.
