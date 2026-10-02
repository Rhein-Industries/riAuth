# Wave 30 A09 legacy metadata independent source review

Review started 2026-10-02; report completed 2026-10-03 (Europe/Vaduz).
Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Reservation: `wave30_A09_legacy_metadata_independent_review`. Existing WT
`ed9ac424-59f4-4520-905b-919aea3521eb`, branch
`roadmap/local-revisions-coordination-wave27`; clean entry HEAD
`0178404a53e5fdcbfef1e7ec59df20b7de9fef18`.
Only this new report is written. No source import/alignment or runtime slot.

**Recommendation: accept 59f5c6b by source inspection for root's separately
reserved ONE build-free retained-archive plus synthetic validation. No concrete
new source blocker or corrective hunk was identified.** This is a narrow
version/profile-specific expected-byte derivation, not a permissive metadata
classifier. Its exact serialization/derived IDs and retained archive acceptance
remain UNEXECUTED here. The original container run remains FAILED; no UID,
fixture, ARM container or whole A09 gate is credited by this report.

## Immutable source and full-body reading

| Input | Actual review and identity |
| --- | --- |
| Candidate helper | `59f5c6b465a380d270acf68ba7a1719a41998673:scripts/check-local-container-cohort.py`: all 1,734 lines read, including all imports, original methods, the full validator and new serializer/derivation body. 99,566 bytes, SHA-256 `c5ede6313bf967ab1ab42429e74fb4ca9fcc03c82bd0da2e4433e2b129412837`. Truncated combined output was completed by bounded body rereads, including source_check501–566. |
| Actual Git parent | `effbeacbd1231aa848588cb0aa0870c6b5a0bf6f`: helper blob `b85c7bcc50cb149cc24604265b2c72efb206cdbe`, 92,964 bytes/SHA-256 `45353945c6d867039d29ada0f110cf2c2b85a907d179bb9049268046875ffc73`. Complete bytes/AST compared; its full validate_image_tar1030–1097 was separately read. |
| Fixed accepted equivalents | Parent helper bytes equal `74e106b819e7186d0ac41964cedbe8217fa7e881` and closed-architecture `d84d753912c8d920dd3f005d03024a9ad28841e4`. These are immutable object comparisons, not main/worktree alignment. |
| Primary diagnosis/design | `05187aac7ad3d7d21f74b72cf245ff3fd5830b80:docs/roadmap/local-wave30-a09-container-independent-review.md`: all 377 appended lines read in complete bounded chunks, including historical evidence, upstream attribution, all six design steps and the prospective validation limits. Whole report 42,881 bytes/SHA-256 `f6c6c39cef63bcd3742f6d84c4ce29ae90c84d7efda4e9a1a8092a4fd84f610e`; preceding report bytes are identity context, not a claim of rereading every preceding paragraph. |
| Current workflow | Entire `.github/workflows/check-local-container-cohort.yml` body at59f read; 4,573 bytes/SHA-256 `9af6d1d40543bd9604d7105730b97cec9c665d0f197548e50ba9a87eeb6007d4`. Whole bytes equal actual parent,74e and d84. No workflow execution. |
| Selected recipe | Entire Dockerfile at x86 product `b619fe25269ccc150e473bbcde47cdb3623ef810` read. Source_check's pinned Dockerfile identity, single serial-resource-prefix span/reversal and selected platform source/tree/receipt checks were read completely. Recipe generation/build was not performed. |

All 117 added diff lines were read. New helper111 lines are1104–1214; hook5
lines are1090–1094; one preceding blank completes117 insertions. Removing
exactly those spans reconstructs every parent byte. AST comparison found
50 existing Cohort methods identical, the existing validator changed only by
the hook, and exactly one added method. All original imports and entire
workflow are unchanged. These checks establish source preservation, not tests.

## Upstream serializer source and access limit

Official Moby version-source reference is v28.0.4 commit
`6430e49a55babd9b8f4d08e70ecb2b68900770fe`. Read-only official source browsing
was used under the task's explicit source-reading permission; no controller
HTTP client, native helper or network program was executed.

