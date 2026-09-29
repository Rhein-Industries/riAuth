# O01 process roles

Status: one local duty gate. O01 stays open. This page records what one
riAuth process owns today, the explicit `gateway` and `worker` selection
added in this slice, and the behavior that was not run.

The omitted role is still the integrated one-process server. Selecting
`gateway` or `worker` does not create a second node, a separate binary, or
a check that some other process covers the duties this process drops.

## What one process owns

These statements describe the current source. They are not a multi-node run.

| Duty | Where it starts | Integrated | Gateway | Worker |
| --- | --- | --- | --- | --- |
| Product HTTP: discovery, OIDC, SAML, portal, administration | [`router`](../../src/api.rs) | yes | yes | no |
| Probes `/livez`, `/readyz`, `/healthz` | [`probes`](../../src/api/probes.rs), including an issuer path prefix | yes | yes | yes |
| LDAP, RADIUS, and proxy listeners | [`ldap_server::start`](../../src/ldap_server.rs), [`radius::start`](../../src/radius.rs), [`proxy_server::start`](../../src/proxy_server.rs) from [`api/server.rs`](../../src/api/server.rs) | when configured, Platform build | when configured, Platform build | refused |
| Background loops | [`start_background`](../../src/api/server.rs) | yes | no | yes |
| Native TLS reload | `serve_http` in the same file, when certificate files are set | yes | yes | yes |
| First-run setup | [`bootstrap::serve`](../../src/bootstrap.rs) | yes | yes | refused before bind |

The background loops are reconciliation (5 seconds), provisioning and
deactivation (250 milliseconds), account mail (5 seconds), logout and
Platform SSF delivery (2 seconds), maintenance (60 seconds), and alerts
(60 seconds). Intervals are scheduling settings, not completion deadlines.
`ManualConnector` is not one of those loops. A management request on the
product router can run that pass in the process that served the request, so
a gateway can still do that work inside an authenticated call.

Embedded redb has one owner. [`store/ownership.rs`](../../src/store/ownership.rs)
refuses a second opener (`storage_owned`, exit 5) and a location where the
file lock cannot stay exclusive (`storage_not_exclusive`). PostgreSQL is the
backend that already allows more than one full riAuth process. That existing
multi-process deployment is full nodes sharing one database. It is not
evidence that a gateway process and a worker process have been run together.

HTTP issuer URLs must be loopback unless they are HTTPS. `trusted_proxies`
defaults to empty. Product routes on the integrated and gateway roles use
the same Core identity and authorization path. The worker does not mount
that router, so it does not publish discovery, JWKS, the browser UI, or
administration.

## Selection

`[process]` is optional. Omitting it, or leaving both fields at their
defaults, is `role = "integrated"` and `accept_partial_duties = false`.
`riauth init` omits the table. Unknown role names and unknown keys in the
table fail configuration parsing. Validation then rejects:

- `gateway` or `worker` unless `accept_partial_duties = true`. The message
  says this process will not run the other duties and that riAuth does not
  check any other process.
- `integrated` when `accept_partial_duties` is true.
- `worker` when `browser_ui` is true. The default is true, so a worker must
  set `browser_ui = false`. The pages are not silently skipped.
- `worker` when any LDAP, RADIUS, or proxy listener is configured. The error
  names the kind and the first listener ids. On an Essentials build, a
  listener stanza still fails first with the Platform-build error.
- `worker` when the store is not initialized. Setup is not served. Use
  `integrated` or `gateway` until initialization finishes.

A role is not refused only because the backend is redb. One process may be
a gateway or a worker on an embedded store after the acknowledgement above.
A second process that opens the same redb file still gets `storage_owned`.
This slice does not require PostgreSQL, because that requirement would
describe a two-process topology this repository has not run.

`/livez` stays free of database work. Its JSON adds `role` and `duties`
(`authentication`, `protocol_listeners`, `background_jobs`). Those flags
say what this process is configured to start. They do not say that a job
succeeded or that a peer is healthy. `/readyz` and `/healthz` still require
storage readiness within two seconds. Integrated and gateway responses keep
`issuer`. A worker omits `issuer`, and its 503 text is “Worker storage is
not ready” while the code stays `not_ready`. Application-worker saturation
fails readiness only for a role that serves authentication. A worker that
cannot serve product routes answers those paths with HTTP 404 and
`error` `not_served`.

On an Essentials build, `duties.protocol_listeners` is false for every role.
The listener implementations are not in that build.

After setup activation, the same duty gate starts listeners only when the
role owns them and background loops only when the role owns jobs. The full
browser setup ceremony was not run as a gateway test. The uninitialized
worker check is the error returned before the process binds.

## Boundaries that stay put

[`process_role`](../../src/process_role.rs) imports neither configuration,
Core, nor the store. Configuration validation and the server startup call
it. Shared identity and authorization stay on the existing product router.
The embedded single-owner rule stays in the store open path. Ongoing module
boundary work in other worktrees is not edited here.

## What this slice ran

On the Platform build, local tests covered:

- omitted configuration stays integrated and does not serialize `[process]`
- an unknown role and an unknown `[process]` key fail parsing
- gateway and worker fail closed without `accept_partial_duties`
- a worker with browser UI or an LDAP listener fails closed; the same
  listener is accepted on a gateway
- an uninitialized worker returns the setup refusal and does not bind
- an integrated process serves discovery and JWKS, finishes a provisioning
  pass, and a second open of that redb file returns `storage_owned`
- a gateway serves discovery with `duties.background_jobs` false, and
  background finished/active counters stay at 0 for two seconds after ready
- a worker serves probes, returns `not_served` for discovery, JWKS, and
  `/api/login`, and finishes a provisioning pass

On this checkout the Platform command in [testing](../testing.md) was run with `--locked --offline`, a private Cargo target, incremental compilation off, and two compiler jobs. The library filter passed 3 tests (the two role checks and the existing saturated-probe check) and filtered out 88 others. `tests/process_role.rs` passed all 6 tests. The only compiler note on that run was the existing `__eh_frame` linker warning. `cargo check --locked --offline --no-default-features --features essentials --lib` finished with no errors. Those role tests were not re-run on Essentials, and no second process was started.

## Still open

- Nothing checks that another process runs the duties this process omitted.
- A gateway and a worker were not run as two processes against one
  PostgreSQL database. Do not treat this page as that deployment.
- `/readyz` can succeed for a gateway while its background loops are off.
- A gateway still serves the product API, so a manual connector pass can
  run inside a request on that process.
- The worker has no admin or discovery surface. Probes are the HTTP surface.
- The post-setup handoff uses the same duty gate. The setup ceremony itself
  was not exercised for a gateway.
- Worker TLS uses the same HTTP server path and was not given its own test.
- Embedded redb remains one owner. There is no mismatch detection between
  processes, no shared-job coordinator, and no separate role binary.
- O02’s file lock is unchanged. Splitting duties does not add a new guard
  for a second host that mounts the same redb volume.
- O03 mismatch detection, operational dashboards, and a tested distributed
  compose layout are outside this slice.
