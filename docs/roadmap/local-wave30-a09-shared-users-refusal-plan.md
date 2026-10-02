# A09 shared user-administration refusal: source-first plan

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Original task: `506e3979-a590-4af3-8fa8-ee90d3a517f2`.
Existing worktree: `a1303b57-4a34-487e-9c63-a841f05b51a0`,
branch `roadmap/local-workflow-safety-wave27`.
Review date: 2026-10-02.
Read-only source/report reservation; no helper/workflow/product ownership
has been exercised. Own starting HEAD:
`e4512d2deb072fa92833d7f536d2d4f67c0e8d63`.

## Finding and proposed reservation

The current shared probe mistakes a filtered collection read for an
administration mutation. At the pinned product source, an authenticated
auditor receives the permitted collection result from `GET /api/users`;
having no matching `user.read` grant yields an empty array, not a collection
403. This does not grant user creation or any other administration mutation.

Recommend one replacement inside
`scripts/check-local-edition-transition-postgres.py::shared_probe`:
use the existing delegate's public CLI to attempt creation of one new
non-administrator, with a fresh current revision and idempotency key.
Require both the actual CLI exit 4 and its HTTP 403 / `access_denied`
envelope. Compare complete ordered PostgreSQL Store records immediately
before and after this refusal. Keep the existing `users_status: 403`
sample field, now describing the actual user-create refusal, so all three
Essentials–Platform–Essentials sample comparisons remain intact.

The only corresponding workflow change would be the one fixed helper
SHA-256 in `FIXED["imports"]`. Both exact prospective diffs and whole-file
identities are archived below. Root must reserve these existing-file
hunks, review the resulting immutable source, choose the distinct validator
commit and separately authorize any remote invocation. No runtime slot was
acquired or released in this audit.

## Pins and finite supplied evidence

| Role | Immutable pin |
| --- | --- |
| Failed remote run / job | `37054511216` / `110995799968` |
| Workflow | `94054b3c9b674e445893b52c1d7de29703fca73c` |
| Validator helper and imports | `30f5a8a884ec4ea398c00bde596ed8f8764a8aab` |
| Installed product | `9a819317efb3a13fa27cd86f884be2be00898fc0` |

Read the complete supplied `controller.json`, `resources.jsonl`,
`cleanup.json` and `helper-redacted.json` under
`/tmp/riauth-wave30-a09-shared-observer-37054511216/members/riauth-local-shared-arm64-37054511216-1/`.
Their SHA-256 identities are:

| Member | SHA-256 |
| --- | --- |
| controller.json | `ac641a8b52265826721366429ed0415762e243b775fbb2b5b09ee8f45f19dfac` |
| resources.jsonl | `1504a0b6e57cc93a7638d53eb5b3dda6c5a83e4b48d8bf19f1b4337bad342a12` |
| cleanup.json | `d3b6f9bc7107fa540408dbf195a74b00919943f8555481969ea6ccf4fdbe35e3` |
| helper-redacted.json | `ea7793e91dfdf9fc5f1d08bacad8429368cd69cd2131927da2ccf74b330c8f08` |

The controller records exit 1, first failure
`phase_exit / focused-shared-helper / exit_code 1`, and helper elapsed
2.803755 s. Its finite observer identifies `AssertionError` frames at
`bootstrap.py:60`, shared helper `370 / 256 / 150`, and matrix `79`.
Helper line 256 is the first Essentials `shared_probe` invocation; line
150 is precisely the `GET /api/users == 403` assertion.
The private stderr identity reported by the controller is 1,148 bytes,
SHA-256 `2081126446f2c418a6f2d201fdccffe256b8aad4328b11af18169c46368dd6da`;
stdout is empty. No private stderr body was read.

The supplied observer does not disclose the actual HTTP result or a response
body. The expected filtered HTTP 200/empty-array behavior is a source-derived
explanation of this assertion boundary, not a freshly measured response.
The first probe had already passed its delegate identity and audit-read
checks in the sequential source before reaching line 150.

Seven resource samples span 19:30:14.720535–19:30:26.540389 UTC. Reported
maximum gap is 2.000528820999989 s; minimum free bytes for all three observed
paths is 115,577,597,952. Cleanup records `failures: []`,
owned fixture/private scratch removal, one reaped owned process and no
remaining owned processes. PostgreSQL PID-file verification is false with
`no_live_owned_postgres`; this is not evidence of a verified live-cluster
shutdown. Helper-redacted remains the fixed pending/not-run placeholder,
the controller sample is `not_run`, and `full_shared_gate` is
`not_certified`.

Both prior failed attempts remain failures. The earlier attempt's missing
inner cause remains unknown; this source diagnosis applies only to the
latest supplied finite frames. None of those earlier receipts or reports
was changed, and no success is inferred for a later phase.

## Exact source contract

All line references in this section are to the product pin above unless
explicitly marked validator.

* `src/api.rs:2064–2079` routes an unpaginated users GET to
  `Core::list_users`. `src/core.rs:497–522` authenticates the principal,
  scans users, adds only views allowed by `user.read` and returns
  `Ok(Value::Array(users))`. An array read does not prove mutation authority.
* `src/agent.rs::principal` checks the live session, identity and active
  exact grants on each credential use. `Principal::allows/require` respects
  edition availability and delegated grants; missing authority produces
  `Error::forbidden`.
* `src/delegation.rs:255–273` permits every active human grant to read
  `state/revision`, while Auditor only permits `audit.read` at its exact
  scope. `active` at 282–343 rechecks the holder/provenance and exact
  `audit/events` / `events` binding. Therefore the existing delegate can
  obtain the current revision itself and cannot create users.
* The complete imported encrypted fixture was read. Its
  `live_identity_and_grant` at validator 184–210 creates a real non-admin
  `delegate` through the administrator's public CLI, assigns the immediate
  exact auditor grant and confirms its stable target. The shared helper's
  `ordinary_fixture` reuses only that live fixture and adds the existing
  group/membership. No historical format-2 probe/run/metadata is imported
  as runtime evidence.
* `src/api.rs:2921–2931` serves revision from a Store read under
  `state.read/state/revision`. The proposed read precedes the refusal
  snapshot. Login and logout remain outside that snapshot interval.
* The complete `gate.cli`, `gate.remote` and their surrounding module
  were read. `cli` captures the installed binary's JSON, requires actual
  exit 4 via `expected=4`, validates `riauth.cli/v1` and `ok=false`,
  and returns the full error envelope. `remote` preserves the current
  server, private session file and noninteractive mode.
* `src/cli.rs:2593–2623` requires both mutation flags, reads the password
  through stdin and sends a genuine authenticated `POST /api/users` with
  a `NewUser`. The proposed username is new and nonreserved; the default
  administrator flag remains false. It neither reuses a creation receipt
  nor submits malformed input to manufacture a refusal.
* `src/cli/transport.rs:194–232` sends the session bearer, fresh
  `Idempotency-Key` and exactly quoted numeric `If-Match`.
  `src/api.rs:872–978` validates these original headers, retains the
  original quoted validator and binds its exact bytes and body to the
  request fingerprint. No alternate request adapter/header reconstruction
  is proposed.
* `src/core.rs:524–538` rejects missing headers with 428.
  `mutation_checked` at 69–110 checks principal/receipt/current revision
  before the writer operation. With a newly generated key and current
  revision, `src/management.rs:1643–1679` reaches
  `actor.require("user.write", ...)` before password hashing/history,
  user/index writes, audit and provenance. A stale revision (409), missing
  headers (428), bad body (400) or dead session (401) would fail this new
  probe, not count as an authorization refusal.
* `src/error.rs` maps forbidden to HTTP 403 / `access_denied`.
  `response_json` at `src/cli.rs:3021–3045` preserves the response status
  and error code; `report_error` at 65–93 maps HTTP 403 to CLI exit 4
  (409/412/428 instead map to exit 5). The proposal checks the envelope's
  `http_status`, `code` and `exit_code` as well as the actual exit.
* `src/store.rs:1095–1135` commits a PostgreSQL writer only after its
  closure returns success. A permission error exits before commit and before
  receipt creation. The proposed runtime comparison verifies the full
  application Store row set rather than relying on that source trace alone.

## Full refusal snapshot and protected behavior

The existing shared `rows` intentionally omits revision, activation,
edition/security metadata and transition history for cross-edition
comparisons. It must not be reused as proof of a full refusal snapshot, and
its filters remain byte-equivalent.

The proposed nested `refusal_rows` selects every key and value of
`riauth_store.records_v1`, hex-encoded and ordered by key. Comparison is
of the complete captured bytes, with no exclusions or hash-only subset:
identities, credentials, receipts, audit, indexes, revision, activation,
security and transition-history rows are all covered. It requires successful
nonempty output and compares immediately around only the attempted create.
This proves unchanged durable Store records if it passes; it does not claim
unchanged PostgreSQL connection statistics, WAL, logs or operational counters.

The fixed fixture creates its own loopback connection file at validator
helper 242–253, mode 0600, and saves it through the existing PostgreSQL
configuration. The nested read uses that configuration's connection file,
resolving relative paths against the config directory. It strictly validates
the existing fixture's literal loopback host, database, user and SSL mode,
then passes those fixed values and its validated port explicitly to psql.
It does not rely on environment connection-string expansion, accept another
target or print the connection contents. The existing controller's root-owned,
hash-checked PG16 bin directory remains first in PATH. Each read disables
psql startup files and password prompting, stops on SQL errors and has a
five-second timeout. Rows and stderr are captured in memory and never
included in assertion text or the public report. The disposable creation
password remains stdin-only; no token/session/body/private connection value
is added to output.

No additional global helper/import, public evidence field or main-call
argument is needed. The existing administrator flow is unchanged. The
delegate/grant/group sample, expiry nonrenewal, logout, previously logged-out
token refusal, strict E-P-E equality, all preserved-row/metadata comparisons,
direct-open/rollback/configuration/live-client refusals and final PG cleanup
remain unchanged. The retained `users_status` name is narrowly interpreted
as user-administration refusal, not the collection-read status.

## Archived exact prospective patch

This patch exists only inside this report; the existing helper has not
been written.

```diff
--- a/scripts/check-local-edition-transition-postgres.py
+++ b/scripts/check-local-edition-transition-postgres.py
@@ -147,8 +147,35 @@
         token = json.loads(session.read_text())["token"]
         matrix.require(authenticated_status(base, "/api/audit?limit=1", token) == 200,
                        "active auditor grant stopped authorizing audit read")
-        matrix.require(authenticated_status(base, "/api/users", token) == 403,
+        # Collection reads filter visible users; creation is the administration boundary.
+        connection = config.parent / tomllib.loads(config.read_text())["postgres"]["connection_file"]
+        target = re.fullmatch(r"host=127\.0\.0\.1 port=([0-9]+) dbname=riauth_transition "
+                              r"user=riauth_test sslmode=disable", connection.read_text().strip())
+        matrix.require(target is not None and 0 < int(target[1]) <= 65535,
+                       "unexpected user-create refusal snapshot target")
+
+        def refusal_rows():
+            result = subprocess.run([
+                "psql", "-h", "127.0.0.1", "-p", target[1], "-U", "riauth_test", "-d", "riauth_transition",
+                "-X", "--no-password", "-v", "ON_ERROR_STOP=1", "-At", "-F", "|", "-c",
+                "SELECT encode(key,'hex'),encode(value,'hex') FROM riauth_store.records_v1 ORDER BY key",
+            ], capture_output=True, timeout=5)
+            matrix.require(result.returncode == 0 and result.stdout,
+                           "full user-create refusal snapshot unavailable")
+            return result.stdout
+
+        revision = gate.remote(server, base, session, "revision")["revision"]
+        before_refusal = refusal_rows()
+        denied = gate.remote(server, base, session, "--if-revision", revision,
+                             "--idempotency-key", os.urandom(16).hex(),
+                             "user", "create", "shared-refused-user", "--password-stdin",
+                             input="q08-refused-disposable-password\n", expected=4)
+        matrix.require(denied["error"]["http_status"] == 403
+                       and denied["error"]["code"] == "access_denied"
+                       and denied["exit_code"] == 4,
                        "ordinary auditor gained user administration")
+        matrix.require(refusal_rows() == before_refusal,
+                       "refused user creation changed durable records")
         time.sleep(0.2)
         again = gate.remote(server, base, session, "whoami")
         matrix.require(again["expires_at"] == me["expires_at"], "session expiry was renewed")
```

The required fixed-validator mapping is also prospective only:

```diff
--- a/.github/workflows/check-local-shared-handoff.yml
+++ b/.github/workflows/check-local-shared-handoff.yml
@@ -236,7 +236,7 @@
             "imports": {
               "check-exact-edition-matrix.py": "f07d934f9cfd7af20eb086f5838863c28f840ee848e62bea1d865b330643d887",
               "check-installed-release-gate.py": "cbe8dcb42b4b1c36dc8df00204103b9c955feb8057275a441f60f3072d2bf8b5",
-              "check-local-edition-transition-postgres.py": "575dfb049324ede3cb0ddf4bd71e0b42ef46704a015f6a576a38c4052a1ff2c1",
+              "check-local-edition-transition-postgres.py": "d86d9a99c9e09e29eb43af52f409332e11a70caaed90b5545bf132df1f2a1982",
               "check-local-encrypted-edition-transition.py": "09e137d88eb8e873a488448b7bdfdbe80d2327fbca716a93f445eaaae1ea9e8e",
               "spdx_sbom.py": "ca063ab3d4abb6ec815151bcf762447e96682076a7f72d2dbfa9d7e6d3a1032c"
             },
```

At workflow 769–787, `Controller.source` verifies the distinct full
validator commit and every fixed import hash. `Controller.helper` passes
this same fixed helper hash to the bootstrap, records it and rechecks imports
after execution. Updating only the dictionary value therefore preserves
both pre- and post-run enforcement.

The future `validator_source_sha` must be the reviewed commit containing
the corrected helper and all other pinned imports, distinct from the product
and workflow commit. Its commit identity is not yet known or authorized.
Product `9a819317efb3a13fa27cd86f884be2be00898fc0`, build workflow
`036a392656b4b5070cc86a11d5ca3258b7b868d2`, run `37016520583`,
job `110868629053`, artifact `11232871527`, its full ZIP digest and
18-file allowlist, all five installed binary pins, the other four import
hashes, native/PG prerequisite checks, transport limits, resource policy,
ownership/reaping/retention, bootstrap/finalization and upload remain fixed.
No new archives, compilation, provider setup or product change is proposed.

## Static byte and AST witnesses

Virtual texts were reconstructed from Git objects in memory. Only standard
Python parsing/hash/diff operations were used; no helper/module/imported
fixture/controller/bootstrap was executed or imported.

| Text | Original bytes / SHA-256 | Prospective bytes / SHA-256 |
| --- | --- | --- |
| Complete shared helper | 21716 / `575dfb049324ede3cb0ddf4bd71e0b42ef46704a015f6a576a38c4052a1ff2c1` | 23518 / `d86d9a99c9e09e29eb43af52f409332e11a70caaed90b5545bf132df1f2a1982` |
| Complete workflow | 77543 / `45304da275d3072c2c0de9f6debed1acc335ffa99e0ed6baa15064549bab5574` | 77543 / `5290d7a5d9e15eaeaa19169523668e9658b0341d4eb76ca47695c532e7aa27b2` |
| Complete inline controller | 63045 / `ffa6d792bcf9170018f6bcd2bbd58d0920444dd2720c5570bdd9de9fadfc0e32` | 63045 / `1d1419c6b726bfd015712a25cc129ee54951e7ac76fb9c4a25891fef040278a7` |

Original/prospective helper Git blobs:
`d55d51aad16f508b3cb993a15f44e23aeb912729` /
`593220583104cf075bf3635e2e6c9feefb3177fd`.
Original/prospective workflow Git blobs:
`b43653d09de8ce5db3bc3300a5e78457aa0c7b82` /
`20df5c4a18ce214746ed2232fa78a9231bd8db69`.

Static assertions passed:

* Exact one-hunk replacement and reversal restore the entire 21,716-byte
  helper and its original SHA/blob. The top-level AST differs only in
  `shared_probe`; every other function, import, constant and entry point
  is identical. Its return statement/sample is AST-identical.
* The workflow contains the original helper digest exactly once.
  Reversing the single literal replacement restores the whole 77,543-byte
  workflow and its original SHA/blob.
* Decoding the complete `FIXED` JSON shows exactly the one helper import
  value changed. Normalizing that literal makes the entire controller AST
  identical; every executable controller body remains unchanged.
* Bootstrap bytes are identical: 3678 bytes,
  SHA-256 `27361ddd98813b9e539758ca26a433ea86a1b085b0fd68036a39678ed464f7e2`.
* The four other validator imports were identity-checked from Git against
  the fixed map. Complete gate/matrix/encrypted/helper executable bodies
  were read, not merely their hashes; the SPDX dependency was identity-checked.
  Product authorization, CLI, header, error, writer and session source
  sections were inspected, not executed.

One source search for nonexistent rate/throttle path names returned no matches
(exit 1); it is not a runtime failure or proof of absent rate limiting.
The source conclusion and proposal instead rely on the exact API and Store
bodies above.

## Scope and remaining prerequisites

Only this new report may be committed. The existing helper, workflow,
production, old reports and private evidence are untouched; no alignment
merge/reset, cleanup/deletion, native executable, Cargo, service, PostgreSQL,
HTTP/network/remote query/download/dispatch, desktop, worker/contact or status
action occurred. No new runtime result, current CI success, original A09
completion or universal release/host claim is made. I02/I10/R05/W02/W05 remain
DONE; original A09 interpretation/integration/publication remain root-owned.