- Complete rendered save.go (599 lines) was read, including ordered export,
  saveImage/saveConfigAndLayer/saveConfig. It calculates the legacy ID before
  assigning intermediate OS, then stores serialized metadata by its content
  digest. [Pinned exporter](https://github.com/moby/moby/blob/6430e49a55babd9b8f4d08e70ecb2b68900770fe/image/tarexport/save.go).
- Complete rendered container.Config (68 lines) was read. Mandatory zero
  fields remain serialized; optional ports and ArgsEscaped occupy their
  declaration positions; Labels/Volumes are maps. [Pinned Config](https://github.com/moby/moby/blob/6430e49a55babd9b8f4d08e70ecb2b68900770fe/api/types/container/config.go).
- Complete rendered rootfs.go (50 lines) was read; RootFS delegates ordered
  DiffIDs to layer.CreateChainID. [Pinned RootFS](https://github.com/moby/moby/blob/6430e49a55babd9b8f4d08e70ecb2b68900770fe/image/rootfs.go).

Raw and GitHub-blob reads of image/v1/imagev1.go, image/image.go and
layer/layer.go returned cache-miss/internal errors: six unsuccessful source
fetches, not product/runtime failures. No executable HTTP fallback was used.
Their CreateID/V1/ChainID mechanisms were instead checked against the complete
immutable05187aa design's source-attributed descriptions and declared schema/
field order. I did not directly read those three full upstream bodies or
independently rehash upstream raw files in this slice. The six upstream file
SHA values in that report retain the author's provenance, not fresh verification.
Rendered line numbering collapses blank lines; no raw-byte identity is inferred.

The relevant source-attributed mechanisms are: CreateID sorts its outer
RawMessage map while keeping nested struct serialization order; each next
ChainID hashes the previous prefixed chain, one space and next prefixed DiffID;
saved V1 metadata uses unprefixed id/parent. These details have source witnesses
in05187aa lines302–349/404–459; exact agreement with retained bytes remains a
future validation oracle, not a derived-ID result from this review.

The actual daemon receipt states ServerVersion28.0.4 but contains no daemon
GitCommit/build attestation. Official-version format compatibility is therefore
the limit; neither the candidate nor this review proves that daemon binary's
origin. This is disclosed without inventing a new mandatory all-host gate.

## Profile and exact-byte binding review

Candidate hook1090–1094 activates only for recorded ServerVersion28.0.4 and
canonical lower-hex blob layer paths. Other layouts retain the existing
non-blob compatibility and final refusal path. The new method first checks
budget and a closed six-key selected image profile: architecture equals the
closed platform's OCI architecture, OS linux, history a list, edition one of
Essentials/Platform and owner exactly a09 plus32 lowercase hex digits.
The architecture comes from the existing closed x86_64/arm64 selector; source,
tree, native receipts, builder pins and workflow inputs are not generalized.
The historical metadata is amd64; an ARM-compatible code branch is not an
executed ARM exporter result.

Rootfs has exactly type/diff_ids, type layers,1–128 distinct canonical SHA-256
DiffIDs. Their ordered paths must equal the manifest layers. Every layer must
be a regular member with its already streamed byte hash equal to its DiffID.
Thus the ChainID inputs come from selected actual layer bytes on the future
validator path, not merely JSON assertions or observed legacy parent values.

Runtime config matches exactly nine typed fields and their closed nested
values: fixed user/command/entrypoint/workdir/volume/port/PATH, exact boolean
ArgsEscaped:true and exactly four owner/edition/selected-source/repository
labels. The recursive exact comparator distinguishes bool from number and
null/list/map substitutions; unknown config/nested keys fail. The selected
history is not serialized into V1 metadata; its list check is a profile
condition, not a new claim that arbitrary history contents were audited.

Created is an exact string in UTC canonical RFC3339Nano shape: valid calendar
date/time, Z suffix and optional1–9 fractional digits ending nonzero. Calendar
validation preserves the original fractional digits for serialization; it
does not round to Python microseconds or silently normalize. Unsupported date/
timezone/profile shapes refuse. This is the selected finite profile, not a
general implementation of all Go time encodings.

Zero ContainerConfig has the pinned mandatory field order and zero strings,
false booleans and null slices/maps. Runtime config overlays that structure,
inserts ExposedPorts after AttachStderr and ArgsEscaped after Cmd, and sorts
the Labels map only. Ports and volume maps each contain one fixed key. Outer
CreateID keys alone are sorted; nested config struct order is kept. All
constructed strings in this profile are ASCII without HTML-sensitive content,
and serialization is compact JSON without newline/floats. No arbitrary parsed
legacy JSON controls generated values, ordering or permitted names.

Each layer derives ChainID and then its own legacy ID. Base has no parent;
successors use the previous computed ID, prefixed for the ID input and
unprefixed for the saved V1 object. Intermediate pre-ID includes epoch creation
and zero container config **without OS**. Top pre-ID uses selected creation,
runtime config, selected architecture and linux OS. Saved intermediate OS is
added only afterward; saved top OS was already present. This matches the
directly read exporter ordering and the immutable design's serializer schema.

For each generated saved object, only its content digest's blob path can be
returned. It must be distinct from ordinary/previously derived names, present
and regular, have exact integer size equal to the bounded expected bytes,
matching streamed digest and byte-for-byte payload. The method returns the
set only after the entire chain succeeds, without mutating the ordinary set.
Extra fields, alternate whitespace/order, duplicate keys, altered id/parent/
OS/config or scalar metadata cannot match those generated exact bytes. This
is source reasoning, not a performed negative case or recomputed observed ID.

## Original archive, architecture and ownership gates stay intact

Full original validator1030–1097 and complete candidate1030–1214 were read.
Compressed/expanded2GiB caps, streamed1MiB hashing and budget checks, small
payload8MiB cap, truncation/size checks, canonical relative/no-traversal paths,
duplicate-member refusal and regular/directory-only restriction are exact old
source. No filesystem extraction is added. Existing manifest exactly-one-
image/tag, config digest equal to selected image, architecture/owner/edition/
source and OCI index/manifest/config/ordered-layer checks remain. Final
`set(hashes) <= allowed` and all blob path/digest checks are unchanged.
The new set adds only derived regular metadata members, no wildcard blob/
JSON/path/parent-directory/extra-image allowance. Existing directory treatment
and optional ordinary metadata checks are preserved, not newly broadened or
claimed independently hardened.

The entire selected recipe/image/builder/public-receipt flow was read. It
retains immutable product inputs, clean checkouts, bounded public native
archive identity/capability parity, token-free child environment and private
serial-resource-only Dockerfile reversal. Builder container/volume/private
instance/creation interval, actual pinned tool image, application user10001,
read-only rootfs, dropped capabilities/no-new-privileges and exact creation
identities remain. New derivation opens no file, invokes no command/provider,
creates no process/volume/container or changes cleanup logic. Image save/load/
revalidate, UID probes, stopped-writer transitions, authorization/refusal/
revision/idempotency, logout and configuration protections are unchanged
unreached body assertions, not outcomes supplied by this source patch.

## Actual retained failure and authorized public files

Only three allowlisted public files under
`/tmp/riauth-wave30-container-37061329815` were opened. Both metadata bodies
were read completely; receipt JSON was parsed fully and its daemon/source/
phase/status/limits, cleanup fields and every54 step control were examined.
No resources JSONL, launch file, ZIP, image archive, layer contents or private
input was opened; no extraction or expected-ID computation occurred.

| File | Bytes | Fresh file SHA-256 identity |
| --- | ---: | --- |
| essentials-image-inventory.json | 3222 | `11f3352299d6715750b7f2547df31e0df85dc62f1e110b5c2cc5e4a7c9c57258` |
| essentials-small-metadata.json | 17651 | `430471f20c63eb533003415c19c3d19c8f83b542af7acb75621cfe150040e5b4` |
| container-cohort.json | 16872 | `ab7c5ce7b87e672db255edc4393ccb27348cb777b14f5f7880f87a68a986ab7d` |

The inventory's20 entries are two directories and18 regular files. Selected
config and manifest identify the same six ordered layers; the six extra
observed V1 records total3,525 bytes. Observed intermediate epoch/zero-config
and top selected-config/architecture/time/parent relationships agree with
the described format. These are public metadata observations; no expected
legacy IDs, metadata digests or actual layer hashes were independently
derived/rechecked by executing the candidate or an equivalent algorithm.

Actual run37061329815/job111018425393 receipt identifies controller56bd0829,
old helper SHA0f499284 and x86 productb619/treea627df2. Its historical workflow
SHA7a0b7fb1 differs from the current closed-architecture workflow9af6d1d4;
historical runtime is not credited to the later source. The receipt lacks a
top-level architecture field; daemon x86_64 and selected historical pins are
the actual recorded architecture witnesses. New closed ARM selection is unrun.

Essentials and Platform builds exited0 (762.677s/972.371s), Essentials save
exited0 (17.976s), then validation FAILED at
`image_tar_unreferenced_member` in phase images. Platform save, load/revalidate,
image product execution, UID/mount probes and E→P→E fixture are unreached;
checks/uid_probes are empty, shared_full_gate:not_run, official_release:false.
Cleanup_errors is empty; builder children removed and owned inventory empty;
remaining tracked groups/containers/volumes are zero. All54 recorded children
are reaped/groups empty. These are retained historical observations, not a
fresh probe. Receipt resource summary records888 samples/minimum85,502,218,240
bytes and maximum gap2.0004333619999954s; raw resource samples were not reread.

Root's original publication `246e505dddd4defa90ce549441611be893e35d64`
adds the immutable `docs/roadmap/evidence/wave30-a09-container-37061329815.json`
receipt; that commit's publication/path metadata was checked, not its entire
1,363-line receipt reread. Outer artifact11252311290 identity and ZIP
SHA-256 `d150a5950cbe6ff2dc4399614eca2bf4dfa274ac0db755739ef328a29f9c76b6`
retain root/05187aa attribution. This slice did not open or rehash that ZIP
and has no inner archive hash to claim. Stale b619 native-notices limitation,
earlier UID failures, historical native artifact results and their original
pins remain separate. No failed cohort is retrospectively relabeled passed.

## Actual static checks, limits and smallest next reservation

Own source/data utilities performed Git-object reads, AST parsing, SHA identity
checks and whole117-line reversal; all substantive checks EXIT0. There was
one initial EXIT128 lookup because the three-character shorthand d84 is not a
resolvable Git revision; immutable path history supplied full d84d7539, and
its exact body equivalence then passed. No source correction or runtime retry.
Large combined read output was truncated; bounded rereads completed all
claimed helper/design bodies. Official source access failures are recorded
above, without unsupported body/hash claims or an executable network fallback.

No validator, pure derivation/serializer, selected definition, stub, test,
harness, helper import/constructor/main or controller was executed or compiled.
No actual metadata-ID computation, Docker/build/load/native/provider/product
CLI/Cargo/PG/HTTP client/socket/listener/Driver/browser, deletion or runtime
slot. No author/worker/orchestrator contact, new task/worker/WT/managed shell,
alignment/merge/main/push/status or other report change. Closed rows and
primary assignments stay unchanged. The separate actual D01 descriptor128
memory pass is root's new evidence; this A09 slice neither rereviews it nor
credits a browser journey.

The smallest next seam remains ONE root-reviewed build-free retained Essentials
archive plus synthetic validation at exact59f helper. Pin the retained inner
archive before invoking it; stream opaque layers, never inspect their filesystems.
Preserve the original failure and test exact six required derived members and
focused extra/altered/rehash/chain/type/date/path/cap refusals within the existing
bounded design. No driver/source is created and no runtime command/slot is
released here. This can establish exact-byte validator compatibility, not
load, UID, fixture, ARM, release or whole-A09 completion. Root owns source
integration, prospective driver review and that single validation reservation.

Actual report-only checks: `python3 scripts/check-docs.py` EXIT0, Markdown
links and build-directory layout checked; `python3 scripts/check-repo-hygiene.py`
EXIT0,946 tracked files checked with this report staged. `git diff --check`
and `git diff --cached --check` EXIT0, no whitespace errors. Static scope checks
proved exactly this one new report, all pre-existing tracked files unchanged,
no other staged/unstaged/untracked path and no candidate helper imported into
the own branch. Final newline/trailing-whitespace checks passed. Source/data
checks are distinct from the recorded shorthand lookup/source-fetch errors;
no validator/runtime correction or retry was performed.

The report-only commit and post-commit clean-state check are the immutable
handoff. Root owns source acceptance/integration and any later bounded
validation; no gate/status mutation or execution authorization is made here.
