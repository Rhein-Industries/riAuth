# O06 remote-signer failure attribution: proposal

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, O06 task
`9899f8e6-05ff-4e11-b9a0-b9ca184221a6`, worktree
`9c54c023-a95f-48e9-a908-6f97a5677c98`. Fixed main is
`755a7763e0e2aa7e4d5c92d18c432ebc0c8a3e87`.

This is a read-only proposal for the smallest seam behind finding 4 of the
[independent review](local-wave30-o06-independent-review.md). Lead: Claude Opus
5.5. All source was read from Git objects at the fixed main; nothing was built,
tested or edited. No production or test file changes until root approves the
reservation below and separately releases a Cargo slot.

## What the operator sees today

O06 asks that key problems be exposed so that an operator can tell which
component is failing, what remains safe, and what corrective action is
required.

For a remote (Vault Transit) signing key, every failure in `Core::sign_jwt`
adds one to the unlabeled `riauth_signing_errors_total`
(`src/kms.rs:98-108`, `src/telemetry.rs:350,479`). That counter does not say
which step failed. `src/kms.rs` has no log statement of its own.

The public errors are not all `signer_unavailable`. Today they are:

| Failure site (`src/kms.rs`) | Public error today | Log today |
| --- | --- | --- |
| Stored remote metadata invalid: `key.jwk()?` (`:113`). `algorithm()?` and `decoding_key()?` (`:129,135`) run the same validation that `:113` already passed, so they cannot fail after it | 400 `invalid_request` "External key metadata mismatch", or the JWK validation error | none |
| No configured signer with the stored name, version and pin (`:114-119`) | 503 `signer_unavailable` | none |
| Configured signer fails `validate()` at sign time (`:35`) | 400 `invalid_request` (address, version or namespace text) | none |
| Token file not readable as a private file of at most 4096 bytes (`:36-38`) | 400 "Vault credential must be a private file of at most 4096 bytes" | none |
| Token empty, too long or not ASCII graphic (`:40-41`) | 400 "Invalid Vault credential file" | none |
| `ca_file` unreadable (`:48`) | 500 `server_error` | `Error::internal` logs the raw error ("operation failed") |
| `Certificate::from_pem` (`:48-49`) | Cannot fail in this build. reqwest 0.13.5 is built with rustls only, and `from_pem` just stores the bytes (reqwest `src/tls.rs:191-198`) | none |
| HTTP client build (`:52`), which is where a malformed PEM block in `ca_file` is parsed and refused. A file with no PEM block adds no certificate and does not fail | 503 `signer_unavailable` | none |
| Request not sent: connect, DNS, TLS, timeout, reset, or a request-build error (`:63`) | 503 | none |
| Non-success status, or Content-Length over 64 KiB (`:64-67`) | 503 | none |
| Body read failed (`:68-72`) | 503 | none |
| Body over 64 KiB (`:73-75`) | 503 | none |
| Body not JSON (`:76`) | 503 | none |
| `data.signature` missing, or a `vault:v<N>:` prefix that is not the pinned version (`:77-80`) | 503 | none |
| Signature not Base64 (`:82-86`) | 503 | none |
| Returned signature fails verification against the pin, or the verified claims differ (`:134-139`) | 503 | none |
| Header or claims serialization (`:120-124`); `serde_json::to_vec` of a `Value` does not fail in practice | 500 `server_error` | `Error::internal` log |

Essentials refuses a stored remote key with a 503 `signer_unavailable`
"Remote signing key requires the Platform build; no token was issued"
(`src/kms_essentials.rs:13-36`).

There is no key availability report at `755a776`. The only "not measured"
report is the storage-pressure decorator in
`src/operations/storage_diagnostics.rs`. Nothing at the fixed main
attributes a remote signing failure to a component.

## Proposal

Classify each existing remote failure site into one fixed reason. Count it in
one bounded Prometheus counter and the runtime JSON, and log the reason
token alone.

**Unchanged:**

- every public status, code and message, byte for byte, including the
  `signer_unavailable` 503 body;
- fail-closed verification and response bounds;
- `riauth_signing_errors_total` and its alert;
- every writer, configuration key, schema and API route;
- dependencies.

### Reason labels

