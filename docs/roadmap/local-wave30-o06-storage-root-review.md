# O06 storage availability: root acceptance

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; 2026-10-02.

Accept storage source `39e8896ac1a49ef96e95807f3c6c62c67cd213d7`, test-only
correction `bc412e8d1d7b4e3ee23875fc1c149372f7982d5f`, and
[actual evidence](local-wave30-o06-sol-completion.md). Full O06 remains open.

The new pure decorator labels pressure unavailable, states that capacity has
not been measured, distinguishes unavailable or stale allocation, and provides
fixed host-specific remedies. It never turns successful bytes, doctor or
readiness into evidence of headroom or safe writes. This explicitly exposes
the missing pressure measurement; it does not implement physical pressure.

Root inspected the complete decorator and both additive call sites. Core
permission checks still precede stat; JSON metrics still call the same cache
once only after the existing dual resource checks. No new storage, network,
writer or catalog operation is introduced. Prometheus, raw Store/cache,
readiness and doctor behavior remain unchanged. Existing raw assertion bodies
remain unchanged; public wrappers strip and strictly check the new subdocument
before applying those original checks. The thirteen public call-site changes
are mechanical and no raw calls accept the public subdocument.

The initial test target compiled unsuccessfully because an unavailable Tokio
feature was used. The isolated test correction uses the supported blocking
process pattern with no dependency change. The worker then passed two focused
new tests and one exact edited-helper regression on macOS/redb. These cover
real CLI/HTTP output, permission boundaries, allocation cache states, redaction
and unchanged records. They do not provide PostgreSQL, Essentials runtime,
physical pressure, external-peer or overall CI evidence. Root ran no Rust test. The [independent static slice review](local-wave30-o06-storage-independent-review.md) also accepts this bounded change and verifies the raw assertion/call-site reconstruction.

The [independent six-facet review](local-wave30-o06-independent-review.md) is
accepted as a source-derived finding inventory, not a completion claim. Root
keeps current signer attribution, controller visibility, job/evidence remedy
and pressure gaps separate. Missing delivery evidence cannot be labelled
verified remote success. Bounded follow-ups have explicit ownership; no new
readiness, mutation or credential contract is authorized by this report.
