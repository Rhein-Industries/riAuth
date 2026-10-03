# Q11 original release-assurance scope: source audit and bounded plan

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original Q11
`420d4ebb-f42d-4052-9c14-9efe6afcc18f`. Reservation
`wave30_Q11_release_assurance_original_scope_source_audit`, 2026-10-03.
Supporting worktree `e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`, existing branch
`roadmap/local-extension-isolation-wave27`, starting HEAD
`3e2fd1a26322e899ec265a3087b33ebd328606ec`.

Fixed published source: `544d1340b80cd3e040dc13142cdcbc1d75fea4cb`.
No signature/provenance bypass was demonstrated by the reviewed bodies. One
small source-backed documentation overstatement is identified below: preparing
three SPDX documents before writing is not atomic publication of all three.
The smallest proposed correction is wording only, subject to root reservation;
no existing document or helper was edited. Actual release assurance still needs
a selected release subject, bundle, independently approved trust root and policy
pins. Internal independent worker review is not an external security assessment.

This is a report-only source/design result, not Q11 completion. Q08's bundle
catalogue and the separately owned Q10 installed gate are referenced rather than
repeated. No runtime lane was acquired or released.

## Exact original row and current-export boundary

The complete original row was read from the project
`planning/current-tasks.json` under
`/Users/dominik/.local/share/riwork/orchestrators/projects/891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Observed file: 244354 bytes, SHA256
`0b38ca3eb8810a4628436eb5083f6329de848f3f4ddaa972c850601ad45c0835`,
mtime `2026-10-03T05:51:05.061711+00:00`, 94 rows.

Original title: **[P1] Q11 — Publish verifiable releases and independent review
evidence**. Exact requested outcome:

> Sign artifacts, publish dependency inventories and provenance, document review scope, and maintain a vulnerability-response process.

Prerequisites: Q07, Q08, Q10. The export records Q07 done and Q08/Q10 todo;
these are exported board states, not artifact verification. The workstream gate
requires evidence appropriate to security, compatibility, speed and recovery;
success in one category does not establish another. The row requires relevant
implementation/tests/docs/released artifacts as applicable, actual checks, gaps
and external prerequisites, and prohibits completion from a report alone.

Q11 is exported `todo`, original assignment
`4a44dab0-5a93-4646-8b49-e73f324c13d6`, creation epoch `1790534568`, update
epoch `1790894719` (2026-10-01 22:45:19 UTC). The export-file mtime is newer than
that row update; no live board API was queried here. The dated leave-todo
scheduling paragraph does not override the user's current scoped authorization.
Neither the original assignment nor status was changed.

## Bodies read, reused and identity-only observations

The full Q08 audit used immutable `6a4066c72ac58225cd444a806032eef8515e646b`.
The following current objects are byte-equal to those already fully read there:

| Reused complete body at fixed 544d | Bytes | SHA256 |
| --- | ---: | --- |
| `.github/workflows/release.yml` | 9864 | `da8607c5ea33a906185141e0caebec18c2282c22676981e0c659fd65eef4369f` |
| `.github/workflows/ci.yml` | 11879 | `fc5245ef15f0ac8d0a460b71ded533ad96ad3dcbd795b31f38b3e84221940b5a` |
| `scripts/package-release.sh` | 4762 | `3060b16559b004e6e213d7ed484f575a7375df3d92d29b39fcf0726f936a2f0a` |
| `scripts/check-release-evidence.py` | 19722 | `bf0c6c2a7c748dd9a7164c71f50cb191feb2a93ea8acf0a751deec5377771532` |
| `docs/roadmap/q11-release-evidence.md` | 26192 | `5a3a0981ebdf7215c497fd748f32934e1a2cc7ba3cef4c98982c2c70f0fe1f39` |
| `docs/release-notes.md` | 29522 | `e670dd60457bca9143f087ab8f0a45aedd419660d039627c764c8045a373df1d` |

The Q11 procedure was additionally reread completely in bounded chunks. Release
subject/attest/publish steps and the CI assurance step were reread for this scope.
The bundle, installed and edition checkers, limitations and editions guide also
equal their Q08 objects; their complete prior body review is reused without
another installed/bundle audit. The unchanged native artifact workflow and four
accepted native/PG/x86 replay receipts retain the Q08 evidence attribution.

New complete-body reads in this audit:

| Source or review | Bytes | SHA256 |
| --- | ---: | --- |
| `SECURITY.md` | 981 | `2556771d58d09a2a4754e7484645b7e948b84286ef0d21cc66169b920a31c4d8` |
| `scripts/check-release-attestation.py` | 20918 | `c239cdabd3f3915ce902ccad8c96b55900bb2f56781c9340ffb8ac75fd4952f3` |
| `scripts/spdx_sbom.py` | 36727 | `ca063ab3d4abb6ec815151bcf762447e96682076a7f72d2dbfa9d7e6d3a1032c` |
| `scripts/generate-third-party-notices.py` | 12055 | `db96e420c69c668f4ed542188f5df80bdfe124d9e5bee58caa9949e8f3637dc8` |
| `tests/test_release_attestation.py` | 21780 | `8ff1abd5365aee6cb143cd4628cca85b899217dacceef1681a9df2aff5f47018` |
| `tests/test_release_evidence.py` | 11152 | `40c1d17165543d4fd3c21378c7f156981b6aa1a076d3436a10ea9d2fac5e9467` |
| `docs/security/threat-model.md` | 22431 | `ca395448b6ae55dbd464bbd839b5ee19ab5e0fb66125e2b11c59c399962ec69a` |
| `docs/roadmap/local-wave30-a09-legacy-metadata-independent-review.md` | 18092 | `db836dd977247695f7d764f0ab61dc26ec33b76b9374f696192bd20e9b4c3322` |

The attestation helper also equals its Q08 body; the SPDX helper's previously
partial graph review was completed by reading all 899 lines here. No module was
imported and no candidate function or fixture ran. Static AST parsing found 17
attestation, 14 release-evidence and 34 SPDX `test_` definitions. These are source
definition counts, not executed test counts; SPDX test bodies were indexed, not
all 1377 lines claimed as reviewed.

`THIRD_PARTY_NOTICES.md` header/inventory examples and whole-file identity were
read, not all 14514 lines of embedded license texts: 752651 bytes, SHA256
`8a92f38f2a648d08aa21f415f9ad34b966ee7df6bbf6ba89ddd926ef8833efa2`.
The invariant catalogue header/coverage/evidence boundary was inspected; an
initial oversized read was truncated, so no complete 1330-line review is claimed.
The full threat model explicitly covers baseline `96e23e2...` inspected on
2026-09-27. Its then-observed gaps are not a fresh assessment of current product
or grounds to reopen completed rows. General coverage inventory metadata was
not a substitute for dependency or external review evidence.

The changed current local-container workflow was identified by bytes only
(4640 bytes, SHA256 `63887b4f57c959ce5256865c2befc7cc568b95e9eb7d10eff0909cf3dc6747b8`).
No full new ARM workflow audit or result for root-owned run `37101183416` is
claimed. This lane neither queried nor waited for it.

## Original assurance matrix

| Original outcome | Source contract and accepted evidence | Remaining assurance boundary |
| --- | --- | --- |
| Sign artifacts | Four pinned GitHub artifact-attestation requests; one provenance digest list plus three edition/client inventory subject lists. Offline verifier binds supplied subject, certificate and run before predicate checks. | No real riAuth release bundle/approved root was supplied or verified here; workflow text and mock verification results are not a signature. |
| Publish dependency inventories | Locked per-edition/per-target SPDX producer and packager byte bindings; checked-in versioned normal-dependency notices; dated local unsigned produce/verify recorded in the Q11 procedure. | Published, authenticated inventories for the selected release remain unbound. Source notices are not an SPDX release document; Debian packages/dev/build-only scope is explicitly outside the normal Rust inventory. |
| Publish provenance | v4 producer records source/repository/run/attempt/native target/features/image IDs/maintenance hashes/toolchain/locks; attest request covers its bytes. | Unsigned build metadata does not authenticate itself or prove native execution. Signed certificate identity and native receipts are separate evidence. |
| Document review scope | Dated Q01 threat/invariant evidence labels; current scoped internal source reviews with exact pins, limits and failed-source/runtime receipts. | No recruited external assessor, completed external assessment scope/report/findings or remediation/retest record is evidenced by these internal reviews. |
| Maintain vulnerability response | SECURITY defines supported v0.1.x consideration, conditional private GitHub reporting, public contact-request fallback without details, acknowledgement/fix/disclosure coordination, and no promised deadline. | The policy is present; private-reporting availability and an actual intake/response incident were not checked. No fake incident, contact, SLA or external review is needed or invented to demonstrate the written process. |

### Signing subject and identity checks

The package job alone receives `id-token: write`, `contents: read` and
`attestations: write`. The draft-publish job has `contents: write` without signing
permissions. The four action calls are pinned to
`actions/attest@1e69f48acb82d1966a394da916b4c1698aa569d6` (the procedure's
dated v4.2.2 identity observation). Its own upstream signature is not a signature
on riAuth. Public Good/OIDC policy is documented; no new signing service/key is
selected here.

The all-subject digest list hashes the exact packaged directory after its gates.
The three SPDX attestations use only each document's named archive/image subjects.
Subject lists live outside the asset directory. This is a source assertion about
the requested attestations, not an observed action execution or saved bundle.
The workflow ultimately creates a draft; publication requires the separately
documented maintainer review. Package attestations are not automatically new
signature filenames in Q08's exact asset set.

`verify_attestation` first validates the tag, 40-hex source, positive run/attempt,
predicate selection and regular non-symlink artifact/bundle/root inputs. Bundle
schema/signature-presence and matching subject name/SHA256 are prechecks only.
The production CLI then invokes `gh attestation verify` with the artifact and:

- Repository `Rhein-Industries/riAuth` and exact certificate identity
  `https://github.com/Rhein-Industries/riAuth/.github/workflows/release.yml@refs/tags/TAG`.