| Label | Site | Already true | Operator action (for the docs table) |
| --- | --- | --- | --- |
| `stored_key` | Stored remote metadata is inconsistent (`:113`); the later pin uses at `:129` and `:135` map here too but are unreachable | No token is issued. JWKS and verification do not call Vault. Nothing was sent. | Do not edit the store by hand. Restore from a verified backup, or bind a new key through `riauth keys bind` after review. |
| `configuration_binding` | No configured signer with the stored signer name, `key_version` and `public_jwk` | Same | Restore that signer entry on this node exactly as bound. A different version or pin needs a reviewed new bind, not a pin edit. |
| `signer_configuration` | The configured signer fails validation at sign time | Same | Correct the signer entry (address scheme, mount, key name, version, namespace). |
| `credential_read` | Token file missing, not an owner-only regular file, or larger than 4096 bytes | Same; nothing was sent | Fix the file's path, owner, mode or size. |
| `credential_shape` | Token empty after trimming, too long, or not ASCII graphic | Same; nothing was sent | Rewrite the token file with the Vault token only. |
| `ca_setup` | Configured `ca_file` could not be read (`:48`) | Same; nothing was sent | Check the CA file's path and permissions. |
| `client_setup` | HTTP client could not be built (`:52`), for example because of a malformed PEM block in `ca_file` | Same; nothing was sent | Check the CA file's contents. |
| `transport` | The request was not sent, or its response or body was not received: connect, DNS, TLS, the 3-second timeout, reset, a request-build error, or a failed body read | Same | Check Vault reachability, the address and TLS from this node. |
| `http_3xx` | The answer was a redirect, which is not followed | Same | Check which service answers at `address`; redirects are never followed. |
| `http_4xx` | The answer was a 4xx status | Same | Check the token's policy, namespace, mount and key name. |
| `http_5xx` | The answer was a 5xx status | Same | Check the health of the service at `address`. |
| `http_other` | Any other non-success status (1xx or 600 and above) | Same | Check which service answers at `address`. |
| `response_size` | Declared or actual body over 64 KiB | Same | Check that `address` and `mount` reach the Transit sign endpoint. |
| `response_shape` | Body not JSON, no `data.signature` string, a signature without a `vault:v<digits>:` prefix, or a signature that is not Base64 | Same | Check that `address` and `mount` reach the Transit sign endpoint. |
| `response_version` | `data.signature` starts with `vault:v`, one or more ASCII digits and `:`, but not the pinned `vault:v<key_version>:` | Same | Compare the Vault key's versions with the configured `key_version`. Re-pinning needs a reviewed bind. |
| `signature_verification` | The returned signature does not verify against the pinned public key, or the verified claims differ | Same; the token was refused | Check that `key_name`, `mount` and `key_version` address the pinned key. Do not re-pin to make it pass without review; see [credential compromise](../credential-compromise.md). |
| `encoding` | Header or claims serialization failed; not reachable in practice | Same | Report it. |
| `edition_unsupported` | Essentials build with a stored remote key | Same | Run the Platform build, or bind a local key. |

The labels are fixed. They carry no key id, signer or domain name, URL,
namespace, raw error, token, claims, path, status code or body. One label
is chosen per failure; the HTTP status gives only its class.

### Exact hunks

**`src/telemetry.rs`:**

- Add `pub enum RemoteSigningFailure` with the 18 variants above, plus
  `ALL` and `label()`, following `ReadContext` and `Activity`.
- Add `pub fn status(status: reqwest::StatusCode) -> Self` mapping
  `is_redirection`, `is_client_error` and `is_server_error` to the three
  `http_*` classes and anything else to `http_other`. It is `pub`, not
  `pub(crate)`: Essentials never calls it, and `-D warnings` would reject an
  unused crate-private function.
- Add the field
  `pub remote_signing_failures: [AtomicU64; RemoteSigningFailure::ALL.len()]`,
  so the list and the array cannot drift. `Default` covers arrays of this
  size, as for `Reads`.
- Add `pub(crate) fn remote_signing_failed(&self, reason)`. It increments
  that slot and runs
  `tracing::warn!(reason = reason.label(), "remote signing failed")`, the
  same shape as `note_alert_failure` (`src/operations.rs:1126-1132`).
- `snapshot()` gains `"remote_signing_failures": {label: count}` for all 18
  labels.
- `render()` always writes `# TYPE riauth_remote_signing_failures_total
  counter`, then one `{reason="..."}` line only for each reason observed in
  this process. The `activity` histograms already do this
  (`src/telemetry.rs:441-442`). Three reasons for the choice:
  - `tests/contention.rs` `assert_well_formed` (`:828-926`, call at `:1031`)
    panics on any label name it does not allow, so always-written lines
    would break that CI test;
  - no zero series stands in for health;
  - the loop is `for reason in RemoteSigningFailure::ALL`, not the
    `for (name, x) in [...]` shape that `scripts/check-grafana-dashboard.py`
    parses.

**`src/kms.rs`** (Platform):

