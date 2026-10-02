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
