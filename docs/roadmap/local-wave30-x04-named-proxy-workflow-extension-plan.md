# X04: named proxy and exceptional-workflow extension audit

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original X04 task
`484beda3-9015-4f4c-9052-67d80d0defbe`; reservation
`wave30_X04_named_proxy_workflow_extension_audit`. Support worktree
`42bb51c6-c198-4adb-bd92-0a5222853231`, existing shell
`f1575610-c6f0-4dbf-9f33-d01ed057a194`, branch
`roadmap/local-module-boundaries-wave27`. Audit date: 2026-10-03.

**Disposition: demand-deferred, pending one named integration requirement.**
All four original directions were considered against the current supported
boundaries. No owner-approved additional gateway, proxy, credential-injection
application or exceptional-workflow peer was found in the inspected public
source and planning exports. No additional production hunk is justified by
those records. I03, W07 and Q01 remain closed; X04's observed `todo`/unassigned
state is unchanged. Existing real nginx/Traefik results and native guest
isolation are reusable foundations, not evidence of an unspecified extension.
P3 assessment remains separate from completing Essentials.

## Immutable source and original row

Published source inspected through Git objects:
`1a517a1a461b7017c353d37a5e498d2c7cfa7985`. Own clean entry HEAD:
`1319ebc83309d35f92e04feb82b2121a486ccc1b`. No checkout alignment, merge,
reset or old-file replacement occurred. Earlier X03/I06/G05/D01 reports and
history are retained. This reservation writes only this new report.

The full original row was read from the local project export
`/Users/dominik/.local/share/riwork/orchestrators/projects/891e7443-8dac-4c1b-897f-9e53cb59c7ee/planning/current-tasks.json`:
244347 bytes, SHA-256
`5cfd882213745a86dcdeeb7e4debccf24fd22299c0fc7be73d6d988512739752`.
This is an **export observation, not a live board query**. No task/project MCP
was exposed in this session; no provider or unrelated coordination was used.

Exact original row content (JSON serialization; values unchanged):

```json
{
  "id": "484beda3-9015-4f4c-9052-67d80d0defbe",
  "project_id": "891e7443-8dac-4c1b-897f-9e53cb59c7ee",
  "title": "[P3] X04 — Advanced proxy orchestration and specialized extensions",
  "details": "Backlog ID: X04\nProject UUID: 891e7443-8dac-4c1b-897f-9e53cb59c7ee\nWorkstream: Demand-driven extensions\nProposed priority: P3 — demand-driven extensions (orchestrator recommendation; user supplied priority meanings).\nRecommended model effort: medium. Apply this setting when a worker is launched; this task detail does not change a running model.\nEffort rationale: Focused documentation or a bounded extension assessment with established contracts.\nRecommended implementation phase: 6; security work continues throughout all phases.\nProposed prerequisites: I03, W07, Q01\n\nRequested outcome and acceptance scope:\nConsider managed gateways, additional proxies, constrained credential injection, and exceptional workflow integrations.\n\nWorkstream goal: These should not delay a complete Essentials experience. Scope them separately when a real integration requires them.\nWorkstream completion gate: Each extension solves a named requirement, has its own security model, and is tested against a real peer.\n\nProduct boundaries:\n- riAuth Essentials: OIDC/OAuth, passkeys, complete browser self-service, compact administration, groups and claims, API/CLI, audit, backups, LDAP import, and outbound SCIM.\n- riAuth Platform: Essentials plus configurable workflows, broader protocols, advanced federation, cloud connectors, device integrations, and expanded administration.\n- riauthctl: Remote administration, with optional terminal USB-authenticator support separated from the server.\nBoth server builds retain the same identity, authorization, revocation, and credential-protection semantics for shared capabilities.\n\nCompletion evidence: Review the relevant implementation, tests, documentation, and released artifacts as applicable. Report verification actually performed, remaining gaps, and external prerequisites. Do not mark implementation complete from documentation or a worker report alone.\nScheduling: Leave todo and unassigned. User requested waiting for the existing dependency update and returning the task list; no worker launch is authorized by this task record.\nDesktop automation: Use RiWork's Cua.ai Driver only. If unavailable or missing macOS permissions, report the problem and direct the user to RiWork's Cua setup.",
  "status": "todo",
  "worktree_id": null,
  "created_at": 1790534568,
  "updated_at": 1790534568
}
```

