# riAuth target product contracts

Status: **Semantic inclusion, safe defaults and scenario acceptance frozen as
the A02 target contract for implementation; accepted by the project orchestrator
after documentation and matrix validation on 2026-09-27.
Not implementation or release evidence.**
Prepared against dependency-updated source revision
`96e23e2dedf84db4e39091a004cbd6591acbec97` on 2026-09-27. The authorized
inputs are `target/riwork/assignment.txt`,
`target/riwork/assignment-A02-finalization.txt`, `target/riwork/assigned-tasks.json`
and the user boundaries and priority meanings in `target/riwork/backlog.json`.
Those files are local assignment inputs, not required files in a release.
The finalization assignment records acceptance of A01/Q01 documentation and
resolves routine scope and naming choices; it supersedes the earlier draft
review blockers and planning-only scheduling text.

This document defines desired behavior. The accompanying
[capability matrix](capability-matrix.json) records inclusion, runtime defaults,
prerequisites, source evidence, gaps and measurable acceptance checks for each
capability. Its format is a target-contract artifact, not an implemented API schema.
Capability IDs below match that matrix. Configuration terminology here is a
contract; it does not introduce valid settings or commands into today's server.

## Evidence and dependency boundary

A01's accepted `docs/roadmap/coverage-inventory.md` and JSON twin were refreshed
read-only in `/Users/dominik/orca/projects/riAuth-public-preview-roadmap-a01-coverage`
on 2026-09-27. The matrix records their snapshot digests and maps every contract
row to A01 capability and backlog evidence. A01 identified 94 backlog items and
31 broader capabilities; this contract splits them into 46 acceptance units.
Q01's accepted `docs/security/threat-model.md` and `invariants.md` were read in
`/Users/dominik/orca/projects/riAuth-public-preview-roadmap-q01-security-invariants`.
The matrix records current snapshot digests, stable invariant/trust-boundary
crosswalks and source evidence. Acceptance of these documents does not establish
runtime enforcement: A01/Q01 inspected tests without executing them, and A02 ran
only documentation validation. Published assets were not inspected. Product
semantics are fixed here; the later evidence gates below control implementation,
compatibility and release claims.

A01 C31 now assigns direct local maintenance to separate server maintenance for
both builds. This contract uses `riauth-maintenance`, with no local store or
restore authority in `riauthctl`. A01 C24's broader all-sources Platform proposal
is split explicitly into optional ordinary OIDC/OAuth federation E20 in both
builds and advanced Platform federation P06/SAML P02 by the finalization assignment.

The source currently has [one crate](../../Cargo.toml), a combined
[CLI dispatcher](../../src/cli.rs) and unconditional terminal USB-authenticator
dependencies. It does not provide the target official Essentials, Platform
and separate riauthctl assemblies. The [release workflow](../../.github/workflows/release.yml)
and [packager](../../scripts/package-release.sh) describe Linux x86-64 artifacts;
they do not establish Linux ARM64 or separate-client release support. Released
assets were not downloaded or executed in this task.

[Browser sign-in and consent](../oidc-profiles.md), the
[application portal and passkey add/remove](../PORTAL.md) exist in source.
[Account lifecycle](../lifecycle.md) still requires the terminal for invitations,
email verification, password recovery/change, TOTP, recovery-code rotation,
session/consent management and device approval. There is no compact
administration GUI. Source-stage local-factor completion lacks a browser OTP
form. These are gaps to build, not reasons to reduce Essentials.

A01 additionally identifies a fixed capability list/static protocol discovery,
several separate plan/apply engines, bearer-only self-service APIs, and no
installed-bundle upgrade/recovery gate. The Chromium/Firefox/WebKit Playwright
project exists but is not invoked in CI; its virtual authenticators and phone
viewports do not establish physical hardware or mobile-device support. Current
restore preserves sessions/grants; the safer desired default below needs new
implementation and regression evidence.

[Release limitations](../limitations.md) describe evaluation/pilot scope,
unpublished independent OIDC conformance, the backup archive cap and redb-only
restore. [Google Workspace](../enterprise/ENT-03.md) currently depends on a token
broker; [device trust](../enterprise/ENT-06.md) keeps the local stand-in as the default, and its Verified Access v2 adapter has no managed Chrome tenant;
[Windows login](../enterprise/ENT-13.md) is a protocol without a packaged
credential provider. None establishes the desired finished integration.