A separately reserved source implementation and fixed-validator workflow
mapping are the next prerequisite. A separately authorized same-product,
same-artifact hosted invocation must then measure the actual HTTP/CLI
refusal, full Store snapshot equality and all subsequent E-P-E checks.
The new snapshot must refuse, not fall back to filtered rows, on any read or
comparison failure. Any unexpected current-revision/header/permission
behavior remains a failure for source-first diagnosis rather than a weaker
assertion or automatic rerun.

## Actual report checks

`python3 scripts/check-docs.py` exited 0. Static reconstruction from both
diff fences in this written report reproduced the prospective whole-file
hashes and exact reversals; the shared-helper AST scope check also passed.
The first staged `git diff --cached --check` exited 2 for one extra blank
line at this new report's EOF; that report-only formatting issue was removed.
The corrected staged whitespace check and repeated docs check exited 0.
The prospective connection setup was narrowed during static design from
environment connection-string expansion to the exact existing loopback
fixture shape; only the report's archived virtual proposal changed.
No production or fixture failure was observed locally because none was run.

## Implementation reservation: baseline preflight and held helper, 2026-10-02

Root approved the exact archived helper/workflow source delta in
`wave30_A09_shared_user_create_refusal_implementation`, with runtime held.
The starting branch was clean at the report commit
`5515a317070f9a35b51134721acc4c35cf4babcf`. No alignment was performed.

The published baseline was independently resolved and read from Git at
`69f46cf75390cde70a992eb528c7ee5f769aeeca`. Its helper is exactly 21,716
bytes / SHA-256 `575dfb049324ede3cb0ddf4bd71e0b42ef46704a015f6a576a38c4052a1ff2c1`,
and its workflow is exactly 77,543 bytes / SHA-256
`45304da275d3072c2c0de9f6debed1acc335ffa99e0ed6baa15064549bab5574`.
These match the archived validator/workflow objects byte for byte.

The local helper instead remains 13,127 bytes / SHA-256
`d6b0d023164756b2fcabe1a89794400d5ee70b43b6b4874abed3fd2f6a2cce7d`.
It lacks the archived user-list refusal hunk. This is a baseline mismatch,
not a runtime or authorization failure; the current published source was
not mistaken for this older branch's file. All four other local imports
match their fixed hashes exactly. The local workflow was absent.

The explicit absent-workflow preparation permission was exercised:
`eba66b4931b1de1400a79c183743e40fbc17d4ce` adds only the exact published
workflow baseline. **Root must not integrate this preparation commit.**
It includes no fixed-hash correction or new executable body. It preserves
the own older helper and all other source/history.

Static checks completed for this prepared baseline: installed Ruby Psych
parsed the complete YAML; all three extracted run blocks passed `bash -n`;
complete controller and bootstrap strings passed Python `ast.parse` without
imports/execution; staged whitespace passed. The actual Ruby/Psych and bash
invocations were syntax parsers only, not workflow/controller execution.
The two archived diffs were also reconstructed in memory against published
`69f46cf...`: their expected complete prospective hashes, exact reversals,
helper AST-only change, normalized complete controller AST, unchanged
bootstrap bytes and four other import hashes all passed.

The approved helper edit remains held. Applying only its archived one-hunk
delta cannot transform the local older file into the approved 23,518-byte
result. The reservation explicitly authorized baseline materialization for
an absent workflow, but did not explicitly authorize replacing the older
helper with the published baseline. A clarification was requested before
that additional existing-file preparation. No helper replacement, stale
whole-file import, merge or reset was used as a workaround.

The smallest necessary extension is exact materialization of **only** the
21,716-byte published helper above in a separate preparation commit that
root will not integrate; then the already approved two-file delta can be
committed independently for integration. Its result must still match the
archived `d86d9a99...` helper and `5290d7a5...` workflow in full, with every
protected function/import/product span unchanged. A different baseline or
broader edit is not proposed. No helper/module/harness/native product,
PostgreSQL/HTTP/Cargo/network/remote/service/browser action occurred, and no
runtime slot was acquired or released. Prior failure interpretations and
closed-task statuses remain unchanged; no source implementation or A09
shared full-gate completion is claimed by this preparation phase.

The appended report passed `python3 scripts/check-docs.py` and staged
`git diff --cached --check` (both exit 0). An explicit byte comparison
preserved the full original 20,060-byte / 344-line report prefix and the
original local helper. This evidence append adds no existing-source change.

## Authorized exact implementation and static evidence, 2026-10-02

Root explicitly extended the preparation reservation to the exact reviewed
helper. This supersedes the hold for new work without rewriting the dated
baseline mismatch above. Own history retains the older 13,127-byte helper,
the complete original proposal and that mismatch record. No alignment,
merge/reset or other source import was performed.

Preparation commits, **both excluded from root integration**:

* Existing workflow preparation:
  `eba66b4931b1de1400a79c183743e40fbc17d4ce`.
* New helper-only baseline preparation:
  `7c44cc15c19260e4066cea9cf4bf18080fd6f7e2`, directly from validator
  `30f5a8a884ec4ea398c00bde596ed8f8764a8aab`. It is exactly 21,716 bytes,
  SHA-256 `575dfb049324ede3cb0ddf4bd71e0b42ef46704a015f6a576a38c4052a1ff2c1`,
  and preserves the reviewed/local `100644` mode. Its preparation diff
  contains only the helper; no workflow/hash correction is mixed into it.

Integration source commit:
`f7af6cc85b938738e6152b4b0e5109e36a3abdec`, parent
`7c44cc15c19260e4066cea9cf4bf18080fd6f7e2`. It changes only these two files,
29 insertions / 2 deletions:

| Source file | Actual committed bytes / SHA-256 | Git blob |
| --- | --- | --- |
| scripts/check-local-edition-transition-postgres.py | 23,518 / `d86d9a99c9e09e29eb43af52f409332e11a70caaed90b5545bf132df1f2a1982` | `593220583104cf075bf3635e2e6c9feefb3177fd` |
| .github/workflows/check-local-shared-handoff.yml | 77,543 / `5290d7a5d9e15eaeaa19169523668e9658b0341d4eb76ca47695c532e7aa27b2` | `20df5c4a18ce214746ed2232fa78a9231bd8db69` |

The patch was extracted from the two diff fences in immutable
`5515a317070f9a35b51134721acc4c35cf4babcf`. `git apply --check -` and
`git apply -` both exited 0. No context or substitution variation was used.
Actual complete `difflib.unified_diff` output equals those archived fences
exactly; in-memory reversals restore the reviewed whole helper/workflow
bytes and their `575dfb04...` / `45304da2...` identities.

Actual static checks all passed:

* Python `ast.parse` of the complete edited helper. Its top-level AST
  differs only in `shared_probe`. Complete raw source bytes of `load`,
  `rows`, `transition_metadata`, `require_target_metadata`,
  `require_preserved`, `authenticated_status`, `ordinary_fixture`,
  `shared_config_refusals` and `main` are identical to the reviewed helper.
  Imports/constants/entry point and the sample return AST are unchanged.
* Installed Ruby Psych parsed the complete edited workflow. Each of its
  three extracted run blocks passed `bash -n`. The complete edited inline
  controller and bootstrap strings passed `ast.parse`; none was imported
  or executed. Bootstrap remains 3,678 bytes / SHA-256
  `27361ddd98813b9e539758ca26a433ea86a1b085b0fd68036a39678ed464f7e2`.
* Decoding both complete `FIXED` maps proves the sole change is the helper
  import hash. Normalizing that literal restores complete controller AST
  equality. All other workflow bytes, executable bodies, product/artifact
  identities, refusal/transport/resource/ownership/finalization/upload
  boundaries remain unchanged.
* All four other fixed imports match both validator bytes and hashes in
  the actual committed source tree, not merely the working directory:
  matrix `f07d934f9cfd7af20eb086f5838863c28f840ee848e62bea1d865b330643d887`;
  gate `cbe8dcb42b4b1c36dc8df00204103b9c955feb8057275a441f60f3072d2bf8b5`;
  encrypted fixture `09e137d88eb8e873a488448b7bdfdbe80d2327fbca716a93f445eaaae1ea9e8e`;
  SPDX `ca063ab3d4abb6ec815151bcf762447e96682076a7f72d2dbfa9d7e6d3a1032c`.
* `python3 scripts/check-docs.py`, `git diff --check`, and the staged
  `git diff --cached --check` exited 0. Staged source scope was checked
  before commit; immutable commit scope contains only the two approved paths.
  No `src`, crates/vendor, manifest/lock or toolchain changes occurred.

This implements exactly the archived public user-create refusal check:
fresh delegate revision before the complete snapshot, fresh idempotency key,
non-admin valid creation input/password on stdin, actual CLI exit 4 plus
HTTP 403 / `access_denied`, and complete ordered `records_v1` byte equality.
The snapshot target accepts only the fixture's exact loopback connection
shape and explicit psql host/port/database/user; reads have five-second
timeouts, no startup files/password prompting and no raw captured values
in output. All prior E-P-E comparisons and refusal checks remain intact.
These are source/static assertions; the HTTP/CLI result and snapshot
equality have not been evaluated in this phase.

No static check failed in this implementation phase. The earlier report
formatting correction, source baseline hold and historical remote failures
remain recorded in full. The latest finite remote boundary remains diagnosed
by source; an earlier missing inner cause remains unknown. No borrowed or
fresh runtime pass, successful hosted shared gate or A09 closure is claimed.

Root still owns full review, integration/publication and a distinct reviewed
validator-source commit containing this corrected helper plus all four exact
imports. The workflow continues to require separate product, workflow and
validator roles. There is no remote/runtime release here: no helper/module
main/import/harness, native product, PostgreSQL/HTTP/Cargo, remote query,
download/dispatch, service/browser, cleanup/deletion, worker/contact or
task/board/main/push action occurred. No runtime slot was acquired or released.
Original A09 shared full gate remains open; I02/I10/R05/W02/W05 remain DONE.

## Root immutable implementation review and one hosted reservation

Root fully read the original 344-line proposal, exact two-file source diff and 91-line actual static appendix. Accepted validator source `bfe0ade02fdddb75c572c648866c7c16e6cb2735` contains the corrected 23,518-byte helper SHA256 d86d9a99c9e09e29eb43af52f409332e11a70caaed90b5545bf132df1f2a1982 and all four other fixed imports. The workflow changes only that fixed helper digest. Independent whole-byte reversals, helper AST comparison, archived-patch identity and unchanged public CLI/Core/delegation contracts support this exact correction. GET collection filtering is no longer credited as an administration refusal. The new valid public creation must return actual CLI exit 4, HTTP 403/access_denied and leave the complete ordered Store row snapshot unchanged. These new runtime outcomes are pending.

Root reviewed and preserves the baseline-preparation hold and all prior remote failures. Preparation commits are excluded from integration. After publishing this reviewed source, root reserves one build-free native ARM hosted shared check using this immutable validator, historical product9a and the separately published workflow. Product, validator and workflow provenance remain distinct. All fixed archive/import, PostgreSQL, finite-command, resource and owned-cleanup controls remain unchanged; no threshold reduction, retry or borrowed success is authorized. Original A09 stays open pending actual evidence and the independent container slice.

## One corrected hosted invocation: retained failure and released lane

The [actual root receipt](evidence/wave30-a09-shared-37060776569.json) records manual run37060776569/job111016616062 at workflowa4824600f03053d4218460992cb29295bfb11d6d, validatorbfe0ade02fdddb75c572c648866c7c16e6cb2735 and historical product9a819317efb3a13fa27cd86f884be2be00898fc0. Immutable transport, five products and PostgreSQL16.15 tool checks completed. The focused helper exited1 in3.204601s. Fixed AssertionError frames397/283/177/matrix79 identify the complete Store-equality assertion. The preceding valid public create refusal predicates CLI4/HTTP403/access_denied passed; nonrenewal and all later handoff checks were not reached. No changed row/key/value or actual cause was retained, so no bookkeeping/product diagnosis is established. The pending helper-redacted placeholder is not a completed sample result.

Root fully read controller14,035B SHA8fff25430be48b1666819ac98c634121af222a6bd582d2f94d5888859b684946, cleanup294B SHAd3b6f9bc7107fa540408dbf195a74b00919943f8555481969ea6ccf4fdbe35e3 and all six resources1,757B SHAdc8af457dc6fb13cf08b020d176cd04928f79f6b762c2afa77db463674a35988. Minimum115577208832B/maxgap2.000573s met unchanged margins. Cleanup failures[]/remaining0, owned fixture/private scratch gone and one owned supervisor child reaped; the serialized runtime lane was released immediately after that proof, before further review. Artifact11250204873 name/digest are API/upload reported; decoded members were retained and rehashed, outer ZIP was not retained/rehashed. Earlier failures remain unchanged. No automatic repeat or source correction occurred. A read-only exact-source diagnosis is separately reserved; A09 remains open.

## Corrected hosted refusal: snapshot failure and source diagnosis, 2026-10-02

Reservation: `wave30_A09_user_create_snapshot_source_diagnosis`, project
`891e7443-8dac-4c1b-897f-9e53cb59c7ee`, original A09
`506e3979-a590-4af3-8fa8-ee90d3a517f2`, existing worktree
`a1303b57-4a34-487e-9c63-a841f05b51a0`. This phase appends only this report.
Its starting HEAD is `baaeb94d13257d6ed9a6325cd01e14e91f8eb18b`; no alignment,
source edit or new runtime was performed. All earlier failures, preparation
holds, proposals and static implementation evidence above remain dated evidence.

### Actual finite observation

Root published the corrected source at workflow/main
`a4824600f03053d4218460992cb29295bfb11d6d`. The separately selected validator
is `bfe0ade02fdddb75c572c648866c7c16e6cb2735`; product remains
`9a819317efb3a13fa27cd86f884be2be00898fc0`. Hosted run `37060776569`, job
`111016616062`, failed in `focused-shared-helper` with exit 1 after
3.204601 seconds. The fixed observer records `AssertionError` at helper
397 / 283 / 177 and matrix 79, with bootstrap 60. Helper 283 is the first
Essentials shared probe, so this attempt did not complete the later E-P-E
probes or full shared gate.

The exact helper's public user-create refusal at 173–176 completed before
the equality assertion at 177 failed: the public CLI returned actual exit 4
and its envelope asserted HTTP 403 / `access_denied` / exit 4. This failure
is different from the earlier collection-GET permission assumption. No row
diff, changed key, value, collection, timestamp or writer origin was retained.
The observed failure is therefore **complete Store byte inequality after a
successful refusal**, with its actual changed rows and cause still unmeasured.

I read all four authorized finite files in full and recomputed their hashes.
Their common retained directory is
`/tmp/riauth-wave30-a09-corrected-shared-37060776569/members/riauth-local-shared-arm64-37060776569-1/`.

| File | Bytes | SHA-256 |
| --- | --- | --- |
| controller.json | 14,035 | `8fff25430be48b1666819ac98c634121af222a6bd582d2f94d5888859b684946` |
| cleanup.json | 294 | `d3b6f9bc7107fa540408dbf195a74b00919943f8555481969ea6ccf4fdbe35e3` |
| resources.jsonl | 1,757 | `dc8af457dc6fb13cf08b020d176cd04928f79f6b762c2afa77db463674a35988` |
| helper-redacted.json | 70 | `ea7793e91dfdf9fc5f1d08bacad8429368cd69cd2131927da2ccf74b330c8f08` |

`helper-redacted.json` is the pending `not_run` placeholder, not a completed
helper report. Controller records `sample_result=not_run`,
`full_shared_gate=not_certified`, `official_release=false`. The private
stderr's recorded size/hash is 1,128 bytes /
`3aad71af3e7f1d3a78e18ba44471b284c4fe4f4119d59eb4d1713abed89a9be1`;
I did not open its body or any database rows/private logs. The retained
controller identity is
`1d1419c6b726bfd015712a25cc129ee54951e7ac76fb9c4a25891fef040278a7`.

Resource records contain six samples from
`2026-10-02T20:28:22.238610` through `20:28:32.181952` UTC, maximum gap
2.000572659999989 seconds, and minimum free space 115,577,208,832 bytes
across the sampled paths. Cleanup reports `failures=[]`, zero remaining
processes, one reaped process, and fixture/private directories gone.
`postgres_pid_file_verified=false` with `no_live_owned_postgres` does not
establish a live-cluster PID-file verification. These are the retained
observer's historical measurements, not a new local cleanup or runtime pass.

### Complete relevant source bodies and transaction boundaries

Git-object inspection confirms the exact helper is 23,518 bytes /
`d86d9a99c9e09e29eb43af52f409332e11a70caaed90b5545bf132df1f2a1982`
at the validator pin; the workflow is 77,543 bytes /
`5290d7a5d9e15eaeaa19169523668e9658b0341d4eb76ca47695c532e7aa27b2`
at the workflow pin. Both equal the unchanged own source. The complete
helper, gate CLI/remote/serving bodies, encrypted fixture context, matrix
assertion, and workflow failure-observer/forwarding bodies were read.
The complete product bodies listed below were traced at product `9a81931`;
this does not claim an audit of every unrelated body in those large files.

