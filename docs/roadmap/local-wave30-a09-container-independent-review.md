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


## Archive failure diagnosis — source-only reservation, 2026-10-02

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original A09
`506e3979-a590-4af3-8fa8-ee90d3a517f2`; existing supporting worktree
`e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`. Reservation
`wave30_A09_container_archive_failure_source_diagnosis` authorizes this
append-only diagnosis/design. It does not authorize a validator change or
execution. Primary `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2` remains assigned.
The original row in the planning snapshot still asks for produced/tested
server, client, container and maintenance artifacts for explicitly supported
Linux x86-64 and ARM64, with shared identity/authorization/configuration
semantics. Its snapshot status is in_progress; no status change was made.

**Concrete blocker:** immutable controller
`56bd0829514ed8014cc9563fc7b0e727dba46d1d`,
`scripts/check-local-container-cohort.py::Cohort.validate_image_tar`
(929–996), rejects the selected Docker export's six legitimate legacy V1
metadata blobs at `image_tar_unreferenced_member`. The source handles legacy
`VERSION`/`json` only under non-blob layer directories. The smallest proposed
correction is one strict expected-member derivation at this function's
allowlist boundary, plus its pure serializer/derivation helper in the same
script. No changes to builds, fixture, production, workflow or cleanup are
proposed. This supersedes the earlier runtime-unknown/no-observed-blocker
assessment for this concrete failure, while preserving that historical
static review and its limitations verbatim.

### Actual evidence read, with provenance and reached boundary

Root's actual native x86 container run `37061329815`, job `111018425393`,
used controller56bd and product
`b619fe25269ccc150e473bbcde47cdb3623ef810`, tree
`a627df2ce21a4d255b8c1914d4f543e32f40f4de`.
I read the complete bounded public metadata bodies below, parsed their JSON,
and independently rehashed these files and the outer ZIP. I did not open the
ZIP, read image/layer payloads, extract a filesystem, read private rows or
execute any controller/validator. The outer ZIP hash supplies retained-artifact
identity, not an independent image-archive content verification.

| Retained public file under `/tmp/riauth-wave30-container-37061329815` | Bytes | SHA-256 |
| --- | ---: | --- |
| `artifact.zip` (root artifact11252311290; `riauth-local-container-x86_64-37061329815-1`) | 49206215 | `d150a5950cbe6ff2dc4399614eca2bf4dfa274ac0db755739ef328a29f9c76b6` |
| `container-cohort.json` | 16872 | `ab7c5ce7b87e672db255edc4393ccb27348cb777b14f5f7880f87a68a986ab7d` |
| `launch.json` | 163 | `cf616ca243565a9e433e40fcfef3b61d80571c15b0e551a27121a8f94ecbb672` |
| `resources.jsonl` | 158341 | `c778352aaf5dc0babdf08fada27f1af1bb2047eafe35a79c32025e4a099039ef` |
| `essentials-image-inventory.json` | 3222 | `11f3352299d6715750b7f2547df31e0df85dc62f1e110b5c2cc5e4a7c9c57258` |
| `essentials-small-metadata.json` | 17651 | `430471f20c63eb533003415c19c3d19c8f83b542af7acb75621cfe150040e5b4` |

The controller's entire85922-byte script is byte-identical to the prior
reviewed be22 source, SHA-256
`0f499284f5a053026230586f936b0ba38e58898a4f8fe36885efd30161bbbcfa`.
The receipt workflow hash is the prior reviewed
`7a0b7fb12a6c2650a1a7413e856ed6a9053aed7f5e1f06c2172413c7fdf84302`.
This is actual pinned-source reading, not reliance on the old hold.

| Actual receipt step / boundary | Observed outcome |
| --- | --- |
| Build Essentials | exit0, 762.677seconds |
| Build Platform | exit0, 972.371seconds |
| Save Essentials | exit0, 17.976seconds |
| Validate Essentials | `image_tar_unreferenced_member`, failed phase `images` |
| Save Platform, remove/load/revalidate/inspect image product, UID/mount probes, E→P→E fixture | unreached |
| Shared full gate | `not_run` |
| Cleanup | `cleanup_errors=[]`; builder children removed; owned image inventory empty; pending/tracked containers/volumes empty; remaining owned CLI groups/containers/volumes0 |
| Command cleanup proof in receipt | all54 recorded command children reaped and groups empty |

Expected absence checks with exit1 are recorded as absence steps and do not
contradict the successful build/save exits. `checks=[]` and `uid_probes=[]`.
Launch free space was92410826752bytes at each recorded path. All888 resource
samples were read; each category's observed minimum was85502218240bytes.
JSONL gaps round to maximum2.0seconds; the receipt's unrounded maximum is
2.0004333619999954seconds. These are observed samples, not a hard disk quota.
The receipt marks the cohort local/not official release and explicitly
retains the later-discovered stale b619 native notices limitation. Earlier
native run37046857550 and its archive/root receipts remain separate evidence;
this container failure does not inherit their smoke outcome as container proof.
Root already released its runtime. This worker acquired/released no slot.

### Immutable official exporter source and identity limit