Prerequisite rows were also read in full from that same export:

| Original prerequisite | Exact outcome and relevant boundary | Observed status |
| --- | --- | --- |
| I03 `e1324767-e7fd-4f6d-b238-8fc1c792452c` | “Validate headers, cookies, origins, backend isolation, WebSockets, and explicitly supported bypass or routing rules.” Advertised integrations need setup, lifecycle and failure handling. | `done`, WT `5275b226-917f-484d-ade0-34bdb27f63b1` |
| W07 `63ab3917-0fe0-4d02-819e-fc0c09b72ca6` | “Isolate exceptional custom logic with explicit permissions and bounded execution, data access, and network access.” Platform conditional enrollment/authentication; Essentials needs no designed workflow. | `done`, WT `42bb51c6-c198-4adb-bd92-0a5222853231` |
| Q01 `0ec31a21-cbc1-41ab-9259-cf932aa730bb` | “Document trust boundaries for accounts, recovery, agents, connectors, workflows, proxies, and devices.” Evidence categories do not establish each other. | `done`, WT `452ae4e0-f9fb-43c0-819b-da2b6365470c` |

Read full pinned [CONTRIBUTING](../../CONTRIBUTING.md) and
[SECURITY](../../SECURITY.md). Their blobs are respectively
`64708527ea0d85b741c5d8ffe88d4955a734bb1f` and
`047208fb72e97fc942fe5d4d988b162c28f80d5c`.
No applicable `AGENTS.md` was found in the previously checked workspace
ancestors or the pinned repository inventory. The explicit static-only
reservation excludes normal contribution build/test/index-edit instructions.
Public source/fixture inputs were inspected; no private credential, token,
certificate-key, session or tenant contents were opened.

## Current supported bodies and all four proposed directions

Read the complete [proxy guide](../proxy.md),
[Platform forward-auth recipe](../recipes/platform-forward-auth.md), nginx
and Traefik templates, and the HAProxy example. Reviewed the controlled and
isolated extension guide sections (workflows.md:1109–1380), relevant capability
and edition sections, and the current production/fixture bodies below. Line
witnesses in this report refer to the fixed published Git object, not mutable
checkout line numbers.

| Original direction | Source-backed current boundary | Disposition for an additional extension |
| --- | --- | --- |
| Managed gateways | `Settings::validate/target`, `Core::outpost_start/callback/auth/forward/logout`, `ProxyRequests`, `proxy_server::handle/start_with_port` provide configured application SSO and a local listener. The guide's line60 assigns nginx/Traefik deployment to infrastructure operators; no remote installation/update/fleet manager is advertised. Agents manage client settings with existing `client.write`, conditional updates and reviewed manifest plan/apply. | A managed control plane would introduce remote installation/configuration/credential authority and effect ordering. No requested fleet, control API or rollback owner is supplied. Existing client configuration is not gateway orchestration evidence. |
| Additional proxies | nginx `auth_request`, Traefik forwardAuth and the embedded Rust listener are the explicitly supported application paths. Full nginx/Traefik real-process fixture bodies and the templates were inspected. HAProxy's complete example forwards the public issuer to two riAuth nodes, rewrites forwarding context and checks `/readyz`; deployment-examples.md:148–185 gives operator installation and backend restrictions. | HAProxy is a named load-balancer example, not a new application forward-auth integration or a supplied X04 demand. No additional application-proxy peer/version/topology requirement was found. Do not choose an arbitrary product simply to add another adapter. |
| Constrained credential injection | `outpost_auth` returns scoped identity headers plus filtered application cookies. The templates clear `Authorization`; the embedded listener strips caller authorization, forwarded context, private cookies and punctuation aliases, then reconstructs selected identity and forwarding headers. `outpost_callback` terminates OAuth credentials at the outpost and revokes their family. The guide excludes Basic credential injection and bypass-path rules. | A backend credential would add custody, audience/target binding, rotation/revocation and authorized disclosure requirements. No named legacy application, accepted credential mechanism or credential custodian is supplied. Identity-header assertion and passing an application's own cookie do not provide a backend-password injection implementation. |
| Exceptional workflow integrations | The configured path admits the extension-then-password shape, binds its exact module to the run, sends granted identifiers only, denies network, and treats one declared routing label as no proof. `workflow::extension::Host` is a held trusted in-process contract; its `fetch` returns a budget permit, no socket/body. It is not called by the configured executor. | No named external logic/peer, new graph, input field or network operation is requested. Wiring the trusted native Host, adding guest imports/network, or letting a label issue a proof would change the reviewed security boundary. W07 does not require those unspecified extensions. |