- OIDC issuer `https://token.actions.githubusercontent.com`, exact source ref,
  source digest and signer digest, and denial of self-hosted runners. The signer
  digest is the selected riAuth workflow/source commit, not the action pin.
- Supplied local bundle/custom trusted root, selected predicate and JSON output.

Only a zero crypto-verifier exit proceeds to certificate fields and witnessed
timestamps. Every returned result must match SAN, issuer, signer/source digests,
source repository URI/ref, hosted-runner environment and certificate
`runInvocationURI` with the exact run **and attempt**. Predicate content cannot
supply these identity fields. A provenance predicate must then match its
invocation ID; an SPDX 2.3 predicate must bind the subject filename/hash.

The injected `runner` in Python unit fixtures returns synthetic verification
results; the public CLI does not select it. The fixture's nonempty fake signature
or mock certificate/timestamp is not cryptographic proof. An actual successful
`verify` would mean that supplied subject met the checks, while its
`signed_artifact=false` deliberately avoids claiming this command produced a
signature. The dated gh 2.96.0 flag/schema inspection is not verification of a
currently installed executable. Native OS/architecture and locked feature
provenance are producer-controlled metadata/log evidence, not extra certificate
fields or proof derived from a capability count.

### Inventories and review scope

The SPDX producer hashes exact named regular inputs, binds selected target,
features and lock bytes to the resolved normal-dependency graph, records registry
checksums from the lock without fetching crates, and marks path packages/unknown
license conclusions honestly. It does not re-evaluate caller-supplied cfg graphs,
audit dependencies for vulnerabilities or inventory every Debian component.
Reproducible canonical `verify` is byte/graph evidence, not signer authentication.

