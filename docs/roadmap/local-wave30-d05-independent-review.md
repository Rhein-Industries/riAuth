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

---

## Current-evidence addendum and proposed refresh — 2026-10-02

This is a later, report-only phase of original D05 support in the same project,
task and worktree. The entire preceding `09398ada2f8e01fe08ef50c4bbae4830ba906bac`
report remains a **dated prior phase**, including its then-current gate language,
limits and checker failure. Its 17493 bytes, blob
`bafe944aff8778de4eb39b7c6ac430eb8e9292d2`, SHA-256
`2fbd2550d375f1d7a5ebfebbc00673001db532abf78d27002f9c0fc3ac2c2430`,
are preserved as the exact byte prefix. This addendum supersedes only the earlier
absence claims identified below. No existing D05 JSON/Markdown artifact was edited.

The original RiWork D05 row was reread with explicit project ID. It requires
evaluation of the ten named categories, documentation of completed user/operator
tasks, and that a new user and new operator can independently complete the
documented workflows. Its proposed prerequisite list is not a new rule that every
category target, tenant, device or release must pass before an honest evaluation
can finish. The current row remains `in_progress`; this worker changes no status.
The fresh `wave30_D05_current_workflow_addendum` ownership reservation permits
only an append to this report and holds all runtime.

### Finding and one concrete local gap

**The operator-command, password-browser and actual local recovery-RP absence
claims are now resolved within their exact scopes. No missing category assessment
or two-artifact integrity error was found. One concrete remaining local documented
workflow gap is the Essentials section 3 browser application journey.**

The printed guide says to start the application at `http://localhost:3000/callback`
and use its sign-in action, then complete riAuth browser sign-in/consent and return
to that callback. Operator evidence stops after registering confidential
`local-demo` and checking discovery/identity. The password browser fixture had no
application and an empty catalogue. R05 executed a genuine scripted **public**
OIDC RP before/after restore, with service authentication as its prerequisite;
it did not execute that guide's confidential-client browser task. Combining their
counts cannot establish one independently completed browser application workflow.

This is the prioritized local evidence gap for root's original-gate interpretation,
not a demand to pass all category targets or an assertion that every other guide
task has run. The other invitation, factor, ordinary-nonadmin, integration and
deployment limits remain individually labeled. Physical passkeys, optional mail,
LDAP/SCIM tenants, installation/release artifacts, an inexperienced-human study,
universal browser administration and HA are not added as universal D05 gates.

Recommendation: accept the three bounded current evidence additions and adjudicate
that one printed application-flow gap with D01's owner. Do not infer full original
workflow completion from the existing records or rewrite all unmet inputs as
failures. If root reserves a next checkpoint, its exact scope should be the printed
section 3 application sign-in/consent/callback and fresh protected application
access with matching artifacts and explicit disposable application setup. This
report authorizes no such setup or runtime and makes no product-correction claim.

### Fixed publication, accepted staging and exact records

Fixed published input is **`88790deb62d32c84fa17dceb12cd93a727224e94`**.
Operator/R05 author commits need not be its ancestors: their report bytes are
present there unchanged. Root D01 browser staging/review is deliberately separate
and later; its report is absent at that fixed published pin.