## Distribution boundaries

Both server builds come from the same source revision and use one identity core,
management service, storage contract and shared browser components. Platform is
additive. A smaller build changes available adapters and administration depth;
it must not change who a user is or weaken the semantics of shared operations.
No license tier or entitlement mechanism is implied by these boundaries.

The supplied Essentials scope remains exactly OIDC/OAuth, passkeys, complete
browser self-service, compact administration, groups and claims, API/CLI, audit,
backups, LDAP import and outbound SCIM. The shared core/operation contracts below
make those promises safe to deliver; optional ordinary federation is separately
configured within OIDC/OAuth. It is not a prerequisite for local account journeys.
Platform's optional client-certificate login and external signer integrations
remain distinct from shared HTTPS/TLS, WebAuthn, local token signing, hashing and
secret-custody protections, which retain identical semantics in both builds.

| Distribution | Required inclusion | Exclusion or responsibility boundary |
| --- | --- | --- |
| **riAuth Essentials** | OIDC/OAuth; passkeys; complete browser self-service; compact administration; groups and claims; API/CLI administration; audit; backups; LDAP import; outbound SCIM. Also the shared storage, operation and credential protections needed to deliver them. | Configurable workflow engine/editor, SAML, LDAP serving, RADIUS, inbound SCIM, advanced federation, cloud-directory connectors, integrated proxies, device/Windows integrations, temporary access and expanded delegated/review administration are Platform capabilities. Terminal USB access is client-only. |
| **riAuth Platform** | Every Essentials capability and its defaults, plus configurable workflows, broader protocols, advanced federation, cloud connectors, device integrations and expanded administration, including certificate login, external signing and the event map. | Demand-driven X extensions are separately scoped; inclusion of a protocol family does not promise every binding, direction or peer. No terminal USB stack in the server. |
| **riauthctl** | Remote administration of either build through its authorized management APIs; desired-state plan/apply, diagnostics and capability-aware commands. Optional separately packaged terminal CTAP2 USB support. | No identity store, local server, direct database access, local restore/migration/recovery, or alternative authorization engine. It cannot activate missing server capabilities or approve a human login as an agent. |

The default Essentials browser experience requires no workflow design and no
terminal handoff for ordinary account tasks. Advanced configuration belongs in
Platform; the shared browser components still serve its ordinary users.

### Essentials capabilities

| IDs | Desired contract |
| --- | --- |
| E01–E03 | OIDC issuer and OAuth authorization server with exact issuer/subject continuity, S256 code flow, refresh, service credentials and device grants. Preserve existing provider profiles: private-key client authentication, pinned JWT workload grants, access-token exchange, PAR, JAR, JARM, DPoP, resource audiences, pairwise subjects, signed/encrypted responses, restricted registration and logout. These are included, selectively configured features; no implicit/hybrid grants or unrestricted registration. |
| E04–E05 | Passkey enrollment, sign-in, additional authenticators, list/rename/remove/cancel and fresh verification; single-use ownership-verified first-admin setup; invitation acceptance and email verification entirely in the browser. |
| E06–E07 | Local password change/reset that preserves factors and account-type restrictions; TOTP enrollment/confirmation/replacement/removal and recovery-code rotation/use. Password recovery must not silently reset MFA or create a local password on an upstream-only account. |
| E08–E10 | Application catalogue and launch; session list/selected revoke/sign-out everywhere; remembered-consent list/withdrawal; browser device approval with application/scopes review; passkey-only administration with independent backup credentials and explicit emergency recovery. |
| E11 | Complete ordinary journeys on supported desktop/mobile browsers using keyboard, assistive technology and small screens. Browser/device support is an evidence-gated matrix, not an assertion that every authenticator works. |
| E12–E14 | Compact Applications, People, Groups and Security administration; application wizards for callbacks, group access, claims and secrets; safe diagnostics; shared groups/claims and server authorization across GUI, API and CLI. |
| E15–E17 | One authorized versioned management service with validation, conditional writes, immutable actor-bound plan/apply, idempotent retry receipts and redacted audit; consistent encrypted backups and isolated verified restore into the selected supported backend, including external-key/configuration recovery prerequisites. |
| E18–E19 | LDAP identity import and directory password validation, without copying directory passwords; outbound SCIM users/groups/lifecycle with durable delivery outcomes. |
| E21–E23 | Equivalent redb/PostgreSQL security contracts; integrated operation, probes, bounded background work and actionable diagnostics; safe upgrade, build transition, migration and recovery. |

