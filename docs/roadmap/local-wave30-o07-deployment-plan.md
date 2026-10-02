# O07 initial deployment audit and bounded small-instance proposal

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original O07 task
`6c981199-62dd-464d-a12a-4ce4e27f428f`; assigned existing worktree
`e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`, branch
`roadmap/local-extension-isolation-wave27`. Audit date: 2026-10-02.
Fixed published source: `60437b59933cadd40a1f5fbbb91ba153aee56456`.
Own clean starting HEAD: `248402b8ae36dc376b47aceb62bb6fa2f04bfab5`.
Source, deployment documentation and tracked evidence were read from immutable
Git objects; this audit does not align or merge the branch.

**Recommendation: reserve one local small-service harness slice. No tracked
Compose correction is justified yet.** This report is the sole authorized edit.
The proposed command below has not run; neither harness code nor deployment
runtime is authorized until root makes the exact reservation. O07's distributed
outcome remains explicit, and no whole-task closure is recommended here.

## Original acceptance and current state

The original row requires: “Cover small installations and distributed setups,
with clear database HA and recovery responsibilities.” Its workstream gate is
that an operator can identify the failing component, what remains safe and the
required corrective action. Prerequisites are O01, O03, O04, O05 and R05.
The project planning row was reread from `planning/current-tasks.json` in the
explicit project directory: it currently records O07 `todo` with historical
worktree/scheduling metadata. The user's new assignment authorizes this audit in
the worktree above; root owns board reconciliation. O01/O03/O04/O05 are recorded
`done`; R05 is `in_progress`. None of their accepted dispositions is reopened.

Both Compose files were introduced in accepted ancestor
`5724ffc2ee40ef783d85377e1926c0aa496feae7`. The fixed pin has:

| Artifact | Source-derived coverage and remaining execution boundary |
| --- | --- |
| `deploy/compose-small.yml` | One UID/GID 10001 integrated server, one named `/data` volume, read-only root/config, `/tmp` tmpfs, no-new-privileges, Linux host network, bounded CLI readiness healthcheck, explicit tools-profile init, restart and graceful-stop policy. No tracked harness/test/CI reference to this Compose file was found. |
| `deploy/compose-distributed.yml` | One integrated server per service host; external PostgreSQL and load balancer; read-only local configuration, private ephemeral `/data`, no shared redb volume. Its exact-line `[postgres]` shell guard refuses the omitted stanza before serving. The guard is a template check, not full configuration validation or proof of database availability. No execution of this printed layout is established. |
| `docs/deployment-examples.md`, `deploy/postgres.json.example`, `deploy/haproxy-riauth.cfg.example` | Explicit TLS/secret mounts, UID assumptions, initialize once, common edition/revision/key/policy, per-node connection budget, readiness routing, separate load-balancer availability, database fencing and offline recovery. The small example delegates public TLS to host Caddy. The distributed example delegates PostgreSQL HA and the TLS load balancer to their operators. |