| Proposed stable ID | Class; actual current-outcome delta | Exact authored/accepted provenance |
| --- | --- | --- |
| `E-D01-OPERATOR` | `executed local fixture`; 27 printed CLI/probe commands exited 0 and two original-config servers exited 0, 607.862s. Essentials §§2/3/5/6–8 init, distinct sessions, doctor, confidential client/discovery, online backup, closed offline restore, original restart, groups, claims simulation and correlated audit/CSV are executed. Restored store remained pending and was never served/attested. | Author `9cde81dd93ff171fa54194b2ed514142451484d9`; identical report at fixed published88790deb. c01-equivalent production, execution HEAD `950fc6ecb14fcbafd4587f95987b9231984994c2`; printed guide blob `be68580333ad5f2238ac1966d164bf0fbf80aca1`. Root's published remaining-gates review read all 34 redacted records; that is root verification, not a new read of the private JSONL here. |
| `E-D01-BROWSER` | `executed local fixture`; actual fresh password administrator browser sign-in with empty code, signed-in identity/empty catalogue, security-panel open/close, logout, password relogin and final logout. RiWork Driver MCP0.30.4, isolated profile/exact returned binding, explicit DOM clicks followed by fresh snapshots. No app, passkey, invitation or ordinary-nonadmin browser journey is credited. | Author `6837b745576552b0917b3bd1f1424515621a0a85`; identical blob at accepted root staging `937d0da38a52ff1e2e4a510b61e79f22087a94f1`; root review `1711b531f720b57c22bed8e59a9657faaa6370bc`. Guide pin `6b4db4f0317e4427c187f55e063989ccba124217`; production c01. Accepted staging is not publication88790deb. |
| `E-R05-RP` | `executed local fixture`; one invocation exited 0, 19 unique passed checks, including actual pre-backup/post-restore callback, S256 exchange, native RS256/JWKS and claims verification, userinfo, new RP cookie and protected access403/200; gated restore, explicit synthetic reconciliation and old-session denial. This resolves the actual local representative-RP gap, not deployment escrow or browser usability. | Author `0cc3515eb54a3bcbef83e7fbe03a00ddbcea6cbc`; original disposition `dcd6882c510b6380e828882e1ebadf558fe86f8b` and exact tracked JSON at fixed published88790deb. Runtime HEAD `b5cea614c4f46d82aff2380c052bd2dffc760f9e`, Platform binary, c01-equivalent `src` tree `3adc2b59c3547d22bff202daccfd8ad97f1e78ab`. R05 remains accepted DONE; no reopening. |

Stable IDs above do not collide with the existing seventeen. The operator's 27
commands are not 27 tests, its two server exits are not additional command passes,
and R05's 19 checks are not added to its older sixteen-check runs. Browser
transitions are observations, not a new CI suite count.

I authored/performed `E-D01-BROWSER` and am the reviewer writing this addendum.
Rereading/hashing my own report is **not independent verification of my actions**.
The separate root review accepts the recorded observations and recomputed hashes;
it explicitly did not replay GUI actions. Operator/R05 records are other authored
evidence plus root-reviewed records, not runs performed or replayed by me here.

The D01 server and maintenance hashes are respectively
`7abf745c10691a012a1918c83089168d0e0d88b42764dbf8ed538b90c828b606` and
`86490c7f71de6b7ae9d4dabdf9060a8757100f3aedbdaa670284d9e1206a2a95`;
operator base client is `bfbbb322f1a66d0ac9998beb9fb5838097cea0442cea3fd3a1d057fcb3f600cf`.
R05 binary is `0f137475af5a8040d96a794b1ad331e7430be4467046b81b1312fb974b7e8a6a`.
These runtime artifact hashes are credited from the exact accepted records,
including my preceding browser phase's actual rehash; no binary was opened or
rehashed anew in this read-only phase. They are not current-main/released binaries.

The proposed evidence references should use these exact Git identities:

| Pin/path | Git blob | Content SHA-256 |
| --- | --- | --- |
| published88790deb, `docs/roadmap/local-wave30-d01-walkthrough-report.md` | `92f85d5f4c242e221fded43635754f8f961ac294` | `9d8dc3d135013efd88e91759d416effb7119eba0458524aec89494fc70481236` |
| worker6837b745 / staging937d0da, `docs/roadmap/local-wave30-d01-user-browser-review.md` | `0beaceecb32b3fda2b6f06ff5bac2c26d6c3aee5` | `68f4b61ed388d9a6ec980bc93a28eb6964c0ca75d36ac64ee4bd9108d837ffd4` |
| review1711b53, `docs/roadmap/local-wave30-d01-root-user-review.md` | `100d3b5924c92a387fed1416366fa5a265f44965` | `af64815f3b6370a3e11f663dc2809abc48a1ed4e1a207419d38b0d8ce0f506e7` |
| published88790deb, `docs/roadmap/local-wave30-r05-drill-plan.md` | `e07cba637b9108aff0c146b9d58e7135feb51a55` | `8d86e5ac504c650f8f2880f36183ccf200859e70f22fa9ce41b4dd38a4f3a313` |
| published88790deb, `docs/roadmap/local-wave30-r05-root-disposition.md` | `01b7ffe1c012663c5eefdf2508d869165289a821` | `098cfb40cb7f27453455f72a58c013449e9a4cb5dda9609e91ee57e6ef3070a5` |
| published88790deb, `docs/roadmap/evidence/r05-local-rp-2026-10-02.json` | `215c719d05f581fc95278e9013aa6e85b54aee5e` | `5498bbf1f089224947ef3f0cbc7e163c4aa35683eb8d0256943100f49012df41` |
| published88790deb, `docs/essentials-guide.md` | `e48371d3a34a7b6441cbcfce700daf954aa5f2ae` | `9df3c2861984751d48b28d284cfb47c07801fd3934072e2ccb829edeb35579a0` |
| published88790deb, `docs/roadmap/recovery-drill-r05.md` | `5f6b5582faed4368cfc38f3495f7b987a387ef14` | `bc2000c3059e72bd11ec9180a72a512fc68e64aa125f3572dd0616b93d68309b` |
| runtimeb5cea614, `scripts/recovery-drill.py` | `ada2dfa93ccac1ec132cc15fb4a2b5a042fad878` | `7de1728211621a9bfb2e32d6712fac3960d91b4c31abaf0bab02d68dc85c2437` |
| runtimeb5cea614, `scripts/recovery-drill-oidc.py` | `3be747d03146f1bcaa3ec012ee8d173b61fa737d` | `f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d` |

