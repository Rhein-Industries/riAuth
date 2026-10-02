# D05 two-artifact refresh — independent review

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; D05 task
`6a2381ab-0f2f-48c6-adf9-8115b78cc143`; 2026-10-02.
Report-only reservation in existing worktree
`7c85f5ef-3fac-4f72-aaed-08474d7fb454`.

## Verdict

**No blocking preservation, correspondence or inspected provenance error found.**
Recommend root accept refresh `eb1412aa6799ab959b90bdf71f29c983e15b67aa` as a
dated, evaluated-with-gaps evidence snapshot. This is not a D05 completion
recommendation. The full gate remains false; D05 remains `in_progress`.

The original RiWork row was reread with explicit project membership. It asks to
evaluate usability, workflows, administration, interoperability, footprint,
performance, availability, recovery, migration and security evidence. Its goal
is documenting completed user/operator tasks, and its gate is that a new user
and a new operator can independently complete the documented workflows. Both
artifacts preserve that distinction from accepted original implementation rows.

The artifact's source pin is published
`c01c39ab4e092423d5522bedc50fff87656d8c0a`; its board observation is dated
`2026-10-02T11:40:43.377271+00:00`. These are not current observations of later
main `6b4db4f0317e4427c187f55e063989ccba124217`. Root has subsequently closed
original O06; this review leaves it closed. The snapshot's O06-active statements
and earlier diagnostic limits remain historical observations, not an instruction
to reopen O06 or silently import later readiness/signer/budget evidence.

## Exact inputs and preservation

Reviewed commit `eb1412aa6799ab959b90bdf71f29c983e15b67aa` has parent/approved
proposal `da787b4525a368220c63995ea1129d5f7243754e`. Its entire delta consists of
the two authorized evidence artifacts. This report's local parent is
`9610631c2b3e722696be82a29f654f2c17a7c084`; no source alignment or merge occurred.

| Artifact at reviewed refresh | Git blob | Bytes | SHA-256 |
| --- | --- | --- | --- |
| `docs/roadmap/d05-acceptance-evidence.json` | `efe71acea3b549d0027bdf42e8eb3ae07ca91ce2` | 180187 | `84cfbd6c8f8c83dbb5367e00c0b389157548cdf79d0a533de915f9d7049f9ef7` |
| `docs/roadmap/d05-acceptance-evidence.md` | `6651a2ecc016a2f86d79d1b99716522cd533e6ae` | 118897 | `f72e875bfc2428d9dcad09d6dd45b14996f4a12a4c01db5c8f97c346251bb066` |

The complete previous JSON **value**, not just categories or selected fields,
equals `prior_snapshots[0].snapshot`. Previous JSON bytes at the refresh parent
and at `c01c39a` are identical. Its historical blob is
`501e4b4af8ce1529c52bc3edececbd2f30949a5c`, SHA-256
`679bd9a31f6d6c58c7fc2057d256e6c717a6b642b6c4b75abd613ff669f5b3a8`.
Both old and new JSON parse without duplicate keys.

The previous 53521 Markdown bytes occur exactly once between the explicit
historical markers; removing those bytes leaves only surrounding whitespace
inside the markers. Historical Markdown blob
`9e3116c1cc50a984ed3ba3e1613efff560fa271c`, SHA-256
`bc462d87981630c53fec7d0ffe4f2a8602d6877ec451f4a0c33f7b6dee8a95cb`,
matches the preserved reference. Prior statuses, counts, failures and old
remaining text are therefore retained verbatim.

The old ten `newer_slices` objects are an unchanged prefix. Seventeen separately
identified current evidence objects follow them, for 27 total slices. The
refresh explicitly explains that newer mappings supersede obsolete historical
category-effect/remaining statements without erasing their recorded outcomes.

## JSON/Markdown correspondence

Checked all ten category sections against their JSON target, fulfilled scope,
unmet inputs, claim limits, next evaluation, accepted/active original-row IDs,
evidence IDs and status. Accepted rows agree with the artifact's own dated board
snapshot, not a substituted current board. Each category retains
`evaluated_with_gaps` and `category_passed: false`; top-level `d05_passed` is false.

