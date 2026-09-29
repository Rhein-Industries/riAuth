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
origin guards. Administration now includes **Reviewed membership** at
`/admin#/membership-review`, also linked from each group's page.
Approve/execute/cancel accept only `{"digest":"..."}`; extra fields are rejected.
The CLI uses the normal remote transport, including `--if-revision` and
idempotency keys. No CLI or browser adapter implements an alternate writer.

In the browser, load the current members of an existing protected group, then edit
the complete replacement as one exact username per line. The page displays the
loaded current members, warns about removals, and requires explicit confirmation
before staging. Ordinary groups retain their immediate controls; trying to stage
an ordinary group is refused by the shared service.

Share the staged change's review link or ID with the other administrators. Its
read-only view shows before/after usernames and stable IDs, added/removed/retained
members, author/reviewers/executor, digest, dependency fingerprints, management
revision, creation time and expiry. Approve and execute require a fresh explicit
confirmation and send only the displayed digest. The page disables these actions
for authors, prior reviewers, affected members, and expired or consumed changes.
After this view receives HTTP 409 or 403 for the proposal, it also disables approve
and execute and shows the proposal as stale. Refresh loads the stored proposal and
sends the live management revision as If-Match. The server accepts that proposal
when participant authority, the exact membership, and the resource and policy
fingerprints still match, including when an unrelated audited write has advanced
the management revision. Cancellation remains available for an open proposal.

Toolbar refresh retains an unsent draft's content and original revision, clears
its confirmation, and never silently rebases it. **Load current members** explicitly
loads a new baseline while retaining the typed replacement for another check.
Drafts live only in memory and are cleared on account/sign-out transitions; a full
page reload discards unsent edits. Staged links reload their stored exact proposal
with confirmation cleared. A lost staging response locks the content, revision and
idempotency key so **Retry same staging request** retrieves the original receipt,
including after toolbar refresh. Resolve that outcome before reloading the page.
A lost approval/execution/cancellation response disables actions until the change
is refreshed. Error messages never display raw server detail.

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

Approval and final execution revalidate participant authority, the exact membership
snapshot, the resource fingerprint, and the policy fingerprint inside the serialized
transaction. The proposal still stores, digest-binds, and audits the management
revision captured at staging. That stored revision is the staging snapshot.
Unrelated audited writes may advance `meta.revision` while the proposal stays
usable. Direct management If-Match, reviewed grants, reviewed client policy,
client creation, client status, and client endpoint plans still require the
global management revision. A desired-state manifest that names any resource
besides groups still compares its stored base revision with that counter. A
groups-only manifest keeps the stored base revision and compares live group
membership, member identity and ownership, and membership policy; see
[removal safeguards](removal-safeguards.md). SCIM User and Group If-Match
compares the resource ETag. A caller that sends If-Match on this membership
flow must send the current management revision. An unsent browser draft still stages
with the revision it loaded, so an unrelated write between load and stage still
conflicts on If-Match. A display-name change for a member in the before or after
set still changes the resource fingerprint. Live PAM grants are outside this
dependency set; the policy fingerprint covers configured approver groups.
Execution writes the membership and its ordinary group audit, records the executor,
marks the proposal consumed, and emits the review audit atomically. A fresh replay
fails. An exact idempotency-key retry returns its existing receipt without a new
effect, and a failed attempt does not store a receipt. Staging, approval,
execution, cancellation and expiry cleanup each have `reviewed_memberships.*`
audits. A rejected open proposal can be cancelled and replaced.

This dependency check applies only to reviewed group membership. It adds no new
revision counter. Renaming or otherwise changing an affected member still
invalidates the stored proposal. Other conditional writes keep the single global
revision.

