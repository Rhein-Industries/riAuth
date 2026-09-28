# Q07 bounded dependency/parser assurance

Local review handoff dated 2026-09-27 against assigned base
`96e23e2dedf84db4e39091a004cbd6591acbec97`, branch
`roadmap/q07-dependency-fuzz`. Assignment, backlog and A01/A02/Q01 prerequisite
snapshots were consulted. A02 remains a draft. The authorized repair follow-up
changes only malformed intermediate SCIM member handling from a panic to a client
error; supported parsing, product boundaries, identity, authorization, revocation
and credential semantics remain unchanged. Checks exercise that base plus this
uncommitted parser/test/hook diff.
Whole-Q07 acceptance belongs to the orchestrator.

## Demonstrated baseline finding and local repair: Q07-SCIM-01

An authenticated operator that owns a SCIM group can submit this PATCH:

```json
{"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"replace","path":"members","value":0},{"op":"add","path":"members","value":[]}]}
```

In the assigned baseline, the first operation makes the temporary `members` value
a number. The second reaches `data[path].as_array_mut().unwrap()` in
[patch_resource](../src/scim.rs), before final resource validation, and panics.
The [161-byte retained input](../fuzz/corpus/parsers/scim-members-shape-change)
reproduced this through the pure fuzz adapter. The
[regression](../tests/parser_assurance.rs)
`scim_member_shape_change_must_reject_without_panicking` also reproduced it in
`Core::scim_write` and an in-process HTTP `PATCH /scim/v2/Groups/{id}` with a valid
scoped credential and current If-Match revision. HTTP returned **500**, rather
than the expected client rejection; the blocking worker panicked. Initial failure
logs remain in `target/riwork/q07/final-smoke.log` and `scim-shape-reproducer.log`.

The explicit `assignment-Q07-repair.txt` authorization permits the minimal
production correction. `patch_resource` now converts a failed `as_array_mut()`
into `Error::bad("members must be an array")`. Absent members still initialize as
an empty array; valid array additions and duplicate handling retain their behavior.
The seed and both retained gates stay enabled. The direct call now returns a
**400 client error** without panic, and the HTTP regression returns **400**.

Full redb snapshots before/after direct and HTTP rejection remain equal, including
identity, groups, revision, credentials, audit and receipts. Administrator access
and the owner's group read still succeed. Two adjacent unit regressions cover
number/null/boolean/string/object intermediate members, absent initialization,
existing-member append, replace/add and remove/add sequences, and duplicate member
handling. Q07-SCIM-01 is locally repaired and regression-verified; broader Q07
acceptance remains open. No state corruption, authentication bypass or whole-process
termination was demonstrated in the original finding. PostgreSQL and released
binaries were not tested.

## Added coverage and limits

The existing `parsers` target now also exercises OAuth token/authorization form
decoding, SCIM query/filter/resource/email/PATCH parsing, SAML Redirect/POST form
fields, and complete pinned JWKS validation. Hooks are behind `fuzzing`, call
pure production helpers, and open no store or network connection.

The initial slice added **15 seeds** (22 total at that point): encoded/empty duplicate OAuth fields and max-age
overflow; invalid/duplicate SAML fields; SCIM case collisions, filtered-member
syntax, partial disable, valid member operations, escaped filters and the crash;
duplicate JWK fields/kids, padded key encoding, and deeply nested JSON.

Campaign limits: 65,536 input bytes; 32,768 bytes for new structured/form
adapters; at most 128 decoded form pairs, 32 JSON levels and 2,048 JSON values
including containers, checked before SCIM dispatch. JSON deserialization retains
serde_json's default recursion limit. These campaign bounds do not change
production acceptance. Existing XML byte/depth/node limits and SCIM's 100-operation
and 1,024-byte filter limits remain in effect. Corpus replay caps the corpus at
64 files, checks each file's size, exhausts the first 256 truncations, samples
longer boundaries and applies nine byte mutations per nonempty seed. It catches
panics only to finish collecting failures, then fails the test.

New HTTP regressions verify typed duplicate/unknown JSON fields, malformed UTF-8,
truncation, body-size and JSON recursion rejection. OAuth duplicate fields cannot
consume a valid code; successful redemption afterward is checked. Six malformed
SCIM patches cannot partially disable/update an identity, and a valid patch still
succeeds. All compare full store snapshots. The existing chunked-backup regression
now checks nine corrupt/reordered/duplicate/truncated/type/encoding/timestamp
frames: every rejection leaves the output directory absent and the source store
unchanged, followed by successful valid restore. Legacy restore still passes.

