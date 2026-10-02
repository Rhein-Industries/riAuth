# Wave30 A09 independent launcher source review

2026-10-02. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, original A09
task `506e3979-a590-4af3-8fa8-ee90d3a517f2`. This independent review uses
existing WT `a1303b57-4a34-487e-9c63-a841f05b51a0`, branch
`roadmap/local-workflow-safety-wave27`, starting clean at
`2414b5d83bc1ebbe31ffc99684ef7e4896e9d1d1`. Primary A09 WTf2 and its
assignment are unchanged. The sole reserved write is this new report.

**Disposition: hold the exact proposed launcher for two narrow source
corrections.** Its whole-text identity checks pass, but the artifact run-head
guard uses the product export pin, and an invalid validator input can enter
the public failure report unchanged. Neither finding is a freshly observed
launcher/API/native failure. Root owns correction reservations, workflow
materialization, dispatch and any runtime acceptance.

## Immutable inputs and depth of review

Read `47c28516df7508d4839a0705d55cd284d5c6a30a`:
`docs/roadmap/local-wave30-a09-artifact-plan.md`, Phase I from line 1307,
including the complete embedded workflow at lines 1360–2638. The original
row and shared distribution gate were read from that immutable report:
produce and test supported server/client/container/maintenance artifacts,
including Linux x86-64 and ARM64, while distribution switches preserve
users, authorization and configuration behavior. This review does not
assert a new live-board observation or completion of that original gate.

| Reviewed complete text | Actual bytes | Recomputed SHA-256 |
| --- | ---: | --- |
| Proposed workflow, including final newline | 73,279 | `0bad3f22ac0c1534e64f12b5454efd686bac8bb260919f7d16753f968bb518b4` |
| Inline controller, including final newline | 59,371 | `8018cd2119e9763aa1832664d03ca28732739f2c4749d2b2a2cfcdb266e5a6b4` |
| Embedded bootstrap | 1,506 | `1adc47c2b81f78c75f1f2122aabff7f08f48786fafd840880f3106418326e4de` |
| PostgreSQL helper at `d36e13ad17541d21c88ed90d842e0a3e6db2280d`, all 370 lines | 21,716 | `575dfb049324ede3cb0ddf4bd71e0b42ef46704a015f6a576a38c4052a1ff2c1` |
| Encrypted helper at product `9a819317efb3a13fa27cd86f884be2be00898fc0`, all 398 lines | 22,045 | `09e137d88eb8e873a488448b7bdfdbe80d2327fbca716a93f445eaaae1ea9e8e` |

Full bodies were read, beyond parsing/hashing: every controller function and
method, nested capture drain and signal handler, bootstrap constructor and
context exits, all mode/exception branches, and the helper bodies listed
above. Controller source ranges: utility/ownership/process/transport bodies
229–468; bootstrap 471–501; `Controller` methods 504–1121; initialization,
mode routing and entry guard 1124–1203. Workflow shell/action/always-upload
steps were also read with their original indentation.

The full fixed matrix helper (367 lines) and installed-gate helper (421
lines) at product `9a81931` were read to trace imported subprocess, loopback
HTTP, cluster, serving and CLI behavior. Their SHA-256 values match
`f07d934f9cfd7af20eb086f5838863c28f840ee848e62bea1d865b330643d887` and
`cbe8dcb42b4b1c36dc8df00204103b9c955feb8057275a441f60f3072d2bf8b5`.
The entire SPDX module was parsed and hashed, and its import-time statements
and guarded entry point were read; its packaging functions are outside the
reachable sample. Hash:
`ca063ab3d4abb6ec815151bcf762447e96682076a7f72d2dbfa9d7e6d3a1032c`.
No inspected module was imported or executed. In-memory AST parsing is an
identity/structure check, not evidence that the launcher works.

## Blocker 1: artifact run-head guard binds the wrong source role

Proposal line 1860, controller line 462, requires:

```python
run.get("head_sha") == FIXED["product_sha"]
```

The build's originating run and its checked-out product export have distinct
identities. The complete accepted root receipt at
`a74d3225dd8c845d0c7dff43ab922749c3a957b5` was read:
`docs/roadmap/evidence/wave30-a09-native-arm64-37016520583.json`, SHA-256
`2cbf8ee46dbdd8b2ab43ad933913dd0a16c20b3183497c2d3b9cf3d1287da227`.
It binds workflow `036a392656b4b5070cc86a11d5ca3258b7b868d2` and product
`9a819317efb3a13fa27cd86f884be2be00898fc0` separately.

