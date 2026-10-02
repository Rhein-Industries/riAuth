# Independent A09 container cohort source review

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original A09 task
`506e3979-a590-4af3-8fa8-ee90d3a517f2`; existing supporting worktree
`e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`, branch
`roadmap/local-extension-isolation-wave27`. Reservation:
`wave30_A09_container_independent_source_review`. Starting HEAD was clean
`08131c21657b93a8ec0e4705c5b582b475d53712`; no alignment or history import was
needed. Primary A09 worktree `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2` is unchanged.
This new report is the sole tracked write. Root owns integration, any remote
execution reservation and original disposition.

## Recommendation

**ACCEPT the bounded source slice
`be22eebb8cc957037f5e5c1f153f87df81f3dc25` by inspection.** No concrete reachable
source/interface blocker was established in the reviewed bodies. This is not
an executed container result, a runtime release, or whole-A09 closure. No
existing-file correction or additional implementation ownership is requested.

The local exact-UUID planning export records A09 `in_progress`. Its original
outcome is to produce and test server, client, container and maintenance
artifacts for explicitly supported platforms, including Linux x86-64 and ARM64.
Its distribution gate requires the same users and authorization rules and no
silently ignored configuration. Both editions retain shared identity,
revocation and credential protection. This review credits that original scope;
it introduces no all-browser/all-verbs/tenant study or extra release campaign.
The planning file was read locally, not queried as a fresh RiWork board/API.

## Immutable bodies and coverage

The following bodies were read in full through immutable Git objects, including
the entire controller's failure and cleanup paths:

| Pin / body | Lines / bytes / SHA-256 |
| --- | --- |
| `be22eebb8cc957037f5e5c1f153f87df81f3dc25` — `.github/workflows/check-local-container-cohort.yml` | 88 / 3398 / `7a0b7fb12a6c2650a1a7413e856ed6a9053aed7f5e1f06c2172413c7fdf84302` |
| Same source — `scripts/check-local-container-cohort.py` | 1516 / 85922 / `0f499284f5a053026230586f936b0ba38e58898a4f8fe36885efd30161bbbcfa` |
| `a7a314e74c49bb4a980728e90c9fdb604746ee5d` — complete design report | 746 / 52019 / `8584ee2e9b536dcbf0eb543dac191c362f149caa9e8440eb1af7fcbcedfc45b4` |
| `005c6223117f0efe805e4651f4f31721ee6062c2` — complete cumulative design/source-static report | 947 / 65198 / `2714b5ab9ca6e3a0fdce224f1a26df892a60800d97b55749e1675d6c1101a4eb` |
| Product `b619fe25269ccc150e473bbcde47cdb3623ef810` — `Dockerfile` | 28 / 1512 / `458ebb247170c6d4af2ff45b22e5a36e0a5380612140a43448d2ddcebc5d83cb` |
| Same product — `.dockerignore` | 57 / 794 / `4fbd9472316442bdd8b72e268feb1140701e87ac28dd02295c687b6d379eb093` |
| Same product — `deploy/compose-small.yml` | 50 / 1570 / `2db9060a4bbd8dd895401f5e58f2653ca3b774abf8e1867b9a41a52c0ae0cea6` |
| Review `66c814a339665e6b3f8e6c22bc59d4f6f0aa224c` — native-x86 root receipt | 341 / 11244 / `ccf7c3fb6c0bd85816097c32a24a1c1b0a67b9597346d571cd68975a4680b1af` |
| Same review — BuildKit source-pin root receipt | 220 / 8628 / `4d263888b2bdc5a2eb922b29c85535e8321dc2cba7c246b5b6b355cef57e57b0` |

Both receipt bodies equal their published copies at
`5dcd86a00743d5a23b24dea2273c830f3947f4c1`. The complete design remains an exact
prefix of the cumulative static report. CONTRIBUTING and SECURITY were read;
no applicable AGENTS.md was found in the worktree/ancestor/report directories.

Product contract review covered the relevant complete function/type bodies,
rather than claiming every line of their containing files: CLI parsing/login/
emit/error handling; standalone session persistence and issuer verification;
user/group/grant handlers and mutation headers; `UserView`, Core initialize/
open/me/session/logout/create-user/group/audit entrypoints; human provenance,
grant preparation/binding/authority and immediate grant writer; maintenance
keygen/init/plan/activate/envelopes; configuration defaults, validation and rate
serialization; format3 node-security agreement and edition assessment/token/
activation. Those were read at product `b619`, not mutable worker source.
The corrected O07 actual appendix at published `5dcd` was also read in full.

