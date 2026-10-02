# A09 native helper failure: independent source diagnosis

2026-10-02; reservation `wave30_A09_native_helper_failure_source_diagnosis`.
Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; existing worktree
`ed9ac424-59f4-4520-905b-919aea3521eb`, own branch
`roadmap/local-revisions-coordination-wave27`, starting clean at
`073b4fa35319e506faf8e2c7016aed453426c553`. This new report is the only write.
I02's root-owned DONE disposition is preserved; no assignment/status change.

No certain early setup/import/fixture defect justifies a source correction.
The focused bootstrap/helper subprocess actually ran and failed. Its inner
failure location, type and output remain **UNKNOWN**. One prospective bounded
observer is proposed below; no source ownership, execution or rerun is claimed.

## Immutable inputs and actual public evidence

The four fixed public documents are under
`/tmp/riauth-wave30-a09-shared-37043196924/riauth-local-shared-arm64-37043196924-1`;
`job.log` is in their parent directory. All four documents were read/parsed;
the entire public job log was decoded and hashed. No private stderr was present
or sought. Local downloaded-document modes were0600.

| Public file | Bytes | SHA256 |
| --- | ---: | --- |
| `controller.json` | 13,487 | `0d5603a712881c3d35e957797b4c89beae06b0047d0977e2fa26195eed817a8f` |
| `helper-redacted.json` | 70 | `ea7793e91dfdf9fc5f1d08bacad8429368cd69cd2131927da2ccf74b330c8f08` |
| `resources.jsonl` | 1,755 | `2c9bb4c59fc148b65db40604dc89f09a5267c11db019359bb4778f4bdafe8f28` |
| `cleanup.json` | 294 | `d3b6f9bc7107fa540408dbf195a74b00919943f8555481969ea6ccf4fdbe35e3` |
| `job.log` | 239,508 | `5a71cf88d168378c819816aed087da10ee633ed6319e0a2641931833c97a0778` |

Three distinct immutable roles were resolved and checked:

- Workflow `b619fe25269ccc150e473bbcde47cdb3623ef810`,
  `.github/workflows/check-local-shared-handoff.yml`, blob
  `a47187df6d2374310471138b07a930c481877655`, 73,380 bytes/1,279 lines, SHA256
  `0de9c7be9b380bd32c748a19699739e8ed7dacb2313d1a366bb000f701e15037`.
- Corrected validator checkout `0a243c9afd9c18d38192145e58485357a2f30b93`.
- Product `9a819317efb3a13fa27cd86f884be2be00898fc0`, from build workflow
  `036a392656b4b5070cc86a11d5ca3258b7b868d2`, build run37016520583/
  job110868629053, input artifact11232871527. No current-product equivalence
  beyond this fixed identity is inferred.

The public controller reports full ZIP transport49,177,062 bytes, SHA256
`fc2a83dd286b70e455802af7713b60724d97b9e598240f6d9037c279601e4d30`.
Its18 exact validated input members and five product records equal the complete
workflow FIXED allowlists. This is verification of the uploaded records against
source, not a new local download/extraction or five new native executions.
Validator-head, artifact-metadata and artifact-zip phases each exited0.
All five installed PG tool version phases (`initdb`, `pg_ctl`, `createdb`,
`psql`, `postgres`) exited0; every recorded version is16.15, with binary and
version-output hashes present. Version probes do not prove a cluster workload.

Actual focused phase: `focused-shared-helper`, exit1, statusfailed,
elapsed3.205067s. First failure is exactly `phase_exit` for that phase/exit1;
controller exit1, sample_result `not_run`, full_shared_gate `not_certified`,
official_release false. Its private stderr capture was1,148 bytes, SHA256
`bb8d5727bf57759c8fc0876b081d5ab41ade7a33859a54734e79213440b8915d`;
stdout was0 bytes. Those hashes cannot recover exception text or a caller.
`helper-redacted.json` is the initialization placeholder
`riauth.local-build-free-pending/v1`/`not_run`, not an assertion that the helper
was never invoked. The failed child phase precedes reading/publishing helper.json.

Cleanup records failures[], remaining_owned_processes0, reaped1, owned fixture
and private scratch removed, postgres_state `no_live_owned_postgres`,
postgres_pidfile_verified false. The public job reports both run and finalize
exit1; finalize returns the preserved run result, not a newly demonstrated
cleanup failure. Four sanitized files were uploaded as artifact11243930343,
16,120 bytes. Six resource records independently recompute minimum free
115,575,992,320 bytes and maximum actual gap2.0006397049999975s; the8GiB floor
was respected by these observations. No fresh process/PID probe ran here.

Root's prior equal-SHA failure remains the earlier root invocation error.
It is not retrospectively reclassified as this unlocated inner helper failure.
Neither outcome is turned into a pass or a new runtime attribution.

## Full source inspection and bounded findings

The entire pinned workflow/controller/bootstrap and all five helper/import files
were read in bounded spans. Initially combined output and the large echoed
job log truncated; required source spans were reread. The full1452-line public
log's controller heredoc was normalized and byte-compared to the entire pinned
extracted controller:59,472 bytes/1,203 lines, SHA256
`b84868ddce285e3c2979318b778bbb4d522ca0d212a1310c59d25c12bbd8f9e3`,
also equal to the public controller digest. The bootstrap literal is1,506 bytes/
30 lines, SHA256
`1adc47c2b81f78c75f1f2122aabff7f08f48786fafd840880f3106418326e4de`.
Actual public run/finalize errors and the four-file upload body were read
separately from source echo. Source identity and AST parsing are not execution.

| At corrected0a243, under `scripts/` | Full bytes/lines | SHA256 matching workflow and public import records |
| --- | ---: | --- |
| `check-local-edition-transition-postgres.py` | 21,716/370 | `575dfb049324ede3cb0ddf4bd71e0b42ef46704a015f6a576a38c4052a1ff2c1` |
| `check-exact-edition-matrix.py` | 18,394/367 | `f07d934f9cfd7af20eb086f5838863c28f840ee848e62bea1d865b330643d887` |
| `check-installed-release-gate.py` | 22,321/421 | `cbe8dcb42b4b1c36dc8df00204103b9c955feb8057275a441f60f3072d2bf8b5` |
| `check-local-encrypted-edition-transition.py` | 22,045/398 | `09e137d88eb8e873a488448b7bdfdbe80d2327fbca716a93f445eaaae1ea9e8e` |
| `spdx_sbom.py` | 36,727/899 | `ca063ab3d4abb6ec815151bcf762447e96682076a7f72d2dbfa9d7e6d3a1032c` |

Concrete source trace, using immutable workflow line numbers:

- Source740–746 validates all five files before the child. Helper26–36 loads
  matrix/gate/encrypted by explicit sibling paths; gate25–33 loads SPDX;
  encrypted25–33 repeats the same explicit imports. Imported module main
  guards do not execute Cargo, release gates or encrypted probes. SPDX's
  import-time constant assertion is internally consistent. No missing relative
  import or deterministic import-time subprocess/fixture call was found.
- Bootstrap510–540 checks the helper digest, retains only the exact
  `local-a08-postgres-` temporary-directory request under owned private scratch,
  writes its ownership marker, maps the existing helper arguments, then calls
  runpy. Helper234 supplies that exact prefix and evidence.parent directory.
  No import changes or unexpected nested TemporaryDirectory call exist on the
  focused helper's called path. The retained context's False exit preserves
  exceptions; it does not swallow a failed oracle.
- Given this source's sole fixture creator and successful fixture-proof cleanup
  at968–982/1059–1061, owned_fixture_removed=true supports that retained fixture
  allocation/marker creation was reached. Thus a pre-main import or pre-fixture
  argument mismatch is not a defensible cause of this observed run. This
  ordering inference does not identify any later failing HTTP/native/SQL step.
- Matrix181–193 initializes/starts disposable PostgreSQL; helper236 onwards
  uses that cluster, then native maintenance init, readiness, live identity/
  authorization and explicit transitions. PG version probes and final absence
  of a pidfile do not reveal which of these operations ran or failed: helper's
  finally364–366 may stop a previously live cluster before controller cleanup.
  Reaped1 does not count completed native children or establish their success.
- Controller917–928 calls exactly the bootstrap/helper child; command659–728
  records a nonzero child as phase_exit and preserves first_failure. Public
  helper publication929–966 is reachable only after child success. Cleanup
  1049–1071 hashes private captures and removes owned scratch after settlement.
  This explains the missing diagnostic while preserving the actual failure.

No changed oracle, fixture assumption or product patch is justified by these
facts. A missing inner diagnosis is a concrete evidence gap, not a demonstrated
native/product/PG defect. Root owns joining the source worker's actual appendix;
no other worker was contacted and none is awaited by this review.