The already-downloaded `/tmp/riauth-wave30-a09-37016520583/evidence.json`
was independently read and rehashed: 60,299 bytes, SHA-256
`fd8280125b38e6afbe13970be4617df8fdbe72e7280a4e6dedb2f2a84e4c9ab1`.
It records real `GITHUB_SHA` **and** `GITHUB_WORKFLOW_SHA` as `036a392…`,
event `workflow_dispatch`, ref `refs/heads/main`, run `37016520583`, attempt
1; its verified `source_sha` is separately `9a81931…`. The old build
workflow at `036a392` records those GitHub values before its checkout and
checks out `${{ inputs.source_sha }}` at workflow line 306. Checkout does
not change the originating run's commit identity.

`workflow_run.head_sha` is a run-head binding; using the export SHA here
expects the two distinct roles to be equal. The source-derived expected
result is `artifact_metadata_mismatch` before ZIP transport for metadata
identifying the measured build event at `036a392…`. No artifact API response
was fetched/read here, so an actual API refusal is not claimed.

**Smallest proposed owned correction:** only the `transfer` run-head
comparison and, if desired, one explicit fixed `build_run_head_sha` constant
set to the measured `036a392…`. Bind it to that actual event SHA; retain
product `9a81931…` in the extracted build-evidence/source-tree checks. For
this exact recorded run, `FIXED["build_workflow_sha"]` equals the measured
event SHA, but an explicit run-head name avoids conflating those roles for
future cohorts. Keep artifact ID/name/run/expiration/size/digest and all
product/archive/binary/input checks unchanged. Root can confirm retained
metadata before reserving that hunk; no new API request is proposed for this
review lane.

## Blocker 2: invalid validator input is serialized before validation

Proposal line 1929, controller line 531, places raw
`os.environ["A09_VALIDATOR_SOURCE_SHA"]` in `self.data`. The full-lowercase-
SHA refusal is later, in `Controller.run` at proposal lines 2457–2458
(controller 1059–1060). Its ordinary failure path still enters cleanup and
`save`, which writes that pre-existing field to public `controller.json`.
The two `always()` steps finalize and upload that fixed filename even when
preflight failed.

Source-only counterexample: supply the harmless marker `INVALID_SHA_MARKER`.
Constructor retains it; preflight sets `validator_full_sha_required`; final
save still exposes that same marker. Substituting an accidentally entered
credential would follow the same public-output path. This trace was read
from the bodies; the controller was not executed with any input, and no
real secret was used or disclosed.

**Smallest proposed owned correction:** at the one public-data initializer,
serialize a validator SHA only after the exact 40-lowercase-hex check;
otherwise store `null` or a fixed invalid-input status. Preserve validation
against the original input and its existing refusal code before checkout or
transport. Do not echo the invalid value, its prefix or its raw exception.
No helper, credential, transport, authority or API change is required.

## Remaining full-body findings, subject to those corrections

