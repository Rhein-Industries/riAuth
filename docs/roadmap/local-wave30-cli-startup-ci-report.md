# Wave 30: CLI certificate fixture startup source diagnosis

Date: 2026-10-02. Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Existing worktree: `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`.
Reservation: `wave30_CI_cli_startup_source_diagnosis`, this new report only.

The failed Linux check reached the configured server's startup wait and observed its child exit. The actual child exit status and stderr were discarded, so the cause remains **UNKNOWN**. Source inspection identifies one unsafe readiness probe: the parent repeatedly binds the child's intended address. The proposal below removes that temporary occupation using a bounded TCP connect. It is a source-first correction proposal, not a claim that a port collision caused the recorded failure.

No test, helper, product, workflow, or existing report was edited. No source was aligned or imported. No Cargo, typecheck, test, native binary, socket, listener, connect, HTTP, provider, browser, service, network query, download, or runtime command ran. No shared runtime slot was acquired or released. Root owns any subsequent source reservation and runtime release.

## Actual CI receipt and its limits

| Item | Actual identity or result |
| --- | --- |
| Run / check job | `37061329345` / `111020195345` |
| Checked source | `56bd0829514ed8014cc9563fc7b0e727dba46d1d` |
| Source tree | `4b5ceade9e9e09ea8e48ae80d8de732f1e411045` |
| Downloaded check log, read only | `/tmp/riauth-wave30-ci-37061329345/check.log` |
| Log bytes / mode | `320779` / `0600` |
| Log SHA-256 | `bad1860b1454adc7db80f5fcb35c621e46fc70214855deb2bfe63d4f5dd1dc45` |
| Actual test command | `cargo test --all-targets --features test-support,fuzzing --locked` |
| CLI target result | `21 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out`, `23.47s` |
| Failed test | `cli_certificate_bind_and_revoke_require_retry_binding` |
| Panic | `tests/cli.rs:954:9`, `server exited early` |
| Cargo/check-step exit | `101`; this is not the server child's numeric exit |
| Child exit / stderr | Not retained; no diagnosis of their contents is possible |

The log records an Ubuntu 24.04.5 runner, image version `20260927.320.1`, and the exact source checkout at log lines 104–122. The test step's environment at lines 1293–1297 records build jobs `2`, development debug `0`, `RUST_TEST_THREADS=4`, and incremental `0`. It does not supply evidence of a test-profile debug override. These are the historical CI settings, not a newly authorized local budget.

The complete CLI result/failure tail was read at log lines 1954–1995. The CLI target began at `2026-10-02T20:51:13.0976821Z`; the failed test was reported at `20:51:26.0335840Z`; the Cargo step reported exit 101 at `20:51:36.6481922Z`. The early-child assertion at line 954 precedes the helper's login at lines 961–967 and return at line 968. Therefore this run did not reach the selected test's certificate bind/revoke, retry-binding, replay, revision, or audit assertions. A startup failure supplies no result for those assertions.

Root supplied that audit job `111020195553` and integration job `111020195467` succeeded. Root's prior whole-CI success for run `37053472817` remains dated historical evidence. This worker did not query those runs or reinterpret them as success of the current failed check. Later steps following the failed all-targets command have no success credit from this log.

The whole downloaded log was read as bytes for its identity, and its checkout, command/environment, target summaries, and complete CLI failure tail were inspected. This does not claim a manual review of every unrelated compiler/build line. Initial broad source/log output was truncated by tool output limits; the relevant source bodies and failure tail were reread in bounded spans. Those truncations do not recover missing child stderr or constitute a runtime check.

## Immutable source coverage

All source witnesses below refer to `56bd0829514ed8014cc9563fc7b0e727dba46d1d`. Body reading is distinguished from whole-file identity comparison.

| Body read / witness | Relevant conclusion |
| --- | --- |
| `tests/cli.rs:1–64` | Imports, owned `Server(Child)`, its kill/wait `Drop`, CLI spawn/output handling, and success assertions were read in full. Drop expresses cleanup attempts; the failed run does not establish their observed success. |
| `tests/cli.rs:196–254` | The first separate fixture allocates a loopback port, spawns its server with null output streams, and also uses a repeated bind probe. This distinct loop is identified and remains outside the proposed hunk. |
| `tests/cli.rs:912–969` | Both helper bodies were read in full: initial port allocation/drop, successful init invocation, configuration closure, owned child spawn, startup checks, login, and return. |
| `tests/cli.rs:1951–2142` | The entire selected test was read: public CA fixture, configured forwarded-certificate profile, bind/revoke refusal and replay cases, revision checks, and exactly-once audit assertions. |
| `tests/cli.rs:2145–2355` | The adjacent RADIUS certificate-file helper and its configured startup caller were read, including its distinct native-TLS configuration. It is not the failed fixture's configuration. |
| `src/main.rs:1–33`; `src/cli.rs:65–91, 1433–1436` | Full main dispatch and relevant error reporting / serve dispatch: configuration load leads to bootstrap; propagated errors lead to a CLI exit. Its actual value and message were not retained for the failed child. |
| `src/cli/local.rs:374–425` | Init validates configuration, initializes the core/store, and writes the initial config through the existing private writer. No init or credential writer was changed. |
| `src/bootstrap.rs:352–375` | Full serve entry: config validation, store inspection, initialized-core open, then API serving. Startup can fail before listening. |
| `src/api/server.rs:1–279`, relevant TLS helpers through line 359 | Serving preflight / role startup / router construction precede HTTP serving. `serve_http` computes TLS/profile material and binds `config.listen` at line 198 with `await?`. There is no bind retry in this path. |
| `src/config.rs:451–494, 549–596, 733–915, 940–1108` | Relevant defaults, validation, config/path loading, private-directory and private-secret/read/write guards were inspected. This is not a claim to have read the unrelated load span 916–939. |
| `src/mtls_config.rs:1–89`; `src/mtls.rs:1–115` | Full profile declaration/validation and bounded trust-material reading. The selected forwarded-header configuration requires its existing trusted-proxy setting. Public CA material is distinct from private secret input. |
| `src/core.rs:90–245`; `src/node_security.rs:1–245` | Initialization/open and relevant agreement construction/enforcement were inspected. The agreement's issuer, capability, authentication, and rate settings are preserved. The fixture's local trusted-proxy / certificate-profile change does not by itself prove an agreement mismatch. |
| `src/assembly.rs:247–269` | Store configuration selects PostgreSQL or the local redb path, using the optional database-key reader. The selected init fixture uses the existing local defaults. |
| `src/api/probes.rs:1–145`; `src/api.rs:253–254` route witnesses | Existing health/readiness endpoints use store/capacity readiness semantics. Adding an HTTP readiness transaction would be a larger change than the proposed bounded connect. |
| `src/cli/transport.rs:62–104` | The unchanged login transport uses the configured issuer, existing timeout, and no redirects. Login remains the existing protocol step after the wait. |

Static enumeration covered every spawn and readiness loop in `tests/cli.rs`: three `.spawn()` calls (CLI invocation, first separate server fixture, configured helper), two repeated-bind readiness loops, and two initial ephemeral-port allocations. The wrapper at line 912 delegates directly to the configured helper. Its 14 direct plain callers are at lines `981, 1080, 1221, 1323, 1441, 1624, 1791, 2681, 2749, 2810, 2909, 3063, 3319, 3592`; the two direct configured callers are the certificate fixture at `2004` and RADIUS fixture at `2304`. The complete unrelated caller assertion bodies were not all manually reread. They are protected by the whole-file reversal below.

The selected fixture generates a public EC CA and configures `ClientCertificateMode::Optional`, a trust-anchor file, `X-Client-Cert`, and trusted proxy `127.0.0.1`. It does not configure native HTTPS certificate/key files. The child nevertheless reads/validates the configured trust material before its HTTP listener is bound. These are source-backed startup paths, not evidence of a provider, CA, permission, configuration, or agreement failure in the actual run. No actual fixture file, private credential, or unretained server log was inspected.

## One unsafe probe seam and one proposed correction

The configured helper obtains an address from an initial `127.0.0.1:0` listener, drops that listener, writes the intended address into its fixture config, and spawns its own child. Its wait condition at line 953 then repeatedly evaluates `TcpListener::bind(addr).is_ok()`.

A successful parent probe owns that address briefly, until its temporary listener is dropped. The child independently executes `tokio::net::TcpListener::bind(config.listen).await?`. A scheduling interleaving can put the child's bind inside that temporary parent occupation and propagate a bind failure. This is a concrete source-level risk; the actual log cannot establish that interleaving. Conversely, any bind error makes the old `while` terminate without checking the child or deadline for that iteration. A bind error alone does not identify the listener owner or establish readiness.

Propose only adding `TcpStream` to the existing standard-library import and replacing this helper's wait condition with `TcpStream::connect_timeout(&addr, Duration::from_millis(30))`. After each bounded attempt, keep the existing owned-child assertion and exact 15-second deadline assertion, then accept a successful connection. The existing 30 ms sleep remains unchanged after an unsuccessful attempt. No CLI retry, server restart, port fallback, sleep increase, new dependency, product bind change, stderr capture, or assertion relaxation is proposed.