Action hashes, lockfile/dependency inputs and base-image references were checked
as source identities. No action implementation, OCI layer, downloaded archive,
native binary behavior or resolved apt package body was independently executed
or fully reviewed here. This distinction also applies to historical receipt
claims: reading a root check result does not repeat that check.

## Source, tool and historical native identity

The workflow is manual-only, one native `ubuntu-24.04` job, read-only
`contents`/`actions`, 150-minute job timeout and the legacy serialization key
`riauth-local-artifacts-arm64`, with cancellation disabled. Checkout is pinned
to `3d3c42e5aac5ba805825da76410c181273ba90b1`; upload to
`ea165f8d65b6e75b540449e92b4886f43607fa02`. Checkout disables persisted
credentials and separately selects controller, product and immutable review.
The controller requires Linux/x86_64, runner X64/github-hosted, manual branch
dispatch in the exact repository, clean HEADs, product tree and controller
agreement with both GitHub source/workflow SHAs. The tool daemon must be local
Linux/x86_64 and refuse Desktop; no daemon/context/topology fallback exists.

The BuildKit receipt's v0.33.1 index selects the one Linux/amd64 descriptor
`sha256:98cc6a3fc46220d00f8224ae483f3274fc874e9be8d7dd1e2e2c5481209228b5`.
The exact tool reference is
`docker.io/moby/buildkit@sha256:98cc6a3fc46220d00f8224ae483f3274fc874e9be8d7dd1e2e2c5481209228b5`;
future image inspection requires its config ID
`sha256:27933730df224df80c41f4e5a9b33fa78831a79fd31903df3bb7deb49363422f`.
The receipt states no downloaded layers and no native execution. That is a
reviewed descriptor pin, not a Buildx driver compatibility result.

The unchanged Dockerfile pins Rust `1.98.1-trixie` digest
`a8a5f0a1e5fe7dfe1d352591e4a1c7dd2c08fd70475cae872cf3458ba0df0546`
and Debian `trixie-slim` digest
`a99cfc517144bc59b1978475ec53b46ecabec7e43635402ee5b77cc54cd1b20a`.
The private recipe prefixes only the single existing Cargo command with private
Cargo paths, jobs1/incremental0 and dev/test/release debug0. Independent
AST-literal reconstruction reverses to every original Dockerfile byte. The
derived1694-byte recipe hashes to
`a55e90aa4da02870996c9dc755bbbbc80e0a1658674b0ab3f5fb141450a07440`.
Release thin LTO/strip, COPY inputs, edition selection, runtime packages,
licenses, labels, UID and entrypoint are retained. This is an explicit resource
variant, not an unmodified canonical image build or an apt/toolchain audit.

Historical native input is run `37046857550`, attempt1, job `110970324302`,
artifact `11246575279`, 52428080 bytes. Product source is `b619` / tree
`a627df2ce21a4d255b8c1914d4f543e32f40f4de`; executed native workflow source is
`0d090169f20f61f7cb59b685dccb13203526539f`. These identities are distinct.
The full declared outer ZIP digest is
`sha256:7c1bcad0f78d90eb611ab76d67943a32a0d691cd5bbd9e5588512bdde635e2a0`.
Root explicitly records it as API/upload-reported, not a retained/rehashed
outer ZIP. Its receipt records five binaries, ELF62, eleven exit0 steps and
root member/log/input checks, with `shared_full_gate=not_run` and no official
release. This audit read the full receipt and compared its seven inputs to
immutable product blobs; it did not reread/extract/run those external binaries
or freshly consume their ZIP/logs. The stale b619 notices stay historical.

## Gate mechanics and interface trace