For the operator record, retain the exact 27 redacted argv strings from its
actual-outcome table, including its disclosed `--password-stdin` and
`--non-interactive` substitutions; do not relabel it interactive browser setup.
For the browser record, list Driver actions/snapshot transitions rather than
inventing an HTTP/CLI authentication command. For R05 retain exactly
`python3 scripts/recovery-drill.py --binary "$PWD/.target-wave27/debug/riauth" --evidence "$PWD/.target-wave27/r05-wave30-local-rp.json"`
as its historical runtime command, with the reviewed script/helper pins above.
All three new records should say `runtime_run_by_this_slice: false` and carry
verification labels limited to authored/root records and tracked result JSON.
Private JSONL/log/observer hashes remain reported provenance, not newly checked
external raw files or fields to silently copy into a passing verifier claim.

### Exact minimal two-artifact delta proposed for owner review

This is a proposal only; only the reserved report is written now.

1. Preserve every existing field/value in the dated c01 assessment, including
   `prior_snapshots[0].snapshot`, complete prior Markdown, old ten-slice prefix,
   original seventeen evidence records, all 27 existing slices, dated board,
   source/CI observations, failure history and preserved-boundary text. At fixed
   published88790deb both artifact bytes still equal reviewed eb1412aa; their
   prior-phase blob/SHA pins above remain unchanged.
2. Append exactly the three stable records above to `evidence_records` and
   three corresponding `newer_slices`: 17→20 IDs, 27→30 slices. Use existing
   reference/class/summary/verification/recorded-command fields. Operator/R05
   `accepted_published_pin` is fixed88790deb. Browser's published pin is null;
   add explicit `accepted_staging_pin`937d0da and `root_review_pin`1711b53,
   plus authored/runtime/guide provenance, instead of falsely calling it published.
3. Add one `current_evidence_addendum` object with read date2026-10-02,
   fixed publication88790deb, the separate browser staging/review pins,
   `new_evidence_ids`, the ten-row category delta below, resolved bounded gaps,
   and `remaining_local_workflow: Essentials section3 browser application sign-in/consent/callback`.
   This object labels the previous current language as its dated prior assessment.
   Do not change top-level task status, `d05_passed`, category target booleans or
   gate outcome without root's exact classification decision. If adopted as
   proposed, record the independent workflow gate as partially evidenced with
   that local gap; a ten-category evaluation can remain evaluated-with-gaps.
4. Append one matching dated current-evidence section to the Markdown twin:
   exact three summaries/classes/IDs/references/commands/counts/failures, this
   ten-category delta, and the single current local gap. Keep all earlier text
   byte-for-byte. Match JSON/Markdown current fields; historical sections remain
   dated rather than silently rewriting O06/R05 or c01 observations.

