# ENT-02 — Parent-user ownership for agents

[Implementation](../../src/agent.rs) and tests ([ownership](../../tests/agent_parent.rs), [owner authority](../../tests/agent_authority.rs)). Available in Essentials and Platform.

Scoped agent credentials stay independent principals. Parent ownership records which user an agent belongs to. It does not copy that user's rights: the agent acts with its approved permissions limited to what the owner holds now.

## Create

A human administrator bootstraps every agent. `riauth --if-revision '<revision>' --idempotency-key '<unique-key>' agent create <id> --parent <username> --out FILE` and `POST /api/agents` accept an optional `parent` username. Creation requires the current numeric revision from `riauth revision` and a unique, stable operation key (`If-Match` and `Idempotency-Key` over HTTP). Only the first committed response discloses the credential; a server-side exact retry returns 409 `credential_already_issued` without it. The CLI needs a new unused `--out` path to send that retry if the first file exists. If the credential file was never written, inspect the agent and rotate its credential separately. The parent must already exist and be enabled; an administrator may be a parent. Unknown and disabled parents are rejected, and so is any permission outside the parent's current authority. A parent who is not an administrator can only be approved exact resources or `self`, never `*`. The response records `authorized_by`, the administrator who approved the permissions. The stored field is `parent_user`, the parent's user id. Omit `parent` and the agent has no owner, which is the previous behavior.

The link is immutable. Rotation replaces the token and extends expiry only. Id, permissions, and `parent_user` stay as created. Revoke still sets `enabled` false and deletes the token.

## Constrained inheritance

The explicit permission list is a ceiling, and the owner's current authority is a second one, recomputed on every use: see [owner authority](../agent.md#owner-authority). Parent ownership does not add permissions, group membership, administrator status, or an end-user session. Agents still cannot create administrators, alter administrator users, issue agent credentials, or authenticate or consent as end users.

The constraint is the parent's account. If that user is disabled or deleted, agent authentication and continued worker authority fail immediately. The shared user-write transition revokes every child agent in the same transaction (`enabled` false and the token index removed), revokes Windows devices and outstanding sign-in tickets, invalidates sessions, and enqueues an account-disabled SSF event. This covers administrator updates, SCIM PUT/PATCH/DELETE, desired-state apply, LDAP and cloud-directory sync, scheduled offboarding, and inbound SSF account-disable. Promoting the parent to administrator also revokes its agents, as it removes delegated grants. Re-enabling the parent never restores those credentials. Re-enable also revokes child credentials left active by older snapshots and advances the session epoch, without replaying an account-disabled event. Offboarding rechecks the parent again at execution, including for legacy agent rows still marked enabled.

[SSF entry-point regressions](../../tests/ssf.rs), [cloud-directory regressions](../../tests/cloud_directory.rs), and [offboarding regressions](../../tests/offboarding.rs) exercise durable revocation and disable/re-enable behavior.

## Audit

Agent create, rotate, and revoke events include `details.parent_user` when the agent has a parent, and `details.authorized_by` when the approving administrator was recorded. Actions performed by an agent, including manifest apply, include the same fields. Audit records do not contain the agent token.

See [agent administration](../agent.md).