- `VaultSigner::sign` returns
  `std::result::Result<Vec<u8>, (RemoteSigningFailure, Error)>`. Each site
  pairs its existing `Error` value with its label. The expressions stay as
  they are, so `Error::internal`'s existing CA-file log is unchanged and
  nothing new is logged there.
- The status check splits into "not success" (labelled by class) and
  "Content-Length over 64 KiB" (`response_size`). Both still return
  `unavailable()`.
- The signature prefix check keeps its exact acceptance rule. On refusal,
  the label is `response_version` when the string matches
  `vault:v<one or more ASCII digits>:` but is not the pinned prefix, and
  `response_shape` otherwise. Both still return `unavailable()` and refuse.
- The CA arm keeps both `map_err(Error::internal)` calls and pairs them
  with `ca_setup`. The `from_pem` arm cannot fire. The client build pairs
  with `client_setup`.
- `sign_jwt_inner` returns
  `std::result::Result<String, (Option<RemoteSigningFailure>, Error)>`.
  The local branch maps to `(None, error)`, so local keys are never
  labelled.
- `sign_jwt` keeps its timer and `signing_errors` increment. On a labelled
  error it also calls `remote_signing_failed`, then returns the same
  `Error`.
- `external_signing_key` is unchanged. Its test signature goes through
  `sign_jwt`, so a failed bind is attributed too.

**`src/kms_essentials.rs`:** on the remote branch, also call
`remote_signing_failed(EditionUnsupported)`. The error stays identical.

**Docs:**

- `docs/kms.md`: a "Failure reasons" table built from the columns above.
- `docs/connector-incidents.md` (Vault Transit section): one paragraph naming
  the metric and runtime field, with the limits below.
- The paragraph in `docs/operations.md` that lists the runtime counters:
  one sentence naming the new counter.

## One focused test target

`tests/o06_remote_signing_attribution.rs`, gated
`#![cfg(feature = "platform")]`. It uses `tests/common`'s `Fixture` and is not
in the CI-owned `tests/identity` family. The local Transit fake is copied as a
pattern, not imported, from `tests/identity/operations.rs:5050`. It is an axum
router on `127.0.0.1:0` that signs RS256 with `openssl` and has a mode switch.

1. Bind one remote RS256 key through `configure_key` with a working fake, and
   issue one token, as the baseline. Use a distinctive signer and Vault key
   name that appears in no metric name. Run `configure_key` and `core.token`
   in `spawn_blocking`, because `reqwest::blocking` cannot run on an async
   thread.
2. For each case below:
   - assert the public error's status, `code` and `message` are exactly
     today's;
   - assert `remote_signing_failures` grew by one for that label only, and
     `signing_errors` by one;
   - assert the full store snapshot is unchanged, so no code was consumed. A
     failed exchange returns before the prepared write commits;
   - restore the fixture and assert the same exchange now issues a token.