The full [Q01 threat model](../security/threat-model.md) was read. Its declared
baseline `96e23e2dedf84db4e39091a004cbd6591acbec97` and historical gap language
are retained; they are not fresh findings against current editions/workflows.
TB-06/TB-07/TB-11 and the read RI-WF controlled-extension section and
RI-PROXY-001/002 (invariants.md:717–837) constrain any later direction: exact
request/account/module and immediate-peer/target binding, no forged proof,
filtered credentials/cookies, and live parent/client policy. A new extension
needs its own explicit dataflow and authority model, rather than a generic
claim that Q01 or the present sandbox covers it.

Essentials separation has concrete source witnesses. `api.rs:675–676` gates
`platform_routes`; its outpost routes at865–870 and handlers at2830–2936 are
Platform-only. `edition.rs:394–441` rejects nonempty `proxy_listeners`,
`workflows`, `workflow_extensions` and Platform rate overrides on Essentials.
`workflow.rs:244–333` requires the exact guest/password shape and a Password
proof; `executor.rs:1160–1289` checks the live manifest hash, projects only
permitted facts and finishes the guest step with `None` evidence. The small
Essentials guide starts with local sections1–3; Platform's guide:105–108 leaves
proxy and advanced protocols unset for the first install. No X04 configuration
or workflow-editor requirement is added to that experience.

### Bodies actually read, versus whole-file identity only

| Fixed source path | Direct body review in this audit |
| --- | --- |
| `src/outpost.rs` | Entire237-line protocol/settings module: exact origins/domain/TTL/profile, target grammar, opaque cookies, duplicate headers, unsafe provenance and forwarded-target reconstruction. |
| `src/assembly/outpost.rs` | Entire397-line implementation: trusted peer before store work; live client fingerprint and same-browser state; issuer/nonce/PKCE and claimed callback; parent/client authorization; token-family termination; live scopes/groups; logout and cleanup ordering. |
| `src/proxy_server.rs` | Entire603-line transport: ambiguity filtering, fixed routes/upstream/no redirect/no environment proxy, limits, provenance before dispatch, selected outpost exceptions, identity/cookie filtering, TLS/CA setup, body/stream bounds, WebSocket handshake/30s recheck/shutdown and startup leases. |
| `src/assembly/proxy_server.rs` | Entire163-line capability adapter: per-operation Core dispatch; private Core trust marker; live profile; stored rate ledger. |
| `tests/outpost.rs` | Entire692 lines, including shared `exercise`: in-process riAuth plus test-authored Axum echo/WS app, real nginx process or embedded listener, CLI approval, cookie/query/header assertions, write/logout refusals, builtin-only alias/body/host/WS closure, optional Chrome and PID-scoped drop. |
| `tests/outpost_traefik.rs` | Lines1–177 and745–1079: complete setup/call helpers, owned process drop, full `traefik_forward_auth_real` and cookie helper. No claim of rereading every in-process test in the intervening section. |
| `src/workflow/extension.rs` | Entire production section1–627: held native Host, permission projection, bounds/slots, cooperative thread timeout and budget-only network permit. Not the whole test module. |
| `src/workflow/extension_gate.rs` | Complete registration/binding/coverage/manifest/admission entry/project/execute bodies at364–699; launch/wait/expiry/interpret bodies at1448–1612; guest entry1736–1773; native fixture helper and isolation/refusal tests2255–2397; full granted-identifiers test3004–3069. Remaining module hash/equality is not a new complete3552-line review. |
| `extension_gate/isolation.rs`, `guest-macos.sb`, `native-probe.c` | Entire109/24/61 lines: canonical exact executable/default-deny profile, before-IPC descriptor/native confinement checks, unsupported-host refusal, and synthetic read/write/network/fork/spawn plus inherited-descriptor/kqueue assertions. No native probe compiled or run. |
| API/edition/workflow/CI modules | The complete named bodies/spans above; CI selected proxy recipe199–208. Whole blobs hashed for identity, without claiming full-module review or fresh CI execution. |

