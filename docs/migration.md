# Authentik migration

`import-authentik` is an offline converter and blocker report, followed by the ordinary plan/apply workflow. It does not connect to or modify Authentik. A successful conversion means the input is ready for a server plan, **not that an application cutover has passed**.

For a new instance without Authentik data, follow [getting started](getting-started.md). For a migration, keep Authentik available for rollback, initialize riAuth at its intended issuer, and take a verified backup before applying an import. Keep exports and generated credentials in private storage.

| Stage | Review before continuing |
| --- | --- |
| Export | Confirm every Authentik collection is complete and record the current application, issuer and subject contracts. |
| Convert | Run `import-authentik --preflight`, then `import-authentik --out`; review every classified item and resolve every blocker in the private report. A draft is not an applicable manifest. For other source systems, see [other source systems](#other-source-systems). |
| Plan and apply | Review the manifest and redacted plan against a disposable riAuth instance before applying to the intended instance. |
| Rehearse | Test each application, factor, logout and rollback path with its real peer. Record the result and rollback owner in your deployment inventory. |

## Build the input bundle

Inspect `riauth --json schema authentik-import`. The bundle uses `api_version: "riauth.authentik-import/v1"`, a target `issuer`, and these complete Authentik API exports:

| Bundle field | Authentik API collection |
| --- | --- |
| `users` | `/api/v3/core/users/` |
| `groups` | [`/api/v3/core/groups/`](https://api.goauthentik.io/reference/core-groups-list/) |
| `providers` | `/api/v3/providers/oauth2/` |
| `applications` | `/api/v3/core/applications/` |
| `policy_bindings` | [`/api/v3/policies/bindings/`](https://api.goauthentik.io/reference/policies-bindings-list/) |
| `sources` | [`/api/v3/sources/all/`](https://api.goauthentik.io/reference/sources-all-list/) |

Collect every page into an array. A single complete `results` response is accepted only when its pagination reports a complete collection. Incomplete pages are rejected. Supply empty arrays only when the inventory establishes that the collection is empty. Keep the original exports private: provider responses can contain client secrets. The converter uses the [Authentik user serializer](https://github.com/goauthentik/authentik/blob/main/authentik/core/api/users.py) and [OAuth provider serializer](https://github.com/goauthentik/authentik/blob/main/authentik/providers/oauth2/api/providers.py) field names.

Add `passwords`, keyed by username, containing `reference`, `version`, and optional `hashed: true`. References point to an environment variable or a private file on the migration agent. The normal user API does not return password hashes. Obtain them through an explicitly authorized offline export, or supply newly established passwords through a managed reset process.

Add `clients`, keyed by client ID. Each entry requires:

- Exact `issuer` from that provider's discovery document and the scopes to preserve.
- Reviewed `settings` containing declarative claims and policy translations.
- `translated_mapping_ids` and `translated_binding_ids` identifying reviewed source mappings/bindings. Every exported mapping/binding must be accounted for; expressions are never executed or automatically deemed equivalent.
- `authentication_flow_reviewed: true` only after checking browser and terminal login, consent and MFA requirements; set `require_mfa` and scope rules accordingly. Until then the report carries the blocker "authentication flow has not been reviewed for browser/terminal login and MFA".
- `federation_reviewed: true` only after translating exported JWT federation trust into pinned `machine_trust`, `exchange` and `token_managers` settings. Exported encryption keys require reviewed recipient keys for both ID and access tokens.
- For confidential clients, `secret_ref` and `secret_version` pointing to the existing secret, or a deliberate new secret coordinated with the RP.

The importer carries over exact redirects, supported grant permissions, token lifetimes, ID-token claim placement and back-channel logout URLs. An explicitly narrower `settings.allowed_grants` can select the supported subset needed by your RP; it cannot add grants absent from a nonempty source list. Confirm that restriction before cutover. Authentik exports `grant_types` from 2026.5; for an older export without it, or with an empty list, the report blocks until `settings.allowed_grants` inventories the grants the RP uses. Regex redirects and unsupported provider/source behavior produce blockers. Configuration containing SAML/provisioning providers or upstream identity sources cannot be treated as OIDC-only parity. Authentik proxy (forward-auth) providers are not converted: an application that uses one produces the blocker "uses an unexported or unsupported provider", and it is recreated as a riAuth [proxy client](proxy.md). Regex redirects must become exact registrations before export; a client can register up to 32.

## Preflight report

`riauth-maintenance import-authentik --file <bundle> --preflight` converts the bundle in memory and prints the classified findings without writing any file. The legacy `riauth import-authentik` command remains available. `--out <new directory>` writes the same findings to the private `report.json`, plus `manifest.json` when ready. Both outputs contain `ready_for_plan`, `blockers` and `summary`, a count per classification plus the number of blocking items. `report.json` and `--preflight` also list every finding in `items`. `riauth --json schema migration-finding` describes one item:

| Field | Meaning |
| --- | --- |
| `kind` | `user`, `password`, `totp`, `passkey`, `session`, `subject`, `group`, `source`, `provider`, `authentication_flow`, `property_mapping`, `policy_binding`, `federation`, `signing_key`, `encryption_key`, `client_secret`, `grant`, `token_lifetime`, `redirect_uri`, `logout`, `application`, `manifest` or, in an inventory report only, `source_native` |
| `source_kind` | Present only on `source_native` findings: the source system's own element type, such as a Keycloak `realm` |
| `id` | Exported identifier (username, group name, source/binding pk, client ID or application slug), scoped as `client/item` for mappings, redirects and federation, and `username/client` for subjects. `*` covers every user. |
| `classification` | `exact`, `convertible`, `manual` or `unsupported` (below) |
| `blocking`, `blocker` | Whether this item keeps `ready_for_plan` false, and its entry in `blockers` |
| `reason`, `action` | Why it has this classification, and what to do next |

- **exact**: carried over unchanged. Examples: strict redirect URIs, grant lists, supported subject modes, logout URIs, and group names without parents.
- **convertible**: carried over through a documented transformation. Examples: flattened parent groups, `authentik-<pk>` local IDs, duration strings, logout redirects, application portal metadata, the built-in source, referenced password hashes and TOTP secrets.
- **manual**: depends on reviewed operator input or an explicit decision. A manual item is non-blocking once that input is present. Examples: client resolutions, authentication flows, property mappings, policy bindings, federation, client secrets, source resolutions and newly established passwords. Group attributes and Authentik superuser status or roles granted through groups are non-blocking manual findings. They are not copied or promoted, so grant any riAuth permissions explicitly.
- **unsupported**: cannot be represented or transferred. Examples: regex redirects, unknown subject modes or logout methods, non-OAuth2 providers, provisioning providers, sources without an adapter, subjects outside the OIDC format and duplicate subjects. Passkeys, and live sessions and tokens, always appear as non-blocking unsupported items, because they cannot move and users must sign in and enroll again.

A provider without a `clients` entry is still classified item by item, as if its review were empty. Its provider finding, authentication flow, and any exported mappings, federation, encryption key or confidential client secret block, while intrinsic items such as strict redirects, grants and the subject mode keep their normal classification. Its application is classified from the export but blocks until the provider is reviewed. Every application that shares a provider also gets its own blocking finding. Duplicate subjects are checked for every exported provider. No client, and no per-client subject, is converted from an unreviewed provider. A `clients` or `source_resolutions` entry that names nothing in the export blocks and is not applied, including any binding IDs it translates.

Classifications describe what the converter does with the export. They are not evidence that an application, factor or credential works after cutover. Password hashes and TOTP secrets are referenced but never read during conversion. Their formats are checked only when the plan is applied. Items never contain credential values. Findings are sorted by kind and identifier. `blockers` keeps its earlier messages, with three changes: source blockers name the source family and the adapter it needs; rejected or stale resolutions add their own blockers; and an unreviewed provider also reports its review-dependent blockers. Treat the report as private: it contains usernames, email addresses and redirect URIs.

## Other source systems

`riauth migration-preflight --file <input>` is the source-aware entry point. It reads `api_version` and never writes a report or manifest; `--output-file` captures its output privately.

- For a `riauth.authentik-import/v1` bundle it prints the same `ready_for_plan`, `summary`, `blockers` and `items` as `import-authentik --preflight`, plus `source`.
- For a `riauth.migration-inventory/v1` inventory it classifies a declared list of another system's configuration.
- Any other `api_version` is rejected.

The Authentik bundle is the only export format riAuth converts. No other identity system has a named export contract here, so an inventory is a checklist, not an import: it declares what the source contains and carries no configuration or credential values. `riauth --json schema migration-inventory` describes it:

```json
{"api_version": "riauth.migration-inventory/v1", "system": "keycloak",
 "elements": [{"kind": "user", "id": "*"}, {"kind": "provider", "id": "grafana"},
              {"source_kind": "realm", "id": "master"}, {"source_kind": "role", "id": "realm-admin"}]}
```

`system` is a lowercase name (letters, digits, hyphens). Each element has an `id` from the source, or `*` for every element of that type, and exactly one of:

- `kind`: a finding kind from the table above, other than `manifest` and `source_native`.
- `source_kind`: the source's own type when no finding kind fits, such as a Keycloak `realm` or `role`, or an Okta `authorization-server`. It is 1–64 lowercase letters, digits, hyphens or underscores, starting with a letter. A type that matches a finding kind (hyphens read as underscores) must use `kind`, so it cannot bypass a directory route.

A source-native element becomes a `source_native` finding that keeps its `source_kind`. It is unsupported and blocking, with the action to rebuild what it provides as reviewed riAuth configuration or retire it; for `authentik` it points to the export bundle like every other element. Uniqueness covers the type and the ID together, so a realm and a role can share an ID.

Every declared element gets exactly one finding, and every finding blocks. The report also has a blocking `manifest` finding, `ready_for_plan` is always false, and no manifest is ever produced. `source` names the system and format, with `converter` null.

| `system` | Elements with a riAuth route (manual, blocking) | Everything else |
| --- | --- | --- |
| `ldap`, `openldap`, `active-directory` | `user`, `group`: import through a configured [LDAP directory](ldap.md) and review its plan. `password`: not copied; checked by an LDAP bind at each login. | unsupported, blocking |
| `google-workspace`, `entra-id` | `user`, `group`: import through `riauth directory workspace` or `riauth directory entra` ([Workspace](enterprise/ENT-03.md), [Entra](enterprise/ENT-04.md)) and review the plan. | unsupported, blocking. `password` is unsupported, because these imports do not enable password sign-in. |
| `authentik` | Every element is manual and blocking: build the export bundle instead. | — |
| Any other name, such as `keycloak`, `okta` or `auth0` | None | unsupported, blocking, with a per-kind rebuild action |

A route is remediation, not conversion. The directory plans are the reviewed preflight for directory data, and nothing in an inventory establishes identity continuity. Subjects, local IDs, password hashes, factors and sessions from these systems are not carried over. Accounts are never linked by email, username or DN.

An inventory is rejected, and nothing is written, when it:

- has an unknown field
- has an invalid `system`, `kind` or `source_kind`, an empty or overlong ID, or an ID with control characters
- has an element with both or neither of `kind` and `source_kind`
- has a duplicate element, or declares a `manifest` or `source_native` element through `kind`
- has no elements

Rejections never quote input values. A malformed bundle or inventory is reported by line and column only.

## Identity continuity

Subjects follow the exported provider's mode. `hashed_user_id` uses the exported `uid`; other supported modes use the exported numeric ID, UUID, username, email or UPN. Never reconstruct Authentik's hashed UID from the numeric ID alone: it depends on the source instance. See [Authentik subject resolution](https://github.com/goauthentik/authentik/blob/main/authentik/providers/oauth2/id_token.py) and [user UID generation](https://github.com/goauthentik/authentik/blob/main/authentik/core/models.py).

Local IDs are stable `authentik-<pk>` values; per-client subjects preserve RP identity independently of that local ID. Duplicate subjects block conversion, and server planning checks collisions with existing users. Email equality is never used to link accounts. Existing names/IDs cannot be silently reassigned.

Parent-group membership is flattened into explicit memberships. Parents are read from `parents` (an array, Authentik 2025.12 and later) together with `parent` (a single value, older exports). Diamond-shaped hierarchies are fine; a path back to the starting group fails the conversion with "Cycle in exported group hierarchy", as does a group with more than 1024 ancestors. Every exported group is walked, including groups no user belongs to. Custom attributes are copied. `email_verified` starts false, and Authentik administrative status is not promoted automatically. Keep the riAuth bootstrap administrator while reviewing administrative roles. Imported authentication sources, service identities and administrative roles require explicit handling and can block conversion.

Password hash imports support Django `pbkdf2_sha256` with 1,000–2,000,000 iterations and Argon2id/i v19 with bounded memory/time/parallelism. Django's Argon2 prefix is normalized. PBKDF2 and Argon2i hashes, and Argon2id hashes whose memory, time or parallelism differ from riAuth's defaults, are rehashed to default Argon2id after the first successful login, without changing user identity or invalidating a legacy short password. Until then, an imported hash that is slower to verify than riAuth's one-second floor on failed logins (for example Argon2 with `m=262144,t=10`) makes failures for that account measurably slower than for others, which can reveal that the account exists. New password creation/reset still enforces the new-password policy. Invalid or excessive-cost hashes fail the atomic apply.

## Rehearse and cut over

Run the converter from the repository root, where `deployment-private/` is ignored by Git, or use a private operator directory outside the checkout. Keep the input bundle in that directory too.

```sh
mkdir -p deployment-private
riauth-maintenance import-authentik --file deployment-private/authentik-import.json --out deployment-private/migration
riauth validate --file deployment-private/migration/manifest.json --json
riauth plan --file deployment-private/migration/manifest.json --out deployment-private/migration/plan.json --json
riauth apply --plan deployment-private/migration/plan.json --non-interactive --run-id migration-rehearsal --json
```

For a structurally valid input, the converter writes a private report in a new directory. Malformed exports fail conversion before a report is written. If blockers remain, `ready_for_plan` is false and **no applicable manifest file is written**; the report contains a draft for translation work. Conversion does not read credential values. Planning checks current permissions, dependencies, issuer context and subject uniqueness. Applying resolves references and commits the whole plan or nothing.

A single store supports per-provider issuer URLs in `settings.issuer`, including Authentik's per-application paths. Provider discovery publishes that issuer and the shared protocol endpoints. Route each issuer host to the same instance. Import the corresponding signing domain with `riauth keys import` and preserve its `kid` when needed. Changing issuer requires an explicit RP account migration; merely preserving `sub` is insufficient. Issuer equality must be verified against every RP's discovery and token-validation configuration.

Rehearse in an isolated environment first. For every application, verify code/device login, original issuer/subject, group/role claims, scope policy, MFA, refresh, logout and callback behavior. Record each application's result and rollback procedure. This repository's synthetic RP tests do not replace that inventory. There is no claim that arbitrary Python mappings or authentication flows are equivalent to the supplied declarative translations.

Automatic API conversion does not extract private signing keys, live tokens, sessions or MFA secrets. Explicit private-key import can preserve a reviewed signing domain and `kid`; explicit TOTP secret references can migrate supported factors as described below. Live Authentik cookies and opaque tokens are not imported. Users reauthenticate and enroll any factors that were not migrated, and RPs refresh discovery/JWKS as required by the selected key strategy. Plan existing application sessions and offline token validators explicitly.

Before cutover, preserve the existing Authentik deployment, take and verify a riAuth backup, and record application/proxy configuration. Rollback restores the previous RP routing/configuration and deployment; it does not merge identities or sessions created after cutover. Restore procedures are documented in [operations.md](operations.md).

## Factors and passkeys

TOTP migration accepts `totp` entries mapping usernames to private `TotpImport` JSON references and credential versions. The schema supports base32 or hexadecimal secrets, SHA1/SHA256/SHA512, 6/8 digits and 15–120 second periods. The current time step is marked spent at import, so users need a fresh code. Secrets stay out of manifests, plans and audit output. Existing sessions are revoked when factors change.

**Passkeys are not migrated.** Authentik's WebAuthn credentials are bound to Authentik's hostname and stay in Authentik's database. Users who were passwordless in Authentik have no password to import and block the conversion until they are reset or invited. After import, users sign in to the portal with their password (and imported TOTP, if any) and add a passkey under **Sign-in and security**; the first passkey needs only a recent sign-in, and adding it signs the account out everywhere. Applications created with `require_mfa` accept a passkey sign-in or a password with a TOTP or recovery code, so plan the re-enrollment before those applications move. riAuth never asks the browser to forget unknown passkeys, so Authentik passkeys keep working for Authentik during a rollback. See [passkeys](passkeys.md#passkeys-in-the-browser).

## Sources and newer settings

Reviewed upstream source translations go in `source_resolutions`, keyed by exported source ID. Each is a `SourceSpec` (`riauth --json schema source`); the converter validates the supplied configuration rather than translating the Authentik source automatically. `source_links` explicitly maps a source and stable upstream subject to a local username. External users with these mappings can disable local passwords. Never infer links from email equality.

A resolution is applied only where riAuth has a matching adapter:

| Exported source | Without a resolution | With a resolution |
| --- | --- | --- |
| OAuth (`authentik_sources_oauth.oauthsource`) | manual, blocking | An OIDC or OAuth-profile `SourceSpec` is validated and applied (manual, non-blocking). A SAML spec blocks and is not applied. |
| SAML (`authentik_sources_saml.samlsource`) | manual, blocking | A SAML `SourceSpec` is validated and applied (manual, non-blocking). An OIDC or OAuth-profile spec blocks and is not applied. |
| Built-in (`managed` is `goauthentik.io/sources/inbuilt`) | convertible, non-blocking: Authentik's local login, with each local user's password classified separately | Blocks and is not applied; remove the entry. |
| LDAP | unsupported, blocking: configure a riAuth [LDAP directory](ldap.md) and retire the Authentik source before the final export | Still blocks; the resolution is not applied. |
| Plex, Kerberos, SCIM, Telegram and unrecognized sources | unsupported, blocking | Still blocks; the resolution is not applied. |

A resolution for a source that is not in the export also blocks. An applied resolution is validated as riAuth configuration; the report does not establish that it behaves like the Authentik source, and Authentik user matching modes such as `email_link` are not translated. Live Authentik cookies and opaque refresh tokens are not imported by these mappings.

The importer does not provision the newer enterprise settings from Authentik
exports: parent-owned agents, Google Workspace/Entra sync configuration, HTTPS
certificate bindings, device trust, SSF streams, PAM approvers, scheduled
offboarding and Windows device enrollment need explicit configuration and
acceptance. Record these requirements and their tested results in your
deployment inventory before claiming migration parity.