## Dependency evidence

Installed `cargo-audit 0.22.1` (CI specifies 0.22.2) fetched RustSec into
`target/riwork/q07/advisory-db`: **1,271 advisories**, revision
`e2111519ba6d14a5da59a7b2e5c8083ae8a37c01`, committed
`2026-09-25T19:51:57+02:00`. Both root (456 dependencies) and fuzz (447) audits
exit 0 with **zero reported vulnerabilities or warnings**, no ignores and no
target filters. The no-fetch JSON omits revision/date; the database's Git revision
was separately recorded. Results are specific to this database snapshot.

`cargo tree --duplicates` reports multiple parser families, including
`asn1-rs` 0.6.2/0.7.2, `x509-parser` 0.16.0/0.18.1, three `base64` versions and
`tungstenite` 0.29.0/0.30.0. These remain dependency-review surfaces, not newly
demonstrated vulnerabilities. Installed `cargo-outdated 0.17.0` returned an empty
root-dependency update list **offline**, using cached registry data; no current
registry freshness claim follows.

Both lockfiles and dependency manifests are unchanged. SHA-256:

```text
Cargo.lock       3bd9eda0937289814675ba18fba2d7bc96cef0e1c31860dda4d772ebf40fcd94
fuzz/Cargo.lock  afdface8f4cc118e0cef6e1d039ba5992da53e190766ff7b1f8e769197916660
```

## Initial dependency checks and repair verification

All builds/tests use `CARGO_BUILD_JOBS=2`; artifacts/logs remain under `target/`.
Rust toolchain: 1.98.1, local aarch64 macOS. Raw logs/JSON are in
`target/riwork/q07/`. Dependency/tool-availability checks below are from the initial
Q07 slice; no dependency versions or lockfiles changed during repair. Repair logs
use the `repair-` prefix, preserving the original failure evidence.

| Command | Outcome |
| --- | --- |
| `cargo audit --db target/riwork/q07/advisory-db --json` | Exit 0; fresh database fetch, root audit above. |
| `cargo audit --db target/riwork/q07/advisory-db --no-fetch --json --file fuzz/Cargo.lock` | Exit 0; same database, fuzz audit above. |
| `cargo tree --locked --duplicates` | Exit 0; graph recorded in `dependency-duplicates.log`. |
| `cargo outdated --offline --root-deps-only --format json` | Exit 0; cached-data-only result above. |
| `scripts/test-parser-assurance.sh` | Repair: exit 0; bounds 1 passed, adjacent SCIM 2 passed, operations 14 passed, assurance 5 passed; corpus 2,607 calls / 0 panics, 1 test passed. All 23 retained smoke tests pass. Initial run exited 101 with Q07-SCIM-01 failures. |
| `cargo test --locked --features test-support,fuzzing --test identity scim -- --nocapture` | Repair: exit 0; 2 passed, 124 filtered. Inbound ownership/atomicity/retries/session deprovisioning and outbound group provisioning/remote preservation/stale-job checks pass. |
| `cargo test --locked --features test-support,fuzzing --lib saml::wire::document_tests` | Exit 0; all 3 existing XML depth, deep-unclosed and CDATA regressions pass. |
| `cargo clippy --locked --features test-support,fuzzing --lib --test parser_assurance --test fuzz_corpus --test operations --test identity -- -D warnings` | Repair: exit 0 on final code; focused targets clean. |
| `cargo fmt --all -- --check`, `git diff --check`, `bash -n scripts/test-parser-assurance.sh` | Repair: exit 0. |
| `python3 scripts/check-docs.py`, `python3 scripts/check-repo-hygiene.py` | Repair: exit 0. |
| `cargo fuzz --version` | Exit 101: cargo-fuzz unavailable. Installed nightly is 1.90.0, older than the 1.98 crate MSRV. No instrumented/libFuzzer run or coverage percentage claimed. |

The [smoke script](../scripts/test-parser-assurance.sh) records the exact locked
test invocations, includes the two adjacent SCIM unit regressions and runs
integration targets with `--no-fail-fast`.

## Continuation: LDAP BER and outbound SCIM responses (2026-09-28)

The parser target now calls the listener's actual `LdapCodec` over an in-memory
Tokio reader, retaining the listener's 32 KiB BER frame and 16-level decoder
limits. The adapter accepts at most 65,536 input bytes and reads at most 1,000
messages, matching the listener's per-connection request cap. Five retained BER
seeds cover a valid Unbind, valid present-filter Search, truncated frame,
indefinite length and excessive filter nesting. Direct tests assert valid message
IDs and malformed rejection. This is decoder coverage only: it does not perform
bind, search authorization, TLS, filter policy checks or directory snapshots.

