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