| Category | Exact mapped evidence IDs | Gate/scope conclusion |
| --- | --- | --- |
| Usability | E-CI, E-GUIDE | Browser/software-authenticator execution is credited; independent user completion remains unrecorded. |
| Workflows | E-WORK, E-GUEST, E-MGMT | Accepted bounded executor/enrollment/confinement and management contracts are distinct from an independent operator completing the documented workflow. |
| Administration | E-MGMT, E-S04, E-DIAG | Original M03/M07/A03/S04 acceptance is credited without adding universal browser resource coverage; dated operator-journey/O06 observations remain separate. |
| Interoperability | E-CI, E-PEER, E-GUIDE, E-SCIM, E-WSPACE | Exact local peers and supported SCIM/Workspace implementation are credited, without named-tenant/family-wide certification. |
| Footprint | E-ARM, E-Q09 | Native local binaries/archives and historical RSS are distinct from same-revision official shipped bundle/installation gates. |
| Performance | E-S04, E-Q09 | Measured bounded writer-cost removal and paced observations are credited; no maximum throughput, statistical/general speed or large-Group claim. |
| Availability | E-O03, E-CI, E-PAGE | Supported agreement/coordination, disposable promotion and local paging evidence do not establish deployment HA or running-peer health. |
| Recovery | E-REC, E-ARM | Encrypted logical and physical-base-backup drills retain their different scopes, pending gate and external reconciliation inputs. |
| Migration | E-MIG, E-ARM | Accepted conversion/reimport and local edition transitions do not establish customer cutover, actual RP callback or older-binary rollback. |
| Security | E-CI, E-GUEST, E-MGMT, E-O03, E-S04, E-REC, E-RELEASE | Scoped contract/confinement/refusal evidence and source/local inventories are distinct from installed-release, signing, independent conformance and publication evidence. |

All seventeen stable evidence IDs appear in their own Markdown sections and are
linked by the category mappings. Category `records` agree with the global record
class and fixed source pin. Every appended slice agrees with its evidence record's
ID, summary, class, verification and accepted-source pin. Every evidence summary,
verification label, recorded command, referenced commit, path/blob/SHA tuple and
raw-log path/hash/byte count corresponds in JSON and Markdown. Because summaries
match exactly, their embedded counts and failure statements also match; that
correspondence is separate from independently checking the underlying evidence.

E-SCIM/E-WSPACE share a historical narrative, and E-PEER/E-GUIDE share another.
They have different mappings/references/commands, but do not establish duplicate
independent executions. The refresh expressly forbids adding reused counts or
subsets/repeats across categories. Seventeen IDs are not seventeen new runs.

## Provenance and evidence bodies actually inspected

Identity verification covered **75 unique current evidence-reference paths plus
the two historical artifact paths, 77 total**. For each, the declared pin resolves
the declared Git blob and the content SHA-256 matches. This is a hash/path check,
not 77 complete semantic source-body reviews.

The explicit referenced-object inventory contains **20 commits and one blob**.
Actual `git cat-file -t` types match. The blob is the unchanged pagination fixture
`aa1c3bdc70b232796dfb0c150179b8165384b85a`; it is not incorrectly treated as a
commit. Commit existence is not assumed to prove direct main ancestry or a
matching execution. Source/run/accepted-port roles remain separately narrated.

The following body checks substantiate the selected claims and limits:

