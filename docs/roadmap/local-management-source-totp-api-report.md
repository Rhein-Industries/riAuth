# Configured source-TOTP bearer route: evidence report

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`. M03 task
`3b9fbfa6-716a-4ce8-842a-2c953bab3d0f` and M07 task
`0da684c3-b5cd-45e5-b190-1d0ce97f2c80`, worktree
`9c54c023-a95f-48e9-a908-6f97a5677c98`, branch
`roadmap/local-management-wave27`. Both tasks stay in progress. Lane worker:
Claude Opus 5.5. Implementer and reviewer subagents reported
`claude-sonnet-5-5`.

## Commits

| Commit | Kind | Content |
| --- | --- | --- |
| `213b8ea4b76c0c1a215dfef6efd8121ad20ff6a7` | Alignment merge | Main `c907598e4a6c1277d74fc7739e41593615fdedf8`, which carries W02's reviewed service (`91bb5b1`). The only conflict was this lane's connector-status port report, which main carries in root's reconciled form. Main's version was kept, and both sides and the conflicted file were backed up outside the repository before resolution. Afterwards the branch differed from main only by this lane's wave reports and two unclaimed tests. |
| `af3f0040df7de5a39ba264143c2e524a67694a91` | Code | The route, handler, rate-bucket arm, focused test and two documentation lines. |
| `bf1a152a5b9136344dbd462c97ae001912a6f064` | Tests and docs only | Review follow-up: precise refusal order and pass-through outcomes in the docs, a real pending-factor 403, and ignored body and retry headers with no receipt or session. No production change. |

## Change

- `src/api/workflow.rs`:
  - `POST /api/workflows/configured/{workflow}/source-totp`, registered
    next to source-passkey.
  - `configured_source_totp_start`, which has the same shape as
    `configured_source_passkey_start`: bearer from the existing guard,
    `workflow` from the path, then
    `app.run(core.workflow_configured_source_totp_start(&token, &workflow))`,
    returning `SourceStart` unchanged.
  - There is no body extractor, idempotency or `If-Match` handling, API-level
    id bound, cookie, protocol binding, initial issuance or receipt.
- `src/api.rs` makes the one granted change: the `source_start` rate arm now
  matches `/source-totp` beside `/source-passkey`. Every other category,
  default, agreement entry and route is unchanged.
- `docs/workflows.md` gets one paragraph in the source-TOTP section, and
  `docs/api.md` one row.

## Outcomes, as implemented by the reviewed service

| Request | Result |
| --- | --- |
| No `Authorization` bearer | 401 from the handler's bearer guard, for any workflow |
| Well-formed but unknown or expired token on the configured chain | 401 |
| Workflow id that is not configured | 404 `not_found` "Configured workflow is unavailable" (`Error::missing` in `configured_definition_in`), checked before the session, so a well-formed unknown token also gets 404 |
| Configured workflow that is not the source-then-current-TOTP chain | 409, also before the session |
| Malformed `If-Match` or `Idempotency-Key` | 400 from the shared `protect` middleware, as on every route; well-formed ones are ignored by this route |
| Live linked session without a current TOTP, or with a current TOTP and a pending factor | 403 |
| Disabled user or barred administrator; disabled or missing source; OAuth-profile source or changed policy | 403; 404; 409, passed through from the service |
| Second start while a run is active on the session | 409 |
| Eligible linked session | 200 with exactly `workflow` and `authorization_url` |
| `GET` on the route | 405 |

The brief listed an unknown workflow as 409. The service returns 404 for an
unconfigured id and 409 for a configured workflow of another shape; the route
passes Core's error through, as the source-passkey route does. The docs state
both.

## Wire evidence

`tests/workflow_configured_source_totp_api.rs` (Platform only) drives
`riauth::api::router` with in-process requests and a local signed upstream. Its
helpers are copied from W02's `tests/workflow_configured_source_totp.rs`, not
imported.

| Test | What it shows |
| --- | --- |
| `refusals_write_nothing` | Every refusal case above, including the refusal order and a pending factor next to a current TOTP, leaves the store snapshot unchanged except rows the explicit rate-ledger filter allows (`http_rates/`, `index_expiry_http_rates/`, `index_counts/http_rates`): no run, active-session marker, receipt or audit. On the redb fixture rate counters stay in memory, so the filter is inert there |
| `an_eligible_linked_session_gets_only_the_workflow_and_the_authorization_url` | Exact two-key response; the workflow id is under `workflow.binding.workflow`; the URL is on the fixture upstream with state, nonce and code challenge; no `Set-Cookie`, session or token |
| `the_unchanged_continuations_finish_the_chain_over_http` | Signed callback, then `/api/workflows/{id}/source`, `/totp/start` and `/totp` complete the run |
| `a_second_start_on_an_active_session_is_a_conflict` | 409 |
| `only_post_is_routed` | 405 |
| `a_body_and_idempotency_headers_are_ignored_and_leave_no_receipt` | A JSON body with well-formed `Idempotency-Key` and `If-Match` gets the same two-key response as a plain start, with no new row under `receipts/` or `sessions/` and the same write profile; reusing the key gives the active-run 409, not a replay |
| `the_source_start_rate_override_covers_source_totp_and_source_passkey` | With `rate_limits {"source_start": 1}` set before `Core::initialize` records the node-security agreement, a second request from one address gets 429 on `/source-totp`, and the same holds for `/source-passkey` as a control |

The implementer removed the new `/source-totp` arm from `src/api.rs` and the
rate test failed, then restored it.

## Checks actually run

All with a private `CARGO_TARGET_DIR` under this worktree, `CARGO_BUILD_JOBS=1`,
`CARGO_INCREMENTAL=0` and `--locked`, by the lane worker after the subagent:

| Check | Result |
| --- | --- |
| `cargo test --locked --features test-support --test workflow_configured_source_totp_api` | 6 passed at `af3f004`; 7 passed at `bf1a152` |
| `cargo clippy --locked --features test-support,fuzzing --lib --test workflow_configured_source_totp_api -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `python3 scripts/check-docs.py` | passed |