| Source span / contract | Source-derived disposition |
| --- | --- |
| `source_check`, `native_transport` (script401–636) | Receipt hashes/project/schema/source/tree/run/attempt/job/architecture/outcome precede use. Same-repository artifact metadata is checked against the build-workflow SHA, not product SHA. One capped download must match the actual outer ZIP bytes/digest before extraction or execution. Signed storage gets no GitHub bearer; redirects/origin are restricted. Strict ZIP inventory/types/duplicates and512 MiB compressed/expanded limits; exact three regular TAR members, license/notices/member/binary hashes and ELF62 precede native capabilities. |
| `build_images`, `validate_image_tar`, `inspect_product` (875–1028) | Source builds two explicitly labeled LOCAL images. Save is streamed under2 GiB compressed/expanded caps. Canonical regular/directory members, exact tag/config ID/layers, optional OCI descriptor-to-manifest/config/layer binding and blob hashes are checked before load. Loaded tag IDs must match saved IDs. Inspected image UID/entrypoint/Cmd/working directory/volume/source/owner/edition/amd64, public server ELF/hash, licenses and capabilities/version are separate checks. Ubuntu native and Debian-built server hashes are not equated. Unsupported save/index layouts refuse rather than relax validation. |
| `create_container`, `tool`, `new_fixture`, `start_app` (782–1140) | Explicit10001:10001, read-only root, dropped capabilities/no-new-privileges, PID/memory/CPU limits, bounded tmpfs and no socket/secret mounts. New owner-labeled volumes are copy-up/stat probed at `/data` for exact10001:10001/0700 before credentials. Config/key/store must become0600 under that owner. Serving config is read-only and data writable, matching the small template's separated layout. Only one selected high random loopback port/host network and matching HTTP issuer; no root repair or bridge fallback. Product State.ExitCode is checked separately from Docker attach return code. |
| Public maintenance calls (script1029–1045,1197–1307) | Mounted native tools alone are public0755; credentials stay private. Keygen/init flags agree with the public parser. Offline plans are read-only; blocked plans use ok:false/data.ready:false/exit5, not a generic error envelope. Direct startup bad-request refusals use exit2. Both handoffs use the exact store/config-bound plan token and stopped writer. No agreement adoption/raw-store adapter appears. |
| Public client and authority (1131–1196,1309–1354) | Explicit issuer-bound sessions, password stdin, request deadline and verified discovery. Mutations use fresh admin-readable revision and request key; quoted If-Match is emitted by the accepted client. Complete12-field UserView and stable me/user/group/exact auditor grant/JWKS/issuer agree across E→P→E. Audit permission is tested at `/api/audit`, not inventory-audit. Human administrator creation records elevation provenance; immediate low-risk auditor binding is valid without bypassing reviewed privileged grants. |
| Refusal and revocation assertions | Delegate attempts real user creation with valid request headers; the client must report HTTP403/exit4 and user listing must remain exact. Two distinct saved sessions are logged out, their live session files removed and preserved copies must report real server401/client3 across handoffs. Fresh logins use different files/tokens; rejected copies are not renewed or substituted. The default8-hour session TTL exceeds the1200-second fixture bound. This is a selected sample, not a full raw-state rollback/revocation campaign. |
| Configuration and Platform boundary | Three parsed exact one-field variants change issuer, valid session_ttl28800→28801, or effective login rate20→21; targeted plan/startup reasons plus original config/key/data equality are required. Format3/all16 effective rates are enforced by the accepted startup/transition functions before startup writes. The isolated Platform agent uses the existing NewAgent schema, exact current quoted revision and key; its direct/preflight Essentials downgrade must refuse retained Platform state. The one-time credential response stays unlogged and is not retried. |

Named public contracts checked at `b619` include
`src/cli/local.rs::dispatch/emit_transition`, `src/cli.rs::report_error`,
`crates/riauthctl/src/transport.rs::verify_issuer/save_login/mutate_with`,
`crates/riauthctl/src/main.rs::emit/report_error`,
`src/management.rs::create_user`, `src/management/grants.rs::prepare/write_immediate_grants`,
`src/core.rs::open_store/me/session/audit_events`,
`src/management/sessions.rs::revoke_sessions`, and
`src/edition/transition.rs::assess/activate`. Their request/response fields and
authorization meanings match the new controller's calls. No accepted
credential-receipt, route-header or PAM exception is reopened; no source Group
or protected membership/privileged grant workaround is used. Product protocol
deadlines and all existing writers remain untouched.

## Bounds, own creation failures and cleanup

The launch/source/download/builder/build/fixture preflights require30 GiB.
Controller7200 seconds, fixture1200, each build1800, job150 minutes and
cleanup240 are explicit; command/API/download/save/load operations have their
own smaller limits. Nominal2-second host/private/workspace/actual-Docker-store
sampling stops owned work below10 GiB with an8 GiB margin/floor policy. Logs and
receipt/resource streams are capped8 MiB. Sampling/cancellation are not hard
quotas, and checkout/upload actions are outside the controller's sampler.