This removes the parent's listener occupation. A TCP connection establishes socket acceptance, not HTTP readiness or ownership of the listener. The original ephemeral-port handoff still exists, and an unrelated listener could accept the connection; no ownership proof or alternative listener is introduced. The same configured issuer, owned child, login, and certificate assertions remain the subsequent checks. The deadline declaration/check is unchanged and checked before accepting readiness; each connection attempt is bounded to 30 ms. This is not a new global timing or write-safety guarantee.

Exact zero-context prospective diff, against the immutable CI source; **not applied**:

```diff
--- a/tests/cli.rs
+++ b/tests/cli.rs
@@ -4 +4 @@
-    net::TcpListener,
+    net::{TcpListener, TcpStream},
@@ -953 +953,2 @@
-    while TcpListener::bind(addr).is_ok() {
+    loop {
+        let ready = TcpStream::connect_timeout(&addr, Duration::from_millis(30)).is_ok();
@@ -958,0 +960,3 @@
+        if ready {
+            break;
+        }
```

The complete proposed wait block is:

```rust
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let ready = TcpStream::connect_timeout(&addr, Duration::from_millis(30)).is_ok();
        assert!(
            server.0.try_wait().unwrap().is_none(),
            "server exited early"
        );
        assert!(Instant::now() < deadline, "server did not start");
        if ready {
            break;
        }
        thread::sleep(Duration::from_millis(30));
    }
```

## Proposal identity and protected-byte proof

| In-memory document | Bytes / lines | SHA-256 |
| --- | --- | --- |
| Immutable complete `tests/cli.rs` | `118817` / `3794` | `b7e9641345c609eb2dc788b820b3a4090d89d8650b48949d6b7e6d9cfabd8a59` |
| Complete prospective `tests/cli.rs` | `118935` / `3798` | `4941d65dd62abfb6614be55260a6d14077e5f17bb2126d366f3e54ca57968e59` |
| Exact zero-context diff above, including final LF | `346` / `13` | `d5958c04a6420ce60a927e015a5cd39f9c06232ee218d98d455ce0e3ad63ab0f` |
| Selected original test, lines 1951–2142 | `5768` / `192` | `cda1580af7caf51a4201cd4e7c2f182242cceb96c9de0da1ccf51bc3c2c5a600` |
| Original suffix from line 1951 through EOF | `58590` / `1844` | `92d2b53bef639703f13df98509fa4284a685fecdbac09eb5e3b3575f4cfe2a81` |

Each replacement anchor was unique. Applying the zero-context hunks independently in memory reproduced the complete candidate. Reversing only the import and loop replacements reproduced all `118817` original bytes exactly. The prefix outside the import, suffix following the wait loop, entire certificate test, and entire remaining file were unchanged. The two existing 30 ms sleep statements, two 15-second deadline declarations, three spawns, null server output streams, environment removal, initial port allocation, configuration closure, init/login commands, `Server::drop`, and all certificate/retry/revision/audit/refusal/mutation assertions are retained. Rust parsing, compilation, typechecking, and candidate execution were not performed.

At entry this worktree was `0ec651d6f7b30e320a8840683bd1e2d2bab0e484`. Root's published comparison pin resolves to `74e106b819e7186d0ac41964cedbe8217fa7e881`. The CI, own entry, and published `tests/cli.rs` blobs all equal `70084462c1102c2a3e19a963b8bee185b961fe87`. Whole-byte comparisons also matched the examined CI workflow, main/CLI/config/bootstrap/core/server/mTLS/node-security/transport files to the published pin. Those comparisons are identity checks, not claims to have read every body in those files. The own worktree's `src/api/probes.rs` differs from the CI/published version; its relevant body was read from the immutable CI object, with no import or alignment. This report does not claim that the whole worktree matches the CI tree.

The previously published D01 descriptor-aware design and existing helper/report are outside this reservation and remain unchanged. Root reported the D01 design accepted/published, with independent design review pending; this CI source review grants no D01 memory, native, or browser release.

## One prospective focused runtime command, held

Only after root reviews/reserves the exact test-source seam and separately releases a monitored private build/service budget, the following single filtered command is proposed:

```sh
env CARGO_TARGET_DIR="$PWD/target" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support,fuzzing --test cli cli_certificate_bind_and_revoke_require_retry_binding -- --exact --test-threads=1
```

This preserves the CI features and exercises the selected real startup, bind/revoke retry-binding refusals, replay/revision, and audit assertions. It requires the native test dependencies and root-owned private target with an actually monitored minimum 8 GiB floor; this report neither measures current capacity nor claims dependencies available. A one-thread filtered pass would not reproduce the original four-thread all-targets CI schedule, prove the historical cause, validate all 16 helper callers, or establish a current all-green check. No command above was executed. No slot was reserved, acquired, or released by this worker.

## Actual static checks and handoff boundaries

The downloaded log's exact byte count, mode, and SHA-256 were verified. The source/candidate/diff hashes, unique anchors, independent in-memory hunk reconstruction, complete reversal, protected suffix/test bytes, and unchanged spawn/sleep/deadline counts passed. These operations inspected source text and Git objects only; they did not import or execute test/helper/product code.

Before and after report creation, `python3 scripts/check-docs.py` exited `0`: `Markdown links and build-directory layout checked`. There were no observed pre-existing build-layout errors to excuse. The report's fenced diff and full proposed loop reconstructed the pinned candidate with its exact hash; `tests/cli.rs` on disk remained byte-identical to the immutable baseline. Trailing-whitespace/final-LF checks and `git diff --check` passed. Initial final-scope inspection found no tracked working/index changes and exactly one new file, this report. The diff line-count entry was corrected from 12 to the measured 13 before committing; the proposal bytes/hash did not change. No static Rust typecheck result or runtime result is claimed.

Final staged verification passed: exactly this new report, index/file equality, no other working or untracked files, `git diff --cached --check` exit `0`, and the report-aware docs checker exit `0`. The existing D01 helper SHA-256 `75bfb8a63d68aee6ee6b2e500959c0e7d80a6331f1c690022c4300134c7d34f1` and D01 report SHA-256 `9530a35876c00d4fd6e949bb51ab56b12b36e0d829358a1d3e8672273ca670bb` were also rechecked unchanged. The commit itself changes no source or existing document. Clean handoff verification follows the commit; its immutable hash is returned separately rather than embedded in its own content.

Root must review this exact proposed seam before granting any test-source ownership. The observed startup cause remains **UNKNOWN**, child numeric exit/stderr remain unavailable, the failed run remains failed, and current overall CI success remains unestablished. Existing closed rows and all unrelated lanes are unchanged.

## 2026-10-03: exact passive-readiness source materialization

Reservation: `wave30_CI_cli_startup_passive_readiness`, source only. Root reported full review of the historical `8ea5c9ebc0bc626935bd96b8570642b9ab31c27e` diagnosis and acceptance at `4f8e45f`, then authorized exactly its archived candidate in `tests/cli.rs` and an append-only evidence update here. All original `17719` report bytes / `142` lines are preserved as the prior read-only phase; their SHA-256 remains `1db401fcba85d7b23db4c3e3e1c8ed171c302b2ba3ea966dd5f415d1df647f48`. Earlier statements that the proposal was not applied describe that historical phase.

Before editing, the complete published `722def77ea2e9e52ad1b560c0cb0dcb54543bb5a:tests/cli.rs`, reviewed CI `56bd0829514ed8014cc9563fc7b0e727dba46d1d:tests/cli.rs`, and own on-disk test file were verified equal: `118817` bytes, SHA-256 `b7e9641345c609eb2dc788b820b3a4090d89d8650b48949d6b7e6d9cfabd8a59`. The published Git blob is `70084462c1102c2a3e19a963b8bee185b961fe87`. Entry HEAD was `8ea5c9ebc0bc626935bd96b8570642b9ab31c27e`, and the working tree and index were clean. No source alignment or import occurred; the whole worktree is not claimed to match current published production.

| Materialized source identity | Actual value |
| --- | --- |
| Source commit | `f142ca460124b245bbe2b5328f1554e21fc95bd5` |
| Source commit parent | `8ea5c9ebc0bc626935bd96b8570642b9ab31c27e` |
| Sole changed source path | `tests/cli.rs` |
| Git blob | `aac85be6bd8bb395170b8fe26319a0a05ea31c2b` |
| Complete source bytes / lines | `118935` / `3798` |
| Complete source SHA-256 | `4941d65dd62abfb6614be55260a6d14077e5f17bb2126d366f3e54ca57968e59` |
| Exact archived zero-context diff | `346` bytes / `13` lines; SHA-256 `d5958c04a6420ce60a927e015a5cd39f9c06232ee218d98d455ce0e3ad63ab0f` |
| Source commit change size | `6` insertions, `2` deletions, one file |

The source materialization contains exactly the three archived hunks: the `TcpStream` import, the configured helper's bounded 30 ms connection attempt, and readiness acceptance after its unchanged child-alive and 15-second deadline assertions. Its unsuccessful-attempt sleep remains exactly 30 ms. The first separate repeated-bind loop is unchanged. The original port allocation, configuration closure, server spawn, null stdout/stderr, environment handling, init/login calls, owned `Server::drop`, selected certificate test, and all remaining fixture/security/refusal/replay/revision/audit/mutation bytes are unchanged.

