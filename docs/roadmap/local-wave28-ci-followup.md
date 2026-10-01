# Local wave 28 CI follow-up: runs 36937149832 and 36936535723

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, worktree
`f1d9035f-514b-4364-9abc-208762c6a933`, task
`9899f8e6-05ff-4e11-b9a0-b9ca184221a6`, which stays in progress.

The branch merged reviewed main `c540ef4` as `eb05123`, preserving history.
Both conflicts were resolved by taking main's version byte for byte:
- `tools/browser/setup.spec.js`, which keeps root's `#setup-password` from `4bdd3be`;
- this lane's wave 28 report, which keeps root's integration note.

The O06 diagnostics work is unchanged. I also stopped a stray read-only
`bfs /` search left running by this lane's reviewer, along with its shells.

## Commits

| Commit | Fix | Files |
| --- | --- | --- |
| `01348ff` | Provider schema pinned per edition in the client config boundary | `tests/client_config_boundary.rs` |
| `505469f` | The same expectation for the two sibling tests that fail next. These files are outside this lane's assigned ownership, so root can take or drop this commit. | `tests/jwk_boundary.rs`, `tests/provider_settings_boundary.rs` |
| `ce80088` | Firefox COOP context swap disabled in the Playwright test browser | `tools/browser/playwright.config.js` |
| this commit | This report | `docs/roadmap/local-wave28-ci-followup.md` |

## Provider schema digest

`populated_configuration_json_and_schemas_match_baseline` hashes
`riauth::schema::schema("provider")`, which is
`schemars::schema_for!(ProviderSettings)`. The current digest is `31f187d5…`
and the baseline is `6f68906a…`, recorded in `30ed02f` and copied into
`f8c3602` and `6c98c99`.

**The delta.** It comes entirely from `cf999a6`, "Add Platform conditional
application policy and claim projection" (W04). That commit adds the
Platform-only field `Policy.conditional`. In the schema this adds:
- `$defs.Policy.properties.conditional`, which is optional and nullable (`anyOf` a `$ref` and `null`), not required, with no default; `additionalProperties: false` is kept;
- the new `$defs` `ConditionalPolicy`, `ConditionalClaimMapping`, `Predicate` (nine recursive variants), `AuthenticationProof` and `AssuranceLevel`.

No field was removed, renamed or retyped, and no default changed. The `settings_json`, `client_json` and `client_view_json` digests still match, because an absent `conditional` serializes to nothing. `docs/oidc-profiles.md` documents the setting, including that Essentials does not accept or advertise it.

**The proof.**
- **Reconstructing the baseline.** The current schema bytes are compact JSON with sorted keys, since serde_json has no `preserve_order`. With exactly the paths above removed, they hash to `6f68906a…` byte for byte.
- **Essentials still matches.** A `--no-default-features --features essentials` build emits `6f68906a…`.

**The fix.** The test now pins Platform at `31f187d5…` and Essentials at the
original `6f68906a…`, so neither edition loses its check.

**Sibling tests.** `jwk_boundary::jwk_stream_and_provider_bytes_and_schemas_match_baseline` and
`provider_settings_boundary::existing_provider_schemas_remain_byte_stable`
fail on the same digest. CI did not reach them only because `cargo test` stops
at the first failing test binary. `505469f` applies the same expectation to
both. The other ten provider-settings schema digests are unchanged.

## Firefox page.goto never resolving

**What failed.**
- Run 36937149832 hung at `authenticator-recovery.spec.js:195`: a new page's first `goto` to `/identity/apps`.
- Run 36936535723 hung in `setup.spec.js` on `goto` to `/setup` after `/apps`.
- Chromium and WebKit passed the same steps.
- The runs uploaded no artifacts, because `ci.yml` has no upload step and `setup.spec.js` disables tracing on purpose. Only the logs were available.

**The cause.** Playwright 1.63.0, with Firefox build 1543, can lose
`Page.navigationCommitted` when Firefox swaps browsing contexts for a
`Cross-Origin-Opener-Policy: same-origin` document. `goto` then never
resolves under any `waitUntil`, although the page has loaded.
- This is microsoft/playwright#42731, closed 2026-09-22.
- 1.63.0 is still the latest published `@playwright/test`.
- `portal_html` (`src/portal/http.rs`) deliberately sends COOP on portal pages. The standalone 409 and error pages do not.

**The evidence.** I reproduced it locally against the real portal fixture,
with `/apps` (COOP `same-origin`, 200) followed by `/setup` (409, no COOP):

| Run by | Default prefs | With `ce80088` |
| --- | --- | --- |
| Lane | 54 of 200 navigations hung | 0 of 200 |
| Orchestrator | 7 of 80 hung | 0 of 80 |

Every hung page was already at `readyState: complete`. The recovery-spec site,
a first navigation from `about:blank`, did not reproduce locally (0 in 300+
trials). Upstream reports it on other platforms. I attribute it to the same
mechanism from the identical symptom; that attribution is not proven locally.

**The fix.** `ce80088` sets the Firefox preference
`browser.tabs.remote.useCrossOriginOpenerPolicy=false`, for the Firefox
Playwright project only.
- The server still sends the header, and `tests/portal.rs` asserts it.
- No spec, selector or assertion changed, and there is no timeout increase or retry.

**Checks with the fix.**
- `authenticator-recovery.spec.js --project=firefox --repeat-each=3`: 3 of 3 passed.
- `setup.spec.js --project=firefox`: 3 of 3 passed.
- `authenticator-recovery.spec.js` on chromium and webkit: passed.
- Remove the preference once Playwright is 1.64 or later.

## Checks actually run

| Command | Result |
| --- | --- |
| `cargo test --locked --features test-support,fuzzing --test client_config_boundary --test jwk_boundary --test provider_settings_boundary` | before `505469f`: client config 5 passed, the two siblings FAILED on `provider_schema`; after: 5, 3 and 3 passed |
| `cargo clippy --locked --features test-support,fuzzing --test client_config_boundary --test jwk_boundary --test provider_settings_boundary -- -D warnings`; rustfmt | clean |
| Lane: `--no-default-features --features essentials,test-support --test client_config_boundary populated_configuration` | passed, with the Essentials digest `6f68906a…` |
| The COOP stress loop and the focused Playwright runs above | as stated |

**Not run:**
- the full `cargo test --all-targets`;
- the other CI steps;
- Playwright specs other than `authenticator-recovery` and `setup`.

## Residuals for root

- **Firefox COOP coverage.** Firefox in the browser suite no longer exercises COOP context-group isolation. The Rust header tests and Chromium/WebKit still do.
- **Non-COOP goto hangs.** A rare Firefox goto hang on a non-COOP page (microsoft/playwright#42183) is not covered by this change. If it recurs, the next step is a bounded superseding `goto`, not a longer timeout.
- **CI artifacts (proposed, not done).** `ci.yml` could upload `target/browser-results` on failure for specs that keep tracing on. `ci.yml` is not this lane's file.
- **Sibling tests.** Root decides on `505469f`.
- **Essentials in CI.** CI runs the schema tests only with Platform features, so the Essentials branch of the expectation runs only locally.