## One prospective failure-only projection; report-only exact diff

Candidate ONLY for a future separately approved reservation in
`.github/workflows/check-local-shared-handoff.yml` at b619: wrap existing runpy
and project an optional private diagnostic into existing controller.json after
the child command refuses. All five pinned helper/import files stay unchanged.
No implementation or new invocation is authorized or performed by this report.

The projection compares full trusted source filenames internally, emits only
one of six fixed basenames plus integer line1..4096, and a fixed exception-class
literal (unknown types become Other). It examines at most64 traceback links,
keeps at most8 trusted frames, and exclusively writes at most2KiB mode0600 in
owned private scratch. No exception text, raw traceback, arguments, SQL, private
path, token, protocol value, source line or locals are formatted or exported.
Only a failing escaping exception can write this observation; successful
SystemExit0/None is excluded. An observation error is swallowed and the original
exception is re-raised. The controller strictly validates keys/types/classes/
basenames/bounds, best-effort copies it only after refusal, then re-raises the
same Refusal. Diagnostic absence or malformed data cannot turn failure into
success, change first_failure or skip cleanup.

Four fixed upload names and their existing schema strings remain unchanged;
only the controller v1 gains an optional failure field. Helper placeholder,
all success oracles, private capture8MiB cap, existing file bounds,600s helper/
1080s controller budgets,30/10/8GiB capacity gates, ownership/reaping/removal
contracts and all original invocation arguments are retained. The private
diagnostic is removed by existing cleanup. This records the terminal exception
escaping runpy; it does not inspect exception chains or promise the first inner
cause if an existing helper finally block replaces an earlier exception. A
process kill or failure before runpy can leave the field absent.

```diff
--- a/.github/workflows/check-local-shared-handoff.yml
+++ b/.github/workflows/check-local-shared-handoff.yml
@@ -538,3 +538,42 @@
                       "--source-revision", sys.argv[4]]
-          runpy.run_path(str(helper), run_name="__main__")
+          def failure_source(error):
+              names = ("check-local-edition-transition-postgres.py", "check-exact-edition-matrix.py",
+                       "check-installed-release-gate.py", "check-local-encrypted-edition-transition.py",
+                       "spdx_sbom.py")
+              trusted = {str(root / "bootstrap.py"): "bootstrap.py",
+                         str(helper): "check-local-edition-transition-postgres.py"}
+              trusted.update({str(helper.resolve().parent / name): name for name in names})
+              classes = {AssertionError: "AssertionError", RuntimeError: "RuntimeError",
+                         ValueError: "ValueError", TypeError: "TypeError", KeyError: "KeyError",
+                         IndexError: "IndexError", NameError: "NameError", ImportError: "ImportError",
+                         ModuleNotFoundError: "ModuleNotFoundError", OSError: "OSError",
+                         FileNotFoundError: "FileNotFoundError", PermissionError: "PermissionError",
+                         SyntaxError: "SyntaxError", SystemExit: "SystemExit",
+                         KeyboardInterrupt: "KeyboardInterrupt"}
+              frames, examined, tb = [], 0, error.__traceback__
+              while tb is not None and examined < 64:
+                  name = trusted.get(tb.tb_frame.f_code.co_filename)
+                  line = tb.tb_lineno
+                  if name is not None and type(line) is int and 1 <= line <= 4096:
+                      frames.append({"file": name, "line": line})
+                      frames = frames[-8:]
+                  tb, examined = tb.tb_next, examined + 1
+              value = {"exception_class": classes.get(type(error), "Other"), "frames": frames}
+              payload = (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode("ascii")
+              if len(payload) <= 2048:
+                  fd = os.open(private / "helper-failure-source.json",
+                               os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
+                  with os.fdopen(fd, "wb") as out:
+                      out.write(payload)
+          try:
+              runpy.run_path(str(helper), run_name="__main__")
+          except BaseException as error:
+              try:
+                  if type(error) is not SystemExit or not (
+                      error.code is None or type(error.code) in (int, bool) and error.code == 0
+                  ):
+                      failure_source(error)
+              except BaseException:
+                  pass  # Observation must not replace the original failure or exit.
+              raise
           '''
@@ -925,5 +964,25 @@
                   helper = pathlib.Path(os.environ["GITHUB_WORKSPACE"]) / "scripts" / HELPER
-                  self.command("focused-shared-helper",
-                               [sys.executable, "-I", "-B", str(bootstrap), str(helper), str(installed),
-                                FIXED["imports"][HELPER], FIXED["product_sha"]], 600, env)
+                  try:
+                      self.command("focused-shared-helper",
+                                   [sys.executable, "-I", "-B", str(bootstrap), str(helper), str(installed),
+                                    FIXED["imports"][HELPER], FIXED["product_sha"]], 600, env)
+                  except Refusal:
+                      try:
+                          value = read_json(PRIVATE / "helper-failure-source.json", 2048)
+                          classes = {"AssertionError", "RuntimeError", "ValueError", "TypeError", "KeyError",
+                                     "IndexError", "NameError", "ImportError", "ModuleNotFoundError", "OSError",
+                                     "FileNotFoundError", "PermissionError", "SyntaxError", "SystemExit",
+                                     "KeyboardInterrupt", "Other"}
+                          require(type(value) is dict and set(value) == {"exception_class", "frames"} and
+                                  type(value["exception_class"]) is str and value["exception_class"] in classes and
+                                  type(value["frames"]) is list and len(value["frames"]) <= 8 and
+                                  all(type(frame) is dict and set(frame) == {"file", "line"} and
+                                      type(frame["file"]) is str and frame["file"] in set(FIXED["imports"]) | {"bootstrap.py"} and
+                                      type(frame["line"]) is int and 1 <= frame["line"] <= 4096
+                                      for frame in value["frames"]), "helper_failure_source_shape")
+                          self.data["helper_failure_source"] = value
+                          self.save()
+                      except Exception:
+                          pass  # Missing or malformed diagnostics preserve phase_exit and cleanup.
+                      raise
                   report = read_json(PRIVATE / "helper.json", 64 * 1024)
```

Static in-memory evidence ONLY: original and candidate controller/bootstrap
AST parse; all five full-file import AST/hash checks pass. Exact reversal of
the two replacement anchors returns every byte of the original workflow.
Attribute-free AST comparison changes only BOOTSTRAP and Controller.helper;
all other top-level objects/controller methods remain identical. Proposed
bootstrap length69 lines; prospective workflow SHA256
`45304da275d3072c2c0de9f6debed1acc335ffa99e0ed6baa15064549bab5574`, controller SHA256
`ffa6d792bcf9170018f6bcd2bbd58d0920444dd2720c5570bdd9de9fadfc0e32`, bootstrap SHA256
`27361ddd98813b9e539758ca26a433ea86a1b085b0fd68036a39678ed464f7e2`.
These identify an unimplemented candidate, not runtime artifacts or a pass.
During source review, the prospective SystemExit success check was tightened
from equality alone to None or an integer/boolean zero, so a non-integer code
cannot hide a failed exit. This corrected only the in-memory/report proposal;
no candidate code was executed. The initial report-diff read omitted its last
context line's newline and failed an equality assertion; retaining that newline
corrected the static inspection wrapper, without changing any pinned source.
The first final-whitespace assertion also caught two blank diff context lines
carrying a trailing space. Regenerating the same exact candidate diff with one
context line removed that report formatting issue; candidate hashes are unchanged.

## Checks and disposition

Actual local checks: repository/ancestor guidance and clean own branch read;
fixed Git object/hash/source-body inspection; full public JSON/JSONL parsing
and log/controller comparison; phase/count/resource/cleanup consistency;
baseline `python3 scripts/check-docs.py` exited0 with
“Markdown links and build-directory layout checked”, no pre-existing flags;
source/candidate AST and exact in-memory reversal as above. Final report link/
LF/fence/whitespace/scope checks accompany its sole-path commit.

No helper/controller/bootstrap import or execution, native/library/compiler/
Cargo/service/provider/HTTP/browser/desktop runtime, network query/download/
dispatch, source alignment/merge/reset/main/push/status, worker contact or new
worker/task/worktree/managed shell occurred. Only static standard-library
inspection and the documentation checker ran. No checker/cache cleanup.

Recommendation: retain this run as helper phase_exit1 with unknown inner cause;
consider only the bounded observer above for root's separate source reservation.
No correction/rerun/pass or original A09 closure is inferred. I02 DONE and all
previously accepted product/security contracts remain untouched.