| Pinned body | Source consequence |
| --- | --- |
| Helper `shared_probe` 135–194; first call 283 | Delegate login, whoami, audit read and revision read precede the first full snapshot. Between the two snapshots the helper explicitly makes one user-create CLI call with fresh revision/key and password on stdin. SQL remains the complete ordered `records_v1` key/value hex capture; no exclusions. Later sleep/logout/admin/grant/group traffic is after the failing comparison. |
| `src/cli/local.rs::from_legacy` 208–218; `src/cli.rs::run`, `run_user` 2593–2623; complete `src/cli/transport.rs` | User creation uses the public remote path, not local maintenance dispatch. The create arm performs one explicit authenticated POST; transport validates the session, disables redirects, carries the original headers and sends the request. There is no explicit create retry/relogin/discovery call in this path. |
| `src/api.rs::protect` 872–1202; protected router 625–638 | Before reaching the handler, a PostgreSQL HTTP request executes a separate shared rate-limit writer. POST `/api/users` uses category `general`. The source comment at 1117–1118 explicitly places completion of this transaction before authorization. The redb branch instead uses an in-memory rate table. |
| `src/store.rs::shared_rate_limit[_within]` 736–770 | `Store::write` reads `http_rates/K`, then puts `(start, count.saturating_add(1))`, including requests which are admitted and later denied by authorization. This is an independently committed writer, not part of the denied Core mutation. Capacity handling can reclaim/evict rate records; occurrence in this run is unmeasured. |
| `src/store/maintenance.rs::update_indexes` 203–219; `index_key` 112–114 | A rate put removes/recreates the exact rate expiry index and conditionally updates the exact `index_counts/http_rates` row if collection membership changes. It does not authorize a general index-count exemption. |
| `src/api.rs::create_user` 2467–2475; `src/core.rs::create_user` 524–538, `mutation[_checked]` 38–110; `src/management.rs::create_user` 1643–1679 | The thin handler calls ordinary Core `run`, not connector admission. The mutation resolves the current principal, replay/authority and revision fences. Management requires `user.write` for the user resource before user construction, password/history/index/audit/provenance work. A failing closure does not reach successful receipt saving. |
| `src/store.rs::write`, `postgres_write` 1047–1135; `Tx::put/import_record/raw_put/delete` 1477–1585 | PostgreSQL writers acquire the shared advisory transaction lock and commit only after their closure returns success. The denied creation closure cannot commit its mutation transaction. That rollback cannot undo the preceding HTTP middleware transaction. |
| `src/agent.rs::principal` 153–201; `Core::session/identity_user` 1045–1080; `src/delegation.rs::active` 282 onward; `src/context.rs::management_permissions` 55–63 | Current session/account/grant authority is checked from Store state; these read paths do not supply a denied user-creation receipt or identity mutation. The completed 403 also distinguishes this attempt from missing-header 428. |
| `src/store.rs::record_change` 1332–1388; `src/core.rs::audit_with_details` 1130–1189; assembly/identity transition bodies | Rate bookkeeping is outside the identity change categories and does not call the management audit/revision path. Protected audit, receipt, credential, identity and revision rows remain part of the full comparator. |
| `src/api.rs::App::run/admission`, server `start_background/start_role`, `src/process_role.rs`, background `execute/spawn/cadence` | Worker admission telemetry/semaphores for this handler are process-local. The default integrated server also launches maintenance and other workers. Maintenance cadence is 60 seconds, with the interval's initial tick able to run at startup; no retained evidence identifies an interleaving here. |
| `Core::cleanup` 912–1044; lifecycle cleanup 1217–1254; `maintenance_page` 675–703 | Background cleanup can modify expired records and its own `maintenance_cursors`/`maintenance_bounds` in separate writers. No such write is recorded in this failure. Their possibility cannot justify suppressing proof/session/receipt/audit changes or every maintenance-associated row. |
| Complete readiness route/probe bodies and router composition | Health probes are merged outside `protect`; readiness waits occur before the refusal snapshot. Source does not show a helper-created parallel HTTP polling loop between these two snapshots. This does not rule out independently scheduled background work. |

Exact rate record derivation is source-backed, not an observed changed key:
`K = crypto::digest(grouped_peer_ip + NUL + "general")`, where
`rate_key` preserves IPv4 and groups IPv6 as in `src/api.rs` 2958–2966.
`crypto::digest` at `src/crypto.rs` 62–64 is URL-safe unpadded base64 SHA-256,
not hexadecimal. `record_key` at `src/store.rs` 1139–1141 stores
`bucket + "/" + key`. The possible rate bookkeeping namespaces are therefore
`http_rates/`, `index_expiry_http_rates/`, and the single exact record
`index_counts/http_rates`. The expiry id is the source's 20-digit expiry
followed by `/` and `digest(K)`. No key, peer, counter or expiry from this
run was inspected or retained.

The independently committed rate writer is a concrete source explanation
for why **Core refusal rollback does not promise an HTTP request makes zero
durable Store writes**. It is not evidence that these were the actual changed
rows, that they were the only changes, or that the shared gate passed. A
production authorization/rollback defect is not established. Original A09's
shared gate is not silently redefined as blanket storage-write-free; the
current strict snapshot assertion remains unchanged pending root review.

### One proposed next slice: retain counts without changing the comparator

Recommend one bounded diagnostic data flow, separately reserved and reviewed
before implementation/runtime. No comparator exclusions or production
correction are proposed in this phase.

1. In the existing helper's `shared_probe` refusal block, retain the already
   required second `refusal_rows()` result in a local variable. Compare its
   complete bytes to `before_refusal` with the same assertion/message. On
   that exact `AssertionError`, derive an optional fixed count projection
   from these two in-memory captures, attach it to the exception, and re-raise
   the original exception. No new SQL, request, retry, sleep or clock adjustment.
2. In workflow `BOOTSTRAP.failure_source` 539–567, optionally copy only this
   validated count projection into the existing finite observer. In
   `Controller.helper` 969–987, validate and retain the same optional field
   in `controller.json` alongside the existing class/frames. Update only the
   helper's fixed import digest after its exact reviewed source is selected.
   The four other imports, product/artifact roles, upload allowlist,
   refusal/cleanup/resource boundaries remain protected.

The proposed optional field is `store_snapshot_counts`, with exactly these
six fixed labels. These predicates classify diagnostics only; **none filters
the full comparison or grants permission for a write**.

| Fixed label | Internal decoded Store-key predicate |
| --- | --- |
| http_rates | Starts with exact bytes `http_rates/` |
| http_rate_expiry | Starts with exact bytes `index_expiry_http_rates/` |
| http_rate_count | Equals exact bytes `index_counts/http_rates` |
| maintenance_cursors | Starts with exact bytes `maintenance_cursors/` |
| maintenance_bounds | Starts with exact bytes `maintenance_bounds/` |
| protected_or_other | Every remaining row, including all identity, credential, configuration, revision, receipt, audit and other index rows |

Each label has exactly five integer fields: `before`, `after`, `added`,
`changed`, `removed`. Added/removed mean key presence changes; changed means
the same key has different complete captured value bytes. Internal parsing
must reject malformed hex/rows, duplicate keys or over-limit counts without
emitting their content. Public values must be true integers, not booleans,
within 0–1,000,000. All six labels partition the complete snapshots; no raw
key/id, key digest, value/value digest, subject, timestamp, URI, credential,
private path, exception message or arbitrary collection name is emitted.
Parser/projection failure must omit the optional field and preserve the
original full equality failure; a byte mismatch with zero projected row
differences must also remain a failure, not be normalized away.

The bootstrap/controller must accept only the exact optional schema and only
for the exact `AssertionError`; malformed/missing optional data falls back to
the original fixed class/frames observation. Keep the existing 2,048-byte
packet limit, 64 examined / 8 retained trusted-frame bounds, exclusive 0600
private observer creation and original exception/phase exit. If the optional
data cannot fit, retain the original observer without it. A helper-only print
or success-report field would be insufficient: the failure path currently
retains only class/frames, and the success helper report is not read after
the command exits nonzero. No new public artifact or success claim is needed.

This diagnostic would identify collections and counts, not establish writer
origin, exact key predicates or authorization safety. Any later exception
would require its own exact source-created row/value predicate and proof;
all protected rows remain fully compared now. A further hosted run, helper
source ownership and workflow materialization are root-controlled decisions,
not authorized by this report.

### Scope and limits of this phase

Only Git/static source reads and the four authorized finite documents were
used. No helper import/execution, native product, PostgreSQL/HTTP, Cargo,
remote query/download/dispatch, provider/browser, secret row/private log
inspection, cleanup/deletion, contact, merge/reset/main/push or task/status
action occurred. No runtime slot was acquired or released. This append does
not alter the earlier source hashes, interpreted failure boundaries or
I02/I10/R05/W02/W05 DONE statuses. Earlier failures without an inner cause
remain unknown; the newest retained equality failure remains unmeasured at
row level. Original A09 shared full gate remains open.

Static checks actually completed in this diagnosis phase:

* `python3 scripts/check-docs.py` and `git diff --check` exited 0.
* Explicit byte assertions preserved the complete 29,373-byte previous
  report prefix (SHA-256
  `4dd666b76cae578911d247c133a12246ff6fd0a77d3259636da1f8eb3dd32e76`),
  verified all four authorized evidence identities, and proved the helper
  and workflow remain byte-equal to their exact validator/workflow pins
  and starting own HEAD. Only this report appears in the diff.
* A standalone standard-library JSON size calculation, using six fixed
  categories, all thirty counts at 1,000,000 and eight longest trusted-file
  frame labels at line 4096, produced 1,246 bytes within the 2,048-byte
  observer cap. This evaluates no helper/controller function and proves
  only the prospective fixed packet's capacity, not an implementation,
  observed counts, redaction enforcement or runtime compatibility.


### Root review of snapshot diagnosis

Root read the complete195-line `5c9776ed18a0148aa7bfb0c663fbe4f8cc7d0e3d` appendix and the exact retained failure boundary. The public user-create refusal passed; complete Store equality failed. The separately committed PostgreSQL rate writer supplies a source-backed distinction between HTTP bookkeeping and Core mutation rollback, without identifying the actual changed rows. Root reserves only an exact source-first diagnostic design for the existing full comparison: fixed collection/count projection propagated through both finite observer stages, no comparator exclusion or new request. Source ownership and any hosted repeat remain separately held. The report cherry-pick had an append conflict with root's prior receipt; concatenating the entire existing report and the exact195-line author appendix preserved both in full.

## Exact snapshot-counts design archive, 2026-10-02

Reservation: `wave30_A09_snapshot_counts_exact_design`. Project
`891e7443-8dac-4c1b-897f-9e53cb59c7ee`; A09 support, existing worktree
`a1303b57-4a34-487e-9c63-a841f05b51a0`. This append is design only.
Starting own HEAD is `5c9776ed18a0148aa7bfb0c663fbe4f8cc7d0e3d`. No source/workflow file was changed,
no validation harness or proposed function was executed, and no runtime slot
was taken or released. Remote container run `37061329815` owns the
runtime lane; this design does not overlap it.

The prospective two-file patch below applies against immutable
`f7af6cc85b938738e6152b4b0e5109e36a3abdec`, whose helper/workflow are byte-identical to current own
source and the preceding report's `d86d9a99...` / `5290d7a5...`
pins. It leaves the complete snapshot byte comparator, actual public CLI
4 / HTTP 403 / `access_denied` checks, all E-P-E checks and successful
return data intact. There are no exclusions, new SQL, requests, retries,
sleeps or clock/due-state writes. Actual changed rows and origin in hosted
run `37060776569` remain UNKNOWN.

### Exact prospective identities, not implemented bytes

| Prospective source | Bytes | SHA-256 | Prospective Git blob |
| --- | --- | --- | --- |
| scripts/check-local-edition-transition-postgres.py | 26,547 | `4c60a7448d3c836215068fcc4f6822655c483f172a822e085f077d27b6a0c6fa` | `af764b71b95f8ad4d5e1d9b26d094dfa2eaafe49` |
| .github/workflows/check-local-shared-handoff.yml | 82,149 | `226483dd9edcd5e566b6c5de9f7ab2a36ebdf0c0f204f8ec992645542a3f395f` | `f63110098c13d717446a198c487c00678cfeb60a` |

The helper patch is 3,516 bytes /
SHA-256 `181f58bf3a54dc08b9161324f7047e709111e86f9b26d00a25310b55f1f3082a`; the workflow patch is
7,647 bytes / SHA-256 `d58a77d3f4073a508de5eb2cbdf955fe468ce36ee0fac73cadd98798de52726a`.
Each diff fence contains that exact UTF-8 patch, including its final newline.
They are zero-context unified diffs, intended for later
`git apply --unidiff-zero --check` against the exact baseline only.
Neither git apply nor a source-file write was performed in this design phase.

The prospective extracted inline controller is 66,841 bytes / SHA-256
`bda0a0fa30db096d92404de22dd5193c91d111c3890e30fe43034405abdd2369`;
its bootstrap string is 5,111 bytes / SHA-256
`a8f7df1a172e1dc89672d2be54fa147eabf6a61864935e82fd86e2cf64839d33`.
These are complete source-string identities, not executed controller/helper
or produced runtime artifact identities.

### Bounded parsing and fallback contract

The new parser accepts only exact `bytes` captures. Each capture is
bounded at 8 MiB before parsing; there are at most 1,000,000 unique rows per
capture. Every row must end with LF and have exactly two canonical lowercase,
even-length hex fields separated by `|`. Key hex is 2–8,192 bytes
(1–4,096 decoded bytes); value hex is 0–2,097,152 bytes (at most 1 MiB decoded).
Keys must be valid UTF-8, have a nonempty collection before `/`, have
no NUL, and be unique. Blank, truncated, CRLF, odd/invalid/uppercase hex,
extra separators, duplicate key, wrong input type or oversize input/key/value
omits the optional diagnostic. Empty raw captures are supported by this pure
parser; the unchanged live `refusal_rows` still requires a nonempty
successful capture.

Hex values are validated and retained only in memory as complete canonical
hex, without decoding JSON or decrypting/normalizing stored values. Equality
of canonical value hex is equality of the full encoded Store value. The
original raw snapshot byte equality remains the authority: even a formatting
difference that projects to zero row changes is still a failure. The parser
neither serializes nor logs its internal keys/hex values.

Six categories and five count fields are exactly the reviewed schema.
`protected_or_other` counts every remaining Store row; an unknown
Store collection is counted there and does not become a new public label.
In contrast, any unknown/missing diagnostic category/field is invalid.
Every count is an exact built-in integer (not bool/subclass/float/string),
0–1,000,000. Consumers copy only the thirty accepted integers under fixed
labels. No keys/digests/values, URI, subject, timestamps, private paths,
arbitrary namespace names or raw error messages enter the optional packet.

Only an exact built-in `AssertionError` from the full equality check
gets an attempted attachment. Any parser/projection exception, including
interrupt/memory failures during optional observation, leaves that same
original error to be re-raised. Other assertions/subclasses receive no counts.
The bootstrap independently validates the attachment before serialization,
keeps its original fixed class/frames payload ready, and replaces it only if
the projected packet is valid and <=2,048 bytes. Failure retains the original
fixed packet and exception. Existing 64 examined / 8 trusted frame bounds,
exclusive 0600/O_NOFOLLOW output creation and original re-raise stay intact.

Controller reads at most 2,049 bytes after the existing regular-file/size
check; >2,048 bytes is refused. A local JSON pairs hook detects duplicate keys
anywhere in the packet without retaining their names in diagnostics. Duplicate
or unknown keys, invalid optional counts, wrong exception class or a projection
failure suppress only the optional counts when the original class/frames
remain valid. The controller reconstructs that fixed base packet explicitly.
Invalid/unreadable base JSON, invalid fixed frames/class or an oversized
packet cannot safely yield a recovered base packet: original phase_exit and
cleanup are preserved, and no unvalidated observation is published. The
existing observer file and four-file upload allowlist do not change.

Controller/global `read_json` is unchanged: this stricter, finite
reader is isolated to the failure observer. The helper fixed import digest
is prospectively updated to `4c60a7448d3c836215068fcc4f6822655c483f172a822e085f077d27b6a0c6fa` only; the other four
imports, product/artifact/build/validator roles, resource bounds, deadlines,
ownership, retention and finalization logic remain byte-exact.

### Exact zero-context helper patch

```diff
--- a/scripts/check-local-edition-transition-postgres.py
+++ b/scripts/check-local-edition-transition-postgres.py
@@ -132,0 +133,60 @@
+
+
+def refusal_snapshot_counts(before, after):
+    labels = ("http_rates", "http_rate_expiry", "http_rate_count",
+              "maintenance_cursors", "maintenance_bounds", "protected_or_other")
+    fields = ("before", "after", "added", "changed", "removed")
+
+    def read(raw):
+        if type(raw) is not bytes or len(raw) > 8 * 1024 ** 2 or raw and not raw.endswith(b"\n"):
+            raise ValueError("refusal_snapshot_shape")
+        result, cursor = {}, 0
+        while cursor < len(raw):
+            end = raw.find(b"\n", cursor)
+            split = raw.find(b"|", cursor, end)
+            key_size = split - cursor
+            value_size = end - split - 1
+            if (end < 0 or split < 0 or not 2 <= key_size <= 8192 or key_size % 2
+                    or not 0 <= value_size <= 2 * 1024 ** 2 or value_size % 2
+                    or len(result) >= 1_000_000):
+                raise ValueError("refusal_snapshot_shape")
+            key_hex, value_hex = raw[cursor:split], raw[split + 1:end]
+            if re.fullmatch(rb"[0-9a-f]+", key_hex) is None or re.fullmatch(rb"[0-9a-f]*", value_hex) is None:
+                raise ValueError("refusal_snapshot_shape")
+            key = bytes.fromhex(key_hex.decode("ascii"))
+            key.decode("utf-8")
+            bucket, separator, _ = key.partition(b"/")
+            if not bucket or not separator or b"\x00" in key or key in result:
+                raise ValueError("refusal_snapshot_shape")
+            result[key] = value_hex
+            cursor = end + 1
+        return result
+
+    def category(key):
+        if key.startswith(b"http_rates/"):
+            return "http_rates"
+        if key.startswith(b"index_expiry_http_rates/"):
+            return "http_rate_expiry"
+        if key == b"index_counts/http_rates":
+            return "http_rate_count"
+        if key.startswith(b"maintenance_cursors/"):
+            return "maintenance_cursors"
+        if key.startswith(b"maintenance_bounds/"):
+            return "maintenance_bounds"
+        return "protected_or_other"
+
+    before_rows, after_rows = read(before), read(after)
+    result = {label: {field: 0 for field in fields} for label in labels}
+    for key, value in before_rows.items():
+        counts = result[category(key)]
+        counts["before"] += 1
+        if key not in after_rows:
+            counts["removed"] += 1
+        elif after_rows[key] != value:
+            counts["changed"] += 1
+    for key in after_rows:
+        counts = result[category(key)]
+        counts["after"] += 1
+        if key not in before_rows:
+            counts["added"] += 1
+    return result
@@ -177,2 +237,11 @@
-        matrix.require(refusal_rows() == before_refusal,
-                       "refused user creation changed durable records")
+        after_refusal = refusal_rows()
+        try:
+            matrix.require(after_refusal == before_refusal,
+                           "refused user creation changed durable records")
+        except AssertionError as error:
+            if type(error) is AssertionError:
+                try:
+                    error.store_snapshot_counts = refusal_snapshot_counts(before_refusal, after_refusal)
+                except BaseException:
+                    pass  # Diagnostics must not replace the original complete-snapshot failure.
+            raise
```