**Optional ordinary federation (E20).** Both builds include ordinary upstream
OIDC code and OAuth JSON-identity source capability, inactive until explicitly
configured. When activated, sign-in, explicit verified linking/unlinking and any
required local factor must complete in the browser under the shared invariants.
Platform adds SAML sources, advanced trust/assurance management and configurable
federation stages. This is the adopted target E20/P06 boundary, not evidence of
completed browser source journeys or an unresolved approval request.

LDAP **import** in E18 is riAuth acting as a client of an external directory;
LDAP **serving** in P03 is an application binding/searching riAuth. Outbound
SCIM in E19 sends desired users/groups to a target; inbound SCIM in P04 accepts
provisioning from another system. These directions must be explicit in setup,
capability reports and compatibility documentation.

### Platform additions

| IDs | Desired contract and bounded initial profiles |
| --- | --- |
| P01 | Typed, versioned authentication, enrollment, recovery, consent and sensitive-action workflows; bounded executor, conditional policies/claims, templates and visual editor over the same canonical API representation. Custom stages have explicit permissions and bounded execution/data/network access; a stage cannot manufacture an authentication proof. |
| P02–P03 | SAML IdP/SP source browser Redirect/POST login and supported logout; LDAP read-only provider bind/search over verified TLS. Setup, key/certificate rotation, mapping and revocation need real-peer evidence. |
| P04–P05 | Inbound SCIM Users/Groups operations with resource-level versioning, declared attributes/filters and ownership; RADIUS PAP/RadSec/EAP-TLS with declared certificate and revocation behavior. Neither family implies unsupported extensions. |
| P06–P07 | Advanced federation/trust/assurance lifecycle and integrated proxy/forward-auth behavior, including origin/header/cookie/backend isolation and WebSocket tests. |
| P08–P10 | Direct supported Google Workspace authorization with an optional broker; Entra token acquisition/paging/group mapping/removals/rotation; operational connector controllers with mappings, tests, scoped jobs, schedules, bounded retries and durable outcomes. Runtime scheduling is a desired product capability, not authorization to create a schedule in this task. |
| P11–P13 | Real managed-device trust with verified provenance/freshness/binding; separately installable Windows login component with signed updates/uninstall/recovery and explicit connectivity-loss policy; temporary access approvals and dependent agent/device credentials with expiry and revocation. |
| P14–P16 | Scoped delegated human administration; exact-content multi-party approval separating author/reviewer/executor; safe policy explanations/simulation; versioned desired state for advanced resources; coordinated optional split roles and multi-node operation, with dashboards and diagnostics. |
| P17 | Signed push Shared Signals receive/send with explicit stream, issuer, audience, subject and event trust; honest durable delivery status. |
| P18–P19 | Explicit HTTPS client-certificate authentication with enrollment/mapping/rotation/revocation and no invented MFA claim; external signing with verified custody/failure behavior and authorized aggregate event-map administration. |

Initial profile exclusions in **both** builds where applicable: implicit/hybrid
OAuth, RFC 7592 registration management, automatic sector-identifier retrieval,
upstream encrypted ID tokens/private-key JWT until separately verified; SAML
SOAP/artifact/ECP/encrypted NameID; LDAP writes/additional schemas; RADIUS
accounting/CoA/PEAP/TTLS; SCIM bulk/sort/nested groups/custom enterprise schemas
until named peer requirements define them; SSF polling/stream verification/subject
management; outbound cloud-directory writes; managed gateway orchestration and
constrained credential injection. X01–X04 assess demand-driven additions.
These exclusions do not remove the supplied Essentials families or excuse an
incomplete supported setup/lifecycle/failure path.

## Shared invariants and measurable gates

The matrix's G01–G12 acceptance checks apply to every affected capability in
both builds, supported backends, architectures and interfaces. Every check
must have zero unexplained failures before that capability is advertised.

