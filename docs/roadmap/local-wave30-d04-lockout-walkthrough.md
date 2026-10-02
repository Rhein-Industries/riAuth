# D04 independent second-administrator checkpoint: initial plan

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original D04 task
`ec76d0c5-2efe-4005-bb49-1f3b54878146`; supporting worktree
`e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`, branch
`roadmap/local-extension-isolation-wave27`. Date: 2026-10-02.
Fixed published source/documentation: `88790deb62d32c84fa17dceb12cd93a727224e94`.
Own clean starting HEAD: `fe68307ca86d2e3b29a896cc8ca170766fdbf545`.

**Recommendation: reserve one fresh, password-only Essentials lockout incident,
bounded to 900 seconds including cleanup. Runtime remains held.** Only this
report is edited. No listener, product binary, Cargo, Docker, provider, browser
or Driver has been run for this audit; no worker was contacted. Root reviews
the exact scope and releases execution separately. O07's observed UID/GID 0:0
blocker is preserved; this proposal does not rerun or bypass that fixture.

## Original row and live state

The live row was reread with the explicit project's `riwork task list --json`.
D04 is **in_progress**, with primary writer worktree
`f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`. This support does not change assignment
or status. The row's earlier scheduling text is historical; the user's current
bounded support instruction authorizes this report, not another worker launch.

Original outcome: **“Cover lockout, credential incidents, failed connectors,
outages, key loss, restore, migration, and rollback.”** The original workstream
goal is to document completed user/operator tasks; its gate is that a new user
and a new operator can independently complete the documented workflows. This
one operator incident cannot finish the other seven incident families or that
whole gate. The live prerequisite rows currently show U09 done, O06 done,
O07 in_progress, R05 done and G05 todo; none is reopened or reclassified here.

## Printed procedure and artifact provenance

`docs/admin-lockout.md` at the fixed source has blob
`0f3ab36243e5298aab203d4aef75dee37fc3c5ec`, SHA-256
`6a7e5ae99e25d9eea57ecf5d0d4b0f4e69cf6cf122f1299b957467c517ce09e3`.
Its second-administrator blocks print server-CLI `login`, `whoami`, `revision`,
`user passwd` and optionally `user reset-mfa`, using the live config/private
session and request-specific revision/idempotency key. Its historical library
and CLI-process drill outcomes are source/report evidence read here, not new
executions. The historical CLI fixture initializes through Core with login
rate 1000 before the security agreement is stamped. This proposal does not
copy that fixture setup, invoke Core, edit its records or change rate policy.

All three supplied regular, nonsymlink artifacts were read and hashed in place,
without executing, copying, replacing or rebuilding them. Artifact directory:
`/Users/dominik/orca/projects/riAuth-public-preview-local-module-boundaries-wave27/target/d01-essentials-c01c39a/aarch64-apple-darwin/debug`.

| Artifact | Verified SHA-256 | Size; mode | Proposed use |
| --- | --- | --- | --- |
| `riauth` | `7abf745c10691a012a1918c83089168d0e0d88b42764dbf8ed538b90c828b606` | 183038592 bytes; 0755 | One owned server plus the printed server-CLI procedure. |
| `riauth-maintenance` | `86490c7f71de6b7ae9d4dabdf9060a8757100f3aedbdaa670284d9e1206a2a95` | 56499296 bytes; 0755 | Fresh local `init` only; no recovery or adoption. |
| `riauthctl` | `bfbbb322f1a66d0ac9998beb9fb5838097cea0442cea3fd3a1d057fcb3f600cf` | 20406784 bytes; 0755 | Integrity checked; not executed in this checkpoint, which follows the printed server-CLI commands. |

These are the supplied **local-source c01 Essentials** artifacts, not official
released assets or a fresh build from `88790deb`. The immutable D01 report at
`88790deb` records their build provenance and earlier local operator execution;
that is historical evidence, not an execution performed by this lane.

Blob comparisons against `c01c39ab4e092423d5522bedc50fff87656d8c0a` found the
runbook and the following relevant implementation/fixture files byte-equivalent
to fixed `88790deb`:

| File | Equal Git blob at both pins |
| --- | --- |
| `src/cli.rs` | `581b8ee69ff73b1512fa583cc6be5dbc9f0745ee` |
| `src/cli/local.rs` | `57eccb2d1a7461e3b8b2442830984890233b8e8a` |
| `src/cli/transport.rs` | `28a1d2238c67b9fcb5d5d30a6217c0c0d099485e` |
| `src/core.rs` | `f02efcde5e9604b7251a8171e0aa437dbd8d8ae2` |
| `src/management.rs` | `8ff588258fd859949015604518003c5950bcd2ad` |
| `src/password.rs` | `54f246ea4a7b0525640b0382513aff1e14fe3052` |
| `src/authenticator.rs` | `5def93723db8864247945617cff684986dce51eb` |
| `src/context.rs` | `6b42201f0de33e5f28d89b5ef76e49ee7e3ca5db` |
| `tests/admin_lockout_cli.rs` | `32905ac359907d377066c3b25e7926d766b93c8c` |

The inspected `src/config.rs` delta adds an optional storage-allocation
diagnostic budget, default None, and its validation. This fixture would omit it.
These comparisons justify using the printed lockout semantics as expectations;
they do not assert that the entire artifact equals current main or validate all
later storage/telemetry behavior.

## Fresh fixture prerequisites and printed setup boundaries

After exact release only, use a new nonce-named, owner-only 0700 directory
under **this** worktree's `target`, with a project/task/run ownership marker.
Require at least 8 GiB free before setup and each non-cleanup child. Rehash the
three selected artifacts and refuse any mismatch. Preserve HOME; supply private
XDG_CONFIG_HOME directories and explicit private session paths for the affected
administrator, relief administrator and login-attempt caller. Use an allowlisted
environment without inherited RIAUTH_SERVER/CONFIG/SESSION_FILE/AGENT_FILE/OTP,
HTTP proxy or password overrides. Never read existing home sessions/configs.

Choose an available random high **loopback** port after release; never use
localhost9000 or probe D01's endpoint. Use the same exact
`http://127.0.0.1:$PORT` issuer and `127.0.0.1:$PORT` listen address at init.
If a race prevents the owned serve process binding, fail without choosing a
second port or contacting that listener. Only the captured serve PID/process
group is ours; readiness alone is not ownership proof.

Fresh initialization is the printed Essentials guide section 2 command with
the substituted fixture config/listen/issuer and private noninteractive stdin:

```sh
"$MAINT" --config "$LAB/riauth.toml" --json --non-interactive init \
  --issuer "$ISSUER" --listen "$LISTEN" --data-dir data \
  --admin affected --password-stdin
"$RIAUTH" --config "$LAB/riauth.toml" serve
```

No configuration/store exists initially. Use default redb/security agreement
and default login limit **20 per minute**; do not edit generated config or run
security-agreement-record. The fresh redb is private local fixture data, not a
deployment backup or encrypted-store/recovery proof. Passwords are distinct
random synthetic values of 32 or more ASCII characters, never argv/env literals:
P0 (affected initial), PR (relief), PW (wrong), P1 (replacement), P2 (different
request). Pass exactly one line on bounded stdin; no prompting/forced clock.

Before inducing lock, sign in as `affected`, read its current revision, and
create the second enabled human administrator through the normal printed
server-CLI user command, with a unique key and that revision. This preparation
uses `src/cli.rs::run_user` and the same public writer; it is not a Core/ledger
shortcut. Separate session files keep the healthy relief session from being
overwritten by affected-account login attempts. Both administrators are local,
factor-free, nondelegated humans. No hardware, TOTP/recovery-code enrollment or
factor-retention claim is included.

The emergency page properly assumes an already serving instance and an
available second administrator. It is not a complete all-CLI fresh-drill setup;
the Essentials init above and explicit second-admin creation fill that lab
preparation without changing its incident procedure. No product correction is
established. The one conditional printed-command gap is addressed below.

## One proposed incident command/outcome sequence

Aliases are public plan values, not an executed transcript. `$RIAUTH`/`$MAINT`
are the absolute verified artifacts above. `$LAB` is the fresh private root;
`$CFG` is `$LAB/riauth.toml`. `$SA`, `$SR`, `$ST` are distinct mode-0600 session
paths beneath their role-specific private XDG directories. Each server-CLI
command includes `--config "$CFG" --session-file ROLE_SESSION --json
--non-interactive --request-timeout 5`; the table abbreviates that prefix as
`A`, `R` or `T` for affected/relief/attempts respectively. Keys K_CREATE,
K_STALE, K_REUSE and K_RESET are fresh unique labels per run; the missing-flags
command deliberately sends neither header. R0/R1/R2 are read from
`data.revision`, never invented constants.

