# Wave30 shared-capacity read-only inventory

2026-10-02. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, reservation
`wave30_shared_capacity_readonly_inventory`, existing WT
`a1303b57-4a34-487e-9c63-a841f05b51a0`, branch
`roadmap/local-workflow-safety-wave27`, clean starting HEAD
`0b6f4f678ea7a1903d8f6c5843c8adf4517d238e`.

**Result: continuing shared-volume pressure; no safe cleanup allowlist
established.** Fresh free disk fell from 3.323 GiB to 2.877 GiB during this
inventory, already below the 8 GiB floor. Own cache allocation stayed exactly
unchanged between measurements. Large retained Cargo targets are a concrete
capacity burden, but these observations do not identify the writer or cause
of the abrupt decline. No file was deleted, moved, truncated or cleaned.

Root reports D01 refused its fixture before setup after fresh free disk fell
from approximately 11 GiB to 4.415 and then 4.141 GiB, with desktop FREE.
Those are root-supplied incident observations, not this lane's execution.
Root holds preparation for hosted A09 run `37043196924`; no remote run state
was queried here. No local Cargo/runtime was authorized or launched. The
earlier 10/16/14-file pruning permissions are exhausted and confer no new
permission on this lane.

## Actual filesystem observations

Own filesystem: `/dev/disk3s5`, mounted `/System/Volumes/Data`, device ID
`16777234`. `df -k` reports rounded capacity 100% in both snapshots.

| Actual UTC observation | Free bytes | Free GiB | Method |
| --- | ---: | ---: | --- |
| 2026-10-02T17:49:38.432298Z | 3,568,140,288 | 3.323090 | `shutil.disk_usage` and `df -k` |
| 2026-10-02T17:55:04.223106Z | 3,089,403,904 | 2.877232 | `shutil.disk_usage` and `df -k` |

The decrease is 478,736,384 bytes, approximately 0.446 GiB, over those
read-only observations. The prior in-session accepted SCIM receipt recorded
13,688,934,400 bytes free at 16:05:11Z; comparison with the first inventory
snapshot is a historical 9.426 GiB decline. That historical value was not
re-read from evidence bodies or recreated by another run here. It does not
attribute the decline to a particular cache, worker or filesystem operation.

At 17:50:29Z, `du -sk` examined 94 existing nonsymlink
`/Users/dominik/orca/projects/riAuth-public-preview*` directories. Their
reported aggregate allocation was 135,499,036 KiB (129.221951 GiB). The
following directory-level comparison shows the dominant measured targets;
these paths are inventories, **not deletion candidates or grants**:

| Project directory suffix after `riAuth-public-preview` | Whole directory GiB | Dominant cache directory | Cache GiB |
| --- | ---: | --- | ---: |
| `-roadmap-m03-scim-management-parity-wave15` | 20.215649 | `target` | 20.200333 |
| `-local-management-wave27` | 13.695358 | `target` | 13.652279 |
| `-roadmap-integration-accepted` | 12.741901 | `target` | 12.531853 |
| `-local-revisions-coordination-wave27` | 12.690002 | `target` | 12.668129 |
| `-roadmap-q08-exact-bundles-wave16` | 11.938850 | `target` | 11.923252 |
| `-local-ci-diagnostics-wave27` | 10.878757 | `target` | 10.836861 |
| `-local-extension-isolation-wave27` | 8.513363 | `target` | 8.489967 |
| `-roadmap-s02-index-paginate-wave15` | 8.241688 | `target` | 8.229664 |
| `-roadmap-a03-module-boundaries-wave7` | 8.169189 | `target` | 8.153717 |
| `-local-workflow-safety-wave27` (own) | 6.472767 | `.target-wave27` | 6.449368 |
| `-sol-management-wave30` | 3.990562 | `target` | 3.968781 |
| `-sol-diagnostics-wave30` | 2.682980 | `target` | 2.660786 |

Only directory allocation was inspected outside this WT. No other lane's
file/evidence contents, process arguments, credentials or environment were
read. Directory snapshots establish present occupancy, not growth since a
prior run: there is no comparable earlier directory-level allocation baseline
in this audit. `du`/file blocks are not guaranteed reclaimable bytes on a
shared APFS volume with clones/snapshots; the project aggregate is not full
volume accounting. Unrelated directories/snapshot contents were not surveyed.

## Own cache metadata and noneligibility evidence

The own nonsymlink `.target-wave27` directory has UID 501, mode 0755. It
remained exactly **6,762,652 KiB** at the 17:50:29Z and 17:55:04Z snapshots.
Its regular-file allocation is 6,924,955,648 bytes; logical lengths total
6,908,252,584 bytes. All 5,365 regular files inspected have UID 501. This
establishes filesystem ownership, not cleanup eligibility.

