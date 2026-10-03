# Wave30 A09 closed ARM container completion source plan

Source design only, 2026-10-03. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`,
original A09 `506e3979-a590-4af3-8fa8-ee90d3a517f2`, reservation
`wave30_A09_arm_container_completion_source_plan`. Independent support in existing
WT `ed9ac424-59f4-4520-905b-919aea3521eb`, existing shell0164, branch
`roadmap/local-revisions-coordination-wave27`, parent
`d0f4f39d7e7bb43e83c858f13a4f321031c07d25`. Primaryf2 remains unchanged.
This new report is the sole write. No workflow/helper/product change, merge,
alignment, runtime, dispatch, download, artifact extraction, worker contact or
board mutation occurred.

## Recommendation and exact prospective ownership

Recommend root review of **one existing-workflow slice comprising three line
replacements** in `.github/workflows/check-local-container-cohort.yml`: advertise
the existing closed ARM choice, preserve default x64, and pass the existing
`--prebuilt-x86` flag only for x64. The existing runner expression already selects
`ubuntu-24.04-arm` for ARM. No helper change is proposed.

The full helper supports the existing ordinary native ARM build path. Its ARM
product, source tree, review receipt, native transport, reader source and build
inputs agree by immutable data checks. I found no static missing-input or
contradictory-pin blocker. Availability of the future native daemon/Buildx,
pinned images and retained artifact remains unobserved here and must be checked
by the existing fail-closed route at a separately released invocation. A runner
label alone supplies neither those tools nor capacity.

The staged Linux bind-probe correction is independently owned:
`77a9785fbde5c27c06c690d6c15964761d72552c` adds only two lines before the existing
loopback bind. This report does not apply, change or validate that correction.
The current x64 failure and future x64 retry remain separate from this ARM plan.
Source recommendation is not A09 completion or permission to run.

## Original row, guidance and state

I read the entire matching original task row from the exact project's
`planning/current-tasks.json`, and the original A09 record in
`planning/riauth-backlog.json`. Their immutable read identities in this session:

| Local project planning snapshot | Bytes | SHA-256 |
| --- | ---: | --- |
| current-tasks.json | 243722 | `b17fca3cbecf381938c7cf7371c1fd46c76dfab7c8816fde9da6accc09eb1dec` |
| riauth-backlog.json | 72789 | `3b41c17bc920449ab7392917825252316993e56bee1a938392fe8a70aabfa070` |

Both reside under
`/Users/dominik/.local/share/riwork/orchestrators/projects/891e7443-8dac-4c1b-897f-9e53cb59c7ee/planning/`.
The task row explicitly matches the project UUID and original A09 task UUID.
Its outcome is “Produce and test server, client, container, and maintenance
artifacts for explicitly supported platforms, including Linux x86-64 and ARM64.”
Its workstream gate is “Switching distributions does not create different users,
different authorization rules, or silently ignored configuration.”
Prerequisites are A04/A05/A08/Q08; completion requires relevant actual evidence,
not documentation alone.

These are local dated snapshots, **not a new live-board/API read**. Tool metadata
exposes no RiWork orchestration read/send tool in this session; no Orca or other
provider was substituted. The dated task row says in_progress and names primary
WTf2; the original backlog says todo/unassigned. Neither is reconciled or changed
here; the latest user's support reservation supplies this report's authority.

I reread CONTRIBUTING.md. Repository AGENTS.md search found no applicable file.
Own Git status was clean and parent unchanged before writing; no model discovery,
new task/worktree/shell, desktop or other worker interaction was needed.
The already completed D01 d0f4 plan and prior independent reports remain intact.

## Complete body reads and immutable identities

I read all 2,337 staged helper lines in contiguous chunks, including imports,
platform literals, every method, native/prebuilt branches, all fixture/refusal
paths, diagnostics, resource monitoring and cleanup. I read all 109 current
workflow lines, all 326 lines of the historical native ARM workflow, and the
complete ARM native, parent BuildKit, ARM BuildKit, x64 native and new ARM
PostgreSQL root receipts. This is body review, not a filename-only inventory.

The fixed product reads include complete Dockerfile (28 lines), .dockerignore
(57), server manifest (91), client manifest (37), rust-toolchain (4), native
archive checker (316), store probe (118), and LICENSE (21). Both complete lockfiles
were TOML data-parsed and hashed, with manifests; the 14,514-line notices file was
hashed as an input, not independently audited line by line. I read the support
declaration and relevant original artifact-plan context at the fixed publication;
I do not claim an audit of every product source file or all that plan's appendices.

| Object | Pin / SHA-256 witness |
| --- | --- |
| Fixed published tree | `143d99dbe43fefd6aeb3d274a0d974fd025292e0` |
| Staged helper source | `77a9785fbde5c27c06c690d6c15964761d72552c` |
| Current workflow, identical at both pins | 4574 bytes / 109 lines / `3cf3d01fa751e418673b79dabed4e3edc7f44dd0a395d3fb2b58e98585743851` |
| Staged helper | 138392 bytes / 2337 lines / `6726dfd945445873214e934aa1f20815dc99ab24660b2205a45fd43c1486f221` |
| Published helper before separately owned two-line fix | 138268 bytes / 2335 lines / `9f95975a70caf47c06b224226134649f552169714f9ebf3b54b4b55da98f5908` |
| ARM product | `9a819317efb3a13fa27cd86f884be2be00898fc0` |
| ARM product tree | `1528b61ba463d9262d6252d54174748a176f313b` |
| ARM review | `b71b7b0041a549793233e8c7a81bbb61797e20f3` |
| ARM native workflow | `036a392656b4b5070cc86a11d5ca3258b7b868d2`; 18186 bytes / 326 lines / `0a98865e5ee25d2f94b745b32602949553bf4f4e3b83d8bd302fade98eafd948` |

[Staged helper source](https://github.com/Rhein-Industries/riAuth/blob/77a9785fbde5c27c06c690d6c15964761d72552c/scripts/check-local-container-cohort.py)
and [fixed workflow](https://github.com/Rhein-Industries/riAuth/blob/143d99dbe43fefd6aeb3d274a0d974fd025292e0/.github/workflows/check-local-container-cohort.yml)
were read as local Git objects; these links are immutable references, not web
queries or present remote-state verification.

The staged-helper inverse removed the exact 124-byte, two-line Linux
`SO_REUSEADDR` insertion, restoring the **entire** published helper bytes.
No other helper difference exists between those two pins. The proposed workflow
has no helper delta. A static map of all 61 Cohort method ASTs (locations plus
SHA-256; sorted compact JSON) hashes to
`3098672b35a5adfb89d07831838c5244124eb39b5af1500975094e59edb58145`.
That map is a source identity check, not execution of those methods.

## Exact workflow candidate and inverse

Current workflow: 4574 bytes / 109 lines, SHA-256
`3cf3d01fa751e418673b79dabed4e3edc7f44dd0a395d3fb2b58e98585743851`.
Prospective candidate: **4640 bytes / 109 lines**, SHA-256
`63887b4f57c959ce5256865c2befc7cc568b95e9eb7d10eff0909cf3dc6747b8`.

The complete zero-context diff is:

```diff
--- a/.github/workflows/check-local-container-cohort.yml
+++ b/.github/workflows/check-local-container-cohort.yml
@@ -7 +7 @@
-        description: Verified dated x86 LOCAL container fixture replay
+        description: LOCAL container cohort with x86 replay or native ARM build
@@ -11 +11 @@
-        options: [x86_64]
+        options: [x86_64, arm64]
@@ -95 +95 @@
-          python3 controller/scripts/check-local-container-cohort.py --prebuilt-x86 \
+          python3 controller/scripts/check-local-container-cohort.py ${{ inputs.architecture == 'x86_64' && '--prebuilt-x86' || '' }} \
```

Each old and new literal occurs once. Replacing these three candidate literals
with their respective old literals, in reverse order, restores every original
workflow byte. Psych parsed both documents; after restoring just description,
options and the helper step's run string in parsed data, all remaining YAML data
was equal. Default `x86_64`, existing runner expression, source/review mappings,
step ordering, permissions, concurrency, timeouts, action pins and upload
contract remain equal.

The expression emits only the fixed flag `--prebuilt-x86` or an empty string.
It never emits the supplied architecture as a shell argument. The preflight
still rejects any architecture outside the two literal mappings before any
checkout/helper operation. Unknown input cannot reach the ARM build path.
For x64 the rendered command is the original command; for ARM the flag is absent.
Argparse then keeps `prebuilt_x86=False`; the constructor continues to refuse
ARM plus an explicit prebuilt flag. No new action authority or cross-architecture
fallback is introduced.

Complete candidate, source/data only:

```yaml
name: Check LOCAL native Linux container cohort