| Evidence ID | Body/result inspection and conclusion |
| --- | --- |
| E-CI | Existing integration log checkout and command/result lines: three XMLsec filters each 1 pass; default PG 2/17.51s; selected PG contracts 91/90 filtered/209.68s; Q05 1/1 filtered/13.98s; LDAP/browser/portal/nginx/Traefik each 1; setup 9; authenticator 22 with 2 skipped. Disposable promotion recovery 1113ms is logged. The PG script dispatch bodies confirm exact target/features/filter expansions. This is a named historical job, not 91 O03-only tests or a current full CI pass. |
| E-WORK | Entire W02 closure review and selected enrollment-fixture evidence: conditional 6/6 and separately corrected exact regression 1/1 remain distinct, with prior baseline failures retained. No universal graph/protocol/hardware claim. |
| E-GUEST | Isolation report outcome/initial-failure/unsupported-host sections and entire six-task root review: final native macOS 19/95 filtered/33.06s and reported accepted equivalence are credited. The guest raw log was not independently rehashed here. |
| E-MGMT | Entire M03 and activation parity root reviews: 38 API functions including the 8-case subset, 10 approval functions with 6 PG ignored, 2 configured-adapter functions. Authority-before-receipt, Optional/Required routes, reviewed creation receipt-secret and PAM contracts remain deliberate. Root-reported history is not a newly executed suite. |
| E-O03 | Entire O03 root closure, scoped historical records and the CI source comparisons: format-3 local evidence remains separate from older format-1/2 process checks. Sixty-second nonrenewed admission and paused external-I/O limitations are not upgraded. |
| E-S04 | Entire root disposition and existing raw log. Independently parsed 24 cost rows and 8 security rows: 96 calls/lane; global 192 holds/96 commits/749695µs occupancy including 495196µs commit time; scoped zero. Recomputed 16 apply refusals and 24 ordered cases, AB/BA/AB order, and final 1 pass/17 filtered/20.47s. The reported 24 final apply/replay cases are credited through the accepted review, not newly fault-injected here. |
| E-PAGE | Entire settlement root review and both existing logs: baseline 0 pass/1 fail/1 filtered/1.44s at line229, corrected 1 pass/1 filtered/1.71s. Unchanged fixture blob is correctly typed. No new Linux or release-failure fault-injection claim. |
| E-DIAG | Entire storage/key/controller/missing-evidence/gauge root reviews at `c01c39a`: configured-controller 1/1.06s; gauge expected422/actual400 failure then 1/2.14s; unresolved uncertainty and remote-outcome distinctions survive. Storage's earlier compile failure/correction remains in its referenced record. These are earlier bounded O06 slices, not later O06 closure evidence. |
| E-ARM | Selected RUN/ENCRYPTED-RUN bodies plus tracked native-v2/encrypted manifests and integration-review fields: source/validator pins, five binary sizes and local release-gate false agree. Logical preserved rows 30/33 redb and 36/39 PG, ciphertext checks and historical format2 are retained. Earlier harness/validator/late smoke failures are not erased. No binaries/images were independently executed or rehashed. |
| E-Q09 | Secure-run provenance, TLS/SCRAM, pacing and result tables in the benchmark page: harness `4ab934a`, binaries `6ca4779`, four edition/backend combinations, 12 successful reads/pass, 71.574398–71.657009s, writer2success/2expected409 and RSS maxima50408–55992KiB agree. Product sslmode=require and libpq verify-full remain distinct. Twelve samples make p95/p99 maxima; success/s includes pacing. External report files were not rehashed. |
| E-REC | Tracked encrypted PG restore JSONs each contain 16 passed checks and the stated distinct binary hashes. Physical PG16.14 report contains 16 passed checks, `passed_gate_closed`, no source-commit field, same-lineage blind spot and unmade completion attestation. Selected D04 base/PITR outcome/check bodies preserve observed status-before-invalidation and pending/refused serving afterward. They are not summed into 32 deployed recovery passes. |
| E-MIG | Entire six-task root disposition and selected completion-review conversion/reimport evidence: accepted original conversion/continuity outcome is distinct from historical synthetic G05 reference-RP cases and real customer cutover. No runtime log was independently rehashed. |
| E-SCIM | Selected six-task source/assertion review supports 130-user redb1/1 and plain/encrypted PG2/2 at the stated accepted pin, supported schemas and disable/revocation/downstream intent. No new source implementation audit or named SaaS certification. |
| E-WSPACE | Same accepted review's direct signed grant/endpoint/fresh-key/expiry/failure section supports the historical cloud_directory36/36 and reviewed helper equivalence. Local fake peers remain explicit; no cloud mutation or tenant validation occurred. |
| E-PEER | Preserved previous Q04 body and LDAP recipe's named OpenLDAP2.7.1 commands/results identify loopback LDAPS/STARTTLS and retain historical radclient3.2.10 provenance. D01 walkthrough sections retain local tools. These are scoped peer utilities, not hardware NAS or provider-family certification. |
| E-GUIDE | D01 walkthrough overview, selected run/outcome/unrun sections and guide completion limits retain eleven partial disposable runs and incomplete journeys. They are not eleven independent complete user/operator acceptances. |
| E-RELEASE | Q11 local unsigned inventory section and release-evidence limits:301/353/121 packages differ deliberately from normal closure280/329/106. Tracked source tooling/attestation requests and local inventories exist; an executed signed/attested/published bundle is not established. No live release lookup or bundle validation. |

