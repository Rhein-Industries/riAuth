# ENT-02 — Parent-user ownership for agents

[Implementation](../../src/agent.rs) and [tests](../../tests/agent_parent.rs).

Scoped agent credentials stay independent principals. Parent ownership records which non-administrator user an agent belongs to. It does not copy that user's rights.

## Create

A human administrator bootstraps every agent. `riauth agent create <id> --parent <username>` and `POST /api/agents` accept an optional `parent` username. The parent must already exist, be enabled, and must not be an administrator. Unknown, disabled, and administrator parents are rejected. The stored field is `parent_user`, the parent's user id. Omit `parent` and the agent has no owner, which is the previous behavior.

The link is immutable. Rotation replaces the token and extends expiry only. Id, permissions, and `parent_user` stay as created. Revoke still sets `enabled` false and deletes the token.

## Constrained inheritance

The explicit permission list is a ceiling. Parent ownership does not add permissions, group membership, administrator status, or an end-user session. Agents still cannot create administrators, alter administrator users, issue agent credentials, or authenticate or consent as end users.

The constraint is the parent's account. If that user is disabled or deleted, agent authentication and continued worker authority fail immediately. The shared user-write transition revokes every child agent in the same transaction (`enabled` false and the token index removed), revokes Windows devices and outstanding sign-in tickets, invalidates sessions, and enqueues an account-disabled SSF event. This covers administrator updates, SCIM PUT/PATCH/DELETE, desired-state apply, LDAP and cloud-directory sync, scheduled offboarding, and inbound SSF account-disable. Re-enabling the parent never restores those credentials. Re-enable also revokes child credentials left active by older snapshots and advances the session epoch, without replaying an account-disabled event. Offboarding rechecks the parent again at execution, including for legacy agent rows still marked enabled.

[SSF entry-point regressions](../../tests/ssf.rs), [cloud-directory regressions](../../tests/cloud_directory.rs), and [offboarding regressions](../../tests/offboarding.rs) exercise durable revocation and disable/re-enable behavior.

## Audit

Agent create, rotate, and revoke events include `details.parent_user` when the agent has a parent. Actions performed by an agent, including manifest apply, include that same parent id. Audit records do not contain the agent token.

See [agent administration](../agent.md).