The actual source was compared with the in-memory replacement of the unique original import and configured-loop anchors. Its generated zero-context diff matched the archived fence byte for byte. Reversing only those anchors restored the entire `118817`-byte published baseline exactly. The suffix after the configured loop and the complete suffix from the selected certificate test through EOF matched the baseline. The file still has exactly three spawns, two original 30 ms sleep statements, and two original 15-second deadline declarations. These are text/byte proofs; they are not a typecheck or runtime test.

Actual authorized formatter command, using the installed pinned toolchain binary directly, with no rustup setup/download or version query:

```sh
/Users/dominik/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin/rustfmt --edition 2024 --check tests/cli.rs
```

It exited `0` with no output. The edition was read from the existing manifest and the `1.98.1` pin from `rust-toolchain.toml`; neither file was edited. This is a static Rust formatting/parse check. No Cargo, compiler, typecheck, test, product binary, helper, TCP connection, socket, listener, HTTP request, provider, or service was executed. The formatter did not rewrite the exact source bytes.

The baseline verification, exact candidate/hash, three-hunk equality, whole-file reversal, protected suffix checks, and source-only scope checks all passed. `git diff --check`, source-staged `git diff --cached --check`, and `python3 scripts/check-docs.py` each exited `0`. The docs checker reported `Markdown links and build-directory layout checked`; no pre-existing layout error was observed. Source-stage index/file equality was verified, and the source commit changed exactly `tests/cli.rs`. The worktree was clean immediately after that source commit. No static check failure was observed in this phase.

The report appendix is committed separately. Complete historical-prefix equality, source-commit/file equality, report-only working scope, trailing-whitespace/final-LF, and existing D01 helper/report hash checks passed. Report-aware `python3 scripts/check-docs.py` and `git diff --check` both exited `0`. Those D01 bytes remain exactly `75bfb8a63d68aee6ee6b2e500959c0e7d80a6331f1c690022c4300134c7d34f1` and `9530a35876c00d4fd6e949bb51ab56b12b36e0d829358a1d3e8672273ca670bb`. The separate report commit follows final index equality/scope/whitespace checks; its hash and clean handoff are returned separately because they cannot be embedded in their own commit. No other existing report, helper, manifest, workflow, product, or test path was changed.

The recorded Linux CI `37061329345` / job `111020195345` remains failed. Its startup cause is **UNKNOWN**, and discarded child stderr/status are still unavailable. This materialization establishes no Linux-fixed result, rare-race reproduction, current all-green CI, listener-ownership proof, or HTTP-readiness proof. Root owns immutable source review, any later exact production alignment, measured private-cache/capacity assessment, and a separate release for the previously proposed single filtered command. No runtime slot was acquired or released, no cache was deleted, and no native build/service/test or runtime command was attempted. Original A09, D01, and all closed-row gates are unchanged; D01 memory/browser/native work remains held.

## Root source review, 2026-10-03 local

Root read the142-line diagnosis, complete f142 source diff and35-line static
appendix. Reversing only the import and configured-loop changes independently
restored the entire118817-byte published baseline. The exact118935-byte candidate
was staged without changing production or the first separate fixture loop.
Typechecking and the selected startup/certificate filter remain unexecuted;
full production alignment and measured private-cache preparation precede any
separate runtime release. The56 CI failure stays failed with unknown child cause;
the later b71 CI success is credited solely to its earlier exact source.

## 2026-10-03: alignment attempt stopped; private-cache preparation only

Reservation: `wave30_CI_cli_startup_aligned_preparation`. Entry HEAD was `b6c51a98208f38b010746b27ecc4eb443ae1d0d1`, with a clean index/worktree. Root authorized one history-preserving merge of fixed published `35c3fd3007c52d8142c7bee1d69aee127cc42a95`, tree `397db5efe3e3a47f6789ebe6d62f1c3d3524c4df`, and explicitly required stopping before any production/test/helper/workflow/config conflict resolution. That condition occurred. **Alignment is not completed, and runtime remains held.**

The own report at entry was `23604` bytes, SHA-256 `1a3ab3cacc6c2fdde0d3c4fdbc1e6146c5b9104861dbf1d1451ff852074fbdfa`. Before the attempt, the published report was verified to contain that complete prefix: `24266` bytes, SHA-256 `ec78b287ef165d9007853856136105201b9e283b3f0ed18dbbcbfd0a16bd39a7`. Its additional `662` bytes, SHA-256 `f0b2aca3b986152586a6a41490643066903e74c7f238b883f797e9cb1c6e0bf5`, are the exact root note retained immediately above. This append preserves the complete published report prefix as well as all own author bytes; it does not replace a stale whole file.

### Actual merge refusal and clean restoration

The sole merge attempt was:

```sh
git -c core.hooksPath=/dev/null merge --no-ff --no-commit 35c3fd3007c52d8142c7bee1d69aee127cc42a95
```

It exited `1`: automatic merge failed with the five conflicts below. No conflict was resolved and no merge commit was created. The merge base was `60437b59933cadd40a1f5fbbb91ba153aee56456`; HEAD remained `b6c51a98208f38b010746b27ecc4eb443ae1d0d1` and MERGE_HEAD was the fixed published pin while the attempt was pending.

| Conflicted path / type | Own stage 2 blob | Published stage 3 blob |
| --- | --- | --- |
| `.github/workflows/check-local-artifacts.yml`, add/add | `0e5426c9212433fffbbd578b71ec9fb682ad9c07` | `a1b136642c1cb3e9043e5cf9e74dd7ce9dc97cc9` |
| `.github/workflows/check-local-shared-handoff.yml`, add/add | `b43653d09de8ce5db3bc3300a5e78457aa0c7b82` | `f63110098c13d717446a198c487c00678cfeb60a` |
| `scripts/check-local-edition-transition-postgres.py`, content | `d55d51aad16f508b3cb993a15f44e23aeb912729` | `af764b71b95f8ad4d5e1d9b26d094dfa2eaafe49` |
| `docs/roadmap/local-wave30-cli-startup-ci-report.md`, add/add | `764c03bd0deb4d2a5de50126e5502b2d4d25a392` | `8551eb08a3e5c058ab99c4beb19d95ad71469d50` |
| `docs/roadmap/local-wave30-d01-unexpected-failure-plan.md`, add/add | `39ec67610af8915ffc38dfeee413d32f070e600a` | `ed2b68acd9dd2982baa62194ae9883e5afbf7ac1` |

All conflict entries had mode `100644`. The transition helper's stage 1 blob was `6d885b3df4aa2afec1870a9e42738266cfb81bba`. Its stage 1/2/3 lengths and SHA-256 values were respectively `13127` / `d6b0d023164756b2fcabe1a89794400d5ee70b43b6b4874abed3fd2f6a2cce7d`, `21716` / `575dfb049324ede3cb0ddf4bd71e0b42ef46704a015f6a576a38c4052a1ff2c1`, and `26547` / `4c60a7448d3c836215068fcc4f6822655c483f172a822e085f077d27b6a0c6fa`.

The artifact workflow's own/published lengths and SHA-256 values were `18186` / `0a98865e5ee25d2f94b745b32602949553bf4f4e3b83d8bd302fade98eafd948` and `19487` / `62c378975021066d62d5b32d0eee2c87de8501d01ded1ae0fae8147e920e58d7`. The shared workflow's were `77543` / `45304da275d3072c2c0de9f6debed1acc335ffa99e0ed6baa15064549bab5574` and `82149` / `226483dd9edcd5e566b6c5de9f7ab2a36ebdf0c0f204f8ec992645542a3f395f`. These distinct protected sources were not chosen, edited, or semantically resolved. Root must coordinate the exact next resolution; this report grants none.

Both report conflicts had proven prefix relationships. The published D01 report contained the complete `410731`-byte own report (`9530a35876c00d4fd6e949bb51ab56b12b36e0d829358a1d3e8672273ca670bb`) as a prefix; its published version was `412260` bytes (`80940c43c31b480217b409fd84b71e3e2cf73fbd776896e89c46d8e309bc0ced`). No D01 report conflict was resolved or imported. Only the already authorized CLI root-note suffix is retained in this report after restoration.

To restore the clean entry state without source resolution or history rewriting, `git -c core.hooksPath=/dev/null merge --abort` exited `0`. HEAD remained the exact entry commit, MERGE_HEAD was absent, and index/working/untracked checks were clean. The selected CLI file, own report, D01 helper, and D01 report hashes were rechecked unchanged before this append. No reset, rebase, source import, second merge attempt, cache deletion, or main/push mutation occurred.

### Protected source comparisons and their limits

The selected `tests/cli.rs` was byte-identical at entry, fixed published pin, pending automatic merge, and restored entry: `118935` bytes, SHA-256 `4941d65dd62abfb6614be55260a6d14077e5f17bb2126d366f3e54ca57968e59`. The pending merge showed no content differences from the fixed pin under `src/`, `crates/`, `.cargo/`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `build.rs`, and the selected CLI file. This is a temporary comparison, not certification of a committed aligned tree; the merge was aborted.