Final preparation evidence: all six distinct cited Git objects resolved;
sole-new-report scope, LF/fences/trailing whitespace, exact two-hunk candidate
application/reversal and unchanged-other-body AST checks passed. The candidate
has63 added/four removed lines. Documentation checking with the report present
and Git whitespace checks exited0. Two final append attempts used a mistyped
context and were rejected without changing bytes; the exact read tail was then
used. These were report-edit/static-tool issues, not native runtime failures.

## wave30_A09_failure_projection_memory_validation

2026-10-02. Root authorized an append here and one bounded pure stdlib-memory
validation of the extracted observer/predicate, with an explicit stop on the
first unexpected failure and no retry/correction. Outcome: **preparation failed;
memory validation UNRUN, zero cases executed**. No candidate defect or security
pass is inferred from this host extraction error.

The starting branch was clean at
`6ed52573c3b3f0db34583040ce7022807eebcc1e`. The complete prior report equalled that
commit. Before the extraction error, the script reconstructed its exact candidate
from the embedded two-hunk diff against b619 and checked all three fixed hashes:
workflow `45304da275d3072c2c0de9f6debed1acc335ffa99e0ed6baa15064549bab5574`,
controller `ffa6d792bcf9170018f6bcd2bbd58d0920444dd2720c5570bdd9de9fadfc0e32`,
bootstrap `27361ddd98813b9e539758ca26a433ea86a1b085b0fd68036a39678ed464f7e2`.
These exact candidate bytes and the entire prior report remain unchanged.

The static extraction script selected failure_source, the existing runpy
exception handler, the diagnostic classes/shape predicate, the existing read_json
size guard and the FIXED import allowlist as AST/literals. The complete candidate
controller/bootstrap ASTs parsed, and the isolated function definition parsed.
It then tried ast.parse on ast.unparse of the ExceptHandler by itself:
`except BaseException as error:`. An except clause requires its enclosing try;
the host preparation therefore raised SyntaxError before emitting the extracted
bundle or executing any candidate function, branch or predicate. This is an
extraction-wrapper error; the existing full candidate's grammar did not fail.

Actual preparation command exit1; tool-recorded process wall time0.137632042s.
This is not a measured validation-body duration or a successfully enforced
validation deadline. No private temporary workspace or projection output was
created, and no synthetic exception/frame/security case executed. Consequently
there was no owned fixture to clean. The no-retry instruction was honored:
no repair, re-extraction, candidate invocation or alternative validation followed.

The fixed result was retained in session memory and hashed before report
comparisons. Its canonical sorted compact ASCII JSON with a final LF is
726 bytes, SHA256
`31298828f042e38046bb11ed31fc1a5639bf73bdd121ccef563d2e238f46eeb2`:

```json
{"candidate_bootstrap_sha256":"27361ddd98813b9e539758ca26a433ea86a1b085b0fd68036a39678ed464f7e2","candidate_controller_sha256":"ffa6d792bcf9170018f6bcd2bbd58d0920444dd2720c5570bdd9de9fadfc0e32","candidate_function_executed":false,"candidate_workflow_sha256":"45304da275d3072c2c0de9f6debed1acc335ffa99e0ed6baa15064549bab5574","cleanup":"not_needed_no_fixture","controller_predicate_executed":false,"exact_branch_executed":false,"executed_cases":0,"owned_fixture_created":false,"passed_cases":0,"process_elapsed_seconds":0.137632042,"process_exit_code":1,"projection_output_created":false,"schema":"riauth.a09-projection-memory-validation/v1","status":"preparation_failed_before_validation","unexpected_preparation_failures":1}
```

All requested behavioral cases remain unexecuted: trusted basename/line and
untrusted-frame filtering,64-link/last8 caps, fixed-class/unknownOther privacy,
sentinel text/private-path/locals exclusion, exact SystemExit0/None/failure
handling, exclusive duplicate-file refusal, write-failure/original-exception
preservation, and controller exact-key/type/bool/int/string/size/class/frame
rejections. The earlier static design review does not substitute for them.

The smallest remaining preparation seam, if root releases a fresh bounded
attempt, is to carry the already selected handler AST/body under a controlled
synthetic try/except rather than parsing a standalone except clause. That would
change the validation harness only, with no candidate edit. No correction or
fresh invocation is performed or requested as an action by this append.

Original37043196924 focused-helper phase_exit1 remains UNKNOWN internally;
the earlier equal-SHA root input failure remains separately classified. This
new host preparation failure changes neither historical runtime result. I02
DONE and all other closed rows remain preserved; no task/status action occurred.

Only this report append is written. No real helper/controller/bootstrap main,
import/runpy/source script, native/library/compiler/Cargo, PG/service/provider/
HTTP/network/download/dispatch/browser/desktop or other worker was invoked.
Host Python standard-library AST/literal/hash inspection and immutable Git reads
are preparation, not execution of the candidate or surrounding source.
Root retains source implementation/invocation ownership in the primary lane.

The full original report prefix is19,193 bytes, SHA256
`06f769cd4894d3a73190eeb8af3629725577219997487ffe672fba48e6cc9843`.
Append-only byte preservation, receipt canonical/hash correspondence, repository
documentation/whitespace/scope and clean-commit checks complete this slice;
they do not restart the failed preparation or establish behavioral validation.

Final append checks passed: whole prior-prefix/candidate preservation, canonical
726-byte receipt/hash, zero-case/no-fixture correspondence, append-only sole-file
scope with zero deletions, LF/fences/trailing whitespace, documentation checker
exit0 and Git whitespace exit0. Staged report-only/whitespace and clean own branch
checks accompany the separate append commit.

## wave30_A09_corrected_projection_memory_validation

2026-10-02. Root separately released one corrected <=30s pure stdlib-memory
child after reviewing22715c. Starting clean own branch
`22715c3f9fcc3c8a06b271f677cf6178eb1f7a16`; same project/worktree/report reservation.
**Actual result:84 planned,84 executed,84 passed; child exit0.** One child only,
no retry, unexpected failure, post-execution harness fix or candidate correction.
All previous bytes, including FAILED preparation0cases, remain historical.

The materialized observer at
`565b2afc874d5527f16df0c03ad36dc9c0ccc8c5` was read through immutable Git objects.
The whole workflow equals the reviewed45304da candidate: SHA256
`45304da275d3072c2c0de9f6debed1acc335ffa99e0ed6baa15064549bab5574`;
its extracted controller SHA256
`ffa6d792bcf9170018f6bcd2bbd58d0920444dd2720c5570bdd9de9fadfc0e32`;
bootstrap SHA256
`27361ddd98813b9e539758ca26a433ea86a1b085b0fd68036a39678ed464f7e2`.
No source copy, edit or alignment occurred.

Preparation corrected only the extraction mechanism: select the ExceptHandler
from the complete parsed bootstrap, deepcopy it beneath a synthetic ast.Try
whose controlled body raises the supplied synthetic exception. The real runpy
body was never copied/executed. The exact failure_source FunctionDef, handler,
controller shape expression/classes assignment and existing read_json size
expression were copied into the host harness. The entire resulting harness was
unparsed, reparsed and compiled in memory before any execution. Attribute-free
AST dumps of all five selected objects equal the immutable source; source-line
positions were intentionally relocated in this host script. This is structural
identity, not execution of surrounding controller/bootstrap/helper code.

| Exact selected source AST; SHA256 of ast.dump(include_attributes=False) | Hash |
| --- | --- |
| `failure_source` | `ba9f70af426868a9f31ca57295c2e6dc82335ab58f2c1b5a9a71f6a4419d81d5` |
| `handler` | `832ad7408b2ea88ce3f64d6ce8ec1e7a2b049817daeb5d47befec2662dc563c4` |
| `shape_predicate` | `54132d0cffb1f29daec0aeb20a86c8ae97b738f3a27ff7488d0d56cc35a0c565` |
| `size_predicate` | `41d79137b63623714ac89dd7f3203c7a1560a69ebac397b91da43eb34f42a1ce` |
| `classes` | `6bed3558dc317f0c6029ee2b025e3b94bc89fec3f010aaf2d6aa946cc0a80402` |

Full prepared/executed stdin harness:21,293 UTF-8 bytes including final LF, SHA256
`cb0546e69199b312fcbed63bc66996ccd9f8700b8234c863c35c443ac7279743`.
All nine imports are host stdlib; the only additional import is fixed builtins
lookup for the fifteen known exception constructors. Static checks exclude
runpy/subprocess/socket/urllib/importlib references or import-from in the child.
Controlled synthetic frame text and canonical JSON final LF were statically
parsed/checked before the one child; no preparation error in this release.