| Boundary | Source finding and exact limit |
| --- | --- |
| Provenance and cohort | Fixed repository, product/tree, build workflow/run/job, artifact ID `11232871527`, name `riauth-local-arm64-37016520583-1` and full ZIP SHA `fc2a83dd286b70e455802af7713b60724d97b9e598240f6d9037c279601e4d30` match the accepted receipt. Validator checkout and all five helper/import hashes are checked separately. The run-head mismatch above remains blocking. |
| Transport | One metadata attempt and one ZIP attempt; 120 s each, 15 s socket operations, 2 MiB metadata/256 MiB ZIP, private part files, no retries or proxy use. At most one HTTPS storage redirect from the fixed GitHub endpoint; Authorization is removed before following it, and another redirect refuses. Full ZIP EOF/content length/digest/API-size checks precede extraction. No new transfer was performed. |
| ZIP/tar/native installation | Exactly 18 named ZIP regular-file entries, no duplicates/extras/directory/encrypted/link/NUL-truncated/traversal entry, exact sizes and hashes, manual exclusive extraction. Every tar has exactly the binary/LICENSE/notices regular non-sparse members with exact size/hash. All five binaries pass ELF64 little-endian AArch64 source guards before execution. No archive was opened or extracted here. |
| Bound identities | Constants for 18 member identities, five product tuples, eleven log records, seven inputs and prior resource aggregates were compared in memory with the accepted receipt. Compressed archives total 48,748,203 bytes, binaries 125,824,744 bytes, five document pairs 3,768,635 bytes. These comparisons are not new installed-binary or outer-ZIP measurements. |
| Build/package exclusion | Reachable launcher modes only retrieve, check, install and invoke the pinned helper. Matrix `main`/dependency builds, encrypted `run`/format-2 metadata/probe, installed release `main`/packaging and SPDX commands are not called. No Cargo, package installation, Docker, new archive or PG fallback is introduced. |
| PostgreSQL prerequisite | Only already-installed `/usr/lib/postgresql/16/bin/{initdb,pg_ctl,createdb,psql,postgres}`; regular native ELF, root-owned/non-writable/executable, measured hashes, five bounded 10 s version commands and matching sanitized PG16 versions. Actual installation/dynamic-library availability remains an unmeasured host prerequisite; failure does not authorize setup or substitution. |
| Resource/budget | Native hosted Linux/aarch64/unprivileged guards; 30 GiB in every initial scope, 10 GiB stop and separate 8 GiB floor, two-second continuous monitor, six-second gap/heartbeat refusal, device IDs and minima without adding same-device free space. Supervisor 1,080 s lifetime and 60 s cleanup reserve; helper cannot start without 600 s plus reserve. These sampled safeguards do not promise a floor between samples or under stalled/forced host loss. |
| Retention adapter | Exact temporary-directory call shape is replaced by one fresh marked retained fixture. Adapter context exit never deletes it; other shapes refuse. The shared tempfile module is patched only inside the isolated bootstrap process; imported but unreachable historical temporary-directory paths do not run. This is an explicit retention behavior change, not identical helper cleanup behavior. |
| Owned shutdown/reaping | Verified Linux subreaper precedes helper children. UID/start-tick fingerprints and descendant ancestry govern signals; live PG PID-file/cluster/freshness/native-executable/cmdline proof is checked. Missing or failed proof retains private fixture; owned descendants still receive bounded TERM/KILL and actual waitpid reaping. A new scan and capture-reader checks precede deletion. No generic pg_ctl/killall/unrelated PID action is present. Runtime ownership and cleanup remain unexercised. |
| Delete ordering | Pinned helper tries immediate `pg_ctl` shutdown in its finally block. The adapter retains the fixture even if that stop raises or helper is killed; supervisor cleanup stops/reaps owned descendants before deleting proven-owned scratch. Any proof/signal/reaping/capture/survivor failure prevents deletion and fails the controller. Cleanup failure remains authoritative even if the sample had passed. |
| Captures and upload | Native-command stdout/stderr are separately drained to finite private captures; children do not inherit retrieval/proxy/Python-path/PG credential variables. Database/WAL are not constrained by a capture file-size rlimit. Upload has only four explicit sanitized public files, no raw JSON/config/database/session/URL/token/capture/archive paths. Fixed helper key/type/value/hash validation limits public results; invalid validator input is the exception described above. |
| Failure evidence | First phase failure is retained; cleanup failures are recorded separately. Public not-run placeholders exist before supervisor preflight. Mode entry catches raw exceptions without console text. Finalize validates public-file ownership/size and supervisor exit; always-upload is attempted after failures. Forced job/VM loss can prevent finalization/upload; no guaranteed cleanup receipt is inferred then. |
| Shared sample | Format 3/all 16 effective rates, exact non-transition-row preservation through E→P→E, stable ordinary user/group/grants, audit allow/users deny, logout refusal and fixed expiry, authentication/general-rate refusal and connected-client refusal remain authored checks. The encrypted helper's only invoked function is `live_identity_and_grant`; its format-2 runner/probe is excluded. This is not every authorization/configuration path. |

All eleven `${{ ... }}` expressions on ten lines were inspected at their actual workflow keys:
job env uses `inputs`; step env uses `github.token`; checkout `with` uses
`inputs`; upload `with` uses `github` and `env`; two step conditions use
`always()`. `RUNNER_TEMP` is read in shell, exported locally and persisted to
`GITHUB_ENV`, avoiding the earlier job-env `runner.temp` defect. This
cross-check uses the official context/environment-file references already
pinned in the proposal at lines 885–899. No new documentation fetch or GitHub
semantic/dispatch acceptance is claimed.

## Actual checks and unexecuted limits

Performed only source/read-only checks: clean/head/guidance inspection;
immutable Git-body reads; complete workflow/controller/bootstrap SHA and
length reconstruction; in-memory AST parsing without imports; full-body
traces; accepted-receipt/constant/import-hash comparisons; existing JSON
hash/provenance read. An initial read-only `rg` included an absent local
`planning` directory and exited 2; subsequent evidence came from the exact
tracked Git receipt and known existing downloaded JSON. Two oversized reads
were truncated; relevant omitted source ranges were reread in bounded chunks.
Neither event executed or modified proposed code.

Only this new report was written. Docs/whitespace, new-file-only scope and
unchanged tracked-source checks accompany its separate commit. No merge,
reset, launcher/YAML/controller/bootstrap/helper import or execution, Cargo,
native/service/PG/subreaper runtime, network/provider query, download,
package setup, new archive, deletion, worker contact, task/status/main/push
action occurred. No existing report or source was changed.

Historical five LOCAL ARM64 archives and smoke success remain credited only
to their accepted source/run. Root did not rehash the historical outer ZIP;
this review adds no ZIP/host/PG/cleanup/E→P→E execution evidence. The exact
proposal is not cleared for runtime until the two narrow findings are
resolved and independently reviewed. Linux x86-64, containers, TLS,
passkey/device/full client, encrypted/redb, universal shared gate,
physical/tenant/escrow/paused-IO and release acceptance remain separate.
Root owns A09 integration/status and later dispatch. I10/R05/W02/W05 remain
DONE.