An exhaustive tracked mode/type/blob/path NUL-manifest comparison after restoration covered `src/`, `crates/`, `.cargo/`, and root Cargo/toolchain/build/formatter configuration paths. The restored entry had `347` entries, manifest SHA-256 `c3e083951ced9a926de6e44c62222fac84ae8e8f9962ba2ea83701d682f46f5b`; fixed published had `350`, manifest SHA-256 `f1decf551c0ad8bf8f613b1c8d8ffba80477f1733bb6180c1011b018070adf47`. Fourteen paths differ or are absent locally, so the current production tree is explicitly **not aligned**:

```text
src/api.rs
src/api/probes.rs
src/assembly/cloud_operations.rs
src/background.rs
src/kms.rs
src/kms_essentials.rs
src/portal.rs
src/portal/source-stage.html
src/portal/source-stage.js
src/portal/source_stage.rs
src/provisioning.rs
src/reconciliation.rs
src/store/maintenance.rs
src/telemetry.rs
```

There are `0` branch-only tracked paths when comparing restored entry with fixed published. That path-set fact does not establish identical blobs, a successful build, or inclusion/validation of unrelated build targets. No unrelated target was built or run; only owned-cache metadata for previously built targets was inspected. Root's exact protected conflict resolution and a new complete modes/blobs comparison must precede any runtime release.

### Own private target and dependency preparation

Only the existing own target was inventoried: `/Users/dominik/orca/projects/riAuth-public-preview-sol-diagnostics-wave30/target`. It is a real directory, not a symlink, on device `16777234`, mode `0755`. No second target was created, another worker's cache read, cached artifact executed, or cache content altered/deleted. The observed existing log files had mode `0644`; they were inventoried by metadata only and are not proposed as future private log sinks.

`du -sk target` returned `2790036` KiB, or `2.660786` GiB. The initial `df -k target` receipt reported `25474684` KiB available (`24.294552` GiB) on `/dev/disk3s5`. A later metadata receipt at `2026-10-02T22:14:15.544441+00:00` (2026-10-03 local) reported `25451156` KiB available. These are shared-filesystem snapshots, not reserved capacity or a guarantee against other concurrent growth. No other build/registry cache or filesystem was inventoried, and no runtime-capacity success is claimed; installed toolchain stat/hashes are separately recorded below.

The own target contains `444` fingerprint JSON files and seven root-package fingerprints, all with `default, essentials, platform, test-support`; **none includes `fuzzing`**. No `debug/deps/cli-*` artifact or CLI fingerprint exists. The cached lib/server/maintenance and O06 test products are historical partial warmth, not the requested exact test build. Their cached root rustflags are empty, config fingerprint is `9396254390672932401`, and profile hashes are `12672335563272108896` for lib/binaries and `11094973624911973823` for tests. Numeric profile hashes alone do not prove current dev/test debug settings or future freshness.

| Own metadata identity | SHA-256 / observed detail |
| --- | --- |
| `.rustc_info.json`, `1965` bytes | `27df402be20083ab5b4835c05762e2b77beed67288686dd8fda9193b24cdd7c4`; cached banner says `1.98.1 (48a229cea 2026-09-01)`, host `aarch64-apple-darwin` |
| Lib fingerprint | `ac6f00d7f80260b8f443564cbbdd8e64f91d91aa42f8d001c0a23e13778f1c10` |
| Server fingerprint | `375004fd606e5dd7d89a5b0304f6997376e0d1db7ac5e60fb751a811255c952d` |
| Maintenance fingerprint | `e2abf7febe96532e829cea37586930bb5c0d835d1451648b0cb034e3b8657408` |
| Test-lib fingerprint | `a67afdeadbe407ddaa1ce2e557db11ec070e21495a10cfbab8058d5b597f5e08` |
| AWS-LC native lib fingerprint | `eb05db7ea539f72226bb8b771ab8097a80417150c4c39399f0f33bb9be48d46b` |
| OpenSSL native lib fingerprint | `899b5b8698182ecf7326104d2cc08b2628783d0290a4b3a4a9c13556d5316ca3` |
| OpenSSL own build-link metadata | `7bdbfe7503fa8dda18755d698151124161d8e24df0f386cd7aa06b8f0f7469d8`; cached include/library paths under `/opt/homebrew/opt/openssl@3`, dynamic `ssl` / `crypto` links |

The cached compiler banner was read from metadata; no compiler/tool/version command ran. The existing installed `1.98.1-aarch64-apple-darwin` toolchain binaries were inspected by stat/hash only: rustc `412504` bytes / `766eda9d8f53afd6fc7f27b3cd2e444dd22afacb5afa710a5625fc8e45b8c941`, Cargo `31960040` bytes / `6e17e865f3a20dd55a1d212f849f58b77124179f0de7c52973096d84ba34118d`, rustfmt `4455400` bytes / `98e8da71078a8b5710f1818c98da25d92de162a122bd9a6db222ca929e545468`; each mode was `0755`. None was executed in this preparation phase. Their presence and old native link metadata do not verify current external native-header/library availability.

Own and published `Cargo.lock` are byte-identical, SHA-256 `b5c9d11c001244b8017303ce8c20516e02946483845eee40720b0910758d4426`. The unchanged manifest SHA-256 is `58e5ef824ed96290179c9f76fea208dc37173caeee21b6ce8d37f8a6dd1abcb8`; toolchain TOML is `887f9be066a15585a2c583578e84b0fcb541126d81546276bad3d2ff00d61167`. The manifest default is `platform`, which includes `essentials`, AWS-LC-backed SAML/TLS and the existing native/default dependency graph. `test-support` and `fuzzing` each declare an empty dependency feature list but change root cfg compilation. Locked relevant versions include AWS-LC rs/sys `1.18.1` / `0.45.0`, OpenSSL/sys `0.10.81` / `0.9.117`, reqwest `0.13.5`, rustls `0.23.45`, risaml `0.7.0`, tempfile `3.27.0`, and both WebAuthn crates `0.5.5`. Registry checksums were read from this lockfile; no registry/package cache or network was consulted.

No repo `.cargo/config` or `.cargo/config.toml` or `build.rs` was present. The manifest defines only its release profile (`lto=thin`, `strip=true`); dev/test debug zero will be set by the explicit future environment. Allowed environment names for rustflags, build target, target directory, toolchain, jobs, incremental, dev/test debug, and OpenSSL paths were checked and were unset. Shared/global registry/configuration or credential files were not read. Thus exact future Cargo freshness remains unproven despite a matching lockfile and partial dependency/native cache. The installed cache remains unchanged; root library/binaries and the missing CLI executable need compilation/linking for the aligned source and `fuzzing` feature. No fresh duplicate target is proposed.

### Concrete estimate and one future bounded run, still held

Observed cache product sizes were lib rlib/rmeta `407452200` / `46131722` bytes, server `260210352`, maintenance `68403040`, and test-lib executable `218037168`; root copies and dependency products also contribute to the measured target. Use these as scale witnesses, not as hashes or current-source binaries. For planning, allow up to **4 GiB additional** warm-target growth for new root outputs, the uncached CLI executable, object/link overlap, metadata and ordinary fixture files. This gives a concrete estimated own-target peak of about **6.661 GiB**. This is an estimate, not a measured build peak or guarantee; native/profile/freshness invalidation could exceed it and must trigger the resource stop rather than deleting caches or creating a cold duplicate.

Require a fresh start receipt of at least **13 GiB filesystem availability** immediately before any future launch: 4 GiB estimated growth above a **9 GiB stop threshold**, with an **8 GiB floor**. The snapshots exceed that planning threshold, but no space is reserved and they expire as a release decision. Incomplete alignment and unverified exact cache freshness remain blockers independently of capacity.

After root separately reviews/reserves the protected alignment, proves its modes/blobs and selected CLI bytes, and explicitly releases one measured runtime budget, propose only this command using the existing private target:

```sh
env CARGO_TARGET_DIR="$PWD/target" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support,fuzzing --test cli cli_certificate_bind_and_revoke_require_retry_binding -- --exact --test-threads=1
```

Proposed future outer envelope, not materialized or executed here:

- One launch, no retries, with a `1200`-second monotonic outer deadline. Start an owned process group containing Cargo and its descendants; record that group's actual identity. The normal fixture retains its own init/serve/login/CA/bind/revoke/refusal/revision/audit assertions and `Server::drop`. No PostgreSQL fixture/service is added; “owned PG” here means the owned process group.
- An exclusively created private `0700` evidence directory and `0600` files. Capture combined stdout/stderr privately with an `8 MiB` hard cap; cap exhaustion is a first failure and stops the owned run. Keep actual exit/signal information, timeout/resource/cap reasons, bounded resource samples, and cleanup results before assessing test expectations. Publish only reviewed/redacted outcome evidence, never raw credential/session/config/protocol output.
- Sample the actual own-target filesystem every `2` seconds, recording monotonic gaps, availability and sampled minimum, with at most `610` fixed-field samples / `1 MiB` resource evidence. Failed/lost sampling, deadline, cap exhaustion, or observed availability at/below `9 GiB` stops the owned process group. Any observed crossing of `8 GiB` is recorded as a floor failure; sampling does not guarantee unseen values or protection from unrelated filesystem growth.
- On stop, retain the first failure, send owned-group TERM, allow at most `5` seconds, then KILL as needed and wait/reap the owned child. Preserve cleanup failures separately; stop/cancel/timeout is not a passing test. No unrelated process, cluster, fixture, or cache is terminated/deleted. Root must review the actual finite controller/ownership/reap implementation before release; this prose is not an executed ownership proof.
- Release the sole slot immediately after actual exit and owned cleanup/reap evidence, including on failure; no edition/PG/broad follow-up or extra runtime is implied. No slot is reserved, acquired, released, or occupied by this preparation.

