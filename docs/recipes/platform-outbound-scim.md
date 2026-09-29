# Platform outbound SCIM recipe

This page is `outbound_scim_plans_provision_groups_preserve_remote_attributes_disable_departures_and_stop_stale_jobs` in [tests/identity/policy.rs](../../tests/identity/policy.rs). [tests/identity.rs](../../tests/identity.rs) includes that file as module `policy_tests`. The attribute is `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]`. The function has no `ignore` attribute and no `cfg` on that attribute. The test was not executed while this page was written.

`directory.scim_outbound` is listed in `FEATURES` and is absent from `PLATFORM_FEATURES` ([src/agent.rs](../../src/agent.rs)). `compiled_for` is false only for a `PLATFORM_FEATURES` name when the target is Essentials ([src/capability.rs](../../src/capability.rs)). Runtime configuration is `!config.scim_targets.is_empty()`. `directory.scim_inbound` remains in `PLATFORM_FEATURES`. Cargo's default features select `platform` ([Cargo.toml](../../Cargo.toml)). The check job's `cargo test --all-targets` line does not pass `--no-default-features`. A later step in that job builds Essentials and runs one CLI USB test. It does not name this function. This page does not claim an Essentials delivery run. The destination in this function is `riauth::api::router` from the same build as the test.

The operator contract stays in [scim.md](../scim.md). The [inbound SCIM recipe](platform-inbound-scim.md) is the other direction, and its HTTP peer is `oneshot`. This function serves the destination with `axum::serve` on a bound loopback port. This page does not use the assertions in [tests/offboarding.rs](../../tests/offboarding.rs), [tests/reconciliation_jobs.rs](../../tests/reconciliation_jobs.rs), or [tests/scim_oauth.rs](../../tests/scim_oauth.rs). The OAuth file drives a different outbound client. This target sets `oauth` to `None`.

## Three different claims

| Claim | What it means here |
| --- | --- |
| Check-job script | The check job in [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) runs `cargo test --all-targets --features test-support,fuzzing --locked` on `ubuntu-24.04` with Rust 1.98.1. It does not pass `--ignored`. This function is a non-ignored test in the `identity` integration binary, so that command's text includes it. The same job's later Essentials command is `cargo test --no-default-features --features essentials --locked --test cli legacy_usb_commands_fail_locally_with_client_guidance -- --exact`. The integration job's `cargo test --test identity` steps filter on `saml_independent_xmlsec`, `saml_source_independent_xmlsec`, and `saml_logout_independent_xmlsec` with `--ignored`. No workflow step names this function. The workflow text is not a result for this revision. |
| This revision | No Cargo command was run while this page was written. The test was not started. |
| Named SaaS directory | The peer is a second `Fixture` served by `axum::serve` at `http://{listener}/scim/v2` after `TcpListener::bind("127.0.0.1:0")`. Okta, Entra, Google Workspace, and the `payroll.example.com` target in [scim.md](../scim.md) are unconnected. |

## Records the fixture creates

`Fixture::new` copies a store from `Core::initialize` ([tests/common/mod.rs](../../tests/common/mod.rs)). The administrator username is `admin`, the display name is `Administrator`, email is absent, `admin` is true, and the password is the fixture constant `test-password-for-fixtures-only`. `Config` is `Default` apart from `data_dir`. The function builds two of those fixtures, `source` and `destination`. `source.admin` is the session token from `login`. It does not start with `ri_agent_`.

| Record | Value the test sets |
| --- | --- |
| Source group and user | `create_group` `staff`. `user("provisioned")` creates username `provisioned`, email `provisioned@example.test`, display name `Test User`, `admin` false, and password `PASSWORD`. `group_member` adds `provisioned` to `staff`. |
| Destination listener | `127.0.0.1:0`. URL `http://{local_addr}/scim/v2`. `tokio::spawn` runs `axum::serve(listener, riauth::api::router(destination.core))`. The function ends with `server.abort()`. |
| Target id `directory` | `token_file` is `{temp}/scim-token`, written with `destination.admin` bytes. `oauth` is `None`. `ca_file` is `None`. `groups` is `["staff"]`. `export_groups` is true. The test does not set `scim_reconciliation_modes`. |
| Agent id `provisioner` | Permissions `provisioner.read` and `provisioner.sync`, each on `provisioner/directory`. TTL `3600`. Parent `None`. The bearer used for plan and apply is `credential["credential"]["token"]`. |

`user` returns that user's session token. The provisioning calls use the agent token. The destination SCIM reads and the remote `scim_write` use `destination.admin` through `Core`, in process.

## Assertions the function makes

These checks run only when the test runs. This revision did not run them.