| Step | Exact command suffix / stdin | Expected checkpoint, not execution evidence |
| --- | --- | --- |
| Setup | Fresh maintenance init above / P0; one serve PID; `T status` | Exit 0; selected loopback issuer, owned live process, ready. No other store opener. |
| Initial authority | `A login affected --password-stdin` / P0; `A whoami`; `A revision` | Exit 0; enabled human admin, existing session; record R0. |
| Prepare relief | `A --idempotency-key "$K_CREATE" --if-revision "$R0" user create relief --admin --password-stdin` / PR | Exit 0; exactly one relief admin. Next revision R1 = R0 + 1. |
| Relief session | `R login relief --password-stdin` / PR; `R whoami`; `R revision` | Exit 0; independent healthy admin session; capture current R1. |
| Lock incident | Five separate `T login affected --password-stdin` / PW | Each exit 3, HTTP 401 `invalid_credentials`, retryable false. Stop if any result differs. |
| Establish active lock | `T login affected --password-stdin` / P0 | Exit 6, HTTP 429 `rate_limited`, retryable true, `Too many attempts; try again later`. |
| Still-safe authority | `A whoami`; `R whoami` | Both exit 0 while fresh password sign-in is locked; relief is the writer for every following mutation. |
| Missing binding refusal | `R user passwd affected --password-stdin` / P1 | Exit 1, `operation_failed`, HTTP status 0; message contains `User update requires --idempotency-key and --if-revision`. Local refusal, no claim of HTTP 428. |
| Stale revision refusal | `R --idempotency-key "$K_STALE" --if-revision "$R0" user passwd affected --password-stdin` / P1 | Exit 5, HTTP 409 `conflict`, `Configuration revision changed`. No successful mutation. |
| Password-history refusal | `R --idempotency-key "$K_REUSE" --if-revision "$R1" user passwd affected --password-stdin` / P0 | Exit 2, HTTP 400 `invalid_request`, `Password was used recently`. |
| Refusals preserve remedy boundary | `R revision`; `T login affected --password-stdin` / P0 | Revision remains R1; correct-password sign-in still returns the account-lock 429/exit 6. |
| Printed remedy | `R --idempotency-key "$K_RESET" --if-revision "$R1" user passwd affected --password-stdin` / P1 | Exit 0; public user result; revision advances once. |
| Ordinary mutation exact retry | Repeat the immediately preceding argv/stdin, including original R1 and K_RESET | Exit 0, identical public success data; revision must not advance again. This retains the accepted ordinary-mutation replay contract. |
| Different fingerprint refusal | Same K_RESET and R1 command, stdin P2 | Exit 5, HTTP 409 `conflict`, `Idempotency key was used for a different request`. No second replacement. |
| Current outcome | `R revision`; `R user list` | R2 = R1 + 1; exactly one affected and one relief enabled admin. No raw store snapshot/receipt/ledger read. |
| Old session revoked | `A whoami` using the untouched original session | Exit 3, HTTP 401 `invalid_token`. A fresh saved file is not substituted to hide revocation. |
| Old password refused | `T login affected --password-stdin` / P0 | Exit 3, HTTP 401 `invalid_credentials`. |
| Operator restored access | `T login affected --password-stdin` / P1; `T whoami`; `R whoami` | Exit 0; affected can sign in with P1 and relief authority still works. |

There are **11 login requests**: two setup logins, five wrong-password attempts,
two active-lock probes, one old-password refusal and one new-password login.
This stays below the unmodified fresh-instance address limit of 20 even if all
are in one minute.
No unknown-user enumeration burst, second lock cycle or rate override is needed.
Normal new writes use the current revision/new keys; only explicit refusal
probes and the documented exact retry intentionally reuse old inputs. Wrong
passwords and active-lock probes legitimately write attempts/audit state; the
plan does not promise a globally unchanged storage snapshot for those calls.

## Time bound, private evidence and cleanup plan

Root's release would authorize this one sequence, not a broad script/test
campaign. Use one monotonic 900-second outer deadline, reserve the final
60 seconds for cleanup, and stop operations by second 840. Cap each child at
30 seconds (startup readiness total at most 30); API calls also use the printed
5-second request deadline. Wait for the captured serve PID's own listener using
read-only socket metadata, then invoke the single `status` command. For this
host the ownership query is `lsof -nP -a -p "$SERVE_PID" -iTCP:"$PORT" -sTCP:LISTEN -F pn`;
require availability at release and matching loopback
socket/own PID. Waiting for that PID's startup is not retrying a failed incident
command. Refuse ownership ambiguity or an early process exit.
Cap stdin at 64 KiB and combined child stdout/stderr at 256 KiB, join IO before
accepting outcomes, and abort at the first unexpected result without retry,
alternate API, raw Core/ledger edit, clock change or authorization bypass.
Handle deliberately expected failures individually; do not run the incident
table under an unchecked `set -e` block or ignore a mismatched exit envelope.