| Own directory | Allocated KiB | Allocated GiB |
| --- | ---: | ---: |
| `.target-wave27/debug/deps` | 6,358,452 | 6.063892 |
| `.target-wave27/debug/build` | 70,520 | 0.067253 |
| `.target-wave27/debug/.fingerprint` | 11,808 | 0.011261 |
| `.target-wave27/debug/incremental` | 0 | 0 |
| Own separate `target` / `target/wave28` | 4 | 0.000004 |

Metadata classification found 131 regular files with executable permission
bits, totaling 4,519,088,128 allocated bytes (4.208729 GiB). No binary header
or body was read to classify them as native executables. The three largest
regular files are `libriauth-507476e43f01092f.rlib`,
`libriauth-f685f1e2409027d4.rlib` and
`libriauth-082f619b43905096.rlib`, approximately 409/407/405 million logical
bytes each. The apparently duplicate `debug/riauth` and a `deps/riauth-*`
entry have equal length/mtime, but metadata equality neither establishes
content identity nor proves reclaimable space or historical nonexecution.
These are not proposed cleanup items.

Current retained-file mtimes after `2026-10-02T16:05:12Z` identify only two
regular files, totaling 8,666 logical bytes and 16,384 allocated bytes:
the accepted SCIM helper-run `.verification.json` and `.verification-v2.json`
receipts, mode 0600, last modified at 16:06:22Z/16:06:55Z. Their bodies were
not opened. The cited SCIM target `scim_oauth-951d26c550be29f2` has 211,639,968
logical bytes, mode 0700 and mtime 16:04:40Z; its known in-session 24/0 use
and accepted receipt exclude it from a never-run list. No binary hash was
recomputed, because reading binary bodies is outside this reservation.

This current mtime inventory does not show multi-GiB retained-file growth in
the own cache after that run. It does not reconstruct past deletions,
overwrites, preserved timestamps, transient allocation or APFS accounting,
and does not prove the lane caused no earlier allocation. The two cache-size
snapshots are the direct evidence that own reported allocation stayed steady
while shared free bytes declined during this inventory.

At 17:53:11Z, bounded `lsof -nP -F pn +D` on **only this own cache** reported
zero open processes/files, no stderr, exit 1 (no matching opens); the wrapper
completed normally. No raw output containing arguments/environment/private
values was printed. This is a point-in-time visibility observation, not a
historical never-run assertion, future-use guarantee or deletion authority.

**No exact allowlist or ignored manifest is proposed.** Size, age, ownership,
inactive/open-file status, apparent duplicates or lack of an observed
citation cannot establish both **never run and never cited**. Complete
historical execution/citation provenance is not established by this metadata
inventory. Absence of a cited hash is not evidence of nonexecution; binary,
protocol and evidence bodies were deliberately not inspected. This result
does not assert that every cache file has been executed, only that no safe
eligible candidate was established under the requested standard. Accepted
artifacts/reports/logs/backups remain untouched.

## Capacity prerequisite and exact performed scope

Keep all local preparation/Cargo/runtime held. Root must decide remediation
under a **new exact authorization**, if any, then require a fresh credible
capacity/peak/live-use preflight before separately releasing work. Recovery
above the 8 GiB floor alone does not provide build headroom: the previously
reviewed local SCIM command required at least 11 GiB start with a 9 GiB stop.
A hosted A09 attempt has its separate unmeasured fresh 30 GiB start/10 GiB
stop/8 GiB floor prerequisite; this local inventory neither establishes nor
changes that host's capacity or run outcome. No provider substitution or
cache deletion is implied.

Actual read-only methods: `stat`/`lstat` via filesystem metadata, `df -k`,
directory-only `du -sk`/`du -k -d 1/2`, own regular-file size/block/UID/mode/
mtime inventory and bounded own-cache `lsof`. `du`/metadata wrappers exited
0; the no-open-files `lsof` result is separately identified above. No
product/native-version/helper/Cargo/service/runtime/network, desktop,
worker contact, binary/protocol/evidence body read, raw args/env/private-value
readout, deletion/move/truncation/cleanup, manifest, new worker/task/WT/shell,
source edit, merge/reset/status/main/push action occurred.

Sole write is this new report. Whitespace/new-file-only scope and clean-branch
checks accompany its commit; a whole-repository docs checker was not run
because this reservation excludes reading existing evidence bodies. Root
alone decides remediation/release/publication/status. No D01/A09 success is
manufactured, and I10/R05/W02/W05 remain DONE.