Root supplied that earlier-source `b71` CI run `37063876066` completed whole SUCCESS. It remains dated earlier-source evidence and was not queried here or reinterpreted as proof of this risk correction. The old `56bd0829514ed8014cc9563fc7b0e727dba46d1d` run remains failed with cause **UNKNOWN**. No Linux-fixed/current-all-green, rare-race reproduction, A09 artifact/shared completion, D01 execution, or closed-row status change is claimed.

Actual report-only checks passed: complete own/published prefix preservation, no residual merge state, unchanged selected CLI/D01 bytes, report-only working scope, trailing whitespace/final LF, `git diff --check` exit `0`, and `python3 scripts/check-docs.py` exit `0` (`Markdown links and build-directory layout checked`). No source change remains relative to restored entry. Final one-file index equality and staged-whitespace checks precede the report commit, followed by clean handoff verification. The actual merge failure and inability to complete protected alignment are retained. This report append is the only subsequent mutation; root owns exact protected-source resolution and any later runtime decision.

## 2026-10-03: exact alignment committed; supervisor source proposal only

Reservation: `wave30_CI_cli_startup_exact_alignment_resolution`. Root authorized one new history-preserving merge from clean `e4d0a6ff49d39f82bd40fdb5f8d0a1c69027f37d` and the five exact conflict decisions. The preceding aborted attempt remains preserved above. This phase completed alignment without any semantic source edit, reset, rebase, second pin, main/push/status change, or cache deletion. Runtime remains held.

| Committed alignment receipt | Actual identity |
| --- | --- |
| Merge commit | `22a0e4b7b41266d958c802ea8fdd1e41522df127` |
| First / second parents | `e4d0a6ff49d39f82bd40fdb5f8d0a1c69027f37d` / `35c3fd3007c52d8142c7bee1d69aee127cc42a95` |
| Merge tree | `b727a071497ce61851882d6383cab5b5ac6129d3` |
| Fixed published tree | `397db5efe3e3a47f6789ebe6d62f1c3d3524c4df` |
| Entire tree differences from fixed35 | Exactly `docs/roadmap/local-wave30-cli-startup-ci-report.md`, containing the retained own preparation append |
| Branch-only tracked paths | `0`; this does not claim unrelated build targets were run |
| Protected production/config manifest | `350` entries, SHA-256 `f1decf551c0ad8bf8f613b1c8d8ffba80477f1733bb6180c1011b018070adf47`, equal fixed35 |
| CLI source | `118935` bytes, SHA-256 `4941d65dd62abfb6614be55260a6d14077e5f17bb2126d366f3e54ca57968e59`, unchanged |

The authorized merge again exited `1` with exactly the previously observed five conflicts; there was no additional or ambiguous conflict. Before resolving anything, the five-path conflict set, exact source decisions, and both report prefix relationships were reverified. The three protected paths were restored from fixed35 directly: artifact workflow blob `a1b136642c1cb3e9043e5cf9e74dd7ce9dc97cc9`, shared workflow `f63110098c13d717446a198c487c00678cfeb60a`, and transition helper `af764b71b95f8ad4d5e1d9b26d094dfa2eaafe49`, all mode `100644`. No hunk was composed, edited, or inferred from a superseded own proposal.

The CLI report retained WHOLE own e4: `40568` bytes / SHA-256 `33da0b15c81ef80dc6c6413b4b71e9da8e0ede9eb143c5dce79082cd213a4a83`, including the complete published `24266`-byte prefix. The D01 report took WHOLE published: `412260` bytes / SHA-256 `80940c43c31b480217b409fd84b71e3e2cf73fbd776896e89c46d8e309bc0ced`, containing every own `410731`-byte prefix byte. Its helper remains the accepted descriptor source, SHA-256 `75bfb8a63d68aee6ee6b2e500959c0e7d80a6331f1c690022c4300134c7d34f1`. Existing D01/A09 runtime and acceptance limits were not changed.

Before committing, the entire index and working modes/blobs were checked against the fixed protected set: all `src/`, `crates/`, `.cargo/`, and root Cargo/toolchain/build/formatter configuration paths. Every regular file's executable mode and Git blob were independently computed; symlink mode/content handling was explicit. All 350 entries matched. The whole staged tree differed from fixed35 solely by the CLI report; no own branch-only tracked path remained. `git diff --cached --check` and `python3 scripts/check-docs.py` exited `0`. After the merge commit, the exact two parents, sole report difference, and clean index/worktree/untracked state were verified. Alignment does not establish compilation or a runtime result.

### Fresh metadata-only preparation receipt

At `2026-10-02T22:32:15.212406+00:00` (2026-10-03 local), `df -k target` recorded `25439612` KiB available on `/dev/disk3s5`, or `24.261105` GiB. The preceding 2.661 GiB own-target inventory and 4 GiB additional-growth estimate remain planning inputs, not guarantees. A future launch still requires a fresh at-least-13 GiB receipt, a 9 GiB stop threshold, and an 8 GiB floor. No space was reserved and no cache was written/deleted.

A read-only `ps` PID/PPID/PGID/executable-basename filter found no matching Cargo, rustc, riAuth, CLI, clang, cc, or linker process at that instant. It used no arguments/environment/credential fields. This is momentary process-presence metadata, not a claim about other worker ownership, slot availability, or future absence. `waitid`, `WNOWAIT`, `WEXITED`, `WNOHANG`, and `P_PID` interfaces were observed present without invoking the proposed controller.

The exact four cached root/native fingerprint hashes in the controller below were freshly rechecked unchanged. Root lib/server profiles remain `12672335563272108896`; native OpenSSL/AWS-LC profile is `8378823585564974145`; rustflags remain empty, config fingerprint `9396254390672932401`. Root cached features still omit `fuzzing`; no CLI executable exists. Manifest profiles remain release-only with thin LTO/strip, and default/platform plus empty dependency declarations for test-support/fuzzing remain unchanged. Repo/global Cargo config-file existence checks were negative; no contents, registry cache, or credential file were read. The relevant rustflags/wrapper/target/profile/OpenSSL environment overrides and CARGO_HOME/RUSTUP_HOME were unset. Cache reuse is still partial and exact future freshness is unproven. No compiler, Cargo, rustup, native/version/product/helper/test/service/TCP/HTTP/Driver command ran.

### Complete finite supervisor source, archived only

This is a complete source proposal for root's immutable review and later separate materialization/release. It is not installed or executed. The sole future Cargo argv is the previously approved filter; the supervisor prefixes the already installed pinned toolchain's bin directory to child PATH so literal cargo/rustc resolve without rustup setup. It refuses unexpected wrapper/configuration/source/cache changes instead of silently substituting another source, toolchain, cache, port, feature set, or target. It does not inspect secret/credential files.

Source: `27303` UTF-8 bytes / `638` LF-terminated lines; SHA-256 `1be75053e7960de6bb1568cffcd1c092a554dbeb0de2ca63e0231385cd61f695`. The exact fenced text is the complete payload, with no hidden bootstrap, imports from a repository helper, or external harness. Only Python AST parsing and text/node checks have been performed; no proposal imports, definitions, main, supervisor, subprocess launcher, signal handler, observer, native library, network, or runtime path was executed.

The 1200-second monotonic outer deadline reserves 20 seconds for cleanup, so owned Cargo work stops by the outer deadline minus that reserve. It samples nominally every 2 seconds, refuses failed/scope-changing or over-5-second-gap sampling, caps 610 samples/1 MiB resource evidence and 8 MiB private console capture, records sampled minima and actual gaps, and forces a terminal sample. These limits do not guarantee unseen filesystem values or timely kernel I/O. Initial space must be at least 13 GiB; observed at/below 9 stops; observed at/below 8 records a floor failure.

The direct Cargo leader is observed with WNOWAIT and remains unreaped until dedicated-group cleanup, retaining its PID/group identity. The source requires default SIGCHLD reaping policy. It records numeric exit durably before comparisons, uses bounded PID/group/state-only ps metadata for group members, signals only its verified new group, allows TERM 5 seconds then KILL as needed, and waits/reaps the direct child. It does not claim to reap grandchildren it cannot parent or to prove absence of descendants that escape the owned group. Unknown/remaining cleanup state is a refusal. Explicit SIGINT/SIGTERM requests preserve first failure and reach owned cleanup. No unrelated process, cluster, worker/cache, or provider is touched.

Private receipt/log/resource files are exclusively created mode 0600 under a new mode-0700 owned directory. Existing evidence refuses rather than overwriting. Raw console output stays private; the stdout summary contains only fixed tags, numeric exit and cleanup/persistence/release flags. Atomic/fsynced receipt writes retain actual exit/cleanup/hashes before any grading; failed persistence prevents a passing result. First failure is preserved separately from later cleanup failures. Grading, only after completed receipt capture, requires exit zero and the selected one-test/ok/one-pass markers. Marker booleans expose no raw console text.