on:
  workflow_dispatch:
    inputs:
      architecture:
        description: LOCAL container cohort with x86 replay or native ARM build
        required: true
        type: choice
        default: x86_64
        options: [x86_64, arm64]

permissions:
  contents: read
  actions: read

# Deliberately retains the native artifact job's legacy serialization key.
# Root also serializes this job against the separately owned shared-store job.
concurrency:
  group: riauth-local-artifacts-arm64
  cancel-in-progress: false

jobs:
  native-container:
    runs-on: ${{ inputs.architecture == 'arm64' && 'ubuntu-24.04-arm' || 'ubuntu-24.04' }}
    timeout-minutes: 150
    defaults:
      run:
        shell: bash
    env:
      PYTHONOPTIMIZE: '0'
      ARCHITECTURE: ${{ inputs.architecture }}
    steps:
      - name: Establish private evidence and initial capacity
        timeout-minutes: 2
        run: |
          set -euo pipefail
          umask 077
          python3 - <<'PY'
          import json, os, pathlib, platform, shutil
          architecture = os.environ.get('ARCHITECTURE')
          platforms = {
              'x86_64': {'machine': 'x86_64', 'runner_arch': 'X64',
                         'source': 'b619fe25269ccc150e473bbcde47cdb3623ef810',
                         'review': '66c814a339665e6b3f8e6c22bc59d4f6f0aa224c'},
              'arm64': {'machine': 'aarch64', 'runner_arch': 'ARM64',
                        'source': '9a819317efb3a13fa27cd86f884be2be00898fc0',
                        'review': 'b71b7b0041a549793233e8c7a81bbb61797e20f3'}}
          if architecture not in platforms:
              raise SystemExit('unsupported_native_architecture')
          native = platforms[architecture]
          if (platform.system(), platform.machine(), os.environ.get('RUNNER_ARCH'),
              os.environ.get('RUNNER_ENVIRONMENT')) != ('Linux', native['machine'], native['runner_arch'], 'github-hosted'):
              raise SystemExit('unsupported_native_host')
          paths = ('/', os.environ['GITHUB_WORKSPACE'], os.environ['RUNNER_TEMP'], '/var/lib/docker')
          free = {p: shutil.disk_usage(p).free for p in paths}
          if min(free.values()) < 30 * 1024**3:
              raise SystemExit('initial_capacity_below_30GiB')
          root = pathlib.Path(os.environ['RUNNER_TEMP']) / ('riauth-container-' + os.environ['GITHUB_RUN_ID'] + '-' + os.environ['GITHUB_RUN_ATTEMPT'])
          root.mkdir(mode=0o700)
          (root / 'evidence').mkdir(mode=0o700)
          (root / 'evidence' / 'launch.json').write_text(json.dumps({'initial_free_bytes': free}) + '\n')
          with open(os.environ['GITHUB_ENV'], 'a') as out:
              out.write('A09_CONTAINER_ROOT=' + str(root) + '\n')
              out.write('A09_PRODUCT_SOURCE=' + native['source'] + '\n')
              out.write('A09_ROOT_REVIEW=' + native['review'] + '\n')
          PY
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1
        timeout-minutes: 5
        with:
          path: controller
          persist-credentials: false
          fetch-depth: 1
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1
        timeout-minutes: 5
        with:
          ref: ${{ env.A09_PRODUCT_SOURCE }}
          path: product
          persist-credentials: false
          fetch-depth: 1
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1
        timeout-minutes: 5
        with:
          ref: ${{ env.A09_ROOT_REVIEW }}
          path: review
          persist-credentials: false
          fetch-depth: 1
      - name: One bounded LOCAL container cohort
        timeout-minutes: 125
        env:
          A09_GH_TOKEN: ${{ github.token }}
        run: |
          set -euo pipefail
          umask 077
          python3 controller/scripts/check-local-container-cohort.py ${{ inputs.architecture == 'x86_64' && '--prebuilt-x86' || '' }} \
            --controller "$GITHUB_WORKSPACE/controller" \
            --product "$GITHUB_WORKSPACE/product" \
            --review "$GITHUB_WORKSPACE/review" \
            --root "$A09_CONTAINER_ROOT"
      - name: Upload LOCAL evidence, including refusals and cleanup limits
        timeout-minutes: 15
        if: ${{ always() }}
        uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02
        with:
          name: riauth-local-container-${{ inputs.architecture }}-${{ github.run_id }}-${{ github.run_attempt }}
          path: ${{ env.A09_CONTAINER_ROOT }}/evidence/
          if-no-files-found: error
          compression-level: 0
          retention-days: 14
