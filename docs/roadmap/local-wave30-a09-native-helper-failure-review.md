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
