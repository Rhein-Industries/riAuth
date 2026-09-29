# Platform inbound SCIM recipe

This page is `scim_http_provisioning_is_owned_atomic_retriable_and_deprovisions_sessions` in [tests/identity/http.rs](../../tests/identity/http.rs). [tests/identity.rs](../../tests/identity.rs) includes that file as module `http_tests`. The HTTP client is `riauth::api::router(...).oneshot` from `tower::ServiceExt`. The call stays in the test process. The test was not executed while this page was written.

`directory.scim_inbound` is listed in `PLATFORM_FEATURES` ([src/agent.rs](../../src/agent.rs)). `compiled_for` is false for a Platform-only name when the target is Essentials ([src/capability.rs](../../src/capability.rs)). The `/scim/v2` routes are built in `platform_routes`, and the SCIM handlers are compiled with `feature = "platform"` ([src/api.rs](../../src/api.rs)). Cargo's default features select `platform` ([Cargo.toml](../../Cargo.toml)). The check job's `cargo test --all-targets` line does not pass `--no-default-features`. A later step in that job builds Essentials and runs one CLI USB test. It does not name this function. This page does not claim an Essentials run.

The operator contract stays in [scim.md](../scim.md). Outbound provisioning, below the inbound section of that page, is a different direction. This page does not use the assertions in [tests/scim_filters.rs](../../tests/scim_filters.rs), [tests/scim_pagination.rs](../../tests/scim_pagination.rs), [tests/scim_resource_versions.rs](../../tests/scim_resource_versions.rs), [tests/scim_projection.rs](../../tests/scim_projection.rs), [tests/scim_patch_valuepath.rs](../../tests/scim_patch_valuepath.rs), [tests/scim_name_patch.rs](../../tests/scim_name_patch.rs), [tests/scim_patch_compat.rs](../../tests/scim_patch_compat.rs), or [tests/scim_oauth.rs](../../tests/scim_oauth.rs). The OAuth file drives an outbound client against a local mock.

## Three different claims

| Claim | What it means here |
| --- | --- |
| Check-job script | The check job in [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) runs `cargo test --all-targets --features test-support,fuzzing --locked` on `ubuntu-24.04` with Rust 1.98.1. It does not pass `--ignored`. This function is a `#[tokio::test]` with no ignore attribute, so that command's text includes it. The integration job's `cargo test --test identity` steps filter on `saml_independent_xmlsec`, `saml_source_independent_xmlsec`, and `saml_logout_independent_xmlsec` with `--ignored`. They do not name this function. The workflow text is not a result for this revision. |
| This revision | No Cargo command was run while this page was written. The test was not started. |
| Named client | The HTTP peer is the in-process router. No named SCIM client is connected. No Okta, Entra, or other directory product sends these requests. |

## Records the fixture creates

`Fixture::new` copies a store from `Core::initialize` ([tests/common/mod.rs](../../tests/common/mod.rs)). The administrator username is `admin`, the display name is `Administrator`, `admin` is true, and the password is the fixture constant `test-password-for-fixtures-only`. `Config` is `Default` apart from `data_dir`. The default issuer is `http://localhost:9000` ([src/config.rs](../../src/config.rs)). The test then calls `Core::create_agent` with the administrator session.

| Record | Value the test sets |
| --- | --- |
| Agent id | `scim-agent` |
| TTL and parent | `600`, `None` |
| Permissions | `user.read`, `user.write`, `group.read`, `group.write`, and `group.members`, each with resource `*` |
| Credential | `agent["credential"]["token"]`, sent as `Authorization: Bearer {token}` on the two creates |
| Password literal | `PASSWORD`, which is `test-password-for-fixtures-only`. That literal belongs to the fixture. |

`USER` is `urn:ietf:params:scim:schemas:core:2.0:User` and `GROUP` is `urn:ietf:params:scim:schemas:core:2.0:Group` ([src/scim_shared.rs](../../src/scim_shared.rs)). The create body uses `USER`.

## HTTP assertions

These checks use `oneshot`. They run only when the test runs. This revision did not run them. The builder for the first request does not set a method, so the request is GET `/scim/v2/Users` with an empty body and no `Authorization` header.

| Check | Asserted result |
| --- | --- |
| Unauthenticated GET | Status `UNAUTHORIZED` (401). Header `content-type` equals `application/scim+json`. |
| Create, twice | POST `/scim/v2/Users` with `authorization: Bearer {credential}`, `idempotency-key: scim-create-42`, and `content-type: application/scim+json`. The body is the same JSON both times. |
| Create status | Each response is `CREATED` (201). |
| Create headers | Each response contains `etag` and `location`. The test checks presence. |
| Create body | Each body parses as JSON. The UTF-8 bytes do not contain `test-password-for-fixtures-only`. The second JSON value equals the first. The test reads `id` from that JSON. |