The nginx fixture asserts actual application username, cleared Authorization,
filtered cookie, exact query path, positive same-origin/API-intent writes,
refused missing/cross-site provenance, refused logout leaving access intact,
then successful logout and subsequent denial/redirect. Its builtin-only
WebSocket echo and closure within35s after logout are **not nginx assertions**.
The nginx template does not clear every forwarding header; its fixture does
not prove those unasserted fields absent. Deployment must constrain which
identity/context the application trusts and prevent a direct backend bypass.

The Traefik fixture launches the selected program with static
`aliasHeadersStrategy: delete`, dynamic `trustForwardHeader: false`, a separate
issuer router hiding `/auth`/`/traefik`, and a test-authored Axum app. It asserts
exact return URL despite planted context, background401, in-process user
approval, alias/Authorization/private-cookie removal, cross-site403, missing
or wrong WebSocket Origin refusal, echo for the valid origin, hidden issuer
endpoints404 and next-request302 after parent logout. The open socket is not
read after logout. The dashboard variant, load-balancer trust-forwarding and
production TLS topology were not exercised by that fixture. These limits
remain relevant even when the selected real-process test passed.

## Accepted execution and immutable reuse

**No runtime was run for this audit.** Named command results below were read in
accepted execution/review records, not inferred from test names or path/hash
counts, and no private raw log or executable was reopened. Current source
review, historical native/real-peer results and tenant/deployment facts are
separate quantities.

| Recorded execution / actual pin | Accepted result and precise reuse boundary |
| --- | --- |
| Real proxies: run `37006213470`, job `110834995871`, checkout `88790deb62d32c84fa17dceb12cd93a727224e94`, default Platform. Root's complete [integration review](local-wave30-ci-88790de-integration-review.md) and [D03 root review](local-wave30-d03-d05-root-review.md) were read. | `RIAUTH_TEST_NGINX=<nginx>` plus selected Chrome, `cargo test --test outpost --locked -- --ignored`: **1 passed,1 filtered,17.77s**. `RIAUTH_TEST_TRAEFIK=<selected binary> cargo test --test outpost_traefik --locked -- --ignored`: **1 passed,13 filtered,3.15s**. Accepted raw log265774B SHA-256 `0d327b38cb4999bedc20df3f739eafac161baa6d571aa3a81a4d139189c67859`. Real proxy programs; in-tree riAuth and test-authored application peers on loopback. No managed gateway/customer application/tenant. |
| Earlier proxy CI record `E-CI`, checkout `2d05c00deed3a6dafcd458a5f1a1a9beb17d0dd8`, run `36950097067`, job `110661000640` | Complete published JSON command/result record read: nginx and Traefik each1 passed. Raw log265770B SHA-256 `458a30895b5987c0bebe2fb7368a1a32969fa0d41ce90acbfa56f43299ccd186`. Dated historical corroboration, not additional fresh runs or evidence that the entire later server is byte-identical. |
| Embedded listener: accepted ledger check206, `934213793ede837198d47a01e60575dbc7bcd682` | Recorded `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 cargo test --locked --offline --no-default-features --features platform --test outpost rust_reverse_proxy_terminal_sso_headers_websocket_and_revocation -- --exact --quiet`: **1/1 pass**, terminal SSO/header/WS/revocation. Read actual ledger command/result. No new invocation; earlier own source/assembly moves do not turn this into a current whole-build pass. |
| In-process proxy contracts: ledger check231, `c27f69e7c1f15b8874624f2d52d7db4c31923edd` | `CARGO_BUILD_JOBS=2 cargo test --locked --features test-support --test outpost_traefik -- --quiet`: **13 passed,1 ignored**. Router contracts, not the ignored real Traefik process. |
| W07 native isolation: final source `d772d29fbfc4e681774593ebbe32839df5f5f032`; implementation `6421a1e79392fe483130f727a365066cbdb88ee6`; accepted `afd7ad2d7e3299778cfad218d4610ba676c9573b` / `1610156480a7205d185180bc47c3ec4a28680a9a` | Recorded `cargo test --locked --lib workflow::extension_gate::tests -- --test-threads=1`: **19 passed,0 failed,95 filtered,33.06s**, macOS26.2 Apple Silicon, native source-built server. Log SHA-256 `619315c304faf46a656024e1bda649e48f703d5c758fcae088a2f99a7b45701e`. Full isolation report, W07 section of the [six-row review](local-wave30-six-task-completion-review.md) and complete [root disposition](local-wave30-six-task-root-disposition.md) read. Root accepted macOS support plus explicit unsupported-host refusal; no new Linux/Windows guest backend is an original W07 gate. |