| Category | Proposed current IDs and bounded change | Remaining claim-specific limit |
| --- | --- | --- |
| Usability | `E-D01-BROWSER`: replace blanket absence of actual password browser completion with accepted sign-in/navigation/logout/relogin evidence. | Application browser journey is the one local gap above; ordinary-nonadmin, factor/invitation/device/accessibility claims are not supplied. |
| Workflows | `E-D01-OPERATOR`: document the executed built-in Essentials setup/management sequence. | It is not a configured Platform graph/operator browser-authoring journey; do not reopen accepted W rows or invent arbitrary-graph gates. |
| Administration | `E-D01-OPERATOR`: client/group/claim/audit commands and exact authority/revision/idempotency/secret-file behavior are actual tasks. Current context acknowledges accepted O06 DONE. | No People/Security browser ceremony or universal browser resource coverage is inferred; retain accepted M03/receipt/header/PAM decisions. |
| Interoperability | `E-R05-RP`: actual local synthetic OIDC application callback/exchange/verification/protected-access evidence. | A public local RP is not the guide's confidential browser task, a named third-party tenant or family certification. |
| Footprint | No new ID or measurement assertion. Artifact hashes identify the executed binaries only. | Existing native/RSS/release-size/install limits remain. |
| Performance | No new ID or benchmark assertion. | 607.862s/186.997s are bounded checkpoint intervals; R05's 19 checks are not load/latency/throughput evidence. |
| Availability | `E-R05-RP`: actual stopped-source outage/refusal and restored local service access. Current context acknowledges O06 DONE. | No deployed HA/failback/fleet health or old-worker quiescence inference. |
| Recovery | `E-D01-OPERATOR`, `E-R05-RP`: online backup/closed offline restore plus separate completed synthetic redb recovery and fresh real local application access. Current R05 original acceptance is DONE. | Do not merge the independent labs or relabel the pending operator restore/older physical/PITR observations as completed deployment reconciliation. |
| Migration | No new ID or migration-completion assertion. | R05's stable subject within one recovery lineage is not Authentik export/cutover/G05 or older-binary route rollback. |
| Security | All three IDs: scoped secret-file handling, actual password browser session transitions and actual restore/refusal/signature/protected-access controls. | Keep source/native fixture, software/device, release/independent review and deployment claims distinct. |

### What was actually checked in this phase

Fresh identity inventory: fourteen fixed `(pin,path)` references, thirteen
distinct blobs, all actual blob types and SHA-256 values checked. This includes
the two unchanged D05 artifacts and prior report, worker/staging duplicate
browser identity, exact current records/guide and two runtime script identities.
Publication-presence comparisons additionally confirm operator/R05 author bodies
equal their fixed-main bodies. The c01 and R05 runtime `src` tree hashes agree;
that whole-tree identity is not a complete source-body review or new execution.
The tracked R05 JSON was parsed: 19 distinct IDs, all results passed, actual
application observations and limits read. No binary/log/fixture was executed.

Body reads: the prior D05 report, own browser report, D01 root user review,
R05 root disposition and published remaining-gates root review were read in full.
The entire operator128-line and R05 actual148-line appendices were read, not
their complete earlier report bodies. All ten category target/scope/unmet/limit
records and the D05 Markdown gate/table were reread; guide intro/application/
passkey-entry and recovery-guide sections received selected inspection. Script
identities were hashed without semantic body review. These counts are separate
from the prior phase's 77 hashes/twelve review bodies and are not added to them.

A nonmutating identity helper initially guessed `scripts/recovery-oidc.py` and
reported that path absent. Pinned script-tree inspection found the actual
`scripts/recovery-drill-oidc.py`; its blob/hash matches the authored evidence.
This reviewer lookup error exposed no artifact/product failure and changed no
source. RiWork's unsupported `task --help` query also exited2; the subsequent
explicit-project JSON task read succeeded. Existing operator handoff exit2
(terminal input/copy mode), earlier build/checker/harness failures, skips and
historical recovery blind spots remain in their records and this preserved phase.

Only this report is appended. No Cargo/test/runtime/browser/desktop/build,
merge/reset, product/guide/evidence-artifact edit, worker/task/worktree creation,
other-worker contact, main/push or status mutation occurred. Runtime remains
closed. Root alone reviews/classifies/integrates/publishes/statuses. All accepted
credential, permission/review/receipt/removal/audit, header, PAM, held Group,
nonrenewed60s/paused-I/O and provider preferences remain intact. RiWork Cua.ai
Driver remains the sole allowed desktop provider; no driver call was needed here.

Actual addendum checks passed: immutable prefix17493 bytes/SHA, one reserved
changed path, unchanged existing D05 artifact bytes, operator27 zero-exit table
rows, tracked R05 nineteen unique passing checks, proposed noncolliding ID/count
arithmetic and balanced fences. `python3 scripts/check-docs.py` exited0 with
“Markdown links and build-directory layout checked”; Git whitespace passed.
These are documentation/identity checks, not fresh product tests or gate changes.