`release_required` requests root's immediate scheduler release after direct-child reaping and no remaining owned member, or if no Cargo child was launched. It does not mutate an orchestrator or claim a slot actually released. An unverified/failed reap or remaining-member state is explicit and requires root cleanup/remediation. This supervisor proposal does not reserve, acquire, release, or occupy a runtime slot.

```python
#!/usr/bin/env python3
"""Proposed single CLI-filter supervisor. ARCHIVED ONLY; runtime release required."""
import hashlib
import json
import os
from pathlib import Path
import re
import selectors
import shutil
import signal
import stat
import subprocess
import sys
import time

REPO = Path("/Users/dominik/orca/projects/riAuth-public-preview-sol-diagnostics-wave30")
TARGET = REPO / "target"
EVIDENCE = TARGET / "wave30-cli-startup-filter-35c3"
TOOLCHAIN = Path("/Users/dominik/.rustup/toolchains/1.98.1-aarch64-apple-darwin")
PRODUCT = "35c3fd3007c52d8142c7bee1d69aee127cc42a95"
ALIGNMENT = "22a0e4b7b41266d958c802ea8fdd1e41522df127"
MANIFEST_SHA = "f1decf551c0ad8bf8f613b1c8d8ffba80477f1733bb6180c1011b018070adf47"
CLI_SHA = "4941d65dd62abfb6614be55260a6d14077e5f17bb2126d366f3e54ca57968e59"
GIB = 1024 ** 3
START_FREE, STOP_FREE, FLOOR_FREE = 13 * GIB, 9 * GIB, 8 * GIB
OUTER_NS, CLEANUP_RESERVE_NS = 1200 * 10**9, 20 * 10**9
SAMPLE_NS, MAX_GAP_NS = 2 * 10**9, 5 * 10**9
LOG_CAP, RESOURCE_CAP, SAMPLE_CAP = 8 * 1024**2, 1024**2, 610
META_CAP, RECEIPT_CAP = 2 * 1024**2, 64 * 1024
TEST = "cli_certificate_bind_and_revoke_require_retry_binding"
COMMAND = [
    "env", "CARGO_TARGET_DIR=" + str(TARGET), "CARGO_BUILD_JOBS=1",
    "CARGO_INCREMENTAL=0", "CARGO_PROFILE_DEV_DEBUG=0",
    "CARGO_PROFILE_TEST_DEBUG=0", "cargo", "test", "--locked",
    "--features", "test-support,fuzzing", "--test", "cli", TEST,
    "--", "--exact", "--test-threads=1",
]
BIN_HASHES = {
    "cargo": "6e17e865f3a20dd55a1d212f849f58b77124179f0de7c52973096d84ba34118d",
    "rustc": "766eda9d8f53afd6fc7f27b3cd2e444dd22afacb5afa710a5625fc8e45b8c941",
}
CACHE_HASHES = {
    "debug/.fingerprint/riauth-0d65ce4b80b71950/lib-riauth.json":
        "ac6f00d7f80260b8f443564cbbdd8e64f91d91aa42f8d001c0a23e13778f1c10",
    "debug/.fingerprint/riauth-25aded663d5712f5/bin-riauth.json":
        "375004fd606e5dd7d89a5b0304f6997376e0d1db7ac5e60fb751a811255c952d",
    "debug/.fingerprint/openssl-sys-1830e900e62375b9/lib-openssl_sys.json":
        "899b5b8698182ecf7326104d2cc08b2628783d0290a4b3a4a9c13556d5316ca3",
    "debug/.fingerprint/aws-lc-sys-06c91758ec414101/lib-aws_lc_sys.json":
        "eb05db7ea539f72226bb8b771ab8097a80417150c4c39399f0f33bb9be48d46b",
}
BLOCKED_ENV = (
    "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC", "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER", "RUSTUP_TOOLCHAIN", "RUSTUP_HOME", "CARGO_HOME",
    "CARGO_BUILD_TARGET", "CARGO_BUILD_RUSTFLAGS", "CARGO_BUILD_RUSTC",
    "CARGO_BUILD_RUSTC_WRAPPER", "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
    "OPENSSL_DIR", "OPENSSL_LIB_DIR", "OPENSSL_INCLUDE_DIR",
)

class Refusal(Exception):
    """Only fixed, source-defined tags reach evidence."""
    def __init__(self, tag):
        self.tag = tag
        super().__init__()

def digest_file(path):
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()

def write_all(fd, data):
    view = memoryview(data)
    while view:
        amount = os.write(fd, view)
        if amount <= 0:
            raise Refusal("evidence_write")
        view = view[amount:]

def capture_metadata(argv):
    """Bounded metadata child; never used for Cargo or any product."""
    query = subprocess.Popen(
        argv, cwd=REPO, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL, close_fds=True,
    )
    selector = selectors.DefaultSelector()
    result = bytearray()
    deadline = time.monotonic_ns() + 2 * 10**9
    try:
        os.set_blocking(query.stdout.fileno(), False)
        selector.register(query.stdout, selectors.EVENT_READ)
        eof = False
        while not eof:
            remaining = deadline - time.monotonic_ns()
            if remaining <= 0:
                raise Refusal("metadata_deadline")
            for key, _ in selector.select(min(0.05, remaining / 10**9)):
                chunk = os.read(key.fd, 65536)
                if not chunk:
                    eof = True
                    break
                if len(result) + len(chunk) > META_CAP:
                    raise Refusal("metadata_cap")
                result.extend(chunk)
        remaining = max(0.001, (deadline - time.monotonic_ns()) / 10**9)
        if query.wait(timeout=remaining) != 0:
            raise Refusal("metadata_exit")
        return bytes(result)
    finally:
        selector.close()
        if query.stdout is not None:
            query.stdout.close()
        if query.returncode is None:
            query.kill()
            query.wait(timeout=1)

def protected(path):
    return path.startswith(("src/", "crates/", ".cargo/")) or path in (
        "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "rust-toolchain",
        "build.rs", "rustfmt.toml", ".rustfmt.toml",
    )

class Supervisor:
    def __init__(self):
        self.started_ns = time.monotonic_ns()
        self.deadline_ns = self.started_ns + OUTER_NS
        self.stop_signal = None
        self.child = None
        self.pgid = None
        self.group_verified = False
        self.exit_observed = False
        self.log_fd = self.resource_fd = None
        self.log_selector = None
        self.log_eof = False
        self.log_bytes = self.resource_bytes = 0
        self.sample_count = 0
        self.last_sample_ns = None
        self.next_sample_ns = self.started_ns
        self.target_identity = None
        self.persist_broken = False
        self.evidence_created = False
        self.data = {
            "schema": 1, "product_pin": PRODUCT, "alignment_pin": ALIGNMENT,
            "protected_manifest_sha256": MANIFEST_SHA, "cli_sha256": CLI_SHA,
            "command": COMMAND, "source_head": None, "cargo_started": False,
            "owned_pid": None, "owned_pgid": None, "exit_observed": False,
            "actual_exit": None, "direct_child_reaped": False,
            "remaining_owned_members": None, "first_failure": None,
            "cleanup_failures": [], "persist_failed": False, "grade": None,
            "controller_exit": None, "release_required": False,
            "sample_count": 0, "minimum_free_bytes": None,
            "maximum_sample_gap_ns": 0, "floor_observed": False,
            "log_bytes": 0, "log_eof": False, "log_sha256": None,
            "resource_bytes": 0, "resources_sha256": None,
            "elapsed_ns": None,
        }

    def on_signal(self, number, _frame):
        if self.stop_signal is None:
            self.stop_signal = number

    def save(self):
        if self.persist_broken or not self.evidence_created:
            raise Refusal("evidence_write")
        self.data["sample_count"] = self.sample_count
        self.data["log_bytes"] = self.log_bytes
        self.data["resource_bytes"] = self.resource_bytes
        self.data["elapsed_ns"] = time.monotonic_ns() - self.started_ns
        raw = json.dumps(self.data, sort_keys=True, separators=(",", ":")).encode() + b"\n"
        if len(raw) > RECEIPT_CAP:
            self.persist_broken = True
            self.data["persist_failed"] = True
            raise Refusal("evidence_write")
        fd = None
        try:
            fd = os.open(EVIDENCE / "receipt.pending",
                         os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
            write_all(fd, raw)
            os.fsync(fd)
            os.close(fd)
            fd = None
            os.replace(EVIDENCE / "receipt.pending", EVIDENCE / "receipt.json")
            directory = os.open(EVIDENCE, os.O_RDONLY | os.O_DIRECTORY)
            try:
                os.fsync(directory)
            finally:
                os.close(directory)
        except BaseException:
            self.persist_broken = True
            self.data["persist_failed"] = True
            raise Refusal("evidence_write") from None
        finally:
            if fd is not None:
                os.close(fd)

    def fail(self, tag):
        if self.data["first_failure"] is None:
            self.data["first_failure"] = tag
        self.save()

    def cleanup_error(self, tag):
        if tag not in self.data["cleanup_failures"]:
            self.data["cleanup_failures"].append(tag)
        if self.data["first_failure"] is None:
            self.data["first_failure"] = "cleanup_failure"

    def setup_evidence(self):
        if Path.cwd().resolve() != REPO or REPO.is_symlink():
            raise Refusal("repository_identity")
        if TARGET.is_symlink() or not TARGET.is_dir() or TARGET.resolve() != TARGET:
            raise Refusal("target_identity")
        target_stat = TARGET.stat()
        self.target_identity = (target_stat.st_dev, target_stat.st_ino)
        try:
            os.mkdir(EVIDENCE, 0o700)  # Existing evidence is never overwritten.
        except FileExistsError:
            raise Refusal("evidence_exists") from None
        self.evidence_created = True
        self.log_fd = os.open(EVIDENCE / "console.log",
                              os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
        self.resource_fd = os.open(EVIDENCE / "resources.jsonl",
                                   os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
        self.save()

    def sample(self, initial=False, force=False):
        now = time.monotonic_ns()
        if not initial and not force and now < self.next_sample_ns:
            return
        current = TARGET.stat()
        if TARGET.is_symlink() or (current.st_dev, current.st_ino) != self.target_identity:
            raise Refusal("resource_scope")
        free = shutil.disk_usage(TARGET).free
        gap = 0 if self.last_sample_ns is None else now - self.last_sample_ns
        row = json.dumps({
            "elapsed_ns": now - self.started_ns, "gap_ns": gap,
            "free_bytes": free, "device": current.st_dev,
        }, sort_keys=True, separators=(",", ":")).encode() + b"\n"
        if self.sample_count >= SAMPLE_CAP or self.resource_bytes + len(row) > RESOURCE_CAP:
            raise Refusal("resource_cap")
        write_all(self.resource_fd, row)
        os.fsync(self.resource_fd)
        self.resource_bytes += len(row)
        self.sample_count += 1
        self.last_sample_ns = now
        self.next_sample_ns = now + SAMPLE_NS
        old_minimum = self.data["minimum_free_bytes"]
        self.data["minimum_free_bytes"] = free if old_minimum is None else min(old_minimum, free)
        self.data["maximum_sample_gap_ns"] = max(self.data["maximum_sample_gap_ns"], gap)
        if free <= FLOOR_FREE:
            self.data["floor_observed"] = True
            raise Refusal("resource_floor")
        if free <= STOP_FREE:
            raise Refusal("resource_stop")
        if gap > MAX_GAP_NS:
            raise Refusal("resource_gap")
        if initial and free < START_FREE:
            raise Refusal("start_capacity")

    def preflight(self):
        if len(sys.argv) != 1:
            raise Refusal("controller_arguments")
        if os.name != "posix" or not all(hasattr(os, name) for name in (
            "waitid", "WNOWAIT", "WEXITED", "WNOHANG", "P_PID", "O_NOFOLLOW",
        )):
            raise Refusal("required_interface")
        if signal.getsignal(signal.SIGCHLD) != signal.SIG_DFL:
            raise Refusal("child_reaping_policy")
        if any(os.environ.get(key) for key in BLOCKED_ENV):
            raise Refusal("configuration_override")
        for path in (
            REPO / ".cargo/config", REPO / ".cargo/config.toml",
            Path.home() / ".cargo/config", Path.home() / ".cargo/config.toml",
        ):
            if path.exists():
                raise Refusal("implicit_cargo_configuration")
        head = capture_metadata(["git", "rev-parse", "HEAD"]).strip()
        if re.fullmatch(rb"[0-9a-f]{40}", head) is None:
            raise Refusal("source_identity")
        self.data["source_head"] = head.decode("ascii")
        capture_metadata(["git", "merge-base", "--is-ancestor", ALIGNMENT, "HEAD"])
        for args in (
            ["git", "diff", "--name-only"],
            ["git", "diff", "--cached", "--name-only"],
            ["git", "ls-files", "--others", "--exclude-standard"],
        ):
            if capture_metadata(args):
                raise Refusal("dirty_repository")
        source = capture_metadata(["git", "ls-tree", "-r", "-z", "HEAD"])
        manifest = []
        for row in source.split(b"\0"):
            if row:
                _, path_bytes = row.split(b"\t", 1)
                if protected(path_bytes.decode("utf-8")):
                    manifest.append(row + b"\0")
        if len(manifest) != 350 or hashlib.sha256(b"".join(manifest)).hexdigest() != MANIFEST_SHA:
            raise Refusal("source_manifest")
        # Only the already pinned allowlist is read from the working tree.
        for terminated_row in manifest:
            row = terminated_row[:-1]
            if not row:
                continue
            meta, path_bytes = row.split(b"\t", 1)
            path = path_bytes.decode("utf-8")
            if not protected(path):
                continue
            mode, kind, blob = meta.decode("ascii").split()
            if kind != "blob" or mode not in ("100644", "100755", "120000"):
                raise Refusal("source_manifest")
            local = REPO / path
            local_stat = local.lstat()
            if mode == "120000":
                if not stat.S_ISLNK(local_stat.st_mode):
                    raise Refusal("source_manifest")
                content = os.readlink(local).encode()
            else:
                if not stat.S_ISREG(local_stat.st_mode):
                    raise Refusal("source_manifest")
                if bool(local_stat.st_mode & 0o111) != (mode == "100755"):
                    raise Refusal("source_manifest")
                content = local.read_bytes()
            actual_blob = hashlib.sha1(b"blob " + str(len(content)).encode() + b"\0" + content).hexdigest()
            if actual_blob != blob:
                raise Refusal("source_manifest")
        cli = (REPO / "tests/cli.rs").read_bytes()
        if len(cli) != 118935 or hashlib.sha256(cli).hexdigest() != CLI_SHA:
            raise Refusal("cli_identity")
        for name, expected in BIN_HASHES.items():
            path = TOOLCHAIN / "bin" / name
            if path.is_symlink() or not path.is_file() or digest_file(path) != expected:
                raise Refusal("toolchain_identity")
        for name, expected in CACHE_HASHES.items():
            path = TARGET / name
            if not path.resolve().is_relative_to(TARGET) or digest_file(path) != expected:
                raise Refusal("cache_identity")
        self.sample(initial=True)
        self.save()

    def launch(self):
        if self.stop_signal is not None:
            raise Refusal("controller_signal")
        if time.monotonic_ns() >= self.deadline_ns - CLEANUP_RESERVE_NS:
            raise Refusal("outer_deadline")
        environment = os.environ.copy()
        # Resolve literal "cargo"/"rustc" to the pinned installed binaries, not rustup setup.
        environment["PATH"] = str(TOOLCHAIN / "bin") + os.pathsep + environment.get("PATH", "")
        self.child = subprocess.Popen(
            COMMAND, cwd=REPO, env=environment, stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
            close_fds=True, start_new_session=True,
        )
        self.pgid = self.child.pid
        self.data["cargo_started"] = True
        self.data["owned_pid"] = self.child.pid
        self.data["owned_pgid"] = self.pgid
        self.group_verified = os.getpgid(self.child.pid) == self.pgid
        if not self.group_verified:
            raise Refusal("process_group_identity")
        os.set_blocking(self.child.stdout.fileno(), False)
        self.log_selector = selectors.DefaultSelector()
        self.log_selector.register(self.child.stdout, selectors.EVENT_READ)
        self.save()

    def observe_exit(self):
        # Never poll/reap the leader before group cleanup: WNOWAIT reserves its PID.
        if self.child is None or self.exit_observed:
            return self.exit_observed
        observed = os.waitid(os.P_PID, self.child.pid, os.WEXITED | os.WNOHANG | os.WNOWAIT)
        if observed is None:
            return False
        if observed.si_pid != self.child.pid:
            raise Refusal("exit_identity")
        if observed.si_code == os.CLD_EXITED:
            code = observed.si_status
        elif observed.si_code in (os.CLD_KILLED, os.CLD_DUMPED):
            code = -observed.si_status
        else:
            raise Refusal("exit_identity")
        self.exit_observed = True
        self.data["exit_observed"] = True
        self.data["actual_exit"] = code
        self.save()  # Numeric exit is durable before any expectation comparison.
        return True

    def pump(self, delay=0.05):
        if self.log_selector is None or self.log_eof:
            time.sleep(delay)
            return
        for key, _ in self.log_selector.select(delay):
            for _ in range(8):  # Bounded work per tick; do not starve resource sampling.
                try:
                    chunk = os.read(key.fd, 65536)
                except BlockingIOError:
                    break
                if not chunk:
                    self.log_eof = True
                    self.log_selector.unregister(key.fileobj)
                    return
                room = LOG_CAP - self.log_bytes
                kept = chunk[:room]
                if kept:
                    write_all(self.log_fd, kept)
                    self.log_bytes += len(kept)
                if len(chunk) > room and self.data["first_failure"] is None:
                    self.fail("log_cap")

    def tick(self):
        self.pump()
        self.sample()
        self.observe_exit()

    def run_filter(self):
        while True:
            if self.stop_signal is not None:
                self.fail("controller_signal")
                return
            if time.monotonic_ns() >= self.deadline_ns - CLEANUP_RESERVE_NS:
                self.fail("outer_deadline")
                return
            self.tick()
            if self.exit_observed:
                if self.data["actual_exit"] != 0:
                    self.fail("cargo_exit")
                return
            if self.data["first_failure"] is not None:
                return

    def members(self):
        # Integers/state only, never argv, environment, command text, paths or credentials.
        raw = capture_metadata(["ps", "-axo", "pid=,pgid=,stat="])
        result = []
        for line in raw.splitlines():
            fields = line.split()
            if len(fields) != 3 or not fields[0].isdigit() or not fields[1].isdigit():
                raise Refusal("group_snapshot")
            pid, pgid = int(fields[0]), int(fields[1])
            if pgid == self.pgid and pid != self.child.pid:
                if not re.fullmatch(rb"[A-Za-z+<>-]+", fields[2]):
                    raise Refusal("group_snapshot")
                result.append(pid)
        return result

    def send_owned(self, number):
        if not self.group_verified:
            raise Refusal("process_group_identity")
        try:
            os.killpg(self.pgid, number)
        except ProcessLookupError:
            pass

    def grace(self, seconds):
        until = min(time.monotonic_ns() + int(seconds * 10**9), self.deadline_ns)
        while time.monotonic_ns() < until:
            try:
                self.tick()
            except Refusal as error:
                if self.data["first_failure"] is None:
                    self.data["first_failure"] = error.tag
                self.cleanup_error("cleanup_observation")
            except BaseException:
                self.cleanup_error("cleanup_observation")
            if self.exit_observed:
                # Leader remains unreaped here; descendants are checked at phase boundaries.
                time.sleep(0.02)

    def cleanup(self):
        if self.child is None:
            self.data["remaining_owned_members"] = 0
            self.data["release_required"] = True
            return
        try:
            self.observe_exit()
            remaining = self.members() if self.group_verified else None
            if not self.exit_observed or remaining:
                if self.data["first_failure"] is None:
                    self.fail("owned_descendants")
                if self.group_verified:
                    self.send_owned(signal.SIGTERM)
                    self.grace(5)
                    remaining = self.members()
                    if not self.exit_observed or remaining:
                        self.send_owned(signal.SIGKILL)
                        self.grace(3)
                        remaining = self.members()
                else:
                    self.cleanup_error("process_group_identity")
                    self.child.kill()
            self.data["remaining_owned_members"] = None if remaining is None else len(remaining)
            if remaining:
                self.cleanup_error("owned_members_remain")
        except BaseException:
            self.cleanup_error("group_cleanup")
            # The unreaped direct child still reserves the verified group identity.
            try:
                if self.group_verified:
                    self.send_owned(signal.SIGKILL)
                else:
                    self.child.kill()
            except BaseException:
                self.cleanup_error("kill_failure")
        try:
            self.observe_exit()
            actual = self.child.wait(timeout=2)
            self.data["direct_child_reaped"] = True
            if not self.exit_observed:
                self.data["actual_exit"] = actual
                self.data["exit_observed"] = True
            elif actual != self.data["actual_exit"]:
                self.cleanup_error("exit_disagreement")
        except BaseException:
            self.cleanup_error("direct_child_wait")
        self.data["release_required"] = (
            self.data["direct_child_reaped"] and self.data["remaining_owned_members"] == 0
        )

    def finalize_files(self):
        if self.child is not None and self.log_selector is not None:
            until = min(time.monotonic_ns() + 10**9, self.deadline_ns)
            while not self.log_eof and time.monotonic_ns() < until:
                self.pump()
            if not self.log_eof:
                self.cleanup_error("log_incomplete")
        if self.log_selector is not None:
            self.log_selector.close()
        if self.child is not None and self.child.stdout is not None:
            self.child.stdout.close()
        for fd in (self.log_fd, self.resource_fd):
            if fd is not None:
                os.fsync(fd)
                os.close(fd)
        self.log_fd = self.resource_fd = None
        self.data["log_eof"] = self.log_eof
        if self.evidence_created:
            self.data["log_sha256"] = digest_file(EVIDENCE / "console.log")
            self.data["resources_sha256"] = digest_file(EVIDENCE / "resources.jsonl")

    def grade(self):
        if self.data["first_failure"] is not None or self.persist_broken:
            return
        if self.data["actual_exit"] != 0 or not self.data["direct_child_reaped"]:
            self.fail("cargo_exit")
            return
        if self.data["remaining_owned_members"] != 0 or self.data["cleanup_failures"]:
            self.fail("cleanup_failure")
            return
        raw = (EVIDENCE / "console.log").read_bytes()
        checks = {
            "running_one": len(re.findall(rb"(?m)^running 1 test\r?$", raw)) == 1,
            "selected_ok": len(re.findall(
                rb"(?m)^test " + TEST.encode() + rb" \.\.\. ok\r?$", raw)) == 1,
            "summary_one": len(re.findall(
                rb"(?m)^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; "
                rb"[0-9]+ filtered out; finished in [0-9.]+s\r?$", raw)) == 1,
        }
        self.data["grade"] = checks
        if not all(checks.values()):
            self.fail("filter_result")

    def finish(self):
        if self.stop_signal is not None and self.data["first_failure"] is None:
            self.data["first_failure"] = "controller_signal"
        try:
            self.cleanup()
        except BaseException:
            self.cleanup_error("cleanup_exception")
        try:
            self.sample(force=True)
        except Refusal as error:
            if self.data["first_failure"] is None:
                self.data["first_failure"] = error.tag
        except BaseException:
            if self.data["first_failure"] is None:
                self.data["first_failure"] = "resource_unavailable"
        try:
            self.finalize_files()
        except BaseException:
            self.cleanup_error("file_finalize")
        if time.monotonic_ns() > self.deadline_ns and self.data["first_failure"] is None:
            self.data["first_failure"] = "outer_deadline"
        if self.stop_signal is not None and self.data["first_failure"] is None:
            self.data["first_failure"] = "controller_signal"
        try:
            self.save()  # Actual exit + cleanup + hashes are retained before grading.
            self.grade()
        except BaseException:
            if self.data["first_failure"] is None:
                self.data["first_failure"] = "evidence_or_grade"
        self.data["controller_exit"] = 0 if (
            self.data["first_failure"] is None and not self.persist_broken
            and self.data["grade"] is not None and all(self.data["grade"].values())
        ) else 1
        try:
            self.save()
        except BaseException:
            self.data["controller_exit"] = 1
        summary = {
            "controller_exit": self.data["controller_exit"],
            "first_failure": self.data["first_failure"],
            "actual_exit": self.data["actual_exit"],
            "direct_child_reaped": self.data["direct_child_reaped"],
            "cleanup_failed": bool(self.data["cleanup_failures"]),
            "persist_failed": self.persist_broken,
            "release_required": self.data["release_required"],
        }
        try:
            os.write(1, json.dumps(summary, sort_keys=True).encode() + b"\n")
        except BaseException:
            pass
        return self.data["controller_exit"]

def main():
    os.umask(0o077)
    supervisor = Supervisor()
    signal.signal(signal.SIGINT, supervisor.on_signal)
    signal.signal(signal.SIGTERM, supervisor.on_signal)
    try:
        supervisor.setup_evidence()
        supervisor.preflight()
        supervisor.launch()
        supervisor.run_filter()
    except Refusal as error:
        if supervisor.data["first_failure"] is None:
            supervisor.data["first_failure"] = error.tag
        try:
            supervisor.save()
        except BaseException:
            pass
    except BaseException:
        if supervisor.data["first_failure"] is None:
            supervisor.data["first_failure"] = "controller_exception"
        try:
            supervisor.save()
        except BaseException:
            pass
    return supervisor.finish()

if __name__ == "__main__":
    raise SystemExit(main())
```