1. **Identity continuity (G01).** Preserve local IDs, source ownership/links,
   durable groups, subject overrides and pairwise seeds. Switching build names
   cannot create users or change issuer/subject. Equal email, username or DN is
   never sufficient authority to adopt/link an identity. Renames preserve IDs.
2. **One authorization model (G02).** Evaluate live actor authority, user/client
   eligibility, deny rules, groups, scopes, assurance/freshness and required
   device signals consistently. Administrator status does not bypass application
   policy. Agents remain scoped management principals and cannot impersonate
   end users, delegate human administration or escape ownership boundaries.
3. **Revocation (G03).** After a successful disable, credential/policy security
   transition or session/token revoke commits, the next affected online check
   must deny access, including refresh/exchange and dependent credentials.
   Re-enabling an account cannot revive revoked agents/devices/proofs/grants.
   In-flight ordering and node/cache coherence must be tested. Offline JWT
   validation can retain signed claims until expiry; riAuth does not promise to
   terminate an application's independent session. State these limits plainly.
4. **Credential protection (G04).** Use the same hashing, verification, WebAuthn
   UV/RP/origin checks, secret-file permissions, TLS trust and at-rest custody
   rules. Never weaken them to reduce footprint. Browser credentials stay out
   of JavaScript/local storage. Secrets are redacted from inventory, plans,
   audit and diagnostics, with explicit private one-time output where needed.
   Database encryption remains an explicit custody choice; durable email bodies
   contain proofs until delivery, so an unencrypted store is not advertised as
   encrypted. Backup encryption is mandatory.
5. **Bound proofs and atomic transitions (G05–G06).** Bind one-time state to
   purpose, actor/account, request, browser/session, issuer and current authority
   as applicable. Race two consumers: at most one successful consumption. Record
   each path's defined success, failure-consumption and rollback boundary; rejected
   input cannot undo spent state or revoke an unrelated actor's credential family.
   Fresh verification for sensitive factor changes is at most five minutes
   old and uses existing MFA when enrolled. Terminal-approved browser access
   does not authorize browser factor planting. Mail-scanner GETs cannot consume
   invitations/reset links. Failed/expired/cancelled operations grant no access.
6. **Management equivalence (G07).** GUI, API and riauthctl call the same service.
   Changes share validation, atomicity, revisions, permission revalidation,
   idempotent retry and audit. Approval binds exact content and dependencies;
   stale or unauthorized execution fails with no partial local mutation. Bind
   security-relevant resolved secret content through a protected commitment or
   immutable verified version; changed material requires a new review without
   revealing plaintext in the plan. Cached retry-result disclosure requires the
   principal's current permission to read that result/credential and current
   target/parent authority; it cannot become a revoked-actor secret-read bypass.
7. **Local and remote honesty (G08).** Commit local revocation and downstream
   delivery intent durably together. Track pending/succeeded/failed/ambiguous
   outcomes per target. Remote delivery is at least once unless a named peer
   supplies stronger guarantees; a local success never means all peers finished.
8. **Assembly, configuration and recovery safety (G09–G12).** Test exact official
   artifacts, missing modules, unavailable dependencies, rejected transitions,
   both storage backends, native architectures and verified restore. Redb has
   one owning process; no client/worker shares its database file. An older
   restore must invalidate restored sessions/proofs/grants by default before
   traffic resumes. Persistent agent/device/source credentials restored from
   older state require revocation reconciliation or explicit rotation/re-enrollment;
   clearing sessions alone cannot prevent their revival. Reconcile consumed factor/
   replay state, retry receipts and pending/uncertain remote jobs before serving
   or dispatching them. An older authenticated snapshot does not prove freshness.

Q01's accepted catalog supplies 35 stable invariants. This target adopts their
contracts while retaining their observed-enforcement/gap distinctions. The matrix
maps every capability to those invariants and TB-01–TB-11; the shared gate mapping is:

| A02 gate | Q01 stable invariant coverage |
| --- | --- |
| G01 | RI-ACC-001; RI-DIST-001; RI-STORE-003–004 |
| G02 | RI-ACC-002; RI-MGT-001/005; RI-AUTH-001–002 |
| G03 | RI-SES-004–005; RI-CON-003–004; RI-STORE-001 |
| G04 | RI-CRED-001–003; RI-SES-001; RI-STORE-002–003 |
| G05 | RI-SES-002–003; RI-WF-001–002; RI-DEV-001–002 |
| G06 | RI-CRED-002–003; RI-SES-001–002 |
| G07 | RI-MGT-001–005; RI-STORE-001 |
| G08 | RI-CON-002–004; RI-MGT-004 |
| G09 | RI-DIST-001–002 |
| G10 | RI-DIST-002–003; RI-AUTH-002; RI-WF-002 |
| G11 | RI-STORE-001–004; RI-MGT-003–004 |
| G12 | RI-DIST-001/003; capability-specific peer/device/proxy invariants |

## Runtime defaults and rationale

These are target defaults for both builds; current defaults are explicitly
identified. Compiling a capability does not enable a listener or initiate a
connector. Platform starts with the same ordinary experience as Essentials.

| Area | Target default | Rationale and boundary |
| --- | --- | --- |
| Assembly | Essentials or Platform identified explicitly; same revision/core; no USB/terminal stack in either server | Smaller dependency surfaces without divergent identity behavior. Target conventions; the separate builds do not exist today. |
| Process/storage | One integrated service with redb, one owner; PostgreSQL explicitly configured | Keeps ordinary deployment simple. Both backends are shared capabilities, not a Platform entitlement. PostgreSQL does not imply database HA. |
| First run | Fail closed; loopback-only bootstrap until ownership is verified with an expiring single-use proof; no anonymous reusable admin setup or default password | Complete browser bootstrap while retaining an explicit protected offline recovery path. This target is not current CLI initialization behavior. |
| Issuer/listen | Local evaluation `http://localhost:9000`, `127.0.0.1:9000`; production requires an explicit stable HTTPS issuer | Retains current development defaults. Reject external plaintext issuers/listeners; native TLS or an explicitly trusted proxy must match the issuer. Trusted proxies default to empty. |
| Browser | Shared embedded account/admin shells and protocol interaction pages on | Essentials remains usable without a terminal. Platform uses the same components and server-derived capability/permission-aware navigation. Target full coverage is incomplete today. |
| Client registration | No application, registration template, service trust or connector created automatically; code clients use S256; consent required unless a permitted first-party configuration says otherwise | Inclusion does not grant applications access. PAR/JAR/DPoP/encryption and workload/exchange trusts require explicit validated client settings. |
| Sessions/tokens | Access token 300 s, refresh lifetime 2,592,000 s, session 28,800 s; sensitive-change freshness at most 300 s | Preserve existing shared defaults; short access tokens bound offline revocation delay. No refresh token without permitted grant/scope/consent. These are lifetimes, not performance claims. |
| Passwords/factors | Existing password protections and five-history default; no automatic factor bypass; passkey-only users/admins supported without a password | Preserve credential semantics. Existing limit of 16 passkeys/account remains until reviewed quota changes. |
| Email proofs | Verification 24 h, reset 30 min, invitation seven days; single use; no SMTP until configured | Preserve current lifetimes while adding browser completion. A required mail-dependent path is unusable until valid delivery is configured; no successful delivery claim from enqueue alone. |
| Optional capabilities | LDAP import, SCIM targets, sources, and all Platform adapters/controllers inactive until explicitly enabled and configured | No accidental network exposure, trust or remote change. Connector jobs start only for an authorized approved plan or explicit automatic-mode policy. |
| Reconciliation | Manual reviewed plan/apply; guarded automatic/automatic modes require explicit scoped policy and removal safeguards | All modes use one engine. Shared LDAP/outbound SCIM do not become Platform-only; Platform adds broader controller orchestration. |
| Audit | Shared redacted audit enabled; retain the current 90-day default | Core security events are never silently switched off with a UI option. Retention/exports need workload/quota evidence; retention is not compliance certification or tamper-proof archival. |
| Backups/keys | Authenticated encrypted backups; explicit key custody; live-record encryption explicitly configured; no automatic backup schedule | Avoid hidden restore prerequisites. Target quotas and backup delivery format must be set and tested; today's 64 MiB limit is current behavior, not a new capacity promise. |
| Workflow/distribution | Fixed safe ordinary flows; no custom workflow, split role or multi-node activation by default | Keep Essentials straightforward and avoid exposing unfinished Platform topology as ready. |
| Recovery | Fresh sufficient verification and independent recovery credentials; invalidate restored sessions/proofs/grants and reconcile persistent credentials/replay/jobs before use | Prevent ordinary recovery or an old snapshot from silently restoring revoked authority. Implementation and runtime demonstration remain required. |
| Windows offline access | Off; enabling disconnected access requires a named operator requirement and an explicit bounded expiry/revocation-delay/recovery policy | A disconnected device cannot learn immediate revocation. The current protocol's limits do not establish a supported Windows component. |

