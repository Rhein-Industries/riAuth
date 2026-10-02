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