```

## ARM transport and source equivalence

The three receipt blobs at the **existing fixed ARM review b71** match the
helper's literal allowlist:

| Review receipt | Bytes / lines | SHA-256 |
| --- | --- | --- |
| wave30-a09-native-arm64-37016520583.json | 10559 / 321 | `2cbf8ee46dbdd8b2ab43ad933913dd0a16c20b3183497c2d3b9cf3d1287da227` |
| wave30-container-buildkit-source-pin.json | 8628 / 220 | `4d263888b2bdc5a2eb922b29c85535e8321dc2cba7c246b5b6b355cef57e57b0` |
| wave30-container-buildkit-arm64-source-pin.json | 3653 / 82 | `e234b809df9212785c818ace0a287eb356de0b9d5ca8560aab0818e5111d0de2` |

The ARM native receipt binds run **37016520583**, job **110868629053**,
attempt 1, source9a, workflow036a and artifact **11232871527** named
`riauth-local-arm64-37016520583-1`. Expected complete ZIP is **49177062 bytes**,
SHA-256 **fc2a83dd286b70e455802af7713b60724d97b9e598240f6d9037c279601e4d30**.
The original native receipt records an API/upload digest and root did not
rehash that outer ZIP there. The later actual ARM PG receipt independently
records the same 49177062-byte input ZIP rehash with that complete digest.
This session rehashed neither ZIP nor binary; it compared immutable receipts
and Git input blobs only.

The helper's existing ARM metadata adapter preserves original products, input
hashes, step logs and raw schema attribution while supplying the closed common
metadata shape. Its source check requires the exact native success, all five
products and all eleven zero-exit steps, source/tree, target and not_run shared
attribution. Native transport still requires exact completed historical run,
repository/workflow/source metadata, nonexpired artifact, exact name/ID/size/
digest, restricted HTTPS redirect origin, a bounded complete ZIP digest **before**
extraction, an exact allowlist of 18 files plus the archives directory, and
regular/nonduplicate/capped members. Five inner archives retain exactly binary,
LICENSE and notices; every binary hash/size and ELF64 little-endian **machine
183** is checked before native use. Both native capability envelopes must equal
their recorded envelopes. Failure refuses; no alternative artifact is adopted.

Seven source9a native inputs were independently matched to the ARM receipt:

| Input | Bytes | SHA-256 |
| --- | ---: | --- |
| `Cargo.lock` | 109243 | `b5c9d11c001244b8017303ce8c20516e02946483845eee40720b0910758d4426` |
| `Cargo.toml` | 4020 | `58e5ef824ed96290179c9f76fea208dc37173caeee21b6ce8d37f8a6dd1abcb8` |
| `LICENSE` | 1076 | `ef79ab7079893da02af81ebb8a57bec27dc7a601d505b228b1d5356a3aa5b1a9` |
| `THIRD_PARTY_NOTICES.md` | 752651 | `142c0e5150de9436513d9f6f215c5422b8a3af84d4eb7d0708f155e3e18fce05` |
| `crates/riauthctl/Cargo.lock` | 70912 | `2998555ddb2d00e8130a970fcacfed19dfee7a0b2e3305011ebd40746f67b4db` |
| `crates/riauthctl/Cargo.toml` | 1139 | `af385d4d989c53987396edfdfa684cd3902a603d1cf65dd31ac72da7f08de9ea` |
| `scripts/check-edition-artifacts.py` | 13118 | `2f477665ed584bc148fb230c0658f6d0cc66aceaa5af631c4dc467872fa9ecce` |

The server/client manifests use the frozen version and distinct assemblies;
native builds are release/locked/no-default-features, both server/maintenance
editions and the separate base client. The original native workflow uses pinned
checkout `3d3c42e5aac5ba805825da76410c181273ba90b1`, Rust action
`02cb101ec7c40f2c49e1d9714d64511d8e1b74de` with **1.98.1**, and upload
`ea165f8d65b6e75b540449e92b4886f43607fa02`. Those historical commands and results
are not rerun by this design. The container workflow retains its own same pinned
checkout/upload actions; it obtains compiler/native dependencies from the
existing pinned Docker build recipe, not a new host-install step.

## Existing ordinary ARM build and reader paths

[Product Dockerfile](https://github.com/Rhein-Industries/riAuth/blob/9a819317efb3a13fa27cd86f884be2be00898fc0/Dockerfile)
pins Rust 1.98.1/Trixie to digest
`a8a5f0a1e5fe7dfe1d352591e4a1c7dd2c08fd70475cae872cf3458ba0df0546`
and Debian Trixie-slim to
`a99cfc517144bc59b1978475ec53b46ecabec7e43635402ee5b77cc54cd1b20a`.
The private serial recipe inserts only the existing resource environment prefix
before the exact edition build command. Whole-byte reversal passed: original
Dockerfile SHA `458ebb247170c6d4af2ff45b22e5a36e0a5380612140a43448d2ddcebc5d83cb`;
derived 1694-byte recipe SHA
`a55e90aa4da02870996c9dc755bbbbc80e0a1658674b0ab3f5fb141450a07440`.
The recipe preserves the edition case guard, source COPY set, --locked and
--no-default-features; private CARGO_HOME/target, jobs1, incremental0 and
dev/test/release debug0 remain explicit. Apt resolution inside pinned bases is
not thereby frozen or newly verified.

Existing `main` (2259 onward) always performs source check, native transport and
daemon setup first. With the flag absent it calls `build_images` and then
`build_reader`; it never calls the x64 importer. ARM uses
`--platform linux/arm64` on the actual native ARM daemon and the ordinary
Buildx builder. No emulation or architecture fallback is added.

Source witnesses at staged77a:

- `source_check` 505–570 retains exact clean controller/product/review checkouts,
  actual hosted machine, source tree, all input hashes and manual branch identity.
- `daemon_setup` 759–810 requires native Linux/non-Desktop/aarch64 daemon,
  measurable actual Docker storage, existing Buildx, exclusive builder names and
  the pinned tool image's actual config/architecture. The ARM tool is
  `docker.io/moby/buildkit@sha256:3ad6bb9bc8c78c0069d03247adb9a59b3b43d68e55353e876e558b888c6c1768`,
  from the reviewed v0.33.1 index
  `cec9f139f45e93c5c69c60f8b07cfad9f43f4ef6b6a6cd917527fea5ff2e3dea`.
  Its exact ARM config is
  `f27f9c00a3aca2c219642d1500610eade3ddcb6b873ea8847852ab156663d32f`.
  The receipt verifies manifest selection, not tool execution; future pull,
  actual config equality and builder bootstrap still must succeed.
- `build_images` 985–1037 uses the selected builder and two literal editions,
  owned labels/tags, exact source revision and image identities, bounded saved
  archives, no-prune removal, and same-hash reload. No new image is named as
  already existing by this report.
- `validate_image_tar` 1039–1111 and `expected_v28_legacy_members` 1113–1223
  retain ordered streamed layer/content hashes and the closed metadata profile,
  including the narrow Moby 28.0.4 legacy reconstruction. A future incompatible
  daemon/serialization is a refusal, not permission to enlarge the allowlist.
- `create_container` 892–911 and identity confirmation retain UID/GID
  10001, readonly root/config, dropped capabilities, no-new-privileges,
  pids256/memory2g/cpus2, bounded tmpfs and exact owner/time/image checks.
  Application containers never receive the daemon socket or privilege.
- `inspect_product` 1225–1255 preserves container server ELF183 and capability
  parity, license/notices identity, and matching native maintenance evidence.

The **existing** reader source is present at product9a with the exact required
size/hash; no current-S02 code or reader is imported. All six reader input
identities match the `build_reader` literals:

| Reader input | Bytes | SHA-256 |
| --- | ---: | --- |
| `Cargo.toml` | 4020 | `58e5ef824ed96290179c9f76fea208dc37173caeee21b6ce8d37f8a6dd1abcb8` |
| `Cargo.lock` | 109243 | `b5c9d11c001244b8017303ce8c20516e02946483845eee40720b0910758d4426` |
| `rust-toolchain.toml` | 86 | `887f9be066a15585a2c583578e84b0fcb541126d81546276bad3d2ff00d61167` |
| `Dockerfile` | 1512 | `458ebb247170c6d4af2ff45b22e5a36e0a5380612140a43448d2ddcebc5d83cb` |
| `.dockerignore` | 794 | `4fbd9472316442bdd8b72e268feb1140701e87ac28dd02295c687b6d379eb093` |
| `tests/edition_transition_store_probe.rs` | 5722 | `da486cb8cd7c9d6dfcd78da2704e27688b43a8dd40c7fa58e5b4574f678aed32` |

The complete probe body was read: the helper selects its `snapshot` action,
which uses read-only Store inspection and complete logical row/metadata output.
Other actions in that source are not selected by the helper. The unchanged
named context is exactly this one source file in a private directory; source
Dockerfile/.dockerignore remain unchanged.

The extracted reader stage is 1172 **runtime bytes** (SHA
`9e8fdf2c2fb44b15f8c3c999d1bc61dd36394f3961c2e7f186a07a21c73bdc65`);
Python literal continuation makes its runtime byte line count differ from its
readable source layout. Combined reader recipe is 2866 bytes / 35 byte-lines,
SHA `ba68e6ccb19b2421b250a75e01fa41844b772a3c892f99777f7d5e375ff8adc7`.
This is data reconstruction, not recipe execution.

The future reader compiles release/locked/no-default-features
`platform,test-support`, target `edition_transition_store_probe`, **--no-run**,
private jobs1/inc0/debug0. It exports exactly one regular owned single-link
binary, capped at128MiB and ELF183, then retains that binary/hash as public-source
evidence in mode0600. There is no pre-existing ARM reader artifact/result claimed.

## Unchanged security, resources, CLI and cleanup prerequisites

The helper's full fixture/control paths remain unchanged by the candidate,
including normal native CLI use with private sessions, explicit credentials and
configuration, authority/refusal checking, receipts, recovery and shared writers.
The existing fixture (2100–2145) requires ordinary identities across E→P→E,
all twelve UserView fields, group membership/grants, scoped auditor200 versus
user-create403/client4, logout401/client3, fixed issuer/policy/login-rate
refusals, agreed all-sixteen effective rates and format3, unchanged discovery/
JWKS, readonly plan plus exact token, wrong direct build refusal and isolated
Platform-agent/downgrade refusal. Logical snapshots preserve complete row and
metadata equality at the defined boundaries; no collection or assertion is
excluded by this proposal. Definitions of those eleven checks are not ARM
container runtime results.

The workflow preserves manual dispatch and read-only contents/actions permissions,
the legacy `riauth-local-artifacts-arm64` serialization group with
cancel-in-progress false, three isolated clean checkouts, no stored checkout
credentials, and always() failure/refusal evidence upload. Source controller SHA
must equal both GITHUB_SHA and GITHUB_WORKFLOW_SHA on the dispatched branch.
The token remains confined to authenticated artifact API reads; private product
credentials and sessions stay outside upload/source contexts.

Capacity remains measured, not assumed from `ubuntu-24.04-arm`: preflight requires
30GiB on root/workspace/temp/Docker; helper measures actual daemon storage and
requires30GiB before heavy operations, samples at intended2s intervals, and stops
owned work below10GiB. The 8GiB floor remains a safety margin, **not a hard quota
or guarantee against between-sample depletion**; this workflow does not introduce
a new threshold. Build commands remain bounded by1800s, log8MiB,
transport512MiB/180s with finite read timeouts, image archive2GiB,
fixture1200s/controller7200s, cleanup240s and existing process identity/reap
bounds. Job150min, helperstep125min, checkout5min, preflight2min and upload15min
are unchanged; these are existing bounds, not a claim every maximal phase fits
the job or all failure paths finish before its platform cancellation.

Full cleanup (2147–2256) retains creation/ID/label/time-bound ownership,
private-builder identity and mounts, exact application container/volume/image
confirmation, no global prune/cache deletion, process-group start-identity
checks, bounded owned signaling/reaping, monitor joining and private-input
removal while retaining public evidence. Unknown/replaced ownership blocks
cleanup and records failure; it is not forcibly erased. ARM images use the new
invocation owner, not historical x64 image-owner labels. Existing cleanup failure
still makes the final result failed; no earlier failure is promoted to pass.

## Historical evidence and remaining original scope

| Evidence | Actual bounded outcome and provenance | Limit retained |
| --- | --- | --- |
| ARM native run37016520583/job110868629053 | Source9a/workflow036a; five binaries/five local archives; eleven zero-exit native build/capability/archive-smoke steps; ELF183 and hashes recorded. | Native archive slice passed, shared_full_gate not_run, official_release false; no ARM container pass. Root original outer-ZIP rehash false. |
| x64 native run37046857550/job110970324302 | Sourceb619/workflow0d090169f20f61f7cb59b685dccb13203526539f; five binaries/five archives and eleven zero-exit steps; ELF62 recorded. Artifact11246575279, 52428080 bytes, SHA `7c1bcad0f78d90eb611ab76d67943a32a0d691cd5bbd9e5588512bdde635e2a0`. | Historical native slice only; receipt notes later CI stale notices and max sample gap6.585916s. No shared/container/TLS/passkey/tenant/official-release inference. |
| ARM shared PostgreSQL run37082572962/job111086077482 | Source9a, workflowd644e2c617501aca8efd43f6350c72b788c19d4d; PG16.15 E→P→E identity/authorization/config sample passed; all effective rates preserved, wrong client/authentication/general-rate refused;57 baseline/upgrade rows,68 downgrade rows. Root rehashed same ARM input ZIP. | full_shared_gate not_certified and release_gate_result false; plaintext bounded sample, not a container/tenant/TLS/device/passkey/whole-release pass. |
| ARM PG output and cleanup | Artifact11259328231,19973 bytes,SHA `2fb5052a1ee04d9aafbda4cfbad334ea34d7d3065f6468d44d9bc1a26e28a46b`; helper sample20.219462s, controller0; cleanup failures[],one reaped,zero remaining, fixture/private removed,no_live_owned_postgres. | pidfile verified false; terminal absence was separately verified in that dated root receipt. This session launched or probed no PG. |
| x64 prebuilt run37097066234 | User-provided actual FAILED start_app errno98. | No query/log attribution here; no port/TIME_WAIT/sender diagnosis, no retry or pass. Next x64 runtime separately held. |
| Earlier x64/PG failures | Preserved as failures at their original pins, including37087561409's unexpected OSError and earlier private shared-row mismatches. | Original inner cause/private values remain UNKNOWN; later passes do not rewrite them. |

The complete dated ARM PG root receipt at published143d is
`docs/roadmap/evidence/wave30-a09-native-pg-37082572962-root-review.json`,
27301 bytes / 794 lines, SHA
`750b4c95b2598c3cb53a8a1b477638c7ac43aab70762efb786797a0a3089189d`.
Its fourteen resource samples record min115571736576 bytes and
maxgap2.0005534919999945s. It is actual retrospective evidence at its recorded
source, not runtime evidence for this new workflow candidate.

The restored manual choice would reach the smallest missing **local ARM
container cohort** seam without rebuilding the already measured native archives.
It would build two new containers and a new reader only during a future
authorized ARM run. Neither new archive names nor reader hashes are assigned
here. There is no current-S02, official build or deployed HA credit.

For the required Linux x64/ARM matrix, both native server/maintenance/base-client
archive slices have historical passes; ARM's current-format shared PG slice also
has a bounded pass. Both closed container cohorts still lack a full successful
result in the evidence reviewed here. The proposed future ARM result would need
complete ordinary shared-authority/configuration/refusal checks and owned cleanup,
not just compilation or a label. The separately held x64 retry supplies its own
evidence only if actually released and passed.

[Supported-target contract at the fixed pin](https://github.com/Rhein-Industries/riAuth/blob/143d99dbe43fefd6aeb3d274a0d974fd025292e0/docs/roadmap/product-contracts.md#supported-platform-targets-and-release-gate)
declares Linux x64/ARM, not new macOS/Windows server/client/maintenance releases.
It separately retains named distro/libc support baselines, exact packaged HTTPS/
discovery/management/passkey/revocation/config/transition/backup evidence and
truthful release inventory/provenance/attestations. This one local loopback
container cohort would not supply every such artifact gate. Base-image pinning
does not freeze apt or certify dependency notices; historic input equality does
not repair the stale-notices issue recorded by the x64 receipt. No arbitrary
all-host/human-study/full-suite campaign is added. Root determines remaining
A09 disposition from original outcome plus actual accumulated evidence.

## Exact future ARM invocation, held

Root must first review this exact candidate, reserve the single workflow hunk,
integrate it with the independently owned helper source, and publish a reviewed
branch. It must record the resulting full workflow/controller commit and actual
helper SHA; this report does not invent that future publication identity.

Only **after the preceding x64 job exits**, its ownership/cleanup is assessed,
and root separately grants the exclusive artifact/runtime lane, the prospective
dispatch is:

```sh
gh workflow run check-local-container-cohort.yml --repo Rhein-Industries/riAuth --ref "$ROOT_ARM_WORKFLOW_BRANCH" -f architecture=arm64
```

`ROOT_ARM_WORKFLOW_BRANCH` denotes the separately reviewed published branch whose
head/root-recorded full SHA contains this exact workflow candidate and accepted
helper. It is not today's unmodified143d branch or an invented immutable ref.
This command is printed only; no gh/API/dispatch/query occurred. The workflow
accepts architecture alone; no unsupported source_sha/build/reader inputs are
invented.

The source-backed prerequisites and expected finite evidence are:

1. Root confirms no simultaneous x64/shared/native build/D01 desktop fixture
   ownership conflict and records the released single invocation/branch identity.
   Current x64 helper materialization stays separately owned; no contact here.
2. Runner must actually report Linux/aarch64/ARM64/github-hosted; Docker must be
   native Linux/aarch64, non-Desktop, with measurable storage and Buildx. Missing
   tool/storage/capacity is a fixed refusal, not an installation/emulation workaround.
3. Exact b71 receipt blobs/source9a/tree and the existing artifact11232871527 must
   still be readable/nonexpired with the same full size/digest. Future API and
   complete transport checks precede extraction/native execution; no fetch now.
4. Reviewed ARM BuildKit/base-image dependencies must be available through the
   existing bounded builder route with actual config/architecture checks. No new
   mutable tool tag, ARM archive, reader or dependency substitute is allowed.
5. Root retains the full evidence/recipes/tool metadata/native input identities,
   actual binaries/images/reader hashes, all fixture outcomes, finite resource
   samples and cleanup result from this one invocation. All eleven checks and
   cleanup must actually finish before a cohort pass is credited; a step failure,
   timed-out upload or unresolved ownership remains a gap. First actual failure
   is reported; no unreserved retry or assertion change is proposed.

## Static checks actually performed and preparation corrections

All candidate sources remained memory/data. No candidate function, helper import,
main, workflow command, probe, compiler, Docker or protocol was executed.

- Final standalone static checker exited0: whole-workflow literal inverse and
  published/staged workflow equality; Psych parsing/data normalization limited
  to exactly three changed paths; five Bash `--noprofile --norc -n` parses
  (old/new preflight, old helper run, both fixed candidate command renderings);
  four Python AST parses (two preflight bodies, full helper, native archive
  checker); native manifests/locks and reader TOML parsed.
- Three b71 receipt SHA checks, exact source tree, all seven native input SHA
  checks, all six reader input SHA checks and exact probe5722-byte check passed.
  Serial Dockerfile and reader stage were reconstructed as bytes from literal
  AST data; exact Dockerfile reversal and all displayed recipe hashes passed.
- Full staged→published helper two-line byte inverse and static 61-method AST
  identity map passed. YAML defaults/permissions/concurrency remained exact.
- Initial static attempt failed Psych at description line7 because my first
  proposed plain description contained an unquoted colon. I removed that colon
  in the memory-only candidate; no existing workflow was changed.
- Two later checker-preparation assertions failed: I initially assumed a
  `RUN cargo build` span instead of the helper's actual literal edition-command
  needle, then used insufficient indentation for the separately owned two-line
  helper inverse. The checker was corrected to extract the real AST needle and
  match the exact twelve/sixteen-space source span. Neither was a helper/product
  defect or runtime failure. The final fourth static invocation exited0.
- An initial Git read used a nonexistent
  `.github/workflows/check-local-native-linux-artifacts.yml` path; Git refused.
  Tree lookup identified the actual `check-local-artifacts.yml`, subsequently
  read in full. A checker-path search likewise reported absent
  `scripts/check-docs.sh`; the actual `scripts/check-docs.py` was read and used.
  AGENTS.md no-match is not a build/test failure.
- Pre-write `python3 scripts/check-docs.py` passed. Final report-only
  candidate/pin/link/whitespace/scope checks are recorded with the handoff commit.
  No native/CI result is claimed from these static checks.
- Post-write report candidate reversal, fixed-object/source pins, sole-path scope
  and unchanged tracked-source/parent checks passed; the docs checker passed again.
  The initial staged whitespace check flagged a final blank line at EOF; it was
  removed before the final whitespace check and commit.

Runtime remains HELD. This source plan does not acquire/release a runtime slot,
authorize any dispatch, close A09, reopen closed rows, or alter primary assignments.
Root owns source reservation, integration, publication, release and status.