| Call | Asserted result |
| --- | --- |
| `provisioning_targets` with the agent | The JSON array length is 1. The test does not read `id`, `url`, `groups`, or `reconciliation_mode`. |
| `provisioning_plan` with the agent for `directory` | `resources` length is 2. The test does not read each resource's `kind`. `plan.to_string()` does not contain `destination.admin`. |
| `provisioning_apply` with `source.admin` | `is_err`. The test does not match an error string. |
| `provisioning_apply` with the agent | Returns a value. |
| `provisioning_step`, twice, each in `spawn_blocking` | Both calls return a value. `provisioning_jobs` for the agent, index 0, has `completed` true. This is the only `completed` assertion in the function. |

The destination then lists Users with `destination.admin`. `totalResults` is 1. `Resources[0].active` is true. The test keeps that resource's `id`. The Groups list has `Resources[0].members[0].value` equal to that id. The test does not read a group `totalResults`, and it does not read `userName`, `displayName`, `externalId`, or `emails` on the created user.

The function then writes the destination user itself, with `scim_write` patch flag `false`. The body is schema `riauth::scim::USER`, `userName` `provisioned`, the listed `externalId`, `active` true, `name.givenName` `Remote-owned name`, and email `provisioned@example.test` with `primary` true. On the source, `update_user` sets `display_name` to `Updated Name` and leaves the other `UserPatch` fields at `Default`. A new plan is applied with the agent, without a confirmation argument, and `provisioning_step` runs twice. `scim_get` shows `displayName` `Updated Name` and `name.givenName` `Remote-owned name`. The test does not read `completed` on that second job.

`group_member` then removes `provisioned` from `staff`. The next plan is applied with `provisioning_apply_confirmed`. The third argument is `Some` of that plan id. `provisioning_step` runs twice. `scim_get` on the same user id shows `active` false. The Groups list shows `Resources[0].members` equal to an empty JSON array. The GET returns the user. The test does not call `scim_delete`, and it does not read `totalResults` after the departure.

One more plan is applied with `provisioning_apply` and the agent. `revoke_agent` disables `provisioner`. One `provisioning_step` follows. `provisioning_jobs` is called with `source.admin`. The job whose `id` equals that last plan id has `stale` true. The test does not read `error`, `attempts`, `next_attempt`, `delivery_state`, `uncertain`, or `completed` on that job.

## Deactivation

The departure observation above is `active` false, an empty `members` array, and a user id that `scim_get` still resolves. That is the deactivation this function records.

[src/provisioning.rs](../../src/provisioning.rs) builds a selected enabled non-administrator as `active` true, with `externalId` `urn:riauth:{digest(issuer)}:Users:{id}`. A managed link whose user is no longer in that selected set is copied with `active` false. A selected group exports `member_ids` as the intersection of the group and the users still selected. The test does not read those plan bodies. It also does not assert the `externalId` prefix. The remote write reuses whatever `externalId` the Users list returned.

`ReviewBinding::confirm` in [src/connector_guard.rs](../../src/connector_guard.rs) returns a conflict when `review_required`, `missing_users`, or `removed_memberships` is set and the reviewed id is not that plan id. The message is `Connector removals require explicit review; confirm this exact plan ID after inspecting removal_impact and changes`. `RemovalImpact::assess` sets `review_required` when membership was removed, when users are missing, or when `large_removal` is true. `large_removal` is true when the disabled count equals the active count, including one of one. The departure call passes the plan id. The test does not read `removal_impact` or `review_required`, and it does not match that message. The create apply, the display-name apply, and the apply after departure call `provisioning_apply`, which passes no confirmation argument. Those three calls return a value in the function. The test does not show why those three plans were confirmable.

[scim.md](../scim.md) describes a separate offboarding path. A transaction that disables or deletes an enabled account writes one `provisioning_deactivations` row for each outbound user link whose target last reported the account active. Both editions record and deliver those rows. Scheduling the offboarding job is `identity.scheduled_offboarding`, which is in `PLATFORM_FEATURES`. `manual-review` and `guarded-automatic` hold the row until a reviewed plan delivers the disable. `automatic` can dispatch it. The row stops after five attempts. This function removes `provisioned` from `staff`. It does not disable or delete that local user, and it does not read `provisioning_deactivations`. It does not call `provisioning_reconcile`, and it does not set `scim_reconciliation_modes`. `ReconciliationMode` defaults to `ManualReview`. An omitted target uses that default ([src/provisioning.rs](../../src/provisioning.rs)). The four-attempt budget in [scim.md](../scim.md) counts failed controller snapshot pages. That counter, the five-attempt deactivation row, and `MAX_ITEM_ATTEMPTS` are three different limits.

## Retry and ownership