The parent executed exactly `[sys.executable, '-I', '-B', '-']` with this stdin,
capture pipes,30s communicate timeout and one private TemporaryDirectory under
this worktree. Only two fixed synthetic-child environment keys were supplied;
no inherited credentials/tokens/proxy/environment dump. Child alarm27s reserves
cleanup time. Parent owned/reaped the one child, removed its own0700 temporary
fixture, then sealed exit/times/full stdout+stderr hashes BEFORE parsing the child
result or checking expectations. Inside each projected-output case, bounded
payload/hash/mode receipts were retained before expectations; ordinary files
were unlinked first, with the two duplicate cases retaining only their original
file until the attempted second write/read comparison completed.

Actual child elapsed0.035503s; child-recorded body0.008769s; parent wrapper
including fixture0.035873s. Timed_out false, reaped true, owned_fixture_removed
true. These durations document this bound, not throughput or general performance.
Stdout9,958 bytes SHA256
`deac393e9146649730d955614d69845271ddfd2bf8fb6609c106fb5146abfd46`;
stderr0 bytes SHA256
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
The full fixed child result is9,299 canonical sorted compact ASCII JSON bytes
including LF, SHA256
`4c12e51867b11722e9e469ae68f3dee352c6531cb2eadd1ffca523b9c75baf26`.
The parent receipt below is537 canonical bytes including LF, SHA256
`0fe2d2b675d0719724c1d57542c86d1c2e0a9586c5d91c87c1c2dd06178b31d6`.

## Finite security cases and practical limits of this invocation

| Cases, all passed in this one child | Evidence/meaning |
| --- | --- |
| P01–P11:11 | Exact trusted filename/line inclusion; a different private path with the same basename excluded; mixed filtering; line0/4097 excluded and4096 included;12 links retain last8;80 links inspect only first64 and retain their lines357–364; all six fixed basenames; opaque unknown exception/locals/privacy; duplicate direct and exact-handler no-overwrite; injected write failure preserves the original exception object. |
| L01–L15:15 | Each fixed built-in exception constructor projects its literal approved class. No exception text/repr is formatted. Other is separately exercised by P08 and D06. |
| S01–S08:8 | Exact copied handler re-raises the original SystemExit object. None/int0/boolFalse emit no file; int1/boolTrue/negative/string/float0.0 produce only the fixed SystemExit class/frames. This tests the branch, not interpreter-native exit-code conversion. |
| D01–D10:10 | Valid/empty/eight-frame/line1/line4096/Other acceptance, exact2048-byte read acceptance,2049 refusal, malformed JSON and empty-byte refusal in the memory adapter. |
| N01–N40:40 | Exact root/frame key sets, missing/extra keys, wrong root/class/frame container/file/line types, bool rejection for integer lines, unknown/path-valued basename, nine-frame cap, zero/negative/4097/large bounds; exact shape predicate and bounded memory JSON read both refuse. |

The exact source size expression executes with a host regular(path) adapter
returning len(memory_bytes); the complete real read_json/regular/reader and
controller command/cleanup methods never execute. Memory JSON decoding is a host
adapter that returns rejection on errors, matching the observer's ignored-invalid
diagnostic outcome. Only the extracted shape expression/classes and size
comparison are source executions. No real fixture/source import, PG/native
subprocess, protocol response, real exception trace or actual bootstrap occurs.

Controlled compile filenames create synthetic frames only; frame locals and
exception arguments contain synthetic secret/private-path sentinels. Unknown
OpaqueError str/repr deliberately raise if called. Every emitted payload is
checked for sentinel bytes, temporary-root/private-path bytes and local-variable
names: NONE serialized. No raw exception/frame/args/locals/text/path or sentinel
content is printed. The32 output-capture receipts below (including duplicate
reads) all record mode0600, regular single-link owned files,49..464 bytes,
within2KiB. Exclusive-file collision does not overwrite the original bytes/hash;
an injected PermissionError in the observer is swallowed by the exact handler
while the original exception remains the one re-raised. No symlink race, real
peer, native exception/exit or controller end-to-end invocation is credited.

## Exact full harness retained; source surrounding it was never executed