### Exact zero-context workflow patch

```diff
--- a/.github/workflows/check-local-shared-handoff.yml
+++ b/.github/workflows/check-local-shared-handoff.yml
@@ -239 +239 @@
-              "check-local-edition-transition-postgres.py": "d86d9a99c9e09e29eb43af52f409332e11a70caaed90b5545bf132df1f2a1982",
+              "check-local-edition-transition-postgres.py": "4c60a7448d3c836215068fcc4f6822655c483f172a822e085f077d27b6a0c6fa",
@@ -314,0 +315,67 @@
+
+
+          def snapshot_counts(value):
+              labels = ("http_rates", "http_rate_expiry", "http_rate_count",
+                        "maintenance_cursors", "maintenance_bounds", "protected_or_other")
+              fields = ("before", "after", "added", "changed", "removed")
+              if (type(value) is not dict or len(value) != len(labels)
+                      or any(type(key) is not str for key in value) or set(value) != set(labels)):
+                  return None
+              result = {}
+              for label in labels:
+                  counts = value[label]
+                  if (type(counts) is not dict or len(counts) != len(fields)
+                          or any(type(key) is not str for key in counts) or set(counts) != set(fields)
+                          or any(type(counts[field]) is not int or not 0 <= counts[field] <= 1_000_000
+                                 for field in fields)):
+                      return None
+                  result[label] = {field: counts[field] for field in fields}
+              return result
+
+
+          def failure_packet(raw):
+              if type(raw) is not bytes or len(raw) > 2048:
+                  return None
+              duplicate = False
+
+              def pairs(items):
+                  nonlocal duplicate
+                  result = {}
+                  for key, value in items:
+                      if key in result:
+                          duplicate = True
+                      result[key] = value
+                  return result
+
+              try:
+                  value = json.loads(raw.decode("utf-8"), object_pairs_hook=pairs)
+              except (UnicodeError, ValueError, RecursionError):
+                  return None
+              classes = {"AssertionError", "RuntimeError", "ValueError", "TypeError", "KeyError",
+                         "IndexError", "NameError", "ImportError", "ModuleNotFoundError", "OSError",
+                         "FileNotFoundError", "PermissionError", "SyntaxError", "SystemExit",
+                         "KeyboardInterrupt", "Other"}
+              base_keys = {"exception_class", "frames"}
+              if (type(value) is not dict or not base_keys <= set(value)
+                      or type(value["exception_class"]) is not str or value["exception_class"] not in classes
+                      or type(value["frames"]) is not list or len(value["frames"]) > 8
+                      or not all(type(frame) is dict and set(frame) == {"file", "line"}
+                                 and type(frame["file"]) is str
+                                 and frame["file"] in set(FIXED["imports"]) | {"bootstrap.py"}
+                                 and type(frame["line"]) is int and 1 <= frame["line"] <= 4096
+                                 for frame in value["frames"])):
+                  return None
+              result = {"exception_class": value["exception_class"],
+                        "frames": [{"file": frame["file"], "line": frame["line"]} for frame in value["frames"]]}
+              if duplicate or set(value) != base_keys | {"store_snapshot_counts"} or value["exception_class"] != "AssertionError":
+                  return result
+              try:
+                  counts = snapshot_counts(value["store_snapshot_counts"])
+                  if counts is not None:
+                      projected = dict(result, store_snapshot_counts=counts)
+                      payload = (json.dumps(projected, sort_keys=True, separators=(",", ":")) + "\n").encode("ascii")
+                      if len(payload) <= 2048:
+                          result = projected
+              except BaseException:
+                  pass  # Retain only the original fixed class/frames if projection fails.
+              return result
@@ -538,0 +606,18 @@
+          def snapshot_counts(value):
+              labels = ("http_rates", "http_rate_expiry", "http_rate_count",
+                        "maintenance_cursors", "maintenance_bounds", "protected_or_other")
+              fields = ("before", "after", "added", "changed", "removed")
+              if (type(value) is not dict or len(value) != len(labels)
+                      or any(type(key) is not str for key in value) or set(value) != set(labels)):
+                  return None
+              result = {}
+              for label in labels:
+                  counts = value[label]
+                  if (type(counts) is not dict or len(counts) != len(fields)
+                          or any(type(key) is not str for key in counts) or set(counts) != set(fields)
+                          or any(type(counts[field]) is not int or not 0 <= counts[field] <= 1_000_000
+                                 for field in fields)):
+                      return None
+                  result[label] = {field: counts[field] for field in fields}
+              return result
+
@@ -562,0 +648,10 @@
+              if type(error) is AssertionError:
+                  try:
+                      counts = snapshot_counts(getattr(error, "store_snapshot_counts", None))
+                      if counts is not None:
+                          projected = dict(value, store_snapshot_counts=counts)
+                          optional = (json.dumps(projected, sort_keys=True, separators=(",", ":")) + "\n").encode("ascii")
+                          if len(optional) <= 2048:
+                              payload = optional
+                  except BaseException:
+                      pass  # Keep the original fixed class/frames and original failure.
@@ -971,12 +1066,5 @@
-                          value = read_json(PRIVATE / "helper-failure-source.json", 2048)
-                          classes = {"AssertionError", "RuntimeError", "ValueError", "TypeError", "KeyError",
-                                     "IndexError", "NameError", "ImportError", "ModuleNotFoundError", "OSError",
-                                     "FileNotFoundError", "PermissionError", "SyntaxError", "SystemExit",
-                                     "KeyboardInterrupt", "Other"}
-                          require(type(value) is dict and set(value) == {"exception_class", "frames"} and
-                                  type(value["exception_class"]) is str and value["exception_class"] in classes and
-                                  type(value["frames"]) is list and len(value["frames"]) <= 8 and
-                                  all(type(frame) is dict and set(frame) == {"file", "line"} and
-                                      type(frame["file"]) is str and frame["file"] in set(FIXED["imports"]) | {"bootstrap.py"} and
-                                      type(frame["line"]) is int and 1 <= frame["line"] <= 4096
-                                      for frame in value["frames"]), "helper_failure_source_shape")
+                          observer = PRIVATE / "helper-failure-source.json"
+                          require(regular(observer).st_size <= 2048, "json_size_limit")
+                          with observer.open("rb") as source:
+                              value = failure_packet(source.read(2049))
+                          require(value is not None, "helper_failure_source_shape")
```

### Complete new/changed executable function bodies

These are lexical source segments from the in-memory candidate ASTs, not
imports or executions. Nested functions are included in their full parent
body. The identical validator appears twice in actual candidate source,
once in the controller and once inside the bootstrap; its complete body is
shown once below. No executable body is omitted from the changed-function
inventory.

#### Helper: refusal_snapshot_counts (including read/category)

```python
def refusal_snapshot_counts(before, after):
    labels = ("http_rates", "http_rate_expiry", "http_rate_count",
              "maintenance_cursors", "maintenance_bounds", "protected_or_other")
    fields = ("before", "after", "added", "changed", "removed")

    def read(raw):
        if type(raw) is not bytes or len(raw) > 8 * 1024 ** 2 or raw and not raw.endswith(b"\n"):
            raise ValueError("refusal_snapshot_shape")
        result, cursor = {}, 0
        while cursor < len(raw):
            end = raw.find(b"\n", cursor)
            split = raw.find(b"|", cursor, end)
            key_size = split - cursor
            value_size = end - split - 1
            if (end < 0 or split < 0 or not 2 <= key_size <= 8192 or key_size % 2
                    or not 0 <= value_size <= 2 * 1024 ** 2 or value_size % 2
                    or len(result) >= 1_000_000):
                raise ValueError("refusal_snapshot_shape")
            key_hex, value_hex = raw[cursor:split], raw[split + 1:end]
            if re.fullmatch(rb"[0-9a-f]+", key_hex) is None or re.fullmatch(rb"[0-9a-f]*", value_hex) is None:
                raise ValueError("refusal_snapshot_shape")
            key = bytes.fromhex(key_hex.decode("ascii"))
            key.decode("utf-8")
            bucket, separator, _ = key.partition(b"/")
            if not bucket or not separator or b"\x00" in key or key in result:
                raise ValueError("refusal_snapshot_shape")
            result[key] = value_hex
            cursor = end + 1
        return result

    def category(key):
        if key.startswith(b"http_rates/"):
            return "http_rates"
        if key.startswith(b"index_expiry_http_rates/"):
            return "http_rate_expiry"
        if key == b"index_counts/http_rates":
            return "http_rate_count"
        if key.startswith(b"maintenance_cursors/"):
            return "maintenance_cursors"
        if key.startswith(b"maintenance_bounds/"):
            return "maintenance_bounds"
        return "protected_or_other"

    before_rows, after_rows = read(before), read(after)
    result = {label: {field: 0 for field in fields} for label in labels}
    for key, value in before_rows.items():
        counts = result[category(key)]
        counts["before"] += 1
        if key not in after_rows:
            counts["removed"] += 1
        elif after_rows[key] != value:
            counts["changed"] += 1
    for key in after_rows:
        counts = result[category(key)]
        counts["after"] += 1
        if key not in before_rows:
            counts["added"] += 1
    return result
```

#### Helper: complete shared_probe

```python
def shared_probe(server, config, base, scratch, revoked_token=None):
    session = scratch / f"delegate-{time.monotonic_ns()}.json"
    admin = scratch / f"admin-{time.monotonic_ns()}.json"
    with gate.serving(server, config, base, scratch / f"shared-{time.monotonic_ns()}.log"):
        if revoked_token is not None:
            matrix.require(authenticated_status(base, "/api/me", revoked_token) == 401,
                           "previously logged-out session became valid")
        gate.remote(server, base, session, "login", "delegate", "--password-stdin",
                    input="q08-delegate-disposable-password\n")
        me = gate.remote(server, base, session, "whoami")
        matrix.require(me["user"]["username"] == "delegate" and not me["user"]["admin"],
                       "ordinary credential did not identify the same non-admin user")
        token = json.loads(session.read_text())["token"]
        matrix.require(authenticated_status(base, "/api/audit?limit=1", token) == 200,
                       "active auditor grant stopped authorizing audit read")
        # Collection reads filter visible users; creation is the administration boundary.
        connection = config.parent / tomllib.loads(config.read_text())["postgres"]["connection_file"]
        target = re.fullmatch(r"host=127\.0\.0\.1 port=([0-9]+) dbname=riauth_transition "
                              r"user=riauth_test sslmode=disable", connection.read_text().strip())
        matrix.require(target is not None and 0 < int(target[1]) <= 65535,
                       "unexpected user-create refusal snapshot target")

        def refusal_rows():
            result = subprocess.run([
                "psql", "-h", "127.0.0.1", "-p", target[1], "-U", "riauth_test", "-d", "riauth_transition",
                "-X", "--no-password", "-v", "ON_ERROR_STOP=1", "-At", "-F", "|", "-c",
                "SELECT encode(key,'hex'),encode(value,'hex') FROM riauth_store.records_v1 ORDER BY key",
            ], capture_output=True, timeout=5)
            matrix.require(result.returncode == 0 and result.stdout,
                           "full user-create refusal snapshot unavailable")
            return result.stdout

        revision = gate.remote(server, base, session, "revision")["revision"]
        before_refusal = refusal_rows()
        denied = gate.remote(server, base, session, "--if-revision", revision,
                             "--idempotency-key", os.urandom(16).hex(),
                             "user", "create", "shared-refused-user", "--password-stdin",
                             input="q08-refused-disposable-password\n", expected=4)
        matrix.require(denied["error"]["http_status"] == 403
                       and denied["error"]["code"] == "access_denied"
                       and denied["exit_code"] == 4,
                       "ordinary auditor gained user administration")
        after_refusal = refusal_rows()
        try:
            matrix.require(after_refusal == before_refusal,
                           "refused user creation changed durable records")
        except AssertionError as error:
            if type(error) is AssertionError:
                try:
                    error.store_snapshot_counts = refusal_snapshot_counts(before_refusal, after_refusal)
                except BaseException:
                    pass  # Diagnostics must not replace the original complete-snapshot failure.
            raise
        time.sleep(0.2)
        again = gate.remote(server, base, session, "whoami")
        matrix.require(again["expires_at"] == me["expires_at"], "session expiry was renewed")
        gate.remote(server, base, session, "logout")
        matrix.require(authenticated_status(base, "/api/me", token) == 401,
                       "logout failed to revoke the session")
        gate.remote(server, base, admin, "login", "admin", "--password-stdin",
                    input="q08-disposable-password\n")
        grants = gate.remote(server, base, admin, "grants", "get", "delegate")
        matrix.require(grants["grants"] == [{"role": "auditor", "scope": "audit/events",
                                            "target_id": "events"}], "auditor grant changed")
        group = gate.remote(server, base, admin, "get", "group", "shared-fixture")
        matrix.require(me["groups"], "ordinary membership missing")
        gate.remote(server, base, admin, "logout")
        return {"user": me["user"], "groups": me["groups"], "group": group,
                "grants": grants, "audit_status": 200, "users_status": 403}, token
```

#### Inline controller AND bootstrap: snapshot_counts (identical complete body)

```python
def snapshot_counts(value):
    labels = ("http_rates", "http_rate_expiry", "http_rate_count",
              "maintenance_cursors", "maintenance_bounds", "protected_or_other")
    fields = ("before", "after", "added", "changed", "removed")
    if (type(value) is not dict or len(value) != len(labels)
            or any(type(key) is not str for key in value) or set(value) != set(labels)):
        return None
    result = {}
    for label in labels:
        counts = value[label]
        if (type(counts) is not dict or len(counts) != len(fields)
                or any(type(key) is not str for key in counts) or set(counts) != set(fields)
                or any(type(counts[field]) is not int or not 0 <= counts[field] <= 1_000_000
                       for field in fields)):
            return None
        result[label] = {field: counts[field] for field in fields}
    return result
```

#### Inline controller: failure_packet (including pairs)

```python
def failure_packet(raw):
    if type(raw) is not bytes or len(raw) > 2048:
        return None
    duplicate = False

    def pairs(items):
        nonlocal duplicate
        result = {}
        for key, value in items:
            if key in result:
                duplicate = True
            result[key] = value
        return result

    try:
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=pairs)
    except (UnicodeError, ValueError, RecursionError):
        return None
    classes = {"AssertionError", "RuntimeError", "ValueError", "TypeError", "KeyError",
               "IndexError", "NameError", "ImportError", "ModuleNotFoundError", "OSError",
               "FileNotFoundError", "PermissionError", "SyntaxError", "SystemExit",
               "KeyboardInterrupt", "Other"}
    base_keys = {"exception_class", "frames"}
    if (type(value) is not dict or not base_keys <= set(value)
            or type(value["exception_class"]) is not str or value["exception_class"] not in classes
            or type(value["frames"]) is not list or len(value["frames"]) > 8
            or not all(type(frame) is dict and set(frame) == {"file", "line"}
                       and type(frame["file"]) is str
                       and frame["file"] in set(FIXED["imports"]) | {"bootstrap.py"}
                       and type(frame["line"]) is int and 1 <= frame["line"] <= 4096
                       for frame in value["frames"])):
        return None
    result = {"exception_class": value["exception_class"],
              "frames": [{"file": frame["file"], "line": frame["line"]} for frame in value["frames"]]}
    if duplicate or set(value) != base_keys | {"store_snapshot_counts"} or value["exception_class"] != "AssertionError":
        return result
    try:
        counts = snapshot_counts(value["store_snapshot_counts"])
        if counts is not None:
            projected = dict(result, store_snapshot_counts=counts)
            payload = (json.dumps(projected, sort_keys=True, separators=(",", ":")) + "\n").encode("ascii")
            if len(payload) <= 2048:
                result = projected
    except BaseException:
        pass  # Retain only the original fixed class/frames if projection fails.
    return result
```

#### Bootstrap: complete failure_source

```python
def failure_source(error):
    names = ("check-local-edition-transition-postgres.py", "check-exact-edition-matrix.py",
             "check-installed-release-gate.py", "check-local-encrypted-edition-transition.py",
             "spdx_sbom.py")
    trusted = {str(root / "bootstrap.py"): "bootstrap.py",
               str(helper): "check-local-edition-transition-postgres.py"}
    trusted.update({str(helper.resolve().parent / name): name for name in names})
    classes = {AssertionError: "AssertionError", RuntimeError: "RuntimeError",
               ValueError: "ValueError", TypeError: "TypeError", KeyError: "KeyError",
               IndexError: "IndexError", NameError: "NameError", ImportError: "ImportError",
               ModuleNotFoundError: "ModuleNotFoundError", OSError: "OSError",
               FileNotFoundError: "FileNotFoundError", PermissionError: "PermissionError",
               SyntaxError: "SyntaxError", SystemExit: "SystemExit",
               KeyboardInterrupt: "KeyboardInterrupt"}
    frames, examined, tb = [], 0, error.__traceback__
    while tb is not None and examined < 64:
        name = trusted.get(tb.tb_frame.f_code.co_filename)
        line = tb.tb_lineno
        if name is not None and type(line) is int and 1 <= line <= 4096:
            frames.append({"file": name, "line": line})
            frames = frames[-8:]
        tb, examined = tb.tb_next, examined + 1
    value = {"exception_class": classes.get(type(error), "Other"), "frames": frames}
    payload = (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode("ascii")
    if type(error) is AssertionError:
        try:
            counts = snapshot_counts(getattr(error, "store_snapshot_counts", None))
            if counts is not None:
                projected = dict(value, store_snapshot_counts=counts)
                optional = (json.dumps(projected, sort_keys=True, separators=(",", ":")) + "\n").encode("ascii")
                if len(optional) <= 2048:
                    payload = optional
        except BaseException:
            pass  # Keep the original fixed class/frames and original failure.
    if len(payload) <= 2048:
        fd = os.open(private / "helper-failure-source.json",
                     os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
        with os.fdopen(fd, "wb") as out:
            out.write(payload)
```

