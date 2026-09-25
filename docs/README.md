# Documentation

Choose a path for v0.1.1:

- **New to riAuth?** [Get started locally](getting-started.md), then read the [project README](../README.md) and [release notes](release-notes.md).
- **Signing in or managing your account?** Use [My applications](PORTAL.md), [passkeys](passkeys.md), and [account lifecycle](lifecycle.md).
- **Operating or migrating an instance?** Start with [operations](operations.md), [availability](availability.md), and [migration](migration.md).
- **Connecting an application?** Follow [OIDC profiles](oidc-profiles.md), [SAML](saml.md), or [proxy SSO](proxy.md), with the [API reference](api.md) when needed.
- **Contributing?** Read [CONTRIBUTING](../CONTRIBUTING.md), [architecture](architecture.md), and [testing](testing.md).

Read [release limitations](limitations.md) before a production cutover. The [testing guide](testing.md) separates local checks from application and deployment validation.

## Start and operate

| Guide | Purpose |
| --- | --- |
| [Getting started](getting-started.md) | Install, start a local instance, sign in, and register an OIDC web application |
| [Project README](../README.md) | Install, initialize and try browser and terminal sign-in |
| [Architecture](architecture.md) | Components, request paths, state and worker boundaries |
| [Operations](operations.md) | TLS, probes, metrics, backups, restore, maintenance and releases |
| [Configuration example](../examples/riauth.toml) | Base configuration; optional feature blocks are in their guides |
| [Availability](availability.md) | redb, PostgreSQL, shared state and failover boundaries |
| [External signing](kms.md) | Vault Transit keys and custody limits |
| [Migration](migration.md) | General Authentik import, continuity, cutover and rollback guidance |
| [Release compatibility](release-notes.md) | Behaviour changes, schema and backup compatibility, artifacts and rollback |
| [Testing](testing.md) | Local checks and deployment validation |
| [Release limitations](limitations.md) | Unsupported profiles and deployment responsibilities |

## Interfaces and identity

| Guide | Purpose |
| --- | --- |
| [Agent administration](agent.md) | Permissions, manifests, plans, atomic mutations and private output |
| [API](api.md) | HTTP routes, authentication and error contracts |
| [OIDC profiles](oidc-profiles.md) | OAuth/OIDC grants, claims, keys and upstream sources |
| [Portal](PORTAL.md) | Browser and terminal sign-in, passkey management, application launch, policy and event-map page |
| [Passkeys](passkeys.md) | Browser passkeys, USB and split WebAuthn ceremonies |
| [Account lifecycle](lifecycle.md) | Invitations, email verification, password recovery and what still needs the terminal |
| [SAML](saml.md) | IdP/source profiles, signatures, encryption and logout |
| [SCIM](scim.md) | Inbound directory API and outbound reconciliation |
| [LDAP synchronization](ldap.md) | Upstream directory plans and password authentication |
| [LDAP provider](ldap-provider.md) | Read-only LDAP listener and search behavior |
| [RADIUS](radius.md) | PAP, RadSec, EAP-TLS, certificate lifecycle and policy |
| [Proxy SSO](proxy.md) | nginx and Traefik forward auth, shared cookies and the embedded reverse proxy |

## Advanced features

| Area | Guides |
| --- | --- |
| Access and assurance | [Temporary group access](enterprise/ENT-01.md), [parent-owned agents](enterprise/ENT-02.md), [HTTPS client-certificate login](enterprise/ENT-05.md), [device-trust signal](enterprise/ENT-06.md), [password history](enterprise/ENT-08.md) |
| Directory and lifecycle | [Google Workspace sync](enterprise/ENT-03.md), [Microsoft Entra ID sync](enterprise/ENT-04.md), [scheduled offboarding](enterprise/ENT-10.md), [upstream source stages](enterprise/ENT-11.md), [outbound SCIM OAuth](enterprise/ENT-12.md) |
| Signals and administration | [Shared Signals](enterprise/ENT-07.md), [audit review](enterprise/ENT-09.md), [Windows device-login protocol](enterprise/ENT-13.md), [Events Map](enterprise/ENT-14.md), [CSV export](enterprise/ENT-15.md), [deployment alert routing](enterprise/PLATFORM-04.md) |

SAML XML handling uses Rhein Industries' maintained [risaml](https://github.com/Rhein-Industries/risaml), [ribergshamra](https://github.com/Rhein-Industries/ribergshamra), [ritsp-ltv](https://github.com/Rhein-Industries/ritsp-ltv), and [riptering](https://github.com/Rhein-Industries/riptering) forks. Their source and licenses are documented in [third-party notices](../THIRD_PARTY_NOTICES.md).
