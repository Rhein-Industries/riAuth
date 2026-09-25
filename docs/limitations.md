# v0.1 release scope and limitations

riAuth v0.1.1 is an early release for evaluation and controlled pilots. It is not OpenID certified, and compatibility with a particular application, identity provider, directory, browser, or network device requires testing in that environment. The [testing guide](testing.md) lists the checks to record before a cutover.

| Area | Current boundary |
| --- | --- |
| OIDC | Authorization code with S256 PKCE and the documented service/device profiles are available. Independent conformance results for the advertised profiles have not been published. |
| SAML | The browser Redirect/POST profile is implemented. SOAP, artifact binding, ECP, and encrypted NameIDs are outside this profile; test the selected service provider's login and logout behavior. |
| Authentik migration | The offline importer does not transfer live sessions, opaque tokens, passkeys, arbitrary Python mappings, or unsupported proxy-provider settings. Recreate unsupported integrations and rehearse account continuity and rollback. |
| Directories and provisioning | The LDAP provider is read-only. Outbound SCIM needs target-specific testing. Scheduled offboarding revokes local access; downstream deprovisioning remains an operator action. |
| Shared Signals | Signed push events are implemented. Polling, stream verification, and subject-management endpoints are not included in v0.1. |
| Network and devices | RADIUS PAP, RadSec, and EAP-TLS have local coverage; accounting, CoA, PEAP, and TTLS are not supported. A packaged Windows credential provider is not included. Test actual supplicants, certificates, and managed devices separately. |
| Availability and recovery | redb has one owning service process. PostgreSQL permits multiple service processes, while database replication, election, and fencing remain deployment responsibilities. Backups have a 64 MiB archive limit; restore creates a new redb instance, including when the backup came from PostgreSQL. |

The [protocol guides](README.md#interfaces-and-identity) give the exact supported behavior. The [operations guide](operations.md) covers key custody, backup and restore, probes, and upgrade order. Local regressions and disposable fixtures cannot establish recovery objectives or interoperability with your production peers.