### Browser and headless operation

Headless is an explicit runtime choice in either distribution, not a third
product or a reduced Essentials build. Default shared operation serves the
account, compact administration and protocol interaction components from the
server binary without a separate frontend service or external CDN.

The target **headless management** mode suppresses account/admin catalogue
shells and their navigation but keeps required authentication, consent,
enrollment/recovery and device-decision interaction pages for enabled interactive
flows. Remote API/riauthctl administration remains authorized normally.
Protocol metadata cannot advertise an interactive path whose UI is absent.

A deployment may disable **all** embedded HTML only when its configuration is
service/API-only: reject browser-dependent grants, interactive source stages,
invitations/browser setup or other required interactive paths before activation.
An external UI is not currently established and is not a substitute prerequisite
claimed by this contract. Any future external interaction mode needs its own
binding/security and end-to-end acceptance evidence. Preserve stored configuration
when disabling shells and re-enabling them; do not rewrite users or factors.

## Compiled, enabled, configured and usable

Capability reports must distinguish these states per capability/profile and per
configured instance, in both operator inspection and machine-readable metadata:

| State | Required meaning |
| --- | --- |
| **Compiled** | Code and dependencies for this bounded profile are in this exact artifact; it says nothing about runtime activation or completed product verification. |
| **Enabled** | Operator activation policy permits this module/instance in this process role. An enabled module with no configured instances is allowed but not advertised as usable. |
| **Configured** | Required settings, referenced capabilities and custody prerequisites have passed structural/security validation. External peer availability is a separate operational state. |
| **Usable** | Compiled, enabled and configured for the requested profile/role, required interaction paths available and local dependencies ready. Caller permissions still apply. Remote outages surface degraded/unavailable status rather than a false success. |

Report build/revision, exact profiles/directions, backend/platform, instance
state and redacted reasons. Public protocol discovery includes only enabled,
locally usable protocol functionality, not the union of compiled modules.
Operator inventory may show compiled inactive functionality. The GUI hides or
explains unavailable actions and still relies on server checks. riauthctl fails
unsupported operations before submission when possible; the server remains
authoritative for stale/malicious clients. Static roadmap inclusion is never
used as runtime discovery.

### Configuration rejection

Validate startup and every live configuration/manifest change against the exact
assembly, process role and all stored dependency references before activation.
Reject unknown keys, excluded/uncompiled modules (even a dormant stanza), enabled
instances without required settings, active resources requiring disabled modules,
unavailable protocol profiles, missing required secrets/keys, invalid TLS/issuer
settings, workflow/federation/device policy dependencies, and UI/grant conflicts.
Use actionable field/resource/capability errors without secret values. Do not
silently ignore settings, drop policy stages, fallback to weaker authentication,
or substitute a different backend. An invalid live update leaves the previous
validated configuration active, unchanged and auditable; an invalid first startup
does not accept traffic. Remote transient outage is operational degradation, not
permission to disable policy or treat an incomplete import as empty.

## Safe build, backend and version transitions

1. Preflight the exact source and target artifacts, schema/backup formats,
   configuration, stored resources, active sessions/workflows and queued jobs.
   Produce blockers with capability/resource references and a reviewed migration
   plan before any store/configuration mutation.
2. Essentials → Platform preserves IDs, subjects, issuers, groups, credentials,
   authorization and revocation state. New adapters remain inactive. Platform →
   Essentials blocks if any active **or retained** configuration/resource/job
   requires an excluded capability. No automatic policy simplification, identity
   deletion, key loss or queue abandonment is permitted.
3. A deliberate conversion must explicitly retire incompatible policies/jobs,
   explain lost behavior, preserve an authenticated export and audit, and pass
   preflight again. Inert opaque preservation is allowed only after a documented
   tested migration format exists; it is not the default downgrade escape hatch.
