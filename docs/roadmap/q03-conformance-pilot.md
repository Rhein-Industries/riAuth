# Q03 independent OIDF pilot: local preflight

Q03 remains open. On 2026-09-29, at source commit
`19a69c66b473480c8b570498231fad4d4edb7a31`, the pinned runner existed,
but its private pilot inputs did not. No independent OIDF plan was run, no OIDF
result was obtained, and no certification is claimed.

| Required input | Local finding |
| --- | --- |
| Independent suite checkout at `440eec8bac7b12b7389d7ca9cbc459b53507a443` | No `run-test-plan.py` checkout was supplied or found in a bounded name-only search of the worktree and immediate local project locations. |
| Private suite configuration | No path was supplied; no pilot-named configuration was found in the worktree filename scan. File contents were not searched. |
| Pilot server endpoint | `CONFORMANCE_SERVER` was unset in this session. No endpoint was supplied. |
| Pilot authorization token | `CONFORMANCE_TOKEN` was unset in this session. No token value was inspected or printed. |
| Exact upstream plan and variants | Not supplied; the pinned suite is unavailable locally, so its plan inventory could not be checked. |

The search was limited to local filenames and directory names; it is not a
claim that private files or credentials do not exist elsewhere. The runner
itself has no built-in endpoint, token, plan, or private configuration.

## Bounded runner validation

[`run-conformance.py`](../../scripts/run-conformance.py) still pins the same
independent suite revision and runs one upstream plan with `--no-parallel`. It
now refuses a different or dirty suite checkout, a missing suite entry point,
an unreadable or non-private configuration, a malformed endpoint, a missing
token, tracked changes in the riAuth source tree, or an existing evidence directory.
HTTP is limited to loopback; other endpoints require HTTPS. The output must
be outside the suite checkout.

The runner captures its managed stdout, stderr, and suite exports in a private
temporary directory. It writes only `run.json` to a new directory with mode
`0700`; the report has mode `0600`. The report records exact source and suite
commits, one plan name, a hash of the endpoint and config, the upstream exit
code, and hashes, counts, and sizes of the captured output. It retains no raw
runner-managed logs or exports. A zero upstream exit with no export files is
reported as incomplete and exits nonzero. This metadata does not substitute
for an independent detailed conformance report; the suite or configuration
could also write data outside the runner-managed temporary directory.

With the private inputs present, run from a clean riAuth commit using the exact
suite checkout and a new output path:

```sh
python3 scripts/run-conformance.py \
  --suite /path/to/pinned-suite \
  --config /path/to/private-pilot-config \
  --plan 'EXACT-UPSTREAM-PLAN-AND-VARIANTS' \
  --output /path/to/new-private-evidence-directory
```

Set `CONFORMANCE_SERVER` and `CONFORMANCE_TOKEN` through the private pilot
environment before invoking it. Review the exact suite plan and scope before
running; the command above is a template, not an executed pilot. Preserve a
separately secured, redacted independent suite report if detailed test outcome
evidence is required. The remaining Q03 gate is a scoped independent run
against the authorized pilot, its reviewed results, and any separately required
certification process.