`command` (265–373) binds Linux `/proc` start ticks/session/group to the actual
Popen leader. A failed group proof signals only the direct child. Proven group
signals occur before leader reap; the direct process is waited/reaped and
group absence is checked. CLI command groups are a separate proof from Docker
daemon children: killing the CLI alone does not remove a builder/container.

Fresh names are checked absent using typed not-found output plus daemon liveness.
Pending container/volume records are saved before creation. Recovery requires
the exact owner label/name/creation interval and image, then stable IDs or
volume metadata. Builder recovery binds private Buildx instance, name/driver/
endpoint/tool reference, creation interval, actual container ID and exact
state-volume mount/metadata. A lost response or failed bootstrap does not
authorize guessed deletion. If any ownership/absence/group proof fails, cleanup
records a blocker and the result cannot be passed.

`cleanup` (1356–1462) explicitly removes only proven app/tool containers,
builder container/state/private instance, labeled fixture volumes and owned
image tags/IDs; final owner-image inventory and remaining tracked resources
must agree. Source refusal before verified daemon setup creates no Docker
cleanup claim. Content-addressed tool/base layers are intentionally retained;
there is no prune/shared cache deletion. Private disposable inputs are removed
only under the exclusive workflow root. Redacted evidence/image archives stay.
Sensitive command/HTTP responses remain in bounded memory and are not raw-logged;
child environment excludes the transport token/operator configuration. This
does not claim Python secret zeroization or measured cleanup success.

## Actual review checks, failures and remaining seam

Actual work was immutable Git/full-body reading, SHA-256/JSON comparisons,
standard-library Python AST parsing without helper import or execution,
214 explicit self-method keyword compatibility checks, staticmethod descriptor
checks for `elf`/`process_identity`, exact recipe reversal, receipt/selected
amd64-descriptor/product-input checks, and `bash -n` on the two workflow run
blocks. None of those blocks was executed. A selected read initially requested
nonexistent `src/management/users.rs`; Git refused, and the actual
`src/management.rs::create_user` body was subsequently read. Large combined
tool output was truncated, so bounded immutable reads covered the missing
spans before claiming complete body coverage. These were read/tool-output
limitations, not product/container failures.

No runtime blocker is asserted without observation. Before root grants an
execution reservation, actual native Docker/buildx/private-instance schema,
archive layout, UID copy-up/mount permissions, mounted native-tool dynamic
library compatibility, capacity/drain, all public fixture assertions and
ownership cleanup remain unmeasured. Their smallest future seam is the already
reviewed single native x86 cohort itself; no extra source hunk or campaign is
proposed here. A concrete future refusal must retain its exact phase/result and
request a bounded source correction before any existing-file ownership change.

Both earlier Desktop O07 failures remain failures: original1745-byte receipt
SHA-256 `308dfe8d3b424ff9a4e54369d809f560847b6aefda43a83ca229842b34068adc`
retains the classifier limitation; corrected1878-byte receipt SHA-256
`351349aa31e383fabd76a13dce188019437c9174af56d92a9824888abdba8af9`
records child0 but observed UID/GID0:0, mode0700 and `uid_mapping_unsupported`.
No application, credential, TLS/HA or current-image gate was reached there.
This separate native-host source proposal does not repair, bypass, rerun or
retroactively pass them. A09 and O07 original gates remain root-owned/open.

No Docker/buildx/pull/version/native binary/helper import/main/Cargo/test/PG/
HTTP/network/query/download/dispatch/browser/desktop/service or slot operation
occurred. No other worker was contacted, no source/other report was changed,
and no new task/worker/worktree/managed shell, deletion, merge/alignment,
accepted/main edit, push or board mutation occurred. All prior history, I04
receipts and completed-row assignments are preserved. RiWork Cua.ai Driver
remains the sole desktop provider if ever separately authorized.

Final actual static/report checks: `python3 scripts/check-docs.py` exited0;
report fence/whitespace and new-file-only scope checks passed. Reviewed source
pins and the entire prior I04 report remain exact. Staged `git diff --check`
passes before the report-only commit. These checks supply no Docker/native
runtime outcome or execution authorization.