4. Stop incompatible writers; redb remains single-owner. Backend changes copy
   into an isolated target, validate identity and credential equivalence, and
   preserve the original for recovery. Node cohorts must agree on schema,
   capabilities and security settings; mixed operation is denied unless the exact
   version pair has verified compatibility and feature-activation rules.
5. Verify readiness, admin access, allowed/denied application flows, refresh,
   revocation and jobs before traffic. Rollback uses a compatible pre-transition
   backup in an isolated target; never assume an older binary can open newer
   data. Older-state restore invalidates restored sessions, pending proofs and
   grants by default and requires explicit reauthentication/reconciliation.
   Restored persistent credentials also require validated post-snapshot revocation
   reconciliation or rotation/re-enrollment before use; current restore's preserved
   state cannot establish this guarantee.

G01–G03 and G09–G12 measure these transitions. A rejected downgrade must leave
canonical identity/configuration/credential/job state unchanged and start no
listener or downstream job; diagnostics alone may record the attempt.

## Client, server and maintenance responsibilities

The `riauth` server owns protocol validation, identity, authentication ceremonies,
authorization, atomic mutations, secret access, audit, storage and durable jobs.
Browser components and riauthctl submit intent; they never decide permissions.

riauthctl C01 uses authenticated HTTPS (loopback HTTP only for local evaluation),
issuer-bound private credentials, explicit trust roots, no redirect-following,
bounded request timeouts and capability-aware commands. Preserve the current
30-second request deadline as a client default; long operations may use an
explicit bounded override. Retries preserve idempotency keys and report ambiguity.
It can administer advanced resources only when the server supports and authorizes
them. C02's optional CTAP2 USB package requests PIN/touch locally; USB dependencies
and permissions do not enter server/container/base-client artifacts. Platform
keychains, Bluetooth and hybrid phone transport are not promised for this CLI.
Browser authenticator transport support remains a separate browser matrix.

`riauth-maintenance` T01 is the separate offline/local executable for init,
restore, migration, index/schema work and emergency admin recovery. It uses the
same validators/formats and requires explicit local authority, private inputs
and exclusive access or an isolated target. It must refuse a live-store conflict,
record recovery actions, preserve factors unless an explicit authorized emergency
procedure resets them, and require post-recovery verification. It is not remote
riauthctl and is not an unauthenticated server endpoint. A01 C31 agrees with this
server-maintenance responsibility for both builds.

### Adopted implementation conventions

These conventions remove routine naming/assembly blockers. They are future
implementation targets; today's Cargo manifest, binaries, commands and settings
do not implement them. No example here authorizes or implies running a future
command against the current binary.

| Package/build convention | Future executable/capability relationship |
| --- | --- |
| `riauth-essentials` | Server executable `riauth`; additive capability feature `essentials`; all shared Essentials security and browser contracts. |
| `riauth-platform` | Server executable `riauth`; additive feature `platform` includes `essentials` and Platform adapters. These are alternative server packages for one deployment, built from the same revision. |
| `riauthctl` | Remote-only executable/package `riauthctl`; no server/store/local-maintenance or USB dependencies in the base client. |
| `riauthctl-usb` | Opt-in client build/package variant delivering `riauthctl` with terminal USB support; replaces the base client package rather than installing a second conflicting binary. No USB stack in server/containers. |
| `riauth-maintenance` | Separate local executable/package compatible with each target server assembly/schema; never grants a remote client direct database authority. |

Capability features are additive availability controls, not different identity
implementations or permission grants. `platform` implies `essentials`; neither
feature removes shared checks. Exact internal crate boundaries and configuration/
capability wire names are routine implementation choices under the frozen state,
rejection and responsibility contracts, not pending product-approval gates.

## Supported-platform targets and release gate

Linux **x86-64 and ARM64** are required target architectures for both official
server builds, their container variants, riauthctl and maintenance tools (T02).
Use `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu` as target native
triples, with `linux/amd64` and `linux/arm64` OCI variants. Minimum distro/libc
versions require a named deployment baseline and native evidence (EG02) before a
support claim. ARM64 requires
execution on an ARM64 runner/device; a successful cross-build is insufficient.

