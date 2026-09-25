# ENT-01 Privileged access

[Implementation](../../src/pam.rs) and [tests](../../tests/pam.rs).

Temporary group entitlements. A human asks for one existing group, a configured approver approves or denies, and an approval creates a time-limited grant. Expiry and revocation remove the group from authorization. Durable group membership is not changed.

This is an implementation note for the local riAuth service. It is not a certification, compliance attestation, or claim that the control meets an external privileged-access standard.

## Approver rules

`pam_approvers` on the server configuration maps a group name to the usernames allowed to approve or deny requests for that group. Names use the same rules as other riAuth names. At most 64 groups, and each group needs 1–32 approvers. The field defaults to empty, so existing configuration files keep loading.

```toml
[pam_approvers]
ops = ["approver", "duty"]
```

Restart after changing `pam_approvers` in the configuration file; request and decision checks use the configuration loaded into the process. Existing grants remain stored independently of that map. Approver usernames are not checked against the directory at startup; a typo means nobody in that set can decide until the configuration is corrected.

## Who can act

| Action | Who |
| --- | --- |
| Request | Enabled human session. Not an agent. The group must exist and have an approver rule. |
| Approve or deny | Enabled human in that group's approver set, other than the requester. The first decision wins. |
| Revoke | An approver for that group, or an administrator. |
| List requests or grants | Any human session, or an agent with `access.read` on `access/requests` or `access/grants` (`*` covers both). |

Agents cannot request, approve, deny, revoke, or create a grant. Self-approval is forbidden even when the requester is also an approver. A disabled requester cannot be approved. A second approve or deny of the same request conflicts, including when the two calls race: both run in a write transaction, and the later one sees the decided status.

A request carries a reason of 1–280 characters without control characters or the credential patterns rejected by `validate_reason`, and a duration of 60–86400 seconds. This pattern check does not guarantee that arbitrary sensitive text is detected. Every authenticated human can list every request, including its reason; listing is not restricted to the requester or approvers. At most 1,000 pending requests are accepted across the instance. The duration starts when the request is approved (`not_before` is that time, `expires_at` is that time plus the duration). Denial stores the decision and creates no grant.

## What a grant changes

`groups_for` is the membership set used by client `allowed_groups`, scope and access policy, portal launch, OIDC userinfo and group claims, explain, and the other callers of that helper. It is durable membership plus `pam::extra_groups`: grants for that user with `not_before <= now < expires_at` and no `revoked_at`.

`Group.members` is not updated. Both sides of directory membership use durable records: SCIM `Users.groups` and `Groups.members`, LDAP `memberOf` and group `member`, LDAP `search_groups` eligibility, outbound provisioning and user CSV reports exclude temporary grants. Their views stay reciprocal through approval, expiry and revocation. A temporary grant can allow LDAP authentication through policy while leaving directory visibility unchanged. Authentication claims, `/api/me`, and policy checks use the effective group set, including active grants.

Effective group lookups use a maintained per-user index of unrevoked grants; other users' grants and revoked grants are not scanned. Expiry is checked at read time, so expired rows cannot confer membership while waiting for cleanup. Indexes are rebuilt on upgrade and restore.

Expired and revoked grants grant nothing immediately. Rows are kept for audit until cleanup: decided requests for 7 days after the decision, pending requests for 7 days after creation, and grants until `expires_at + 7 days` or `revoked_at + 7 days`. Cleanup runs from the normal `Core::cleanup` pass. Schema version stays 3; the `access_requests` and `access_grants` buckets appear on first write.

Successful transitions audit `access.request`, `access.approve`, `access.deny`, and `access.revoke`, and those actions bump the configuration revision. The reason is stored on the request for the approver. It is not copied into the audit event. Session tokens and password hashes are not written there.

## HTTP

Bearer session, or an agent credential for the list routes:

| Method | Route | Body |
| --- | --- | --- |
| POST | `/api/access/requests` | `{"group":"ops","reason":"Release coverage","ttl":3600}` |
| POST | `/api/access/requests/{id}/approve` | none |
| POST | `/api/access/requests/{id}/deny` | none |
| POST | `/api/access/grants/{id}/revoke` | none |
| GET | `/api/access/requests` | |
| GET | `/api/access/grants` | |

## CLI

```sh
riauth group create ops
riauth access request ops --reason "Cover the release window" --ttl 3600
riauth access requests
riauth access approve REQUEST_ID
riauth access deny REQUEST_ID
riauth access grants
riauth access revoke GRANT_ID
riauth whoami
riauth audit
```

An agent reader, and not a decider (run from the repository root, where `deployment-private/` is ignored by Git, or use a private operator directory):

```sh
mkdir -p deployment-private
riauth agent create reader --permission access.read='*' --ttl 3600 --out deployment-private/reader.json
riauth --agent-file deployment-private/reader.json access requests
riauth --agent-file deployment-private/reader.json access grants
```