The notices generator uses the locked Linux normal-dependency union, registry
versioned source links, reviewed license overrides and exact embedded texts. It
has its own Cargo calls; it was not invoked here. Its SPDX-expression column is
not a package SPDX document. The present header records the current client lock
`2998555d...`; the dated local unsigned Q11 generation records the older
`6f546c96...` client lock. Those inventories are not interchangeable.

The completely read internal legacy-metadata review is an example of useful
bounded independent engineering evidence: exact 59f helper/profile, static full
body/reversal, disclosed primary-source fetch failures and retained failed
cohort, with no validator/runtime/ARM/release credit in that slice. It is neither
a recruited external review nor a current full-product assessment. External
assurance needs a named assessor and independence, exact reviewed source/artifact
scope, methods/exclusions/dates, findings, and actual fix/retest disposition.
No party, finding, certification, engagement or universal new campaign is invented.

## Actual execution attribution and public observation

The unchanged Q11 procedure records a 2026-09-29 local unsigned inventory run
over retained f3aba63 files. It attributes `produce --local-unsigned` and canonical
`verify --local-unsigned`, not a release job or signing command: Essentials
`c64455270ca2a928ff1dde73a7586152c667bcf8c2626c4652aa0b9fd1d8b289`
(301 packages), Platform
`9e1520c898251ba0bf116f1ddc3994c9f7e6ce8ff21fb3888ab5967b0ee184a9`
(353), and client
`23a62085c911d5e84d532a2e17aeabe04a562f25d234d18e4c7d0dfaa6700e35`
(121). Those files were not opened/rehashed here. Its historical Public CI run
`36342719277` at `96e23e2...` uploaded zero artifacts. Neither observation proves
signing or publication. Its September integration strengthened certificate run/
attempt checks without rerunning those fixtures during that integration.