Outbound SCIM now shares a pure response-body JSON parser and remote lookup
identity check between the live provisioner and the bounded adapter. The adapter
receives only JSON within the campaign's 32,768-byte/32-level/2,048-node guard;
the live HTTP path retains its independent 2 MiB response cap. Five retained
seeds cover empty and single bound lookup results, duplicate/ambiguous and
mismatched identity results, and truncated JSON. Direct tests assert that invalid
count, shape, ID and external identity cannot be selected. An independent
loopback SCIM regression confirms that a mismatched remote lookup records a
`conflict` job error without creating a remote user or local link or changing
stored users/groups. Existing outbound tests exercise credential acquisition,
authorization, conditional updates, remote-owned attributes and stale jobs.

The corpus is now 32 seeds: **3,161 replay/mutation calls, zero panics**.
The smoke script passes **25 tests**, including the original SCIM PATCH repair.
Focused identity SCIM tests pass (2), outbound SCIM OAuth tests pass (14), and
all 11 library tests pass. All-target Clippy with `-D warnings` passes after
moving the earlier SCIM fuzz hooks before their test module. No dependencies or
lockfiles changed. `cargo fuzz --version` still exits 101 because the subcommand
is unavailable; the installed nightly is 1.90.0, below the crate's 1.98 MSRV.
No instrumented campaign or coverage percentage is claimed.

## Third slice: signed SAML Redirect and JOSE claims (2026-09-28)

The bounded parser target now calls the live SAML `verified_xml` path for
Redirect `SAMLRequest` fields against the pinned, test-only public certificate
in `src/saml/fixtures/q07-public.cert`. The `.cert` extension keeps this
PEM-encoded public fixture visible to normal Git staging.
Five retained queries cover a valid RSA-SHA256 signature and raw DEFLATE
request, an altered signature, invalid compressed bytes signed by the fixture
key, trailing DEFLATE data and a signed compression expansion above the XML
limit. The fixture's private key is kept only in ignored `target/` generation
artifacts; the corpus and source tree contain the public certificate and signed
inputs. The adapter accepts at most 32,768 bytes, while production retains its
64 KiB form limit, 48 KiB XML limit, 24-level depth and 2,048-node bounds.
Only one pinned certificate is checked per adapter call. Signature
verification precedes DEFLATE expansion. A retained unit test asserts the
valid request succeeds and every malformed request is rejected.

The existing authenticated SAML flow now compares its full store snapshot
across tampered signatures, wrong ACS URLs, expired requests and nested ID
wrapping, then verifies the signed-in session remains usable.

The same target also calls the production `PublicJwks::verify` and
`verify_set` claim paths with a pinned, test-only Ed25519 JWK; no token supplied
key or URL is trusted. Ten retained JWTs cover valid claims, incorrect issuer,
audience, subject and expiry, an unknown key ID, embedded JWK, altered
signature, malformed claims and a SET without `exp`. Only inputs with exactly
two JWT separators and at most the production 16,384-byte token budget reach
this path. Each input gets two verifier calls against one pinned key, and JSON
deserialization retains serde_json's default recursion limit. A direct test
asserts signature, key, issuer, audience and expiry rejection, plus the
deliberate SET `iat`/`jti` rule. Subject binding is enforced
by the authenticated workload consumer, not by generic JOSE verification.

An authenticated machine-grant regression supplies signed assertions with
incorrect issuer, subject, audience and expiry, plus a malformed signature.
Each rejection leaves the **complete store snapshot** equal, including replay
records, grants, credentials and revision; administrator access remains live.
A valid assertion then issues a token, and its replay is rejected without a
further store change. This exercises authority after the pure parser boundary.

The corpus is now **47 files, 7,075 bounded replay/mutation calls, zero
panics**. The finite smoke passes **28 tests**, including Q07-SCIM-01. This
slice adds coverage of signed Redirect requests and pinned JOSE claims; it does
not claim SAML POST signature coverage, upstream SAML session state, instrumented
fuzz coverage or released/backend behavior. No production acceptance rule,
dependency or lockfile changed.

## Wave 3: fresh dependency triage (2026-09-28)