#### Inline controller: complete Controller.helper

```python
def helper(self, installed, env):
        self.guard()
        require(self.deadline - time.monotonic() >= 660, "full_helper_budget_unavailable")
        bootstrap = ROOT / "bootstrap.py"
        with bootstrap.open("x") as out:
            out.write(BOOTSTRAP)
        bootstrap.chmod(0o600)
        env["A09_ROOT"] = str(ROOT)
        helper = pathlib.Path(os.environ["GITHUB_WORKSPACE"]) / "scripts" / HELPER
        try:
            self.command("focused-shared-helper",
                         [sys.executable, "-I", "-B", str(bootstrap), str(helper), str(installed),
                          FIXED["imports"][HELPER], FIXED["product_sha"]], 600, env)
        except Refusal:
            try:
                observer = PRIVATE / "helper-failure-source.json"
                require(regular(observer).st_size <= 2048, "json_size_limit")
                with observer.open("rb") as source:
                    value = failure_packet(source.read(2049))
                require(value is not None, "helper_failure_source_shape")
                self.data["helper_failure_source"] = value
                self.save()
            except Exception:
                pass  # Missing or malformed diagnostics preserve phase_exit and cleanup.
            raise
        report = read_json(PRIVATE / "helper.json", 64 * 1024)
        booleans = ("other_client_refused", "issuer_and_authentication_preserved", "all_effective_rates_preserved",
                    "active_capabilities_switched", "edition_and_version_metadata_coordinated")
        literals = {"schema": "riauth.local-native-postgres-transition/v1", "release_gate_result": False,
                    "architecture": "linux/aarch64", "backend": "postgresql", "agreement_format": 3,
                    "source_revision": FIXED["product_sha"], "validator_sha256": FIXED["imports"][HELPER],
                    "shared_configuration_refusals": ["authentication", "general_rate"],
                    "shared_identity_authorization_sample": "passed", "full_shared_gate": "not_certified",
                    "editions": ["essentials", "platform", "essentials"]}
        counts = ("baseline_rows", "upgrade_preserved_rows", "downgrade_preserved_rows")
        expected_keys = set(literals) | set(booleans) | set(counts) | {"postgres_version", "binary_sha256", "shared_sample_sha256"}
        require(set(report) == expected_keys and all(report.get(key) == value and type(report[key]) is type(value)
                                                   for key, value in literals.items()), "helper_report_literals")
        require(all(report[key] is True for key in booleans) and
                all(type(report[key]) is int and report[key] > 0 for key in counts) and
                report["baseline_rows"] == report["upgrade_preserved_rows"] and
                report["downgrade_preserved_rows"] >= report["baseline_rows"] and
                isinstance(report["shared_sample_sha256"], str) and
                re.fullmatch("[0-9a-f]{64}", report["shared_sample_sha256"]) is not None, "helper_report_assertions")
        binary_hashes = {edition: {product["binary"]: product["binary_sha256"] for product in FIXED["products"]
                                  if product["edition"] == edition}
                         for edition in ("essentials", "platform")}
        require(report["binary_sha256"] == binary_hashes, "helper_binary_hashes")
        require(type(report["postgres_version"]) is str and
                report["postgres_version"] == (PRIVATE / "postgres-version-psql.stdout").read_text().strip(),
                "helper_postgres_version")
        sanitized = {key: report[key] for key in sorted(expected_keys - {"postgres_version"})}
        sanitized["postgres_version"] = self.data["postgres_versions"]["psql"]["version"]
        sanitized["raw_postgres_version_output_sha256"] = self.data["postgres_versions"]["psql"]["output_sha256"]
        sanitized["helper_evidence_sha256"] = digest(PRIVATE / "helper.json")
        atomic(PUBLIC / "helper-redacted.json", sanitized)
        for name, expected in FIXED["imports"].items():
            self.guard()
            path = pathlib.Path(os.environ["GITHUB_WORKSPACE"]) / "scripts" / name
            regular(path)
            require(digest(path) == expected, "validator_import_changed_during_helper")
        self.data["sample_result"] = "passed"
        self.save()
```


### Proposed one stdlib, synthetic-only memory validation envelope

This is a proposed later reservation, NOT an executed harness or runtime pass.
One isolated Python 3.11+ process, using only ast/copy/json/re/hashlib/difflib
and inert pathlib paths, receives the immutable baseline and candidate text
plus both archived patches. Use -I -B; no candidate module/helper/bootstrap
import, module top-level evaluation, sys.argv entry point or Controller
construction. An outer 30-second timeout and one fixed pass are proposed;
no automatic retry. Synthetic captures together should peak below 32 MiB;
reserve a conservative 128 MiB process envelope for source/AST/stdlib objects,
but actual process memory/time has not been measured.

Static phase of that future envelope: verify the exact identities above,
apply/reverse both zero-context patches entirely in memory, and compare full
bytes and normalized ASTs to the baseline. Reject every changed protected
body/map entry. A whole helper/workflow compile or import is not needed.

Dynamic phase, only if root separately authorizes it:

* Extract only refusal_snapshot_counts from the helper AST, and only the
  pure snapshot_counts/failure_packet definitions from the controller AST.
  Execute those isolated definitions with explicit stdlib globals (re/json
  and the fixed import-name map); imports, source load(), subprocess, filesystem,
  environment and networking globals are absent. Compare the bootstrap and
  controller validator ASTs exactly before using the one definition.
* For producer fallback, extract only failure_source's in-memory prefix
  ending before its file-write conditional; add a return of payload to that
  synthetic AST copy. Supply inert root/helper paths and trusted filename
  fixtures. No os/open/write block, runpy, tempfile adapter or bootstrap
  module is executed. This measures serialization/fallback, not exclusive-file
  creation, native cleanup or the hosted gate.
* For same-error preservation, extract just the new equality try/except
  statement from shared_probe, supplying fixed matrix.require and parser
  stubs plus synthetic before/after bytes. Assert identity of the raised
  original AssertionError, unchanged message, no counts on subclasses,
  and fallback when the parser raises ValueError/MemoryError/KeyboardInterrupt.
  Do not execute the surrounding probe, CLI calls, captures or sleep.

Minimum meaningful synthetic cases and independently specified expectations:

| Area | Inputs / required observation |
| --- | --- |
| Counts | Six baseline keys, one per category, all value A. After: each retained key changes to B except protected key removed/replaced by a different protected key, plus one new http_rates key. Expected http_rates = (before 1, after 2, added 1, changed 1, removed 0); four middle categories each (1,1,0,1,0); protected_or_other = (1,1,1,0,1). Sums before 6 / after 7; complete captures remain unequal. |
| Exact category boundaries | index_counts/http_rates is the only count-row category. index_counts/mail_limits, index_counts/http_rates-extra, http_rates2/x, maintenance_cursors-extra/x and an unadvertised valid UTF-8 bucket all remain protected_or_other. No dynamic label is emitted. |
| Full opaque values | Change one byte in a synthetic non-JSON value and a synthetic ciphertext-like value: each is changed. Identical snapshots produce only equal before/after counts and zeros. Formatting-only valid row reorder projects no value changes, but the unchanged raw equality still raises. |
| Parser failures | Wrong raw type; trailing partial line; blank/CRLF lines; odd/nonhex/uppercase fields; extra separator; duplicate canonical key; invalid UTF-8/NUL/no collection separator; input 8 MiB+1; decoded key 4,097 bytes; value 1 MiB+1. Each must omit projection via the wrapper and preserve the original error. |
| Boundaries | Valid key at 4,096 bytes, value at 1 MiB, empty value and empty pure-parser snapshot are bounded valid inputs. Validate integer 0 and 1,000,000; reject -1, 1,000,001, bool, int subclass, float, string, None and dict subclass. No need to allocate one million records to test schema bounds. |
| Unknown/duplicate schema | Missing/extra category or field suppresses counts. Literal duplicate JSON category/field/root key suppresses counts while valid fixed class/frames survive. Unknown extra root field suppresses counts and is stripped, not published. |
| Packet fallback | Valid AssertionError gets counts; RuntimeError, AssertionError subclass and missing/bad attachment do not. Invalid base class/frame/file/line returns no packet. >2,048-byte raw packet returns no packet. Invalid UTF-8, malformed/deep JSON yields no packet and no raw content. |
| Privacy / transport cap | Put distinctive synthetic key/value/URI/path/message sentinels in captures and invalid optional JSON. Parsed public packets contain only fixed classes/files/labels/field names and numeric line/counts. Inspect structure and serialized bytes; none of the sentinels or hex forms may appear. Verify all thirty counts at 1,000,000 and eight longest fixed frame names at 4096 fit 2,048 bytes. |
| Failure identity | The original complete-equality AssertionError remains the raised object with the unchanged fixed message whether counts succeed or fail. A valid projection never changes equality outcome or permits further probe actions. |

A later harness may report only fixed case labels/pass totals/time/exit and
sanitized assertion identifiers; never repr of snapshots, schema inputs,
captured keys/values or candidate exception messages. No actual Store data,
provider/PG/HTTP fixture, native binary, release/tenant evidence or new success
claim belongs in that envelope. Hosted runtime and source materialization
still require separate root decisions.

### Static design checks actually performed

Only source-text operations, hashes, ast.parse/literal_eval, exact in-memory
substitutions and zero-context forward/reverse application were executed.
No code object for the proposed functions was compiled or executed.

* Full forward and reverse zero-context patch checks restore both complete
  baseline files, not merely changed functions. The current disk source
  remains exactly the immutable baseline.
* Helper AST restoration removes only the new refusal_snapshot_counts and
  restores shared_probe, yielding the entire baseline AST. Imports/constants/
  all other functions are identical. The original SQL/capture function,
  public refusal assertions, all later probe actions and return remain
  byte-exact outside the new comparison/attachment block.
* Controller AST restoration removes only snapshot_counts/failure_packet,
  restores FIXED/BOOTSTRAP literals and Controller.helper, yielding the full
  baseline controller AST. Bootstrap restoration removes its validator and
  restores failure_source, yielding its full baseline AST.
* Both validator ASTs are identical. After normalizing only the helper digest,
  complete FIXED maps are equal. All four other imported source files and
  hashes match own disk and immutable f7af6cc objects, byte for byte.
* Complete changed bodies are archived above; all successful helper report
  validation/output and Controller.helper suffix are unchanged. No product,
  source role, resource/ownership/finalization or cleanup function is altered.

The first design-assembly tool request failed JavaScript parsing with
Unexpected token if before any shell or filesystem operation ran. Correcting
only that request text permitted this static construction. It was not a
candidate parser/function/harness failure, and no source edit or borrowed
runtime result followed it.

The earlier source finding, all remote failures and unknown changed-row cause
are preserved. This exact design is not implementation or runtime evidence.
Original A09 shared full gate stays open; I02/I10/R05/W02/W05 remain DONE.
Root owns later source reservation, independent full-body review, publication,
distinct validator-source selection, memory validation and hosted release.

Archive-only verification independently re-extracted the two diff fences
and six complete function fences from this report. It reconstructed the
candidate files, verified all exact file/patch/blob/controller/bootstrap
identities, reversed both patches to full baseline bytes, and matched every
function fence to its candidate lexical body. Both validator bodies are
also byte-equal after extracting their source contexts. The complete
44,904-byte previous report prefix is unchanged (SHA-256
`e8077e7f985ff88d3db10e8b3faaf7385ba729c43d64bcf147b18048126882de`).
The isolated pure-function ASTs contain no imports or I/O/runtime names;
this is a static source witness, not sandbox enforcement or dynamic privacy
validation. `python3 scripts/check-docs.py` and `git diff --check` both exited
0. Own helper/workflow remain the full immutable baseline; this report is
the only changed file. No prospective function or synthetic case was run.

## Exact source materialization and static evidence, 2026-10-02

Reservation: `wave30_A09_snapshot_counts_source_materialization`, project
`891e7443-8dac-4c1b-897f-9e53cb59c7ee`, original A09 support in existing
worktree `a1303b57-4a34-487e-9c63-a841f05b51a0`. Root approved only the two
exact design diffs above and a separate evidence append. No design adjustment,
alignment, merge/reset, whole-file import or other source change was made.
The complete 89,229-byte `eae9415980bc5826ae8dc74b07af0e1ca8e62122` report
prefix remains preserved, SHA-256
`210bd44af62b2ceaac3d162cb10b1852e58473071487dd300625b826563835ad`.
The earlier statements are evidence of their dated phases, including the
unexecuted synthetic-validation proposal; they are not rewritten as results.

Actual source commit:
`d00c9680004be856c387e4faf03c14c1e62d6b0c`, parent
`eae9415980bc5826ae8dc74b07af0e1ca8e62122`. It contains only these two
files, 172 insertions / 15 deletions; both retain `100644` mode:

| Committed source | Bytes | SHA-256 | Actual Git blob |
| --- | --- | --- | --- |
| scripts/check-local-edition-transition-postgres.py | 26,547 | `4c60a7448d3c836215068fcc4f6822655c483f172a822e085f077d27b6a0c6fa` | `af764b71b95f8ad4d5e1d9b26d094dfa2eaafe49` |
| .github/workflows/check-local-shared-handoff.yml | 82,149 | `226483dd9edcd5e566b6c5de9f7ab2a36ebdf0c0f204f8ec992645542a3f395f` | `f63110098c13d717446a198c487c00678cfeb60a` |

Before editing, own clean HEAD/report and both complete source files were
verified against immutable `f7af6cc85b938738e6152b4b0e5109e36a3abdec`:
helper 23,518 bytes / `d86d9a99...`; workflow 77,543 bytes / `5290d7a5...`.
Both exact zero-context patches were extracted from the immutable `eae9415`
report fences, including their final newlines. `git apply --unidiff-zero
--check -` then `git apply --unidiff-zero -` each exited 0. Complete resulting
file lengths/hashes equal the reviewed candidates, with no substitution or
context variation. No contrary source fact or new dependency was found.

The exact materialized source keeps complete raw snapshot equality and the
original fixed refusal assertion/message. Only an exact `AssertionError`
from that comparison gets an attempted optional counts attachment; projection
failure re-raises the original error. The source retains public CLI 4 /
HTTP 403 / `access_denied`, fresh revision/key, all capture SQL/timeout/
connection guards and all later actions/return data. The 8 MiB per-capture
bound limits optional diagnostic parsing; it does not truncate the existing
captures or filter the complete comparator. The six categories / thirty
exact integer fields, duplicate/unknown-key fallback and finite observer
2,048-byte cap match the reviewed source design without adjustment.

Static checks actually run in this materialization phase all passed:

* `git apply --unidiff-zero --reverse --check -` exited 0. Independent
  in-memory reversal of both complete edited files restored every baseline
  byte. Fresh actual zero-context diffs equal both archived patches exactly.
* `ast.parse` of the complete helper, extracted complete inline controller
  and complete bootstrap succeeded. All six changed/new function segments
  equal their full archived bodies. Controller/bootstrap validator bodies
  are byte-equal. No candidate code object/function or module was executed.
* Full helper AST restoration permits only the new parser and changed
  `shared_probe`; full controller restoration permits only the two pure
  functions, `FIXED`/bootstrap literals and `Controller.helper`; full
  bootstrap restoration permits only its validator and `failure_source`.
  All restored ASTs equal the complete baseline ASTs. Fresh byte witnesses
  preserve the original capture/authority section, public refusal assertions,
  probe suffix and complete successful helper-report validation/output suffix.
* Only the helper digest differs in the complete decoded `FIXED` map. Every
  other value/role and all four other import hashes/source bytes are unchanged:
  matrix `f07d934f9cfd7af20eb086f5838863c28f840ee848e62bea1d865b330643d887`;
  gate `cbe8dcb42b4b1c36dc8df00204103b9c955feb8057275a441f60f3072d2bf8b5`;
  encrypted fixture `09e137d88eb8e873a488448b7bdfdbe80d2327fbca716a93f445eaaae1ea9e8e`;
  SPDX `ca063ab3d4abb6ec815151bcf762447e96682076a7f72d2dbfa9d7e6d3a1032c`.
* The complete actual inline controller is 66,841 bytes / SHA-256
  `bda0a0fa30db096d92404de22dd5193c91d111c3890e30fe43034405abdd2369`;
  its bootstrap source string is 5,111 bytes / SHA-256
  `a8f7df1a172e1dc89672d2be54fa147eabf6a61864935e82fd86e2cf64839d33`.
  These are source identities, not runtime artifact receipts.
* Installed Ruby `Psych.safe_load` parsed the full workflow with no permitted
  classes/symbols or aliases. Its three extracted run blocks each passed
  `bash -n` (exit 0): owned preflight/supervisor, transport/sample and
  cleanup/finalization. No shell run block/controller/bootstrap was executed.
* The complete sequence of GitHub context expressions is byte-equal to the
  baseline. Workflow bytes before/after the embedded controller are unchanged;
  the new function/digest hunks remain in the same existing run-block context.
  Product/artifact/build/validator boundaries, actions/permissions, resource
  limits, sampling/deadlines, owned PID/cluster cleanup and fixed uploads
  remain protected. No new context reference required an external query.