`provisioning_apply_confirmed` loads the plan, calls `actor.require("provisioner.sync", "provisioner/{target}")`, and then returns `Error::forbidden()` when `actor.id` differs from `plan.actor` ([src/provisioning.rs](../../src/provisioning.rs)). `Principal::allows` returns true for a principal that is neither an agent nor a delegated human ([src/agent.rs](../../src/agent.rs)). `source.admin` is a user session for the administrator, so that `require` succeeds, and the following id comparison is the forbidden result. The plan actor is `agent:provisioner` because `principal` prefixes an agent id. The test's administrator apply is `is_err` and leaves the error unread. The same function's agent apply returns a value. `plan.to_string()` omits `destination.admin`, which is the destination session token stored in the target file. Inbound SCIM record ownership is the other recipe. This function does not repeat those checks.

`revoke_agent` sets the agent `enabled` false and deletes its token index ([src/management.rs](../../src/management.rs)). The following `provisioning_step` claims through `provisioning_job_eligible`. That check rebuilds the plan actor and calls `authority_active`. A disabled agent fails that check. `claim_provisioning` then sets `stale` and the error `Plan authority or source configuration changed; inspect partial results and create a new plan`, and it returns no job, so the step returns without a remote write. The test asserts `stale` true on that plan id. It does not read the error, so it does not distinguish this string from `Source or authority changed during delivery; inspect partial results and replan`, which `finish_provisioning` stores when authority fails after a claimed write.

`MAX_ITEM_ATTEMPTS` is 12. `claim_provisioning` increments `attempts` before dispatch. On a dispatch error the next attempt is `now` plus `2^attempts.min(12)`, capped at 3600 seconds. At `attempts >= 12`, when the job is not already stale, the job becomes stale and the error begins `Stopped after 12 attempts on this item:`. `delivery_state` is `succeeded` when `completed` is true, `ambiguous` when `uncertain` is true, `failed` when `stale` is true, and `pending` otherwise. `job_view` includes `attempts` and `delivery_state`. The test unwraps each of the first six `provisioning_step` calls. `finish_provisioning` sets `attempts` back to 0 after a verified item. The test never reads `attempts` or `delivery_state`. The dispatch-error branch, including the 12-attempt stop, is source behavior this function leaves unexercised. The deactivation row's five-attempt stop is the separate counter above.

`Target::validate` requires canonical HTTPS or HTTP loopback, one to 64 groups, and exactly one of `token_file` or `oauth`. `provisioning_plan` calls `validate`. The fixture URL is loopback HTTP, and the plan call returns a value. The function does not try a non-loopback HTTP URL.

## CLI counterpart

[scim.md](../scim.md) shows `riauth provision targets`, `riauth provision plan payroll`, `riauth provision apply`, and `riauth provision jobs`. That example target is `payroll`, URL `https://payroll.example.com/scim/v2`, token file `payroll-scim-token`, group `payroll-users`, and mode `guarded-automatic`. The fixture target is `directory`, the URL is the bound loopback address, the group is `staff`, and no reconciliation mode is set. The fixture calls `Core` directly. Those CLI commands were not run. When `review_required` is true, the same guide says to pass `--confirm-removals` or `X-riAuth-Confirm-Removals` with the plan id. The test passes that id to `provisioning_apply_confirmed` on the departure plan and does not read the flag.

## What this fixture leaves open

The peer is the second in-tree riAuth router on `127.0.0.1`. No named SaaS directory is connected. The function leaves `delivery_state`, the 12-attempt item stop, the five-attempt deactivation row, reconciliation modes, the controller's four-attempt budget, OAuth, and `scim_delete` unexercised. Offboarding deactivation intent remains the separate queue in [scim.md](../scim.md). Recording that intent is a different operation from this function's reviewed departure plan.

## Other D03 recipes

Still without a recipe page: RADIUS, Workspace, Entra, Shared Signals, device trust, HTTPS client certificates, Vault Transit, Windows device login, the embedded reverse proxy, and shared-domain SSO. The [forward-auth](platform-forward-auth.md), [LDAP provider](platform-ldap-provider.md), [OIDC relying party](oidc-relying-party.md), [SAML IdP](platform-saml-idp.md), [LDAP import](ldap-import.md), [upstream OIDC](upstream-oidc.md), [inbound SCIM](platform-inbound-scim.md), and [SAML source](platform-saml-source.md) recipes are separate. The SAML source IdP is the in-process `Upstream` helper, and no named external IdP is connected. The inbound SCIM client is in-process `oneshot`, and no named SCIM client is connected. The relying-party client is the in-tree axum fixture, and a named external relying party remains an open peer. The SAML IdP signer is xmlsec1, and a named service provider remains an open peer. The import page follows disposable loopback OpenLDAP, and a real Active Directory directory remains an open peer. The upstream issuer is the in-process loopback token endpoint, and Okta, Entra, and Google are not connected. The peer on this page is a second riAuth router on loopback HTTP, and no named SaaS directory is connected. D04 emergency runbooks and D05 acceptance are separate work. The [capability matrix](../capability-matrix.md) records the protocol limits those recipes still have to cite.