```python
import hashlib
import json
import os
import pathlib
import signal
import stat
import sys
import time
import types
FIXED = {'imports': {'check-exact-edition-matrix.py': 'f07d934f9cfd7af20eb086f5838863c28f840ee848e62bea1d865b330643d887', 'check-installed-release-gate.py': 'cbe8dcb42b4b1c36dc8df00204103b9c955feb8057275a441f60f3072d2bf8b5', 'check-local-edition-transition-postgres.py': '575dfb049324ede3cb0ddf4bd71e0b42ef46704a015f6a576a38c4052a1ff2c1', 'check-local-encrypted-edition-transition.py': '09e137d88eb8e873a488448b7bdfdbe80d2327fbca716a93f445eaaae1ea9e8e', 'spdx_sbom.py': 'ca063ab3d4abb6ec815151bcf762447e96682076a7f72d2dbfa9d7e6d3a1032c'}}
CASE_IDS = ['P01_trusted', 'P02_untrusted', 'P03_mixed', 'P04_line_bounds', 'P05_last8', 'P06_first64', 'P07_six_basenames', 'P08_unknown_privacy', 'P09_duplicate_direct', 'P10_duplicate_branch', 'P11_write_failure', 'L01_AssertionError', 'L02_RuntimeError', 'L03_ValueError', 'L04_TypeError', 'L05_KeyError', 'L06_IndexError', 'L07_NameError', 'L08_ImportError', 'L09_ModuleNotFoundError', 'L10_OSError', 'L11_FileNotFoundError', 'L12_PermissionError', 'L13_SyntaxError', 'L14_SystemExit', 'L15_KeyboardInterrupt', 'S01_none', 'S02_zero', 'S03_false', 'S04_one', 'S05_true', 'S06_negative', 'S07_string', 'S08_float_zero', 'D01_valid', 'D02_empty_frames', 'D03_eight_frames', 'D04_line_one', 'D05_line_4096', 'D06_other', 'D07_size2048', 'D08_size2049', 'D09_malformed_json', 'D10_empty_bytes', 'N01_extra_root_key', 'N02_missing_class', 'N03_missing_frames', 'N04_root_list', 'N05_root_null', 'N06_root_int', 'N07_root_string', 'N08_root_bool', 'N09_class_int', 'N10_class_bool', 'N11_class_null', 'N12_class_unknown', 'N13_class_list', 'N14_frames_dict', 'N15_frames_null', 'N16_frames_string', 'N17_frames_bool', 'N18_nine_frames', 'N19_extra_frame_key', 'N20_missing_file', 'N21_missing_line', 'N22_frame_null', 'N23_frame_list', 'N24_frame_string', 'N25_file_path', 'N26_file_unknown', 'N27_file_int', 'N28_file_bool', 'N29_file_null', 'N30_file_list', 'N31_line_true', 'N32_line_false', 'N33_line_float', 'N34_line_string', 'N35_line_null', 'N36_line_zero', 'N37_line_negative', 'N38_line_4097', 'N39_line_large', 'N40_line_list']
NEGATIVE = [('N01_extra_root_key', {'exception_class': 'AssertionError', 'frames': [{'file': 'check-local-edition-transition-postgres.py', 'line': 17}], 'extra': 1}), ('N02_missing_class', {'frames': [{'file': 'check-local-edition-transition-postgres.py', 'line': 17}]}), ('N03_missing_frames', {'exception_class': 'AssertionError'}), ('N04_root_list', []), ('N05_root_null', None), ('N06_root_int', 1), ('N07_root_string', 'invalid'), ('N08_root_bool', True), ('N09_class_int', {'exception_class': 1, 'frames': [{'file': 'check-local-edition-transition-postgres.py', 'line': 17}]}), ('N10_class_bool', {'exception_class': True, 'frames': [{'file': 'check-local-edition-transition-postgres.py', 'line': 17}]}), ('N11_class_null', {'exception_class': None, 'frames': [{'file': 'check-local-edition-transition-postgres.py', 'line': 17}]}), ('N12_class_unknown', {'exception_class': 'UnlistedError', 'frames': [{'file': 'check-local-edition-transition-postgres.py', 'line': 17}]}), ('N13_class_list', {'exception_class': [], 'frames': [{'file': 'check-local-edition-transition-postgres.py', 'line': 17}]}), ('N14_frames_dict', {'exception_class': 'AssertionError', 'frames': {}}), ('N15_frames_null', {'exception_class': 'AssertionError', 'frames': None}), ('N16_frames_string', {'exception_class': 'AssertionError', 'frames': 'invalid'}), ('N17_frames_bool', {'exception_class': 'AssertionError', 'frames': False}), ('N18_nine_frames', {'exception_class': 'AssertionError', 'frames': [{'file': 'check-local-edition-transition-postgres.py', 'line': 17}, {'file': 'check-local-edition-transition-postgres.py', 'line': 17}, {'file': 'check-local-edition-transition-postgres.py', 'line': 17}, {'file': 'check-local-edition-transition-postgres.py', 'line': 17}, {'file': 'check-local-edition-transition-postgres.py', 'line': 17}, {'file': 'check-local-edition-transition-postgres.py', 'line': 17}, {'file': 'check-local-edition-transition-postgres.py', 'line': 17}, {'file': 'check-local-edition-transition-postgres.py', 'line': 17}, {'file': 'check-local-edition-transition-postgres.py', 'line': 17}]}), ('N19_extra_frame_key', {'exception_class': 'AssertionError', 'frames': [{'file': 'check-local-edition-transition-postgres.py', 'line': 17, 'extra': 1}]}), ('N20_missing_file', {'exception_class': 'AssertionError', 'frames': [{'line': 17}]}), ('N21_missing_line', {'exception_class': 'AssertionError', 'frames': [{'file': 'check-local-edition-transition-postgres.py'}]}), ('N22_frame_null', {'exception_class': 'AssertionError', 'frames': [None]}), ('N23_frame_list', {'exception_class': 'AssertionError', 'frames': [[]]}), ('N24_frame_string', {'exception_class': 'AssertionError', 'frames': ['invalid']}), ('N25_file_path', {'exception_class': 'AssertionError', 'frames': [{'file': 'nested/check-local-edition-transition-postgres.py', 'line': 17}]}), ('N26_file_unknown', {'exception_class': 'AssertionError', 'frames': [{'file': 'untrusted.py', 'line': 17}]}), ('N27_file_int', {'exception_class': 'AssertionError', 'frames': [{'file': 1, 'line': 17}]}), ('N28_file_bool', {'exception_class': 'AssertionError', 'frames': [{'file': True, 'line': 17}]}), ('N29_file_null', {'exception_class': 'AssertionError', 'frames': [{'file': None, 'line': 17}]}), ('N30_file_list', {'exception_class': 'AssertionError', 'frames': [{'file': [], 'line': 17}]}), ('N31_line_true', {'exception_class': 'AssertionError', 'frames': [{'file': 'check-local-edition-transition-postgres.py', 'line': True}]}), ('N32_line_false', {'exception_class': 'AssertionError', 'frames': [{'file': 'check-local-edition-transition-postgres.py', 'line': False}]}), ('N33_line_float', {'exception_class': 'AssertionError', 'frames': [{'file': 'check-local-edition-transition-postgres.py', 'line': 17.0}]}), ('N34_line_string', {'exception_class': 'AssertionError', 'frames': [{'file': 'check-local-edition-transition-postgres.py', 'line': '17'}]}), ('N35_line_null', {'exception_class': 'AssertionError', 'frames': [{'file': 'check-local-edition-transition-postgres.py', 'line': None}]}), ('N36_line_zero', {'exception_class': 'AssertionError', 'frames': [{'file': 'check-local-edition-transition-postgres.py', 'line': 0}]}), ('N37_line_negative', {'exception_class': 'AssertionError', 'frames': [{'file': 'check-local-edition-transition-postgres.py', 'line': -1}]}), ('N38_line_4097', {'exception_class': 'AssertionError', 'frames': [{'file': 'check-local-edition-transition-postgres.py', 'line': 4097}]}), ('N39_line_large', {'exception_class': 'AssertionError', 'frames': [{'file': 'check-local-edition-transition-postgres.py', 'line': 1000000}]}), ('N40_line_list', {'exception_class': 'AssertionError', 'frames': [{'file': 'check-local-edition-transition-postgres.py', 'line': []}]})]
CLASS_NAMES = ['AssertionError', 'RuntimeError', 'ValueError', 'TypeError', 'KeyError', 'IndexError', 'NameError', 'ImportError', 'ModuleNotFoundError', 'OSError', 'FileNotFoundError', 'PermissionError', 'SyntaxError', 'SystemExit', 'KeyboardInterrupt']
SOURCE_AST_HASHES = {'failure_source': 'ba9f70af426868a9f31ca57295c2e6dc82335ab58f2c1b5a9a71f6a4419d81d5', 'handler': '832ad7408b2ea88ce3f64d6ce8ec1e7a2b049817daeb5d47befec2662dc563c4', 'shape_predicate': '54132d0cffb1f29daec0aeb20a86c8ae97b738f3a27ff7488d0d56cc35a0c565', 'size_predicate': '41d79137b63623714ac89dd7f3203c7a1560a69ebac397b91da43eb34f42a1ce', 'classes': '6bed3558dc317f0c6029ee2b025e3b94bc89fec3f010aaf2d6aa946cc0a80402'}

class CheckFailed(Exception):
    pass

class OpaqueError(Exception):

    def __str__(self):
        raise CheckFailed()

    def __repr__(self):
        raise CheckFailed()

def failure_source(error):
    names = ('check-local-edition-transition-postgres.py', 'check-exact-edition-matrix.py', 'check-installed-release-gate.py', 'check-local-encrypted-edition-transition.py', 'spdx_sbom.py')
    trusted = {str(root / 'bootstrap.py'): 'bootstrap.py', str(helper): 'check-local-edition-transition-postgres.py'}
    trusted.update({str(helper.resolve().parent / name): name for name in names})
    classes = {AssertionError: 'AssertionError', RuntimeError: 'RuntimeError', ValueError: 'ValueError', TypeError: 'TypeError', KeyError: 'KeyError', IndexError: 'IndexError', NameError: 'NameError', ImportError: 'ImportError', ModuleNotFoundError: 'ModuleNotFoundError', OSError: 'OSError', FileNotFoundError: 'FileNotFoundError', PermissionError: 'PermissionError', SyntaxError: 'SyntaxError', SystemExit: 'SystemExit', KeyboardInterrupt: 'KeyboardInterrupt'}
    frames, examined, tb = ([], 0, error.__traceback__)
    while tb is not None and examined < 64:
        name = trusted.get(tb.tb_frame.f_code.co_filename)
        line = tb.tb_lineno
        if name is not None and type(line) is int and (1 <= line <= 4096):
            frames.append({'file': name, 'line': line})
            frames = frames[-8:]
        tb, examined = (tb.tb_next, examined + 1)
    value = {'exception_class': classes.get(type(error), 'Other'), 'frames': frames}
    payload = (json.dumps(value, sort_keys=True, separators=(',', ':')) + '\n').encode('ascii')
    if len(payload) <= 2048:
        fd = os.open(private / 'helper-failure-source.json', os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 384)
        with os.fdopen(fd, 'wb') as out:
            out.write(payload)

def observed(error):
    try:
        raise error
    except BaseException as error:
        try:
            if type(error) is not SystemExit or not (error.code is None or (type(error.code) in (int, bool) and error.code == 0)):
                failure_source(error)
        except BaseException:
            pass
        raise

def diagnostic_shape(value):
    classes = {'AssertionError', 'RuntimeError', 'ValueError', 'TypeError', 'KeyError', 'IndexError', 'NameError', 'ImportError', 'ModuleNotFoundError', 'OSError', 'FileNotFoundError', 'PermissionError', 'SyntaxError', 'SystemExit', 'KeyboardInterrupt', 'Other'}
    return type(value) is dict and set(value) == {'exception_class', 'frames'} and (type(value['exception_class']) is str) and (value['exception_class'] in classes) and (type(value['frames']) is list) and (len(value['frames']) <= 8) and all((type(frame) is dict and set(frame) == {'file', 'line'} and (type(frame['file']) is str) and (frame['file'] in set(FIXED['imports']) | {'bootstrap.py'}) and (type(frame['line']) is int) and (1 <= frame['line'] <= 4096) for frame in value['frames']))

def diagnostic_size(path):
    maximum = 2048
    return regular(path).st_size <= maximum

def regular(path):
    return types.SimpleNamespace(st_size=len(path))

def ensure(condition):
    if not condition:
        raise CheckFailed()

def controller_accept(payload):
    if not diagnostic_size(payload):
        return False
    try:
        value = json.loads(payload.decode('ascii'))
        return diagnostic_shape(value)
    except Exception:
        return False

def canonical(value):
    return (json.dumps(value, sort_keys=True, separators=(',', ':')) + '\n').encode('ascii')

def run_validation():
    global root, private, helper, os
    started = time.monotonic()
    passed = []
    executed = []
    receipts = []
    active = 'setup'
    status = 'unexpected_failure'
    original_os = os
    root = pathlib.Path(os.environ['A09_SYNTHETIC_ROOT'])
    private = root / 'private'
    helper = root / 'sources' / 'check-local-edition-transition-postgres.py'
    output = private / 'helper-failure-source.json'
    secret = hashlib.sha256(b'synthetic-private-sentinel-only').hexdigest()
    private_sentinel = str(root / 'synthetic-sensitive-path')

    def alarm(signum, frame):
        raise CheckFailed()
    signal.signal(signal.SIGALRM, alarm)
    signal.setitimer(signal.ITIMER_REAL, 27.0)
    try:
        os.umask(63)
        private.mkdir(mode=448)
        ensure(stat.S_IMODE(root.stat().st_mode) == 448)
        ensure(stat.S_IMODE(private.stat().st_mode) == 448)
        ensure(root.stat().st_uid == os.getuid() and private.stat().st_uid == os.getuid())
        names = list(FIXED['imports'])
        frame_code = 'def synthetic_frame():\n    secret_local = sentinel\n    private_local = private_marker\n    return sys._getframe()\n'

        def make_frame(filename):
            namespace = {'sys': sys, 'sentinel': secret, 'private_marker': private_sentinel}
            exec(compile(frame_code, str(filename), 'exec'), namespace)
            return namespace['synthetic_frame']()
        trusted = {name: make_frame(helper.resolve().parent / name) for name in names}
        trusted['bootstrap.py'] = make_frame(root / 'bootstrap.py')
        untrusted = make_frame(root / 'untrusted' / names[0])

        def chain(entries):
            result = None
            for frame, line in reversed(entries):
                result = types.TracebackType(result, frame, frame.f_lasti, line)
            return result

        def error_for(entries, cls=AssertionError):
            return cls(secret).with_traceback(chain(entries))

        def capture(remove=True):
            info = output.lstat()
            with output.open('rb') as f:
                payload = f.read(2049)
            receipt = {'case_id': active, 'bytes': len(payload), 'sha256': hashlib.sha256(payload).hexdigest(), 'mode': format(stat.S_IMODE(info.st_mode), '04o')}
            receipts.append(receipt)
            if remove:
                output.unlink()
            ensure(stat.S_ISREG(info.st_mode) and info.st_nlink == 1 and (info.st_uid == os.getuid()))
            ensure(info.st_size == len(payload) and len(payload) <= 2048 and (receipt['mode'] == '0600'))
            return payload

        def privacy(payload):
            ensure(secret.encode() not in payload)
            ensure(private_sentinel.encode() not in payload and str(root).encode() not in payload)
            ensure(b'secret_local' not in payload and b'private_local' not in payload)
            ensure(controller_accept(payload))

        def emit(error, expected_class, expected_frames):
            failure_source(error)
            payload = capture()
            privacy(payload)
            ensure(json.loads(payload) == {'exception_class': expected_class, 'frames': expected_frames})

        def expected(entries):
            return [{'file': name, 'line': line} for name, line in entries]

        def preserve(error):
            try:
                observed(error)
            except BaseException as caught:
                ensure(caught is error)
            else:
                ensure(False)

        def duplicate_direct():
            failure_source(error_for([(trusted[names[0]], 17)]))
            before = capture(False)
            caught = False
            try:
                failure_source(error_for([(trusted[names[1]], 22)], OpaqueError))
            except FileExistsError:
                caught = True
            after = capture()
            ensure(caught and after == before)
            privacy(after)

        def duplicate_branch():
            failure_source(error_for([(trusted[names[0]], 17)]))
            before = capture(False)
            original = error_for([(trusted[names[1]], 22)], OpaqueError)
            preserve(original)
            after = capture()
            ensure(before == after)
            privacy(after)

        def write_failure():
            global os

            def refuse(*args, **kwargs):
                raise PermissionError()
            os = types.SimpleNamespace(open=refuse, fdopen=original_os.fdopen, O_WRONLY=original_os.O_WRONLY, O_CREAT=original_os.O_CREAT, O_EXCL=original_os.O_EXCL, O_NOFOLLOW=original_os.O_NOFOLLOW)
            try:
                preserve(error_for([(trusted[names[0]], 17)], OpaqueError))
            finally:
                os = original_os
            ensure(not output.exists())

        def system_exit(code, should_write):
            original = SystemExit(code).with_traceback(chain([(trusted[names[0]], 17)]))
            preserve(original)
            if should_write:
                payload = capture()
                privacy(payload)
                ensure(json.loads(payload) == {'exception_class': 'SystemExit', 'frames': [{'file': names[0], 'line': 17}]})
            else:
                ensure(not output.exists())

        def predicate_case(value, wanted):
            payload = canonical(value)
            ensure(diagnostic_shape(value) is wanted)
            ensure(controller_accept(payload) is wanted)

        def size_case(count, wanted):
            payload = canonical({'exception_class': 'AssertionError', 'frames': [{'file': names[0], 'line': 17}]})
            payload += b' ' * (count - len(payload))
            ensure(len(payload) == count and diagnostic_size(payload) is wanted)
            ensure(controller_accept(payload) is wanted)
        cases = [('P01_trusted', lambda: emit(error_for([(trusted[names[0]], 17)]), 'AssertionError', expected([(names[0], 17)]))), ('P02_untrusted', lambda: emit(error_for([(untrusted, 17)]), 'AssertionError', [])), ('P03_mixed', lambda: emit(error_for([(trusted[names[0]], 17), (untrusted, 99), (trusted[names[1]], 22)]), 'AssertionError', expected([(names[0], 17), (names[1], 22)]))), ('P04_line_bounds', lambda: emit(error_for([(trusted[names[0]], 0), (trusted[names[0]], 4096), (trusted[names[0]], 4097)]), 'AssertionError', expected([(names[0], 4096)]))), ('P05_last8', lambda: emit(error_for([(trusted[names[0]], n) for n in range(201, 213)]), 'AssertionError', expected([(names[0], n) for n in range(205, 213)]))), ('P06_first64', lambda: emit(error_for([(trusted[names[0]], n) for n in range(301, 381)]), 'AssertionError', expected([(names[0], n) for n in range(357, 365)]))), ('P07_six_basenames', lambda: emit(error_for([(trusted[n], 17) for n in names + ['bootstrap.py']]), 'AssertionError', expected([(n, 17) for n in names + ['bootstrap.py']]))), ('P08_unknown_privacy', lambda: emit(error_for([(trusted[names[0]], 17)], OpaqueError), 'Other', expected([(names[0], 17)]))), ('P09_duplicate_direct', duplicate_direct), ('P10_duplicate_branch', duplicate_branch), ('P11_write_failure', write_failure)]
        for i, name in enumerate(CLASS_NAMES):
            cls = getattr(__import__('builtins'), name)
            cases.append(('L%02d_%s' % (i + 1, name), lambda cls=cls, name=name: emit(error_for([(trusted[names[0]], 17)], cls), name, expected([(names[0], 17)]))))
        codes = [('S01_none', None, False), ('S02_zero', 0, False), ('S03_false', False, False), ('S04_one', 1, True), ('S05_true', True, True), ('S06_negative', -1, True), ('S07_string', secret, True), ('S08_float_zero', 0.0, True)]
        for name, code, wanted in codes:
            cases.append((name, lambda code=code, wanted=wanted: system_exit(code, wanted)))
        valid = {'exception_class': 'AssertionError', 'frames': [{'file': names[0], 'line': 17}]}
        cases += [('D01_valid', lambda: predicate_case(valid, True)), ('D02_empty_frames', lambda: predicate_case({'exception_class': 'AssertionError', 'frames': []}, True)), ('D03_eight_frames', lambda: predicate_case({'exception_class': 'AssertionError', 'frames': valid['frames'] * 8}, True)), ('D04_line_one', lambda: predicate_case({'exception_class': 'AssertionError', 'frames': [{'file': names[0], 'line': 1}]}, True)), ('D05_line_4096', lambda: predicate_case({'exception_class': 'AssertionError', 'frames': [{'file': names[0], 'line': 4096}]}, True)), ('D06_other', lambda: predicate_case({'exception_class': 'Other', 'frames': []}, True)), ('D07_size2048', lambda: size_case(2048, True)), ('D08_size2049', lambda: size_case(2049, False)), ('D09_malformed_json', lambda: ensure(controller_accept(b'{') is False)), ('D10_empty_bytes', lambda: ensure(controller_accept(b'') is False))]
        for name, value in NEGATIVE:
            cases.append((name, lambda value=value: predicate_case(value, False)))
        ensure([name for name, work in cases] == CASE_IDS)
        for name, work in cases:
            active = name
            executed.append(name)
            work()
            passed.append(name)
        active = 'cleanup'
        ensure(not output.exists())
        status = 'passed'
    except BaseException:
        status = 'unexpected_failure'
    finally:
        os = original_os
        signal.setitimer(signal.ITIMER_REAL, 0.0)
    result = {'schema': 'riauth.a09-projection-memory-validation/v2', 'status': status, 'planned_case_ids': CASE_IDS, 'planned_cases': len(CASE_IDS), 'executed_case_ids': executed, 'executed_cases': len(executed), 'passed_case_ids': passed, 'passed_cases': len(passed), 'first_failure_case': None if status == 'passed' else active, 'output_receipts': receipts, 'elapsed_seconds': round(time.monotonic() - started, 6), 'source_ast_sha256': SOURCE_AST_HASHES, 'private_fixture_mode': '0700', 'real_helpers_or_surrounding_source_executed': False}
    payload = canonical(result)
    print(json.dumps({'result': result, 'result_bytes': len(payload), 'result_sha256': hashlib.sha256(payload).hexdigest()}, sort_keys=True))
    return 0 if status == 'passed' else 1
if __name__ == '__main__':
    sys.exit(run_validation())
```