* Static AST hygiene of the three isolated pure functions found no imports
  or I/O/runtime names. This is a source witness, not executed privacy/schema
  enforcement, memory measurement or a sandbox claim.
* `python3 scripts/check-docs.py`, `git diff --check`, and source-stage
  `git diff --cached --check` exited 0. Staged scope was exactly the two
  reserved files. After commit, immutable file bytes matched disk and the
  reviewed identities, commit scope was exactly those paths, branch was
  clean, and the complete prior report was unchanged before this append.

No new static check failed in this source phase. All earlier remote failures,
source/design assembly observations and historical measurements remain intact.
Hosted run `37060776569` still supplies only successful strict public refusal
followed by full Store byte inequality; its changed rows and writer origin
remain UNKNOWN. Materializing optional counts does not establish that rate
bookkeeping was the actual cause or that the shared gate has passed.

This source phase stops at immutable source/static evidence. No memory
validation design adjustment, candidate function/harness/helper import,
bootstrap/controller/module main, native product, PG/HTTP/provider/Cargo,
container/dispatch/query/download/test, service/browser, secret row/private
log inspection, cleanup/deletion, other-worker contact, new task/worktree/
shell or main/push/status action occurred. Static Psych/bash parsing above
was syntax/data inspection only. No runtime slot was taken or released.
Next memory-validation design and any execution/hosted repeat require their
own root reservations. Root alone owns review/integration/publication and
distinct validator-source selection. Original A09 shared full gate remains
open; I02/I10/R05/W02/W05 stay DONE, with no status mutation here.


## Root implementation and independent source review, 2026-10-02

Root read95965eb108-line source receipt and verified both actuald00c968 file
identities equal the full reviewed eae candidates. Root read2c9e5ea entire254-line
independent review: no concrete source blocker. Counts-only diagnostics are
accepted as source; full raw Store comparison/public403/re-raised original
error remain authoritative. The separately reserved complete memory design
is pending; no counts or writer cause is inferred from37060776569 and no new
hosted shared-gate invocation is released.

## Exact memory-validation design archive, 2026-10-03

Project 891e7443-8dac-4c1b-897f-9e53cb59c7ee; original A09 task
506e3979-a590-4af3-8fa8-ee90d3a517f2; existing isolated worktree
a1303b57-4a34-487e-9c63-a841f05b51a0. Reservation:
wave30_A09_snapshot_counts_memory_design. This is an unexecuted exact future
memory-validation design, ready for independent source review. No child,
candidate case, source definition, bootstrap, helper module, controller,
native executable, database, HTTP request or hosted run was evaluated.

The source remains the reviewed d00c9680004be856c387e4faf03c14c1e62d6b0c tree.
This append preserves all 96,475 bytes of report 95965eb69f2a26477a81d6ed196131d8c5b976ca
(SHA256 190fa98b9fc23839bd83d9451293cf252256d3074d730fdf9617b3c7ec34104f).
No alignment or source edit is involved. I02/I10/R05/W02/W05 stay closed;
original A09/shared gate and root runtime decisions are unchanged.

### Source pins and executable archive identity

All six inputs below are read as bounded regular files with no-follow opens
by the future child. Their hashes must match before any isolated source
definition is executed. They are never imported as modules. The parent
recomputes these source hashes only after retaining actual child output and
exit. This ordering separates pre-execution source binding from post-outcome
grading; a source mismatch fails closed and cannot manufacture a pass.

| Input | Bytes | SHA256 |
| --- | ---: | --- |
| scripts/check-local-edition-transition-postgres.py | 26547 | 4c60a7448d3c836215068fcc4f6822655c483f172a822e085f077d27b6a0c6fa |
| .github/workflows/check-local-shared-handoff.yml | 82149 | 226483dd9edcd5e566b6c5de9f7ab2a36ebdf0c0f204f8ec992645542a3f395f |
| scripts/check-exact-edition-matrix.py | 18394 | f07d934f9cfd7af20eb086f5838863c28f840ee848e62bea1d865b330643d887 |
| scripts/check-installed-release-gate.py | 22321 | cbe8dcb42b4b1c36dc8df00204103b9c955feb8057275a441f60f3072d2bf8b5 |
| scripts/check-local-encrypted-edition-transition.py | 22045 | 09e137d88eb8e873a488448b7bdfdbe80d2327fbca716a93f445eaaae1ea9e8e |
| scripts/spdx_sbom.py | 36727 | ca063ab3d4abb6ec815151bcf762447e96682076a7f72d2dbfa9d7e6d3a1032c |

The workflow controller literal is 66,841 bytes, SHA256
bda0a0fa30db096d92404de22dd5193c91d111c3890e30fe43034405abdd2369;
its bootstrap literal is 5,111 bytes, SHA256
a8f7df1a172e1dc89672d2be54fa147eabf6a61864935e82fd86e2cf64839d33.
The source-created FIXED import map must equal the helper plus the same four
other imported-script pins. These hashes bind static source, not runtime results.

The three complete UTF-8/LF archives below each include their final newline.

| Future archive | Bytes | SHA256 |
| --- | ---: | --- |
| Child payload | 24494 | 1cd3442cb91d19de9c70134da763d072b9cc5ca317e2ed2bb633df2bb1c04181 |
| Controller | 12634 | 278060f3de48a133b017cffc188bf9adea200337e901370e97a1f1764ded44ee |
| Assembly command | 1427 | 084ed5ae9d4417b4f6fbce9af8d43eb810bc2f29e7c8dea378ed256512e66653 |

### Isolation and case design

The future child AST-selects only helper refusal_snapshot_counts, controller
snapshot_counts/failure_packet and the bootstrap snapshot_counts/failure_source
prefix. The prefix ends before the final os.open/private-file-write block;
only a return of its existing payload is added. The bootstrap is parsed as a
string, never imported. InertPath supplies joining/string/parent/resolve
semantics without filesystem operations. Controlled synthetic raises produce
genuine traceback chains bound to inert filenames. No surrounding shared_probe,
SQL, sleep, HTTP or CLI path is evaluated. The isolated equality try/except is
selected by its exact AssertionError handler; synthetic matrix.require stubs
retain and raise the same error object. Source module imports, entry points,
controller construction and bootstrap writer blocks remain outside the child.

The future executable scope is stdlib parsing, hashing and pure memory work,
plus the parent's single Python child and bounded private evidence writes.
The archive does not authorize its execution. The child rejects imports and
IO names within extracted definitions, uses an explicit builtin namespace,
and binds its six complete source files before definition execution.

Cases have actual group/name records and derived totals; there is no padded
case-count target or predetermined pass receipt. The proposed groups are:

- source_binding: six complete inputs, controller/bootstrap literal identity
  and unchanged five-script import map.
- counts: all six categories/five counts, exact namespace aliases, actual
  opaque non-JSON and ciphertext-like synthetic bytes, and unchanged full values.
- parser: malformed/duplicate/noncanonical hex, missing collection, invalid UTF-8,
  NUL keys, CRLF/partial records and wrong input types; valid 4,096-byte key,
  1 MiB value and exact 8 MiB four-row capture; individual over-limit refusals.
- schema: exact integer 0/1,000,000 boundaries and bool/int-subclass/float/string/null/
  negative/oversize rejection, missing/unknown categories and fields, dict subclass.
- packet: valid counts, maximal eight-frame packet with thirty 1,000,000 counts,
  exact 2,048-byte boundary and 2,049-byte refusal, duplicate keys, unknown root/
  frame keys, unknown fixed frame/class, non-integer/out-of-range frame lines,
  malformed/invalid-UTF-8/deep JSON; forced MemoryError/KeyboardInterrupt
  projection failure retains original fixed class/frames.
- producer: genuine trusted synthetic traceback, invalid/missing attachment,
  exact AssertionError-only attachment and subclass exclusion, bounded 64-frame
  examination/eight retained frames; forced MemoryError/KeyboardInterrupt
  projection failure retains original fixed class/frames.
- equality: byte-identical success; complete-change failure preserving error
  identity/message; reordered rows still fail byte equality although projected
  added/changed/removed counts are zero; parser/ValueError/MemoryError/
  KeyboardInterrupt fallback and AssertionError subclass exclusion.
- privacy: synthetic key/value/hex/path/URI/message sentinels never appear in
  projected fixed packets or public child/controller results. Input rows are
  synthetic only; no actual database/evidence row is read.

Projection remains optional. Counts are diagnostics, never authority for
snapshot equality: even a valid zero-change projection does not resolve a
reordered or otherwise byte-different complete snapshot. Controller fallback
retains only the original fixed class/frames; equality re-raises the original
AssertionError object. No key, row digest, raw value, URI, private path or
exception message is emitted by the child. A failed case exposes only its
fixed group/name/status and allowlisted exception class, never str/repr(error).

### Future command, limits and outcome retention

Only the exact archived assembly command is prospective. It extracts the
unique archived controller and verifies its hash before executing it. The
controller extracts and binds the exact child payload. It creates one new
0700 evidence directory under this worktree's existing private .target-wave27:
a09-snapshot-counts-memory-d00c968-wave30. Every evidence file uses exclusive
0600/no-follow creation and fsync; an existing directory causes refusal, not
reuse/deletion. Capacity below 8 GiB refuses this tiny memory/evidence run.
No capacity was measured or runtime slot acquired in this design phase.

One isolated Python 3.11+ child uses -I/-B/-c, DEVNULL stdin, private bounded
stdout (64 KiB) and stderr (8 KiB), a minimal fixed environment, and its own
process group. Child wall budget is at most 30 seconds including startup;
the payload also has a 30-second alarm/persistent budget flag. Outer elapsed
time begins before assembly extraction and has a 35-second alarm/acceptance
ceiling. A fresh setup must leave at least 30 seconds for the child.
Selectors sample at most every 50 ms and stop the exact unreaped owned process
group on interruption, cap or timeout. No broad PID discovery or pkill occurs.
An emergency bounded five-second wait is permitted solely to reap the owned
child after kill; any late completion fails the final 35-second acceptance
check rather than claiming bounded success.

At child exit, the controller first saves child-exit.json with the actual
numeric exit and reaping state, then the full bounded child.json and private
child-stderr.bin. These saves precede outcome/schema/expected-source grading.
Unexpected stderr is private failure evidence and is not published. The
parent rechecks source hashes only afterward and saves its grading receipt.
Setup refusals report no child/null exit, never a fabricated numeric result.
The final post-save monotonic clock follows every receipt fsync; no evidence
write follows that clock. Outer timeout/interruption/receipt failure rejects
the run. Final stdout is fixed JSON containing actual exit/reaping/save/time
and memory-only grading. It never emits arbitrary errors, paths or payloads.

No actual result exists yet. The prospective acceptance means isolated
synthetic diagnostic compatibility only, not native/shared gate success,
hosted compatibility, actual mutated-row diagnosis or provider evidence.

### Static preparation evidence and limits

The first archive-build tool request encountered a shell here-document
delimiter collision (zsh parse error near a closing parenthesis) before any
report write. A subsequent check confirmed the clean HEAD 95965eb and unchanged
96,475-byte report/source. This was an archive construction failure, not a
candidate case or runtime failure. The corrected static build uses a distinct
outer delimiter.

Static source review also replaced an ambiguous nested-Try AST selector with
the exact outer AssertionError-handler selector. Static inspection finds one
matching equality block and confirms the bootstrap prefix ends before its
os/private writer. No selector or candidate definition was executed.

The preparation parses the archived child/controller/assembly body into AST,
hashes complete UTF-8/LF source, verifies one future Popen site, extracts the
controller/bootstrap literals without evaluation, and compares the six pinned
source bytes to d00c968. It does not compile, exec, eval, import or run the
candidate payload/controller. These are static witnesses, not case passes.

All earlier source/runtime failures remain dated in the prefix, including
37060776569's passed CLI4/HTTP403/access_denied refusal followed by failed full
Store equality. Actual changed rows and origin remain unknown. Pure projection
validation cannot establish that HTTP bookkeeping caused that failure and
cannot establish the A09 shared gate. No hosted repeat is authorized.

### Full future child payload

