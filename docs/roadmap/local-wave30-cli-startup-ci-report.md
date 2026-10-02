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