AST-only validation passed: complete source parse, required preflight/launch/exit/cleanup/grade/finish definitions, no leader poll call, new-session group creation, WNOWAIT observation, bounded metadata/direct waits, TERM/KILL controls, terminal sampling, and receipt save preceding grade. These checks do not execute any source, prove runtime ownership/cleanup, validate log markers against a future run, or establish capacity/build/test success.

Actual report-aware checks passed: the full 40568-byte e4 and complete published prefixes, exact fenced controller size/hash/AST, closed literal refusal tags, CLI/D01 identities, report-only working scope, trailing-whitespace/final-LF, `git diff --check` exit `0`, and `python3 scripts/check-docs.py` exit `0`. No proposed runtime evidence directory exists. Final staged equality/whitespace and post-commit whole protected manifest/selected source/blob/clean checks complete the separate report handoff. Root must fully review this exact supervisor text before separately owning materialization and releasing ONE same-filter runtime. Historical aborted merge, CI56 failed/unknown child cause and earlier-source b71 run37063876066 SUCCESS remain dated. No Linux-fixed/current-all-green, rare-race, original A09 shared/artifact, D01, or closed-row status credit follows from this alignment or source proposal.


## Root complete supervisor source review (2026-10-03)

Root fully read the01cc8e9 preparation/alignment evidence and complete638-line supervisor27303B/SHA1be75053e7960de6bb1568cffcd1c092a554dbeb0de2ca63e0231385cd61f695. Protected350-entry manifest and CLI4941d65 identity were already independently matched to fixed35. Runtime remains held pending a narrow final post-save deadline check before stdout/return; no command, source correction or runtime result is inferred. The proposed process group uses an unreaped leader through WNOWAIT cleanup, actual numeric exit retained before grading, bounded private logs/resources, fresh13GiB start and9GiB stop/8GiB floor. Existing checks are source facts, not execution proof or a guarantee against unseen disk values/kernel I/O. No old branch merge is imported by integrating only these exact report appendices.
