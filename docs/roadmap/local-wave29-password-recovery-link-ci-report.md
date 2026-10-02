# Password recovery link CI assertion correction

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Existing worktree: `a1303b57-4a34-487e-9c63-a841f05b51a0`, branch
`roadmap/local-workflow-safety-wave27`.

Fixture commit: `9c3b9fae2901b67955fb0f9a2a8e4ad4ad9a274a`.
Alignment merge: `e633cfac5a4f629b967e13e6328d4ea5ea9f6bb7`, bringing fixed
reviewed main `979e7153f494cf827a0e39a75a78986544a92c42` into this branch without
conflicts, reset or stale replacement. Prior source/report history is preserved.

This is separate bounded CI support. The original RiWork rows were reread;
W02 `548d114f-9d0a-474a-a4c8-fa03af3ec3b1` and W05
`ceaddee1-2c9a-48d2-9ff4-d1f71396e954` are done. No task status changed;
W02 is not reopened by this fixture correction.

## Observed failure and product trace

Historical Linux run `36963622023`, check job `110706140611`, failed
`password_browser::sign_in_pages_offer_password_recovery` at line 1583:

```rust
html.contains(r#"id="forgot-password" href="/account/reset""#)
```

The downloaded log `/tmp/riauth-wave29-failed-36963622023.log` was inspected, and
its SHA-256 matched
`ef058cdb88df1788983b7223224c7a07817d2dbcb1e06eccc44616ecbfc5c3b4`.
It records 11 passed / 1 failed in this target, 37.38 seconds. That Linux result
is historical CI evidence, not a local full-target run.

The source inspection found a correct link and preserved gate:

- `src/portal/index.html:81` has `id="forgot-password"`, followed by
  `data-capability="identity.email_password_reset"`, then `href="__BASE__account/reset"`.
- `src/portal/http.rs::page` serves that template; `portal_html` substitutes the
  escaped cookie base. It retains all three attributes on the same anchor.
- `src/portal/capabilities.js` reads each element's capability declaration and
  marks it disabled unless the instance feature is usable. `app.css` hides
  capability elements before readiness and when disabled. The reset feature's
  configuration gate still requires local mail material; `account.js` also checks
  that feature. None of these sources changed.

One exact diagnostic baseline kept the old adjacency condition and printed only
the selected public anchor opening tag. Its returned HTML contained exactly:

```html
<a class="text-button" id="forgot-password" data-capability="identity.email_password_reset" href="/account/reset">
```

The original condition failed even though this anchor has the required target and
gate. This establishes a stale attribute-adjacency assumption, not a missing link
or gate defect. No production seam was needed.

## Fixture correction and scope

Only the body of `sign_in_pages_offer_password_recovery` in
`tests/password_browser.rs` changed. It extracts opening tags from the actual
returned `/apps` HTML and selects anchors bearing the exact ID token. It requires
exactly one matching anchor, then requires exactly one attribute of each relevant
name with its exact value: ID `forgot-password`, root-relative href `/account/reset`
and capability `identity.email_password_reset`.

These assertions are scoped to that same anchor, independent of ordering or
unrelated attributes. Missing, duplicate or differently valued relevant attributes
fail; separate strings elsewhere in the page cannot satisfy the link checks.
The test does not introduce a general HTML parser or dependency.

Every byte before the old assertion and after it remains unchanged. This includes
the function prefix/signature, existing password controls, account.js retrieval,
expected reset/request/history strings and forbidden `innerHTML`, `localStorage`,
`http:` and `https:` assertions. No helper or production HTML/JS/capability/config/
workflow/approval/API source was edited.

## Exact local checks

Both Cargo invocations used this private environment and the same exact filter:

```sh
env CARGO_TARGET_DIR="$PWD/.target-wave27" \
  CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test --locked --features test-support,fuzzing \
  --test password_browser sign_in_pages_offer_password_recovery \
  -- --exact --test-threads=1
```

| Invocation | Observed result |
| --- | --- |
| Diagnostic baseline retaining the old adjacency assertion | 0 passed / 1 failed / 11 filtered out, 1.30 seconds; printed the correctly rendered gated anchor above. |
| Corrected same-element assertion | 1 passed / 0 failed / 11 filtered out, 1.11 seconds. |

There was no compiler failure. Both builds printed the existing macOS
`__eh_frame` compact-unwind size warning. The lowest sampled free disk was about
18.0 GiB, above the 8 GiB stop floor; the accepted target was never used.

Additional checks:

- Scoped `rustfmt --check --edition 2024 --config skip_children=true tests/password_browser.rs`
  passed after formatting only that file.
- `git diff --check` passed for the fixture.
- `python3 scripts/check-docs.py` passed (Markdown links/build-directory layout).
- A byte comparison against the alignment merge verified an identical prefix and
  entire suffix surrounding the one replaced assertion block. Git source scope
  verified no production changes.

This uses local macOS/redb fixtures and in-process HTTP rendering/assets, not a
browser or desktop session. It does not execute the JavaScript capability gate;
its source was inspected and its same-element declaration is enforced by the
corrected fixture. No full password target, broader suite, clippy, PostgreSQL,
accessibility, service, tenant or release check was run. A fresh Linux CI rerun
remains root's integration check; other reported CI successes were not rerun here.

Root owns review, integration and push. No main/accepted edit, board mutation,
new task, worktree, worker or managed shell occurred. W02/W05 remain done; this
report claims only the bounded assertion correction and its exact local result.