The JSON object is `schemas` `[USER]`, `userName` `scim-alice`, `externalId` `directory-42`, `displayName` `Directory Alice`, `password` the fixture constant, `active` true, and `emails` `[{value: scim-alice@example.test, primary: true}]`.

`router` applies `protect` before the handler ([src/api.rs](../../src/api.rs)). `scim_create` calls `Core::scim_write(token, kind, None, value, false)` and passes `StatusCode::CREATED` to `response`. The `false` flag means the write is a full resource, which is the POST mapping. The test's later `scim_write` calls are direct `Core` calls. They do not go through `oneshot`.

## In-process assertions in the same test

After the two HTTP creates, the same function calls `Core` directly. These checks also run only when the test runs.

| Call | Asserted result |
| --- | --- |
| `scim_list` Users, filter `userName eq "SCIM-ALICE"` | `totalResults` is 1. The other query fields stay at `Default`. |
| `scim_get` with the administrator session on that id | `is_err`. The test does not match an error string. |
| `login("scim-alice", PASSWORD, None)` | Returns a value. The test reads `session_token`. The third argument is the OTP, and the test passes `None`. |
| `scim_write` Users, that id, patch flag `true` | `is_err`. The patch document is schema `urn:ietf:params:scim:api:messages:2.0:PatchOp` with replace `active` false and replace `roles` `["admin"]`. |
| `me` with the session from login | `is_ok` after that failed write. |
| `scim_write` Groups, no id, patch flag `false` | Succeeds. Body is schema `GROUP`, `displayName` `provisioned`, `members` `[{value: id}]` using the created user id. |
| `scim_get` Users with the agent credential | `groups[0].value` equals the new group id. |
| `scim_write` Groups, patch flag `true`, add `members` value `[{value: "unknown"}]` | `is_err`. A following `scim_get` shows `members` length 1. |
| `scim_write` Groups, patch flag `true`, remove path `members[value eq "{id}"]` | The returned `members` array is empty. |
| `scim_write` Users, no id, patch flag `false`, body schemas `USER` and `userName` `admin` | `is_err`. |
| `scim_delete` Users for the created id | Succeeds. Then `me` is `is_err`, `scim_get` with the agent credential is `is_err`, and `scim_list` Users with a default query has `totalResults` 0. |

HTTP PATCH passes the same `true` patch flag and responds `OK`. HTTP PUT passes `false` and responds `OK`. HTTP DELETE calls `scim_delete` and responds `NO_CONTENT`. This function does not send PUT, PATCH, DELETE, GET by id, or POST `/.search`.

## Source behavior outside those assertions

`bearer` returns `Error::unauthenticated` when the `Authorization` header is absent: status 401, code `authentication_required`, message `Bearer authentication required` ([src/api.rs](../../src/api.rs)). `response` writes schema `urn:ietf:params:scim:api:messages:2.0:Error`, `status`, `detail`, and `scimType`. The code `authentication_required` selects `scimType` `invalidValue`. `content-type: application/scim+json` is set when the status is not 204 ([src/scim_shared.rs](../../src/scim_shared.rs)). The test reads the 401 status and that content-type. It leaves the 401 body unasserted. On the 201 responses it leaves `content-type`, the `etag` value, and the `location` value unasserted.

`response` copies `meta.location` to `Location` and `meta.version` to `ETag`. The view sets `location` to `{issuer}/scim/v2/{kind}/{id}` after trimming one trailing slash from the issuer, and `version` to a quoted digest of `[record.version, projected value, password_hash]` ([src/scim.rs](../../src/scim.rs)). The digest input includes `password_hash`. The projected object removes `password` before that digest is taken. The test's proof that the password was stored is the later `login` success. The test does not read the hash.

When `protect` sees one `idempotency-key` of 1–128 ASCII graphic bytes, it hashes the method, the URI, an optional `If-Match`, and the body into `context.fingerprint`. The body limit on that path is 32 KiB. `Core::mutation_checked` keys the receipt by `digest("{actor.id}\\0{key}")` and, on a matching fingerprint, returns the stored JSON before the precondition and before the mutation ([src/core.rs](../../src/core.rs)). The handler still maps that JSON to 201. The test shows two 201 responses with equal JSON for the same key and the same body. It does not read the receipt row. `scim_idempotency_key_rejects_a_changed_resource_if_match` is a different test. The two creates send no `If-Match`. The later `Core` calls are outside `protect`, so they do not set that header either. The agent `If-Match` rules in [scim.md](../scim.md) are outside this function.

`principal` sets an agent actor id to `agent:{agent.id}` ([src/agent.rs](../../src/agent.rs)). `owned` loads a record only when it is not deleted and `owner` equals `actor.id`; otherwise it returns `Error::missing("SCIM resource not found")`. The administrator session is a different actor, so `scim_get` with that session fails closed. The test checks `is_err` and leaves the message unasserted.