Proposed fresh redacted evidence path:
`$PWD/target/d04-second-admin-c01c39a-88790deb.json` (absent at this audit).
Require it absent/nonsymlink again at release and publish owner-only 0600 with
exclusive creation. Retain source/runbook blobs, artifact hashes, public argv
aliases, start/end/elapsed, observed exit/code/http_status/retryable, revision
numbers, boolean identity/admin/session-mode checks, fixed failure stage and
cleanup results. Scan transient output for the generated password values and
session tokens; a leak becomes a fixed failure without reproducing the value.
Do not retain/print passwords, tokens, cookies, keys, hashes of low-entropy
secrets, raw output, raw config/store/receipts or real-user identity data. Server
output should be discarded; credentials and session files stay only in the
private lab until cleanup. No memory-zeroization guarantee is proposed.

On success, expected refusal, unexpected error or timeout: terminate only the
captured live serve process/group, wait/join with a bounded grace, kill that
owned group if still alive, and recheck its exit and that its selected port is
closed. Do not kill another PID discovered at the port. Verify the lab's
project/task/run marker and nonsymlink containment, then remove only that new
lab/config/redb/XDG/session tree. No global process/cache deletion or deletion
of any artifact/source/D01 resource. Record any uncertainty or cleanup failure;
cleanup is best effort, never a blanket signal/forced-termination guarantee.
Retain only the separate redacted evidence and this report. Successful incident
acceptance requires all expected checkpoints and confirmed own cleanup within
the outer deadline. No runtime/cleanup success is claimed yet.

## Smallest candidate doc clarification for the primary writer

The printed relief login supplies only the password unless an OTP is separately
provided. `Command::Login` sets `otp` to None without `RIAUTH_OTP` or `--mfa`;
`authenticator::consume_password_factor` refuses a TOTP-enrolled account without
a valid code. Thus an enrolled relief administrator can get credential failures
and eventually its own lock when copying that line literally. This is a
conditional documentation gap, not a demonstrated product defect or part of
this factor-free checkpoint. No MFA/hardware runtime is proposed here.

Smallest optional hunk for the primary writer, after the private-session
paragraph and before “Confirm that sign-in” in `docs/admin-lockout.md`:

```text
The login line below assumes the relief administrator has no authenticator.
For an authenticator-enabled relief account, run `login NAME --mfa` interactively
and supply its password and current code; the password alone cannot complete
sign-in. For a disposable rehearsal, prepare a fresh instance using the Essentials
guide, create the second
human administrator before inducing lock, and keep each session file separate.
```

Only this candidate text is recorded; `docs/admin-lockout.md` remains untouched.
For the proposed password-only fixture no additional printed-command correction
is justified before observing execution. Missing bindings, stale revisions,
password reuse, active-lock refusal and old-session/old-password refusal are
expected checkpoints, not product failures or reasons to bypass a guard.

## Actual audit checks and remaining scope

The original live row/prerequisites, immutable runbook, Essentials init guide,
CLI/init/transport/authentication/password writer definitions, prior D01 report
and relevant test definition were read. The artifact hashes/sizes/modes and
equal source blobs above were actually checked; no binary capability/version
command or fixture was executed. The report's exact task/source/hash/argv/budget
references, Markdown fences, sole-file diff and Git whitespace are checked
before committing. No whole-repository docs checker or source test runs here.
These are source/proposal checks, not independent completion evidence.

The printed `reset-mfa` path, enrolled-factor retention, recovery-code rotation,
browser account recovery, hardware, SMTP/tenant delivery, passkey-only/delegated/
agent/exposure cases, offline break-glass, key loss, restore, migration/rollback,
other editions/hosts/releases and the other D04 incident families remain outside
this one checkpoint. No all-eight-incident closure or task/status change is
recommended. Root receives the immutable report/command/outcome/cleanup plan
before any exact runtime release; the primary writer retains existing-doc
ownership. Desktop remains RiWork Cua.ai Driver only if separately assigned;
none is needed for this CLI plan. Accepted credential-issuance, route-header,
PAM fallback, review/permission/receipt/removal/audit protections stay unchanged.


## Released independent operator checkpoint: actual pass, 2026-10-02