```python
import time
STARTED = time.monotonic()
import ast, builtins, copy, hashlib, json, os, pathlib, re, signal, stat, sys

REPO = pathlib.Path("/Users/dominik/orca/projects/riAuth-public-preview-local-workflow-safety-wave27")
PINS = [('helper', 'scripts/check-local-edition-transition-postgres.py', 26547, '4c60a7448d3c836215068fcc4f6822655c483f172a822e085f077d27b6a0c6fa'), ('workflow', '.github/workflows/check-local-shared-handoff.yml', 82149, '226483dd9edcd5e566b6c5de9f7ab2a36ebdf0c0f204f8ec992645542a3f395f'), ('matrix', 'scripts/check-exact-edition-matrix.py', 18394, 'f07d934f9cfd7af20eb086f5838863c28f840ee848e62bea1d865b330643d887'), ('gate', 'scripts/check-installed-release-gate.py', 22321, 'cbe8dcb42b4b1c36dc8df00204103b9c955feb8057275a441f60f3072d2bf8b5'), ('encrypted', 'scripts/check-local-encrypted-edition-transition.py', 22045, '09e137d88eb8e873a488448b7bdfdbe80d2327fbca716a93f445eaaae1ea9e8e'), ('spdx', 'scripts/spdx_sbom.py', 36727, 'ca063ab3d4abb6ec815151bcf762447e96682076a7f72d2dbfa9d7e6d3a1032c')]
LABELS = ("http_rates", "http_rate_expiry", "http_rate_count",
          "maintenance_cursors", "maintenance_bounds", "protected_or_other")
FIELDS = ("before", "after", "added", "changed", "removed")
HELPER = "check-local-edition-transition-postgres.py"
FILES = (HELPER, "check-exact-edition-matrix.py", "check-installed-release-gate.py",
         "check-local-encrypted-edition-transition.py", "spdx_sbom.py", "bootstrap.py")
GROUPS = ("source_binding", "counts", "parser", "schema", "packet", "producer", "equality", "privacy")
MESSAGE = "refused user creation changed durable records"
SENTINELS = ("A09_PRIVATE_KEY_SENTINEL", "A09_PRIVATE_VALUE_SENTINEL",
             "https://a09-private-uri.invalid/secret", "/a09-private-path-sentinel",
             "A09_PRIVATE_MESSAGE_SENTINEL")
EXPIRED = False
CASES = []
OBSERVED = {}
STAGE = "initial"

class Budget(BaseException): pass
class Halt(Exception): pass

def alarm(signum, frame):
    global EXPIRED
    EXPIRED = True
    raise Budget()

def clock():
    if EXPIRED or time.monotonic() - STARTED >= 30: raise Budget()

def check(value):
    if not value: raise AssertionError()

def failure_class(error):
    return {AssertionError: "AssertionError", ValueError: "ValueError", TypeError: "TypeError",
            KeyError: "KeyError", MemoryError: "MemoryError", KeyboardInterrupt: "KeyboardInterrupt",
            Budget: "Budget"}.get(type(error), "Other")

def case(group, name, work):
    clock()
    status, error_class = "pass", "None"
    try: work()
    except BaseException as error: status, error_class = "fail", failure_class(error)
    if EXPIRED: status, error_class = "fail", "Budget"
    CASES.append({"group": group, "name": name, "status": status, "failure_class": error_class})
    clock()

def read_source(relative, size):
    fd = os.open(REPO / relative, os.O_RDONLY | os.O_NOFOLLOW)
    with os.fdopen(fd, "rb") as incoming:
        if not stat.S_ISREG(os.fstat(incoming.fileno()).st_mode): raise Halt()
        return incoming.read(size + 1)

def function(tree, name):
    nodes = [n for n in tree.body if type(n) is ast.FunctionDef and n.name == name]
    check(len(nodes) == 1)
    node = nodes[0]
    check(not node.decorator_list and not node.args.defaults and not node.args.kw_defaults
          and node.returns is None and all(a.annotation is None for a in node.args.args))
    return copy.deepcopy(node)

def assignment(tree, name):
    nodes = [n for n in tree.body if type(n) is ast.Assign
             and any(type(t) is ast.Name and t.id == name for t in n.targets)]
    check(len(nodes) == 1)
    return nodes[0]

def isolated(nodes, namespace, filename):
    for node in nodes:
        for part in ast.walk(node):
            check(type(part) not in (ast.Import, ast.ImportFrom))
            if type(part) is ast.Name:
                check(part.id not in ("os", "open", "subprocess", "runpy", "tempfile", "socket", "urllib", "print"))
    names = ("type", "len", "any", "set", "dict", "bytes", "str", "int", "getattr",
             "AssertionError", "BaseException", "ValueError", "UnicodeError", "RecursionError",
             "RuntimeError", "TypeError", "KeyError", "IndexError", "NameError", "ImportError",
             "ModuleNotFoundError", "OSError", "FileNotFoundError", "PermissionError", "SyntaxError",
             "SystemExit", "KeyboardInterrupt")
    namespace["__builtins__"] = {name: getattr(builtins, name) for name in names}
    module = ast.fix_missing_locations(ast.Module(body=nodes, type_ignores=[]))
    exec(compile(module, filename, "exec"), namespace)
    return namespace

class InertPath:
    def __init__(self, text): self.text = text
    def __str__(self): return self.text
    def __truediv__(self, name): return InertPath(self.text + "/" + name)
    def resolve(self): return self
    @property
    def parent(self): return InertPath(self.text.rsplit("/", 1)[0])

def snapshot(items):
    return b"".join(key.hex().encode("ascii") + b"|" + value.hex().encode("ascii") + b"\n"
                    for key, value in items)

def zeros():
    return {label: {field: 0 for field in FIELDS} for label in LABELS}

def base_packet():
    return {"exception_class": "AssertionError", "frames": [{"file": HELPER, "line": 1}]}

def wire(value):
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode("ascii")

def clean(value):
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":")).encode("ascii")
    for marker in SENTINELS:
        check(marker.encode() not in encoded and marker.encode().hex().encode() not in encoded)

def rejected_parser(counts, raw):
    try: counts(raw, b"")
    except BaseException as error:
        check(type(error) in (ValueError, UnicodeDecodeError))
        return
    raise AssertionError()

def main():
    global STAGE
    STAGE = "source_binding"
    texts = {}
    for name, relative, size, digest in PINS:
        raw = read_source(relative, size)
        OBSERVED[name] = hashlib.sha256(raw).hexdigest()
        matched = len(raw) == size and OBSERVED[name] == digest
        case("source_binding", name, lambda matched=matched: check(matched))
        if not matched: raise Halt()
        texts[name] = raw.decode("utf-8")
    helper = ast.parse(texts["helper"])
    workflow = texts["workflow"]
    marker = '          cat > "$A09_ROOT/controller.py" <<\'PY\'\n'
    begin = workflow.index(marker) + len(marker)
    end = workflow.index('          PY\n', begin)
    controller = "".join(line[10:] if line.startswith("          ") else line
                         for line in workflow[begin:end].splitlines(keepends=True))
    controller_tree = ast.parse(controller)
    bootstrap = ast.literal_eval(assignment(controller_tree, "BOOTSTRAP").value)
    bootstrap_tree = ast.parse(bootstrap)
    case("source_binding", "controller_string", lambda: check(
        hashlib.sha256(controller.encode()).hexdigest() == "bda0a0fa30db096d92404de22dd5193c91d111c3890e30fe43034405abdd2369"))
    case("source_binding", "bootstrap_string", lambda: check(
        hashlib.sha256(bootstrap.encode()).hexdigest() == "a8f7df1a172e1dc89672d2be54fa147eabf6a61864935e82fd86e2cf64839d33"))
    if any(item["status"] != "pass" for item in CASES): raise Halt()
    validator = function(controller_tree, "snapshot_counts")
    check(ast.dump(validator) == ast.dump(function(bootstrap_tree, "snapshot_counts")))
    fixed = json.loads(ast.literal_eval(assignment(controller_tree, "FIXED").value.args[0]))
    expected_imports = {pathlib.PurePosixPath(relative).name: digest
                        for name, relative, size, digest in PINS if name != "workflow"}
    case("source_binding", "import_map", lambda: check(fixed["imports"] == expected_imports))
    if any(item["status"] != "pass" for item in CASES): raise Halt()
    trace_root = InertPath(SENTINELS[3])
    trace_helper = trace_root / HELPER
    counts = isolated([function(helper, "refusal_snapshot_counts")], {"re": re},
                      str(trace_helper))["refusal_snapshot_counts"]
    packet_ns = isolated([validator, function(controller_tree, "failure_packet")],
                         {"json": json, "FIXED": {"imports": expected_imports}},
                         str(trace_root / "bootstrap.py"))
    validate, consume = packet_ns["snapshot_counts"], packet_ns["failure_packet"]
    producer_node = function(bootstrap_tree, "failure_source")
    writer = producer_node.body[-1]
    check(type(writer) is ast.If and any(type(n) is ast.Name and n.id == "os" for n in ast.walk(writer)))
    producer_node.body = producer_node.body[:-1] + [ast.Return(value=ast.Name(id="payload", ctx=ast.Load()))]
    producer = isolated([function(bootstrap_tree, "snapshot_counts"), producer_node],
                        {"json": json, "root": trace_root, "helper": trace_helper},
                        str(trace_root / "bootstrap.py"))["failure_source"]
    probe = function(helper, "shared_probe")
    equality_nodes = [n for n in ast.walk(probe) if type(n) is ast.Try
                      and any(type(h.type) is ast.Name and h.type.id == "AssertionError" for h in n.handlers)]
    check(len(equality_nodes) == 1)
    check(any(type(n) is ast.Name and n.id == "after_refusal" for n in ast.walk(equality_nodes[0])))
    equality_code = compile(ast.fix_missing_locations(ast.Module(
        body=[copy.deepcopy(equality_nodes[0])], type_ignores=[])), str(trace_helper), "exec")
    STAGE = "cases"
    keys = (b"http_rates/a", b"index_expiry_http_rates/a", b"index_counts/http_rates",
            b"maintenance_cursors/a", b"maintenance_bounds/a", b"users/a")
    before = snapshot([(key, b"A") for key in keys])
    after = snapshot([(key, b"B") for key in keys[:-1]]
                     + [(b"unknown_bucket/b", b"B"), (b"http_rates/b", b"B")])
    expected = {label: dict(zip(FIELDS, (1, 1, 0, 1, 0))) for label in LABELS}
    expected["http_rates"] = dict(zip(FIELDS, (1, 2, 1, 1, 0)))
    expected["protected_or_other"] = dict(zip(FIELDS, (1, 1, 1, 0, 1)))
    case("counts", "six_categories", lambda: check(counts(before, after) == expected))
    aliases = (b"index_counts/mail_limits", b"index_counts/http_rates-extra", b"http_rates2/x",
               b"maintenance_cursors-extra/x", "unadvertised/\u00e9".encode())
    def alias_case():
        actual = counts(snapshot([(key, b"A") for key in aliases]), b"")
        oracle = zeros(); oracle["protected_or_other"].update(before=5, removed=5)
        check(actual == oracle)
    case("counts", "exact_namespace_boundaries", alias_case)
    for name, value in (("opaque_non_json", b"\x00\xffA"), ("opaque_ciphertext", bytes(range(128)))):
        def opaque(value=value):
            actual = counts(snapshot([(b"users/a", value)]), snapshot([(b"users/a", value[:-1] + b"B")]))
            oracle = zeros(); oracle["protected_or_other"].update(before=1, after=1, changed=1)
            check(actual == oracle)
        case("counts", name, opaque)
    def identical():
        oracle = zeros()
        for label in LABELS: oracle[label].update(before=1, after=1)
        check(counts(before, before) == oracle)
    case("counts", "identical_complete_values", identical)
    malformed = (
        ("wrong_type", bytearray(b"61|00\n")), ("partial_line", b"752f61|00"),
        ("blank_line", b"\n"), ("crlf", b"752f61|00\r\n"),
        ("odd_key_hex", b"752f6|00\n"), ("odd_value_hex", b"752f61|0\n"),
        ("nonhex_key", b"zz|00\n"), ("nonhex_value", b"752f61|zz\n"),
        ("uppercase_key", b"752F61|00\n"), ("uppercase_value", b"752f61|FF\n"),
        ("extra_separator", b"752f61|00|00\n"), ("duplicate_key", b"752f61|00\n752f61|01\n"),
        ("invalid_utf8", b"ff2f61|00\n"), ("nul_key", b"752f0061|00\n"),
        ("missing_collection", b"2f61|00\n"), ("no_separator", b"7561|00\n"))
    for name, raw in malformed:
        case("parser", name, lambda raw=raw: rejected_parser(counts, raw))
    case("parser", "input_over_cap", lambda: rejected_parser(counts, b"x" * (8 * 1024 ** 2 + 1)))
    case("parser", "key_over_cap", lambda: rejected_parser(counts, snapshot([(b"u/" + b"a" * 4095, b"")])))
    case("parser", "value_over_cap", lambda: rejected_parser(counts, snapshot([(b"u/a", b"A" * (1024 ** 2 + 1))])))
    def boundary(key, value):
        oracle = zeros(); oracle["protected_or_other"].update(before=1, removed=1)
        check(counts(snapshot([(key, value)]), b"") == oracle)
    case("parser", "key_4096", lambda: boundary(b"u/" + b"a" * 4094, b""))
    case("parser", "value_1mib", lambda: boundary(b"u/a", b"A" * 1024 ** 2))
    case("parser", "empty_value", lambda: boundary(b"u/a", b""))
    case("parser", "empty_snapshot", lambda: check(counts(b"", b"") == zeros()))
    def full_capture():
        cap = 8 * 1024 ** 2
        head = [snapshot([(b"u/0" + bytes([48 + i]), b"\x00" * 1024 ** 2)]) for i in range(3)]
        overhead = len(snapshot([(b"u/03", b"")]))
        tail_size = (cap - sum(map(len, head)) - overhead) // 2
        raw = b"".join(head) + snapshot([(b"u/03", b"\x00" * tail_size)])
        check(len(raw) == cap)
        del head
        oracle = zeros(); oracle["protected_or_other"].update(before=4, removed=4)
        check(counts(raw, b"") == oracle)
    case("parser", "input_8mib_exact", full_capture)
    for name, value in (("zero", 0), ("one_million", 1_000_000)):
        case("schema", name, lambda value=value: check(validate(
            {label: {field: value for field in FIELDS} for label in LABELS}) is not None))
    class IntSubclass(int): pass
    class DictSubclass(dict): pass
    bad_scalars = (("negative", -1), ("over_million", 1_000_001), ("bool", True),
                   ("int_subclass", IntSubclass(1)), ("float", 1.0), ("string", "1"), ("null", None))
    for name, scalar in bad_scalars:
        def bad(scalar=scalar):
            value = zeros(); value["http_rates"]["before"] = scalar
            check(validate(value) is None)
        case("schema", name, bad)
    def wrong_keys(kind):
        value = zeros()
        if kind == "missing_category": del value["http_rates"]
        elif kind == "unknown_category": value[SENTINELS[0]] = value.pop("http_rates")
        elif kind == "missing_field": del value["http_rates"]["before"]
        else: value["http_rates"][SENTINELS[1]] = value["http_rates"].pop("before")
        check(validate(value) is None)
    for name in ("missing_category", "unknown_category", "missing_field", "unknown_field"):
        case("schema", name, lambda name=name: wrong_keys(name))
    case("schema", "dict_subclass", lambda: check(validate(DictSubclass(zeros())) is None))
    base = base_packet()
    projected = dict(base, store_snapshot_counts=zeros())
    case("packet", "valid_projection", lambda: check(consume(wire(projected)) == projected))
    maximum = {"exception_class": "AssertionError",
               "frames": [{"file": max(FILES, key=len), "line": 4096} for _ in range(8)],
               "store_snapshot_counts": {label: {field: 1_000_000 for field in FIELDS} for label in LABELS}}
    case("packet", "maximum_fixed_packet", lambda: check(len(wire(maximum)) <= 2048 and consume(wire(maximum)) == maximum))
    def padded():
        raw = wire(base).rstrip(b"\n")
        check(consume(raw + b" " * (2048 - len(raw))) == base)
        check(consume(raw + b" " * (2049 - len(raw))) is None)
    case("packet", "2048_and_2049", padded)
    case("packet", "unknown_root_fallback", lambda: check(consume(wire(dict(projected, private=SENTINELS[2]))) == base))
    for name, old, new in (
        ("duplicate_category", '"http_rates":', '"http_rates":{},"http_rates":'),
        ("duplicate_field", '"before":0', '"before":0,"before":0'),
        ("duplicate_root", '"exception_class":"AssertionError"', '"exception_class":"AssertionError","exception_class":"AssertionError"')):
        raw = wire(projected).replace(old.encode(), new.encode(), 1)
        case("packet", name, lambda raw=raw: check(consume(raw) == base))
    for name, field, value in (("unknown_file", "file", SENTINELS[3]), ("bool_line", "line", True),
                               ("zero_line", "line", 0), ("over_line", "line", 4097)):
        def bad_frame(field=field, value=value):
            packet = base_packet(); packet["frames"][0][field] = value
            check(consume(wire(packet)) is None)
        case("packet", name, bad_frame)
    def extra_frame():
        packet = base_packet(); packet["frames"][0]["private"] = SENTINELS[4]
        check(consume(wire(packet)) is None)
    case("packet", "unknown_frame_field", extra_frame)
    case("packet", "unknown_class", lambda: check(consume(wire(dict(base, exception_class=SENTINELS[4]))) is None))
    case("packet", "nine_frames", lambda: check(consume(wire(dict(base, frames=base["frames"] * 9))) is None))
    case("packet", "wrong_class_no_projection", lambda: check(consume(wire(
        dict(projected, exception_class="RuntimeError"))) == dict(base, exception_class="RuntimeError")))
    for name, raw in (("invalid_utf8", b"\xff"), ("malformed_json", b"{"),
                      ("deep_json", b"[" * 1000 + b"]" * 1000)):
        case("packet", name, lambda raw=raw: check(consume(raw) is None))
    for name, error_type in (("projection_memory_error", MemoryError), ("projection_keyboard_interrupt", KeyboardInterrupt)):
        def packet_fallback(error_type=error_type):
            original = packet_ns["snapshot_counts"]
            def failed_projection(value): raise error_type()
            packet_ns["snapshot_counts"] = failed_projection
            try: check(consume(wire(projected)) == base)
            finally: packet_ns["snapshot_counts"] = original
        case("packet", name, packet_fallback)
    def traced(error_type=AssertionError, depth=0):
        space = {"Error": error_type, "MESSAGE": SENTINELS[4], "DEPTH": depth, "__builtins__": {}}
        code = "def descend(n):\n    if n:\n        return descend(n-1)\n    raise Error(MESSAGE)\ndescend(DEPTH)\n"
        try: exec(compile(code, str(trace_helper), "exec"), space)
        except BaseException as error: return error
        raise AssertionError()
    def produced(error):
        value = json.loads(producer(error)); clean(value)
        return value
    def producer_valid():
        error = traced(); error.store_snapshot_counts = zeros()
        packet = produced(error)
        check(packet["store_snapshot_counts"] == zeros() and packet["frames"])
        check(all(frame["file"] in FILES and type(frame["line"]) is int for frame in packet["frames"]))
    case("producer", "genuine_bound_trace", producer_valid)
    def producer_bad():
        error = traced(); error.store_snapshot_counts = {SENTINELS[0]: SENTINELS[1]}
        packet = produced(error)
        check(set(packet) == {"exception_class", "frames"} and packet["frames"])
    case("producer", "invalid_attachment_fallback", producer_bad)
    case("producer", "missing_attachment_fallback", lambda: check(set(produced(traced())) == {"exception_class", "frames"}))
    class AssertionSubclass(AssertionError): pass
    for name, error_type in (("runtime_error", RuntimeError), ("assertion_subclass", AssertionSubclass)):
        def no_projection(error_type=error_type):
            error = traced(error_type); error.store_snapshot_counts = zeros()
            check("store_snapshot_counts" not in produced(error))
        case("producer", name, no_projection)
    def many_frames():
        error = traced(depth=80); error.store_snapshot_counts = zeros()
        value = produced(error)
        check(len(value["frames"]) == 8 and len(wire(value)) <= 2048)
    case("producer", "examined64_retained8", many_frames)
    for name, error_type in (("projection_memory_error", MemoryError), ("projection_keyboard_interrupt", KeyboardInterrupt)):
        def producer_fallback(error_type=error_type):
            original = producer.__globals__["snapshot_counts"]
            def failed_projection(value): raise error_type()
            producer.__globals__["snapshot_counts"] = failed_projection
            try:
                error = traced(); error.store_snapshot_counts = zeros()
                packet = produced(error)
                check(set(packet) == {"exception_class", "frames"} and packet["frames"])
            finally: producer.__globals__["snapshot_counts"] = original
        case("producer", name, producer_fallback)
    def compare(left, right, parser=counts, error_type=AssertionError, expected_counts=True):
        held, calls = error_type(MESSAGE), []
        class Matrix:
            @staticmethod
            def require(ok, message):
                calls.append(ok); check(message == MESSAGE)
                if not ok: raise held
        ns = {"matrix": Matrix, "before_refusal": left, "after_refusal": right,
              "refusal_snapshot_counts": parser, "AssertionError": AssertionError,
              "type": type, "BaseException": BaseException, "__builtins__": {}}
        try: exec(equality_code, ns)
        except BaseException as observed:
            check(observed is held and observed.args == (MESSAGE,) and calls == [False])
            check(hasattr(observed, "store_snapshot_counts") is expected_counts)
            if expected_counts: clean(observed.store_snapshot_counts)
            return observed
        check(left == right and calls == [True])
        return None
    case("equality", "identical_success", lambda: check(compare(before, before) is None))
    case("equality", "same_object_complete_change", lambda: check(compare(before, after).store_snapshot_counts == expected))
    reordered = b"".join(reversed(before.splitlines(keepends=True)))
    def row_reorder():
        error = compare(before, reordered)
        check(all(c["added"] == c["changed"] == c["removed"] == 0 for c in error.store_snapshot_counts.values()))
    case("equality", "row_reorder_still_denied", row_reorder)
    case("equality", "malformed_counts_same_error", lambda: compare(b"broken", after, expected_counts=False))
    for name, error_type in (("value_error", ValueError), ("memory_error", MemoryError), ("keyboard_interrupt", KeyboardInterrupt)):
        def fallback(error_type=error_type):
            def failed_parser(left, right): raise error_type()
            compare(before, after, parser=failed_parser, expected_counts=False)
        case("equality", name, fallback)
    case("equality", "subclass_excluded", lambda: compare(before, after, error_type=AssertionSubclass, expected_counts=False))
    def privacy():
        raw = snapshot([(b"users/" + SENTINELS[0].encode(), SENTINELS[1].encode() + SENTINELS[2].encode())])
        clean(counts(raw, b""))
        error = traced(); error.store_snapshot_counts = {SENTINELS[0]: SENTINELS[1]}
        clean(consume(wire(produced(error))))
        untrusted = dict(projected, private_uri=SENTINELS[2], private_path=SENTINELS[3], private_message=SENTINELS[4])
        sanitized = consume(wire(untrusted))
        check(sanitized == base); clean(sanitized)
    case("privacy", "sentinels_and_hex_absent", privacy)
    STAGE = "completed"

signal.signal(signal.SIGALRM, alarm)
signal.alarm(30)
try: main()
except Budget: STAGE = "budget"
except Halt: STAGE = "source_refusal"
except BaseException: STAGE = "internal_failure"
finally: signal.alarm(0)
elapsed = time.monotonic() - STARTED
if EXPIRED or elapsed >= 30: STAGE = "budget"
passed = sum(item["status"] == "pass" for item in CASES)
groups = [{"name": name, "attempted": sum(c["group"] == name for c in CASES),
           "passed": sum(c["group"] == name and c["status"] == "pass" for c in CASES),
           "failed": sum(c["group"] == name and c["status"] == "fail" for c in CASES)}
          for name in GROUPS]
result = {"schema": "riauth.a09-snapshot-memory/v1", "stage": STAGE,
          "attempted": len(CASES), "passed": passed, "failed": len(CASES) - passed,
          "groups": groups, "cases": CASES, "source_sha256": OBSERVED,
          "elapsed_seconds": elapsed, "budget_exhausted": EXPIRED,
          "native_runtime": False, "shared_gate": "not_measured"}
encoded = (json.dumps(result, sort_keys=True, separators=(",", ":")) + "\n").encode("ascii")
exit_code = 0 if STAGE == "completed" and result["failed"] == 0 else 1
if len(encoded) > 65536:
    encoded = b'{"schema":"riauth.a09-snapshot-memory/v1","stage":"output_cap"}\n'
    exit_code = 1
try:
    sys.stdout.buffer.write(encoded); sys.stdout.buffer.flush()
except BaseException: exit_code = 70
raise SystemExit(exit_code)
```