Twelve accepted root-review Markdown bodies were read in full: W02 closure, M03,
activation parity, O03 closure, S04, pagination settlement, six-task disposition,
and the five O06 storage/key/controller/missing-evidence/gauge reviews. Other
report/runbook/recipe/test/source references above received selected section or
field inspection; they are not counted as whole-body reviews. In particular,
hashing product/test sources does not mean every function/assertion was reread.

Ten explicitly selected predecessor CI source/test/script/workflow blobs are
byte-identical between `2d05c00deed3a6dafcd458a5f1a1a9beb17d0dd8` and `c01c39a`:
node_security, config, background targets, token freshness, postgres,
contracts/shared, q05_replay_concurrency, ci.yml and both PG scripts. This supports
the stated transfer boundary for those exact paths. It does not credit changed
state/adapters or the whole current tree with a new execution.

## Retained raw logs independently verified

| Existing log | Bytes | SHA-256 |
| --- | --- | --- |
| `/tmp/riauth-wave29-integration-110661000640.log` | 265770 | `458a30895b5987c0bebe2fb7368a1a32969fa0d41ce90acbfa56f43299ccd186` |
| `/tmp/riauth-wave30-s04-equivalent-authority.log` | 18610 | `6287eb35f69017af138304ca7ca96624765d7a7b60ba49efab936ae2781e0e6d` |
| `/tmp/riauth-wave30-reconciliation-pagination-baseline.log` | 1338 | `5e8be0ae942ccbed9118b9b76a03bf6dc1383d762e13e938ef0a3eff3ae0c7fb` |
| `/tmp/riauth-wave30-reconciliation-pagination-corrected.log` | 772 | `2cc8e8b515edf992ed22d42ea4e596ae8ae45df75271ddc6954537754bcd17b5` |

These logs were read/hashed without execution. Other raw-log or artifact hashes
are reported by their accepted sources; this review does not claim to have
independently checked their original external files. Native unwind warnings,
prior fixture/harness failures, skipped cases and failed overall runs remain
part of the evidence. Named integration/audit success never becomes overall CI
green; run36998781947 is retained as pending at its publication observation.

## Gate, review limitation and ownership

The smallest unmet original D05 gate remains the independently recorded
documented new-user and new-operator journey, with matching supported artifacts
and configuration. D01 owns that work; this review starts no duplicate campaign.
Claim-specific hardware/tenant/release/escrow/measurement inputs remain listed
without becoming universal new completion gates for already accepted original
implementation rows. Category false and original-row DONE can coexist honestly.

The refresh's own failed pre-write reference guards and draft correction remain
disclosed. This independent review's first correspondence script also aborted
without mutation: it treated list-valued unmet-input/claim-limit fields as strings
and raised TypeError. Correcting that checker to iterate their values and use the
actual category-record schema made the complete guard pass. This was a reviewer
checker error, not an artifact/product/runtime failure; no source/artifact was
changed to satisfy it.

Checks performed here are immutable JSON parsing, whole-prior-value/verbatim-byte
and prefix comparisons, complete ten-category/seventeen-ID correspondence,
77 path/blob/content hashes, 21 actual object types, four raw-log hashes and
selected evidence-body/result inspection. `python3 scripts/check-docs.py` passed
with “Markdown links and build-directory layout checked”;
`git diff --cached --check` passed. The staged path list contains only this new
review report. No Cargo, product test, runtime, service,
source merge/reset, existing artifact edit, worker/task/worktree creation,
other-worker contact, desktop, push or task-status mutation occurred.

Root alone integrates, publishes and changes statuses. O06 stays DONE; D05 stays
in_progress. Held Group, canceled accessibility, nonrenewed60s admission,
paused-before-I/O and provider/old-worker-quiescence limits remain intact, as do
receipt-secret, route-specific headers, PAM fallback and permission/review/
receipt/removal/audit/credential protections. Desktop preference remains RiWork
Cua.ai Driver, descriptions/current state first; none was needed.
