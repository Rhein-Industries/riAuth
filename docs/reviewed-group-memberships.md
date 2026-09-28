# Reviewed privileged group membership

M05 now covers complete durable membership replacement for explicitly protected
groups. Configure up to 64 names in `reviewed_membership_groups` in either
Essentials or Platform:

```toml
reviewed_membership_groups = ["production-admins"]
```

Platform groups named in `pam_approvers` also require review for durable membership
changes. The existing temporary PAM request/approval/grant protocol is unchanged;
it does not write durable group membership. Ordinary groups remain immediate.

Create the empty group first, then stage the complete desired membership:

```json
{"members":["alice","bob"]}
```

An empty `members` array proposes complete revocation. Usernames are bound to
stable IDs and sorted by ID. Duplicates, unknown names, disabled members, and
members lacking M04's independent elevation provenance or carrying credential
exposure are refused. These checks cover retained as well as added members;
obsolete or exposed members may be removed. A reviewer never supplies replacement
content, policy, expiry or a caller-selected revision binding.

| Operation | Bearer API | `riauth group review` command |
|---|---|---|
| Stage | `POST /api/groups/{name}/membership-changes` | `stage <group> --file members.json` |
| Inspect | `GET /api/group-membership-changes/{id}` | `change <id>` |
| Approve | `POST /api/group-membership-changes/{id}/approve` | `approve <id> --digest <digest>` |
| Execute | `POST /api/group-membership-changes/{id}/execute` | `execute <id> --digest <digest>` |
| Cancel | `POST /api/group-membership-changes/{id}/cancel` | `cancel <id> --digest <digest>` |

Browser JSON uses `/api/admin` instead of `/api`, with the existing session and
origin guards. There is no new group-review browser page in this slice.
Approve/execute/cancel accept only `{"digest":"..."}`; extra fields are rejected.
The CLI uses the normal remote transport, including `--if-revision` and
idempotency keys. No CLI or browser adapter implements an alternate writer.

The author, every reviewer, and executor must be distinct, currently authorized
full human administrators. None may be a member whose membership this proposal
changes. Existing members whose membership is unchanged may participate. At least
one independent reviewer must approve; at most eight reviewers are retained.

Each immutable proposal includes exact before/after IDs and names, author identity
and epoch, management revision, resource and policy fingerprints, canonical digest,
creation time and a 15-minute expiry. Resource dependencies include each affected
identity, username binding, M04 exposure/provenance, human grant generation and
reviewed-membership credential fence and directory ownership bindings. Secret-bearing
dependencies are hashed and never returned or audited in plaintext. Policy binds
the issuer, protected-group configuration, directory configuration, capability
settings and review bounds. Approval bookkeeping does not advance the management
revision; an actual group write does.

Approval and final execution revalidate content, all participant authority,
membership, identity and policy dependencies inside the serialized transaction.
Execution writes the membership and its ordinary group audit, records the executor,
marks the proposal consumed, and emits the review audit atomically. A fresh replay
fails. An exact idempotency-key retry returns its existing receipt without a new
effect. Staging, approval, execution, cancellation and expiry cleanup each have
`reviewed_memberships.*` audits. Stale proposals can be cancelled and replaced.

The shared management writer refuses unreviewed privileged membership changes
from API/CLI/browser writes, desired-state plans, inbound SCIM, LDAP/Workspace/Entra
reconciliation, invitation acceptance and upstream source auto-provisioning.
These paths keep their existing scoped authority, ownership, lease, removal,
SCIM identity/ETag, audit and receipt checks. Plans needing a protected membership
change fail atomically; apply the exact reviewed membership separately, then
replan. There is no implicit conversion of a connector or manifest plan into M05
approval. The existing `OffboardMember` operation still immediately removes an
already-disabled, identity-matched user under its exact user-write authority.

M04 rejects agent/help-desk credential takeover of live members of currently
protected groups, including memberships predating `reviewed_membership_groups` or
`pam_approvers` activation. Existing help-desk grants against these members become
inactive immediately, and new grants are refused, without a ledger backfill.
Reviewed execution additionally records a durable per-person fence: removing a
group from configuration does not erase a still-held reviewed membership's fence.
Both checks require live membership; every shared group removal, including SCIM
offboarding, updates the historical fence. It survives backup recovery with durable
memberships; pending reviews are invalidated by recovery. The temporary PAM
request/approval/grant contract is unchanged.

This slice supports at most 128 current and proposed members per reviewed group,
128 retained review records, and 128 reviewed groups per person. Larger protected
groups fail closed instead of falling back to an immediate write. Unexpired
records are never evicted to make room; expired records are cleaned during staging.

The worktree was aligned to accepted `f4e655f` after verifying integration of the
earlier grant service (`87a3a26`) and browser UI (`5bd6b83`). The previous worker tip
is preserved at `refs/riwork-recovery/m05-before-group-review-251651c`. This slice
extends accepted M03/M04 code; it does not restore their earlier implementations.

Accepted advanced to `60c5a5d` during this work. The only shared changed file since
the aligned base is `src/config.rs`: accepted changes configured-password workflow
validation (`7b61a8a`), while this slice adds the independent membership-review
setting and its name/count validation in different sections. Both must be retained
at integration; no merge or combined-tree validation was performed here.

Remaining M05 work includes group lifecycle/policy changes, larger groups, a group
review browser page, configurable reviewer roles/quorums, other mutation classes
listed in [reviewed-grants.md](reviewed-grants.md#remaining-resource-classes-and-integration-boundaries),
and a general multi-resource review protocol.

Focused validation (2026-09-28), using the shared accepted Cargo target with
`CARGO_INCREMENTAL=0`, two build jobs and dev/test debug info disabled:

```text
cargo check --locked --offline --lib --bin riauth
cargo test --locked --offline --test reviewed_memberships --test group_management
cargo test --locked --offline --no-default-features --features essentials --test reviewed_memberships
cargo build --locked --offline --no-default-features --features essentials --bin riauth
riauth group review --help
```

The compile check and CLI build/help passed. Platform passed the new regression
and four existing group-management tests; Essentials passed the same new regression.
It checks scoped immediate writes, API/plan/SCIM bypass refusal, exact content,
participant separation, policy and M04 provenance drift without a revision change,
reviewer authority loss, idempotent receipts, single consumption/audit, reviewed
revocation, post-execution credential protection, and SCIM offboarding. New Rust
files passed formatting checks and `git diff --check` passed. No broad suite,
browser automation or PostgreSQL execution was run for this slice.

Credential-fence correction to held source `90920ba` (2026-09-28) threads the live
configuration through every credential/invitation exposure caller and help-desk
activation/binding. Its regression starts with ordinary membership and no holder
row, activates each protection setting, and verifies denied scoped credential
reset, rejected new help-desk binding, and inactive existing target grants with
unchanged stored grants. It then verifies that a reviewed holder stays fenced
after configuration removal, until live membership is removed. Unrelated ordinary
accounts remain manageable. The existing review/replay and offboarding regression
is retained unchanged.

Using the same shared target and environment above, the correction passed only
these focused test commands (including their builds):

```text
cargo test --locked --offline --test reviewed_memberships --test group_management
cargo test --locked --offline --no-default-features --features essentials --test reviewed_memberships
```

Platform: 6 passed; Essentials: 2 passed. `git diff --check` and formatting checks
for `management.rs` and `reviewed_memberships.rs` also passed.
