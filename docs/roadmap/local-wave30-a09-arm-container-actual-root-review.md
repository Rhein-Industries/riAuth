# A09 native ARM container result reviewed by root

2026-10-03. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.

The single native ARM64 run **37101183416**, job **111140870511**, attempt1,
passed its eleven local cohort checks. Product source is
`9a819317efb3a13fa27cd86f884be2be00898fc0`; controller source is
`544d1340b80cd3e040dc13142cdcbc1d75fea4cb`. This is evidence at those exact
revisions, not execution of subsequently published source.

The [root receipt](evidence/wave30-a09-arm-container-37101183416-root-review.json)
records the artifact, source/workflow/helper and member hashes, bounded result
projections, resources, cleanup and review limitations. Artifact11267885639 is
112643167 bytes, SHA256
`82693c5e8b90118e9e927c62605fec387da71bca2cc5869dfd89e0413b466501`.
Root independently hashed the downloaded outer ZIP and all eleven members;
only three JSON/JSONL evidence members were extracted. Images and reader were
neither extracted nor executed by root.

Both images were built natively, and a source-built snapshot reader supplied
18 fresh physical observations. Ten offline packets retained complete logical
row hashes/counts, metadata, configuration and key comparisons with no added,
changed or removed rows. Their physical redb bytes differed; logical equality
must not be described as whole database-file equality. The earlier ARM native
input's `root_outer_zip_rehashed=false` remains distinct from root's fresh
transport rehash of this container cohort.

All2726 owned steps report joined children and empty groups. Nonzero exits
include deliberate refusals and absence checks; this is not an all-commands-zero
claim. Four serve and four owned stop lifecycles exited0. Cleanup reports no
errors, groups, containers, volumes or pending maps; image inventory and builder
children are cleared. The validation/Cargo reservation was released after root
verified terminal outcome and cleanup, before this evidence publication.

1672 resource samples report minimum108513771520 bytes across the monitored
filesystems, above the unchanged30GiB launch/10GiB stop/8GiB floor controls.
Sampling is not a hard quota. Root recomputed count and minima exactly; the
JSONL's rounded2.004-second maximum gap agrees with full-precision
2.004374746999929 within0.0005. An initial root assertion incorrectly required
exact gap equality and failed; its correction changed no source or evidence.

`official_release=false` and `shared_full_gate=not_run` remain unchanged.
This is a LOCAL ARM64/resource-only Dockerfile cohort, with its exact declared
shared identity/authorization/configuration checks. It does not establish a
shipped release, all platforms, tenants, devices or signing. The retained b619
native notices were later found stale and were not regenerated. A09's original
scope disposition remains subject to independent review of the combined native,
container and shared-state evidence. Historical failed runs remain failed.

The job log was retained and hashed before interpretation. Root read bounded
warning/result projections, not every build-log line or an individual replay of
all runtime steps. Source/document checks and exact scope are verified before
this report commit; no additional build, native artifact or runtime is invoked.
