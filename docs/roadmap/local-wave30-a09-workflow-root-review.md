# Wave30 root source review: manual native ARM64 artifact slice

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, original A09
`506e3979-a590-4af3-8fa8-ee90d3a517f2`, 2026-10-02.
Root read the complete hosted-source proposal and its entire inline driver at
`f886ae9e074b94883bc05619748025ce3e84b221`. Materialized source
`5962a63e363028f1994fb4f94a2608ea2dad55f8` changes exactly one new workflow.
Root independently verified all 18130 bytes and SHA-256
`883e428ce876154850889949d21760576229f82800f89d6acde28e2671ef34aa`, parsed
YAML with installed Ruby Psych, parsed four shell blocks with bash -n and
parsed the inline Python AST without executing it. No dependency installation
or product build occurred during root review.

## Accepted seam and prospective actual dispatch

Accept the exact manual-only workflow, contents:read, one native ARM job,
pinned checkout/toolchain/upload actions, no automatic cancellation and
150-minute job bound. Workflow/run identities are real and separately recorded
from the full input product source SHA. The source SHA is validated and checked
against the clean checkout before production commands. Builds are three
sequential locked private jobs1/incremental0/debug0 release invocations:
Essentials server+maintenance, Platform server+maintenance, independent base
client. Five LOCAL archives retain license/notices. Binary/native ELF metadata,
server artifact capability/features and exact hashes are checked; the unchanged
focused archive smoke runs only against those local files. No fake official
packager identity, tag/release/image/registry/attestation is introduced.

Root authorizes one manual dispatch after publication with reviewed product
source `9a819317efb3a13fa27cd86f884be2be00898fc0`, which includes the reviewed
SCIM probe and reports fixture. The workflow definition publication/run pin
will be distinct. This authorization is for the complete bounded job, including
its initial actual resource refusal; no automatic retry or runner substitution.
Root will perform the dispatch and inspect/download actual results. Workers
may not dispatch, run local Cargo, change provider settings or lower guards.
A later shared identity/authorization/config test driver remains separately
reserved work; this archive slice cannot establish the full original A09 gate.

## Resource boundary

The provider's documented standard ARM storage is smaller than the proposal's
30 GiB required starting availability; there is no current measured runner
receipt and no capacity guarantee from the label. The first step records host,
workspace and private available bytes and refuses below 30 GiB before checkout/
tool setup/build. Phase preflights repeat that threshold. Owned subprocesses
are monitored every two seconds and stopped at 10 GiB, preserving margin over
8 GiB. No cache deletion, reduced floor, paid/larger runner or alternate label
is authorized. A failed capacity preflight is evidence of a blocked artifact
slice, not successful production.

The driver owns private process groups, command deadlines/log bounds and
TERM/KILL reaping. Failure/refusal evidence uploads with always(). GitHub-managed
setup actions are outside those owned groups; sampled guards are not a hard
filesystem quota. Upload after cancellation/lost runner and cleanup of escaped
processes are not promised. Native dependency/tool/image mutability and minimum
libc portability remain limits, as in the proposal.

## Current disposition

A09 remains in_progress. No new native binaries, archives, Linux runtime,
container/x86 products, independent distribution identity/authorization/config
checks or official assets exist from this source-only phase. Prior historical
products retain their exact provenance. No artifact success is inferred from
YAML/static checks. Root alone integrates/publishes/dispatches and decides status.
See [the full proposal and earlier artifact matrix](local-wave30-a09-artifact-plan.md).

## First actual dispatch was rejected before execution

After source publication `3a57affd9023a48c32085d6bcfb1d17ca4feb901`, root
invoked exactly one manual dispatch with product source `9a819317...`. GitHub
returned HTTP 422: line27/column17, unrecognized `runner` context in the
job-level `A09_ROOT: runner.temp` expression. No manual run, runner, tool setup,
resource measurement or Cargo build was allocated. The sole runtime slot is
released. Static YAML/bash/AST parsing, including root's checks, missed this
GitHub context restriction; their earlier passes are not platform acceptance.

Two automatic invalid-workflow metadata records on publication are also
retained: `37014464740` (main3a57aff) and `37014464605` (accepteda9279cc),
both event=push/status=completed/conclusion=failure/jobs=[] as actually queried.
They are parse failures, not successful manual dispatches or builds, despite
the intended manual-only trigger. They supply no native resource/artifact proof.

Root checked [GitHub's context availability reference](https://docs.github.com/en/actions/reference/workflows-and-actions/contexts#context-availability):
job env excludes runner. A separately reserved minimal correction computes
A09_ROOT from actual RUNNER_TEMP/run IDs inside the first shell step and writes
it to GITHUB_ENV for subsequent steps; all driver/actions/guards/build logic
remain fixed. One corrected dispatch requires another root source review and
publication; no automatic repeat or resource-policy relaxation is authorized.