Not run: the Core factor security campaign (W02's), any other suite, browser,
accessibility, PostgreSQL, external identity providers or released artifacts.
The upstream is a local signed fixture.

## Evidence limits

- The snapshot filter allows only rate-ledger rows. Whether a given backend
  stores those rows in the snapshot is backend-dependent; the comparison is
  strict for every other bucket.
- The test proves the bearer adapter, rate bucket and continuation routing.
  It does not re-establish the service's factor, replay or drift security,
  which is W02's reviewed evidence.

## Independent review of `af3f004`

A read-only Sonnet reviewer (reported model `claude-sonnet-5-5`) found no
blocking defect.

What held:

- The handler has the passkey handler's exact shape, and the `src/api.rs`
  change is exactly the one arm.
- No other middleware treats `/source-passkey` specially. Metrics use the
  matched route, and the approval adapter label already knows `source-totp`.
- Outcomes and their order match the source-passkey route for the same
  inputs.
- The 429 test fails without the new arm: the route would fall to `general`,
  and the second call would return 401.

Low notes, addressed by `bf1a152` unless stated:

- **Error order:** Core resolves the workflow before the bearer, so an
  invalid bearer on an unconfigured or wrong-shape workflow returns 404 or
  409, not 401. This is the same as source-passkey, and the docs overstated
  401.
- **Incomplete outcome list:** the docs omitted other service outcomes the
  route passes through.
- **Late link check:** the account link and source session are checked at
  `/source`, not at start, which is W02's design.
- **Inert filter:** the rate-ledger filter cannot trigger on the redb fixture,
  where rate counters live in memory, so the test name overstated it.
- **Pending-factor branch:** the pending-factor case hit the same branch as
  "no current TOTP", so the pending branch went unexercised over HTTP.
- **Ignored headers:** nothing pinned that a body, `Idempotency-Key` or
  `If-Match` is ignored without a receipt.
- **Not changed:** a configured workflow whose id is literally `source-totp`
  or `source-passkey` starts at a path that also matches the `source_start`
  arm. This is a parity quirk of path-based classification.