The current CI **source** still pins Traefik v3.7.13 linux/amd64 archive
SHA-256 `52cd039a34258dd61c617a95d69252bc6bcae27c520f338186c31c7fef8f6394`.
The fixture uses its alias-deletion static setting. nginx is a runner distro
package without an exact version pin. No download, binary hash recomputation,
version invocation, job lookup or fresh peer execution was performed here.
Version names alone are not source/artifact equivalence.

Twenty explicit whole-byte Git-object equality comparisons were performed:
four native guest blobs, eight proxy implementation/fixture/template blobs,
seven prior-published guide/review blobs, and the original isolation report.
The following comparison witnesses permit reuse without reinterpreting old
runs as current execution:

| Current path at `1a517a1a461b7017c353d37a5e498d2c7cfa7985` | Compared executed source | Current SHA-256; whole bytes equal |
| --- | --- | --- |
| `src/workflow/extension_gate.rs` | `d772d29fbfc4e681774593ebbe32839df5f5f032` | `db5f3e1360ac5fb87b9c3d6f7f5526ed4b43e9888e78c576194c5ed293ec4495` |
| `src/workflow/extension_gate/isolation.rs` | Same | `0865277b37199fc89496e5d90d2d502c1d7dcf705eba6475b63c2cb4403b6d22` |
| `src/workflow/extension_gate/guest-macos.sb` | Same | `2df8d89f35f46185cbc056f20c16f7936fe07016f51f6b74efad2f009fd84204` |
| `src/workflow/extension_gate/native-probe.c` | Same | `c153883db31856bc3b696afa40d400a3e50eeccde981e90b6c139ed29602b385` |
| `tests/outpost.rs` | `88790deb62d32c84fa17dceb12cd93a727224e94` | `628756b95864c7812298083a58c45b7c6b8dd3d6611e3b05801085939babaa2d` |
| `tests/outpost_traefik.rs` | Same | `3763f0ae94dc4c8298581b20a1203f8e338b7990e5a5baac908cb021d3cc7b6a` |
| `deploy/nginx-forward-auth.conf` | Same | `93f5e8586d7d3e90216a9f67e717bf93dac9b18bbd3df20c355b3d6167c934f5` |
| `deploy/traefik-forward-auth.yml` | Same | `a983efa58278960fadc53f80d27e1937036013b8246df56092fd24de4cf67593` |
| `src/outpost.rs` | Same | `4e7ec3c76d1683f08374f905d96433e65aceb89eacac22d2b74867a2b18212f1` |
| `src/assembly/outpost.rs` | Same | `c92ffc1aa031130cbead8e88a4a4234faeaef36c1c5dc756cd01a7836e142c5b` |
| `src/proxy_server.rs` | Same | `72529af4c5a1d919c06db4f15e43cd027257a8a4de9c38e8793f1aed15d90a9c` |
| `src/assembly/proxy_server.rs` | Same | `e03341e6cf24ff8e828e61d1fbc26ac047043379947ca368bb3b1db4128255c0` |