Root read the complete immutable plan and released one checkpoint under
`wave30_D04_independent_lockout_checkpoint`, with 900 seconds overall,
840 seconds for operations and 60 seconds reserved for cleanup. This append
supersedes only the initial runtime-held/proposal wording for this incident;
the original report is preserved byte-for-byte. Root separately reported an
adjacent relief-authenticator clarification in accepted staging `6466901`.
This lane did not edit that guide or exercise its authenticator branch.

**The one released password-only checkpoint passed.** Controller exit **0**;
all **32 native CLI commands** matched their expected outcomes, including
**13 deliberate nonzero refusals**, with exactly **11 login requests**.
There was no unexpected result, mutation workaround, automatic correction or
second checkpoint. One additional `riauth serve` lifecycle was started and
joined; it is recorded separately from the 32 CLI commands. `riauthctl` was
hashed but never executed. No Cargo, Docker, browser/Driver, remote peer,
existing guide/product/source mutation or task/status action was performed.

### Exact provenance, timing and safety observations

The clean execution HEAD was the plan commit
`e26857b7a0429fc530d40c83badedadebb65f092`. Printed semantics came from fixed
published `88790deb62d32c84fa17dceb12cd93a727224e94`, runbook blob
`0f3ab36243e5298aab203d4aef75dee37fc3c5ec`. The exact c01 native artifacts listed
above matched their SHA-256/mode pins at preflight, were rehashed by the
controller before setup, and matched again in a post-exit read-only check.
This remains local-source Essentials/macOS-arm64 evidence, not official release
or whole-current-main binary equivalence.

Execution used one inline standard-library Python controller; no helper script
or product test was written. Its controller template, excluding here-doc framing and the added trailing
blank line, was **23159 bytes, 323 lines**, SHA-256
`a148e13462ef48df76cf1706c5b2fb1d0ace4c9f461275f1b8bdda262572a26d`.
Python AST syntax was checked before the sole execution, without side effects.
It orchestrated the approved public CLI argv, performed ownership/output/
privacy/deadline checks and cleanup, and printed only fixed redacted progress
and final summary. It did not instantiate Core or inspect/change a store,
receipt, password history, attempts row or clock.
The literal here-doc Python payload adds one trailing blank line: **23160 bytes,
324 lines**, SHA-256
`28b4d9396377c235cb4a6027a724cd60e4e07e4a56fb15ef82df8132f4dc3404`.

| Observation | Actual value |
| --- | --- |
| Start UTC | `2026-10-02T13:00:28.336727+00:00` |
| End UTC | `2026-10-02T13:00:38.368724+00:00` |
| Monotonic elapsed | **10.031960 seconds**, through cleanup/disk check, before final evidence publication |
| Captured serve PID/group | **70340**; new process group verified |
| One selected listener | **127.0.0.1:55533**, issuer `http://127.0.0.1:55533` |
| Startup ownership | Own PID and exact loopback socket verified before the one `status` call |
| D01 localhost9000 | Not selected or queried |
| Default login category | **20/minute**, unmodified; **11** incident login requests |
| Revisions | **R0 = 0**, **R1 = 1**, **R2 = 2** |
| External preflight free disk | **9.895 GiB** |
| Controller first/min/final free disk | **9.909 / 9.909 / 9.909 GiB** (rounded); **38** samples, all above 8.5 GiB stop margin and 8 GiB floor |
| OS metadata calls | **4** `lsof` invocations: own-PID startup waiting plus final selected-port closure; not product retries |
| Controller failure | None |

A fresh nonce-named mode-0700 private lab and exclusive mode-0600 project/task/
run ownership marker were created under this worktree's target. The port was
chosen once by binding an ephemeral loopback socket, held during init, then
released for the single serve launch. No bind race or port fallback occurred.
The three explicit role-specific XDG session paths remained separate; saved
sessions were checked as regular operator-owned mode-0600 files with the exact
issuer. HOME was preserved; RIAUTH/OTP/agent/proxy overrides were not inherited.
The generated distinct 32-character passwords traveled only on stdin, never
argv or environment. There was no post-init configuration/rate change.

The controller bounded each child at 30 seconds and combined stdout/stderr
at 256 KiB, stdin at 64 KiB, used a 5-second CLI HTTP timeout, and checked joined
IO/deadlines before accepting results. Native CLI stdout/stderr were transient,
not printed or logged; generated passwords and saved token values were checked
against them. Server stdout/stderr were discarded, so no server-log scan is
claimed. No password, token, cookie, key, raw config/store/response, private lab
path or low-entropy-secret hash was retained in the redacted evidence.

### Actual command outcomes

