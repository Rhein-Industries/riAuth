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