## Fixed actual parent receipt and full finite child JSON

Parent receipt, canonical form:

```json
{"child_elapsed_seconds":0.035503,"child_exit_code":0,"child_reaped":true,"harness_bytes":21293,"harness_sha256":"cb0546e69199b312fcbed63bc66996ccd9f8700b8234c863c35c443ac7279743","owned_fixture_removed":true,"schema":"riauth.a09-projection-memory-parent/v1","stderr_bytes":0,"stderr_sha256":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","stdout_bytes":9958,"stdout_sha256":"deac393e9146649730d955614d69845271ddfd2bf8fb6609c106fb5146abfd46","timed_out":false,"timeout_seconds":30,"wrapper_elapsed_seconds":0.035873}
```

Full child stdout, including its retained inner-result byte count/hash:

```json
{"result": {"elapsed_seconds": 0.008769, "executed_case_ids": ["P01_trusted", "P02_untrusted", "P03_mixed", "P04_line_bounds", "P05_last8", "P06_first64", "P07_six_basenames", "P08_unknown_privacy", "P09_duplicate_direct", "P10_duplicate_branch", "P11_write_failure", "L01_AssertionError", "L02_RuntimeError", "L03_ValueError", "L04_TypeError", "L05_KeyError", "L06_IndexError", "L07_NameError", "L08_ImportError", "L09_ModuleNotFoundError", "L10_OSError", "L11_FileNotFoundError", "L12_PermissionError", "L13_SyntaxError", "L14_SystemExit", "L15_KeyboardInterrupt", "S01_none", "S02_zero", "S03_false", "S04_one", "S05_true", "S06_negative", "S07_string", "S08_float_zero", "D01_valid", "D02_empty_frames", "D03_eight_frames", "D04_line_one", "D05_line_4096", "D06_other", "D07_size2048", "D08_size2049", "D09_malformed_json", "D10_empty_bytes", "N01_extra_root_key", "N02_missing_class", "N03_missing_frames", "N04_root_list", "N05_root_null", "N06_root_int", "N07_root_string", "N08_root_bool", "N09_class_int", "N10_class_bool", "N11_class_null", "N12_class_unknown", "N13_class_list", "N14_frames_dict", "N15_frames_null", "N16_frames_string", "N17_frames_bool", "N18_nine_frames", "N19_extra_frame_key", "N20_missing_file", "N21_missing_line", "N22_frame_null", "N23_frame_list", "N24_frame_string", "N25_file_path", "N26_file_unknown", "N27_file_int", "N28_file_bool", "N29_file_null", "N30_file_list", "N31_line_true", "N32_line_false", "N33_line_float", "N34_line_string", "N35_line_null", "N36_line_zero", "N37_line_negative", "N38_line_4097", "N39_line_large", "N40_line_list"], "executed_cases": 84, "first_failure_case": null, "output_receipts": [{"bytes": 99, "case_id": "P01_trusted", "mode": "0600", "sha256": "d32575a6e50117c31f1f5643aa1b84f7629f2ac5108ccc76aff1b35087544746"}, {"bytes": 49, "case_id": "P02_untrusted", "mode": "0600", "sha256": "789d62774088b6aee475076d97f65ffdffbcba57f6c82b4f1da1eba724e8f572"}, {"bytes": 152, "case_id": "P03_mixed", "mode": "0600", "sha256": "6fc8e20285198f2c6c81c17a94acf4f46b7a6141f6c5959c38148ba7516255f6"}, {"bytes": 101, "case_id": "P04_line_bounds", "mode": "0600", "sha256": "5a777361831cfa6fd13b11274fbc70066b0d911ef7219959e4ada38764ef065a"}, {"bytes": 464, "case_id": "P05_last8", "mode": "0600", "sha256": "6b4d9ec8d032fdfa7a22da8a92dd50f3225e6ecc2d1981b08ad7113be47985db"}, {"bytes": 464, "case_id": "P06_first64", "mode": "0600", "sha256": "5ff00597493961edda3bb31733f0218b0464940c11d041e1a03bd383678416bb"}, {"bytes": 349, "case_id": "P07_six_basenames", "mode": "0600", "sha256": "2329325dde29d5c9f862ef23824f3118032997cc0ebe79ce6c50d23bc4f2f59b"}, {"bytes": 90, "case_id": "P08_unknown_privacy", "mode": "0600", "sha256": "d4aa3d482b3369e5ceadfc043bba645775e70a73a7790fa820b93147643604ac"}, {"bytes": 99, "case_id": "P09_duplicate_direct", "mode": "0600", "sha256": "d32575a6e50117c31f1f5643aa1b84f7629f2ac5108ccc76aff1b35087544746"}, {"bytes": 99, "case_id": "P09_duplicate_direct", "mode": "0600", "sha256": "d32575a6e50117c31f1f5643aa1b84f7629f2ac5108ccc76aff1b35087544746"}, {"bytes": 99, "case_id": "P10_duplicate_branch", "mode": "0600", "sha256": "d32575a6e50117c31f1f5643aa1b84f7629f2ac5108ccc76aff1b35087544746"}, {"bytes": 99, "case_id": "P10_duplicate_branch", "mode": "0600", "sha256": "d32575a6e50117c31f1f5643aa1b84f7629f2ac5108ccc76aff1b35087544746"}, {"bytes": 99, "case_id": "L01_AssertionError", "mode": "0600", "sha256": "d32575a6e50117c31f1f5643aa1b84f7629f2ac5108ccc76aff1b35087544746"}, {"bytes": 97, "case_id": "L02_RuntimeError", "mode": "0600", "sha256": "948083255d940d94424c7c130cf7af1eba2c0e2e8906a0456ab0aa8c7b578886"}, {"bytes": 95, "case_id": "L03_ValueError", "mode": "0600", "sha256": "be959391f7610dd67cc74b76db58dea15f0878c9d163d1e27884c3a156b1be8f"}, {"bytes": 94, "case_id": "L04_TypeError", "mode": "0600", "sha256": "2476f1365fa9a11e9bee9b6c53e9ebf3cb8f16555380a0d22ef538d4387ef8bf"}, {"bytes": 93, "case_id": "L05_KeyError", "mode": "0600", "sha256": "28dd7c8917b29cb5b34d5f1bcf1301d586fd4c68a75df5ca7fb7f2058de05eda"}, {"bytes": 95, "case_id": "L06_IndexError", "mode": "0600", "sha256": "0077dab295ca64b61ce470022745a33ea8d9997a5d8f77bec4ce550a04f916e7"}, {"bytes": 94, "case_id": "L07_NameError", "mode": "0600", "sha256": "44fdaa5c2f52258f67b3c361d4178a08f2ea0cc17b0ab901c0179a1dfe2a899c"}, {"bytes": 96, "case_id": "L08_ImportError", "mode": "0600", "sha256": "06032bde6f191fd96375fba10b5d93af4170a73d04261c32f8a848bd961b214e"}, {"bytes": 104, "case_id": "L09_ModuleNotFoundError", "mode": "0600", "sha256": "cb007fd221fe892a198e528e879de7ffc533a01bc2660cb14669a2487a852e58"}, {"bytes": 92, "case_id": "L10_OSError", "mode": "0600", "sha256": "7573cfb65b96d64d81481bc2cedd0be36046d6721d287d196f40b29fcb7eab9f"}, {"bytes": 102, "case_id": "L11_FileNotFoundError", "mode": "0600", "sha256": "2e6703b4ec9e0b888c31ab55abe388b7779ee69cd08369cfac4ea068f26322de"}, {"bytes": 100, "case_id": "L12_PermissionError", "mode": "0600", "sha256": "afe3515407845fb1501f30d282c4515140a9255ff1adc900ae9a9d7a03c3c22b"}, {"bytes": 96, "case_id": "L13_SyntaxError", "mode": "0600", "sha256": "b5b84ed791dd025cbb4cc1ac87e9eb53d52c681afb7e61e277b5a2523be59325"}, {"bytes": 95, "case_id": "L14_SystemExit", "mode": "0600", "sha256": "17ba0cc1bdc61c2803ec17fb655b6235ee5b22f8a3eceba8f4b6bdaf22af22df"}, {"bytes": 102, "case_id": "L15_KeyboardInterrupt", "mode": "0600", "sha256": "9dca765b601a27b16d856e5ebca40cad32b527702347e4a6f405bebfcbacb6d6"}, {"bytes": 95, "case_id": "S04_one", "mode": "0600", "sha256": "17ba0cc1bdc61c2803ec17fb655b6235ee5b22f8a3eceba8f4b6bdaf22af22df"}, {"bytes": 95, "case_id": "S05_true", "mode": "0600", "sha256": "17ba0cc1bdc61c2803ec17fb655b6235ee5b22f8a3eceba8f4b6bdaf22af22df"}, {"bytes": 95, "case_id": "S06_negative", "mode": "0600", "sha256": "17ba0cc1bdc61c2803ec17fb655b6235ee5b22f8a3eceba8f4b6bdaf22af22df"}, {"bytes": 95, "case_id": "S07_string", "mode": "0600", "sha256": "17ba0cc1bdc61c2803ec17fb655b6235ee5b22f8a3eceba8f4b6bdaf22af22df"}, {"bytes": 95, "case_id": "S08_float_zero", "mode": "0600", "sha256": "17ba0cc1bdc61c2803ec17fb655b6235ee5b22f8a3eceba8f4b6bdaf22af22df"}], "passed_case_ids": ["P01_trusted", "P02_untrusted", "P03_mixed", "P04_line_bounds", "P05_last8", "P06_first64", "P07_six_basenames", "P08_unknown_privacy", "P09_duplicate_direct", "P10_duplicate_branch", "P11_write_failure", "L01_AssertionError", "L02_RuntimeError", "L03_ValueError", "L04_TypeError", "L05_KeyError", "L06_IndexError", "L07_NameError", "L08_ImportError", "L09_ModuleNotFoundError", "L10_OSError", "L11_FileNotFoundError", "L12_PermissionError", "L13_SyntaxError", "L14_SystemExit", "L15_KeyboardInterrupt", "S01_none", "S02_zero", "S03_false", "S04_one", "S05_true", "S06_negative", "S07_string", "S08_float_zero", "D01_valid", "D02_empty_frames", "D03_eight_frames", "D04_line_one", "D05_line_4096", "D06_other", "D07_size2048", "D08_size2049", "D09_malformed_json", "D10_empty_bytes", "N01_extra_root_key", "N02_missing_class", "N03_missing_frames", "N04_root_list", "N05_root_null", "N06_root_int", "N07_root_string", "N08_root_bool", "N09_class_int", "N10_class_bool", "N11_class_null", "N12_class_unknown", "N13_class_list", "N14_frames_dict", "N15_frames_null", "N16_frames_string", "N17_frames_bool", "N18_nine_frames", "N19_extra_frame_key", "N20_missing_file", "N21_missing_line", "N22_frame_null", "N23_frame_list", "N24_frame_string", "N25_file_path", "N26_file_unknown", "N27_file_int", "N28_file_bool", "N29_file_null", "N30_file_list", "N31_line_true", "N32_line_false", "N33_line_float", "N34_line_string", "N35_line_null", "N36_line_zero", "N37_line_negative", "N38_line_4097", "N39_line_large", "N40_line_list"], "passed_cases": 84, "planned_case_ids": ["P01_trusted", "P02_untrusted", "P03_mixed", "P04_line_bounds", "P05_last8", "P06_first64", "P07_six_basenames", "P08_unknown_privacy", "P09_duplicate_direct", "P10_duplicate_branch", "P11_write_failure", "L01_AssertionError", "L02_RuntimeError", "L03_ValueError", "L04_TypeError", "L05_KeyError", "L06_IndexError", "L07_NameError", "L08_ImportError", "L09_ModuleNotFoundError", "L10_OSError", "L11_FileNotFoundError", "L12_PermissionError", "L13_SyntaxError", "L14_SystemExit", "L15_KeyboardInterrupt", "S01_none", "S02_zero", "S03_false", "S04_one", "S05_true", "S06_negative", "S07_string", "S08_float_zero", "D01_valid", "D02_empty_frames", "D03_eight_frames", "D04_line_one", "D05_line_4096", "D06_other", "D07_size2048", "D08_size2049", "D09_malformed_json", "D10_empty_bytes", "N01_extra_root_key", "N02_missing_class", "N03_missing_frames", "N04_root_list", "N05_root_null", "N06_root_int", "N07_root_string", "N08_root_bool", "N09_class_int", "N10_class_bool", "N11_class_null", "N12_class_unknown", "N13_class_list", "N14_frames_dict", "N15_frames_null", "N16_frames_string", "N17_frames_bool", "N18_nine_frames", "N19_extra_frame_key", "N20_missing_file", "N21_missing_line", "N22_frame_null", "N23_frame_list", "N24_frame_string", "N25_file_path", "N26_file_unknown", "N27_file_int", "N28_file_bool", "N29_file_null", "N30_file_list", "N31_line_true", "N32_line_false", "N33_line_float", "N34_line_string", "N35_line_null", "N36_line_zero", "N37_line_negative", "N38_line_4097", "N39_line_large", "N40_line_list"], "planned_cases": 84, "private_fixture_mode": "0700", "real_helpers_or_surrounding_source_executed": false, "schema": "riauth.a09-projection-memory-validation/v2", "source_ast_sha256": {"classes": "6bed3558dc317f0c6029ee2b025e3b94bc89fec3f010aaf2d6aa946cc0a80402", "failure_source": "ba9f70af426868a9f31ca57295c2e6dc82335ab58f2c1b5a9a71f6a4419d81d5", "handler": "832ad7408b2ea88ce3f64d6ce8ec1e7a2b049817daeb5d47befec2662dc563c4", "shape_predicate": "54132d0cffb1f29daec0aeb20a86c8ae97b738f3a27ff7488d0d56cc35a0c565", "size_predicate": "41d79137b63623714ac89dd7f3203c7a1560a69ebac397b91da43eb34f42a1ce"}, "status": "passed"}, "result_bytes": 9299, "result_sha256": "4c12e51867b11722e9e469ae68f3dee352c6531cb2eadd1ffca523b9c75baf26"}
```