The disposable PostgreSQL primary exercises the same HTTP execute contract.
Four concurrent `POST /api/group-membership-changes/{id}/execute` calls share
one approved proposal: two with one idempotency key at the live revision, one
with a second key at that revision, and one with the staged If-Match. Exactly
one key returns the executed proposal. Its duplicate replays that body. The
other live key and the stale If-Match return `Configuration revision changed`.
The apply stores one `group.members.reviewed` audit, one
`reviewed_memberships.execute` audit, and one receipt. The winning key with
the original If-Match replays that receipt; the staged If-Match or a different
digest returns `Idempotency key was used for a different request`. A new key
at the pre-race revision returns `Configuration revision changed`, and a new
key at the current revision returns `Reviewed membership already consumed or
cancelled`. Dropping the connection pool and opening the same database returns
the same receipt and the same denials. The runner is
`RIAUTH_PG_TEST_TARGET=reviewed_memberships_postgres scripts/test-postgres.sh`.
It creates each test database on the primary only. The plaintext tests use
`sslmode=disable` and `local_unencrypted`. One test restarts that primary so
PostgreSQL presents a loopback certificate, then opens the database with the
production client: TLS is required, the private CA is verified, and records
use a keygen database key (`aes256gcm-v1`). The same execute, replay, stale
If-Match, consumed proposal, and dependency-denial results are required. A
missing key is refused as a format mismatch, and a different key cannot open
the sealed records. That certificate step restarts the same primary. A later
test stops the primary with `pg_ctl -m immediate` and promotes the disposable
standby with `pg_ctl promote`. The membership connection lists both loopback
ports, and riAuth's read-write target selection opens the promoted node. The
executed membership, its `group.members.reviewed` and
`reviewed_memberships.execute` audits, and the idempotency receipt are
unchanged. The original If-Match replays that receipt. A different If-Match or
a new key is refused. This is a loopback drill with trust authentication. It
does not elect a leader, measure a recovery objective, or cover failback,
network partitions, or PITR. The broader fenced failover fixture remains in
`tests/postgres.rs`.

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

Remaining M05 work includes group lifecycle/policy changes, larger groups,
configurable reviewer roles/quorums, other mutation classes
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

The browser slice starts from accepted `5da0fec`, after verifying that source
`90920ba`/`a508be9` exactly match accepted `1a3940e`/`4cf1422`. The previous source
tip is retained at `refs/riwork-recovery/m05-before-group-browser-a508be9`.
The page adds no management writer or authorization-policy changes.

Browser continuation validation (2026-09-28), using the same shared accepted
Cargo target, `CARGO_INCREMENTAL=0`, two build jobs and disabled dev/test debug info:

```text
cargo test --locked --offline --test reviewed_memberships_browser --test reviewed_grants_browser
cargo build --locked --offline --example portal_fixture
cargo test --locked --offline --no-default-features --features essentials --test reviewed_memberships_browser --test reviewed_grants_browser
cargo build --locked --offline --no-default-features --features essentials --example portal_fixture
```

Both editions passed both API regressions. After each edition's fixture build,
only `tools/browser/membership-review.spec.js` ran in Chromium: both scenarios
passed in each edition. The browser runner used the shared fixture through
`CARGO_TARGET_DIR`, the existing Playwright dependencies, a line reporter, and
`/tmp` results instead of another build/report tree. Coverage includes exact
digest-only actions, grant/revoke, distinct and affected participants, receipt
retry after a lost staging response and toolbar refresh, lost execution response,
single execution/audit and replay refusal, stale/cancelled/expired status, safe
errors, draft confirmation reset, account-switch refusal and session-loss cleanup.
The API regression also verifies Origin/session guards and browser-disabled asset
gating. The existing human-grant API browser regression remains passing.

Desktop and 320-pixel mobile screenshots were inspected; the page has no horizontal
overflow and the tested WCAG A/AA accessibility scan reported no violations.
JavaScript syntax checks, new Rust test formatting and `git diff --check` passed.
No broad suite, other browser engines or PostgreSQL execution ran for this slice.
At the final integration read, accepted was `46a93e1`; its changes since `5da0fec`
touch only LDAP implementation, boundary checks and network tests, with no file
overlap here. No merge or combined-tree validation was performed.