`normalize` rejects an attribute other than the supported names with `Error::bad("Unsupported SCIM attribute")`. `patch_resource` calls `normalize` on a path operation, so `roles` fails there. `scim_write` returns that error before `Ok(view)`. `Store::write` commits only after the closure returns `Ok` ([src/store.rs](../../src/store.rs)), and `mutation_checked` saves a receipt only after that `Ok`. The in-memory `active: false` on the failed patch is therefore not committed. The test checks `is_err` and then `me` `is_ok`. It does not re-read `active`. A successful change of `enabled`, or a password on a user write, sets `revoke`, increments `user.epoch`, and calls `logout::queue_user`. This failed patch does not reach that write.

A new user is built with `admin: false`. An existing local user whose `admin` flag is true returns `Error::forbidden()` before a SCIM write changes that user. The `userName` `admin` create hits the earlier branch: the username index or an ASCII case-insensitive username match returns `Error::conflict("Username already exists; provisioning cannot take ownership")`. The test checks `is_err` and leaves that string unasserted.

User `userName` equality uses `exact: false`, and `Filter::matches` implements that with `eq_ignore_ascii_case`. The fixture filter is `SCIM-ALICE` against the created `scim-alice`, and the test asserts `totalResults` is 1. It does not add a second user.

Group member `value` has to be an owned User id. `owned` rejects `"unknown"`. The test checks `is_err` and that `members` length stays 1. The remove path uses `members[value eq "{id}"]`, and the test checks that the returned array is empty. `scim_write` also calls `actor.require("group.members", &resource)` on a group, with `resource` shaped as `group/{displayName}`. `allows` accepts resource `*` for the same action ([src/agent.rs](../../src/agent.rs)). The five grants are what this function creates. The successful create and the successful group writes are the operations those grants cover inside this function.

`scim_delete` calls `disable_scim_user`, which sets `enabled` false and increments `epoch`, then marks the SCIM record `deleted` ([src/management.rs](../../src/management.rs), [src/scim.rs](../../src/scim.rs)). `me` loads the session and `validate_user` rejects a disabled user or an epoch mismatch ([src/identity.rs](../../src/identity.rs)). `owned` hides a deleted record, so the following `scim_get` fails. The test checks `me` `is_err`, `scim_get` `is_err`, and list `totalResults` 0. It does not read the user row, the session row, or the tombstone.

`ServiceProviderConfig` advertises `sort.supported` true and `bulk.supported` false. This function does not GET that document, does not send `sortBy`, and does not send a bulk body. Nested groups and enterprise or custom schemas are outside this function. PostgreSQL paging in `tests/scim_pagination.rs` is marked `#[ignore = "requires an isolated PostgreSQL test cluster"]` on its cluster tests. This page does not use those assertions.

## CLI counterpart

[scim.md](../scim.md) shows `riauth agent create` with the same five permissions and `riauth --agent-file ... scim` for list, POST, GET, and PATCH. Those commands were not run for this page. The fixture calls `Core::create_agent`, `router(...).oneshot`, and `Core::scim_*` directly.

## What this fixture leaves open

The HTTP peer is the in-process router. No named SCIM client is connected. Sort, bulk, enterprise schema, nested groups, projection, POST `.search`, HTTP PUT, HTTP PATCH, HTTP DELETE, and agent `If-Match` are outside the assertions above. The neighboring SCIM files remain separate tests. Several PostgreSQL paging tests are ignored. This revision did not run them.

## Other D03 recipes

Still without a recipe page: outbound SCIM, RADIUS, Workspace, Entra, Shared Signals, device trust, HTTPS client certificates, Vault Transit, Windows device login, the embedded reverse proxy, and shared-domain SSO. The [forward-auth](platform-forward-auth.md), [LDAP provider](platform-ldap-provider.md), [OIDC relying party](oidc-relying-party.md), [SAML IdP](platform-saml-idp.md), [LDAP import](ldap-import.md), [upstream OIDC](upstream-oidc.md), and [SAML source](platform-saml-source.md) recipes are separate. The SAML source IdP is the in-process `Upstream` helper, and no named external IdP is connected. The relying-party client is the in-tree axum fixture, and a named external relying party remains an open peer. The SAML IdP signer is xmlsec1, and a named service provider remains an open peer. The import page follows disposable loopback OpenLDAP, and a real Active Directory directory remains an open peer. The upstream issuer is the in-process loopback token endpoint, and Okta, Entra, and Google are not connected. D04 emergency runbooks and D05 acceptance are separate work. The [capability matrix](../capability-matrix.md) records the protocol limits those recipes still have to cite.