## Preservation and disposition for this separately released attempt

The complete old report prefix is24,692 bytes, SHA256
`f320f18dff6fb811544ebd175d666d0faa7f2c9e1db616e3ed43bd91595067f0`.
Old22715c remains FAILED preparation/0cases; this is one different, now executed
memory release. The original37043196924 PG/helper phase_exit remains internally
UNKNOWN, and the first equal-SHA root input error remains separate. This pass
neither diagnoses nor fixes that original native failure.

No actual surrounding workflow/controller/bootstrap main/import/runpy/helper/
source script, native tool/library, PG/service/provider/HTTP/network/socket/
CLI/browser/desktop, Cargo/slot, query/download/dispatch or other worker was
invoked. Only the approved isolated Python host harness executed selected
projection/handler/predicate AST and synthetic frame declarations. Temporary
owned output was removed; no harness/evidence source file was added to the repo.
No source edit, worker contact, new worker/task/worktree/managed shell, merge/
reset/main/push/status action. I02 DONE and other closed rows are preserved.

Root owns remote observer reservation/invocation, review and integration. This
evidence supports the narrow memory/privacy/shape behavior of the exact565b
observer; it is not a remote native-helper success, universal security proof
or original A09 closure. Append-only/source-payload/receipt/hash/docs/whitespace/
scope/clean commit checks complete the report without another invocation.

Final report checks passed: full24,692-byte prior-prefix preservation; exact
full harness, raw stdout and both canonical receipt byte counts/hashes; all five
source/harness AST identities;84-case/32-capture correspondence; sole append
scope with zero deletions; LF/fences/trailing whitespace; docs checker exit0 and
Git whitespace exit0. Staged sole-report/whitespace and clean own-branch checks
accompany this one append commit. No second child or candidate execution followed.