The fixed receipt `docs/roadmap/evidence/wave30-ci-7869-terminal-root-review.json`
was parsed in full for metadata and selected assurance steps; its giant per-test
summary section was not completely reread after a truncated initial output.
Identity: 49599 bytes, SHA256
`2780f1fe92cd679a89d5c5b5f295776464be724f2cf78562dcc433876f04be84`.
It attributes run `37077762237`, source `7869b964...`, check job
`111074660465`, and successful “Check documentation, notices, and delivery
scripts” at 2026-10-03 00:12:54 UTC. The matching CI source invokes notices check,
release-evidence audit, attestation **static**, and their/SPDX unit tests in that
step. The release/CI/verifier/SPDX/two reviewed test objects are byte-equal at
7869 and fixed 544d. This credits the accepted source/mock-check step, not real
bundle verification. Audit job `111074660373` also records dependency-advisory
step success; no current advisory database scan or new vulnerability conclusion
is inferred. No full raw CI log was read or test count remeasured here.

Q08's accepted native x86/ARM, PG and encrypted-container receipts still establish
their named LOCAL artifact/lifecycle results at b619/9a819. Their bodies are
unchanged from the complete Q08 reads. They carry no official-release/signature
credit. Older failed cohorts, UID failures, unknown native causes and S02/D01
receipts remain unchanged. This audit does not revisit their runtime gates.

Root's public-only retained receipt
`planning/evidence/wave30-q08-draft-release-metadata-root-review.json` was read
completely: 3194 bytes, SHA256
`037bc002576f0fb889fc125c7fb8e1190b73f8f437f8f2c97b748228cecb45c4`.
It observes 2026-10-03 06:09:08 UTC: `v0.1.1` is a draft, `publishedAt=null`,
`targetCommitish=main`, with the four legacy generic x86 files. Mutable main is
not a source pin; asset API digests are metadata, not locally rehashed bytes or
signature proof. No asset/bundle was downloaded. The receipt does not count
attestations. The September `live-public` classifier is a separate dated contract;
it was not invoked to requery or replace this observation. Root already has the
asynchronous source/version/provenance question; no repeat request was sent.

## One smallest local correction proposed, not implemented

At fixed 544d, `docs/roadmap/q11-release-evidence.md:214` says that package-linux
checks named files and “then writes all three or none.”
`scripts/spdx_sbom.py:728` prepares all three documents, then calls `write_bytes`
in sequence at lines 760–761. Each file uses its own temporary/replace. If the
second replace fails, the first publication is retained; there is no set-level
rollback. This is a source-derived documentation overstatement, not a reproduced
filesystem failure or evidence that a partial release can successfully publish.
The packager is fail-fast, and later gates still require the complete byte-bound
asset set. No group-atomic metadata guarantee should be attributed to this code.

Smallest prospective ownership: **only that paragraph** in the existing Q11
procedure, after root reservation. Preserve its subsequent command/fixture/
reproducibility/refusal/disclaimer text. Suggested replacement for the first
sentence and the beginning of the next:

> `package-linux` is the binding the packager uses. It validates the named inputs and prepares all three documents before publishing each file separately. Preparation failures write no documents; an I/O failure during publication can leave a partial set and must stop the release job. `--server-manifest` and `--client-manifest` run the locked offline metadata commands.

No helper, workflow, writer, schema, lint policy or test change is needed to make
that description accurate. No transaction strengthening or fault-injection
campaign is proposed. A later wording change should receive only the existing
focused source-contract/docs checks under its separate authorization. None ran
against a changed procedure in this reservation.

## Future single verifier slice and missing exact inputs

After root selects the release and supplies proof, the smallest actual assurance
slice is **one** existing offline SLSA provenance verification of **one** canonical
subject. It does not duplicate Q08 completeness or Q10 installation:

```sh
python3 scripts/check-release-attestation.py verify \
  --artifact "$EXACT_SUBJECT" \
  --bundle "$EXACT_PROVENANCE_BUNDLE" \
  --trusted-root "$APPROVED_TRUSTED_ROOT" \
  --tag "$RELEASE_TAG" --source "$RELEASE_SOURCE" \
  --run-id "$RELEASE_RUN_ID" --run-attempt "$RELEASE_RUN_ATTEMPT" \
  --predicate https://slsa.dev/provenance/v1
```

All variables are required **unbound inputs**, not proposed new versions/keys:
exact filename/size/SHA256 and retained artifact transport identity; bundle
size/SHA256 and producer/retrieval provenance; approved root bytes/hash/origin and
rotation policy; repository/workflow/tag/source/run/attempt binding; compatible
authenticated verifier/tool identity and output schema. The intended repository/
workflow/OIDC/hosted-runner policy is the fixed source policy above. A caller-
supplied nonempty root is not automatically an independently approved root.
No root fetch, signing, release creation or registry operation belongs to this
local slice. The currently observed draft's mutable target does not fill the
source/run slots. No input is manufactured from a local binary or mock fixture.

Prospective finite controls, for root's later concrete review: pin the Python
and gh executables by actual bytes/compatible provenance without a version probe
in this phase; one Python command and its one owned gh child, outer 125 seconds
and child 120 seconds, joined I/O/reaped group and final post-save clock before
acceptance, 2-second disk monitoring, fresh start at least 10 GiB, stop 9 GiB,
floor 8 GiB. A 1 GiB additional disk allowance is a conservative planning estimate
conditional on bounded real subject/bundle/root/output sizes, not measured peak
or a memory/kernel-I/O quota. No build or copied archive/image is required.

The helper currently uses `capture_output` for gh without an internal timeout or
output cap. A reviewed outer owned-group supervisor can bound elapsed time and
its own output, but does not thereby cap that inner in-memory pipe. Actual input
and verifier-output size expectations must be credible before release; if not,
stop at design and report the specific bound rather than inventing safety.
No supervisor was written, imported or run here. Supplied bundle/root must make
the intended offline gh path sufficient; no unapproved retrieval fallback.

The eventual wrapper should retain exclusive 0600 bounded raw captures (proposed
16 MiB), hashes, numeric Python/gh outcomes, timing, disk and exact owned-group
cleanup before grading; public output uses fixed failure classes, not raw bundle
contents or arbitrary exception text. First unexpected schema, identity/digest/
predicate/run/attempt/witness, crypto, deadline, capacity, capture or cleanup
failure stops with no retry, root/key/pin relaxation or source correction.
Terminate/wait/reap only the captured owned group, prove absence, retain logs and
leave supplied inputs unchanged. No provider, product, native image, listener,
Cargo target or cache prune is involved.

A pass would authenticate the supplied provenance subject under those pins. It
does not sign anything, publish a release, verify every subject/inventory, prove
native compatibility/recovery, or supply an external assessment. Those remaining
original outcomes stay explicitly separate; no universal security campaign or
additional optional gate is added.

## Actual checks, failures, preservation and handoff

Actual checks were read-only Git/hash comparisons, JSON metadata parsing, static
AST parsing of source text and test-definition indexing. No verifier/helper/module
import, pure function/case execution, Cargo/compiler/native/version/CLI-product,
network/query/download/dispatch/signing/provider/socket/browser/Driver or service
ran. A09's runtime lane was not acquired, released or inspected.

One metadata utility exited 1 because a `git show` lookup of this worktree's Q08
report in fixed 544d exited 128: that path is absent at that pin. Its correct
cross-reference is its own immutable `3e2fd1a26322e899ec265a3087b33ebd328606ec`
object. The corrected preservation check exited 0; no root alignment, import or
publication claim was made from it. Oversized invariant/CI output reads were
truncated and are scoped explicitly above, not silently treated as full reads.