The seven review/guide blobs equal prior audit source
`a6d361600a03713fc1b687f367e9db84efe43463`: `docs/proxy.md`,
`docs/recipes/platform-forward-auth.md`, the six-task completion/root
reports, D03 original-scope/root review, and the88790de integration review.
The original isolation report also equals its `d772d29...` object. These are
identity comparisons plus the stated body reads, not eight extra test passes.
All four native guest blobs equal the historical executed source completely;
only the eight listed proxy blobs were compared, not the entire88790de tree.

The local accepted ledger observation was642409B, SHA-256
`77edeafd704b577069ad27f90d8ee0e5ebb93b98de4ac094411f6a1c689d6424`.
Full relevant I03 slice172/181, W07 slice331/548 and selected command/result
records were read. I03 correction sources
`77ce70098c2073d6b8d8f6d074646d3c85c7abf5` and
`e178382d355259977c027f965df6b61b6b1a1938` were accepted as
`7a28fc49a8d1bec5ea5055598b512d54af0c7500` and
`cd56b5a16bbddc848fefa1a8612acb28bade97be`. Current alias/provenance
source and selected actual proxy results are read at their own pins; historical
review/correction stages are not collapsed into an unqualified first-pass claim.

Earlier failures stay recorded. The W07 native report preserves early loader
and closed-descriptor refusals, the deprecated sandbox dependency, the macOS
link warning, unsupported hosts and scheduling limits. Its initial whole-task
`in_progress` statement is historical; the later original-scope root closure is
separate. The read D03 review retains the88790de separate full-check failure
(**11 passed,1 failed,7.06s**, failed identity not established by that support),
older identity **171 passed/7 failed/3 ignored** with missing checkout, later
named local correction, TOTP/diagnostic failures and two invitation-passkey
browser skips. Selected integration success is not overall green. No physical
device, real tenant, protected customer app, universal proxy version, official
release, HA, hard real-time or paused-I/O proof is inferred.

## Named-demand search and ONE next input

Searched the fixed public `docs`, `deploy`, README and contribution records
for `managed gateway`, `additional prox`, `credential injection`,
`exceptional workflow`, `HAProxy`, `Envoy`, `Kong` and `named target`.
The bounded Git search returned39 lines. Read the relevant matching content;
other grants/migration rows using “named target” were not reinterpreted as
proxy demand. Full X04 JSON entries in the coverage inventory and D05 snapshot
were parsed. The inventory still says “No managed gateways or credential
injection. Demand-driven.” and “Named target required”; this is historical
planning, corroborated by current guides rather than treated as a fresh test.
The prior remaining-task matrix likewise classifies X04 as demand-driven.
The current task export's managed/additional/injection/exceptional/gateway
keyword search found only X04's original outcome; the accepted ledger records
existing I03/W07/A03/D03 work and no supplied X04 target. This establishes the
limit of the inspected records, not the absence of demand in private customer
communications or unqueried systems.

**ONE required input before a production reservation:** an owner-approved
named integration record choosing one of the four directions. It needs:

1. The requested operation and named real peer/application, its exact supported
   version/profile, available disposable peer/build and the acceptance example
   that existing I03/W07 cannot satisfy. Example client names `reports`,
   `dashboard`, synthetic guest stages and HAProxy deployment placeholders
   do not identify a requesting customer or mandate another integration.
2. A bounded topology/dataflow: issuer/application/upstream origins, trusted
   immediate peers, methods/paths/WebSocket expectations, backend isolation,
   callback and user/request/subject binding; which side owns each operation.
   State whether the requirement is config deployment, authorization transport,
   backend credential injection or exceptional workflow logic.