The current source supports the small template's commands:
`src/cli.rs::run` delegates keygen/init to `cli/local.rs`; init refuses an
existing configuration/store, creates the instance, and publishes private
configuration. `Command::Status` calls `/healthz` without authentication.
`cli/transport.rs::Remote::new` honors the explicit loopback `--server` and
`--session-file`, so the healthcheck needs no default home/config/session.
The Dockerfile prepares `/data` as 10001:10001, mode 0700. Therefore a speculative
root-owned-volume correction or replacement of init with another binary is
unwarranted. Fresh-volume ownership still needs an actual run with the selected
image. Docker documents default propagation of image content into an empty
volume in its [volume reference](https://docs.docker.com/engine/storage/volumes/).

`docs/availability.md` retains older node-local `forward_auth` wording in its
opening paragraph, while its later agreement discussion and current
`docs/operations.md` describe the accepted shared PostgreSQL counter. The
deployment guide repeats that older exception. This is a documentation
consistency item to resolve with root before claiming a validated distributed
guide; it is not a new O03 product defect or a proposed edit in this slice.

## Prerequisite and artifact evidence actually inspected

These are historical records inspected during this audit, not executions here.
The planning accepted-commit ledger snapshot SHA-256 is
`e5d67b88dd856bcd3bf9af7384a482fb0def5be8fa56c63fd6645e5b5e8ae846`.
Older inventory/ledger `partial` or `in_progress` labels are not substituted for
root's newer accepted original-row dispositions.

| Prerequisite | Accepted source/evidence useful to O07; limits retained |
| --- | --- |
| O01 `6f829cab-b389-4c27-8ec7-b4da8cc0d9c5` | `docs/roadmap/o01-process-roles.md` records integrated defaults, explicit acknowledged partial roles and single-owner redb. Historical local role target: 6 passes; PostgreSQL gateway/worker fixture: 1 pass, 11.78s, one failed logout delivery. Loopback HTTP/unencrypted PostgreSQL, no peer-duty health guarantee, standby promotion or deployed layout proof. Its old open heading does not reopen the done row. |
| O03 `85240c6b-8c87-4a62-a07e-68c7ed1a5d5a` | `local-wave29-o03-final-disposition.md` credits current format-3 startup agreement/rates, shared target admission/cache stamps, and historical node/worker checks. It records 7 current-format local units and one pass each for all-category rates and explicit agreement adoption. Older PostgreSQL disagreement/lease executions are credited at their historical formats; no current-format two-host/TLS deployment execution is inferred. Shared admission is a non-renewed 60s lease, not an external-I/O fence. |
| O04 `1552eec0-ec71-43ab-a5fd-afb07b9748b1` | Accepted activation integrations `08c9dcd0facd280df5fe9fa7b41c4a34937451a5` and `7483ab34f42a3a5c918a0286bedca0a53a9087c6`; current upgrade/rollback documentation and activation/preflight test definitions. The ledger records edition-transition-preflight 2/2 at historical `4787b19`, not a new run. Future schema/index/release/capability activation refusal, readiness fencing, stop all older writers and verified pre-upgrade backup remain required. Same-version source builds are not distinguished by the fence; older binaries that ignore it are not fenced. No old-binary rollback is claimed. |
| O05 `82f6e4d4-36e0-4ca7-b3fa-5e99e6ca7fc1` | Current operations documentation defines foreground/probe admission, background lanes, target exclusion and dedicated deactivation reserve. Accepted `697d596e7aa2feb552664a62d81a606d6aa60db7` records one exact deactivation-under-manual-saturation pass, including foreground sign-in/revocation/health and cancellation retention. This is source/local regression evidence, not deployment throughput, CPU/storage reservations or a new benchmark. |
| R05 `8c477e5e-8055-4cbe-9f3d-0c69fb19a2e8` | Tracked redb and PostgreSQL recovery JSON each record 16 passed checks: authenticated backup, wrong-key/occupied-target refusal, outage, recovery gate, old-session refusal and fresh service login. PostgreSQL outage records `/livez` 200, `/readyz` 503 and login `storage_unavailable`. Native PG16.14 physical-backup report records 16 passes, same-lineage blind spot, explicit invalidation, artifact refusal and a still-closed reconciliation gate. These fixtures do not certify customer escrow, lost-secret retrieval, real relying-party login or deployed two-host recovery. |

The tracked native ARM64 manifest
`docs/roadmap/evidence/a08-native-linux-arm64-2026-09-29/native-arm64-evidence-v2.json`
records local source revision `f3aba63ac3b824843a40b99623f1619ef8edc19f`, matching
the installed image IDs and labels below. That source is not an ancestor of the
published audit pin; cherry-picked/accepted evidence is not an assertion that
its executable equals current main. The same-edition recovery JSON records
isolated encrypted redb restore, old-session refusal and fresh login for both
editions at that source. Both records explicitly set `release_gate_result: false`.
Official exact assets/run provenance, signatures/publication and deployed
recovery remain open. The earlier `6ca4779` native report also distinguishes its
local images from release-workflow outputs and reports failing broader smoke
gates. None of those artifacts establishes Compose execution or HA.

## Exact proposed reservation and command

Proposed files after reservation:

1. `scripts/check-deployment-small.py`: new opt-in Python-standard-library
   controller, using installed Docker/Compose and exact local images. No Cargo,
   image build, pull, registry, PostgreSQL service or new dependency.
2. `deploy/compose-small.yml`: **no planned change**. Only a demonstrated
   runtime defect may justify a separate narrow correction with evidence.
   A new product seam or distributed-template change goes back to root first.
3. This report: later append exact observed results/source pins, or use a
   separately reserved evidence report if root assigns one.

The bounded command to reserve is:

```sh
python3 scripts/check-deployment-small.py \
  --template-revision 60437b59933cadd40a1f5fbbb91ba153aee56456 \
  --image sha256:ae84172a41d2dbe607a581d48fcde79cd1d293e00244994e257a757fd6def842 \
  --image-source-revision f3aba63ac3b824843a40b99623f1619ef8edc19f \
  --edition essentials --artifact-kind local-source \
  --probe-image sha256:9d2e5553305c7c7b0097999bb17187c69b921ccd6bc9d40e4bb5ebe652c00285 \
  --timeout-seconds 180 \
  --evidence "$PWD/target/o07-small-60437b5-f3aba63.json"
```

This interface and command are a proposal; the script does not yet exist.
The selected riAuth image is the installed
`riauth-q08-essentials:f3aba63-aarch64-local`, Linux/arm64, UID/GID 10001,
with the exact manifest ID above. The probe image is installed
`python:3.13-slim`, Linux/arm64, exact ID above. Its restricted helper would
make bounded standard-library HTTP requests from the Docker engine's loopback,
and provision only the new fixture directory when UID mapping allows it.
Explicit helper user choices must precede mounting: root only for fresh-fixture
ownership setup, unprivileged for probes. No helper mounts the Docker socket,
existing stores or any operator secret directory.

Proposed assertions and boundaries:

| Stage | Exact bounded behavior to verify |
| --- | --- |
| Refusal before mutation | Require existing exact Linux/arm64 images, expected edition/revision, local-artifact labeling, new evidence path and at least 8 GiB host free space. Use pull-never/no-build. Missing runtime/image or unsupported mount/network setup is unavailable/refused, never passed. Refuse an unresolved tag or existing run resources. |
| Scoped setup/render | New random Compose project name and exact ownership labels, one new named volume, one private generated directory/config/database key/password. Render the pinned template, preserve UID, host network, read-only flags, tools profile, healthcheck semantics and volume topology. Fixture-only overlay changes the bind source, project-owned labels and loopback port/healthcheck URL. It does not replace host networking with bridge networking or disable security fields. Record the exact template and rendered nonsecret structural fingerprints. |
| Initialization/start | Run printed keygen and password-stdin init sequentially against the fresh volume as UID10001. Verify private modes/readability and encrypted redb diagnosis. Start only the `riauth` service; tools-profile init must not start with it. Check actual container read-only root/config, no-new-privileges, bounded healthcheck and same volume ownership. |
| Live service | Require actual healthcheck success, `/livez`, `/readyz`/`healthz`, discovery with exact fixture issuer, nonempty JWKS, and generated-administrator login. Capture a session privately for an authorized doctor read; never print secret-bearing responses. This is loopback HTTP service validation, not public TLS/proxy/browser or all-feature coverage. |
| Restart/refusal/outage | Stop/restart the owning service using unchanged read-only config/key and the same fresh volume; require readiness and fresh login again. One bounded second-owner open must return `storage_owned` without replacing the store. With the owning service stopped, observe its fixture endpoint closed; test missing configured key refusal without creating a replacement/serving bootstrap, then restore only that generated file and reopen. Record unexpected outcomes instead of weakening checks. |
| Deadline/evidence/cleanup | Whole operation 180s; each subprocess/HTTP call has its own shorter bound. Capture only allowlisted statuses/codes/check results and nonsecret source/image hashes in new owner-only JSON; private transient output must be bounded and secrets never echoed. On failure or interruption stop/remove only created IDs whose project/ownership labels match, remove only the new volume/private fixture, and record cleanup failure. No global prune, shared service discovery or daemon settings change. |

Host networking is **not a network isolation guarantee**. State/resources are
isolated; all requests are restricted to the generated loopback listener and
the ephemeral probe helper. The [Docker host-network reference](https://docs.docker.com/engine/network/drivers/host/)
requires opt-in for Docker Desktop and distinguishes it from native Linux.
Desktop opt-in and direct UID bind mapping have not been established here.
The harness must refuse/report an unsupported setup, never silently change
network topology. It must record Desktop-backed Linux execution accurately,
not claim native Linux host Caddy/firewall behavior. No daemon configuration or
desktop action is proposed.

## Component, safety and remedy; remaining original scope

| Component/evidence | What remains safe | Remedy or input still required |
| --- | --- | --- |
| Runtime/image/mount/host-network prerequisite unavailable | No instance should be initialized; evidence remains unavailable. | Operator supplies a working supported runtime/mapping and the exact installed image. Do not fetch/build or alter Desktop setup implicitly. |
| Small redb owner/key/startup refusal | A failed second owner or missing-key open must not become an alternative writable store or a replacement setup. A running process with an already-loaded key is not proven stopped by file removal. | Stop/reap the owning service as appropriate; restore the correct escrowed key/config or use the rehearsed isolated restore procedure. Never share the volume. |
| Local readiness or process outage | Failed readiness removes traffic; liveness is separate. Neither readiness nor allocation success proves storage headroom, background completion or external dependencies. | Inspect the named startup/storage/duty diagnostic, restore storage/runtime prerequisites and require actual readiness/login before traffic. Preserve the accepted O06 unavailable-capacity contract. |
| Host Caddy/public TLS route | The prospective loopback run leaves public exposure untested. | A separately reserved isolated certificate/proxy fixture must verify Caddy configuration, proxy trust, HTTPS discovery/login and outage routing; the real hostname/certificate/firewall remain operator inputs. No ACME or public listener in this slice. |
| Two real service hosts/external PostgreSQL | Shared-store semantics and local historical worker tests do not establish this template's HA, failover, capacity or recovery objectives. | Named service hosts, verified common artifact/config/key/trust, PostgreSQL CA/endpoints and HA/backup/PITR policy, load-balancer owner and protected node network. Validate both template instances, node removal/re-addition, readiness and representative application access with those inputs. |
| Database promotion/restore | `synchronous_commit=on` alone provides no synchronous standby; riAuth neither elects nor fences a primary. Same-cluster physical restore can roll back revocation without automatic lineage detection. | Database operator owns standby/durability selection, writable routing, promotion and fencing; stop all riAuth writers when commit loss is possible, offline invalidate, reconcile persistent credentials and review the recovery ID before reopening. Rehearse restore/escrow and application access in the intended topology. |

Thus the first slice can establish **small service/template mechanics for one
installed local image**, if it actually passes. Current-main executable/release
artifact validation, public TLS/proxy behavior, other architectures/editions and
real distributed operation remain specifically named, not invented evidence.
The original distributed/HA/recovery acceptance is not reduced to one container.
Root alone integrates, pushes and changes task status. Completed S04/O03/M03 and
credential issuance, route-header, PAM fallback, receipt/removal/audit and
admission contracts are untouched.

## Actual audit checks and source pins

Read-only commands performed: assigned cwd/clean branch/HEAD checks; fixed Git
commit/tree/show/grep/history/ancestry and selected blob hashes; original task and
five prerequisite row reads; accepted-ledger/evidence reads; CONTRIBUTING guidance;
Docker executable/version/Compose version, server version/security fields,
installed image listing and restricted image metadata; host free-space check;
primary Docker documentation reads. No Docker container was launched, pulled,
built, executed, stopped or removed; no Cargo/service/product test ran. No secret
contents or image environment were inspected. No worker contact or desktop use.

Observed tooling: Docker client/engine 29.7.2, Compose v5.4.0, Docker Desktop
4.88.1 (237512), Linux engine arm64/aarch64. Security options reported builtin
seccomp and cgroup namespaces. Image IDs/OS/architecture/edition/revision match
the values above; labels alone are not a newly verified binary digest. Host free
space was 16.38 GiB. One Python image metadata query failed because its optional
`Config.User` key was absent; corrected restricted lookup returned null and the
full ID/OS/architecture successfully. A guessed `cli/remote.rs` path did not exist;
the actual `cli/transport.rs` implementation was then read. Neither lookup is
runtime product evidence.

| Pinned path at `60437b5` | Git blob |
| --- | --- |
| `deploy/compose-small.yml` | `a75307cde81782907014e97fb607884c2d0e731b` |
| `deploy/compose-distributed.yml` | `394b14528b3166edad135b501f4ba558c02f5e0e` |
| `docs/deployment-examples.md` | `a773996bd0f73e2f7591f6226a450fda1a68a0d5` |
| `Dockerfile` | `f445b27204c415c4537676986a94023aaaa72f34` |
| `src/cli/local.rs` | `57eccb2d1a7461e3b8b2442830984890233b8e8a` |
| `src/cli.rs` | `581b8ee69ff73b1512fa583cc6be5dbc9f0745ee` |
| Native ARM64 manifest above | `a888a1f8c617aaa9b3bdec9b09eaa306063eefee` |

Pinned small-template SHA-256:
`2db9060a4bbd8dd895401f5e58f2653ca3b774abf8e1867b9a41a52c0ae0cea6`.
Scoped UUID/source-object/command/boundary/Markdown/whitespace and sole-file
scope checks passed. Staged `git diff --check` passed; the staged stat/name list
contained only this report. No whole-repository documentation checker
was run. These checks attribute no new product runtime result. Gauge runtime
is finished and is not rerun.

## Implementation handoff after root reservation, 2026-10-02

This append supersedes the initial planning-snapshot status and proposal-only
wording above; it preserves that historical audit verbatim. The live RiWork row
was reread with `riwork task list --project 891e7443-8dac-4c1b-897f-9e53cb59c7ee
--json`: O07 is **in_progress**, assigned to existing worktree
`e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`. The original outcome/gate remains
unchanged. No board write was made. Root authorized the harness implementation
and this append only; **image/container/service/runtime execution remains held
pending immutable source review and exact release**.

Code commit: `97a30bd292da85752da7e71646811a51fd070bac`.
Only `scripts/check-deployment-small.py` is in that commit; Git blob
`751842e79904c85bb375b4922a2ab3cd2f58f19f`, file SHA-256
`3dd1ba816c116243b22b4d1746284ecc3c9f4e678a2f3e43d9053f946341e96c`.
The existing branch/history, gauge commits and prior report remain intact.
No alignment merge was necessary: the immutable small-template blob at published
`c01c39ab4e092423d5522bedc50fff87656d8c0a` equals the selected `60437b5` blob
`a75307cde81782907014e97fb607884c2d0e731b`. The harness verifies both objects;
it does not consume a mutable template or copy a stale production file.

Implemented interface and scope:

- The exact image/source/edition/artifact-kind tuple above is required and pinned
  to the tracked local manifest. No unresolved tag, alternate image, released
  artifact claim or current-main binary equivalence is accepted. Evidence records
  harness SHA-256/Git blob/execution HEAD, template revision/blob/digest, separate
  image source revision/ID and manifest digest.
- `--check-source` reads immutable objects and renders the pinned Compose file
  plus the fixture overlay. It directs Docker at a nonexistent Unix socket and
  removes context/TLS overrides, so successful rendering cannot depend on the
  live engine. It creates no containers, services, volumes or networks, and emits
  a source-only structural proof rather than runtime evidence.
- Render comparison is whole-model equality against the original render with
  only the allowed fixture bind source, loopback healthcheck URL, generated
  project identity, ownership labels and pull-never policy applied. UID10001,
  host networking, read-only root/config, no-new-privileges, `/tmp` policy, tools
  profile, readiness healthcheck budgets, restart/stop policy and one named
  volume are checked explicitly. The tools service keeps its original separate
  default network; its generated network receives ownership labels and cleanup.
  No tracked Compose file changed.
- Runtime pins the user's current **local Unix-socket** endpoint, refusing remote
  endpoints or conflicting overrides. It requires the installed Linux/arm64
  images' exact IDs and app edition/revision/user/volume metadata. It refuses a
  helper image declaring anonymous volumes. All launches use pull-never and the
  service start uses no-build; no topology/daemon/provider fallback exists.
- The sole Python helper bind is the freshly created, explicitly empty directory
  during initial UID ownership setup, before any configuration/key/password is
  written. That helper has no network. Subsequent HTTP/port helpers have **no
  mounts**; no helper ever mounts a credential-bearing directory, store or Docker
  socket. The declared application tools service alone retains the printed init
  mounts. All generated credentials travel through bounded stdin, not argv/env;
  the saved session lives in the application's tmpfs. Python zeroization is not
  claimed.
- The operations retain the proposed keygen/init, actual container health/probes,
  discovery/JWKS, administrator login/authorized encrypted-redb doctor read,
  second-owner `storage_owned` refusal, stopped-listener outage, missing-key
  refusal with config/encrypted-store byte hashes unchanged, and same-volume
  read-only restart/login assertions. A controller-host JWKS comparison must
  match the engine-loopback probe, so an engine-only loopback success cannot
  stand in for Desktop host visibility. Unsupported mapping/networking visibly
  fails/refuses; no alternative topology is tried.
- Inputs are capped at 64 KiB; combined child stdout/stderr at 256 KiB; HTTP
  responses at 64 KiB. Selectors drain bounded pipes, every child has a shorter
  deadline, and acceptance checks follow joined IO. Host HTTP probing also runs
  as a bounded child, preventing slow response reads from escaping the parent
  deadline. The 180-second overall budget reserves its last 20 seconds for
  cleanup; a deadline/cancellation cannot produce a passing result. Host free
  space is checked before non-cleanup child commands, and the Linux probe also
  checks its filesystem against the 8 GiB floor.
- Cleanup inventories only the random project/task/run labels, rechecks ownership
  before removal and removes only created containers, the new volume and the
  tools network. It does not prune Docker. Generated private files are removed
  through the declared tools service; an empty directory's mode is relaxed only
  inside its still-private parent so the controller can remove it. Unknown files,
  timeout/uncertain remote commands, remaining resources or failed cleanup are
  reported as failure; no forced-termination or atomic signal/publication
  guarantee is claimed. SIGINT/SIGTERM are cooperative; SIGHUP remains untouched
  for inherited `nohup` behavior. Durable JSON/stdout use selected nonsecret
  fields only; child diagnostics/responses are never echoed.

### Static evidence actually produced

At code commit `97a30bd`, the exact proposed command with `--check-source`
replacing `--evidence ...` exited **0**. It reported disabled engine endpoint,
`engine_contacted: false`, `product_runtime_executed: false` and:

| Proof | Exact value |
| --- | --- |
| Template SHA-256 | `2db9060a4bbd8dd895401f5e58f2653ca3b774abf8e1867b9a41a52c0ae0cea6` |
| Manifest SHA-256 | `638822d3c437d06678d27d4fb68987ebd2f6a884dd36c5843e9addf7727b5c7e` |
| Normalized rendered structural SHA-256 | `62574f39005fbdfcfaff892a215808998cbcae171d383c3151b16a38d1a25be5` |
| Checked safety flags | host network, UID10001, read-only root/config, no-new-privileges, tools profile and one volume all preserved |

The normalized fingerprint replaces only generated fixture path, project/run
identity and loopback issuer. The preceding equality check still covers the
unmodified complete models; normalization does not authorize more changes.
Earlier working-tree source-only renders also passed. Before the final
provenance-only additions to `source_proof`, three instrumented negative render
checks changed the returned model to UID0, bridge networking or writable root;
each raised `unapproved_rendered_change`, with the engine endpoint disabled.
The final commit changes no render-guard behavior from those checks.

Python AST parsing passed for the script and both embedded helpers. A duplicate
JSON key carrying a private marker was rejected with the fixed
`duplicate_json_key` identifier and no marker disclosure. Static checks found no
`shell=True`; helper refusal/size/path checks do not depend on Python `assert`
and remain effective under optimization. An all-zero unreserved image produced
exit **2**, `unreserved_source_or_image`, before fixture/engine setup. Staged and
committed Git whitespace checks passed. These are syntax/source/render guards,
not executions of the product, signal cleanup, kernel/mount behavior or service
assertions. There was no Cargo, image pull/build, Docker container/service or
public-TLS/PG/HA run.

The exact normal command in the earlier proposal is still the required runtime
command, using these same two image IDs, `--timeout-seconds 180` and the new
owner-only evidence path `target/o07-small-60437b5-f3aba63.json`. It is **not run**.
The source/hash/static proof and command were sent to the existing project
orchestrator with explicit project ID before this append. Root reviews and
releases runtime separately; no further authorization is inferred from elapsed
time or from source-only success.

The old `f3aba63` local image can establish only observed small-template
mechanics if that run later passes. It cannot prove the `c01c39a` executable,
current/released artifacts, public TLS/Caddy behavior, other architectures or
editions, distributed service hosts, database HA/fencing, deployed recovery or
whole O07 acceptance. The original distributed/HA/recovery inputs and the
component/safety/remedy table above remain open and named. Completed S04/O03/M03,
accepted credential/header/PAM/admission contracts, source/main/status ownership
and Cua.ai Driver preference remain unchanged.


## One released mechanics run: failure retained, 2026-10-02

Root explicitly released one normal 180-second command after reviewing immutable
harness commit `97a30bd292da85752da7e71646811a51fd070bac`. This section supersedes
only the preceding runtime-held/not-run wording; all earlier evidence remains
historical. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, O07 task
`6c981199-62dd-464d-a12a-4ce4e27f428f`, existing worktree
`e1b4399a-8c0d-46b8-880c-a71a4ebf53e7` and branch are unchanged. No board write,
merge, reset, main edit or push was made. R05 owns the Cargo slot; this run used
no Cargo and does not release or consume that slot.

### Exact command and observed result

The command ran once at clean HEAD
`394a4623d7b77cdc8d483a30ec9863144999c525`:

```sh
python3 scripts/check-deployment-small.py \
  --template-revision 60437b59933cadd40a1f5fbbb91ba153aee56456 \
  --image sha256:ae84172a41d2dbe607a581d48fcde79cd1d293e00244994e257a757fd6def842 \
  --image-source-revision f3aba63ac3b824843a40b99623f1619ef8edc19f \
  --edition essentials --artifact-kind local-source \
  --probe-image sha256:9d2e5553305c7c7b0097999bb17187c69b921ccd6bc9d40e4bb5ebe652c00285 \
  --timeout-seconds 180 \
  --evidence "$PWD/target/o07-small-60437b5-f3aba63.json"
```

Exit **1**; command-tool wall time **0.9044 seconds** (not a separately measured
in-process elapsed field). Exact stdout:

```json
{"result": "failed", "checks": 1, "cleanup_ok": true, "evidence_written": true}
```

Only `installed_images_match_local_manifest` passed. The failure identifier was
`uid_mapping_unsupported`, at the initial empty-directory ownership helper or its
UID/GID/mode postcondition. The engine identified itself as Linux/arm64, version
29.7.2, current context `desktop-linux`. No alternate endpoint/context/daemon
settings, network topology or permissions were tried. Preflight and post-exit
host free space were each **12.04 GiB**, above the 8 GiB floor. The one command
was not repeated, and no additional Docker launch, image pull/build, service,
Cargo, PostgreSQL or benchmark run followed it.

The evidence file remains under this worktree's private target, mode **0600**,
**1745 bytes**, SHA-256
`308dfe8d3b424ff9a4e54369d809f560847b6aefda43a83ca229842b34068adc`.
It was new before execution and was not rewritten afterward. The complete
allowlisted, nonsecret JSON is preserved below so the failed observation does
not depend solely on an ignored local target file:

```json
{
  "checks": [
    {
      "id": "installed_images_match_local_manifest",
      "observed": {},
      "result": "passed"
    }
  ],
  "cleanup": {
    "failures": [],
    "ok": true
  },
  "distributed_verified": false,
  "engine": {
    "arch": "arm64",
    "context": "desktop-linux",
    "host_visibility_test_required": true,
    "os": "linux",
    "version": "29.7.2"
  },
  "failure": "uid_mapping_unsupported",
  "ha_verified": false,
  "project": "891e7443-8dac-4c1b-897f-9e53cb59c7ee",
  "public_tls_verified": false,
  "result": "failed",
  "schema": "riauth.deployment-small-local/v1",
  "scope": "local-template-mechanics",
  "source": {
    "artifact_kind": "local-source",
    "current_source_binary_verified": false,
    "edition": "essentials",
    "equivalent_revision": "c01c39ab4e092423d5522bedc50fff87656d8c0a",
    "execution_head": "394a4623d7b77cdc8d483a30ec9863144999c525",
    "harness_git_blob": "751842e79904c85bb375b4922a2ab3cd2f58f19f",
    "harness_sha256": "3dd1ba816c116243b22b4d1746284ecc3c9f4e678a2f3e43d9053f946341e96c",
    "image": "sha256:ae84172a41d2dbe607a581d48fcde79cd1d293e00244994e257a757fd6def842",
    "image_source_revision": "f3aba63ac3b824843a40b99623f1619ef8edc19f",
    "manifest_sha256": "638822d3c437d06678d27d4fb68987ebd2f6a884dd36c5843e9addf7727b5c7e",
    "probe_image": "sha256:9d2e5553305c7c7b0097999bb17187c69b921ccd6bc9d40e4bb5ebe652c00285",
    "release_verified": false,
    "template_blob": "a75307cde81782907014e97fb607884c2d0e731b",
    "template_revision": "60437b59933cadd40a1f5fbbb91ba153aee56456",
    "template_sha256": "2db9060a4bbd8dd895401f5e58f2653ca3b774abf8e1867b9a41a52c0ae0cea6"
  },
  "task": "6c981199-62dd-464d-a12a-4ce4e27f428f",
  "whole_o07_complete": false
}
```

### Cleanup and exact unexecuted boundary

Actual harness cleanup evidence is `ok: true`, `failures: []`. Static inspection
of the tested harness shows that this result required inventories filtered by
this project's/task's/random-run ownership labels, ownership rechecks before
any removal, a final owned-container inventory, and removal of the empty private
configuration directory. Successful removal of that directory also triggers
removal of the private fixture parent before evidence publication. There was no
global prune/cache deletion. This is the harness's recorded cleanup proof, not a
separate post-run engine inventory or an executed signal-cancellation test.
The random run token/resource identifiers are not in the allowlisted evidence,
so no independent resource-by-resource post-run attestation is claimed.

`Deployment.run` orders the initial ownership helper before port selection,
Compose rendering, key generation, application initialization and service
startup. Thus this failure created no application volume, tools network or
application service via that path, and generated no application credentials.
The helper launch itself was attempted with a fresh empty bind directory,
network none, read-only root, no-new-privileges, bounded memory/pids, pull-never,
root only for ownership setup and the explicit CHOWN/FOWNER/DAC_OVERRIDE caps.
Its return code and output are not in evidence, so this report does not assert
that its container successfully started or that the ownership syscall ran.

No live port/host-network visibility, init ownership/file modes, readiness,
public HTTP/discovery/JWKS, administrator login/doctor, second-owner refusal,
outage, missing-key refusal/content hashes or same-volume restart assertions
were reached. The prior source-only rendered structural proof remains separately
credited static evidence; this failed normal run contains no rendered-model
proof. In particular, successful image metadata checks establish provenance of
the installed old local images, not usable host mapping or host networking.

### Source-derived diagnostic limitation and bounded correction proposal

At tested blob `751842e79904c85bb375b4922a2ab3cd2f58f19f`,
`Runner.success` (lines 166-169) maps every nonzero child status to
`child_command_failed` and discards the numeric status. `Deployment.helper`
(lines 400-403) maps that generic failure for the empty mount to
`uid_mapping_unsupported`. `Deployment.run` (lines 478-479) uses the same code
when the returned setup object does not equal the required UID/GID 10001 and
mode 0700. `Runner.call` bounds/drains stderr but does not retain its contents.
The JSON therefore does **not** distinguish a Docker/helper launch/setup error
from an observed unsupported UID/GID/mode postcondition. No narrower root cause
is proven by this run; no template or product defect is established.

Smallest proposal for a separately reserved harness-only correction: retain a
fixed setup-stage identifier and numeric child return code for this helper;
distinguish nonzero ownership-helper failure from an actually observed mapping
mismatch; optionally retain only type-validated numeric UID/GID/mode fields from
a successful helper result. Do not print raw stdout/stderr, Docker endpoints,
paths, credentials or arbitrary exceptions. Keep all image/template pins,
security controls, empty-mount restriction, deadlines, ownership cleanup and
fail-closed postcondition unchanged. A correction and any further run need root's
specific review/release; neither was performed here, and the failed evidence
must retain its original path/hash.

### Disposition and report-only checks

**Small-template mechanics remain unverified; whole O07 closure is not
recommended.** The local-source `f3aba63` Essentials image does not prove current
source, a released asset, public TLS/Caddy, other architectures/editions,
distributed hosts or database HA/fencing/recovery responsibilities. The original
distributed/TLS/two-host/PostgreSQL/recovery inputs and the existing
component/evidence/safety/remedy gate remain named in the original audit. No
healthy-capacity, deployed-recovery or independent runtime claim is added.
Completed task dispositions and the credential/header/PAM/admission contracts
remain untouched.

Post-exit checks reread the preserved evidence and exact SHA-256/mode/size,
confirmed the clean execution HEAD and read the immutable-equivalent local
harness for static failure/cleanup attribution. This append's prior-report
prefix, exact JSON/command/source pins, sole-file scope and Git whitespace are
checked before its separate report commit. These checks run no product, engine
fixture or additional test. The initial failure/cleanup/evidence hash was sent
promptly to the existing explicit-project orchestrator; the report commit and
bounded diagnostic proposal follow in a separate handoff.


## Reserved empty-setup diagnostics: source ready, runtime held, 2026-10-02

Root accepted the classifier limitation in the preceding failed-run report and
reserved only the empty ownership setup diagnostic hunks in
`scripts/check-deployment-small.py` plus this append. Code commit
`c8a34c8df5184d89f7a36d933f2ec172d01471cd` changes that script alone, with
29 insertions and 8 deletions; blob
`fa77efc53aefcf167bd4f1ce8cb2f60d62bdbb74`, SHA-256
`7e6e2265dad539356a4566131d62dd7904e19df7bba451d4e44588a23c77837d`.
No product, Compose, other documentation, main or board edit was made. Prior
branch history and the failed observation remain intact. **Runtime is held
until root reviews this immutable correction and releases one exact command.**

### Diagnostic contract and bounded scope

The future evidence has `ownership_setup.stage: "empty_directory_ownership"`.
Once the same bounded child call returns after joined IO, its actual integer
status is saved as `ownership_setup.child_rc`; an unavailable/deadline/cancelled
child that never returns a status does not receive a fabricated one. Booleans
are rejected as numeric status values. No generic nonzero result is taken as
evidence of unsupported mapping:

| Initial empty-setup observation | Fixed refusal and allowlisted evidence |
| --- | --- |
| Nonzero joined child status | `ownership_helper_command_failed`; fixed stage and actual integer `child_rc` only. Child output is not decoded or echoed. |
| Successfully decoded, handled chown/chmod/stat OSError | `ownership_setup_syscall_failed`; fixed stage, `child_rc: 0`, one of the literal operations `chown`, `chmod`, `stat`, and integer errno only. No error message, filename or traceback is retained. |
| Validated numeric UID/GID/mode differ from 10001/10001/0700 | `uid_mapping_unsupported`; stage, zero rc and the observed numeric fields. This is a postcondition mismatch, not inferred syscall causality. |
| Invalid JSON, duplicate keys, wrong shape/extra fields or invalid numeric diagnostic | Fixed decode/validation refusal; only already established stage/rc can persist. Arbitrary diagnostic values do not enter evidence. |
| Validated numeric UID/GID/mode equal the required values | Continue to the existing port-selection path; retain numeric observations without claiming any later check passed. |

The embedded setup preserves the original empty-directory check and exact
chown/chmod/stat calls. It catches OSError only for those three operations and
prints a two-field fixed-operation/integer-errno object. That handled branch
exits zero to deliver structured data; the parent explicitly refuses it, so
zero is never sufficient for ownership acceptance. Unhandled errors such as an
initial listdir failure still become a generic command failure, with no claim
about which syscall ran. Errno must be a nonboolean integer in 1..2^31-1.
UID/GID must be nonboolean integers in 0..2^32-1; mode must be a nonboolean
integer in 0..0777. Exact key sets prevent extra/private fields from surviving.
Numeric observations are copied only after validation.

Only the `SETUP_EMPTY` literal, the `Deployment.helper` empty-mount branch and
initial `Deployment.run` setup validation changed. The helper launch still has
the same pinned image, explicit fresh EMPTY mount, root-only setup user,
network none, read-only root, cap-drop ALL plus CHOWN/FOWNER/DAC_OVERRIDE,
no-new-privileges, pids/memory limits, pull-never, stdin/output bounds and
8-second child/global deadlines. Runner, cleanup, nonempty helpers, application
model and all later operations are unchanged. In particular, this reservation
does not change the later application-tool `FILE_MODES` failure classifier.
Raw stdout/stderr/error strings/paths/URLs/private values are not stored or
printed by the new diagnostic branches.

### Checks actually run before the source commit

Python AST parsing passed for the corrected script and embedded setup. After
masking only the reserved setup literal and the two containing methods, the
complete old/new module ASTs were equal. Every byte starting at the existing
`port_info = self.helper(PROBE, {"kind": "port"})` through the end of the script
was also equal to original harness commit `97a30bd`. This separately verifies
no change to later postconditions, application control or cleanup. A mocked
old/new empty-helper invocation produced identical launch argv/options except
for the reserved inline setup code.

One local in-memory guard check passed **32 mocked controller cases** and
**4 mocked embedded setup outcomes**. It covered generic rc 1/125/126/127/-9,
boolean rc refusal, the exact valid postcondition, individual UID/GID/mode
mismatches, all three fixed errno operations, boolean/float/string/negative/
out-of-range numeric refusal, absent/wrong/extra shapes, unknown operations,
duplicate keys and malformed JSON. Fake OSError messages and filenames carrying
a private marker did not appear in helper JSON or retained diagnostics. The
mocked embedded success/chown/chmod/stat cases called fake os functions only;
the accepted controller case stopped before port selection. No subprocess,
Docker command, mount, real ownership syscall or product was executed by these
checks. They are local diagnostic guard evidence, not a second mechanics run
or host-mapping proof.

Source-only Git whitespace and sole-script scope checks passed; no Compose
render rerun, Cargo, image pull/build, container, service, daemon/context change
or alternative topology was invoked. The original 1745-byte failure JSON still
has mode 0600 and SHA-256
`308dfe8d3b424ff9a4e54369d809f560847b6aefda43a83ca229842b34068adc`.
It remains at `target/o07-small-60437b5-f3aba63.json`; the original report prefix
and complete failed evidence are not rewritten.

### One proposed identical mechanics command after separate release

The original template/source/image/edition/probe/timeout tuple remains fixed.
Only the evidence destination is new; its absence was checked without creating
it. Proposed command, **not executed**:

```sh
python3 scripts/check-deployment-small.py \
  --template-revision 60437b59933cadd40a1f5fbbb91ba153aee56456 \
  --image sha256:ae84172a41d2dbe607a581d48fcde79cd1d293e00244994e257a757fd6def842 \
  --image-source-revision f3aba63ac3b824843a40b99623f1619ef8edc19f \
  --edition essentials --artifact-kind local-source \
  --probe-image sha256:9d2e5553305c7c7b0097999bb17187c69b921ccd6bc9d40e4bb5ebe652c00285 \
  --timeout-seconds 180 \
  --evidence "$PWD/target/o07-small-60437b5-f3aba63-ownership-diagnostic.json"
```

If separately released, keep the private target and 8 GiB floor, a fresh owned
random fixture, the same controls and ownership-checked cleanup, and preserve
every failed result. No automatic retry/fallback follows from this correction.
The prior runtime failed early; this source-only change adds no actual mapping,
host visibility, service, TLS, current/release binary, distributed/HA/recovery or
whole O07 evidence. The original task acceptance and component/evidence/safety/
remedy gate remain open to root's review; no task/status claim is made here.
The immutable code pins/scope and exact new-path command, followed by this
separate append-only report commit, are handed to the explicit project
orchestrator before any further runtime authorization.
