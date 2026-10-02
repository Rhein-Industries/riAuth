# W02 conditional enrollment root review

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; W02 task
`548d114f-9d0a-474a-a4c8-fa03af3ec3b1`. Reviewed against published main
`c8079570a819b7d89a694b178b90a8228fdefec2` on 2026-10-02.

## Accepted bounded slice

Source disposition `aa9d05f34a548d07355af83f524c6f226a4901df` is accepted as
`122238a3f1335bb61d2b979a33f004c7e7d6b62e`; source code
`69b9eda4939738d6a9ef8fdb10351991d336feeb` as
`a03731379e259ed8252d09db5a47d9445776caae`; source evidence
`b144a62c874b9157c20c8ea3d74a4a3b2649176a` as
`97ff1d361c2f5bdc9cb8426a78f722b2bf2f2a62`. Preparatory merges are excluded.

The original five executor controls have implementation and focused historical
evidence. Root interprets the server executor workstream's conditional enrollment
gate as requiring one connected executable path. This does not require arbitrary
graphs, all verifier permutations, initial sign-in or a deployment campaign.
The model already expressed conditions; this slice connects one exact graph to
the existing executor.

Root reviewed the complete production diff and all six new test definitions.
Admission compares the exact ordered session/UV-passkey/enroll graph, actions,
conditions, bounds and terminal proof requirements. The start exception applies
only to that exact configured shape. The unconditional matcher and built-in
eligibility stay intact. No verifier, version, approval, source, assembly, config,
API or process-binding file changes are accepted. Existing adapter labeling,
UV proof and atomic registration remain authoritative. Operator documentation
now describes this ninth enrollment shape and its limits.

The worker's final new target passed six tests, covering admission, denial without
enrollment capability, real UV and one-use registration after restart, account
and policy drift, and bounded retries/cancellation/deadlines. Two initial new
fixture failures were corrected without further production changes. The
[worker report](local-wave29-w02-conditional-enrollment-evidence.md) distinguishes
those failures and the compatibility baseline. Root compared immutable source
blobs, reconstructed prior source after removing the two approved hunks, and
checked formatting, documentation and whitespace. Root ran no Rust build/test.

## Compatibility and status

The old named configured enrollment fixture failed both with the new slice and
with unchanged reviewed main: it corrupts the canonical graph, triggers durable
retirement, restores only its old active run and attempts the discarded ceremony.
Accepted review sealing consumes proof and discards registration. This is a
reproduced pre-existing fixture dependency, not passing compatibility evidence.

Root authorized a separate correction confined to that named test: assert
retirement and discarded resources, preserve final state when repairing the
corrupted definition, and use a fresh public run/UV proof/registration for the
positive restart path. No relaxation of production sealing is authorized.
W02 remains in progress pending that correction and root closure review.
W05 and M07 remain done.

Subsequent review: the exact corrected compatibility fixture passed. The
[W02 closure review](local-wave29-w02-closure-root-review.md) resolves this
pending disposition and accepts the original W02 outcome after publication.

## Supporting browser diagnosis

The [browser diagnosis](local-wave29-browser-start-ci-diagnosis.md) is ported as
the exact cumulative report blob at `e59ec21b8921dda9ae5c9da90f41298799a789e1`,
including its correction of `e8e01b43560cf1d0f060c88b62792c7a63f3a4cd`.
Only the corrected report is accepted; no browser diagnostic code is imported.
Whole-test duration and unchanged code do not establish Chrome startup latency
or exclude an unchanged OP/request/runtime failure. Run `36957782065` also had
a successful integration job, while its check job repeated the credential
fixture failure already corrected in published `c807957`. Current run
`36959097782` was still running when reviewed. No green-suite or proven browser
root-cause claim is made; O06 remains in progress.