Aliases/prefixes A/R/T, CFG, LAB, session roles, passwords and keys retain the
initial plan's definitions. M is the exact maintenance artifact with
`--config "$LAB/riauth.toml" --json --non-interactive`; A/R/T each expand to the
exact server artifact with `--config "$CFG" --session-file ROLE_SESSION
--json --non-interactive --request-timeout 5`. The values after `/` are stdin
aliases, not password strings. Keys retain one unique value per label during
this run, and K_RESET was identical across its three documented request cases.
The separate serve argv was exactly
`$RIAUTH --config "$LAB/riauth.toml" serve`.

| # | Recorded stage | Actual suffix / stdin alias | Exit | Observed public outcome |
| --- | --- | --- | --- | --- |
| 1 | `setup.init` | `M init --issuer "$ISSUER" --listen "$LISTEN" --data-dir data --admin affected --password-stdin / P0` | 0 | initialized; exact issuer |
| 2 | `setup.status` | `T status` | 0 | ready after own-PID socket identity |
| 3 | `authority.affected_login` | `A login affected --password-stdin / P0` | 0 | expected human/admin/enabled identity |
| 4 | `authority.affected_whoami` | `A whoami` | 0 | expected human/admin/enabled identity |
| 5 | `setup.revision_before_relief` | `A revision` | 0 | revision 0 |
| 6 | `setup.create_relief` | `A --idempotency-key "$K_CREATE" --if-revision 0 user create relief --admin --password-stdin / PR` | 0 | expected human/admin/enabled identity |
| 7 | `authority.relief_login` | `R login relief --password-stdin / PR` | 0 | expected human/admin/enabled identity |
| 8 | `authority.relief_whoami` | `R whoami` | 0 | expected human/admin/enabled identity |
| 9 | `setup.revision_after_relief` | `R revision` | 0 | revision 1 |
| 10 | `lock.wrong_1` | `T login affected --password-stdin / PW` | 3 | 401, invalid_credentials, retryable false |
| 11 | `lock.wrong_2` | `T login affected --password-stdin / PW` | 3 | 401, invalid_credentials, retryable false |
| 12 | `lock.wrong_3` | `T login affected --password-stdin / PW` | 3 | 401, invalid_credentials, retryable false |
| 13 | `lock.wrong_4` | `T login affected --password-stdin / PW` | 3 | 401, invalid_credentials, retryable false |
| 14 | `lock.wrong_5` | `T login affected --password-stdin / PW` | 3 | 401, invalid_credentials, retryable false |
| 15 | `lock.correct_password_refused` | `T login affected --password-stdin / P0` | 6 | 429, rate_limited, retryable true |
| 16 | `locked.existing_affected_session` | `A whoami` | 0 | expected human/admin/enabled identity |
| 17 | `locked.relief_authority` | `R whoami` | 0 | expected human/admin/enabled identity |
| 18 | `refusal.missing_bindings` | `R user passwd affected --password-stdin / P1` | 1 | 0, operation_failed, retryable false |
| 19 | `refusal.stale_revision` | `R --idempotency-key "$K_STALE" --if-revision 0 user passwd affected --password-stdin / P1` | 5 | 409, conflict, retryable false |
| 20 | `refusal.password_reuse` | `R --idempotency-key "$K_REUSE" --if-revision 1 user passwd affected --password-stdin / P0` | 2 | 400, invalid_request, retryable false |
| 21 | `refusal.revision_unchanged` | `R revision` | 0 | revision 1 |
| 22 | `refusal.lock_remains` | `T login affected --password-stdin / P0` | 6 | 429, rate_limited, retryable true |
| 23 | `remedy.password_replacement` | `R --idempotency-key "$K_RESET" --if-revision 1 user passwd affected --password-stdin / P1` | 0 | expected human/admin/enabled identity |
| 24 | `retry.exact_original_revision` | `R --idempotency-key "$K_RESET" --if-revision 1 user passwd affected --password-stdin / P1` | 0 | same public data |
| 25 | `retry.different_fingerprint` | `R --idempotency-key "$K_RESET" --if-revision 1 user passwd affected --password-stdin / P2` | 5 | 409, conflict, retryable false |
| 26 | `outcome.final_revision` | `R revision` | 0 | revision 2 |
| 27 | `outcome.user_list` | `R user list` | 0 | one affected and one relief enabled admin |
| 28 | `outcome.old_session_refused` | `A whoami` | 3 | 401, invalid_token, retryable false |
| 29 | `outcome.old_password_refused` | `T login affected --password-stdin / P0` | 3 | 401, invalid_credentials, retryable false |
| 30 | `outcome.new_password_login` | `T login affected --password-stdin / P1` | 0 | expected human/admin/enabled identity |
| 31 | `outcome.restored_affected_identity` | `T whoami` | 0 | expected human/admin/enabled identity |
| 32 | `outcome.relief_still_authorized` | `R whoami` | 0 | expected human/admin/enabled identity |