Read-only official GitHub API resolution returned annotated tag object
`3bfe60f8c91fca3ea00d87273756372fae7e1f5d` for v28.0.4 and commit
`6430e49a55babd9b8f4d08e70ecb2b68900770fe`. The API describes this tag as
unsigned; no signature attestation is invented. Tag-path `save.go` was also
fetched and matched the immutable commit's full bytes. [Official tag ref](https://api.github.com/repos/moby/moby/git/ref/tags/v28.0.4),
[official tag object](https://api.github.com/repos/moby/moby/git/tags/3bfe60f8c91fca3ea00d87273756372fae7e1f5d).

| Official file at commit6430e49 | Bytes / SHA-256 | Bodies read for this diagnosis |
| --- | --- | --- |
| `image/tarexport/save.go` | 17887 / `c0647f2f2678b8e17d287ad6c2f0af0317c4299903debab4d40f49d326aa4abd` | manifest construction246–257, saveImage412–499, saveConfigAndLayer502–610, saveConfig612–638; tag and commit byte comparison |
| `image/v1/imagev1.go` | 3343 / `429b4af4df0599bdd04fb10e5c40160e69dc9d8d4283d5644ca92f68eae9f8a5` | CreateID42–69, including its top-level RawMessage map |
| `image/image.go` | 8780 / `937db2805c103aa87cf820e742f25850c2c35dc328eaa202ebcbe9a23bb3da27` | V1Image31–82, Image embedding84–114 and RawJSON126–129 |
| `image/rootfs.go` | 1702 / `74f0339a27b21bf775c9874e1bd3e788eb939a7c15a56b4ec56789809172b0cc` | complete54-line file, ChainID47–54 |
| `layer/layer.go` | 6938 / `ac1abe2e3a1bc610d15d225b7db277504e7cdba33f82cb96525842a8c308bfef` | CreateChainID193–207 |
| `api/types/container/config.go` | 4243 / `304609117ec70c5eacb643be06b961445192ab68101a3d412ca5d0366762bf1a` | complete73-line file, Config field order/defaults44–73 |

Line numbers above are actual immutable raw-source numbers. The web text
renderer elides some blank lines and reports different display numbers.
Some large combined tool output was truncated; bounded subsequent reads
covered the exact relevant spans before this body-coverage claim.

The retained daemon metadata reports `ServerVersion=28.0.4`, linux/x86_64,
Ubuntu24.04.5. It does **not** contain daemon GitCommit/build bytes; therefore
this establishes official version-source compatibility, not proof that the
selected daemon binary is built from commit6430e49. No daemon invocation was
made to fill that limit.

Moby creates a V1 object for each ordered DiffID, calculates its legacy ID,
then serializes its metadata under the SHA-256 of those JSON bytes. The
intermediate objects begin with epoch creation and a zero ContainerConfig;
the final object uses the selected image's V1 fields. OS is assigned after
CreateID for intermediate objects. These blobs are separate from the layer
archives and OCI manifest. [Pinned exporter](https://github.com/moby/moby/blob/6430e49a55babd9b8f4d08e70ecb2b68900770fe/image/tarexport/save.go#L412-L499).

CreateID clears `id`, serializes V1, adds `layer_id` and a prefixed parent
through a sorted top-level RawMessage map, and hashes it. Nested ContainerConfig
serialization remains in struct order. Intermediate OS must be absent from
that pre-ID object; top OS is already present. [Pinned CreateID](https://github.com/moby/moby/blob/6430e49a55babd9b8f4d08e70ecb2b68900770fe/image/v1/imagev1.go#L42-L69),
[pinned V1 fields](https://github.com/moby/moby/blob/6430e49a55babd9b8f4d08e70ecb2b68900770fe/image/image.go#L31-L82).

ChainID starts with the first DiffID; each successor hashes the previous
prefixed ChainID, one space and the next prefixed DiffID. [Pinned ChainID](https://github.com/moby/moby/blob/6430e49a55babd9b8f4d08e70ecb2b68900770fe/layer/layer.go#L193-L207).
Config's mandatory zero-valued fields serialize even when omitted in the
selected OCI config; optional ExposedPorts/ArgsEscaped have their declared
positions. Map-valued Labels/Volumes/ExposedPorts require sorted keys, without
recursively sorting the struct fields. [Pinned Config schema](https://github.com/moby/moby/blob/6430e49a55babd9b8f4d08e70ecb2b68900770fe/api/types/container/config.go#L44-L73).

### Exact public metadata correspondence; no prospective validator run

The inventory has20 entries: two directories and18 regular files. The selected
config blob is `6c1248f6e88133330785d3df42b8e7129027b0e3010ab29fc30e2831408a7c5a`
(3946bytes). The OCI manifest is
`d2d43e3a3f2cacc1c7d5aa3d446d548f846f8e60beacbe1f1c8912ada2a75458`
(1162bytes). It selects that config and the same six ordered uncompressed
layers listed in `manifest.json` and config.rootfs.diff_ids. The OCI index,
layout and repositories metadata account for the remaining ordinary metadata.

The six extra blobs total3525bytes. Following their **observed** parent fields
produces this base-to-top sequence; these legacy IDs were not independently
recomputed by executing a proposed validator during this reservation.

| Layer ordinal | Extra `blobs/sha256/` basename | Bytes | Observed V1 `id` |
| --- | --- | ---: | --- |
| 1 | `f2c95b27390582a885e249206a4731e5d6100ff9a77cd39d9bb0eeec3afd422c` | 401 | `985f6de088aa0092b0205cbd28cef38331ebd1200a372782562f92d2067463e1` |
| 2 | `bfac4c35c38484d57b91e117ef48571d183808557de737290a2db1033a858be9` | 477 | `5123e9bebb510daa6215abe65735aa6ee7d86ce01421a1844cf59e123e75bbe0` |
| 3 | `42e47bd0cf542b5dc4810ade3689b6809f7b6f2b7a0dd9421cf841a7fc70cc6a` | 477 | `df05de371732dc0b2dfa5b9bfe65cad8d95fdd6e88a1d945d5cfdf31bd08fde4` |
| 4 | `403d14cad84ab235eb47caa1c6dc4633ed2666b726a8fcacf871de50f9bb72ff` | 477 | `ba8215617ef9b6c8bcfc3404cf3db67a3698faf6367064b1acedcbb7690d6394` |
| 5 | `1de23d2d3e2ecafe29ef6e2cfbc28f43a2e220b87aa6fdd0aa2bbdf06da2cc24` | 477 | `94f0f9a780d06a31df8b14daf4c205026009a17655bc05e97ebb9aaf48aa652f` |
| 6 | `e9c7179eb746fed1f7cd9b4a0111ac84b505bb79166553e1bbd3aa8f481b05b5` | 1216 | `fe9fd051b6d4fd29879e4da7d859543445e9108fb4eaaef0fbdb845f73f3b5d5` |

Base has no parent; each successor names the preceding observed V1 ID.
Intermediate schema is exactly `id,created,container_config,os`, plus parent
for ordinals2–5. Top adds `config,architecture`, preserving the selected
created timestamp `2026-10-02T20:46:21.377343067Z` and selected runtime config.
All intermediate created values are epoch and OS is linux. Their17-field
ContainerConfig has the declared zero values. No additional legacy fields
are present. These public objects match the official exporter mechanism;
shape/parent correspondence alone is insufficient to authorize extra blobs.

### One prospective validator hunk and strict helper contract

Design only, not implementation or runtime evidence. At the allowlist boundary
before the existing non-blob legacy loop, add:

```diff
+        if (self.receipt.get("daemon", {}).get("ServerVersion") == "28.0.4"
+                and all(re.fullmatch(r"blobs/sha256/[0-9a-f]{64}", x) for x in layers)):
+            allowed |= self.expected_v28_legacy_members(
+                settings, layers, members, hashes, payloads, edition, allowed)
         for layer in layers:
```

The new pure `expected_v28_legacy_members` would derive required paths and
compare exact bytes rather than classify unknown candidates. It must not
scan-and-allow every JSON blob, accept just an `id`/`parent` shape, or accept
self-consistent metadata unrelated to the selected config/layers. Unsupported
extra metadata retains the existing final refusal. This narrow profile is for
the selected v28.0.4 linux/amd64 save and fixed b619 resource-only recipe, not
a generic guarantee for all Docker/containerd exporters.

Exact prospective helper design:

1. Reuse the selected config already bound to `image` by its SHA-256. Require
   its top-level key set to be exactly
   `architecture,config,created,history,os,rootfs` for this profile, with
   linux/amd64 and history a list. Require rootfs to have exactly `type` and
   `diff_ids`, type `layers`, and a nonempty list of at most128 canonical
   prefixed SHA-256 strings. Require the ordered `layers` names to equal
   `blobs/sha256/` plus each corresponding DiffID. Require that each layer is
   regular and its previously streamed content hash equals that DiffID. This
   binds ChainID inputs to the actual selected layer bytes, not arbitrary
   JSON claims. The128 limit is a proposed finite helper guard, not a new
   observed test result; current export has six.
2. Require the selected config's nine-field schema and typed values to match
   the fixed recipe: User10001:10001, Cmd[serve], Entrypoint[riauth,--config,
   /data/riauth.toml], WorkingDir/data, Volumes{/data:{}},
   ExposedPorts{9000/tcp:{}}, the fixed PATH Env list, boolean ArgsEscaped=true,
   and exactly the four source/revision/edition/owner labels. Bind SOURCE,
   REPOSITORY, edition and owner to the current controller. Type-sensitive
   compact sorted JSON comparison can reject `1` for `true`, null/map/list
   substitutions and unknown fields; plain Python dict equality cannot.
   Require edition essentials/platform and owner `a09-[0-9a-f]{32}`. This
   confines all serialized values to the known ASCII profile; do not claim
   an unreviewed general Python replacement for Go JSON/time marshaling.
3. Require created to be canonical UTC RFC3339Nano: valid date/time, Z suffix,
   optional1–9 fractional digits without trailing zero. Use existing datetime
   for calendar validation, retaining all original fractional digits. Refuse
   unsupported/malformed timestamp profiles rather than normalizing a value
   silently. Other V1 fields are absent by the exact selected key set.
4. Construct zero ContainerConfig in this pinned declaration order:
   Hostname,Domainname,User,AttachStdin,AttachStdout,AttachStderr,Tty,OpenStdin,
   StdinOnce,Env,Cmd,Image,Volumes,WorkingDir,Entrypoint,OnBuild,Labels.
   Strings are empty, booleans false, slices/maps null. The runtime Config
   uses those mandatory defaults overlaid with the selected typed config;
   insert ExposedPorts after AttachStderr and ArgsEscaped after Cmd. Sort keys
   only inside its map fields; retain the declared struct order. This finite
   serialization has no floats, non-ASCII or HTML-sensitive values and uses
   compact JSON with no newline. Correct default fields/order are required,
   not merely semantic JSON equality.
5. For each ordered DiffID, derive the ChainID from the selected chain. Set
   the pre-ID V1 fields to created=epoch and zero ContainerConfig for all
   intermediate objects; for the last, use selected created, zero
   ContainerConfig, expanded runtime Config, architecture=amd64,os=linux.
   Construct CreateID's top-level map of RawMessage-equivalent serialized
   values, add `layer_id`=prefixed ChainID and, except at base,
   `parent`=prefixed previous **computed** V1 ID. Sort only that top-level
   map's keys. Hash those exact bytes to derive the expected legacy ID.
6. Serialize the expected saved object in V1 struct order: id, optional
   unprefixed parent, created, container_config, optional top config and
   architecture, os. OS is inserted only after intermediate ID calculation;
   the top already included it. Hash these exact metadata bytes to derive
   the sole permitted `blobs/sha256/<hash>` member for this layer. Require
   this path not already allowed/derived, to be an existing regular member,
   with matching size, streamed hash and **exact payload bytes**. Require
   payload available within the existing small-payload cap. Return exactly
   these derived member names after the entire ordered chain passes.

The decisive hash/allowlist portion, in prospective pseudocode, is:

```python
chain = None
previous = None
expected_members = set()
for ordinal, diff_id in enumerate(selected_diff_ids):
    self.check_budget()
    chain = diff_id if chain is None else "sha256:" + digest(
        (chain + " " + diff_id).encode("ascii"))
    pre_id = intermediate_v1() if ordinal + 1 < len(selected_diff_ids) else top_v1()
    id_map = dict(pre_id)
    id_map["layer_id"] = chain
    if previous is not None:
        id_map["parent"] = "sha256:" + previous
    legacy_id = digest(sorted_top_map_with_struct_ordered_values(id_map))
    saved = ordered_v1_with_id_parent_and_os(pre_id, legacy_id, previous, "linux")
    expected = compact_finite_go_v1_bytes(saved)
    name = "blobs/sha256/" + digest(expected)
    require(name not in expected_members and name not in ordinary_allowed
            and name in members and members[name].isfile()
            and members[name].size == len(expected)
            and hashes.get(name) == digest(expected)
            and payloads.get(name) == expected, "image_legacy_metadata_binding")
    expected_members.add(name)
    previous = legacy_id
return expected_members
```

The exact prospective helper signature is
`expected_v28_legacy_members(self, settings, layers, members, hashes, payloads,
edition, ordinary_allowed)`. It receives the current ordinary allowlist without
mutating it and returns only the verified expected metadata member set. Named
serializer routines above describe the finite exact construction in steps2–6,
not existing callable functions or executed implementation. No data-dependent
JSON snippets may supply values/order for these routines. Generated exact-byte comparison also refuses duplicate JSON
keys, unknown fields, extra whitespace or metadata reserialization instead
of allowing a lossy parser to erase them. Intermediate/top schema and hashes
are derived from selected config and layers, not from observed legacy IDs.

Keep the existing final `set(hashes) <= allowed` and every existing
`image_blob_digest` check unchanged. Retain archive compressed/expanded caps,
streamed hashing/budget checks, path canonicalization, duplicate tar-member
refusal, regular/directory restrictions, truncation/size checks, exact tag,
config image identity, source/owner/edition identity, OCI index/manifest/layer
binding, and existing non-blob legacy compatibility. Do not add a blanket
allowance for `blobs/sha256/*`, arbitrary metadata JSON, parent directories,
symlinks/hardlinks or extra image manifests. No extraction or Docker fallback.
The required new metadata size/content checks are additive. Unrelated existing
validator hardening is not reserved here.

### Smallest build-free future validation seam — not run or released

After root reserves/independently reviews the exact source hunk, a single
bounded standard-library validation invocation can test only the corrected
pure member validator against the retained Essentials archive
`local-essentials-x86_64.docker.tar.gz`. Root must first identify its exact
outer-ZIP member and pin its bytes/size/hash under the already verified
artifact. The failed controller never reached the receipt's archive hash
update; no inner archive hash is invented here. This audit did not open the
outer ZIP or execute such a validator.

Proposed one future entrypoint: `python3 <root-reviewed-build-free-driver.py>
--controller-revision <corrected-full-pin> --retained-artifact
/tmp/riauth-wave30-container-37061329815/artifact.zip --artifact-sha256
d150a5950cbe6ff2dc4399614eca2bf4dfa274ac0db755739ef328a29f9c76b6
--selected-image sha256:6c1248f6e88133330785d3df42b8e7129027b0e3010ab29fc30e2831408a7c5a`.
This is a design placeholder, not a created driver/current command or runtime
reservation. Pinning/driver ownership/output path requires root approval.

Driver scope would be AST-extracted definitions only, without controller
main/import side effects, public config metadata and streamed opaque layer
hashes; never iterate/extract layer filesystems. Use no Docker/native tools,
network, build, daemon, listener, secrets or fixture. Prefer memory-backed
synthetic metadata/member cases, no copied large archive. Bound one process to
120seconds, existing2GiB compressed/expanded archive ceiling and8MiB metadata
capture, fixed result codes and bounded0600 evidence. A proposed2GiB remaining
margin above the existing8GiB floor must be freshly justified before any future
invocation; no slot or threshold change is made in this audit.

Required future cases in that one invocation:

| Future case | Required outcome |
| --- | --- |
| Retained selected Essentials archive | strict validation succeeds, exactly six derived V1 blobs; no load/UID/fixture inference |
| One extra correctly content-addressed JSON blob with plausible V1 shape | final no-extra refusal |
| Metadata at a wrong digest path / altered payload under old path | digest/content binding refusal |
| Rehash changed id, parent, created, os, top config or unknown field | expected-member/content refusal, even if internally self-consistent |
| Reordered/mismatched DiffIDs/layers or metadata chain from another config | selected chain/config binding refusal |
| Malformed/duplicate-key metadata, JSON scalar/list, bool-vs-number substitution | exact generated-byte/schema refusal |
| Omit one expected blob, duplicate tar member, nonregular/path traversal member | required metadata/member refusal |
| Existing wrong tag/config/OCI binding and cap/truncation negatives | existing refusal behavior preserved |

Synthetic cases are definitions only; none was built, loaded or validated.
An unexpected positive retained-archive failure must stop with a fixed public
reason, preserving original evidence; no schema loosening, automatic repair,
new image build or broad cohort rerun is implied.

### Actual review checks and handoff limit

Actual work: complete allowlisted public metadata JSON reading, outer/file
SHA-256 identity checks, all54 step cleanup flags and888 resource sample
summaries, immutable controller body/byte comparison, official read-only API
and raw primary-source fetches, and source/schema reasoning. The web tool could
not access the tag API; a bounded read-only standard-library request to the
same official API succeeded. No authentication, image/layer download, private
protocol output or daemon query was used. Read output initially exceeded the
tool cap; subsequent bounded field/body reads covered the required material.
The observed parent-chain table is metadata analysis, not a run of the proposed
validator or proof of recomputed expected IDs.

Recommendation: reserve the narrow strict metadata-binding correction for
source review and then the single build-free retained-archive/negative seam.
**Do not accept the failed cohort as passed.** No production defect, UID/CLI/
fixture outcome or shared gate completion is established by this diagnosis.
No whole A09/Linux ARM64/release/tenant completion claim is made. Original A09
and O07 gates remain root-owned; I04's accepted DONE disposition and all prior
PASS/failure receipts are untouched. The earlier Desktop UID failures remain
failures, not repaired by the native-host export diagnosis. Source/license
staleness already named in the actual receipt remains a separate limit.

Only this report is appended. No source, workflow, helper, product, existing
other report, private evidence, manifest or accepted/main file was edited.
No validator/helper/native/Docker/Cargo/service/fixture/browser/desktop or slot
operation, source alignment/reset/merge, deletion, worker contact/new task/
worker/worktree/managed shell, status mutation or push occurred. The sole
network operations were read-only official Moby source/API reads. Receipt,
route-header, credential, PAM, Group and60-second contracts are unchanged.
Static report checks and exact prefix/scope results are recorded below after
completion; they cannot supply unperformed runtime evidence.

Final actual checks for this appendix: `python3 scripts/check-docs.py` exited0;
`git diff --check` passed; Markdown fences balanced and the prospective Python
pseudocode parsed as an AST without execution/import. The controller AST/pin
check passed. The original17994-byte report prefix (SHA-256
`0e20c5083bbc8200b315eada712eccb59359aa14dde35bb06d4d141a4f3ec85c`)
remains byte-exact. The only changed file is this reserved report; no source
or runtime validation occurred. Final staged whitespace/scope checks are
required before the report-only commit.


## Reserved v28 legacy metadata binding — source implementation, 2026-10-02

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, original A09
`506e3979-a590-4af3-8fa8-ee90d3a517f2`, existing supporting worktree
`e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`. Root reservation
`wave30_A09_v28_legacy_metadata_binding` authorizes this source-only script
change and append-only report. No validator/controller import, native tool,
archive inspection or execution reservation accompanies it. I04's original
DONE disposition and primary assignments are unchanged.

### Exact alignment and authorized conflict resolution

Own HEAD05187 was merged with fixed published main
`74e106b819e7186d0ac41964cedbe8217fa7e881`, preserving both histories.
The first merge stopped on one add/add source conflict in
`tests/saml_source_peer.rs`; no other file was unmerged. Root independently
reviewed and explicitly authorized choosing the exact published blob, already
accepted as unused-constant removal23814ab. Deleting only the unique69-byte
`const POST` line from own39539-byte file yields the entire39470-byte published
file, SHA-256
`610e29c32d7eed456c33493aea2bb1eab52fc7a5791d36db5686c59532fc5512`.
The root-selected blob was restored for that sole conflict, staged and the
merge continued. No test operation/assertion/native runner changed, and the
old constant remains in preserved source history. Merge commit:
`effbeacbd1231aa848588cb0aa0870c6b5a0bf6f`.

Before the helper edit, `src/`, `crates/`, every `tests/` path, Cargo manifests/
lock, toolchain, `.cargo/`, build.rs, cohort helper and workflow all matched
fixed74 exactly. The independent review report also matched fixed74 exactly,
including the full prior42881-byte prefix. There was no report conflict.
The helper matched accepted architecture source
`d84d753912c8d920dd3f005d03024a9ad28841e4` byte-for-byte:
92964bytes, SHA-256
`45353945c6d867039d29ada0f110cf2c2b85a907d179bb9049268046875ffc73`.
Workflow4573bytes retains SHA-256
`9af6d1d40543bd9604d7105730b97cec9c665d0f197548e50ba9a87eeb6007d4`.
No reset/rebase or unrelated conflict resolution occurred.

One prior read-only Git lookup failed: the audit attempted to read
`scripts/check-local-container-cohort.py` from own05187, where that path did
not yet exist. Git exited128 and the inspection script exited1. Subsequent
fixed74/d84 blob reads established the correct baseline; no source was changed
by that failed lookup. This is a provenance/read failure, not a product or
container outcome.

### Immutable code slice and selected-platform contract

Code commit `59f5c6b465a380d270acf68ba7a1719a41998673` changes only
`scripts/check-local-container-cohort.py`:117 added lines, no removals. The
result is99566bytes, SHA-256
`c5ede6313bf967ab1ab42429e74fb4ca9fcc03c82bd0da2e4433e2b129412837`.
The five-line hook is at1090–1094; the new111-line method
`Cohort.expected_v28_legacy_members` is at1104–1214, plus its separating blank
line. No new imports were added. The method receives settings, ordered layers,
TarInfo members, streamed hashes, bounded payloads, edition and the current
ordinary allowlist; it returns a new verified metadata-name set without
mutating that ordinary set or changing archive contents.

The accepted code calls its closed platform choice `self.selected`, initialized
from `closed_platform(args.architecture)`. The new method uses that existing
`self.selected["oci_arch"]` and `self.selected["source"]`, rather than inventing
`self.platform` or introducing an x86 fallback. Existing PLATFORMS/selection,
all architecture helpers and workflow remain exact:

| Existing selection | Bound OCI architecture | Bound product revision | New runtime evidence |
| --- | --- | --- | --- |
| x86_64 | amd64 | `b619fe25269ccc150e473bbcde47cdb3623ef810` | none |
| arm64 | arm64 | `9a819317efb3a13fa27cd86f884be2be00898fc0` | none |

This is a literal adaptation of the approved finite serializer profile to the
existing selected values. It supplies no synthetic ARM pass or claim that an
unobserved ARM export has already matched the strict schema.

The hook activates only with daemon ServerVersion28.0.4 and canonical SHA-256
blob layer names. Existing unknown-extra refusal remains for other formats.
The helper requires exactly the selected config's six-field top schema,
linux/selected architecture, a list-valued history, exactly rootfs.type/diff_ids,
1–128 canonical distinct DiffIDs and identical ordered layer names. Each layer
must be regular with its already-streamed hash matching the corresponding
DiffID before ChainID derivation. Wrong/missing/reordered layers fail closed.

The exact typed nine-field runtime config binds fixed user, command,
entrypoint, working directory, volume, exposed port, PATH, boolean ArgsEscaped
and four source/edition/owner/revision labels. A bounded-depth type-aware
comparison to that fixed expected structure rejects bool/int equivalence,
unknown fields and wrong dict/list/null shapes. Owner is canonical ASCII
`a09-` plus32 lowercase hex characters; edition is essentials/platform.
Creation time must be valid canonical UTC RFC3339Nano, with optional1–9
fractional digits ending nonzero. Existing datetime parses only the calendar/
whole-second portion; original fractional digits are retained. There is no
wall-clock change, sleep, timestamp repair or normalization.

The zero ContainerConfig retains all17 mandatory fields in pinned Go
structure order. Runtime Config overlays only the fixed expected values,
inserts ExposedPorts and ArgsEscaped in their pinned positions, and explicitly
sorts its Labels map. Other two map fields have one fixed key and an empty
object. All constructed strings are the approved ASCII profile; there are
no floats, arbitrary JSON fragments or unreviewed general Go-JSON substitute.
Compact encoding uses the existing json module without recursively sorting
struct keys or adding a newline.

For each selected DiffID, the method derives the ordered ChainID, then hashes
CreateID's sorted top-level map while preserving nested struct order. It uses
epoch/zero ContainerConfig for intermediates and selected created/runtime
config/architecture/OS for the top. The previous **computed** V1 ID is prefixed
for the next CreateID parent; saved metadata uses the unprefixed parent.
Intermediate OS is added only after computing that ID, as the pinned exporter
requires. Generated saved V1 bytes in declared order determine the expected
metadata filename/content hash. Each required member must be new/distinct,
regular, have an integer matching size within LOG_CAP, matching streamed hash
and exact bounded payload bytes. It never derives names/IDs from observed
extra JSON. Missing, duplicate, unknown/malformed/re-encoded or wrong-parent
metadata cannot acquire allowlist membership through a shape check; exact
bytes also prevent lossy duplicate-JSON-key acceptance.

The original final `set(hashes) <= allowed` and blob digest loop are exact.
Original path/member/duplicate/regular/truncation/cap/deadline/tag/config/OCI
checks, non-blob legacy handling, archive producer, image remove/load flow,
UID/fixture/public-CLI protections and cleanup remain unchanged. There is no
image content rewrite, filtering, deletion, expanded extra-JSON allowance or
fallback to another architecture/daemon.

### Actual static proof and preserved methods

Actual checks were source reading and standard-library AST parsing, a static
new-method call/signature check (seven explicit call arguments; self plus
seven formal arguments), unchanged-import comparison, exact whole-file
reversal, protected-path Git comparisons and whitespace checks. No module was
imported or method executed. Removing only the five-line hook and new helper/
separator reproduces the entire92964-byte d84/fixed74 source and its SHA-256.
That is stronger than a normalized-AST equivalence claim.

The aligned class has51 existing methods. Besides the authorized hook in
validate_image_tar, all50 other existing method source bodies are byte-exact,
covering the reserved protected control methods as well as the remaining
methods: __init__, save, capacity, sample_loop, check_budget,
process_identity, kill_group, command, docker, inspect, absent, prove_absent,
git, source_check, http, github_json, native_transport, elf, capabilities,
daemon_setup, recover_builder, create_volume, confirm_volume,
create_container, confirm_container, remove_container, tool, image_identity,
build_images, inspect_product, maintenance, shell, new_fixture, start_app,
stop_app, client, login, mutate, token, revoked, snapshot, revoke_copy,
offline_hashes, direct_refusal, plan, config_refusals, handoff,
platform_refusal_fixture, fixture_gate and cleanup. Whole-source reversal also
preserves every top-level import/function/constant and the closed platform map.
All protected production/test/manifest/toolchain/workflow paths still match
fixed74 after the code commit.

The first static reversal audit failed an assertion because its extraction
included two separator newlines instead of the one actually added. Only the
audit slice was corrected; the source was unchanged. The corrected audit
passed exact byte reversal, all50 method comparisons, signature/import/parser
checks and protected-path/single-file scope. This is an audit-check failure,
not a validator execution or evidence that the new code accepts the archive.
Code commit staging `git diff --check` passed; its sole changed file was the
reserved script. No Rust/Python typecheck, runtime test or compile was run.

### Retained archive identity and pending validation boundary

Root supplies the exact retained ZIP member
`local-essentials-x86_64.docker.tar.gz`,48934031bytes, SHA-256
`e9e1291895a1cf6c894147acaa0b4efca4a885d4f07dc88f5aee2152c9604905`,
inside artifact11252311290's49206215-byte ZIP with SHA-256
`d150a5950cbe6ff2dc4399614eca2bf4dfa274ac0db755739ef328a29f9c76b6`.
The earlier appendix independently rehashed the outer ZIP/public metadata;
this source phase did **not** open the ZIP/member/layers or independently
recompute that inner archive identity. Attribution stays root-supplied.

The earlier bounded build-free validation proposal remains design only.
Immutable source is now ready for root's separate review and reservation of
that exact retained-archive/synthetic negative seam. No driver, harness,
validator execution, import, probe, download or other runtime was created or
run in this source phase; no next execution command is authorized here.
Strict real-byte agreement with the selected exporter remains unverified
until that separately reserved validation occurs. A mismatch must stop and
retain its precise public refusal, not silently loosen the profile.

Actual container run37061329815 remains FAILED at images/
image_tar_unreferenced_member. Successful builds/save do not imply a validated
archive, image reload, UID/mount gate, E→P→E fixture, shared gate, release or
whole A09 closure. All those later checks remain unreached in that receipt.
Earlier Desktop UID failures and I04 failed/PASS receipts and DONE disposition
are unchanged. Root alone owns future integration, runtime and original task
status. No main/push/status/worker contact/new resource/slot/cache deletion,
network query, Docker/native/Cargo/compiler/provider/browser/service or
unreviewed source edit occurred.

Final actual report checks: `python3 scripts/check-docs.py` exited0 and
`git diff --check` passed. The complete42881-byte published report prefix
(SHA-256 `f6c6c39cef63bcd3742f6d84c4ce29ae90c84d7efda4e9a1a8092a4fd84f610e`)
is exact; fences balance, the code blob equals immutable59f5, and protected
production/test/manifest/toolchain/workflow comparisons to fixed74 remain empty.
The source commit contains only the reserved script; this report commit contains
only this append. Merge-parent history is verified. Final staged whitespace
checks passed before committing. These static checks supply no archive/runtime
outcome, import/execution authorization or task closure.


## Root strict metadata source review, 2026-10-02

Root read the full59f5c6b117-line diff/new111-line method, original complete
validator and e603f9b198-line receipt. Removing only the new method/separator
and exact five-line hook reconstructs everyd84 source byte. Selected native
architecture, exact Go V1/config/ordered-chain derivation and final no-extra/
digest checks are accepted as source only. Independent source review and
complete one build-free validation design remain pending. The actual container
receipt remains failed and its UID/fixture gates unreached; no retry is released.

## Root independent source-review acceptance, 2026-10-03 local

Root read all257 lines of independent report
`370c141412aa99eab42545d076f92022b537ab53`, including the full-helper
read/reversal evidence, upstream access limits and exact profile boundaries. No
concrete source blocker was reported. The strict59f metadata source remains
accepted for one separately reviewed build-free archive/synthetic validation.
No candidate serializer/archive validation or Docker runtime ran in this review;
the retained370613 container cohort remains failed and all downstream gates
remain unreached. Its complete payload/controller design is still pending.


## Build-free validation bundle design — no implementation/execution, 2026-10-03

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, original A09
`506e3979-a590-4af3-8fa8-ee90d3a517f2`, existing supporting WTe1.
Reservation `wave30_A09_legacy_archive_build_free_design` authorizes ONLY
this report appendix. The complete54888-byte prior report remains exact.
The following two complete source listings are **design text**, not installed
files, imports, case execution, a release request or a claimed result. Root
must independently review these complete bodies, the independent source review
and candidate source before separately reserving staging/one invocation.

### Inputs, transport and exact assembly boundary

Use only committed59f5 source text99566bytes, SHA-256
`c5ede6313bf967ab1ab42429e74fb4ca9fcc03c82bd0da2e4433e2b129412837`,
which is byte-equivalent to accepted a645449. Current helper/workflow4573bytes
remain unchanged; no source alignment is required or performed in this design.
The source content hash is authoritative if a later mutable working copy has
changed: refuse rather than import another revision. Root can separately stage
an exact immutable source copy before runtime; no Git child is part of this
one-child design. This version's supervisor reads the existing helper and pins
its complete bytes before freezing controller.txt. It requires that helper
still be exact59f5; otherwise the proposed command refuses before launch.

Outer transport is the sole retained artifact ZIP at
`/tmp/riauth-wave30-container-37061329815/artifact.zip`:49206215bytes, SHA-256
`d150a5950cbe6ff2dc4399614eca2bf4dfa274ac0db755739ef328a29f9c76b6`.
Only the unique literal member `local-essentials-x86_64.docker.tar.gz` may be
streamed to a fresh private regular file:48934031bytes, SHA-256
`e9e1291895a1cf6c894147acaa0b4efca4a885d4f07dc88f5aee2152c9604905`.
The design hashes the opened outer descriptor before opening ZipFile, checks
one matching member/declared size/unencrypted regular-or-unspecified ZIP mode,
streams bounded bytes without extract/extractall, checks the inner hash and
then checks outer descriptor identity/size/times. An unspecified ZIP Unix mode
is not interpreted as a filesystem object: the target is always an exclusive
0600 regular file. Any explicit nonregular ZIP mode refuses. Other members
are never opened. All transport pins precede candidate calls.

Receipt16872bytes SHAab7c5ce7…86ab7d and public-small metadata17651bytes
SHA430471f2…e5b4 are frozen as0600 copies after exact hash checks. Root owner,
tag and image are bound to literal published public values, not invented peers.
The actual selected image is
`sha256:6c1248f6e88133330785d3df42b8e7129027b0e3010ab29fc30e2831408a7c5a`.
This is solely the retained x86 export; no ARM fixture may be reported as an
actual ARM pass.

AST assembly selects only complete original Refusal/require/digest definitions
and complete Cohort.check_budget/validate_image_tar/expected_v28_legacy_members.
It constructs an ArchiveSlice class with no constructor or other Cohort method,
binds existing literal IMAGE_CAP=2GiB/LOG_CAP=8MiB/owner-label/repository and
stdlib modules, and allocates with object.__new__. No cohort module is imported,
no top-level imports/main/constructor, HTTP/Docker/native/CLI method or other
source definition is executed. The four closed source call hooks remain
require, digest, check_budget and expected_v28_legacy_members. Their bodies and
the full validator come from the pinned source AST; no cap/budget/hash helper
is replaced. ArchiveSlice uses the existing x86 selected architecture/source,
real monotonic deadline, an ordinary stop Event and fixture_deadline=None.

The complete original validator is the decisive first positive. A subsequent
independent bounded outer-tar pass streams opaque regular-file hashes and
retains ONLY the twelve known public metadata payloads, checked against the
pinned public metadata sizes/hashes. It never parses or extracts a layer's
filesystem; the original validator's small opaque-layer buffers likewise are
not examined or emitted. One explicit helper call must return exactly the six
externally pinned V1 member names. Its actual count/canonical set digest is
fsynced before comparing to that literal expected set; names are not output. Its returned set is not reused as its own
oracle.

### Finite synthetic seam and independent fixture construction

Only after both retained positive observations pass, use one small two-layer
synthetic tar fixture. Its tiny layer strings are framing/hash inputs, not
filesystem archives or executable images. V1 expected blobs are constructed
from **literal CreateID/V1 byte templates** and actual observed Go Config
fragments from the already pinned retained metadata, outside candidate code.
No candidate encode/exact/legacy-derivation routine constructs the fixture or
expected refusal table. A tiny baseline pass establishes that mutations begin
with a usable complete-validator input; it is not deployment/peer evidence.
The retained real export remains the independent decisive positive oracle.

The extra JSON case retains the full plausible V1 shape with an unrelated
64-hex ID. The duplicate-key case retains every decoded top field/value and
adds a duplicate equal-valued id; it tests exact bytes rather than an already
invalid shape. Duplicate/nonregular tar cases target a required legacy member.
The full case sequence below has no inflated coverage score. Each rehashed
metadata field tests a specified binding; other entries hit distinct schema,
framing, identity, cap, compatibility or original refusal boundaries. All
expected codes are literal, source-derived values. First unexpected outcome
stops without another case/correction/retry. Correctly expected negative
results are journaled, not relabeled successful operations.

| Family | Cases and required observations |
| --- | --- |
| Real input | retained_archive pass, then retained_derived_six count6/exact known set |
| Tiny control | tiny_baseline pass, with independent literal V1 templates |
| Unreferenced/digest | extra_correctly_hashed → image_tar_unreferenced_member; wrong_metadata_digest_path / altered_under_old_path → image_legacy_metadata_binding |
| Legacy binding | rehash id,parent,created,os,config,unknown → image_legacy_metadata_binding; malformed/scalar/duplicate-key metadata and missing metadata → same binding refusal |
| Tar framing | duplicate/nonregular/path → image_tar_member; truncated payload → closed tar_read_error |
| Ordered/config schema | manifest order → oci_image_manifest_binding; streamed mismatch / rootfs order/duplicate → image_legacy_layers; wrong rootfs shape /129 layers → image_legacy_rootfs; bool-as-int → image_legacy_config; top unknown → image_legacy_profile; invalid/noncanonical time → image_legacy_created |
| Limits | advertised compressed stat overflow → image_archive_size, without a2GiB physical file; oversized tar header → image_tar_expanded_cap before payload allocation; manifest payload8MiB+1 → docker_archive_manifest because it is not retained as small metadata |
| Original identity | wrong tag → image_tar_exact_tag; wrong selected image → image_tar_config_layers; OCI config/index/ref/payload → their original distinct refusal codes |
| Compatibility and final guards | unsupported daemon **with legacy extras** → image_tar_unreferenced_member; a no-legacy wrong layer digest under that profile reaches original image_blob_digest; set ordinary stop Event → original resource_monitor_refused |

This does not promise that all unknown daemon formats refuse: source retains
its earlier handling of archives without unknown extras. The selected unknown
profile case specifically refuses admission of the V1 extras. No source cap
is lowered. Compressed-stat overflow is a labeled scalar boundary probe,
not a real giant-file measurement. The8MiB+1 payload streams zeros through a
single gzip fixture; no large archive copy is generated. Calendar tests alter
synthetic config fields, not the system clock. The stop-Event test simulates
budget refusal, not an observed disk shortage.

A local read-only primary-source check of installed Python3.14 tarfile.py
_FileInFile.read (684–707) confirms that short underlying data raises ReadError
before the validator's copied-size comparison. The truncation oracle therefore
uses closed tar_read_error and does not manufacture image_tar_truncated.
That explicit controller code remains unmodified and allowed in observed
outcomes; no fake source reader is substituted to force it. No tar/library
function was invoked during this design read.

### Future staging/ownership, deadlines and result order

Root must separately approve creation of the one fresh private directory
`target/a09-v28-legacy-build-free-59f5c6b-v1` beneath this own worktree, mode0700,
with the exact marker and two0600 listings below. None exists by action of this
design. Staging is a future root-owned operation, not a shell/script run here.
Refuse preexisting runtime output names, symlinked/foreign/multiply linked
inputs or wrong marker/permissions. Root must independently hash both exact
listings before invoking them; their content hashes are recorded after the
listings. Source/receipt/metadata snapshots are exclusive0600 files; the inner
archive is the only sizable new file. The existing outer artifact is retained.

Propose exactly one Python parent and one owned Python child. Both use
`-I -S -B` to exclude environment/user-site/project import injection and bytecode
cache writes; only the reviewed stdlib and explicitly selected candidate AST
definitions execute. No import of a staged/project shadow module is permitted. No Git/pkg-config/
compiler/native/library/CLI/Docker/service/provider helper is a child. Require
fresh minimum10GiB free across output and retained-input filesystems before
launch, retaining2GiB planning allowance above8GiB floor. Copying the49MB inner,
three small snapshots, one reused tiny case file and bounded captures is the
expected disk footprint; this is an estimate, not measured peak/RSS or a disk
quota. The parent samples every2seconds and stops below8GiB; child budget checks
occur between transport/tar blocks and fixture steps. Concurrent drain can
outpace sampling. Root may later approve a measured allowance; this proposal
never reduces its guard automatically.

The120-second child deadline begins at parent start; setup consumes that
budget. Parent has125seconds total, leaving up to5seconds for owned-group
termination, joined IO and reaping. Require the locally reviewed Python3.14.6 and reject platforms without
waitid/WNOWAIT/getpgid/getsid rather than use a new process provider. No version
command is part of this plan; the future parent reads its own sys.version_info. The leader remains
unreaped while any group signal is needed; exact PID=PGID=SID is checked.
Signals never target a reaped/reused group. A remaining group or failed join
cannot be reported passed. SIGINT/SIGTERM use the same owned cleanup. These
are logical deadlines/checks, not a claim that fsync or uninterruptible kernel
IO has a measured hard upper bound. Parent SIGKILL cannot complete its own
cleanup; only actual joined evidence could establish completion.

Child stdout/stderr are each capped64KiB, retained0600 without public replay.
Case/pin observations are closed JSONL capped128KiB, fsynced **before** their
oracle comparisons. Setup/pin guards can refuse before any candidate call.
The parent's closed JSONL is capped32KiB; it fsyncs actual numeric exit/signal,
minimum sampled free bytes, elapsed time, capture sizes and owned cleanup
**before** comparing the final child completion envelope. The parent reads the bounded closed child journal/envelope even for a nonzero
exit, to retain partial count and a validated first-failure/outcome. Public output
is only fixed result/failure enums, numeric counts and cleanup booleans. No raw exception,
path, dynamic member name, protocol/data byte or opaque layer content appears
in case output. Files stay retained under the owned private directory; no
cache deletion, image rewrite/filter, archive cleanup or global prune occurs.
One reused tiny case file preserves its first unexpected failing input; prior
expected cases are reconstructible from exact design text and per-case hashes.

### Complete future payload listing

```python
# DESIGN ONLY: reviewed future payload; not installed or executed in this phase.
import ast, copy, datetime, gzip, hashlib, io, json, os, pathlib, re
import stat, sys, tarfile, threading, time, zipfile

ROOT = pathlib.Path(__file__).resolve().parent
PUBLIC = pathlib.Path("/tmp/riauth-wave30-container-37061329815")
DEADLINE = float(sys.argv[1])
G = 1024 ** 3
SOURCE = "b619fe25269ccc150e473bbcde47cdb3623ef810"
OWNER = "a09-2ad626a6017c4815a4bd2b89d496ed4e"
TAG = "riauth-local/" + OWNER + ":essentials"
IMAGE = "sha256:6c1248f6e88133330785d3df42b8e7129027b0e3010ab29fc30e2831408a7c5a"
MEMBER = "local-essentials-x86_64.docker.tar.gz"
INNER_SIZE = 48934031
INNER_SHA = "e9e1291895a1cf6c894147acaa0b4efca4a885d4f07dc88f5aee2152c9604905"
PINS = {
    "controller.txt": (99566, "c5ede6313bf967ab1ab42429e74fb4ca9fcc03c82bd0da2e4433e2b129412837"),
    "receipt.json": (16872, "ab7c5ce7b87e672db255edc4393ccb27348cb777b14f5f7880f87a68a986ab7d"),
    "small.json": (17651, "430471f20c63eb533003415c19c3d19c8f83b542af7acb75621cfe150040e5b4"),
}
LEGACY = frozenset("blobs/sha256/" + x for x in (
    "f2c95b27390582a885e249206a4731e5d6100ff9a77cd39d9bb0eeec3afd422c",
    "bfac4c35c38484d57b91e117ef48571d183808557de737290a2db1033a858be9",
    "42e47bd0cf542b5dc4810ade3689b6809f7b6f2b7a0dd9421cf841a7fc70cc6a",
    "403d14cad84ab235eb47caa1c6dc4633ed2666b726a8fcacf871de50f9bb72ff",
    "1de23d2d3e2ecafe29ef6e2cfbc28f43a2e220b87aa6fdd0aa2bbdf06da2cc24",
    "e9c7179eb746fed1f7cd9b4a0111ac84b505bb79166553e1bbd3aa8f481b05b5"))
REFUSALS = frozenset((
    "image_archive_size", "image_tar_member", "image_tar_expanded_cap", "image_tar_truncated",
    "docker_archive_manifest", "image_tar_exact_tag", "image_tar_config_layers", "saved_config_identity",
    "saved_oci_index", "oci_manifest_digest", "oci_manifest_payload", "oci_image_manifest_binding",
    "image_tar_unreferenced_member", "image_blob_digest", "image_legacy_profile", "image_legacy_rootfs",
    "image_legacy_layers", "image_legacy_config", "image_legacy_created", "image_legacy_metadata_binding",
    "resource_monitor_refused", "controller_deadline", "fixture_deadline"))
CASE_ORDER = (
    'retained_archive', 'retained_derived_six', 'tiny_baseline', 'extra_correctly_hashed',
    'wrong_metadata_digest_path', 'altered_under_old_path', 'rehash_id', 'rehash_parent',
    'rehash_created', 'rehash_os', 'rehash_config', 'rehash_unknown',
    'duplicate_json_keys', 'malformed_metadata', 'scalar_metadata', 'missing_metadata',
    'duplicate_member', 'nonregular_member', 'path_member', 'manifest_layer_order',
    'streamed_layer_mismatch', 'rootfs_layer_order', 'rootfs_duplicate', 'rootfs_schema',
    'bool_as_int', 'top_unknown_schema', 'invalid_calendar', 'noncanonical_time',
    'layer_count_129', 'compressed_stat_cap', 'expanded_header_cap', 'small_payload_cap',
    'truncated_payload', 'wrong_tag', 'wrong_config_image', 'wrong_oci_config',
    'wrong_oci_index', 'wrong_oci_reference', 'missing_oci_payload', 'unsupported_daemon_with_legacy',
    'original_blob_digest_loop', 'original_resource_budget',
)
CASE_NAMES = frozenset(CASE_ORDER)
SETUP_NAMES = frozenset(("setup", "transport", "assembly", "metadata_read", "synthetic_setup"))
PHASE = "setup"
FIRST = None
COUNT = 0
JOURNAL = None


def H(data):
    return hashlib.sha256(data).hexdigest()


class DriverRefusal(Exception):
    pass


def budget():
    if time.monotonic() >= DEADLINE:
        raise DriverRefusal("driver_deadline")
    s = os.statvfs(ROOT)
    if s.f_bavail * s.f_frsize < 8 * G:
        raise DriverRefusal("driver_disk_floor")


def fresh(name):
    return os.fdopen(os.open(ROOT / name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600), "wb")


def record(data):
    raw = (json.dumps(data, sort_keys=True, separators=(",", ":")) + "\n").encode()
    if JOURNAL.tell() + len(raw) > 128 * 1024:
        raise RuntimeError("closed journal cap")
    JOURNAL.write(raw); JOURNAL.flush(); os.fsync(JOURNAL.fileno())


def read_owned(path, cap):
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW)
    with os.fdopen(fd, "rb") as f:
        s = os.fstat(f.fileno())
        if not stat.S_ISREG(s.st_mode) or s.st_uid != os.getuid() or s.st_nlink != 1 or stat.S_IMODE(s.st_mode) != 0o600 or s.st_size > cap:
            raise RuntimeError("closed input refusal")
        data = f.read(cap + 1)
        if len(data) != s.st_size:
            raise RuntimeError("closed input change")
        return data


def pinned(name):
    size, sha = PINS[name]
    data = read_owned(ROOT / name, size)
    record({"kind": "pin", "input": name, "bytes": len(data), "sha256": H(data)})
    if len(data) != size or H(data) != sha:
        raise RuntimeError("closed pin refusal")
    return data


def copy_inner():
    path = PUBLIC / "artifact.zip"
    with os.fdopen(os.open(path, os.O_RDONLY | os.O_NOFOLLOW), "rb") as f:
        before = os.fstat(f.fileno())
        if not stat.S_ISREG(before.st_mode) or before.st_uid != os.getuid() or before.st_nlink != 1:
            raise RuntimeError("closed transport type")
        h = hashlib.sha256(); n = 0
        while block := f.read(1024 * 1024):
            budget(); n += len(block); h.update(block)
            if n > 49206215:
                raise RuntimeError("closed transport cap")
        record({"kind": "pin", "input": "outer_zip", "bytes": n, "sha256": h.hexdigest()})
        if n != 49206215 or h.hexdigest() != "d150a5950cbe6ff2dc4399614eca2bf4dfa274ac0db755739ef328a29f9c76b6":
            raise RuntimeError("closed transport pin")
        f.seek(0)
        with zipfile.ZipFile(f) as z:
            matches = [x for x in z.infolist() if x.filename == MEMBER]
            if len(matches) != 1:
                raise RuntimeError("closed member cardinality")
            info = matches[0]
            kind = stat.S_IFMT(info.external_attr >> 16)
            if (info.is_dir() or info.flag_bits & 1 or kind not in (0, stat.S_IFREG)
                    or info.file_size != INNER_SIZE or info.compress_size > 49206215):
                raise RuntimeError("closed member metadata")
            with z.open(info) as incoming, fresh("inner.docker.tar.gz") as out:
                h = hashlib.sha256(); n = 0
                while block := incoming.read(1024 * 1024):
                    budget(); n += len(block)
                    if n > INNER_SIZE:
                        raise RuntimeError("closed inner cap")
                    h.update(block); out.write(block)
                out.flush(); os.fsync(out.fileno())
            record({"kind": "pin", "input": "inner_archive", "bytes": n, "sha256": h.hexdigest()})
            if n != INNER_SIZE or h.hexdigest() != INNER_SHA:
                raise RuntimeError("closed inner pin")
        after = os.fstat(f.fileno())
        if (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns, before.st_ctime_ns) != (
                after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns, after.st_ctime_ns):
            raise RuntimeError("closed transport change")
    return ROOT / "inner.docker.tar.gz"


def assembly(raw):
    tree = ast.parse(raw)
    defs = {n.name: n for n in tree.body if isinstance(n, (ast.FunctionDef, ast.ClassDef))}
    methods = {n.name: n for n in defs["Cohort"].body if isinstance(n, ast.FunctionDef)}
    names = ("check_budget", "validate_image_tar", "expected_v28_legacy_members")
    for name, argc in zip(names, (1, 4, 8)):
        node = methods[name]
        if node.decorator_list or node.args.defaults or node.args.kwonlyargs or len(node.args.args) != argc:
            raise RuntimeError("closed assembly signature")
    support = [defs["Refusal"], defs["require"], defs["digest"]]
    cls = ast.ClassDef(name="ArchiveSlice", bases=[], keywords=[], body=[methods[x] for x in names], decorator_list=[], type_params=[])
    module = ast.fix_missing_locations(ast.Module(body=support + [cls], type_ignores=[]))
    ns = {"__name__": "a09_pinned_archive_slice", "__builtins__": __builtins__, "hashlib": hashlib, "json": json, "pathlib": pathlib,
          "re": re, "tarfile": tarfile, "datetime": datetime, "time": time,
          "IMAGE_CAP": 2 * G, "LOG_CAP": 8 * 1024 ** 2,
          "OWNER_LABEL": "org.riauth.local.owner", "REPOSITORY": "Rhein-Industries/riAuth"}
    exec(compile(module, "<pinned-59f5-archive-slice>", "exec"), ns)
    obj = object.__new__(ns["ArchiveSlice"])
    obj.stop_event = threading.Event(); obj.deadline = DEADLINE; obj.fixture_deadline = None
    obj.selected = {"oci_arch": "amd64", "source": SOURCE}
    obj.owner = OWNER; obj.tags = {"essentials": TAG}
    obj.receipt = {"daemon": {"ServerVersion": "28.0.4"}}
    return ns, obj


def observe(name, expected, operation):
    global FIRST, COUNT, PHASE
    if name not in CASE_NAMES or name != CASE_ORDER[COUNT]:
        raise RuntimeError("closed case order")
    PHASE = name
    try:
        operation(); actual = "pass"
    except NS["Refusal"] as error:
        actual = error.args[0] if len(error.args) == 1 and error.args[0] in REFUSALS else "unclassified_refusal"
    except DriverRefusal as error:
        actual = error.args[0] if error.args in (("driver_deadline",), ("driver_disk_floor",)) else "unclassified_exception"
    except tarfile.ReadError:
        actual = "tar_read_error"
    except json.JSONDecodeError:
        actual = "json_decode_error"
    except Exception:
        actual = "unclassified_exception"
    COUNT += 1
    record({"kind": "case", "case": name, "actual": actual})  # fsync before comparison
    if actual != expected:
        FIRST = name
        raise RuntimeError("closed unexpected case outcome")


def maps_from_retained(path, small):
    members, hashes, payloads = {}, {}, {}
    with tarfile.open(path, "r:gz") as t:
        for member in t:
            budget(); name = member.name.rstrip("/"); members[name] = member
            if not member.isfile():
                continue
            h = hashlib.sha256(); n = 0; raw = bytearray()
            with t.extractfile(member) as f:
                while block := f.read(1024 * 1024):
                    budget(); n += len(block); h.update(block)
                    if name in small:
                        if n > small[name]["bytes"]:
                            raise RuntimeError("closed metadata size")
                        raw.extend(block)
            hashes[name] = h.hexdigest()
            if name in small:
                payloads[name] = bytes(raw)
                record({"kind": "metadata_pin", "ordinal": sorted(small).index(name), "bytes": n, "sha256": h.hexdigest()})
                if n != small[name]["bytes"] or h.hexdigest() != small[name]["sha256"]:
                    raise RuntimeError("closed metadata pin")
    if set(payloads) != set(small):
        raise RuntimeError("closed metadata presence")
    manifest = json.loads(payloads["manifest.json"])[0]
    settings = json.loads(payloads[manifest["Config"]])
    index = json.loads(payloads["index.json"])
    oci_name = "blobs/sha256/" + index["manifests"][0]["digest"][7:]
    ordinary = {"manifest.json", manifest["Config"], *manifest["Layers"], "index.json", "oci-layout", "repositories", oci_name}
    return [settings, manifest["Layers"], members, hashes, payloads, "essentials", ordinary]


def derived_oracle(args):
    got = OBJ.expected_v28_legacy_members(*args)
    # The expected set is externally pinned public metadata, not candidate output reused as an oracle.
    if type(got) is not set or len(got) > 128 or any(type(x) is not str or re.fullmatch(r"blobs/sha256/[0-9a-f]{64}", x) is None for x in got):
        raise RuntimeError("closed derived observation shape")
    actual_digest = H("\n".join(sorted(got)).encode("ascii"))
    record({"kind": "witness", "derived_count": len(got), "derived_set_sha256": actual_digest})
    if got != LEGACY:
        raise RuntimeError("closed derived set mismatch")


def fragment(raw, key):
    text = raw.decode("ascii"); marker = '"' + key + '":'
    start = text.index(marker) + len(marker)
    _, length = json.JSONDecoder().raw_decode(text[start:])
    return text[start:start + length].encode("ascii")


def J(value):
    return json.dumps(value, separators=(",", ":"), sort_keys=True, allow_nan=False).encode("ascii")


def tiny_fixture(args):
    # Independent literal Go-ID/V1 templates use observed Moby fragments, never candidate codecs.
    zero = fragment(args[4]["blobs/sha256/f2c95b27390582a885e249206a4731e5d6100ff9a77cd39d9bb0eeec3afd422c"], "container_config")
    run = fragment(args[4]["blobs/sha256/e9c7179eb746fed1f7cd9b4a0111ac84b505bb79166553e1bbd3aa8f481b05b5"], "config")
    data = [b"synthetic-layer-one\n", b"synthetic-layer-two\n"]
    d0, d1 = ["sha256:" + H(x) for x in data]
    base_id = H(b'{"container_config":' + zero + b',"created":"1970-01-01T00:00:00Z","layer_id":"' + d0.encode() + b'"}')
    base = b'{"id":"' + base_id.encode() + b'","created":"1970-01-01T00:00:00Z","container_config":' + zero + b',"os":"linux"}'
    chain = "sha256:" + H((d0 + " " + d1).encode())
    top_id = H(b'{"architecture":"amd64","config":' + run + b',"container_config":' + zero + b',"created":"1970-01-01T00:00:00Z","layer_id":"' + chain.encode() + b'","os":"linux","parent":"sha256:' + base_id.encode() + b'"}')
    top = b'{"id":"' + top_id.encode() + b'","parent":"' + base_id.encode() + b'","created":"1970-01-01T00:00:00Z","container_config":' + zero + b',"config":' + run + b',"architecture":"amd64","os":"linux"}'
    config = copy.deepcopy(args[0]); config["created"] = "1970-01-01T00:00:00Z"
    config["history"] = []; config["rootfs"]["diff_ids"] = [d0, d1]
    fs = {"blobs/sha256/" + H(raw): raw for raw in data + [base, top]}
    legacy = ["blobs/sha256/" + H(raw) for raw in (base, top)]
    return make_case(config, ["blobs/sha256/" + d0[7:], "blobs/sha256/" + d1[7:]], fs, legacy)


def make_case(config, layers, files, legacy):
    fs = dict(files); raw = J(config); image = "sha256:" + H(raw); cn = "blobs/sha256/" + H(raw)
    fs[cn] = raw
    oci = {"schemaVersion": 2, "config": {"digest": image}, "layers": [{"digest": "sha256:" + x[13:]} for x in layers]}
    on = "blobs/sha256/" + H(J(oci)); fs[on] = J(oci)
    fs["index.json"] = J({"schemaVersion": 2, "manifests": [{"digest": "sha256:" + on[13:]}]})
    fs["manifest.json"] = J([{"Config": cn, "RepoTags": [TAG], "Layers": layers}])
    return {"files": fs, "image": image, "config": cn, "oci": on, "legacy": legacy}


def change_config(case, change):
    fs = dict(case["files"]); cfg = json.loads(fs.pop(case["config"])); change(cfg)
    fs.pop(case["oci"]); fs.pop("index.json"); manifest = json.loads(fs.pop("manifest.json"))[0]
    return make_case(cfg, manifest["Layers"], fs, list(case["legacy"]))


def replace_oci(case, change):
    fs = case["files"]; oci = json.loads(fs.pop(case["oci"])); change(oci)
    name = "blobs/sha256/" + H(J(oci)); fs[name] = J(oci); case["oci"] = name
    fs["index.json"] = J({"schemaVersion": 2, "manifests": [{"digest": "sha256:" + name[13:]}]})


def write_case(case, special=None):
    budget(); path = ROOT / "case.tar.gz"
    if path.exists():
        s = path.lstat()
        if not stat.S_ISREG(s.st_mode) or s.st_uid != os.getuid() or s.st_nlink != 1 or stat.S_IMODE(s.st_mode) != 0o600:
            raise RuntimeError("closed case ownership")
        fd = os.open(path, os.O_WRONLY | os.O_TRUNC | os.O_NOFOLLOW)
    else:
        fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, "wb") as raw:
        with gzip.GzipFile(fileobj=raw, mode="wb", mtime=0, filename="") as g:
            if special in ("expanded_cap", "truncated", "metadata_cap"):
                t = tarfile.TarInfo("manifest.json")
                t.size = 2 * G + 1 if special == "expanded_cap" else (128 if special == "truncated" else 8 * 1024 ** 2 + 1)
                g.write(t.tobuf(format=tarfile.USTAR_FORMAT))
                if special == "truncated":
                    g.write(b"x" * 63)
                elif special == "metadata_cap":
                    remaining = t.size
                    while remaining:
                        budget(); n = min(remaining, 1024 * 1024); g.write(b"\0" * n); remaining -= n
                    g.write(b"\0" * ((-t.size) % 512 + 1024))
            else:
                with tarfile.open(fileobj=g, mode="w", format=tarfile.USTAR_FORMAT) as out:
                    for name, data in sorted(case["files"].items()):
                        budget(); t = tarfile.TarInfo(name)
                        if special == "nonregular" and name == case["legacy"][-1]:
                            t.type = tarfile.SYMTYPE; t.linkname = "forbidden"; out.addfile(t)
                        else:
                            t.size = len(data); out.addfile(t, io.BytesIO(data))
                    if special == "duplicate":
                        name = case["legacy"][-1]; data = case["files"][name]
                        t = tarfile.TarInfo(name); t.size = len(data); out.addfile(t, io.BytesIO(data))
                    if special == "path":
                        t = tarfile.TarInfo("../forbidden"); out.addfile(t, io.BytesIO(b""))
        raw.flush(); os.fsync(raw.fileno())
    if path.stat().st_size > 1024 * 1024:
        raise RuntimeError("closed synthetic compressed cap")
    record({"kind": "synthetic_input", "bytes": path.stat().st_size, "sha256": H(path.read_bytes())})
    return path


def run_archive(case, special=None):
    OBJ.validate_image_tar(write_case(case, special), "essentials", case["image"])


class AdvertisedOversize:
    def stat(self):
        return type("DeclaredStat", (), {"st_size": 2 * G + 1})()


def cases(base, retained_args):
    observe("tiny_baseline", "pass", lambda: run_archive(base))
    c = copy.deepcopy(base); extra = json.loads(c["files"][c["legacy"][0]]); extra["id"] = "f" * 64
    raw = J(extra); c["files"]["blobs/sha256/" + H(raw)] = raw
    observe("extra_correctly_hashed", "image_tar_unreferenced_member", lambda: run_archive(c))
    c = copy.deepcopy(base); old = c["legacy"][-1]; c["files"]["blobs/sha256/" + "0" * 64] = c["files"].pop(old)
    observe("wrong_metadata_digest_path", "image_legacy_metadata_binding", lambda: run_archive(c))
    c = copy.deepcopy(base); c["files"][c["legacy"][-1]] += b" "
    observe("altered_under_old_path", "image_legacy_metadata_binding", lambda: run_archive(c))
    mutations = (("id", "0" * 64), ("parent", "1" * 64), ("created", "1970-01-02T00:00:00Z"),
                 ("os", "other"), ("config", {}), ("unknown", True))
    for key, value in mutations:
        c = copy.deepcopy(base); old = c["legacy"][-1]; obj = json.loads(c["files"].pop(old)); obj[key] = value
        raw = J(obj); c["files"]["blobs/sha256/" + H(raw)] = raw
        observe("rehash_" + key, "image_legacy_metadata_binding", lambda: run_archive(c))
    original_top = base["files"][base["legacy"][-1]]
    duplicate_top = original_top[:-1] + b',"id":"' + json.loads(original_top)["id"].encode("ascii") + b'"}'
    for label, raw in (("duplicate_json_keys", duplicate_top), ("malformed_metadata", b'{'), ("scalar_metadata", b'false')):
        c = copy.deepcopy(base); c["files"].pop(c["legacy"][-1]); c["files"]["blobs/sha256/" + H(raw)] = raw
        observe(label, "image_legacy_metadata_binding", lambda: run_archive(c))
    c = copy.deepcopy(base); c["files"].pop(c["legacy"][-1])
    observe("missing_metadata", "image_legacy_metadata_binding", lambda: run_archive(c))
    for label, special in (("duplicate_member", "duplicate"), ("nonregular_member", "nonregular"), ("path_member", "path")):
        observe(label, "image_tar_member", lambda: run_archive(base, special))
    c = copy.deepcopy(base); manifest = json.loads(c["files"]["manifest.json"]); manifest[0]["Layers"].reverse(); c["files"]["manifest.json"] = J(manifest)
    observe("manifest_layer_order", "oci_image_manifest_binding", lambda: run_archive(c))
    c = copy.deepcopy(base); first_layer = json.loads(c["files"]["manifest.json"])[0]["Layers"][0]; c["files"][first_layer] += b"wrong"
    observe("streamed_layer_mismatch", "image_legacy_layers", lambda: run_archive(c))
    for label, change, expected in (
        ("rootfs_layer_order", lambda x: x["rootfs"]["diff_ids"].reverse(), "image_legacy_layers"),
        ("rootfs_duplicate", lambda x: x["rootfs"]["diff_ids"].__setitem__(1, x["rootfs"]["diff_ids"][0]), "image_legacy_layers"),
        ("rootfs_schema", lambda x: x.__setitem__("rootfs", {"type": "layers", "diff_ids": False}), "image_legacy_rootfs"),
        ("bool_as_int", lambda x: x["config"].__setitem__("ArgsEscaped", 1), "image_legacy_config"),
        ("top_unknown_schema", lambda x: x.__setitem__("unknown", True), "image_legacy_profile"),
        ("invalid_calendar", lambda x: x.__setitem__("created", "1970-13-01T00:00:00Z"), "image_legacy_created"),
        ("noncanonical_time", lambda x: x.__setitem__("created", "1970-01-01T00:00:00.10Z"), "image_legacy_created")):
        c = change_config(base, change); observe(label, expected, lambda: run_archive(c))
    a = copy.deepcopy(retained_args); a[0]["rootfs"]["diff_ids"] = ["sha256:" + H(bytes([i])) for i in range(129)]
    observe("layer_count_129", "image_legacy_rootfs", lambda: OBJ.expected_v28_legacy_members(*a))
    observe("compressed_stat_cap", "image_archive_size", lambda: OBJ.validate_image_tar(AdvertisedOversize(), "essentials", base["image"]))
    observe("expanded_header_cap", "image_tar_expanded_cap", lambda: run_archive(base, "expanded_cap"))
    observe("small_payload_cap", "docker_archive_manifest", lambda: run_archive(base, "metadata_cap"))
    observe("truncated_payload", "tar_read_error", lambda: run_archive(base, "truncated"))
    c = copy.deepcopy(base); manifest = json.loads(c["files"]["manifest.json"]); manifest[0]["RepoTags"] = ["wrong"]; c["files"]["manifest.json"] = J(manifest)
    observe("wrong_tag", "image_tar_exact_tag", lambda: run_archive(c))
    observe("wrong_config_image", "image_tar_config_layers", lambda: OBJ.validate_image_tar(write_case(base), "essentials", "sha256:" + "0" * 64))
    c = copy.deepcopy(base); replace_oci(c, lambda x: x["config"].__setitem__("digest", "sha256:" + "0" * 64))
    observe("wrong_oci_config", "oci_image_manifest_binding", lambda: run_archive(c))
    for label, index, expected in (
        ("wrong_oci_index", {"schemaVersion": 1, "manifests": [{}]}, "saved_oci_index"),
        ("wrong_oci_reference", {"schemaVersion": 2, "manifests": [{"digest": "invalid"}]}, "oci_manifest_digest"),
        ("missing_oci_payload", {"schemaVersion": 2, "manifests": [{"digest": "sha256:" + "0" * 64}]}, "oci_manifest_payload")):
        c = copy.deepcopy(base); c["files"]["index.json"] = J(index); observe(label, expected, lambda: run_archive(c))
    OBJ.receipt["daemon"]["ServerVersion"] = "unrecognized"
    observe("unsupported_daemon_with_legacy", "image_tar_unreferenced_member", lambda: run_archive(base))
    c = copy.deepcopy(base)
    for name in c["legacy"]:
        c["files"].pop(name)
    first_layer = json.loads(c["files"]["manifest.json"])[0]["Layers"][0]; c["files"][first_layer] += b"wrong"
    observe("original_blob_digest_loop", "image_blob_digest", lambda: run_archive(c))
    OBJ.receipt["daemon"]["ServerVersion"] = "28.0.4"
    OBJ.stop_event.set()
    observe("original_resource_budget", "resource_monitor_refused", lambda: run_archive(base))
    OBJ.stop_event.clear()


def main():
    global NS, OBJ, JOURNAL, PHASE, FIRST
    JOURNAL = fresh("actual.child.jsonl")
    requested_exit = 2
    try:
        budget(); source = pinned("controller.txt"); receipt = json.loads(pinned("receipt.json")); small = json.loads(pinned("small.json"))
        if (receipt["owner"] != OWNER or receipt["images"]["essentials"]["id"] != IMAGE
                or receipt["images"]["essentials"]["tag"] != TAG or receipt["daemon"]["ServerVersion"] != "28.0.4"):
            raise RuntimeError("closed receipt binding")
        PHASE = "transport"; inner = copy_inner()  # all transport/archive pins before candidate calls
        PHASE = "assembly"; NS, OBJ = assembly(source)
        observe("retained_archive", "pass", lambda: OBJ.validate_image_tar(inner, "essentials", IMAGE))
        PHASE = "metadata_read"; args = maps_from_retained(inner, small)
        observe("retained_derived_six", "pass", lambda: derived_oracle(args))
        PHASE = "synthetic_setup"; base = tiny_fixture(args); cases(base, args)
        if COUNT != len(CASE_ORDER):
            raise RuntimeError("closed incomplete case order")
        requested_exit = 0
    except BaseException:
        if FIRST is None:
            FIRST = PHASE
        record({"kind": "stop", "phase": PHASE, "reason": "first_unexpected_or_setup_refusal"})
    finally:
        raw = J({"schema": "riauth.a09.build-free-child/v1", "count": COUNT,
                 "first_failure": FIRST, "requested_exit": requested_exit})
        with fresh("actual.child.json") as out:
            out.write(raw); out.flush(); os.fsync(out.fileno())
        JOURNAL.close()
    return requested_exit


if __name__ == "__main__":
    try:
        result = main()
    except BaseException:
        os.write(2, b"closed_payload_finalization_refusal\n"); result = 97
    sys.exit(result)
```

### Complete future supervisor listing

```python
# DESIGN ONLY: one future owned child; this supervisor has not run.
import hashlib, json, os, pathlib, selectors, shutil, signal, stat, subprocess, sys, time

WT = pathlib.Path("/Users/dominik/orca/projects/riAuth-public-preview-local-extension-isolation-wave27")
ROOT = WT / "target/a09-v28-legacy-build-free-59f5c6b-v1"
PUBLIC = pathlib.Path("/tmp/riauth-wave30-container-37061329815")
PAYLOAD_SHA = "f6a57790e77e85153c0a2fa84f03c257fa80931621004a53d67946aa1c0db9e3"
MARKER = b"891e7443-8dac-4c1b-897f-9e53cb59c7ee:59f5c6b:build-free-v1\n"
G = 1024 ** 3
START = time.monotonic()
CHILD_END = START + 120
OUTER_END = START + 125
CANCELLED = False
CASE_ORDER = (
    'retained_archive', 'retained_derived_six', 'tiny_baseline', 'extra_correctly_hashed',
    'wrong_metadata_digest_path', 'altered_under_old_path', 'rehash_id', 'rehash_parent',
    'rehash_created', 'rehash_os', 'rehash_config', 'rehash_unknown',
    'duplicate_json_keys', 'malformed_metadata', 'scalar_metadata', 'missing_metadata',
    'duplicate_member', 'nonregular_member', 'path_member', 'manifest_layer_order',
    'streamed_layer_mismatch', 'rootfs_layer_order', 'rootfs_duplicate', 'rootfs_schema',
    'bool_as_int', 'top_unknown_schema', 'invalid_calendar', 'noncanonical_time',
    'layer_count_129', 'compressed_stat_cap', 'expanded_header_cap', 'small_payload_cap',
    'truncated_payload', 'wrong_tag', 'wrong_config_image', 'wrong_oci_config',
    'wrong_oci_index', 'wrong_oci_reference', 'missing_oci_payload', 'unsupported_daemon_with_legacy',
    'original_blob_digest_loop', 'original_resource_budget',
)
CASE_NAMES = frozenset(CASE_ORDER)
SETUP_NAMES = frozenset(("setup", "transport", "assembly", "metadata_read", "synthetic_setup"))
REFUSALS = frozenset((
    "image_archive_size", "image_tar_member", "image_tar_expanded_cap", "image_tar_truncated",
    "docker_archive_manifest", "image_tar_exact_tag", "image_tar_config_layers", "saved_config_identity",
    "saved_oci_index", "oci_manifest_digest", "oci_manifest_payload", "oci_image_manifest_binding",
    "image_tar_unreferenced_member", "image_blob_digest", "image_legacy_profile", "image_legacy_rootfs",
    "image_legacy_layers", "image_legacy_config", "image_legacy_created", "image_legacy_metadata_binding",
    "resource_monitor_refused", "controller_deadline", "fixture_deadline"))
OUTCOMES = REFUSALS | {"pass", "tar_read_error", "json_decode_error", "unclassified_exception", "unclassified_refusal", "driver_deadline", "driver_disk_floor"}


def cancel(signum, frame):
    global CANCELLED
    CANCELLED = True


def owned_read(path, cap, private=False):
    with os.fdopen(os.open(path, os.O_RDONLY | os.O_NOFOLLOW), "rb") as f:
        s = os.fstat(f.fileno())
        if not stat.S_ISREG(s.st_mode) or s.st_uid != os.getuid() or s.st_nlink != 1 or s.st_size > cap or (private and stat.S_IMODE(s.st_mode) != 0o600):
            raise RuntimeError("closed owned input")
        b = f.read(cap + 1)
        if len(b) != s.st_size:
            raise RuntimeError("closed changing input")
        return b


def create(name):
    return os.fdopen(os.open(ROOT / name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600), "wb")


def free():
    return min(shutil.disk_usage(ROOT).free, shutil.disk_usage(PUBLIC).free)


def main():
    proc = None; journal = None; logs = []; selector = selectors.DefaultSelector()
    reason = None; code = None; reaped = False; empty = False; minimum = None; status = None
    child_count = None; first_failure = None; first_outcome = None; phase = "preflight"
    try:
        if ROOT.resolve() != ROOT or pathlib.Path(__file__).resolve().parent != ROOT:
            raise RuntimeError("closed root")
        s = ROOT.lstat()
        if not stat.S_ISDIR(s.st_mode) or s.st_uid != os.getuid() or stat.S_IMODE(s.st_mode) != 0o700:
            raise RuntimeError("closed root ownership")
        if owned_read(ROOT / ".marker", len(MARKER), private=True) != MARKER:
            raise RuntimeError("closed marker")
        if tuple(sys.version_info[:3]) != (3, 14, 6):
            raise RuntimeError("closed reviewed Python version")
        if not all(hasattr(os, x) for x in ("waitid", "WNOWAIT", "WEXITED", "WNOHANG", "getpgid", "getsid")):
            raise RuntimeError("closed wait support")
        minimum = free()
        if minimum < 10 * G:
            raise RuntimeError("closed capacity preflight")
        for name in ("actual.outer.jsonl", "stdout.log", "stderr.log", "controller.txt", "receipt.json", "small.json",
                     "inner.docker.tar.gz", "case.tar.gz", "actual.child.jsonl", "actual.child.json"):
            if (ROOT / name).exists() or (ROOT / name).is_symlink():
                raise RuntimeError("closed existing output")
        journal = create("actual.outer.jsonl")
        def note(data):
            b = (json.dumps(data, sort_keys=True, separators=(",", ":")) + "\n").encode()
            if journal.tell() + len(b) > 32 * 1024:
                raise RuntimeError("closed outer journal cap")
            journal.write(b); journal.flush(); os.fsync(journal.fileno())
        note({"kind": "preflight", "free_bytes": minimum})
        payload = owned_read(ROOT / "payload.py", 64 * 1024, private=True)
        note({"kind": "pin", "input": "payload", "bytes": len(payload), "sha256": hashlib.sha256(payload).hexdigest()})
        if hashlib.sha256(payload).hexdigest() != PAYLOAD_SHA:
            raise RuntimeError("closed payload pin")
        for src, name, size, sha in (
            (WT / "scripts/check-local-container-cohort.py", "controller.txt", 99566, "c5ede6313bf967ab1ab42429e74fb4ca9fcc03c82bd0da2e4433e2b129412837"),
            (PUBLIC / "container-cohort.json", "receipt.json", 16872, "ab7c5ce7b87e672db255edc4393ccb27348cb777b14f5f7880f87a68a986ab7d"),
            (PUBLIC / "essentials-small-metadata.json", "small.json", 17651, "430471f20c63eb533003415c19c3d19c8f83b542af7acb75621cfe150040e5b4")):
            b = owned_read(src, size)
            note({"kind": "pin", "input": name, "bytes": len(b), "sha256": hashlib.sha256(b).hexdigest()})
            if len(b) != size or hashlib.sha256(b).hexdigest() != sha:
                raise RuntimeError("closed source data pin")
            with create(name) as out:
                out.write(b); out.flush(); os.fsync(out.fileno())
        for sig in (signal.SIGINT, signal.SIGTERM):
            signal.signal(sig, cancel)
        if CANCELLED or time.monotonic() >= CHILD_END or free() < 10 * G:
            raise RuntimeError("closed prelaunch refusal")
        logs = [create("stdout.log"), create("stderr.log")]
        phase = "child"
        proc = subprocess.Popen([sys.executable, "-I", "-S", "-B", str(ROOT / "payload.py"), str(CHILD_END)],
            cwd=ROOT, env={"PATH": "/usr/bin:/bin", "LANG": "C", "LC_ALL": "C", "PYTHONDONTWRITEBYTECODE": "1"},
            stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
        if os.getpgid(proc.pid) != proc.pid or os.getsid(proc.pid) != proc.pid:
            raise RuntimeError("closed child identity")
        note({"kind": "launch", "pid": proc.pid, "pgid": proc.pid})
        for stream, log in zip((proc.stdout, proc.stderr), logs):
            os.set_blocking(stream.fileno(), False); selector.register(stream, selectors.EVENT_READ, log)
        next_disk = time.monotonic(); terminated = None; killed = False
        while True:
            now = time.monotonic()
            if now >= next_disk:
                minimum = min(minimum, free()); next_disk = now + 2
                if minimum < 8 * G:
                    reason = "disk_floor"
            if CANCELLED:
                reason = "cancelled"
            if now >= CHILD_END and reason is None:
                reason = "child_deadline"
            if reason is not None and terminated is None:
                if os.getpgid(proc.pid) != proc.pid or os.getsid(proc.pid) != proc.pid:
                    raise RuntimeError("closed cleanup identity")
                os.killpg(proc.pid, signal.SIGTERM); terminated = now
            if terminated is not None and now >= terminated + 1 and not killed:
                if os.getpgid(proc.pid) != proc.pid or os.getsid(proc.pid) != proc.pid:
                    raise RuntimeError("closed cleanup identity")
                os.killpg(proc.pid, signal.SIGKILL); killed = True
            for key, events in selector.select(timeout=0.05):
                block = os.read(key.fileobj.fileno(), 65536)
                if not block:
                    selector.unregister(key.fileobj); key.fileobj.close(); continue
                log = key.data
                if log.tell() + len(block) > 64 * 1024:
                    reason = "capture_cap"; block = block[:max(0, 64 * 1024 - log.tell())]
                log.write(block)
            status = os.waitid(os.P_PID, proc.pid, os.WEXITED | os.WNOHANG | os.WNOWAIT)
            if status is not None and not selector.get_map():
                break
            if now >= OUTER_END - 1:
                reason = reason or "outer_deadline"
                break
        phase = "join"
    except BaseException:
        reason = reason or "setup_or_supervisor_refusal"
    finally:
        # Never poll/wait/reap the leader before any required group signal; its PID pins ownership.
        if proc is not None:
            try:
                status = os.waitid(os.P_PID, proc.pid, os.WEXITED | os.WNOHANG | os.WNOWAIT)
                if status is None:
                    if os.getpgid(proc.pid) != proc.pid or os.getsid(proc.pid) != proc.pid:
                        raise RuntimeError("closed cleanup identity")
                    os.killpg(proc.pid, signal.SIGKILL)
                code = proc.wait(timeout=max(0.01, OUTER_END - time.monotonic())); reaped = True
                try:
                    os.killpg(proc.pid, 0)
                except ProcessLookupError:
                    empty = True
                if not empty:
                    reason = reason or "owned_group_empty_not_proven"
            except BaseException:
                reason = reason or "join_or_group_refusal"
            for stream in (proc.stdout, proc.stderr):
                if stream is not None and not stream.closed:
                    stream.close()
        selector.close()
        captures = []
        for log in logs:
            log.flush(); os.fsync(log.fileno()); captures.append(log.tell()); log.close()
        observed = {"kind": "joined", "child_exit": code, "child_signal": -code if type(code) is int and code < 0 else None,
                    "child_reaped": reaped, "owned_group_empty": empty, "minimum_free_bytes": minimum,
                    "elapsed_seconds": round(time.monotonic() - START, 3), "capture_bytes": captures, "stop_reason": reason}
        if journal is not None:
            note(observed)  # observed numeric exit/cleanup fsynced before completion oracle comparisons
            okay = False
            try:
                final = json.loads(owned_read(ROOT / "actual.child.json", 16 * 1024, private=True))
                raw_rows = owned_read(ROOT / "actual.child.jsonl", 128 * 1024, private=True).splitlines()
                if len(raw_rows) > 512:
                    raise RuntimeError("closed child row count")
                case_rows = []
                for raw in raw_rows:
                    row = json.loads(raw)
                    if type(row) is not dict or "kind" not in row:
                        raise RuntimeError("closed child row schema")
                    kind = row["kind"]
                    if kind == "case":
                        if set(row) != {"kind", "case", "actual"} or row["case"] not in CASE_NAMES or row["actual"] not in OUTCOMES:
                            raise RuntimeError("closed child case schema")
                        case_rows.append(row)
                    elif kind == "pin":
                        if (set(row) != {"kind", "input", "bytes", "sha256"}
                                or row["input"] not in {"controller.txt", "receipt.json", "small.json", "outer_zip", "inner_archive"}
                                or type(row["bytes"]) is not int or not 0 <= row["bytes"] <= 49206215
                                or type(row["sha256"]) is not str or len(row["sha256"]) != 64
                                or any(c not in "0123456789abcdef" for c in row["sha256"])):
                            raise RuntimeError("closed child pin schema")
                    elif kind == "synthetic_input":
                        if (set(row) != {"kind", "bytes", "sha256"} or type(row["bytes"]) is not int
                                or not 0 <= row["bytes"] <= 1024 * 1024 or type(row["sha256"]) is not str
                                or len(row["sha256"]) != 64 or any(c not in "0123456789abcdef" for c in row["sha256"])):
                            raise RuntimeError("closed synthetic row schema")
                    elif kind == "metadata_pin":
                        if (set(row) != {"kind", "ordinal", "bytes", "sha256"} or type(row["ordinal"]) is not int
                                or not 0 <= row["ordinal"] < 12 or type(row["bytes"]) is not int or not 0 <= row["bytes"] <= 8 * 1024 ** 2
                                or type(row["sha256"]) is not str or len(row["sha256"]) != 64
                                or any(c not in "0123456789abcdef" for c in row["sha256"])):
                            raise RuntimeError("closed metadata pin row")
                    elif kind == "witness":
                        if (set(row) != {"kind", "derived_count", "derived_set_sha256"}
                                or type(row["derived_count"]) is not int or not 0 <= row["derived_count"] <= 128
                                or type(row["derived_set_sha256"]) is not str or len(row["derived_set_sha256"]) != 64
                                or any(c not in "0123456789abcdef" for c in row["derived_set_sha256"])):
                            raise RuntimeError("closed witness schema")
                    elif kind == "stop":
                        if (set(row) != {"kind", "phase", "reason"} or row["phase"] not in CASE_NAMES | SETUP_NAMES
                                or row["reason"] != "first_unexpected_or_setup_refusal"):
                            raise RuntimeError("closed stop schema")
                    else:
                        raise RuntimeError("closed unknown row kind")
                if (type(final) is not dict or set(final) != {"schema", "count", "first_failure", "requested_exit"}
                        or final["schema"] != "riauth.a09.build-free-child/v1" or type(final["count"]) is not int
                        or final["count"] != len(case_rows) or final["first_failure"] not in CASE_NAMES | SETUP_NAMES | {None}
                        or type(final["requested_exit"]) is not int or final["requested_exit"] not in {0, 2}
                        or tuple(row["case"] for row in case_rows) != CASE_ORDER[:len(case_rows)]):
                    raise RuntimeError("closed child completion schema")
                child_count = final["count"]; first_failure = final["first_failure"]
                if first_failure in CASE_NAMES:
                    first_outcome = case_rows[-1]["actual"] if case_rows and case_rows[-1]["case"] == first_failure else "unclassified_exception"
                okay = (reason is None and code == 0 and reaped and empty and first_failure is None
                        and child_count == len(CASE_ORDER) and final["requested_exit"] == 0)
            except BaseException:
                reason = reason or "closed_child_envelope_refusal"
            note({"kind": "decision", "result": "passed" if okay else "failed_or_refused", "count": child_count,
                  "first_failure": first_failure, "first_outcome": first_outcome, "stop_reason": reason})
            journal.close()
        else:
            okay = False
        # Only closed results; no captured output, dynamic member names, paths or exception strings.
        print(json.dumps({"result": "passed" if okay else "failed_or_refused", "exit": code,
                          "count": child_count, "first_failure": first_failure, "first_outcome": first_outcome,
                          "reaped": reaped, "group_empty": empty, "stop_reason": reason}, sort_keys=True))
    return 0 if okay else 1


if __name__ == "__main__":
    try:
        result = main()
    except BaseException:
        os.write(2, b"closed_supervisor_finalization_refusal\n"); result = 97
    sys.exit(result)
```

### Immutable design-text pins and proposed one invocation

The payload listing (from its first comment through its final newline, without
Markdown fences) is 25305bytes, SHA-256 `f6a57790e77e85153c0a2fa84f03c257fa80931621004a53d67946aa1c0db9e3`.
The supervisor listing with its actual payload hash embedded is 16622bytes,
SHA-256 `af65db878cda7d529acfe2e4af1c46c3018e7bb9e3e2f42d0601460996b04385`. These are source-text identities, not files installed
or executed by this design. Root must stage those exact bytes and independently
verify the supervisor hash before its single invocation:

```sh
python3 -I -S -B "$PWD/target/a09-v28-legacy-build-free-59f5c6b-v1/supervisor.py"
```

The exact proposed marker is
`891e7443-8dac-4c1b-897f-9e53cb59c7ee:59f5c6b:build-free-v1` followed by one
newline; marker0600, directory0700, listing files0600, same current UID, no
symlinks/multiple links. No command/staging path has been created or invoked
here. Root's separate release must identify the final reviewed design commit,
both listing hashes, current capacity and absent fresh outputs. No automatic
retry, alternative source/profile/feature, unsupported-host fallback or oracle
relaxation is authorized. A finite setup or case refusal is an actual failure
of that future invocation, not an inferred production/ARM/UID defect.

Actual work in this design was pinned Git/source/public JSON reads, AST parser
checks of the listings as text, and hashing that text. The original54888-byte
report prefix,99566-byte helper and4573-byte workflow remain protected. The
outer ZIP/member/layers were not opened; no candidate/module/driver/function/
case/harness/stub/native/helper was executed or imported. No new resources,
slot, deletion, source edit, network/download/dispatch, contact/new worker/
task/worktree/managed shell, alignment/merge/main/push/status or service/browser/
desktop action occurred. Root's independent source review is still separately
attributed, not claimed by this design. Actual run37061329815 remains FAILED;
all later UID/fixture gates remain unreached. Closed I04/other rows are unchanged.

Final actual design checks: both complete fenced listings parsed as AST text;
listing byte counts/hashes, mirrored closed registry and one-Popen declaration
were checked statically, along with stdlib-only imports and isolated flags.
No future listing/function/case was executed or compiled/imported. The complete
54888-byte e603 report prefix is exact. Candidate99566/SHA c5ede631…2837 and
workflow4573/SHA9af6d1d4…007d4 remain unchanged. `python3 scripts/check-docs.py`
and final working/staged whitespace checks are run before the report-only commit;
only this report may differ. These are documentation/parser/identity checks,
not archive validation, process-supervision evidence or a runtime release.


## Root retained-archive design review (2026-10-03)

Root fully read cbf4560 payload25305B/f6a57790 and supervisor16622B/af65db87. The first actual-retained-archive check and independent pinned-member oracle precede the42-case synthetic/archive checks; no archive was opened or case run in this source phase. Scope keeps59f validator methods isolated, no constructor/Docker/native/build path, exclusive private captures and owner WNOWAIT cleanup. Root found a final post-persistence clock gap: an earlier deadline check cannot alone accept completion after the125-second bound. One narrow final-clock source design is separately reserved, with independent archive-design review active. No runtime/staging release follows from this report integration; historical image_tar_unreferenced_member failure remains failed.

## Final post-save clock design correction (source text only)

Reservation `wave30_A09_archive_final_clock_design`, project
`891e7443-8dac-4c1b-897f-9e53cb59c7ee`, original A09
`506e3979-a590-4af3-8fa8-ee90d3a517f2`, existing supporting worktree
`e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`. This appendix preserves every byte
of the complete cbf4560d3ea097e122a092c0686ec3a424724311 report:112120 bytes,
SHA-256 `fe32d99b55042f272bc7e4077ec254e8151b73b51ca6849407c738b2b2f13095`.
It supersedes only that appendix's supervisor listing for future review and
staging. The payload remains25305 bytes,
SHA-256 `f6a57790e77e85153c0a2fa84f03c257fa80931621004a53d67946aa1c0db9e3`;
all42 literal case oracles, candidate source59f5c6b, input identities, captures,
resource thresholds, process ownership and invocation proposal remain exact.
No design listing was installed, imported, compiled or executed.

The prior supervisor could decide `okay`, then stall in decision-journal
write/flush/fsync/close, and still print a successful result after OUTER_END.
The smallest correction samples `time.monotonic()` after that journal closes
(or after its absent-journal branch). At `final_clock >= OUTER_END`, it sets
`okay = False` and the existing fixed reason `outer_deadline`, so the closed
stdout result is `failed_or_refused` and the normal return is1. The added
numeric `final_elapsed_seconds` is the unrounded measured difference between
that final clock sample and START. Equality with the deadline refuses.

The joined observation and decision journal rows are retained unchanged as
pre-final observations. A fsynced decision row saying `passed` does not itself
establish final acceptance: it precedes the new deadline admission check.
A late final stdout result takes precedence, including when its
`outer_deadline` reason supersedes an earlier reason retained in the journal.
No evidence/log/regular-file write, flush, fsync, close, read, or new child
operation occurs after the final clock sample. Only the required closed JSON
stdout stream is emitted before the return; the unchanged exceptional wrapper
can emit its fixed stderr refusal and exit97. There is no later receipt rewrite
that could itself reopen the finalization gap.

This is a final admission check after saving the receipts, not a hard kernel-I/O
quota. A blocked kernel save/close may delay reaching the check; if it eventually
completes late, the check refuses. The required final stdout stream and process
exit can themselves be delayed after the sample. No guarantee of wall-clock
termination by125 seconds, interruption of uninterruptible I/O, or delivery of
a final receipt from an indefinitely blocked operation is made. The existing
120-second child limit and125-second outer controls remain the same, including
the documented limitations of user-space supervision.

### Exact supervisor delta and static reversal

The previous supervisor listing is16622 bytes,
SHA-256 `af65db878cda7d529acfe2e4af1c46c3018e7bb9e3e2f42d0601460996b04385`.
The complete replacement listing below is16833 bytes,
SHA-256 `4b6ca88c509feeb0b0331c6e1ac970ac1e07d81fde2650e3685c8a86af8cb1bd`. Counts/hashes include its first comment and final newline,
excluding Markdown fences. Its exact unified text diff is:

```diff
--- cbf4560-supervisor-design
+++ final-clock-supervisor-design
@@ -250,2 +250,6 @@
             okay = False
+        final_clock = time.monotonic()
+        if final_clock >= OUTER_END:
+            okay = False
+            reason = "outer_deadline"
         # Only closed results; no captured output, dynamic member names, paths or exception strings.
@@ -253,3 +257,4 @@
                           "count": child_count, "first_failure": first_failure, "first_outcome": first_outcome,
-                          "reaped": reaped, "group_empty": empty, "stop_reason": reason}, sort_keys=True))
+                          "reaped": reaped, "group_empty": empty, "stop_reason": reason,
+                          "final_elapsed_seconds": final_clock - START}, sort_keys=True))
     return 0 if okay else 1
```

Actual static checks in this reservation parsed both supervisor listings and
the unchanged payload as AST text only. Removing the four inserted clock/guard
lines and reversing the final output-field change reconstructs the complete
old16622-byte supervisor exactly. Separately removing the inserted assignment
and conditional AST nodes and the final JSON key/value reconstructs its entire
AST exactly (excluding parser location attributes). The payload remains
byte/hash exact, and all three ASTs contain the identical42-entry CASE_ORDER.
No prospective function or case was called to obtain those checks.

The structural control-flow check found exactly one final-clock assignment in
main's finalization block, immediately after the journal-present/absent branch.
The journal-present branch ends with `journal.close()`; the absent branch sets
`okay = False`. The post-sample conditional is precisely `final_clock >=
OUTER_END`, with only `okay = False` and `reason = "outer_deadline"` in its body.
After it, the only function calls in that finalization block are `print` and
`json.dumps`; no file operation or new observation is performed. The final
stdout `result` and normal return both use the resulting `okay`; no intervening
assignment can restore it. The return remains `0 if okay else 1`. All preceding
observations, receipt ordering, WNOWAIT/owned-group signals and reaping, caps,
case comparisons, signal handling and the fixed exceptional exit97 are exact
under whole-byte and whole-AST reversal.

One initial documentation assembly attempt stopped at its pre-write
trailing-whitespace assertion: the default three-line unified diff context
represented an empty context line by a single space. No report bytes changed
in that attempt. The displayed diff uses one context line and passes that
check; this is a diff-format correction only, with identical supervisor bytes.

### Complete corrected supervisor design listing

```python
# DESIGN ONLY: one future owned child; this supervisor has not run.
import hashlib, json, os, pathlib, selectors, shutil, signal, stat, subprocess, sys, time

WT = pathlib.Path("/Users/dominik/orca/projects/riAuth-public-preview-local-extension-isolation-wave27")
ROOT = WT / "target/a09-v28-legacy-build-free-59f5c6b-v1"
PUBLIC = pathlib.Path("/tmp/riauth-wave30-container-37061329815")
PAYLOAD_SHA = "f6a57790e77e85153c0a2fa84f03c257fa80931621004a53d67946aa1c0db9e3"
MARKER = b"891e7443-8dac-4c1b-897f-9e53cb59c7ee:59f5c6b:build-free-v1\n"
G = 1024 ** 3
START = time.monotonic()
CHILD_END = START + 120
OUTER_END = START + 125
CANCELLED = False
CASE_ORDER = (
    'retained_archive', 'retained_derived_six', 'tiny_baseline', 'extra_correctly_hashed',
    'wrong_metadata_digest_path', 'altered_under_old_path', 'rehash_id', 'rehash_parent',
    'rehash_created', 'rehash_os', 'rehash_config', 'rehash_unknown',
    'duplicate_json_keys', 'malformed_metadata', 'scalar_metadata', 'missing_metadata',
    'duplicate_member', 'nonregular_member', 'path_member', 'manifest_layer_order',
    'streamed_layer_mismatch', 'rootfs_layer_order', 'rootfs_duplicate', 'rootfs_schema',
    'bool_as_int', 'top_unknown_schema', 'invalid_calendar', 'noncanonical_time',
    'layer_count_129', 'compressed_stat_cap', 'expanded_header_cap', 'small_payload_cap',
    'truncated_payload', 'wrong_tag', 'wrong_config_image', 'wrong_oci_config',
    'wrong_oci_index', 'wrong_oci_reference', 'missing_oci_payload', 'unsupported_daemon_with_legacy',
    'original_blob_digest_loop', 'original_resource_budget',
)
CASE_NAMES = frozenset(CASE_ORDER)
SETUP_NAMES = frozenset(("setup", "transport", "assembly", "metadata_read", "synthetic_setup"))
REFUSALS = frozenset((
    "image_archive_size", "image_tar_member", "image_tar_expanded_cap", "image_tar_truncated",
    "docker_archive_manifest", "image_tar_exact_tag", "image_tar_config_layers", "saved_config_identity",
    "saved_oci_index", "oci_manifest_digest", "oci_manifest_payload", "oci_image_manifest_binding",
    "image_tar_unreferenced_member", "image_blob_digest", "image_legacy_profile", "image_legacy_rootfs",
    "image_legacy_layers", "image_legacy_config", "image_legacy_created", "image_legacy_metadata_binding",
    "resource_monitor_refused", "controller_deadline", "fixture_deadline"))
OUTCOMES = REFUSALS | {"pass", "tar_read_error", "json_decode_error", "unclassified_exception", "unclassified_refusal", "driver_deadline", "driver_disk_floor"}


def cancel(signum, frame):
    global CANCELLED
    CANCELLED = True


def owned_read(path, cap, private=False):
    with os.fdopen(os.open(path, os.O_RDONLY | os.O_NOFOLLOW), "rb") as f:
        s = os.fstat(f.fileno())
        if not stat.S_ISREG(s.st_mode) or s.st_uid != os.getuid() or s.st_nlink != 1 or s.st_size > cap or (private and stat.S_IMODE(s.st_mode) != 0o600):
            raise RuntimeError("closed owned input")
        b = f.read(cap + 1)
        if len(b) != s.st_size:
            raise RuntimeError("closed changing input")
        return b


def create(name):
    return os.fdopen(os.open(ROOT / name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600), "wb")


def free():
    return min(shutil.disk_usage(ROOT).free, shutil.disk_usage(PUBLIC).free)


def main():
    proc = None; journal = None; logs = []; selector = selectors.DefaultSelector()
    reason = None; code = None; reaped = False; empty = False; minimum = None; status = None
    child_count = None; first_failure = None; first_outcome = None; phase = "preflight"
    try:
        if ROOT.resolve() != ROOT or pathlib.Path(__file__).resolve().parent != ROOT:
            raise RuntimeError("closed root")
        s = ROOT.lstat()
        if not stat.S_ISDIR(s.st_mode) or s.st_uid != os.getuid() or stat.S_IMODE(s.st_mode) != 0o700:
            raise RuntimeError("closed root ownership")
        if owned_read(ROOT / ".marker", len(MARKER), private=True) != MARKER:
            raise RuntimeError("closed marker")
        if tuple(sys.version_info[:3]) != (3, 14, 6):
            raise RuntimeError("closed reviewed Python version")
        if not all(hasattr(os, x) for x in ("waitid", "WNOWAIT", "WEXITED", "WNOHANG", "getpgid", "getsid")):
            raise RuntimeError("closed wait support")
        minimum = free()
        if minimum < 10 * G:
            raise RuntimeError("closed capacity preflight")
        for name in ("actual.outer.jsonl", "stdout.log", "stderr.log", "controller.txt", "receipt.json", "small.json",
                     "inner.docker.tar.gz", "case.tar.gz", "actual.child.jsonl", "actual.child.json"):
            if (ROOT / name).exists() or (ROOT / name).is_symlink():
                raise RuntimeError("closed existing output")
        journal = create("actual.outer.jsonl")
        def note(data):
            b = (json.dumps(data, sort_keys=True, separators=(",", ":")) + "\n").encode()
            if journal.tell() + len(b) > 32 * 1024:
                raise RuntimeError("closed outer journal cap")
            journal.write(b); journal.flush(); os.fsync(journal.fileno())
        note({"kind": "preflight", "free_bytes": minimum})
        payload = owned_read(ROOT / "payload.py", 64 * 1024, private=True)
        note({"kind": "pin", "input": "payload", "bytes": len(payload), "sha256": hashlib.sha256(payload).hexdigest()})
        if hashlib.sha256(payload).hexdigest() != PAYLOAD_SHA:
            raise RuntimeError("closed payload pin")
        for src, name, size, sha in (
            (WT / "scripts/check-local-container-cohort.py", "controller.txt", 99566, "c5ede6313bf967ab1ab42429e74fb4ca9fcc03c82bd0da2e4433e2b129412837"),
            (PUBLIC / "container-cohort.json", "receipt.json", 16872, "ab7c5ce7b87e672db255edc4393ccb27348cb777b14f5f7880f87a68a986ab7d"),
            (PUBLIC / "essentials-small-metadata.json", "small.json", 17651, "430471f20c63eb533003415c19c3d19c8f83b542af7acb75621cfe150040e5b4")):
            b = owned_read(src, size)
            note({"kind": "pin", "input": name, "bytes": len(b), "sha256": hashlib.sha256(b).hexdigest()})
            if len(b) != size or hashlib.sha256(b).hexdigest() != sha:
                raise RuntimeError("closed source data pin")
            with create(name) as out:
                out.write(b); out.flush(); os.fsync(out.fileno())
        for sig in (signal.SIGINT, signal.SIGTERM):
            signal.signal(sig, cancel)
        if CANCELLED or time.monotonic() >= CHILD_END or free() < 10 * G:
            raise RuntimeError("closed prelaunch refusal")
        logs = [create("stdout.log"), create("stderr.log")]
        phase = "child"
        proc = subprocess.Popen([sys.executable, "-I", "-S", "-B", str(ROOT / "payload.py"), str(CHILD_END)],
            cwd=ROOT, env={"PATH": "/usr/bin:/bin", "LANG": "C", "LC_ALL": "C", "PYTHONDONTWRITEBYTECODE": "1"},
            stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
        if os.getpgid(proc.pid) != proc.pid or os.getsid(proc.pid) != proc.pid:
            raise RuntimeError("closed child identity")
        note({"kind": "launch", "pid": proc.pid, "pgid": proc.pid})
        for stream, log in zip((proc.stdout, proc.stderr), logs):
            os.set_blocking(stream.fileno(), False); selector.register(stream, selectors.EVENT_READ, log)
        next_disk = time.monotonic(); terminated = None; killed = False
        while True:
            now = time.monotonic()
            if now >= next_disk:
                minimum = min(minimum, free()); next_disk = now + 2
                if minimum < 8 * G:
                    reason = "disk_floor"
            if CANCELLED:
                reason = "cancelled"
            if now >= CHILD_END and reason is None:
                reason = "child_deadline"
            if reason is not None and terminated is None:
                if os.getpgid(proc.pid) != proc.pid or os.getsid(proc.pid) != proc.pid:
                    raise RuntimeError("closed cleanup identity")
                os.killpg(proc.pid, signal.SIGTERM); terminated = now
            if terminated is not None and now >= terminated + 1 and not killed:
                if os.getpgid(proc.pid) != proc.pid or os.getsid(proc.pid) != proc.pid:
                    raise RuntimeError("closed cleanup identity")
                os.killpg(proc.pid, signal.SIGKILL); killed = True
            for key, events in selector.select(timeout=0.05):
                block = os.read(key.fileobj.fileno(), 65536)
                if not block:
                    selector.unregister(key.fileobj); key.fileobj.close(); continue
                log = key.data
                if log.tell() + len(block) > 64 * 1024:
                    reason = "capture_cap"; block = block[:max(0, 64 * 1024 - log.tell())]
                log.write(block)
            status = os.waitid(os.P_PID, proc.pid, os.WEXITED | os.WNOHANG | os.WNOWAIT)
            if status is not None and not selector.get_map():
                break
            if now >= OUTER_END - 1:
                reason = reason or "outer_deadline"
                break
        phase = "join"
    except BaseException:
        reason = reason or "setup_or_supervisor_refusal"
    finally:
        # Never poll/wait/reap the leader before any required group signal; its PID pins ownership.
        if proc is not None:
            try:
                status = os.waitid(os.P_PID, proc.pid, os.WEXITED | os.WNOHANG | os.WNOWAIT)
                if status is None:
                    if os.getpgid(proc.pid) != proc.pid or os.getsid(proc.pid) != proc.pid:
                        raise RuntimeError("closed cleanup identity")
                    os.killpg(proc.pid, signal.SIGKILL)
                code = proc.wait(timeout=max(0.01, OUTER_END - time.monotonic())); reaped = True
                try:
                    os.killpg(proc.pid, 0)
                except ProcessLookupError:
                    empty = True
                if not empty:
                    reason = reason or "owned_group_empty_not_proven"
            except BaseException:
                reason = reason or "join_or_group_refusal"
            for stream in (proc.stdout, proc.stderr):
                if stream is not None and not stream.closed:
                    stream.close()
        selector.close()
        captures = []
        for log in logs:
            log.flush(); os.fsync(log.fileno()); captures.append(log.tell()); log.close()
        observed = {"kind": "joined", "child_exit": code, "child_signal": -code if type(code) is int and code < 0 else None,
                    "child_reaped": reaped, "owned_group_empty": empty, "minimum_free_bytes": minimum,
                    "elapsed_seconds": round(time.monotonic() - START, 3), "capture_bytes": captures, "stop_reason": reason}
        if journal is not None:
            note(observed)  # observed numeric exit/cleanup fsynced before completion oracle comparisons
            okay = False
            try:
                final = json.loads(owned_read(ROOT / "actual.child.json", 16 * 1024, private=True))
                raw_rows = owned_read(ROOT / "actual.child.jsonl", 128 * 1024, private=True).splitlines()
                if len(raw_rows) > 512:
                    raise RuntimeError("closed child row count")
                case_rows = []
                for raw in raw_rows:
                    row = json.loads(raw)
                    if type(row) is not dict or "kind" not in row:
                        raise RuntimeError("closed child row schema")
                    kind = row["kind"]
                    if kind == "case":
                        if set(row) != {"kind", "case", "actual"} or row["case"] not in CASE_NAMES or row["actual"] not in OUTCOMES:
                            raise RuntimeError("closed child case schema")
                        case_rows.append(row)
                    elif kind == "pin":
                        if (set(row) != {"kind", "input", "bytes", "sha256"}
                                or row["input"] not in {"controller.txt", "receipt.json", "small.json", "outer_zip", "inner_archive"}
                                or type(row["bytes"]) is not int or not 0 <= row["bytes"] <= 49206215
                                or type(row["sha256"]) is not str or len(row["sha256"]) != 64
                                or any(c not in "0123456789abcdef" for c in row["sha256"])):
                            raise RuntimeError("closed child pin schema")
                    elif kind == "synthetic_input":
                        if (set(row) != {"kind", "bytes", "sha256"} or type(row["bytes"]) is not int
                                or not 0 <= row["bytes"] <= 1024 * 1024 or type(row["sha256"]) is not str
                                or len(row["sha256"]) != 64 or any(c not in "0123456789abcdef" for c in row["sha256"])):
                            raise RuntimeError("closed synthetic row schema")
                    elif kind == "metadata_pin":
                        if (set(row) != {"kind", "ordinal", "bytes", "sha256"} or type(row["ordinal"]) is not int
                                or not 0 <= row["ordinal"] < 12 or type(row["bytes"]) is not int or not 0 <= row["bytes"] <= 8 * 1024 ** 2
                                or type(row["sha256"]) is not str or len(row["sha256"]) != 64
                                or any(c not in "0123456789abcdef" for c in row["sha256"])):
                            raise RuntimeError("closed metadata pin row")
                    elif kind == "witness":
                        if (set(row) != {"kind", "derived_count", "derived_set_sha256"}
                                or type(row["derived_count"]) is not int or not 0 <= row["derived_count"] <= 128
                                or type(row["derived_set_sha256"]) is not str or len(row["derived_set_sha256"]) != 64
                                or any(c not in "0123456789abcdef" for c in row["derived_set_sha256"])):
                            raise RuntimeError("closed witness schema")
                    elif kind == "stop":
                        if (set(row) != {"kind", "phase", "reason"} or row["phase"] not in CASE_NAMES | SETUP_NAMES
                                or row["reason"] != "first_unexpected_or_setup_refusal"):
                            raise RuntimeError("closed stop schema")
                    else:
                        raise RuntimeError("closed unknown row kind")
                if (type(final) is not dict or set(final) != {"schema", "count", "first_failure", "requested_exit"}
                        or final["schema"] != "riauth.a09.build-free-child/v1" or type(final["count"]) is not int
                        or final["count"] != len(case_rows) or final["first_failure"] not in CASE_NAMES | SETUP_NAMES | {None}
                        or type(final["requested_exit"]) is not int or final["requested_exit"] not in {0, 2}
                        or tuple(row["case"] for row in case_rows) != CASE_ORDER[:len(case_rows)]):
                    raise RuntimeError("closed child completion schema")
                child_count = final["count"]; first_failure = final["first_failure"]
                if first_failure in CASE_NAMES:
                    first_outcome = case_rows[-1]["actual"] if case_rows and case_rows[-1]["case"] == first_failure else "unclassified_exception"
                okay = (reason is None and code == 0 and reaped and empty and first_failure is None
                        and child_count == len(CASE_ORDER) and final["requested_exit"] == 0)
            except BaseException:
                reason = reason or "closed_child_envelope_refusal"
            note({"kind": "decision", "result": "passed" if okay else "failed_or_refused", "count": child_count,
                  "first_failure": first_failure, "first_outcome": first_outcome, "stop_reason": reason})
            journal.close()
        else:
            okay = False
        final_clock = time.monotonic()
        if final_clock >= OUTER_END:
            okay = False
            reason = "outer_deadline"
        # Only closed results; no captured output, dynamic member names, paths or exception strings.
        print(json.dumps({"result": "passed" if okay else "failed_or_refused", "exit": code,
                          "count": child_count, "first_failure": first_failure, "first_outcome": first_outcome,
                          "reaped": reaped, "group_empty": empty, "stop_reason": reason,
                          "final_elapsed_seconds": final_clock - START}, sort_keys=True))
    return 0 if okay else 1


if __name__ == "__main__":
    try:
        result = main()
    except BaseException:
        os.write(2, b"closed_supervisor_finalization_refusal\n"); result = 97
    sys.exit(result)
```

### Scope, checks and held execution

The future staging/review must use this corrected supervisor hash rather than
the old `af65db87` listing, alongside the unchanged `f6a57790` payload. The
single proposed invocation, marker, private0700 directory/exclusive0600 files,
10GiB start/8GiB floor,120-second child/125-second outer, capture bounds,
ordered receipts, owned-group cleanup and all pinned archive/source inputs are
unchanged. They remain a proposal; no staging directory or files were created
and no archive, layer, validator, synthetic case, driver, helper, native tool,
Docker, Cargo or prospective process supervisor was invoked here.

Candidate helper remains99566 bytes,
SHA-256 `c5ede6313bf967ab1ab42429e74fb4ca9fcc03c82bd0da2e4433e2b129412837`;
workflow remains4573 bytes,
SHA-256 `9af6d1d40543bd9604d7105730b97cec9c665d0f197548e50ba9a87eeb6007d4`.
The source59f5c6b recipe, protected production and architecture handling are
unchanged. Root's independent source/body reviews are separately attributed;
this appendix does not claim to have performed that independent review or any
future runtime. Documentation and whitespace checks are run before the
report-only commit; exact prefix, text/AST reversal, pins and changed-file scope
are checked again from the final report.

The one future build-free invocation remains HELD pending root's final immutable
source/design review and fresh capacity. Actual cohort37061329815 remains FAILED
at `image_tar_unreferenced_member`; UID and fixture gates remain unreached.
No new source/runtime success, ARM evidence, image execution, whole A09 closure,
O07 remedy or task disposition is inferred. All earlier failure receipts and
closed I04/other rows remain untouched. No runtime/preparation slot was acquired
or released. Root alone owns integration, publication and task status.

Actual final documentation check: `python3 scripts/check-docs.py` exited0 with
"Markdown links and build-directory layout checked". `git diff --check` exited0.
A readback of the persisted report passed exact112120-byte prefix preservation,
both listing hashes, byte/AST reversal, exact displayed diff, final-clock
control-flow and unchanged helper/workflow pins; the only changed tracked path
was this report and the index was empty. No archive/case/supervisor runtime was
performed. Staged whitespace and report-only scope are checked before commit,
then the committed prefix/source pins and clean worktree are verified.


## Root final-clock source review (2026-10-03)

Root read complete511b32a prose/diff and independently reversed the entire16833B corrected supervisor4b6ca88c to the previously fully read16622B af65db87. Only the final post-journal clock/closed elapsed field changes; no regular-file operation follows that check. The unchanged25305B f6a57790 payload and42-case registry remain source-only. Saved observations precede final acceptance; a late final stdout refusal overrides an earlier saved pass decision. User-space checks do not impose a hard kernel/output exit deadline. Independent archive review and exact fresh capacity/staging still precede any separate release.

An initial root reversal used an absent extraction filename block1 rather than actual block-1 and failed before writes; the corrected read completed the full inverse above. The no-op commit attempt made no history change. No candidate or runtime execution occurred.


## Root independent archive-design disposition (2026-10-03)

Root read the complete [independent design review](local-wave30-a09-archive-validation-independent-review.md) at4c4b061. It independently confirms the final post-journal elapsed gap and no additional concrete source blocker in the complete payload/supervisor and selected validator bodies. Root separately reviewed511b32a exact full inverse and final-clock fix. The ordered42-case and retained-positive oracles remain UNEXECUTED; no archive reopening, derived member or resource/cleanup result is claimed by either source review. Runtime stays held behind the one released CLI filter until its actual child/owned-group release. A later exact build-free archive release is a separate root scheduling action, with fresh10GiB capacity and no Docker/native/build or second child.

## Actual single retained-archive build-free invocation

Reservation `wave30_A09_retained_archive_single_build_free_511`, project
`891e7443-8dac-4c1b-897f-9e53cb59c7ee`, original A09
`506e3979-a590-4af3-8fa8-ee90d3a517f2`, supporting worktree
`e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`. This actual-evidence appendix
preserves the entire137438-byte511b32a report prefix, SHA-256
`19419051e6feab0d57a5d0e21a894e3f04b1914dc726b533855e14f972c281dc`.
All earlier design listings, reviews, failures and scope limitations remain
historical text. Their former "not executed" statements describe their own
source/design phase; this appendix records the separately released execution.
Root's independent4c4b061 body/source review is attributed to root and that
reviewer, not to this invocation.

### Preflight, exact staging and sole command

The worktree was clean at511b32a2c254fa3183a4c190f246ba3139bd94fd before setup.
Installed Python was3.14.6. Own target parent ownership and resolved path,
required WNOWAIT/process-group support, absent fresh evidence directory,
complete committed report bytes, source helper99566/SHA c5ede631…2837 and
workflow4573/SHA9af6d1d4…007d4 were verified. No alignment or source edit occurred.
The actual read-only preflight rehashed the retained outer ZIP49206215 bytes,
SHA-256 `d150a5950cbe6ff2dc4399614eca2bf4dfa274ac0db755739ef328a29f9c76b6`,
and streamed exactly the selected regular, unencrypted member
`local-essentials-x86_64.docker.tar.gz`:48934031 bytes, SHA-256
`e9e1291895a1cf6c894147acaa0b4efca4a885d4f07dc88f5aee2152c9604905`.
The public root receipt16872/SHA ab7c5ce7…ab7d and public small-metadata
17651/SHA430471f2…e5b4 matched. File identity checks included own UID,
regular/nonsymlink, one link, exact bounded lengths and unchanged stat metadata
across the reads. These preflight reads did not invoke the candidate or cases.

Preflight free space was25088712704 bytes before staging and25088651264 after
staging, both above the released10GiB start requirement. The fresh directory
`target/a09-v28-legacy-build-free-59f5c6b-v1` was absent, then created0700 under
this worktree only. Exact marker, payload and supervisor were created with
O_EXCL/O_NOFOLLOW,0600, fsynced and read back; the directory was fsynced.
Payload25305/SHA f6a57790…b9e3 and corrected supervisor16833/SHA4b6ca88c…b1bd
were materialized from the immutable reviewed report, byte exact. No original
artifact, source, prior evidence or other worker's files were modified.

Exactly one reviewed supervisor invocation was executed, with no substitution,
extra feature, case, source oracle correction or retry:

```sh
python3 -I -S -B "$PWD/target/a09-v28-legacy-build-free-59f5c6b-v1/supervisor.py"
```

The supervisor launched exactly one isolated Python child, using the reviewed
payload, cleared child environment, owned session/group,120-second child limit,
125-second outer controls,8GiB floor and2-second disk sampling interval. The
candidate's complete pinned archive validator and new legacy binding helper
were executed through the reviewed AST-selected class/support definitions;
the controller module main, Cohort constructor, other controller methods,
Docker, native/product code and opaque layer filesystem contents were not
executed. Layer payloads were streamed as opaque bytes for content hashes;
no inner layer filesystem was unpacked, loaded or mounted.

### Actual joined result and final deadline admission

The command completed with numeric parent exit0; the tool's measured command
wall time was0.607238 seconds. Its complete finite stdout was:

```json
{"count": 42, "exit": 0, "final_elapsed_seconds": 0.5080366251058877, "first_failure": null, "first_outcome": null, "group_empty": true, "reaped": true, "result": "passed", "stop_reason": null}
```

The supervisor child PID/PGID was77484. Its child exit0, null signal,
reaped=true, owned_group_empty=true and empty stdout/stderr captures were
fsynced in the joined row before completion grading. The joined elapsed
observation was0.508 seconds, and the post-save final clock measured
0.5080366251058877 seconds, before OUTER_END. The pre-final decision row
recorded passed/count42; the separate final stdout also recorded passed.
There was no first failure, first outcome or stop reason. This execution did
not exercise a stalled receipt, timeout, cancellation, disk-floor termination
or late-clock refusal; those controls remain source-reviewed, not empirically
proved by this fast successful path.

Supervisor preflight free bytes were25089646592; the recorded sampled minimum
was25089495040. Immediately after exit/parent receipt save, fresh free bytes
were24975904768. A later evidence inventory observed25038843904. These are
observed samples, not a continuous minimum or a measured peak-memory claim.
No guard was reduced and no resource cleanup/prune was performed to achieve
capacity. Child captures were empty; the supervisor's closed stdout
above is the actual final envelope, not native or protocol output.

The exact tool numeric return and complete stdout, plus staging observations,
were saved in exclusive0600 `actual.invocation.json` and fsynced before the
independent post-run result comparison. Its recorded UTC timestamp is
`2026-10-02T23:25:10.966724+00:00`. This external controller receipt is separate from
supervisor finalization: the unchanged joined/decision journals remain
pre-final observations, and the supervisor itself wrote no files after its
final clock. Post-run evidence/report writing does not change that ordering.
After joined exit, an independent `killpg(77484, 0)` returned
ProcessLookupError, agreeing with the supervisor's empty-group proof. No
post-reap signal was sent. The child was reaped and the tool reported the
supervisor process complete; no owned runtime child/group remained.

The temporary validation slot was explicitly RELEASED in the immediate actual
handoff before this report appendix. Cargo and desktop were never acquired or
released. The only invocation has finished; no additional runtime is authorized.

### All42 actual observations

All42 recorded case names are in the exact reviewed CASE_ORDER, and the
original payload accepted every recorded outcome against its literal oracle.
Three positive observations passed; the remaining39 observed their required
specific refusals. Each observation was journaled/fsynced before its original
comparison. The retained archive itself is the decisive actual positive; the
small synthetic baseline does not replace it.

| Case | Observed outcome |
| --- | --- |
| `retained_archive` | `pass` |
| `retained_derived_six` | `pass` |
| `tiny_baseline` | `pass` |
| `extra_correctly_hashed` | `image_tar_unreferenced_member` |
| `wrong_metadata_digest_path` | `image_legacy_metadata_binding` |
| `altered_under_old_path` | `image_legacy_metadata_binding` |
| `rehash_id` | `image_legacy_metadata_binding` |
| `rehash_parent` | `image_legacy_metadata_binding` |
| `rehash_created` | `image_legacy_metadata_binding` |
| `rehash_os` | `image_legacy_metadata_binding` |
| `rehash_config` | `image_legacy_metadata_binding` |
| `rehash_unknown` | `image_legacy_metadata_binding` |
| `duplicate_json_keys` | `image_legacy_metadata_binding` |
| `malformed_metadata` | `image_legacy_metadata_binding` |
| `scalar_metadata` | `image_legacy_metadata_binding` |
| `missing_metadata` | `image_legacy_metadata_binding` |
| `duplicate_member` | `image_tar_member` |
| `nonregular_member` | `image_tar_member` |
| `path_member` | `image_tar_member` |
| `manifest_layer_order` | `oci_image_manifest_binding` |
| `streamed_layer_mismatch` | `image_legacy_layers` |
| `rootfs_layer_order` | `image_legacy_layers` |
| `rootfs_duplicate` | `image_legacy_layers` |
| `rootfs_schema` | `image_legacy_rootfs` |
| `bool_as_int` | `image_legacy_config` |
| `top_unknown_schema` | `image_legacy_profile` |
| `invalid_calendar` | `image_legacy_created` |
| `noncanonical_time` | `image_legacy_created` |
| `layer_count_129` | `image_legacy_rootfs` |
| `compressed_stat_cap` | `image_archive_size` |
| `expanded_header_cap` | `image_tar_expanded_cap` |
| `small_payload_cap` | `docker_archive_manifest` |
| `truncated_payload` | `tar_read_error` |
| `wrong_tag` | `image_tar_exact_tag` |
| `wrong_config_image` | `image_tar_config_layers` |
| `wrong_oci_config` | `oci_image_manifest_binding` |
| `wrong_oci_index` | `saved_oci_index` |
| `wrong_oci_reference` | `oci_manifest_digest` |
| `missing_oci_payload` | `oci_manifest_payload` |
| `unsupported_daemon_with_legacy` | `image_tar_unreferenced_member` |
| `original_blob_digest_loop` | `image_blob_digest` |
| `original_resource_budget` | `resource_monitor_refused` |


The real derived-members witness reported six entries, set SHA-256
`ddf09fe5d1c8289056fc4e2282e09269a752f3a44fbef76f551db8d09a774603`, compared against the unchanged six externally
pinned public metadata paths. Twelve public metadata identities were recorded;
there were five input-pin rows,38 synthetic-input rows,42 case rows and one
witness row, with no stop row. The final child envelope was:

```json
{"count":42,"first_failure":null,"requested_exit":0,"schema":"riauth.a09.build-free-child/v1"}
```

The exact positive covers the retained x86_64 Docker28.0.4 export profile,
selected image `sha256:6c1248f6e88133330785d3df42b8e7129027b0e3010ab29fc30e2831408a7c5a`,
Essentials/sourceb619, the original selected owner/tag/config and ordered layer
chain plus six derived legacy metadata objects. It is not an ARM archive run,
a generic daemon-version allowance or permission to accept extra JSON blobs.
The unsupported-daemon and extra-correctly-hashed cases observed their
original `image_tar_unreferenced_member` refusal.

### Private retained evidence identities and limits

All following literal files remain under this own private0700 directory,
regular/nonsymlink, same UID, one link,0600. No raw private protocol, secret,
layer filesystem content or arbitrary error string is published here. File
hashes below identify evidence only; no file was deleted or overwritten after
its captured final state (the reviewed synthetic case file was reused by the
original payload during its one invocation).

| Retained file within the private evidence directory | Bytes | SHA-256 |
| --- | ---: | --- |
| `.marker` | 59 | `055b0e3e497ae240f6fc04c5a1dce0379a659637d45733efb51c7358812b4b17` |
| `payload.py` | 25305 | `f6a57790e77e85153c0a2fa84f03c257fa80931621004a53d67946aa1c0db9e3` |
| `supervisor.py` | 16833 | `4b6ca88c509feeb0b0331c6e1ac970ac1e07d81fde2650e3685c8a86af8cb1bd` |
| `controller.txt` | 99566 | `c5ede6313bf967ab1ab42429e74fb4ca9fcc03c82bd0da2e4433e2b129412837` |
| `receipt.json` | 16872 | `ab7c5ce7b87e672db255edc4393ccb27348cb777b14f5f7880f87a68a986ab7d` |
| `small.json` | 17651 | `430471f20c63eb533003415c19c3d19c8f83b542af7acb75621cfe150040e5b4` |
| `inner.docker.tar.gz` | 48934031 | `e9e1291895a1cf6c894147acaa0b4efca4a885d4f07dc88f5aee2152c9604905` |
| `case.tar.gz` | 1352 | `eb91c69710b272d40fa2678656cde689e7ff4dbe4f289a57ffdbec0b5d789b0f` |
| `actual.child.jsonl` | 9884 | `2e821b62437d666d992cf0180b13d7b24baed684732cbf128010b637809fb4fc` |
| `actual.child.json` | 94 | `2dbda9c3b1fde1f3c4c0be18634a27b6ec93153b7212a564bd55a628d4b3d4a6` |
| `actual.outer.jsonl` | 902 | `c49a2d49376f331a7bebeade2491ce2ce57ef387395647a897421d34c529c35b` |
| `actual.invocation.json` | 1492 | `7384f2e940e4d9976391e47f89cb4ba5b8b704db88cc43f03e0a1a13a6d0467f` |
| `stdout.log` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `stderr.log` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |


Actual cohort37061329815 remains FAILED at its historical
`image_tar_unreferenced_member` boundary. This build-free pass supplies a
bounded regression result for the corrected strict archive validator; it does
not rerun or relabel the container cohort. Container UID10001, private copy-up,
mount permissions, nonroot maintenance, image-save/rm/load runtime lifecycle,
shared full source agreement and E-P-E refusal/nonrenewal gates remain
unreached in that failed cohort. No full A09, O07, Linux deployment/HA, release,
ARM or product-execution closure is claimed. Both earlier Desktop UID/GID0:0
failures and I04/other completed rows remain unchanged. Root alone selects
later runtime, integration and original task disposition.

Only this report is appended. Exact137438-byte prefix/source/workflow pins,
report-only scope, documentation and working/staged whitespace checks are
verified before commit; committed prefix and clean worktree are checked after
commit. No production/helper/workflow edit, Cargo/build/native/Docker/GUI/HTTP/
network/download/query, extra case/alternative profile/retry, deletion/prune,
main/push/task-status change, new worker/task/worktree/managed shell or other
worker contact occurred. Private runtime evidence is retained ignored under
own target; it is not imported as product source or an official artifact.

Actual final checks: `python3 scripts/check-docs.py` exited0, reporting
"Markdown links and build-directory layout checked"; `git diff --check` exited0.
The persisted137438-byte prefix was exact, helper/workflow identities unchanged,
parent receipt exit0/count42/passed intact, and only this report differed.
`git check-ignore` confirmed the private invocation receipt stays ignored.
Staged report-only scope/whitespace are checked before this separate evidence
commit, then committed prefix/source pins and clean worktree are verified.