Only this new report may change. [Q08 plan](local-wave30-q08-exact-shipped-bundle-plan.md)
remains 24056 bytes/SHA256
`7eb7d9bc655580de20bb6a1e88fe59734a96a4ec3dbb6c0f5e47bb7004c3ac1d`;
the [S02 report](local-wave30-s02-index-pagination-plan.md) remains 283833 bytes/
SHA256 `3f8929fc6f5bbdd7f45c4af80bf4319ec8715fb07f7ae0e874e1080d753f9921`.
All existing source/tests/helpers/workflows/config/manifests/reports and old
S02/D01 evidence are preserved. Receipt-secret, route-header, PAM, Group,
authorization/revocation/audit and 60-second protocol contracts are untouched.
No new task, worker, worktree, managed shell, source alignment, main/push or status
mutation occurred, and no other worker was contacted.

Static report handoff checks: UTF-8/fences/relative links/newline/whitespace/conflict
markers, sole-new-file staged scope, unchanged existing files and protected
report hashes. Source readiness is not runtime approval. Root alone chooses any
later paragraph/verifier reservation, integration, publication and original Q11
disposition; this original row remains unchanged.

## Approved SPDX publication wording: source-only implementation

2026-10-03, project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, reservation
`wave30_Q11_spdx_publication_wording`. Root explicitly approved only the first
sentences of the existing Q11 procedure's package-linux paragraph and this
appendix. Original primary/status remain unchanged; no source alignment or
full-file import occurred.

Guide-only commit: `96fb8d181f61fecc05881466ef8961951e1a3850`, parent
`4598a9e337a86bb8eb9d682dbf35ff089e959e63`. Exactly one hunk, four added lines
and three removed lines in [the guide](q11-release-evidence.md). It now states
that named inputs are validated and all three documents prepared before separate
file publication; preparation failures write none, while publication I/O can
leave a partial set and must stop the release job. This is a wording correction,
not new transaction behavior or fault-injection evidence.

Before editing, the entire own guide equaled fixed published
`544d1340b80cd3e040dc13142cdcbc1d75fea4cb:docs/roadmap/q11-release-evidence.md`:
26192 bytes, SHA256
`5a3a0981ebdf7215c497fd748f32934e1a2cc7ba3cef4c98982c2c70f0fe1f39`.
The approved 158-byte old span occurs once; its 283-byte replacement matches
the proposal in the initial 4598 report exactly apart from line wrapping.
Result: 26317 bytes, SHA256
`c36719cbbdb396e7b662b9e8e51c3a156cb085ce5b24d298842fa3640a029394`.

Whole-byte inverse proof completed with exit 0: splitting the fixed original on
that unique old span yields prefix and suffix; the result equals that prefix,
the approved replacement, and that exact suffix. The suffix starts at
`--server-manifest`, so every byte from there onward remains identical. Replacing
the unique new span with the old span reconstructs the entire fixed 26192-byte
file and SHA256 above. Forward reconstruction also equals every resulting byte.
All other guide paragraphs, command text, fixtures, reproducibility/refusal
claims and release/attestation disclaimers are unchanged.

Static checks actually completed with exit 0: whole guide/source byte comparison
and inverse; approved sentence comparison; strict UTF-8/final newline and no
trailing whitespace; ten paired code-fence markers and twelve existing local
Markdown link targets; `git diff --check` and `git diff --cached --check`; sole
guide staged scope, no unrelated diff; and guide-only commit/clean-state checks.
No helper was imported and no source checker, fixture, fault injection, native
binary, compiler, Cargo, tool setup/version, signing/verification, artifact,
dispatch, network/provider/browser or service ran. No validation/Cargo lane was
taken or released; root-owned ARM run `37101183416` was neither queried nor
affected. There was no new check failure in this wording phase; the initial
audit's Git lookup failure and truncated-read limits remain recorded above.

The report prefix from 4598 remains all 23803 bytes, SHA256
`bb094cee5363889086217e95383d065d152b25099d0966b933d18f43925f7e6e`.
This appendix is committed separately from the guide. Only the authorized guide
and report differ from 4598; helpers/workflows/tests/config/manifests and all
other docs/evidence retain their prior bytes. Q08/S02 report hashes and every
prior S02/D01 failed/actual receipt remain intact. No task/assignment/status,
main/push, other worker contact, new resource, receipt/header/PAM/Group or
60-second protocol change occurred.

The dated initial proposal is deliberately retained above; this appendix records
its later approved wording implementation only. Exact release source/version,
subject/bundle/approved trust root, real verification/publication and external
assessment prerequisites are still unbound. Root reviews/integrates/publishes
and owns original Q11 disposition. This correction grants no runtime permission
and does not establish an atomic three-file publication guarantee.