### Full future controller

```python
import hashlib, json, math, os, pathlib, re, selectors, signal, stat, subprocess, sys, time

REPO = pathlib.Path("/Users/dominik/orca/projects/riAuth-public-preview-local-workflow-safety-wave27")
REPORT = REPO / "docs/roadmap/local-wave30-a09-shared-users-refusal-plan.md"
EVIDENCE = REPO / ".target-wave27/a09-snapshot-counts-memory-d00c968-wave30"
PINS = [('helper', 'scripts/check-local-edition-transition-postgres.py', 26547, '4c60a7448d3c836215068fcc4f6822655c483f172a822e085f077d27b6a0c6fa'), ('workflow', '.github/workflows/check-local-shared-handoff.yml', 82149, '226483dd9edcd5e566b6c5de9f7ab2a36ebdf0c0f204f8ec992645542a3f395f'), ('matrix', 'scripts/check-exact-edition-matrix.py', 18394, 'f07d934f9cfd7af20eb086f5838863c28f840ee848e62bea1d865b330643d887'), ('gate', 'scripts/check-installed-release-gate.py', 22321, 'cbe8dcb42b4b1c36dc8df00204103b9c955feb8057275a441f60f3072d2bf8b5'), ('encrypted', 'scripts/check-local-encrypted-edition-transition.py', 22045, '09e137d88eb8e873a488448b7bdfdbe80d2327fbca716a93f445eaaae1ea9e8e'), ('spdx', 'scripts/spdx_sbom.py', 36727, 'ca063ab3d4abb6ec815151bcf762447e96682076a7f72d2dbfa9d7e6d3a1032c')]
PAYLOAD_SHA = "1cd3442cb91d19de9c70134da763d072b9cc5ca317e2ed2bb633df2bb1c04181"
GROUPS = ("source_binding", "counts", "parser", "schema", "packet", "producer", "equality", "privacy")
STARTED = globals().get("_A09_ASSEMBLY_STARTED", time.monotonic())
DEADLINE = STARTED + 35
STOP = False
CHILD = None
CHILD_STARTED = None
REAPED = False
RAW_EXIT = None
CAPS = {"stdout": 65536, "stderr": 8192}
BUFFERS = {"stdout": bytearray(), "stderr": bytearray()}
REASON = "none"
READY = False
OUTCOME_SAVED = False

def signal_stop(signum, frame):
    global STOP
    STOP = True

def encoded(value):
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode("ascii")

def save(name, data):
    if len(data) > 131072: raise ValueError()
    fd = os.open(EVIDENCE / name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, "wb") as out:
        out.write(data); out.flush(); os.fsync(out.fileno())

def kill_owned():
    if CHILD is None or REAPED: return
    try:
        # This Popen child has not been waited/reaped, so its PID cannot be reused.
        if os.getpgid(CHILD.pid) != CHILD.pid: raise ValueError()
        os.killpg(CHILD.pid, signal.SIGKILL)
    except ProcessLookupError: pass

def source_hashes():
    result = {}
    for name, relative, size, expected in PINS:
        try:
            fd = os.open(REPO / relative, os.O_RDONLY | os.O_NOFOLLOW)
            with os.fdopen(fd, "rb") as incoming:
                if not stat.S_ISREG(os.fstat(incoming.fileno()).st_mode): raise ValueError()
                data = incoming.read(size + 1)
            result[name] = hashlib.sha256(data).hexdigest() if len(data) == size else None
        except Exception: result[name] = None
    return result

def prepare():
    global READY, REASON
    if sys.version_info < (3, 11): raise ValueError()
    fd = os.open(REPORT, os.O_RDONLY | os.O_NOFOLLOW)
    with os.fdopen(fd, "rb") as incoming:
        if not stat.S_ISREG(os.fstat(incoming.fileno()).st_mode): raise ValueError()
        archive = incoming.read(524289)
    if len(archive) > 524288: raise ValueError()
    section = archive.decode("utf-8").split("## Exact memory-validation design archive, 2026-10-03\n", 1)[1]
    fence = chr(96) * 3
    marker = "### Full future child payload\n\n" + fence + "python\n"
    payload = section.split(marker, 1)[1].split("\n" + fence + "\n", 1)[0] + "\n"
    # Executable integrity precedes launch; measured-source/result grading follows retention.
    if hashlib.sha256(payload.encode()).hexdigest() != PAYLOAD_SHA: raise ValueError()
    space = os.statvfs(REPO)
    if space.f_bavail * space.f_frsize < 8 * 1024 ** 3:
        REASON = "capacity_refusal"; raise ValueError()
    if EVIDENCE.parent.is_symlink() or not EVIDENCE.parent.is_dir(): raise ValueError()
    if STOP or DEADLINE - time.monotonic() < 30:
        REASON = "setup_budget"; raise ValueError()
    EVIDENCE.mkdir(mode=0o700, exist_ok=False)
    READY = True
    if stat.S_IMODE(EVIDENCE.stat().st_mode) != 0o700: raise ValueError()
    return payload

def capture(payload):
    global CHILD, CHILD_STARTED, RAW_EXIT, REAPED, REASON
    CHILD_STARTED = time.monotonic()
    CHILD = subprocess.Popen([sys.executable, "-I", "-B", "-c", payload], stdin=subprocess.DEVNULL,
                             stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True,
                             env={"LANG": "C", "LC_ALL": "C", "PATH": "/usr/bin:/bin"})
    child_deadline = min(DEADLINE, CHILD_STARTED + 30)
    selector = selectors.DefaultSelector()
    for name, stream in (("stdout", CHILD.stdout), ("stderr", CHILD.stderr)):
        os.set_blocking(stream.fileno(), False)
        selector.register(stream, selectors.EVENT_READ, name)
    try:
        while selector.get_map():
            remaining = child_deadline - time.monotonic()
            if STOP or remaining <= 0:
                REASON = "interrupted" if STOP else "child_budget"
                kill_owned(); break
            for key, _ in selector.select(min(remaining, 0.05)):
                name = key.data
                block = os.read(key.fd, min(4096, CAPS[name] - len(BUFFERS[name]) + 1))
                if not block:
                    selector.unregister(key.fileobj)
                elif len(BUFFERS[name]) + len(block) > CAPS[name]:
                    BUFFERS[name].extend(block[:CAPS[name] - len(BUFFERS[name])])
                    REASON = name + "_cap"; kill_owned(); break
                else:
                    BUFFERS[name].extend(block)
            if REASON != "none": break
        try:
            RAW_EXIT = CHILD.wait(timeout=max(0.001, child_deadline - time.monotonic()))
        except subprocess.TimeoutExpired:
            REASON = "child_budget"; kill_owned()
            RAW_EXIT = CHILD.wait(timeout=5)
        REAPED = True
    finally:
        selector.close()

def retain_outcome():
    global OUTCOME_SAVED
    # No result/schema/expected-source comparison precedes these exclusive fsynced files.
    save("child-exit.json", encoded({"schema": "riauth.a09-memory-exit/v1", "exit_code": RAW_EXIT,
                                    "owned_pid": CHILD.pid, "owned_reaped": REAPED, "stop_reason": REASON,
                                    "stdout_bytes": len(BUFFERS["stdout"]), "stderr_bytes": len(BUFFERS["stderr"]),
                                    "child_wall_seconds": time.monotonic() - CHILD_STARTED}))
    save("child.json", bytes(BUFFERS["stdout"]))
    save("child-stderr.bin", bytes(BUFFERS["stderr"]))
    OUTCOME_SAVED = True

def unique_pairs(items):
    result = {}
    for key, value in items:
        if key in result: raise ValueError()
        result[key] = value
    return result

def grade(actual_sources):
    try:
        value = json.loads(bytes(BUFFERS["stdout"]), object_pairs_hook=unique_pairs)
        keys = {"schema", "stage", "attempted", "passed", "failed", "groups", "cases", "source_sha256",
                "elapsed_seconds", "budget_exhausted", "native_runtime", "shared_gate"}
        valid = type(value) is dict and set(value) == keys and value["schema"] == "riauth.a09-snapshot-memory/v1"
        valid = valid and type(value["cases"]) is list and type(value["groups"]) is list
        valid = valid and all(type(c) is dict and set(c) == {"group", "name", "status", "failure_class"}
                              and type(c["group"]) is str and c["group"] in GROUPS
                              and type(c["name"]) is str and re.fullmatch("[a-z0-9_]{1,64}", c["name"])
                              and type(c["status"]) is str and c["status"] in ("pass", "fail")
                              and type(c["failure_class"]) is str and c["failure_class"] in
                              ("None", "AssertionError", "ValueError", "TypeError", "KeyError",
                               "MemoryError", "KeyboardInterrupt", "Budget", "Other")
                              for c in value["cases"])
        valid = valid and len({(c["group"], c["name"]) for c in value["cases"]}) == len(value["cases"])
        valid = valid and all(type(value[k]) is int and 0 <= value[k] <= 1000 for k in ("attempted", "passed", "failed"))
        valid = valid and value["attempted"] == len(value["cases"])
        valid = valid and value["passed"] == sum(c["status"] == "pass" for c in value["cases"])
        valid = valid and value["failed"] == sum(c["status"] == "fail" for c in value["cases"])
        computed_groups = [{"name": name, "attempted": sum(c["group"] == name for c in value["cases"]),
                            "passed": sum(c["group"] == name and c["status"] == "pass" for c in value["cases"]),
                            "failed": sum(c["group"] == name and c["status"] == "fail" for c in value["cases"])}
                           for name in GROUPS] if valid else []
        valid = valid and all(type(g) is dict and set(g) == {"name", "attempted", "passed", "failed"}
                              and type(g["name"]) is str and g["name"] in GROUPS
                              and all(type(g[k]) is int and 0 <= g[k] <= 1000
                                      for k in ("attempted", "passed", "failed"))
                              for g in value["groups"])
        valid = valid and value["groups"] == computed_groups and all(g["attempted"] > 0 for g in computed_groups)
        expected_sources = {name: digest for name, relative, size, digest in PINS}
        valid = valid and value["source_sha256"] == expected_sources and actual_sources == expected_sources
        valid = valid and value["stage"] == "completed" and value["failed"] == 0
        valid = valid and type(value["elapsed_seconds"]) in (int, float) and math.isfinite(value["elapsed_seconds"])
        valid = valid and 0 <= value["elapsed_seconds"] < 30 and value["budget_exhausted"] is False
        valid = valid and value["native_runtime"] is False and value["shared_gate"] == "not_measured"
        valid = valid and type(RAW_EXIT) is int and RAW_EXIT == 0 and REASON == "none" and not BUFFERS["stderr"]
        return bool(valid)
    except Exception: return False

for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGALRM): signal.signal(sig, signal_stop)
signal.alarm(max(1, math.ceil(DEADLINE - time.monotonic())))
setup_failed = False
try:
    capture(prepare())
except BaseException:
    setup_failed = True
    if REASON == "none": REASON = "controller_failure"
finally:
    if CHILD is not None and not REAPED:
        try:
            kill_owned(); RAW_EXIT = CHILD.wait(timeout=5); REAPED = True
        except BaseException: pass
    if CHILD is not None:
        for stream in (CHILD.stdout, CHILD.stderr):
            if stream is not None: stream.close()

accepted, grading = False, "setup_refusal"
try:
    if CHILD is not None and READY:
        retain_outcome()
        after_outcome_save = time.monotonic() - STARTED
        actual_sources = source_hashes()
        accepted = not setup_failed and REAPED and grade(actual_sources)
        grading = "accepted_memory_only" if accepted else "outcome_refusal"
        save("controller.json", encoded({"schema": "riauth.a09-memory-controller/v1", "grading": grading,
                                        "actual_exit_code": RAW_EXIT, "owned_reaped": REAPED,
                                        "after_outcome_save_seconds": after_outcome_save,
                                        "source_sha256": actual_sources, "payload_sha256": PAYLOAD_SHA,
                                        "memory_only": True, "shared_gate": "not_measured"}))
    elif READY:
        save("controller-setup-refusal.json", encoded({"schema": "riauth.a09-memory-setup/v1",
                                                      "actual_exit_code": None, "stop_reason": REASON}))
except BaseException:
    accepted, grading = False, "receipt_failure"

# This final clock is AFTER all receipt writes/fsyncs. No file writes follow it.
final_clock = time.monotonic() - STARTED
accepted = accepted and OUTCOME_SAVED and not STOP and final_clock < 35
signal.alarm(0)
print(json.dumps({"schema": "riauth.a09-memory-final/v1", "accepted_memory_only": accepted,
                  "actual_exit_code": RAW_EXIT, "owned_reaped": REAPED, "outcome_saved": OUTCOME_SAVED,
                  "final_postsave_seconds": final_clock, "grading": grading, "shared_gate": "not_measured"},
                 sort_keys=True, separators=(",", ":")))
raise SystemExit(0 if accepted else 1)
```

### Full future assembly command

```bash
python3 -I -B - <<'PY'
import time
_A09_ASSEMBLY_STARTED = time.monotonic()
import hashlib, os, pathlib, stat
report = pathlib.Path("/Users/dominik/orca/projects/riAuth-public-preview-local-workflow-safety-wave27/docs/roadmap/local-wave30-a09-shared-users-refusal-plan.md")
try:
    fd = os.open(report, os.O_RDONLY | os.O_NOFOLLOW)
    with os.fdopen(fd, "rb") as incoming:
        if not stat.S_ISREG(os.fstat(incoming.fileno()).st_mode): raise ValueError()
        archive = incoming.read(524289)
    if len(archive) > 524288: raise ValueError()
    heading = "## Exact memory-validation design archive, 2026-10-03\n"
    if archive.decode("utf-8").count(heading) != 1: raise ValueError()
    section = archive.decode("utf-8").split(heading, 1)[1]
    fence = chr(96) * 3
    marker = "### Full future controller\n\n" + fence + "python\n"
    if section.count(marker) != 1: raise ValueError()
    controller = section.split(marker, 1)[1].split("\n" + fence + "\n", 1)[0] + "\n"
    if hashlib.sha256(controller.encode()).hexdigest() != "278060f3de48a133b017cffc188bf9adea200337e901370e97a1f1764ded44ee": raise ValueError()
except BaseException:
    print('{"schema":"riauth.a09-memory-assembly/v1","stage":"assembly_refusal","shared_gate":"not_measured"}')
    raise SystemExit(2)
exec(compile(controller, "<a09-memory-controller>", "exec"),
     {"__name__": "__main__", "_A09_ASSEMBLY_STARTED": _A09_ASSEMBLY_STARTED})
PY
```

### Actual static checks for this append

The archive build and independent re-extraction both exited 0. Only AST parsing,
source/literal reads, hashes and byte comparisons ran. All three full archived
payload/controller/assembly hashes matched their tables; the sole future Popen
site was verified statically. The complete 96,475-byte 95965 prefix reversed
exactly, and all six source files remained byte-identical to d00c968. The working
change scope was this report alone (843 appended lines before this receipt).
No case count or case result was produced.

Actual commands python3 scripts/check-docs.py and git diff --check exited 0;
the docs checker reported Markdown links and build-directory layout checked.
The static source/archive inspection was executed through Python stdlib
ast.parse/hashlib and Git object reads; candidate compile/exec/eval/import and
runtime were absent. A final docs/whitespace/scope check will cover this receipt
before the report-only commit. Runtime remains HELD pending root and independent
review; no slot was taken or released and no hosted rerun was requested.


### Root review of the first snapshot memory archive

Root read the complete 24,494-byte child, 12,634-byte controller, 1,427-byte
assembly command and all selected count/packet/bootstrap function bodies.
The exact 50,525-byte author append from 1aed8a4 follows the complete published
report. Comparing the prior author report to publication found only three
inserted root sections; all original lines and all published bytes remain.
An initial root contiguous-prefix assertion refused before writing because it
did not account for those existing root sections. No source was changed.

A pre-execution harness blocker was found: `failure_packet` calls builtin
`all`, but the isolated builtin name tuple omits it. Evaluating the valid
packet under that namespace would raise `NameError`; this is a source-derived
finding, not an executed case result. Root reserved only an append-only design
correction for that builtin plus the consequent archived hash substitutions.
Independent review of the first archive remains separate. No child, case,
traceback fixture, SQL, PostgreSQL helper, bootstrap or native command ran.
The actual remote changed rows and cause remain unknown, and the shared gate
remains open. Runtime remains held pending the corrected immutable archive,
independent review and a separate release.