3. Its named security model: actor and exact authority, permitted input/output,
   effect/review/idempotency/audit and denial/revocation behavior. For any secret
   operation supply only custodian, secret-reference/rotation/removal policy
   and approved mechanism here, never credential bytes. For a guest say which
   current bound identifier/label is insufficient and why external IO is needed;
   do not silently convert routing into authentication proof.
4. Authorization and isolation for one real-peer fixture: permitted hosts and
   operations, synthetic identities, process/credential custody, finite resource
   bounds, no production mutation, cleanup owner and concrete success/refusal/
   rollback observations. This is input to root's later review, not runtime
   authorization from this report.

Until that record exists, there is no honest exact additional-adapter API,
credential writer, graph or gateway-manager patch to propose. The current
source anchors for a later minimal review are `outpost::Settings` and
`Core::outpost_forward` for transport, `proxy_server::handle` for backend
sanitization, and `supported_configured_extension_password`/
`run_extension_guest` for exceptional logic. Their paths are cited as boundaries,
not ownership claims or a request to edit all of them. No fixture command is
scheduled and no new worker/task is proposed. Root can retain/defer the original
P3 row pending this input; this audit alone does not certify an extension or
set the original task DONE.

Accepted credential-once and reviewed-creation receipt-secret exceptions,
route-specific optional/required retry headers, PAM fallback, held Group
representation and remote at-least-once/removal/review contracts remain intact.
Nonrenewed60s authority/admission and paused-I/O limits are not widened by proxy
or guest evidence. No speculative extension delays Essentials or reopens the
closed dependency rows. Any separately authorized future desktop must use
RiWork Cua.ai Driver MCP after reading descriptions/current state and report
missing setup/permissions; no desktop or accessibility work occurred here.

## Actual static checks and handoff

Performed assigned-cwd/clean-HEAD checks, full exported-row JSON reads, pinned
Git text/body reads, bounded demand searches, source/record SHA-256 and Git
blob checks, and the twenty explicit whole-byte comparisons above. An initial
read-only function-inventory regex had a parser error; it was replaced by a
literal-prefix inventory and succeeded. Some combined output was truncated;
the necessary body spans were subsequently read in bounded calls. No truncated
whole module is represented as a complete body review.

Final documentation/link, whitespace and one-file scope results are recorded
below after the report exists. No Cargo, test/helper import, source compile,
product/proxy/native execution, private capture parsing, runtime preparation,
service, network/provider operation, browser/desktop or managed-state action
was performed. No validation slot was taken or released. Root alone reviews,
integrates, publishes and decides statuses; original primary/assignment remain
unchanged. The final report-only commit hash is supplied in the handoff rather
than inserted into its own contents.

| Check actually performed | Result |
| --- | --- |
| Exact JSON comparison of the quoted full X04 row with the export | PASS; one matching UUID/project row, all values equal. |
| Referenced40-hex Git object existence | PASS,18 commit/blob objects resolved with `git cat-file -e`; this is identity checking, not execution. |
| Twelve current source SHA-256 table witnesses | PASS; hashes recomputed from pinned Git bytes and matched the report. |
| Report Markdown targets and whitespace | PASS;9 local targets exist, terminal newline present and no trailing whitespace. |
| `python3 scripts/check-docs.py` | EXIT1, solely the five retained layout names `target-wave29-source`, `target-wave28-scim`, `target-wave28-portal`, `target-wave28`, `target-wave27`; no Markdown link error. No directories/evidence deleted or checker changed. |
| `python3 scripts/check-repo-hygiene.py` after staging this new report | EXIT0,1026 tracked files. |
| `git diff --check` / `git diff --cached --check` | EXIT0; no whitespace errors. |
| Index/worktree scope | Only the approved new report; original HEAD/history retained, all existing tracked files unchanged. Active executable Git hooks absent. Final clean status and report-only commit are provided to root. |
