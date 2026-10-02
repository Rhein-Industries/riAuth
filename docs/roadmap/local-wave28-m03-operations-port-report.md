# M03 protected operations, backup and SSF port

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`

Task: `3b9fbfa6-716a-4ce8-842a-2c953bab3d0f` (M03)

Worktree: `e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`

Branch: `roadmap/local-extension-isolation-wave27`

Report date: 2026-10-02

The protected connector-independent operation port is ready for root's review
and integration. It includes the corrected F1/F2/F3 unit, backup/SSF protections,
and backend-proof definitions. It does not establish whole-task completion.
Only this assigned worktree and branch were changed; root owns integration,
push and board reconciliation. W07 implementation remains untouched.

## Original acceptance and pinned inputs

The original task record was reread before this work. Its acceptance is:

> GUI, CLI, and API must share authorization, validation, transactions,
> idempotency, and audit behavior.

Its completion gate is:

> The same change has the same permission checks and outcome regardless of
> which interface submits it.

This port adds clients of the accepted server writers; it does not narrow those
gates to compilation, synthetic fixtures or source reports. The task record is
still `in_progress`; this worker made no status change or task-done claim.

| Input/result | Exact commit |
| --- | --- |
| Reviewed pushed main used as product base | `c540ef43b0a6974acd3aa8a5c6328048dd6c70f0` |
| Own-branch alignment, preserving history | `a495a31c885c61dbfc0411a59845268c6dfc1b0e` |
| Accepted ten-row review/checklist | `3ce13b7ef0f915a842308934dd3c7bd2705e99f9` |
| Immutable corrected cumulative port source | `a162347cf0b5d55d2957066fff26ca2145f6197e` |
| Reviewed backend-proof source | `4b9851aa2ad896c83a194f31fd0649e130a479eb` |
| Finished protected code port | `e212630a8e1c8acf3756870236f90892091b4924` |

The exact boundaries and earlier static findings are in the
[operations review](local-wave28-m03-operations-review.md#bounded-source-to-port-checklist).
Source provenance includes `bbbfd638c9e1d90d110792cab64cef08facc3dad`, operation-only
`ee2b2ab3d74c0b523d5f281d1bfa19b99ed1737b`, `7de863506ae25be56b7d42b464d6a62f02c99dba`,
`9471b9aa3b14c703b95f9a4848016c3cd091d05d`, `e2e5e8c64fc072c545ff8a8e4f1609f5aa3c8b4c`,
`174215399559bd77aeb16fc95f26a52ca5b81d7e` (F1),
`07a754dfc9a19a2680a1b2896ce7abf1ae5b09b2` (F2), and the cumulative source above (F3).
These are provenance, not a sequence of unprotected commits to cherry-pick.
All implementation reads used immutable Git objects. The mutable management
source tree was neither read for implementation nor edited; its workers were
not contacted. No management source branch was merged.

Alignment merged reviewed main into this own branch. The only conflict was
add/add in `docs/roadmap/local-wave28-m03-operations-review.md`. Main's report
blob `54bedfaa7efc310b70025fcbcad33b82cdc695c1` was verified equal to the earlier
`1210f6b50e3c18453ff4886d011ab7efd63dd785` report. The later corrected own
`3ce13b7` report was retained, preserving its accepted corrections and checklist.
There were no production-file conflicts. Immediately after alignment, all
production files equaled reviewed main; only the corrected report differed.

## Ten-row port disposition

These are the ten approved checklist rows, applied against the reviewed product
base. New modules use the cumulative source; shared files receive only the
approved hunks or projections preserving their accepted equivalents.

| Row | Destination | Ported boundary and preserved behavior |
| --- | --- | --- |
| 1 | `crates/riauthctl/src/{access,directory,offboard,plans,provision}.rs` | Exact corrected cumulative modules. Temporary access, offboarding, configured-directory and SCIM operations, confirmation, bounded plan handling and typed recovery input; F1 and all F3 callers included. |
| 2 | `crates/riauthctl/src/management.rs` | Only apply-removal confirmation and F3 plan-output changes. Exact cumulative projection retaining accepted export and excluding the connector-status helper and export changes. |
| 3 | `crates/riauthctl/src/transport.rs` | Confirmation headers, optional revision/plan helpers, streaming client and `open_stream`; cumulative blob equals the reviewed narrow hunk result. Existing primary issuer/session binding, no redirects/proxies and safe bounded errors retained. |
| 4 | `crates/riauthctl/src/{archive,backup,ssf}.rs` | Corrected cumulative framing/quotas, envelope/config checks, private no-replace publication, backup UTF-8 refusal, INT/TERM handling and SSF guards/delete allowlist. Backup differs only in comments scoping cooperative notification and cleanup. |
| 5 | `crates/riauthctl/src/main.rs` | Operation enums/routing, removal confirmation, pre-Remote plan-bound flag refusal and backup size parser (32 bytes through 4 GiB). Exact source projection removes only workflow module, enum and dispatch sections. Existing command/emitter behavior retained. |
| 6 | `crates/riauthctl/{Cargo.toml,Cargo.lock}` | Required direct `aws-lc-rs`/`base64`, Tokio `signal`, and associated lock edges/registry record only. Accepted graph and `unsafe_code = "forbid"` retained. |
| 7 | `crates/riauthctl/tests/m03_parity_ops.rs` | Exact source prefix through plan-bound flag refusal, including retained-job, F1 and F3 definitions. Stops before the connector export fixture/tests. |
| 8 | `crates/riauthctl/tests/m03_parity_ssf_backup.rs` | Exact cumulative corrected mock/signal/privacy/archive definitions, including mixed-case/nested SSF deletion and default-HUP/nohup definitions. Compiled, not run. |
| 9 | `tests/{m03_pam_e2e,m03_state_removal_e2e,m03_backup_e2e}.rs` | Exact standalone real-binary PAM/removal/backup definitions from cumulative source; ignored setup/evidence boundaries retained. Not compiled or run here. |
| 10 | `crates/riauthctl/README.md`, `docs/capability-matrix.md`, `docs/operations.md` | Narrow operation/confirmation/backup/SSF/F3 documentation only. Accepted O06 and other-lane text preserved; no workflow/client-connector claims added. Signal-publication, cleanup, OS permissions and duplicate-key promises scoped below. |

Additional explicitly authorized backend-proof scope:
`tests/m03_riauthctl_backends_postgres.rs` is the exact reviewed `4b9851a` blob.
`scripts/test-postgres.sh` adds only its explanatory comment and the target
`m03_riauthctl_backends_postgres` to the allowlist and `test-support` feature
case. Both previous lists are retained, including
`connector_definitions_postgres`. No PostgreSQL service was started.

## Protected behavior and limits

F1/F2/F3 are included together in the one protected code commit:

- F1 offboarding timezone grammar matches the accepted server: one to three
  components, each 1–64 ASCII bytes from `[A-Za-z0-9_+-]`, maximum 194 bytes
  including separators.
- F2 SSF delete refuses a recursive case-insensitive known header field,
  validates `deleted: true` and the requested `stream_id`, then emits only those
  two reconstructed fields. The accepted server uses `stream_id`; no server
  disclosure was observed in the earlier reviews.
- F3 desired-state/LDAP/Workspace/Entra/SCIM plan output checks UTF-8 before
  network/output activity; the shared save helper also checks, and summaries
  use a validated string and fallible serialization. Other output paths owned
  by another lane were not changed or duplicated.

The accepted server production tree is unchanged. Authorization, transactions,
receipt replay, validation and audit remain in its shared writers. Existing
client receipt handling and optional-header/PAM contracts are preserved. PAM
fallback is enabled by the local optional-revision flag and HTTP 403; it does
not require a particular error-code string. Other revision failures remain
errors, and the server independently authorizes mutations. This port does not
resolve broader browser/CLI header-policy differences or change the existing
reviewed client-creation receipt exception.

SSF response protection is operation-specific. The known header field is
matched case-insensitively, and secret guards cover exact keys/values plus
substrings in values for secrets of at least eight bytes. A short secret
coinciding with an ordinary key can cause conservative refusal after a committed
mutation. Arbitrary encodings or transformations are not guaranteed to be
recognized. Fixed refusal messages do not print response bodies or credentials;
delete output is allowlisted. Zeroization is best effort, not a proof of erasing
all copies in the allocator, crypto/network libraries or process memory.

Backup retains bounded streaming/framing/authentication, no overwrite via
hard-link publication, early UTF-8 output refusal and private partial creation.
Connect/read timeouts are not a whole-transfer deadline. Duplicate-key refusal
covers typed envelopes and recursively parsed configuration, not every nested
key inside arbitrary record values. Client `verified` describes archive framing
and transcript authentication; matching server configuration/schema and actual
offline restore remain separate acceptance checks.

INT/TERM handlers are registered before partial creation. The last cooperative
poll checks for a **visible pending notification before the hard link**. Delivery
may lag or a signal may arrive after that poll; publication and synchronous
cleanup have no atomic signal-publication guarantee. Partial cleanup is best
effort, including ignored unlink errors. No SIGHUP listener is installed, keeping
inherited `nohup` HUP ignore intact; ordinary default-HUP termination may leave
a private partial. The changed code comments and product docs say this explicitly.
Unix mode-0600 behavior is described as such; Windows ACL equivalence and host
signal behavior were not established. The non-Unix Ctrl-C fallback is source
behavior, not a newly verified Windows result. No Linux/Windows runtime result
is claimed from this macOS client compile.

## Exact scope and source equivalence

The code commit changes exactly 23 files (7,775 insertions, 47 deletions):

- `crates/riauthctl/Cargo.lock`
- `crates/riauthctl/Cargo.toml`
- `crates/riauthctl/README.md`
- `crates/riauthctl/src/access.rs`
- `crates/riauthctl/src/archive.rs`
- `crates/riauthctl/src/backup.rs`
- `crates/riauthctl/src/directory.rs`
- `crates/riauthctl/src/main.rs`
- `crates/riauthctl/src/management.rs`
- `crates/riauthctl/src/offboard.rs`
- `crates/riauthctl/src/plans.rs`
- `crates/riauthctl/src/provision.rs`
- `crates/riauthctl/src/ssf.rs`
- `crates/riauthctl/src/transport.rs`
- `crates/riauthctl/tests/m03_parity_ops.rs`
- `crates/riauthctl/tests/m03_parity_ssf_backup.rs`
- `docs/capability-matrix.md`
- `docs/operations.md`
- `scripts/test-postgres.sh`
- `tests/m03_backup_e2e.rs`
- `tests/m03_pam_e2e.rs`
- `tests/m03_riauthctl_backends_postgres.rs`
- `tests/m03_state_removal_e2e.rs`

The bounded comparison script checked 19 full blobs or declared projections
against the fixed source commits, checked the accepted locked graph and both
PostgreSQL target lists, and refused any file outside the approved 23-file scope.
Its result was:

> PASS: 19 pinned blob/projection comparisons; accepted locked graph preserved;
> both PostgreSQL target lists additive; 23 approved files only; required
> exclusions absent

The source commit is `a162347cf0b5d55d2957066fff26ca2145f6197e` for every row below
except the backend-proof row, which uses `4b9851aa2ad896c83a194f31fd0649e130a479eb`.
The hashes are Git blob IDs, not inferred equivalence from a worker report.

| File | Source blob | Ported blob | Comparison |
| --- | --- | --- | --- |
| `crates/riauthctl/src/access.rs` | `dff62891ade5172361942b7cadf4b1269b7b25f8` | `dff62891ade5172361942b7cadf4b1269b7b25f8` | exact full blob |
| `crates/riauthctl/src/archive.rs` | `e6eb4ea48bf707ec756c518ce0c1c6db25cbf5a3` | `e6eb4ea48bf707ec756c518ce0c1c6db25cbf5a3` | exact full blob |
| `crates/riauthctl/src/directory.rs` | `a14f26c7eeabf106f7bc42b447f820f7149c6c12` | `a14f26c7eeabf106f7bc42b447f820f7149c6c12` | exact full blob |
| `crates/riauthctl/src/offboard.rs` | `788b501446169b66c352571da20fc9118b2452a8` | `788b501446169b66c352571da20fc9118b2452a8` | exact full blob |
| `crates/riauthctl/src/plans.rs` | `804aa8d8e34a79144391e03ca29d97887181ca26` | `804aa8d8e34a79144391e03ca29d97887181ca26` | exact full blob |
| `crates/riauthctl/src/provision.rs` | `b6d216f324b25d4ef2106dbb3dea3e1c6341ff99` | `b6d216f324b25d4ef2106dbb3dea3e1c6341ff99` | exact full blob |
| `crates/riauthctl/src/ssf.rs` | `0625753ba8b19ee69e9ed80e74a2d7fc0b157ba1` | `0625753ba8b19ee69e9ed80e74a2d7fc0b157ba1` | exact full blob |
| `crates/riauthctl/src/transport.rs` | `dea115cb8e23bd54911cdb66d50b6efa7e9c940b` | `dea115cb8e23bd54911cdb66d50b6efa7e9c940b` | exact full blob |
| `crates/riauthctl/Cargo.toml` | `e97ff28b79770f55e2c1dabb2de9479705d789f0` | `e97ff28b79770f55e2c1dabb2de9479705d789f0` | exact full blob |
| `crates/riauthctl/Cargo.lock` | `a471c5c447c51c8d82002d7f31060e8c32099886` | `a471c5c447c51c8d82002d7f31060e8c32099886` | exact full blob |
| `crates/riauthctl/tests/m03_parity_ssf_backup.rs` | `a827e7f038bd155e9d2101b68f06f5d1ba17ad60` | `a827e7f038bd155e9d2101b68f06f5d1ba17ad60` | exact full blob |
| `tests/m03_pam_e2e.rs` | `4b92f17101f6a22ac5d9950efe76b27dbd0c1e48` | `4b92f17101f6a22ac5d9950efe76b27dbd0c1e48` | exact full blob |
| `tests/m03_state_removal_e2e.rs` | `4c1fd6ef08303ea9149da2e5fe21b065f442d8cb` | `4c1fd6ef08303ea9149da2e5fe21b065f442d8cb` | exact full blob |
| `tests/m03_backup_e2e.rs` | `da4b5359e644a4b781c580329d6e75fdd1d8666a` | `da4b5359e644a4b781c580329d6e75fdd1d8666a` | exact full blob |
| `tests/m03_riauthctl_backends_postgres.rs` | `06cf6e21db5c5b97e17cacbda26afa4f72d4c79b` | `06cf6e21db5c5b97e17cacbda26afa4f72d4c79b` | exact full blob |
| `crates/riauthctl/src/backup.rs` | `985b39142cf6f9fa53017951a925cefbc92e2b54` | `88190f31659b39eba8bd1b9867b57631390d22df` | identical non-comment lines; cooperative cleanup/publication comments scoped |
| `crates/riauthctl/src/main.rs` | `d91107cf667e9a2ff60b261e7edc41270367eb52` | `7a2d74c6763667c079b64d141b55d187623b7d05` | exact projection excluding three workflow wiring sections |
| `crates/riauthctl/src/management.rs` | `b67b5884af81ac81e2a8fe7f8d7ae379ab67fa36` | `14568801d7532b672355ebc173ffde4b26aee16a` | exact projection retaining accepted export and excluding connector-status helper |
| `crates/riauthctl/tests/m03_parity_ops.rs` | `b88f06f415420cd8c06256346944d9c694c1960d` | `a787f5d679d58a937413ae2d45ab943129a0377e` | exact prefix ending before connector export fixture/tests |

The exact source projections are reproducible:

1. `main.rs`: remove `mod workflow;`, the Workflow command enum block, and its
   dispatch arm; retain every other source byte.
2. `management.rs`: replace the region from `pub(crate) async fn export(` to
   `pub(crate) async fn apply(` with that exact accepted-main region. This
   removes the source connector-status helper/export changes and retains the
   accepted export implementation; all other source bytes match.
3. `m03_parity_ops.rs`: keep the exact prefix before
   `/// A server whose export reports the given connector status, if any.`,
   trimming the trailing separator newline. The port has 1,804 lines; the
   excluded `export_server` begins at line 1,807 in cumulative source.
4. `backup.rs`: all non-comment lines match source; only contract comments are
   narrowed for cooperative notification, cleanup and private-permission scope.

The other four scoped files (three product documents and the PostgreSQL script)
were reviewed as narrow deltas against accepted main rather than copied from
cumulative source. `docs/operations.md` has only the command-list update and
scoped backup paragraph; accepted O06 storage/background/admission/rate-limit
content is retained. The earlier accepted export documentation is retained in
the client README; it does not gain held connector behavior.

Manifest changes are direct `aws-lc-rs` version `1.18.1` (default features off,
`aws-lc-sys`), `base64 = "0.23"`, and Tokio `signal`. The lock delta is the
`signal-hook-registry 1.4.8` record, two riauthctl dependency edges and the Tokio
registry edge. Every other accepted package record, version, source, checksum
and dependency is unchanged. Root `Cargo.lock` is untouched. Existing notices
already list these versions; no notice regeneration was needed.

Required exclusions verified absent from this code commit: all server/root
production files, W07/guest-child code, SAML executor/assembly, activation/version
files, workflow client module/enum/dispatch/API/mock/e2e, and client connector
status/export helper/tests. The held M07 slice
`32f9aed565ccf1c3f5dd242aa5df561d38185122` and workflow/client-connector parts of
`ee2b2ab` were not ported. No CI configuration, accepted credential implementation,
main/accepted worktree, task board or source branch was edited. The only branch
merge was the reviewed-main alignment into this own branch.

## Verification actually performed

All commands were run in the assigned worktree. No test case was executed.

| Check | Result and scope |
| --- | --- |
| Immutable source/projection/graph/script/scope comparison | Passed the 19 comparisons and approved 23-file boundary described above. |
| Changed-file `rustfmt` check | Exit 0 for the 17 changed Rust files listed below; no formatting mutation. |
| `python3 scripts/check-docs.py` | Exit 0: Markdown links and build-directory layout checked. Repeated only after this new report was added. |
| `bash -n scripts/test-postgres.sh` | Exit 0; syntax only, no service or test run. |
| `git diff --check` and staged diff check | Exit 0; product scope and separate report-only staged scope checked. |
| One necessary client compile | Exit 0 in 1m10s; binary and the two changed client mock-test targets, no default features, locked graph. |

Changed-file formatting command:

```sh
rustfmt --edition 2024 --config skip_children=true --check \
  crates/riauthctl/src/access.rs crates/riauthctl/src/archive.rs \
  crates/riauthctl/src/backup.rs crates/riauthctl/src/directory.rs \
  crates/riauthctl/src/main.rs crates/riauthctl/src/management.rs \
  crates/riauthctl/src/offboard.rs crates/riauthctl/src/plans.rs \
  crates/riauthctl/src/provision.rs crates/riauthctl/src/ssf.rs \
  crates/riauthctl/src/transport.rs \
  crates/riauthctl/tests/m03_parity_ops.rs \
  crates/riauthctl/tests/m03_parity_ssf_backup.rs \
  tests/m03_backup_e2e.rs tests/m03_pam_e2e.rs \
  tests/m03_state_removal_e2e.rs tests/m03_riauthctl_backends_postgres.rs
```

The compile was the only build/check of product targets, needed for newly wired
modules, headers and signal/dependency entries:

```sh
CARGO_TARGET_DIR=/Users/dominik/orca/projects/riAuth-public-preview-local-extension-isolation-wave27/target/wave28-m03-operations-port \
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  cargo check --manifest-path crates/riauthctl/Cargo.toml \
  --no-default-features --locked --bin riauthctl \
  --test m03_parity_ops --test m03_parity_ssf_backup
```

The target is private to this worktree. Free disk was approximately
35 GiB initially and 32 GiB after the compile, above the 8-GiB stop
threshold throughout the check. No accepted target was used. No real-binary test, root/server test
compile, PG service, full CI, benchmark/conformance campaign, accessibility work,
Grok, real cloud mutation, desktop interaction or external message occurred.
No new task, worktree, RiWork shell or review agent was created.

## Evidence boundaries and remaining acceptance

The earlier static review records source-reported historical mock and real
backup/restore results, with their exact source snapshot and correction limits.
They were not rerun or independently log-verified in this port. The two newly ported
client mock targets, including corrective and signal cases, compile;
this supplies type/wiring evidence, not runtime assertions or proof that every
OS signal is observed before publication. The four root real-binary/backend
proof files are definitions only in this port. No synthetic tenant, native
source-built binary or worker report is presented as an official released
artifact or external acceptance.

Recommend accepting the protected operation code port and its scoped product
documentation for root integration. No additional blocker was found within the
approved corrected source boundaries. This does not recommend closing M03 from
this slice: whole GUI/CLI/API outcome/permission parity, broader interface/header
policy gaps, held workflow replay/live activation/sealing, held connector client
behavior, independent runtime review where required, applicable official release
artifacts and external/deployment gates remain outside this implementation's
evidence. Root and the owning lanes retain those dependencies; no board action
or external completion claim is requested from this worker.

This report is a separate documentation commit after the protected code commit.
Its own exact commit ID is supplied in the explicit-project root handoff rather
than embedded self-referentially here.
