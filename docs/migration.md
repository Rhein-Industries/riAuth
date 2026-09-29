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
| `expression_policies` (optional) | `/api/v3/policies/expression/`; needed to convert group-membership expressions bound to applications |
| `scope_mappings` (optional) | `/api/v3/propertymappings/provider/scope/`; needed to convert the OAuth2 scope mappings of providers (see [scope mappings](#scope-mappings)) |
| `user_source_connections` | `/api/v3/sources/user_connections/all/` (Authentik 2025.4 and later); for older versions, the `oauth/` and `saml/` connection lists combined |

Collect every page into an array. A single complete `results` response is accepted only when its pagination reports a complete collection. Incomplete pages are rejected. Supply empty arrays only when the inventory establishes that the collection is empty. Keep the original exports private: provider responses can contain client secrets. The converter uses the [Authentik user serializer](https://github.com/goauthentik/authentik/blob/main/authentik/core/api/users.py) and [OAuth provider serializer](https://github.com/goauthentik/authentik/blob/main/authentik/providers/oauth2/api/providers.py) field names.

`user_source_connections` records which upstream identity Authentik links to each user: the user's `pk`, the source `pk` (or, before 2024.12, a nested source object) and the upstream `identifier`. It is required whenever a source resolution is applied or `source_links` is set; supply `[]` only when no account has a connection. Access tokens in these responses are write-only and are never read.

`excluded_groups` lists exported group `pk` values to leave out deliberately. Authentik's own groups (`authentik Admins`, `authentik Read-only` from 2024.8 and `authentik Agent-Users` from 2026.8) have spaces in their names, which riAuth cannot store, so list them unless you rename them first. Members of an excluded group lose it, including in group claims.

When you convert again into an instance that already holds imported accounts, set `target_state` to the `manifest` of an administrator's `riauth export` of that instance. The converter then matches accounts across exports by their Authentik UUID, as described under [identity continuity](#identity-continuity). Without it, planning still refuses any change of an existing account's ID or username.
The converted manifest binds the proven issuer and subject pairs from that export: account IDs and usernames, recorded Authentik UUIDs, each explicit subject together with the client issuer and pairwise sector that publish it, source issuers, and source links including the issuer stored on the link. A client's configured issuer and a missing issuer publish the same issuer when the configured value is the instance issuer. Planning and applying recheck this binding against the current target and refuse a manifest that would rename the account or add, replace, or remove `riauth.migration.authentik`. If the target changed after export, export the target again, reconvert, and review the new findings before planning. Changes to unrelated display or contact fields do not stale this binding. An export that names two owners for one account, subject or source link, or a subject whose client is missing, is ambiguous and blocks conversion instead of matching one candidate. A source link whose recorded issuer differs from its source is stale and blocks too; repair that link with a desired-state manifest that has no target fingerprint and sets the link issuer to its source's issuer, then export and convert again. When planning succeeds and applying fails, the apply rolls back and these bindings stay as they were.

Add `passwords`, keyed by username, containing `reference`, `version`, and optional `hashed: true`. References point to an environment variable or a private file on the migration agent. The normal user API does not return password hashes. Obtain them through an explicitly authorized offline export, or supply newly established passwords through a managed reset process.

Add `clients`, keyed by client ID. Each entry requires:

- Exact `issuer` from that provider's discovery document or `/api/v3/providers/oauth2/<pk>/setup_urls/`, and the scopes to preserve. The preflight checks it against the exported `issuer_mode` and application slug (see [identity continuity](#identity-continuity)).
- Reviewed `settings` containing declarative claims and policy translations.
- `translated_mapping_ids` and `translated_binding_ids` identifying reviewed source mappings/bindings. Scope mappings that riAuth reproduces exactly convert without a translation (see [scope mappings](#scope-mappings)). Every exported mapping/binding must be accounted for; expressions are never executed or automatically deemed equivalent. Exact group and user bindings of this provider's application are converted into its access conditions without a translation (see [application access](#identity-continuity)).
- `authentication_flow_reviewed: true` only after checking browser and terminal login, consent and MFA requirements; set `require_mfa` and scope rules accordingly. Until then the report carries the blocker "authentication flow has not been reviewed for browser/terminal login and MFA".
- `federation_reviewed: true` only after translating exported JWT federation trust into pinned `machine_trust`, `exchange` and `token_managers` settings. Exported encryption keys require reviewed recipient keys for both ID and access tokens.
- For confidential clients, `secret_ref` and `secret_version` pointing to the existing secret, or a deliberate new secret coordinated with the RP.

The importer carries over exact redirects, supported grant permissions, token lifetimes, ID-token claim placement and back-channel logout URLs. An explicitly narrower `settings.allowed_grants` can select the supported subset needed by your RP; it cannot add grants absent from a nonempty source list. Confirm that restriction before cutover. Authentik exports `grant_types` from 2026.5; for an older export without it, or with an empty list, the report blocks until `settings.allowed_grants` inventories the grants the RP uses. Regex redirects and unsupported provider/source behavior produce blockers. Configuration containing SAML/provisioning providers or upstream identity sources cannot be treated as OIDC-only parity. Authentik proxy (forward-auth) providers are not converted: an application that uses one produces the blocker "uses an unexported or unsupported provider", and it is recreated as a riAuth [proxy client](proxy.md). Regex redirects must become exact registrations before export; a client can register up to 32.

## Preflight report

`riauth-maintenance import-authentik --file <bundle> --preflight` converts the bundle in memory and prints the classified findings without writing any file. The legacy `riauth import-authentik` command remains available. `--out <new directory>` writes the same findings to the private `report.json`, plus `manifest.json` when ready. Both outputs contain `ready_for_plan`, `blockers` and `summary`, a count per classification plus the number of blocking items. `report.json` and `--preflight` also list every finding in `items`. `riauth --json schema migration-finding` describes one item:

| Field | Meaning |
| --- | --- |
| `kind` | `user`, `password`, `totp`, `passkey`, `session`, `credential`, `subject`, `issuer`, `group`, `source`, `source_link`, `provider`, `authentication_flow`, `property_mapping`, `policy_binding`, `federation`, `signing_key`, `encryption_key`, `client_secret`, `grant`, `token_lifetime`, `redirect_uri`, `logout`, `application`, `manifest` or, in an inventory report only, `source_native` |
| `source_kind` | Present only on `source_native` findings: the source system's own element type, such as a Keycloak `realm` |
| `id` | Exported identifier (username, group name, source/binding pk, client ID or application slug), scoped as `client/item` for mappings, redirects and federation, `username/client` for subjects, and `username/source` for source links (the riAuth source, or the exported source pk when that source is not converted). `*` covers every user, or for `issuer` the target issuer binding; `credential` findings use `static_tokens`, `authenticators` and `tokens`. |
| `classification` | `exact`, `convertible`, `manual` or `unsupported` (below) |
| `blocking`, `blocker` | Whether this item keeps `ready_for_plan` false, and its entry in `blockers` |
| `reason`, `action` | Why it has this classification, and what to do next |

- **exact**: carried over unchanged. Examples: strict redirect URIs, grant lists, `hashed_user_id`, `user_id` and `user_uuid` subjects, issuers that match the exported issuer mode, source links backed by an exported connection, logout URIs, group names without parents, and disabled application bindings, which admit and refuse no one, and scope mappings that return exactly the claims riAuth returns for their scope, such as Authentik's `openid` and `offline_access` defaults.
- **convertible**: carried over through a documented transformation. Examples: flattened parent groups, `user_username`, `user_email` and `user_upn` subjects, which stay fixed after import, `authentik-<pk>` local IDs, duration strings, logout redirects, application portal metadata, enabled group and user bindings of a converted application that riAuth expresses exactly, and single `ak_is_group_member` expressions bound to it, which become its access conditions, the built-in source, referenced password hashes and TOTP secrets, and scope mappings whose claims become riAuth claim mappings.
- **manual**: depends on reviewed operator input or an explicit decision. A manual item is non-blocking once that input is present. Examples: client resolutions, authentication flows, property mappings, policy bindings, federation, client secrets, source resolutions and newly established passwords. Group attributes and Authentik superuser status or roles granted through groups are non-blocking manual findings. They are not copied or promoted, so grant any riAuth permissions explicitly. Identity continuity adds the target issuer binding (non-blocking), reviewed issuers that do not match the export or lack `issuer_mode`, source links the export does not establish, exported connections that are not carried over, excluded groups (non-blocking) and stale `passwords`, `totp` or `excluded_groups` entries. A group-membership expression that does not factor is manual too, but where its condition is required it stays blocking until it is rewritten in Authentik.
- **unsupported**: cannot be represented or transferred. Examples: regex redirects, unknown subject modes or logout methods, non-OAuth2 providers, provisioning providers, sources without an adapter, subjects outside the OIDC format and duplicate subjects. Usernames, names, emails, group names and client IDs that riAuth cannot store unchanged, several groups with one name, one upstream identity linked to several accounts, issuers that several providers share or that collide with riAuth's own, and application bindings riAuth cannot keep exactly are unsupported and blocking. Authentik's internal service accounts are unsupported and non-blocking, and are never converted. Passkeys, live sessions and tokens, static recovery tokens, other authenticator devices, app passwords and API tokens always appear as non-blocking unsupported items, because they cannot move and users must sign in and enroll again.

A provider without a `clients` entry is still classified item by item, as if its review were empty. Its provider finding, authentication flow, and any exported mappings, federation, encryption key or confidential client secret block, while intrinsic items such as strict redirects, grants and the subject mode keep their normal classification. Its application is classified from the export but blocks until the provider is reviewed. Every application that shares a provider also gets its own blocking finding. Duplicate subjects are checked for every exported provider. No client, and no per-client subject, is converted from an unreviewed provider. A `clients` or `source_resolutions` entry that names nothing in the export blocks and is not applied, including any binding IDs it translates.

Classifications describe what the converter does with the export. They are not evidence that an application, factor or credential works after cutover. Password hashes and TOTP secrets are referenced but never read during conversion. Their formats are checked only when the plan is applied. Items never contain credential values. Findings are sorted by kind and identifier. `blockers` keeps its earlier messages, with three changes: source blockers name the source family and the adapter it needs; rejected or stale resolutions add their own blockers; and an unreviewed provider also reports its review-dependent blockers. Identity continuity checks add their own blockers, described below. Treat the report as private: it contains usernames, email addresses and redirect URIs.

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

A relying party knows an account by its issuer and subject. The converter keeps both exactly or blocks: it never renames, merges or re-links an identity, and never reports a credential as portable when it is not.

**Subjects.** Subjects follow the exported provider's mode. `hashed_user_id` uses the exported `uid`; other supported modes use the exported numeric ID, UUID, username, email or UPN. Never reconstruct Authentik's hashed UID from the numeric ID alone: it depends on the source instance. See [Authentik subject resolution](https://github.com/goauthentik/authentik/blob/main/authentik/providers/oauth2/id_token.py) and [user UID generation](https://github.com/goauthentik/authentik/blob/main/authentik/core/models.py). Authentik derives username, email and UPN subjects at every sign-in; riAuth stores the exported value as a fixed per-client subject, so those modes are convertible and a later rename no longer changes the subject. A scope mapping that returns `sub` overrides the mode in Authentik; such a mapping cannot be translated, and its relying party needs an explicit account migration. Duplicate subjects block conversion, and server planning checks collisions with existing users.

**Issuers.** In `per_provider` mode Authentik publishes `https://<host>/application/o/<application slug>/` for a provider, and in `global` mode its base URL, `https://<host>/`; both include a configured web path prefix. The reviewed `issuer` must have that form for the exported `issuer_mode` and application slug, otherwise the provider blocks, because its relying party would see a different issuer. An export without `issuer_mode` blocks, and an unknown mode is unsupported. A `per_provider` provider without an application has no Authentik issuer, so its reviewed issuer is a non-blocking manual finding. An issuer equal to the bundle's target `issuer` is served as riAuth's own; any other becomes the client's per-provider issuer, and riAuth publishes that for one client only. Several providers that keep one Authentik global issuer therefore block unless riAuth is initialized with that issuer, as does an issuer that differs from riAuth's only by a trailing slash. Route every kept issuer host to riAuth and check its discovery document from each relying party before cutover.

The converted manifest (and draft) records the bundle's `issuer`, because every issuer decision above assumes it. The preflight cannot see the instance, so this binding is a non-blocking manual finding (`issuer` `*`). Planning and applying enforce it: they fail, before any change is planned, unless the instance's issuer is exactly that value. Both values must be canonical URLs, and they are compared as exact strings, so a trailing slash matters. Existing bundles need no new input, but the manifests they produce are now bound; convert the export again for a different instance. Manifests without `issuer`, such as `export` output, stay unbound and portable.

**Accounts and names.** Local IDs are stable `authentik-<pk>` values; per-client subjects preserve RP identity independently of that local ID. Usernames, names, emails, group names and client IDs are kept exactly: one that riAuth cannot store unchanged, such as a username with a space, is reported by name, blocks, and is left out of the draft. Email equality is never used to link accounts. Existing names/IDs cannot be silently reassigned, and a repeated user `pk` or username fails the conversion. Authentik's internal service accounts (`ak-outpost-…`) back its outposts and are reported but never converted. Neither are the temporary accounts Authentik generates (`goauthentik.io/user/generated` with an expiry or delete-on-logout), which it deletes itself. An inactive account without a password reference is kept disabled with its local password disabled, so it keeps its ID and subjects without an invented credential. A `passwords` or `totp` entry that names no converted account blocks and is not applied.

**Converting again.** Each converted account records its Authentik `pk` and `uuid` in attribute `riauth.migration.authentik`, set from the export's own fields and never from exported attributes. riAuth IDs and usernames are immutable. With `target_state`, the converter compares every account with the one the target holds under its `authentik-<pk>` ID:

- If Authentik renamed the account and the recorded UUID matches, the account keeps its riAuth username (a convertible finding), and memberships, links and user policies name it by that username.
- A rename without a recorded UUID blocks, because riAuth never adopts an account by its numeric ID alone. So does any recorded UUID that differs from the export's.
- A new account whose username another riAuth account holds blocks; riAuth never gives a username to another account.
- A subject the target already issued stays with its account. A converted subject that would change blocks, and so does one another account holds. An account keeps its subjects for target clients the export no longer converts. A target subject whose client is not in the export is ambiguous, so that subject is not dropped and the export is not used for matching.
- A reviewed client issuer or pairwise sector that differs from the target's proven issuer or sector blocks, as does a reviewed source issuer that differs from the target source. Effective issuers are compared, so a client issuer set to the instance issuer and an omitted issuer are the same binding.
- A source link whose upstream identity the target links to another account blocks and is not applied. When the target link records an issuer, the carried link keeps it, and a reviewed source issuer that differs from it blocks.

An account whose username did not change, but which has no recorded UUID, is updated in place as before.

**Groups.** Parent-group membership is flattened into explicit memberships. Parents are read from `parents` (an array, Authentik 2025.12 and later) together with `parent` (a single value, older exports). Diamond-shaped hierarchies are fine; a path back to the starting group fails the conversion with "Cycle in exported group hierarchy", as does a group with more than 1024 ancestors. Every exported group is walked, including groups no user belongs to. Authentik's default profile mapping lists only direct groups, so members of a child group gain ancestor group names in riAuth group claims; review them for each relying party. Groups that share a name (possible before Authentik 2025.12) block and are never merged. An excluded group keeps no members and is not flattened into anyone; an `excluded_groups` entry that names no exported group blocks.

**Application access.** Authentik admits a user to an application when any enabled binding passes, or every one in `all` mode, and admits everyone when no binding is enabled. A group binding passes for members of the group and of every group below it, which the flattened memberships preserve. For a converted client, the preflight classifies the application's bindings by their `target` (the application's `pbm_uuid` or `pk`). riAuth ANDs the client's any-of `allowed_groups` with the required groups (`all_groups`), denied groups, allowed `users` and denied users of `settings.policy.access`. An enabled, non-expiring binding naming one converted group or account, without a policy, converts as follows. So does a binding to an exported expression policy that is exactly one group-membership check (below), as the group it checks, negated when either the binding or the expression negates it:

| Mode | Converts to |
| --- | --- |
| A single enabled binding (either mode) | A group becomes the only `allowed_groups` entry; a negated group, a user or a negated user becomes a denied group, the `users` list or a denied user |
| `all` | Each group becomes a required group, each negated group a denied group and each negated user a denied user; one user becomes the `users` list, while two or more users block, because no one is several users at once |
| `any` | Groups become `allowed_groups`; without groups, users become the `users` list. Every other binding is an alternative riAuth cannot combine with them, so it blocks and leaving it out only narrows access |

Converted conditions are added to the reviewed `settings.policy.access`; when both name users, riAuth keeps only users on both lists. Disabled bindings are exact and not converted. The remaining enabled bindings fall into two groups:

- **Manual, until acknowledged:** in `any` mode, an alternative riAuth cannot combine with the converted ones. That is a user beside groups, a negated binding, a policy expression, an expiring binding, a group or account that is not converted, or a user the reviewed list leaves out. Listing its ID in that client's `translated_binding_ids` accepts narrower access, which stays within what Authentik admitted because another alternative converts.
- **Unsupported, always blocking:** a condition riAuth cannot keep without admitting users Authentik refused, whatever `translated_binding_ids` says. That is a required condition (`all` mode or a single binding) that does not convert, two or more users in `all` mode (Authentik admits no one), a mode Authentik does not recognize (it admits no one), a missing mode with several bindings, an `any` mode where no alternative converts, and reviewed and converted user lists that share no user. Replace such bindings in Authentik with group or user bindings riAuth expresses exactly, and export again.

A client whose Authentik bindings restricted it, but which would admit every user after conversion, blocks until its reviewed `settings.policy` restricts access. Bindings on other targets, such as flows, and on applications that are not converted keep the reviewed-translation rule above.

**Group-membership expressions.** An expression policy converts only when its whole expression is one statement of this grammar, on one line (blank lines around it are ignored):

```text
statement  := "return" expression
expression := and-term { "or" and-term }
and-term   := not-term { "and" not-term }
not-term   := "not" not-term | "(" expression ")" | check
check      := "ak_is_group_member(request.user, name=" quoted-name ")"
```

At most 32 checks and 8 levels of `not` or parentheses are accepted. Tokens are separated only by spaces and tabs, and the name is a plain single- or double-quoted string. This is Python's own precedence: `not` binds tighter than `and`, and `and` tighter than `or`. Authentik's `ak_is_group_member` checks `user.all_groups()`, so each check passes for members of the group and of every group below it, exactly like a group binding and like riAuth's flattened memberships. A negated binding negates the whole expression.

riAuth ANDs two any-of group lists, `allowed_groups` and `settings.policy.access.any_groups`, with required and denied groups. The preflight factors the formula into exactly that shape:

- **Required groups:** the groups every passing assignment includes.
- **Denied groups:** the groups every passing assignment excludes.
- **Any-of lists:** the remaining groups must form at most two lists of plain groups. Each maximal failing assignment of them leaves out exactly one list.
- **Verification:** each checked group is treated as an independent yes/no variable, and the factored shape must agree with the formula on every assignment of up to 12 distinct groups. The result is therefore exact whatever the group hierarchy.

Examples:

| Formula | Converts to |
| --- | --- |
| `a and b or a and c` | allowed groups `b, c`, required group `a` |
| `a and not d or b and not d` | allowed groups `a, b`, denied group `d` |
| `a or a and b` | allowed group `a` |
| `(a or b) and (c or d)` | allowed groups `a, b` and `any_groups` `c, d` |
| `a and b or c` (read as `(a and b) or c`) | allowed groups `a, c` and `any_groups` `b, c` |

The factored shape converts as follows:

| Factored shape | Converts to |
| --- | --- |
| Required and denied groups only | Required groups (`all_groups`) and denied groups, like the static bindings of `all` mode |
| One any-of list only | An any-of list; in `any` mode it joins the other alternatives' allowed groups |
| Any-of lists with required or denied groups, or two lists | The lists plus required and denied groups, where every condition is required (`all` mode or a single binding); in `any` mode it is an alternative riAuth cannot combine |

Where every condition is required, the application's any-of lists, from all its bindings, fill `allowed_groups` first and then `settings.policy.access.any_groups`. Only one is available when the reviewed settings already set `any_groups`. An application that needs more lists than are available is unsupported and blocks.

The converted formula is then handled like any other converted binding. Some formulas do not factor, such as `a or not b` or three any-of lists. So do a formula that always passes and one with more than 12 distinct groups. Each is a **manual** finding: rewrite it in Authentik, for example by splitting it into separate bindings, and export again.

- Where every condition is required, or no other alternative converts, it always blocks. `translated_binding_ids` cannot clear it, because leaving it out would admit users Authentik refused.
- In `any` mode beside a converted alternative, acknowledging it accepts narrower access, as for other alternatives.

It is unsupported and blocks when:

- the formula never passes (for example `a and not a`), since Authentik admits no one;
- an application needs more any-of lists than riAuth has room for, as above;
- a check names a group that is missing from the export, excluded, not converted, or not the only exported group with that name (Authentik's check also matches an excluded group of the same name).

Every other expression is an unconverted policy binding under the rules above, and expressions are never executed. That includes:

- direct-membership checks such as `request.user.ak_groups.filter(...)`, which miss child groups;
- other calls, lookups, keywords or arguments;
- unbalanced or over-deep parentheses, comments, escapes, semicolons and `==`;
- a line break inside the statement, whitespace other than spaces and tabs, and any character outside ASCII;
- an expression policy missing from `expression_policies`, and an expiring binding.

A binding whose `enabled`, `negate` or `expiring` field is present but not a boolean fails the conversion, so a malformed export cannot change a binding's meaning. The report names the policy but never quotes its expression.

**Source links.** A `source_links` entry is carried over only when `user_source_connections` shows the same connection: its source must be the applied resolution of the connection's Authentik source, its username an exported and converted account, and its subject the connection's `identifier`. Any other link blocks and is not applied, including one the export does not show, one with another subject and one for an account or source that is not converted. Two links for one upstream identity also block. Only an external account with a carried link has its local password disabled. Every exported connection to a converted source is accounted for: one that is not carried over blocks when the reviewed source auto-provisions accounts, because the next upstream sign-in would create a second account; otherwise it is a non-blocking manual finding, and the user links the source again after signing in. Connections to sources that are not converted are reported as unsupported. A carried link matches only if the reviewed source returns the value Authentik stored: the `sub` for OpenID Connect and Okta sources, the Microsoft Graph `id` for Entra ID, the numeric `id` for GitHub and Google, and the NameID for SAML. On a later conversion, a carried link keeps the issuer recorded in the target export, and a reviewed source whose issuer differs from that record blocks. Rehearse each source's sign-in.

**Credentials.** Only referenced password hashes and TOTP secrets can move, and their formats are checked when the plan is applied. The report lists passkeys, live sessions and tokens, static recovery tokens, Duo, SMS, email and other authenticator devices, app passwords, API tokens, and upstream tokens stored with source connections as non-blocking unsupported findings, because they never move.

Custom attributes are copied. `email_verified` starts false, and Authentik administrative status is not promoted automatically. Keep the riAuth bootstrap administrator while reviewing administrative roles. Imported authentication sources, service identities and administrative roles require explicit handling and can block conversion.

Password hash imports support Django `pbkdf2_sha256` with 1,000–2,000,000 iterations and Argon2id/i v19 with bounded memory/time/parallelism. Django's Argon2 prefix is normalized. PBKDF2 and Argon2i hashes, and Argon2id hashes whose memory, time or parallelism differ from riAuth's defaults, are rehashed to default Argon2id after the first successful login, without changing user identity or invalidating a legacy short password. Until then, an imported hash that is slower to verify than riAuth's one-second floor on failed logins (for example Argon2 with `m=262144,t=10`) makes failures for that account measurably slower than for others, which can reveal that the account exists. New password creation/reset still enforces the new-password policy. Invalid or excessive-cost hashes fail the atomic apply.

## Scope mappings

With `scope_mappings`, each OAuth2 scope mapping of a provider is classified from its definition. A mapping converts only when its whole expression is comment and blank lines followed by one `return` of a plain dictionary. The dictionary may span lines and hold comments and a trailing comma. Each value must be one of:

- `request.user.name`, `request.user.username` or `request.user.email`;
- `True` or `False`;
- `[group.name for group in request.user.ak_groups.all()]`, or `request.user.groups.all()` in newer versions (direct groups).

A value converts only when riAuth returns the same value for every converted account:

- a name when every account has a non-empty Authentik name, because riAuth's display name otherwise falls back to the username;
- a username when no account kept an earlier riAuth username after an Authentik rename;
- an email when every account has an address, because Authentik returns an empty string and riAuth returns null;
- direct groups when every account's riAuth groups are exactly its direct Authentik groups, with no ancestor added by flattening and no direct group excluded or unconverted.

riAuth itself returns `name` and `preferred_username` for `profile`, `groups` for `groups`, and `groups` for `profile` when the reviewed client enables `groups_in_profile`. It also returns `email` and `email_verified` for `email` when the account has an address. A mapping for those scopes must return the applicable claims, and the other claims become `claim_mappings` on the mapping's scope, which the reviewed `scopes` must include. When a reviewed client includes `profile`, `groups` or `email` without a corresponding exported provider mapping, the report adds a non-blocking manual finding naming the claims riAuth will emit. This is a reviewed claim expansion, not an exact mapping; remove the scope or compare the emitted claims with the relying party before cutover. Authentik 2025.10's default `openid` and `offline_access` mappings are exact. Its default `profile` mapping converts into `given_name`, `nickname` and `groups` claim mappings when the values above hold.

An inexact mapping can be manual only when its source has one returned literal dictionary with plain, unescaped string keys other than `sub`. Its values may require custom translation, but its emitted claim keys are fixed. Each such mapping blocks until its ID is in `translated_mapping_ids`, for example when it:

- sets the `email` scope, whose `email` and `email_verified` claims riAuth derives from its own verified addresses (Authentik's default asserts every address as verified);
- grants an Authentik scope such as `goauthentik.io/api`;
- is one of several mappings for one scope, which Authentik merges;
- returns another reserved claim, omits a claim riAuth returns for its scope, or maps a claim the client already maps;
- uses helper calls as values under fixed keys, such as the `entitlements` default.

A mapping whose returned keys cannot be proved free of `sub` is unsupported and blocks even when its ID is acknowledged. This includes `sub` keys, escaped or computed keys, wrapper returns and multiple return paths. A provider mapping ID missing from `scope_mappings` is an incomplete export and is likewise an unwaivable subject-continuity blocker; export its definition and retry. Each mapping is checked before several mappings for one scope are classified as a manual merge. Replace an unproven mapping in Authentik with a literal dictionary of safe keys, or plan an explicit relying-party account migration. The report names a mapping but never quotes its expression.

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

Reviewed upstream source translations go in `source_resolutions`, keyed by exported source ID. Each is a `SourceSpec` (`riauth --json schema source`); the converter validates the supplied configuration rather than translating the Authentik source automatically. `source_links` explicitly maps a source and stable upstream subject to a local username, and must match an exported connection (see [identity continuity](#identity-continuity)). Only external users with a carried link have their local passwords disabled. Never infer links from email equality.

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