For every required artifact, test installation/startup, HTTPS/discovery,
management, passkey server verification, revocation, configuration rejection,
build transitions and backup/restore. Check dependency inventory: no terminal
USB stack in either server or base client. Test the optional USB client on each
OS/architecture actually claimed, using named physical hardware and permissions.
Publish verifiable signatures, checksums, dependency inventories, notices and
truthful provenance with the exact tested bundle. Current checksums/provenance
are not cryptographic attestation, signing or an SBOM.

macOS and Windows server/client/maintenance releases are not declared supported
by this contract. Windows login integration P12 is a distinct Platform component,
not a Windows server-support claim. Browser/Windows versions and USB hardware
are EG02/EG03 evidence-gated requirements. Existing source builds are not released
artifact support. No performance, recovery-time, conformance or certification
result is invented; Q03/Q04/Q08–Q11 and backlog D05 supply the relevant evidence later.

## Acceptance, resolved choices and later evidence gates

Every matrix row supplies checks, evidence paths and backlog traceability;
implementation acceptance requires observed results, not merely these criteria.
Use the same fixtures and rejection cases across distributions, backends and
interfaces, then run the exact packaged artifacts on both native architectures.
Record pass/fail/skipped and external prerequisites. Real-peer tests are required
for advertised integrations; mocks cannot certify them. Performance reports must
record workload, dataset, hardware, security settings, peak RSS, p50/p95/p99,
successful throughput, errors and background interference. Product load/RPO/RTO
targets require named operator workloads and measurements (EG04); there is no
promised numeric speed or certification here. The 46 capability units have 138
scenario checks plus 12 shared gates. Their definitions are frozen; their runtime
results remain unverified until the authorized implementation/test work runs.

DEC01 is resolved: retain exactly the supplied Essentials scope, include optional
ordinary federation E20 in both builds, and keep advanced federation, certificate
login and external signing integration in Platform. DEC06 is resolved by the
package/executable/additive-feature conventions above. A01/Q01 documentation
acceptance is recorded by the finalization assignment. None is a new user approval
gate or an assembly blocker.

The remaining gates govern supported-peer, hardware, measured-operation and
runtime-enforcement claims, not semantic contract freeze. EG IDs are separate
from the original backlog task IDs; they replace the draft's DEC02–DEC05 blockers.

| Evidence gate | Required later evidence or named requirement |
| --- | --- |
| EG01 — Protocol peers and certification | Name the application/directory/IdP/proxy/network/cloud tenant and exact version/profile/direction, then exercise setup, lifecycle, negative inputs and failure handling. Independent conformance/certification needs precisely scoped results. Extensions require a named peer requirement; no family-wide claim from mocks. |
| EG02 — Platform, browser and hardware support | Execute exact Linux x86-64/ARM64 packages/containers natively; select distro/libc baselines from named operator deployments and test them. Publish browser/OS/assistive-technology and physical USB/device matrices; choose additional targets only from named requirements. |
| EG03 — Runtime security and recovery | Q02/Q05 and installed-artifact tests demonstrate the fixed Q01/A02 bootstrap, fresh-factor/independent-recovery, exact approval/retry authority, revocation/concurrency, headless/config rejection and restored-state invalidation/reconciliation scenarios. Windows disconnected operation remains off unless a named operator requirement defines and proves a bounded policy. Missing tests do not weaken or deliver these defaults. |
| EG04 — Measured operation | Named operator workloads/datasets set quotas, connector lag/service budgets and RPO/RTO. S/O/R/Q record actual memory, latency, throughput, error, outage and recovery results at equivalent security settings; retain current documented limits until changed and verified. |

User priority meanings remain: **P0 foundations/security invariants; P1 mainstream
product completeness; P2 advanced integrations and operation; P3 demand-driven
extensions**. Recommendations do not redefine them. Preserve the supplied
implementation sequence: (1) A + Q foundation; (2) U + M everyday experience;
(3) W + P with selected I; (4) S + O + R operation at scale; (5) G + Q + D
deployment readiness; (6) X for actual demand. Security work starts in phase 1
and continues throughout. This A02 target contract neither implements architecture nor
delivers the remaining backlog, and RiWork completion belongs to orchestrator
review of the diff and actual validation.
