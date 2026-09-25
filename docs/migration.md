# Authentik migration

`import-authentik` is an offline converter and blocker report, followed by the ordinary plan/apply workflow. It does not connect to or modify Authentik. A successful conversion means the input is ready for a server plan, **not that an application cutover has passed**.

For a new instance without Authentik data, follow [getting started](getting-started.md). For a migration, keep Authentik available for rollback, initialize riAuth at its intended issuer, and take a verified backup before applying an import. Keep exports and generated credentials in private storage.

| Stage | Review before continuing |
| --- | --- |
| Export | Confirm every Authentik collection is complete and record the current application, issuer and subject contracts. |
| Convert | Run `import-authentik`; resolve every blocker in its private report. A draft is not an applicable manifest. |
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

The importer carries over exact redirects, supported grant permissions, token lifetimes, ID-token claim placement and back-channel logout URLs. An explicitly narrower `settings.allowed_grants` can select the supported subset needed by your RP; it cannot add grants absent from a nonempty source list. Confirm that restriction before cutover. Regex redirects and unsupported provider/source behavior produce blockers. Configuration containing SAML/provisioning providers or upstream identity sources cannot be treated as OIDC-only parity. Authentik proxy (forward-auth) providers are not converted: an application that uses one produces the blocker "uses an unexported or unsupported provider", and it is recreated as a riAuth [proxy client](proxy.md). Regex redirects must become exact registrations before export; a client can register up to 32.

## Identity continuity

Subjects follow the exported provider's mode. `hashed_user_id` uses the exported `uid`; other supported modes use the exported numeric ID, UUID, username, email or UPN. Never reconstruct Authentik's hashed UID from the numeric ID alone: it depends on the source instance. See [Authentik subject resolution](https://github.com/goauthentik/authentik/blob/main/authentik/providers/oauth2/id_token.py) and [user UID generation](https://github.com/goauthentik/authentik/blob/main/authentik/core/models.py).

Local IDs are stable `authentik-<pk>` values; per-client subjects preserve RP identity independently of that local ID. Duplicate subjects block conversion, and server planning checks collisions with existing users. Email equality is never used to link accounts. Existing names/IDs cannot be silently reassigned.

Parent-group membership is flattened into explicit memberships. Parents are read from `parents` (an array, Authentik 2025.12 and later) together with `parent` (a single value, older exports). Diamond-shaped hierarchies are fine; a path back to the starting group fails the conversion with "Cycle in exported group hierarchy", as does a group with more than 1024 ancestors. Every exported group is walked, including groups no user belongs to. Custom attributes are copied. `email_verified` starts false, and Authentik administrative status is not promoted automatically. Keep the riAuth bootstrap administrator while reviewing administrative roles. Imported authentication sources, service identities and administrative roles require explicit handling and can block conversion.

Password hash imports support Django `pbkdf2_sha256` with 1,000–2,000,000 iterations and Argon2id/i v19 with bounded memory/time/parallelism. Django's Argon2 prefix is normalized. PBKDF2 and Argon2i hashes, and Argon2id hashes whose memory, time or parallelism differ from riAuth's defaults, are rehashed to default Argon2id after the first successful login, without changing user identity or invalidating a legacy short password. Until then, an imported hash that is slower to verify than riAuth's one-second floor on failed logins (for example Argon2 with `m=262144,t=10`) makes failures for that account measurably slower than for others, which can reveal that the account exists. New password creation/reset still enforces the new-password policy. Invalid or excessive-cost hashes fail the atomic apply.

## Rehearse and cut over

Run the converter from the repository root, where `deployment-private/` is ignored by Git, or use a private operator directory outside the checkout. Keep the input bundle in that directory too.

```sh
mkdir -p deployment-private
riauth import-authentik --file deployment-private/authentik-import.json --out deployment-private/migration
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

**Passkeys are not migrated.** Authentik's WebAuthn credentials are bound to Authentik's hostname and stay in Authentik's database. Users who were passwordless in Authentik have no password to import and block the conversion until they are reset or invited. After import, users sign in to the portal with their password (and imported TOTP, if any) and add a passkey under **Passkeys and security**; the first passkey needs only a recent sign-in, and adding it signs the account out everywhere. Applications created with `require_mfa` accept a passkey sign-in or a password with a TOTP or recovery code, so plan the re-enrollment before those applications move. riAuth never asks the browser to forget unknown passkeys, so Authentik passkeys keep working for Authentik during a rollback. See [passkeys](passkeys.md#passkeys-in-the-browser).

## Sources and newer settings

Reviewed upstream source translations go in `source_resolutions`, keyed by exported source ID. Each is a `SourceSpec` (`riauth --json schema source`) using an implemented OIDC, OAuth-profile or SAML source; the converter validates the supplied configuration rather than translating the Authentik source automatically. `source_links` explicitly maps a source and stable upstream subject to a local username. External users with these mappings can disable local passwords. Never infer links from email equality. Sources outside those implemented adapters remain blockers. A missing resolution can still report the older OIDC-specific blocker wording; it does not establish support for an arbitrary upstream source. Live Authentik cookies and opaque refresh tokens are not imported by these mappings.

The importer does not provision the newer enterprise settings from Authentik
exports: parent-owned agents, Google Workspace/Entra sync configuration, HTTPS
certificate bindings, device trust, SSF streams, PAM approvers, scheduled
offboarding and Windows device enrollment need explicit configuration and
acceptance. Record these requirements and their tested results in your
deployment inventory before claiming migration parity.
