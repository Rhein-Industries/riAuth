# Wave30 O06 configured-controller root review

Accept immutable code `286bd4ddb5e2a84dd63598fe68f9a3f86ed1f0ff`, static report
`0787d12c0cfe205db22ca3b31724bdf4f2ea3903` and separate executed evidence
`27b04e7191f531f335ce8120d3069cabb194f6ff`.

Root read the full production diff and focused test. The diagnostic uses the
existing authorized read transaction and schedule/job lists. Configured keys
without a stored schedule contribute a separate count and redacted controller
row. It neither reads credential files nor creates schedules. Existing stored
counts, dispatch, lease, scheduler and execution behavior are preserved. The
new row ranks after retained actionable failures under the existing 50-row cap.
Configuration validation retains its 96-controller limit.

The one passing public HTTP/Core fixture exercises exact permissions, unchanged
full snapshots, absence without a background worker, an event job coexisting
without a periodic schedule, a disabled stored schedule, 31 missing controllers
plus 20 retained failures and the unchanged cap/priority, and 97-entry refusal.
The worker ran the exact reserved command once: 1 passed, 1.06 seconds after a
3m47s cold build. No correction or additional target was needed. Root did not run
Cargo. Source/test formatting, documentation and whitespace checks are static
integration checks. The existing native linker warning is recorded.

Missing current schedule evidence is not proof of historical nonexecution or
remote lag. The operations action table is updated to cover both the new
configured-without-schedule row and an overdue stored schedule. This slice
resolves that visibility defect only. O06 remains open for the other original
facets; no PostgreSQL, live connector, worker execution, release or HA outcome
is inferred from this local synthetic redb check.