Both existing sessions still answered whoami while new affected password
sign-in was locked. Missing binding, stale revision and password reuse refusals
left revision 1 and the lock in place. The current-revision/new-key password
remedy succeeded; exact retry with original revision 1 returned identical public
data; a different password with the same key returned the fingerprint conflict.
Final revision was exactly 2. The untouched original affected session was
refused, its old password was refused, the replacement password restored
sign-in, and relief authority remained available. These are visible CLI/API
outcomes, not a raw snapshot/audit/receipt inspection or factor-retention test.

### Exact expected refusal envelope values

All 13 deliberate refusals below matched schema, ok=false, exit, HTTP status,
code, full fixed message and retryable boolean. These are canonical JSON values
of the selected CLI envelopes, not raw stdout/stderr or byte-order transcripts.
The missing-bindings refusal is local HTTP status 0; no HTTP 428 is inferred.
Six invalid-credentials refusals comprise five wrong passwords plus the old
password after repair; two rate-limited refusals establish and preserve the
active account lock. No nonzero native CLI invocation is omitted or labelled as exit 0.

```json
[
  {
    "envelope": {
      "error": {
        "code": "invalid_credentials",
        "http_status": 401,
        "message": "Invalid username, password, or one-time code",
        "retryable": false
      },
      "exit_code": 3,
      "ok": false,
      "schema_version": "riauth.cli/v1"
    },
    "stage": "lock.wrong_1"
  },
  {
    "envelope": {
      "error": {
        "code": "invalid_credentials",
        "http_status": 401,
        "message": "Invalid username, password, or one-time code",
        "retryable": false
      },
      "exit_code": 3,
      "ok": false,
      "schema_version": "riauth.cli/v1"
    },
    "stage": "lock.wrong_2"
  },
  {
    "envelope": {
      "error": {
        "code": "invalid_credentials",
        "http_status": 401,
        "message": "Invalid username, password, or one-time code",
        "retryable": false
      },
      "exit_code": 3,
      "ok": false,
      "schema_version": "riauth.cli/v1"
    },
    "stage": "lock.wrong_3"
  },
  {
    "envelope": {
      "error": {
        "code": "invalid_credentials",
        "http_status": 401,
        "message": "Invalid username, password, or one-time code",
        "retryable": false
      },
      "exit_code": 3,
      "ok": false,
      "schema_version": "riauth.cli/v1"
    },
    "stage": "lock.wrong_4"
  },
  {
    "envelope": {
      "error": {
        "code": "invalid_credentials",
        "http_status": 401,
        "message": "Invalid username, password, or one-time code",
        "retryable": false
      },
      "exit_code": 3,
      "ok": false,
      "schema_version": "riauth.cli/v1"
    },
    "stage": "lock.wrong_5"
  },
  {
    "envelope": {
      "error": {
        "code": "rate_limited",
        "http_status": 429,
        "message": "Too many attempts; try again later",
        "retryable": true
      },
      "exit_code": 6,
      "ok": false,
      "schema_version": "riauth.cli/v1"
    },
    "stage": "lock.correct_password_refused"
  },
  {
    "envelope": {
      "error": {
        "code": "operation_failed",
        "http_status": 0,
        "message": "User update requires --idempotency-key and --if-revision (from `riauth revision`)",
        "retryable": false
      },
      "exit_code": 1,
      "ok": false,
      "schema_version": "riauth.cli/v1"
    },
    "stage": "refusal.missing_bindings"
  },
  {
    "envelope": {
      "error": {
        "code": "conflict",
        "http_status": 409,
        "message": "Configuration revision changed",
        "retryable": false
      },
      "exit_code": 5,
      "ok": false,
      "schema_version": "riauth.cli/v1"
    },
    "stage": "refusal.stale_revision"
  },
  {
    "envelope": {
      "error": {
        "code": "invalid_request",
        "http_status": 400,
        "message": "Password was used recently",
        "retryable": false
      },
      "exit_code": 2,
      "ok": false,
      "schema_version": "riauth.cli/v1"
    },
    "stage": "refusal.password_reuse"
  },
  {
    "envelope": {
      "error": {
        "code": "rate_limited",
        "http_status": 429,
        "message": "Too many attempts; try again later",
        "retryable": true
      },
      "exit_code": 6,
      "ok": false,
      "schema_version": "riauth.cli/v1"
    },
    "stage": "refusal.lock_remains"
  },
  {
    "envelope": {
      "error": {
        "code": "conflict",
        "http_status": 409,
        "message": "Idempotency key was used for a different request",
        "retryable": false
      },
      "exit_code": 5,
      "ok": false,
      "schema_version": "riauth.cli/v1"
    },
    "stage": "retry.different_fingerprint"
  },
  {
    "envelope": {
      "error": {
        "code": "invalid_token",
        "http_status": 401,
        "message": "Authentication required or session expired",
        "retryable": false
      },
      "exit_code": 3,
      "ok": false,
      "schema_version": "riauth.cli/v1"
    },
    "stage": "outcome.old_session_refused"
  },
  {
    "envelope": {
      "error": {
        "code": "invalid_credentials",
        "http_status": 401,
        "message": "Invalid username, password, or one-time code",
        "retryable": false
      },
      "exit_code": 3,
      "ok": false,
      "schema_version": "riauth.cli/v1"
    },
    "stage": "outcome.old_password_refused"
  }
]
```