3. Cases:
   - fake returns 302 / 403 / 503;
   - a body over 64 KiB;
   - non-JSON body; missing `data.signature`; `vault:v8:` prefix; non-Base64
     signature; zeroed signature;
   - address on a closed loopback port;
   - token file missing; token file mode 0644 (both 400 `credential_read`);
     token file containing a control byte (400 `credential_shape`);
   - `ca_file` missing (500 `server_error`, `ca_setup`);
     `ca_file` holding `-----BEGIN CERTIFICATE-----\n!!!\n-----END CERTIFICATE-----`
     (503 `signer_unavailable`, `client_setup`);
   - configured `key_version` changed (503, `configuration_binding`);
     configured namespace empty (400 "Vault requires an explicit
     key_version and valid namespace", `signer_configuration`);
   - stored remote key `kid` changed by one `store.write` (400 "External key
     metadata mismatch", `stored_key`), then restored.
4. A failed bind (fake 503) adds `http_5xx` and stores no key.
5. Exposition and runtime JSON:
   - in-process `GET /api/operations/prometheus` shows the `# TYPE` line;
   - it has a `{reason=...}` line for each observed label and none for
     unobserved ones;
   - the runtime JSON key set equals the 18 labels;
   - neither contains the token, address, signer name or Vault key name.

The client-build failure other than the PEM case, the unreachable pin sites
and serialization are covered by the mapping review, not by this target.

## Limits to document

- **Process-local counters.** They reset at restart. A reason's series
  first appears at its first failure, so `increase()` over a window that
  includes that moment can miss the first count.
- **Zero is not health.** A zero count only means no remote signing attempt
  failed in this process since it started. It does not prove the signer
  works.
- **Worker processes.** A worker has no metrics route, so for SSF and
  logout signing on a worker the warn line is the only signal.
- **SAML XML signing** does not use `sign_jwt` and stays uncounted.
- **No live checks.** There is no live Vault probe, cross-node signer
  comparison, or inventory of signers or domains.

## Reservation requested before any edit

| File | Hunk |
| --- | --- |
| `src/telemetry.rs` | Enum, field, `remote_signing_failed`, snapshot key, render block |
| `src/kms.rs` | Labelled error pairs in `VaultSigner::sign`, `sign_jwt_inner` and `sign_jwt`; public errors byte-equivalent |
| `src/kms_essentials.rs` | One `remote_signing_failed(EditionUnsupported)` call |
| `tests/o06_remote_signing_attribution.rs` | New target as above |
| `docs/kms.md`, `docs/connector-incidents.md` (Vault Transit), `docs/operations.md` (one sentence) | As above |

**Alignment first.** Before editing, merge main `755a776` into this branch
with history preserved, keeping main's accepted wording for the seven
remedy docs and root's availability paragraph.

**Checks, once root releases the slot.** All with a private target,
`jobs 1`, no incremental builds and debug info off:

- the new target;
- `tests/operations.rs alert_webhook`, which writes `signing_errors`;
- the essentials `cargo check --no-default-features --features essentials --lib`;
- clippy with `-D warnings`, `fmt`, `check-docs`.

With observed-only rendering, `tests/contention.rs` needs no change. If root
prefers always-written zero series instead, add `tests/contention.rs` to the
reservation: one `("reason", Some(&[the 18 labels]))` entry in its allowed
label map.

Implementation would use a Sonnet implementer, and an independent Sonnet
review would inspect the hunks and the evidence.

## Independent review of this proposal

A read-only Sonnet reviewer (reported `claude-sonnet-5-5`, Git objects at
`755a776` plus the local reqwest 0.13.5 source; nothing built or run)
confirmed:

- every failure site maps to exactly one label;
- each public error can be kept byte-identical, including the existing
  `Error::internal` CA log, which carries no path;
- the telemetry fit, Essentials compilation, test mechanics and per-case
  public errors all hold;
- no test pins the runtime key set.

The lead checked the source for both high findings before applying them.
Corrections applied:

1. **Contention allow-list.** Always-written series would break the CI test
   `tests/contention.rs`. Fixed by observed-only rendering.
2. **CA parsing.** `ca_file` "not PEM" does not fail as `ca_setup` 500 under
   rustls. It fails at client build as 503 `client_setup`, and only when the
   PEM block is malformed. A file with no PEM block adds no certificate and
   does not fail. The rows and test cases were corrected.
3. **Cause speculation.** The `signature_verification`, `http_3xx` and
   `http_5xx` actions asserted causes. They now say what to check.
4. **Reachability.** The unreachable pin sites and `encoding` are marked as
   such. `transport` also covers request-build errors. The
   `response_version` predicate is now explicit.
5. **Visibility and drift.** `status` is `pub`, and the array is sized from
   `ALL`.
6. **Low items.**
   - The `note_alert_failure` pin was corrected.
   - The test now uses distinctive names and a `stored_key` case.

**Not measured.** The reviewer estimated the Prometheus body at about 25 KB
against the 32 KB cap in `tests/identity/http.rs:402`. Observed-only
rendering adds nothing there.

## Implementation and runtime evidence

Root reserved the seven files in the table above
(`wave30_O06_remote_signer_attribution`), reviewed the cumulative delta, and
released one runtime slot for the new target only.

| Commit | Content |
| --- | --- |
| `dec83785b2e7f0934ecf3665af044174d95b2854` | History-preserving merge of main `755a7763e0e2aa7e4d5c92d18c432ebc0c8a3e87`. The only conflict, `docs/availability.md`, took main's accepted wording. |
| `9ea682d612425bd639d2b0802fa20554aef69786` | The seven reserved files. |
| `552fb42f01d11d9bd91d4e02331aebb0710eb197` | Review fixes. A compile-time guard ties `COUNT` to the last variant. The `docs/kms.md` wording now says only a bind whose test signature fails is counted, and `credential_read` also covers a token file that cannot be read as text. |

### As implemented

- **Telemetry:**
  - an 18-label `RemoteSigningFailure` enum with a `COUNT` guard;
  - `remote_signing_failures` counters;
  - `Telemetry::remote_signing_failed`, which logs
    `tracing::warn!(parent: None, reason = label, "remote signing failed")`;
  - a runtime JSON field listing every label;
  - Prometheus with an always-present `# TYPE` line and observed-only
    reason series.
- **Errors:** every failure site keeps its old `Error` value at the same
  point. Local-key errors are unlabelled. Essentials records
  `edition_unsupported`.

### Runtime

The released command was run exactly as given, on committed `552fb42` with a
clean tree. `CARGO_TARGET_DIR` was this worktree's `target/wave27`.

```sh
env CARGO_TARGET_DIR="$PWD/target/wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support --test o06_remote_signing_attribution -- --test-threads=1
```

| Item | Result |
| --- | --- |
| Exit | 0 |
| Tests | 1 passed, 0 failed, 0 ignored, 0 filtered out (`every_remote_signing_failure_keeps_its_public_error_and_adds_one_fixed_reason`), 2.16 s |
| Build | `Compiling riauth v0.1.1`, `Finished test profile in 1m 04s`. The wall time from 2026-10-02T11:14:26Z to 11:15:34Z includes it. |
| Warnings | One existing macOS linker note for the `riauth` binary (`__eh_frame section too large`). No compiler warning came from the library or the test. |
| Log | 14 lines, SHA-256 `7ff0f0a1ff5f95c441ec14d14392a5f010399477b2eb5e698744c26a898f0fe8`, kept in the session scratchpad |
| Disk | Free space stayed at 14 GiB or more, against the 8 GiB floor, with a 20-second monitor. No cache was deleted. |
| Tree | Unchanged after the run, and equal to `552fb42` |

That one test passed every case listed under "One focused test target"
above:

- a failed bind (`http_5xx`) with an unchanged snapshot;
- each Transit answer case;
- the closed port; the three token-file cases; the two CA cases;
- the configuration binding, signer configuration and stored-key cases;
- the parentless reason-only warning, captured inside a marked span, with no
  marker, token, name, address or status;
- Prometheus observed-only series and runtime JSON with all 18 labels.

Each failure case checked the exact public error, one label delta, one
`signing_errors` delta, an unchanged full store snapshot, and a successful
retry of the same exchange.

### Protected files at `552fb42`

| Git blob | SHA-256 | File |
| --- | --- | --- |
| `3a6f02efcef7fff5e50065285570901e8616715b` | `d23e9a03f2129601a1a37ace5ad80b0473bbc63a6ddb47be75728882fb0e12fd` | `src/telemetry.rs` |
| `9fd987eda0df672aa3e58ee3ad7cfddda1d3b699` | `33c28b5e0fc65d4cecf7b0669f4d07ec9b4ce24d5f48b131529689dc2f904879` | `src/kms.rs` |
| `3acb4be056709fe284f5f16dfd858e8c1af9f975` | `3a4b53f68700bb2545c640367bd057ccfa7819b7541d17b2a075f6dad50ce433` | `src/kms_essentials.rs` |
| `31b687fa545edce4ba244a5393e596d347813a1f` | `e223982d230a48ba28c45b08464fbb5163e3b3831586a53c6fdfa49075a0b927` | `tests/o06_remote_signing_attribution.rs` |
| `c9e373afc02d4564aa3bb5866677d6e2781bdbe2` | `86a602010279837251b2dd84fb56f89728a540a2aed73aee75f46a505038e940` | `docs/kms.md` |
| `309e26e3870b9a2154449f7acf2657bcae0c34d7` | `cd76a44dfa0a9d9e22cd61ba37f3e9646b1e1c4421f38c4f8e3f14f0656d4218` | `docs/connector-incidents.md` |
| `6b1d668f95493ebbea983e458195304856ccaa6b` | `1ad31625b1417a708460090cf121eca01129351cd8c104a817203bfa70f688ec` | `docs/operations.md` |

### Static checks and review

- **Before runtime:** `cargo fmt --all -- --check`,
  `scripts/check-docs.py`, `scripts/check-grafana-dashboard.py` and
  `git diff --check` passed.
- **Review:** a read-only Sonnet review of `9ea682d` (reported
  `claude-sonnet-5-5`, nothing compiled) found no blocking defect. Its
  24-site public-error equivalence table was all equal. Its three low
  findings are fixed in `552fb42`.

### Not run, and remaining gates

Root has not yet released these:

- the Essentials library check for the `kms_essentials.rs` hunk and the
  `reqwest::StatusCode` use in `telemetry.rs`;
- clippy with `-D warnings`;
- `tests/operations.rs` `alert_webhook`;
- `tests/contention.rs`.

The test target does not cover `edition_unsupported`, `http_other` or
`encoding`, nor that local-key errors stay unlabelled; those rest on code
reading. Nothing here touches a real Vault.

O06 stays open.
