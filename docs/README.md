# Documentation

Choose a path for v0.1.1:

- **New Essentials operator or user?** Follow the [Essentials guide](essentials-guide.md). Its small install uses the Essentials binaries and loopback redb.
- **New Platform operator or user?** Follow the [Platform guide](platform-guide.md). It walks the same tasks on the Platform binaries, then one configured workflow, SAML, and the LDAP provider listener.
- **Undifferentiated local walkthrough?** [Get started locally](getting-started.md) uses the Cargo default feature set, `platform`. Then read the [project README](../README.md) and [release notes](release-notes.md).
- **Signing in or managing your account?** Use [My applications](PORTAL.md), [passkeys](passkeys.md), and [account lifecycle](lifecycle.md).
- **Operating or migrating an instance?** Start with [operations](operations.md), [availability](availability.md), [migration](migration.md), and [re-enrollment](reenrollment.md).
- **Connecting an application?** Follow [OIDC profiles](oidc-profiles.md), [SAML](saml.md), or [proxy SSO](proxy.md), with the [API reference](api.md) when needed.
- **Contributing?** Read [CONTRIBUTING](../CONTRIBUTING.md), [architecture](architecture.md), and [testing](testing.md).

Read [release limitations](limitations.md) before a production cutover. The [testing guide](testing.md) separates local checks from application and deployment validation.

## Start and operate

| Guide | Purpose |
| --- | --- |
| [Essentials guide](essentials-guide.md) | Essentials operator and user tasks: small install, first administrator, one OIDC app, passkey self-service, backup and recovery entry points, groups, claims, audit review, LDAP import, and outbound SCIM |
| [Platform guide](platform-guide.md) | The same tasks on the Platform binaries, with shared semantics and the remote-client split, then a configured workflow, SAML IdP and source, and an LDAP provider listener |
| [Getting started](getting-started.md) | Undifferentiated local walkthrough; `cargo install --locked --path .` selects the default `platform` feature |
| [Project README](../README.md) | Install, initialize and try browser and terminal sign-in |
| [Architecture](architecture.md) | Components, request paths, state and worker boundaries |
| [Operations](operations.md) | TLS, probes, metrics, backups, restore, maintenance and releases |
| [Linux image deployments](deployment-examples.md) | Single-owner redb and two-host PostgreSQL Compose examples with operator steps |
| [Server editions](editions.md) | Essentials and Platform build commands, assembly behavior and preview limits |
| [Capability matrix](capability-matrix.md) | Compiled Essentials, Platform, and riauthctl inclusion, runtime prerequisites, protocol direction, tested peers, and known limits |
| [Configuration example](../examples/riauth.toml) | Base configuration; optional feature blocks are in their guides |
| [Availability](availability.md) | redb, PostgreSQL, shared state and failover boundaries |
| [Disaster recovery](disaster-recovery.md) | What to keep outside backups, binary choice, restore order, validation and unrecoverable cases for Essentials and Platform |
| [Operational recovery](operational-recovery.md) | Which recovery procedure to run, with preflight, break-glass, failure stops, and the recorded local-drill boundary |
| [Connector dependency incidents](connector-incidents.md) | Read, stop, and retry boundary for LDAP, outbound SCIM, Workspace, Entra, SMTP, Vault Transit, and alert webhooks |
| [Administrator lockout](admin-lockout.md) | Attempt locks, lost recovery codes, and browser recovery while a second administrator can still sign in |
| [Restored-state recovery](recovery.md) | Session, proof and grant invalidation after restores, the serving gate and PostgreSQL recovery duties |
| [External signing](kms.md) | Vault Transit keys and custody limits |
| [Migration](migration.md) | Authentik import, preflight for other source systems, continuity, cutover and rollback guidance |
| [Re-enrollment](reenrollment.md) | Authentik operator decision table and copy-ready notices for passkeys, factors, recovery, sessions, and rollback |
| [Release compatibility](release-notes.md) | Behaviour changes, schema and backup compatibility, artifacts and rollback |
| [Testing](testing.md) | Local checks and deployment validation |
| [Q07 parser assurance](q07-parser-assurance.md) | Bounded parser/dependency checks, a local SCIM parser repair, and remaining assurance work |
| [Q08 exact edition matrix](roadmap/q08-exact-edition-bundles.md) | Local Essentials/Platform build, configuration and storage evidence; Linux release artifact gates |
| [Q09 benchmark slice](roadmap/q09-benchmark-slice.md) | One local session-read measurement protocol, its fixture check, and the open benchmark gate |
| [Q10 installed release gate](roadmap/q10-installed-release-gate.md) | Native packaged-binary transition and recovery gate, local preflight evidence, and remaining external checks |
| [Q03 independent OIDF pilot](roadmap/q03-conformance-pilot.md) | Pinned runner preflight, missing private pilot inputs, and the open independent run gate |
| [Release limitations](limitations.md) | Unsupported profiles and deployment responsibilities |