### Cleanup, evidence retention and operator-runtime release

Cleanup sent SIGTERM only to the verified captured serve group, waited and
joined it with process exit **0**; SIGKILL was not needed. Final read-only
`lsof` reported no listener on the selected port. The exact private project/
task/run marker and ownership/containment were rechecked before deleting that
new lab; the config/redb/XDG/session tree was removed. Actual cleanup:

```json
{
  "failures": [],
  "marker_verified": true,
  "ok": true,
  "owned_server_started": true,
  "private_lab_removed": true,
  "selected_port_closed": true,
  "server_exit": 0,
  "server_joined": true,
  "server_pid": 70340
}
```

Evidence was created exclusively at
`target/d04-second-admin-c01c39a-88790deb.json`, mode **0600**, **16502 bytes**,
SHA-256 `44ce6499604c79471c722d99b8c1e463465d112ecbffdad29ce6a771b2be9350`.
It retains all 32 command outcomes, exact refusal envelope values, identity/
admin/session booleans, source/artifact provenance, revision numbers, timing,
disk samples, own socket check and cleanup booleans. It was not rewritten.
The original O07 evidence files and their accepted hashes remain unchanged.

Actual exit/counts/failure=null/elapsed/disk/cleanup and evidence hash were sent
immediately to the explicit-project orchestrator, with **RELEASE OPERATOR
RUNTIME**. This checkpoint used no Cargo slot and makes no Cargo release claim.
The runtime is finished; only this append-only report remains to commit.

### Method limits and original acceptance disposition

Accept this one local password-only second-human-administrator operator
checkpoint as executed evidence of the printed serving-store remedy. This is
one Essentials native fixture, not all eight D04 incident families or the full
new-user/new-operator gate. Root owns combining it with the primary writer's
other evidence and owns any final disposition; this lane changes no status.

No authenticator/recovery-code enrollment, reset-mfa/factor retention, browser,
hardware, SMTP/tenant, passkey-only/delegated/agent/exposed-account recovery,
key loss, break-glass, backup/restore, migration/rollback, PostgreSQL, deployment,
TLS, other platform/edition or released-asset workflow ran here. The optional
relief authenticator clarification is source-reviewed by root, not tested by
this factor-free execution. This run adds no O07 Docker/deployment proof.

The controller checked live success/refusal data and revision counts; it did
not inspect audit/receipt/attempt ledgers or prove global snapshot equality.
Startup lsof polling's individual raw rows/return codes and raw server logs
were not retained; the successful own-PID check and final closure were. No
cancellation, escaped IO, cleanup-error or forced-termination scenario was
exercised. Elapsed was captured after cleanup immediately before exclusive JSON
write; final fsync/publication/tool-return latency was not separately timed.
No atomic publication, forced-termination cleanup or memory-zeroization
promise is made. Source bytes, commands and actual observed outcomes are
separated from broader runtime/security claims.

Post-exit checks actually verified the unchanged artifact hashes, retained
JSON hash/size/mode/counts, every expected command result, cleanup/revision
fields and clean execution tree. This append's e268 prefix, exact recorded
refusal/cleanup values, bounds/source references, sole-file diff and Git
whitespace are checked before its separate report commit; no runtime retry,
new source test or broad documentation campaign follows. Accepted credential/
header/PAM and review/permission/receipt/removal/audit contracts stay unchanged.