`cargo-audit 0.22.1` fetched a fresh RustSec database checkout under ignored
`target/riwork/q07-wave3/`. Its 1,271-advisory HEAD is
[`e2111519ba6d14a5da59a7b2e5c8083ae8a37c01`](https://github.com/RustSec/advisory-db/commit/e2111519ba6d14a5da59a7b2e5c8083ae8a37c01),
committed 2026-09-25; a 2026-09-28 remote HEAD check matched exactly. Both
exact lockfiles were audited with no ignores, severity threshold or OS/architecture
filter. Root: 456 packages, zero vulnerabilities or warnings. Fuzz: 447
packages, zero vulnerabilities or warnings. There are no reported hits to
discard as false positives and no advisory-driven dependency upgrade in this
wave.

The duplicate-version tree remains a review surface, not an advisory hit.
For nearby RustSec entries, locked `rustls-webpki 0.103.15` is above the
`0.103.13` patched floor in RUSTSEC-2026-0104; `tungstenite 0.29.0/0.30.0`
are above the `0.20.1` floor in RUSTSEC-2023-0065; all three locked `base64`
versions are above the `0.5.2` floor in RUSTSEC-2017-0004. Those older-range
entries do not apply to these exact versions. `x509-parser 0.16.0/0.18.1` and
`asn1-rs 0.6.2/0.7.2` remain parser families worth testing, even though this
RustSec snapshot has no advisory hits for them. A zero-hit audit does not prove
absence of undisclosed vulnerabilities or establish the released binary graph.

## Wave 3: instrumented finite parser campaign (2026-09-28)

`cargo-fuzz 0.13.2` and `rustc/cargo 1.101.0-nightly` were installed only
under ignored `target/riwork/q07-wave3/`. The local nightly's `rust-src` and
LLVM tools were also installed there; the default 1.98.1 toolchain was not
changed. An address-sanitized libFuzzer run compiled the existing production
`parsers` adapter with coverage feedback. It loaded all **47** retained seeds,
including SCIM, LDAP, SAML and JOSE; its mutable copy and artifacts directory
are under `target/riwork/q07-wave3/`.

The finite command capped input at 65,536 bytes, each case at 10 seconds,
memory at 2 GiB, and the campaign at 20,000 runs or 120 seconds. It completed
**20,000 runs in 11 seconds, exit 0**, with no crash artifact. LibFuzzer
reported final `cov: 8371`, `ft: 16527` and an active corpus of 946 inputs;
1,154 new units were added during the run, with minimization/reduction during
evolution. These are process-local coverage feedback counters, not source or
branch coverage percentages. Its adaptive generated-input length reached
3,264 bytes despite the 65,536-byte cap; larger byte-boundary checks remain
in the finite smoke. The complete raw log and mutated corpus are retained
under ignored `target/riwork/q07-wave3/`.

A separate `cargo fuzz coverage --dev --sanitizer none -j 2` pass replayed the
**948 files** in the post-fuzz copied corpus and merged an LLVM profile at
`target/riwork/q07-wave3/coverage/parsers/coverage.profdata`. This pass was
profile collection over fixed inputs, not another fuzz campaign. The retained
selected-file report shows line coverage of 90.48% for `src/fuzzing.rs`, 55.22%
for `src/jose.rs`, 19.05% for `src/saml/wire.rs`, 20.20% for `src/scim.rs` and
1.24% for the much broader `src/ldap_server.rs`. The LDAP decoder lives in
`ldap3_proto`; its `lib.rs` showed 58.33% line coverage in a separate report.
These whole-file figures mix adapter and unrelated production paths and do not
measure SAML/JOSE authorization, full LDAP policy or product-wide coverage.
The LLVM report emitted no branch denominator, so no branch percentage is
claimed.

## Ongoing Q07 obligations

Continue advisory triage with fresh databases and the exact released dependency
graphs. Keep Q07-SCIM-01's passing regression gates enabled and verify the repair
after integration and in release artifacts. Run instrumented
finite fuzz campaigns when compatible tooling is available, retaining minimized
crashes and bounded reproduction commands. Expand SAML POST signature and
upstream session state, LDAP bind/search authorization and directory
snapshots, full outbound SCIM HTTP response limits and recovery, and authenticated
backup-plaintext coverage. Verify shared invariants
after architecture/engine/contracts integration across actual Essentials/Platform
assemblies, supported backends and released artifacts. Recovery freshness/default
invalidation remains separate Q01/A02 work; passing framing tests does not prove
anti-rollback protection. No security, compatibility, performance or recovery
certification follows from this bounded slice. No commits or RiWork completion
status changes were made. Full all-target tests, release builds and external
integration checks were not run for this slice.