## Interfaces and identity

| Guide | Purpose |
| --- | --- |
| [Agent administration](agent.md) | Permissions, manifests, plans, atomic mutations and private output |
| [API](api.md) | HTTP routes, authentication and error contracts |
| [OIDC profiles](oidc-profiles.md) | OAuth/OIDC grants, claims, keys and upstream sources |
| [OIDC relying-party recipe](recipes/oidc-relying-party.md) | In-tree public client `rp`, the assertions the ignored browser test makes, and the peer gaps that fixture leaves |
| [Upstream OIDC recipe](recipes/upstream-oidc.md) | In-process issuer fixture, the assertions that test makes, and the Okta, Entra, and Google gap that fixture leaves |
| [Portal](PORTAL.md) | Browser and terminal sign-in, passkey management, application launch, policy and event-map page |
| [Passkeys](passkeys.md) | Browser passkeys, USB and split WebAuthn ceremonies |
| [Account lifecycle](lifecycle.md) | Invitations, email verification, password recovery and what still needs the terminal |
| [SAML](saml.md) | IdP/source profiles, signatures, encryption and logout |
| [Platform SAML IdP recipe](recipes/platform-saml-idp.md) | Fixture SP entity, the assertions the ignored xmlsec1 test makes, and the peer gaps that fixture leaves |
| [Platform SAML source recipe](recipes/platform-saml-source.md) | In-process IdP fixture, the assertions `exercise` makes, and the named-IdP gap that fixture leaves |
| [SCIM](scim.md) | Inbound directory API and outbound reconciliation |
| [Platform inbound SCIM recipe](recipes/platform-inbound-scim.md) | In-process HTTP fixture, the assertions that test makes, and the named-client gap that fixture leaves |
| [LDAP synchronization](ldap.md) | Upstream directory plans and password authentication |
| [LDAP import recipe](recipes/ldap-import.md) | Disposable OpenLDAP fixture, the assertions the ignored import test makes, and the peer gaps that fixture leaves |
| [LDAP provider](ldap-provider.md) | Read-only LDAP listener and search behavior |
| [Platform LDAP-provider recipe](recipes/platform-ldap-provider.md) | Fixture LDAPS and STARTTLS listener, ldap3 search and denial results, and the peer gaps that fixture leaves |
| [RADIUS](radius.md) | PAP, RadSec, EAP-TLS, certificate lifecycle and policy |
| [Proxy SSO](proxy.md) | nginx and Traefik forward auth, shared cookies and the embedded reverse proxy |
| [Platform forward-auth recipe](recipes/platform-forward-auth.md) | nginx and Traefik fixture configuration, asserted redirect and revocation results, and the peer gaps those fixtures leave |
| [Workflow model](workflows.md) | Typed definitions, Platform authoring and validation, Essentials defaults, and bounded verifier paths |

## Advanced features

| Area | Guides |
| --- | --- |
| Access and assurance | [Temporary group access](enterprise/ENT-01.md), [parent-owned agents](enterprise/ENT-02.md), [HTTPS client-certificate login](enterprise/ENT-05.md), [device-trust signal](enterprise/ENT-06.md), [password history](enterprise/ENT-08.md) |
| Directory and lifecycle | [Google Workspace sync](enterprise/ENT-03.md), [Microsoft Entra ID sync](enterprise/ENT-04.md), [scheduled offboarding](enterprise/ENT-10.md), [upstream source stages](enterprise/ENT-11.md), [outbound SCIM OAuth](enterprise/ENT-12.md) |
| Signals and administration | [Shared Signals](enterprise/ENT-07.md), [audit review](enterprise/ENT-09.md), [Windows device-login protocol](enterprise/ENT-13.md), [Events Map](enterprise/ENT-14.md), [CSV export](enterprise/ENT-15.md), [deployment alert routing](enterprise/PLATFORM-04.md) |

SAML XML handling uses Rhein Industries' maintained [risaml](https://github.com/Rhein-Industries/risaml), [ribergshamra](https://github.com/Rhein-Industries/ribergshamra), [ritsp-ltv](https://github.com/Rhein-Industries/ritsp-ltv), and [riptering](https://github.com/Rhein-Industries/riptering) forks. Their source and licenses are documented in [third-party notices](../THIRD_PARTY_NOTICES.md).

[Connector removal safeguards](removal-safeguards.md) describes snapshot validation, thresholds and exact-plan confirmation.
