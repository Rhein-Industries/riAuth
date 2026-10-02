# Deployment, backup and recovery

The [server editions guide](editions.md) gives the explicit Essentials and Platform
build commands and the downgrade preflight. The Dockerfile defaults to Essentials;
use `--build-arg RIAUTH_EDITION=platform` when the deployment needs Platform
adapters.

This is the v0.1 operator runbook. The default redb backend has one owning process, snapshot reads and serialized commits. PostgreSQL supports multiple service processes with shared sessions, replay state, rate limits and delivery leases; see [availability](availability.md). Database election, replication policy and fencing belong to the deployment. Do not start two servers against the same redb file. riAuth refuses it: while one process holds a redb store, another server, maintenance command or offline inspection of that store fails with `storage_owned` (exit status 5). The store must also be on local storage that enforces file locks. A filesystem that does not enforce them, or on Linux a network or cluster filesystem such as NFS, SMB/CIFS, CephFS, GlusterFS or Lustre, fails with `storage_not_exclusive` (exit status 2) before any record is read or written. Symbolic links in the store path are resolved before this check, and a dangling store link or a directory that does not resolve fails the same way before anything is created. riAuth then opens the file itself and checks it again against a fresh mount table before redb uses it, on Linux through the mount of the open descriptor, so a mount that changes during the open, such as an automount, also fails. Before redb opens the file, riAuth proves that file locks are enforced with a scratch database it creates and removes beside the store, on the store's own filesystem and mount, because redb would otherwise open, initialize or repair a store without a lock. The data directory must therefore be writable, and a store mounted as a single file is refused. Read-only inspection (`recovery status`, `transition-preflight`) answers the same lock question the same way; it opens the store itself only read-only and fails closed where it cannot create that scratch file on the store's own filesystem and mount, as for a store mounted as a single file. A dangling store link is an error there too, not a missing store. Gateways, workers and administration on other hosts use the authorized API, and several service processes need PostgreSQL.

The [released-image deployment examples](deployment-examples.md) give concrete
Linux Compose steps for one redb instance and for two PostgreSQL-backed hosts,
including mounts, readiness, TLS proxying and recovery ownership.

`riauth-maintenance` provides the offline `init`, `prepare-setup`, `restore`, `recover-admin`, `migrate-postgres`, `keygen`, `import-authentik`, `security-agreement-record`, read-only `transition-preflight` and `transition-plan`, and explicit `transition-activate` commands. Released Essentials and Platform maintenance archives contain the same executable name but are built with their respective feature sets. Use the archive matching the server edition for `init`, `restore`, and `recover-admin`; use the Platform archive for cross-edition transition preflight, plan, and activation. For a source build, select `--no-default-features --features essentials` or `--features platform` with `cargo build --locked --bin riauth-maintenance`. These builds have no terminal USB dependency. The executable accepts local configuration and output flags, but has no server URL, session, or HTTP administration commands. Stop the server before operations that need exclusive local database access or an edition switch. The existing `riauth` executable still accepts the older local commands during this transition; `riauth serve` and authenticated commands such as `login`, `backup`, `plan`, `apply`, and `status` keep their current paths. The separate [riauthctl](../crates/riauthctl/README.md) supports remote status, discovery, login and sessions, inventory, user, group, application and agent administration, invitation create, list and revoke, signing keys, dynamic-registration templates, Windows devices, client and RADIUS certificate bindings, upstream source list and put, delegated grants and the reviewed-change commands, terminal authorization approvals, temporary access, scheduled offboarding, directory sync, SCIM provisioning, Shared Signals streams (Platform), encrypted backup, and reviewed plan/apply and export (including explicit removal confirmation). Invitation delivery status, account verification, reset and acceptance, RFC 7591 client registration (`registration register`), source login, linking and unlinking, Windows device login remain in the legacy executable until parity review is complete. See [server editions](editions.md) for transition report semantics and [choose the binary](disaster-recovery.md#choose-the-binary) for restore guidance. Every command that opens a store records the opening binary's edition.

## Before exposing an instance

1. Choose redb for one service process or PostgreSQL for multiple processes. Keep the redb state directory on local storage, private to one host and one running server.
2. Choose a stable HTTPS issuer and decide whether riAuth or a reverse proxy terminates TLS. The default `http://localhost:9000` issuer is for loopback evaluation. Test DNS, certificates and the actual proxy path before directing users to it.
3. Run the server as a dedicated user with private configuration, storage and signing material. Generate a live database key if encrypting the store, and a **different** backup key held in a recovery secret store. Keep copies of external TLS, SMTP, source, certificate and Vault dependencies needed for restore.
4. Set `trusted_proxies` to the exact addresses of the proxy hops whose forwarded client IPs riAuth should trust. Ensure the immediate proxy overwrites forwarded client-IP and, if used, certificate headers. Check that an untrusted peer cannot reach the backend listener.
5. Verify `/readyz`, sign in as the first administrator, enroll its intended MFA, run `riauth doctor`, and exercise one real application login. Rehearse a restore into a new directory and record the result in your deployment's recovery log before directing production traffic to the instance. Review the [release limitations](limitations.md) for integrations that need separate acceptance.

## TLS and process service

For a single Linux host behind Caddy, create a dedicated `riauth` system user/group, install a verified binary at `/usr/local/bin/riauth`, and provision private directories. On a distribution with `useradd`, create the account first:

```sh
sudo useradd --system --user-group --home-dir /var/lib/riauth \
  --no-create-home --shell /usr/sbin/nologin riauth
```

This example keeps the database key outside the state directory; a secret-manager mount at the same path can replace the locally generated key:

```sh
sudo install -o root -g root -m 0755 /path/to/verified/riauth /usr/local/bin/riauth
sudo install -o root -g root -m 0755 /path/to/verified/riauth-maintenance /usr/local/bin/riauth-maintenance
sudo install -d -o riauth -g riauth -m 0700 /var/lib/riauth /etc/riauth
sudo -u riauth /usr/local/bin/riauth-maintenance keygen --out /etc/riauth/database.key
sudo -u riauth sh -c 'cd /var/lib/riauth && /usr/local/bin/riauth-maintenance init --issuer https://id.example.com --listen 127.0.0.1:9000 --database-key-file /etc/riauth/database.key'
```

`init` prompts for the first administrator password and writes `/var/lib/riauth/riauth.toml`. Review the generated configuration before starting the service. If Caddy connects over loopback, set the top-level `trusted_proxies = ["127.0.0.1"]`; use the actual socket peer if it differs. Replace the hostname in the [Caddy example](../deploy/Caddyfile), and arrange for Caddy to serve the public HTTPS issuer. The example overwrites `X-Forwarded-For` with the socket client's address.

From a source checkout, install the [systemd unit](../deploy/riauth.service), which uses this binary and state path:

```sh
sudo install -o root -g root -m 0644 deploy/riauth.service /etc/systemd/system/riauth.service
sudo systemctl daemon-reload
sudo systemctl enable --now riauth
curl --fail --silent http://127.0.0.1:9000/readyz
```

Once the HTTPS proxy is serving the issuer, use an operator account to check the remote CLI:

```sh
riauth --server https://id.example.com login admin
riauth --server https://id.example.com doctor
```

The unit confines writes to its state directory and runs without capabilities. Configuration and encryption keys must be readable by the service account. It is an example; validate it on the target Linux distribution. Keep `/var/lib/riauth`, `/etc/riauth` and administrator session files private.

riAuth walks forwarded chains from the right, skipping trusted addresses, and ignores them from every other peer. Malformed chains from trusted peers are rejected. This follows the trust boundary described in [Caddy's reverse-proxy documentation](https://caddyserver.com/docs/caddyfile/directives/reverse_proxy).

`trusted_proxies` takes at most 64 exact IP addresses; ranges are not supported, because the list also decides whose forwarded client-certificate header and forward-auth requests riAuth accepts. For **Traefik**, list every address Traefik connects from (run it with host networking or a static address). With a **load balancer in front of Traefik**, also list the balancer's addresses, so the walk reaches the real client address instead of counting every user against the balancer. See [Traefik forwardAuth](proxy.md#traefik-forwardauth).

The external issuer determines all published URLs. Configure native HTTPS as described below, or keep the HTTP listener behind a trusted TLS proxy. Issuers can contain `/application/o/<slug>/`; instance endpoints are nested under that path. Host/forwarded-host headers cannot change the issuer.

`GET /api/capabilities` reports the running instance's compiled, enabled,
configured and locally usable feature state. The local `riauth capabilities`
command reports artifact inclusion without opening the store. To turn off the
Platform device-trust verifier, remove every stored client policy that requires
it and remove the `[device_trust]` configuration, then set:

```toml
[capabilities]
disabled = ["identity.device_trust"]
```

An unknown or unsupported disabled name, a verifier configured while disabled,
or a retained client requiring device trust through either its direct setting or
a conditional approved-device policy without an enabled valid verifier stops
startup before the server accepts traffic. The existing client token
policy continues to reject an unmet device proof at request time.

## Container

The Dockerfile uses pinned Rust and Debian image digests, a locked Cargo dependency graph and UID/GID 10001. No data, credentials or local target directory enter the build context.

```sh
docker build -t riauth:development .
docker volume create riauth-data
docker run --rm -it -v riauth-data:/data riauth:development \
  init --issuer https://id.example.com --listen 0.0.0.0:9000
docker run --rm --name riauth -v riauth-data:/data \
  -p 127.0.0.1:9000:9000 riauth:development serve
```

Put TLS in front of the loopback-published port. Set proxy trust to the actual container-side peer address if you use forwarded headers. The example starts with an unencrypted store; for database encryption, mount a private key readable by UID 10001 outside the data volume, pass its path to `init --database-key-file`, and keep that mount available to `serve`. Back up the key separately. A container `--json capabilities` smoke check does not initialize storage or start the HTTP listener; validate service startup, other CPU architectures and your proxy deployment separately.

## Encrypted backup and restore

For a local instance, run this example from the repository root so `deployment-private/backup-lab` is ignored by Git. Outside a checkout, replace it with a private working directory. Initialize storage in one terminal:

```sh
mkdir -p deployment-private/backup-lab
cd deployment-private/backup-lab
riauth-maintenance keygen --out database.key
riauth-maintenance init --database-key-file database.key
riauth serve
```

While the service runs, use the same working directory in another terminal to sign in, generate a **separate backup key**, and take a backup. `backup` is an authenticated request to the running service; `restore` is a local offline operation that creates a new directory. Keep the backup key in your recovery secret store. `backup` streams the archive into a private file beside `--out` and gives it that name only after the whole archive authenticates:

```sh
riauth login admin
riauth-maintenance keygen --out backup.key
riauth backup --key-file backup.key --out backup.riauth
riauth-maintenance restore --backup backup.riauth --key-file backup.key \
  --out restored --database-key-file database.key
# For a new, dedicated PostgreSQL database, add:
#   --postgres-config deployment-private/postgres.json
```

On an Essentials deployment, run `init` and `restore` with the Essentials `riauth` binary instead of the Platform-built `riauth-maintenance` ([choose the binary](disaster-recovery.md#choose-the-binary)). Check the new `restored/` directory and the command's `verified` and `storage` results. Without `--postgres-config`, restore creates redb, even for a PostgreSQL source. With that option, it imports directly into the selected PostgreSQL database and writes a configuration pointing to it. Provision a separate, empty database and private connection file first; restore rejects existing riAuth records, checks again under the PostgreSQL writer lock, and never overwrites the original deployment. It does not start a server or switch traffic. A failed import may leave an empty target schema or a private output directory. If reopening fails after import commits, the isolated target remains recovery-gated without a published `riauth.toml`; discard and recreate it before retrying.

Restore applies the [restored-state policy](recovery.md): it invalidates every restored session, grant and pending proof, advances account epochs, and gates serving (`"serving_allowed": false`) until `riauth recovery complete --recovery-id <id> --persistent-credentials-reconciled` records that restored persistent credentials were reconciled. To rehearse service recovery, make the external keys and referenced files available in an isolated environment, then test administrator login, JWKS and representative application flows with the restored configuration. Do not expose a second server under the live issuer. For a remote service, use `riauth --server https://id.example.com` for `login` and `backup`; the account running `restore` needs read access to the archive and keys.

Backup takes a consistent database read snapshot while the server runs and encrypts the configuration and records with AES-256-GCM. `riauth backup` and `POST /api/operations/backup/stream` produce framed, authenticated `riauth.backup/v3` streams; the legacy `POST /api/operations/backup` endpoint still returns authenticated JSON chunks (`riauth.backup/v2`) of at most 64 MiB. Restore also accepts the earlier single-blob `riauth.backup/v1` envelope. For v3 it authenticates the entire stream before opening a target, then authenticates every frame again during import and compares the final transcript. Restore validates schema/issuer, requires a new output directory, applies the restored-state policy, checks signing keys and an enabled administrator, and reopens the restored store. A v2 manifest authenticates the chunk order, count and digests so truncated or reordered archives fail validation. The `verified` result covers these local checks, not a password or application login or access to external credentials and services. The restored configuration cannot serve until recovery reconciliation is attested.

The standalone `riauthctl backup --key-file backup.key --out backup.riauth` uses the same v3 stream route and publishes without replacing an existing destination after authenticating its framing and transcript. Its `verified` result does not decide server configuration or schema import compatibility; rehearse with the matching offline restore. It handles SIGINT/SIGTERM cooperatively and checks visible pending notifications before publication, with best-effort partial cleanup and no atomic signal-publication guarantee. SIGHUP is left untouched so an inherited `nohup` ignore stays in force; ordinary HUP termination can leave a private partial. See the [riauthctl guide](../crates/riauthctl/README.md) for size, timeout, OS permission and secret-handling limits.

The backup key, optional live-storage key and contents of external credential/certificate files are not included in the backup. The archive holds the server's parsed configuration without `database_key_file`, with each path as the server resolved it at startup; with an absolute `--config`, as in the systemd unit and container image, those are absolute paths on the original host. Restore rewrites only `data_dir`, `database_key_file` (from `--database-key-file`; omitting it produces a plaintext store) and `[postgres]`, and keeps every other reference. Re-provision and review those paths before starting the recovered service. On Platform, restore itself reads configured device-trust verifier material (a PEM, a JWKS, or a Verified Access service-account file) and RADIUS listener material, so provision them first. Vault private keys remain in Vault. The [disaster recovery runbook](disaster-recovery.md) lists what to keep outside the archive and the full restore order. [Operational recovery decisions](operational-recovery.md) chooses the procedure and records the failure stops. A lost backup key while that store is still serving is [Backup key lost, store still serving](operational-recovery.md#backup-key-lost-store-still-serving). [Connector dependency incidents](connector-incidents.md) is the read, stop, and retry boundary when LDAP, outbound SCIM, Workspace, Entra, SMTP, Vault Transit, or the alert webhook fails. [Administrator lockout](admin-lockout.md) is the serving-store path when a second administrator can still sign in. Losing the corresponding key makes those encrypted data unrecoverable. Record values are encrypted at rest; record names, sizes and access patterns are not hidden. Encryption does not protect a running compromised process. Backups contain operational secrets and should be treated as complete instance credentials.

To encrypt an existing plaintext database, take a backup, restore to a new directory with `--database-key-file`, verify the result, stop the old server and switch configuration. Never enable encryption by changing only the key field on an existing plaintext database. Key rotation for storage uses the same backup/restore procedure into a new directory.

## Upgrade and rollback

This version uses logical schema 3. Opening a schema-1 or schema-2 store performs an atomic migration. Schema 1 first backfills pairwise-subject seeds; schema 3 rebuilds due-time, age, quota-count/expiry and parent-session retention indexes. Credentials and signing keys are preserved, migrations are recorded, and the configuration revision advances. Unknown future schemas are rejected without mutation.

Startup also refuses an index revision newer than this binary before rebuilding any index. A successful initialization or open records the release version, edition, compiled capabilities, schema, index revision, and configuration revision in `meta/version_activation`. An open that would move to an older release, remove a compiled capability, or move the stored schema/index/revision behind that activation fails before migration. New activation on an existing store advances the configuration revision and preserves identity, credentials, sessions, grants and revocations. `/readyz` checks the record against the running binary on every probe; if another build activates the store, the earlier process fails readiness. Serving startup applies the same readiness gate before listeners and jobs start. Edition and configured-capability preflight still run before migration.

This is a store-local fence for binaries that implement the record, not proof that every writer has stopped. An older binary that ignores the record can still write, and separate source builds with the same package version, edition and compiled capability set are indistinguishable. Keep the stop-all-writers and backup steps below. A physical same-cluster restore can also remove the record; use the restored-state recovery procedure before serving.

The read-only `transition-preflight` report checks `meta/version_activation` against the requested target edition using the same startup compatibility rule and reports index-revision differences. A clean store last activated by Platform needs the explicit `transition-plan` and `transition-activate` maintenance handoff before an Essentials build can serve; current resource inventory alone does not clear the fence. Version activation and edition provenance are independent blockers, and the report retains both when applicable.

For an upgrade, use this order:

1. Record `riauth --version` and the deployed binary hash. While the old service is still running, take an encrypted backup with that version and run `restore` into a new directory to verify the archive. Preserve the old binary, backup key, database key and referenced external files.
2. Confirm the rollback binary can read that archive format. `riauth backup` writes `riauth.backup/v3` streams, which need a v3-capable reader; v0.1.1 reads only v1 and v2. A compatible backup made by the old version may be necessary. Test the entire restore and startup path before depending on it.
3. Stop the old service and all other writers. For redb, never overlap the two server processes on one file. For PostgreSQL, coordinate all nodes and database failover/fencing before allowing a new writer.
4. Replace the binary and start the new service against the **existing** configuration and store. Its first open performs any required migration. Check `/readyz`, `riauth doctor`, administrator login and representative OIDC/SAML/application flows before restoring traffic.
5. If rollback is required, stop every new writer and restore the pre-upgrade backup into a **new** redb directory with the compatible older reader and keys. For a redb deployment, point the old binary at that restored configuration. For PostgreSQL, use a rehearsed database recovery or migrate the restored redb store offline into a fresh PostgreSQL database before resuming multi-node service. After a database-native recovery, run `riauth recovery invalidate --database-restored` before any node serves; see [restored-state recovery](recovery.md). Every rollback signs users out, and serving waits for `riauth recovery complete`. A rollback binary released before this policy neither applies nor enforces it: its restore brings back restored sessions and grants as earlier releases did. Verify login and applications again. Do not run an old binary against a store the new binary has opened or modified.

Backup frequency determines your recovery point. Restore, service restart, DNS/proxy updates and application verification determine recovery time; neither has an HA guarantee in this implementation.

## Diagnostics and recovery

`doctor` reports schema, revision, administrator count, the active signing key id, whether `database_key_file` is set, and pending logout deliveries. `healthy` is true when at least one enabled administrator exists. The read counts `users`, `clients`, and `logout_deliveries` one storage page of 128 at a time inside one snapshot. The body fields stay the same. Each page decodes stored values in full, and this read does not size-cap a value. See [doctor page counts](roadmap/o06-doctor-page-counts.md). `riauth storage` is a separate read of physical allocation and does not change these doctor fields. `metrics` returns JSON process counters and worker capacity; counters reset at process start. Operational details require permissions.

Scheduled offboarding has a separate Platform read, `GET /api/operations/offboarding` (`riauth offboard diagnostics`). It requires `operations.read` on `operations/offboarding`. The body is `riauth.offboarding-diagnostics/v1`: counts for every stored offboarding job, plus at most 50 redacted attention items. An item is a `failed` job, a `done` job whose live downstream state is `pending` or `incomplete`, or a `scheduled` or `running` job that already has `last_error` or is overdue. A job is overdue when the stored local time shows it more than 300 seconds (`limits.overdue_grace_seconds`) past its due time: `max(execute_at, next_attempt)` for `scheduled`, `max(lease_until, next_attempt)` for `running`. An overdue item has `next_action` `check_worker_duty` even without a stored error and sorts ahead of failed items. Every item reports `overdue` and `overdue_seconds`, and `counts.overdue` and `counts.oldest_overdue_seconds` cover every job. Overdue means no maintenance pass has executed the job, so it has not committed its own local revocation; that says nothing about other changes to the account. To contain the account now, disable it with `PATCH /api/users/{username}` and `enabled: false`, which revokes its sessions and tokens. To restore duty, run a process with the background-jobs duty whose maintenance pass succeeds. A healthy pass claims at most 8 jobs a minute, so a batch due at one instant can read overdue until it drains: overdue is not proof that no worker runs. It is not remote completion, and it adds no Prometheus series or alert. Each item's username and targets also require `user.offboard` on the stored username; other jobs remain in the counts as `withheld`. `status: done` means the local revocation committed. `remote_completion_verified` is true only for downstream state `delivered`. `resolved` is an operator attestation. A hidden target still decides the state, and `next_action` is `inspect_hidden_targets` when any target is hidden. `affects_readiness` is false, so this report leaves `doctor.healthy`, `/readyz`, and `/livez` on their existing answers. The read scans every offboard job, because a done job with incomplete downstream work is absent from the failed queue index; the listed items stop at 50. Each item and visible target reports `has_error` when a stored `last_error` is present, and `next_action` uses that presence and the overdue verdict. The aggregate leaves the stored text on `GET /api/offboard/jobs/{id}`. See [offboarding diagnostics](roadmap/o06-offboarding-diagnostics.md) and [ENT-10](enterprise/ENT-10.md).

Deactivation delivery has its own Platform read, `GET /api/operations/offboarding/deactivations`. It uses the same `operations.read` on `operations/offboarding`. The body is `riauth.offboarding-deactivation-diagnostics/v1`: counts for every stored deactivation row, plus at most 50 redacted attention rows whose delivery state is `pending`, `failed`, `ambiguous`, or `dismissed`. `succeeded`, `resolved`, and `cancelled` stay in the counts. A row no offboarding job records is included. The read pages that bucket 128 rows at a time, retains at most 50 attention rows, and does not load offboarding jobs or report job linkage. An item also requires `user.offboard` on the account's current username. The stored username is not an authorization key and is omitted when it differs. A missing account is visible to a full administrator as `account_present: false`, without that stored name. A hidden target omits its name. `has_error` records a stored `last_error`, and `next_action` is a fixed token. The aggregate leaves error text, URLs, remote identifiers, evidence, leases, and actors on `GET /api/provisioning/deactivations`. `affects_readiness` is false. Essentials keeps that detailed list and does not serve this route. `riauth offboard diagnostics` remains the job aggregate. See [deactivation diagnostics](roadmap/o06-deactivation-diagnostics.md).

Provisioning deactivations have a separate shared read, `GET /api/operations/provisioning/deactivations`. It requires `operations.read` on `operations/provisioning`. Essentials and Platform both serve it. The body is `riauth.provisioning-deactivation-diagnostics/v1`: counts for every stored deactivation, plus at most 50 redacted attention rows whose delivery state is `pending`, `failed`, `ambiguous`, or `dismissed`. The read pages `provisioning_deactivations` 128 rows at a time. An item requires `provisioner.read` on `provisioner/<stored target>` and `user.read` on `user/<current username>`. The stored username is not an authorization key. A missing account is withheld for every caller, including a full administrator, so the stored username and target stay off this response. `user.offboard` is not consulted. The item reports the current username, `recorded_username_matches`, `has_error`, and a fixed `next_action`. It omits the target name, stored username, error text, URLs, and remote identifiers. Those stay on `GET /api/provisioning/deactivations` for a caller who can already read that row. `affects_readiness` is false. There is no CLI command and no Prometheus series. The Platform route above is unchanged. See [provisioning deactivation diagnostics](roadmap/o06-provisioning-deactivation-diagnostics.md).

Provisioning jobs have a separate read, `GET /api/operations/provisioning`. It requires `operations.read` on `operations/provisioning`. Essentials and Platform both serve it, on the same router as `GET /api/provisioning/jobs`. The body is `riauth.provisioning-job-diagnostics/v1`: counts for every stored provisioning job, plus at most 50 redacted attention rows. The read pages `provisioning_jobs` 128 rows at a time. An attention row is a `failed` job, an `ambiguous` job, or a `pending` job that already has `error`. `succeeded` jobs and clean `pending` jobs stay in the counts. `has_error` records a stored error. `next_action` is `inspect_provisioning_job`, `review_ambiguous_delivery`, or `wait_for_retry`. An item also requires `provisioner.read` on `provisioner/<target>`. Other rows stay in the counts as `withheld` and are omitted, so their ids and target names are not returned. The aggregate leaves error text, plan bodies, resource identifiers, leases, actors, and fingerprints on `GET /api/provisioning/jobs`. `id` is the stored job key used by stop and resolve. `affects_readiness` is false, so `doctor.healthy`, `/readyz`, and `/livez` stay on their existing answers. There is no `riauth` diagnostics subcommand and no Prometheus series. `queues.provisioning_jobs` pending and failed keep their index predicates: pending is a job that is not completed and not stale, and failed includes a stale job or any stored `error`, so a pending retry that already stores `error` can increment both. That pair is not this read's `failed` count. Each page still decodes full job values, including the retained plan, and this read does not size-cap a value. Apply still retains at most 64 jobs; a value written directly into the store can exceed that cap, and this read counts those rows too. See [provisioning job diagnostics](roadmap/o06-provisioning-job-diagnostics.md).

Connector lag is not a metric in this tree. A reconciliation schedule stores `next_run`, `last_job`, `last_error`, `last_outcome`, and optional `last_completed_at`. `last_completed_at` is local unix seconds when that schedule's then-current `last_job` is stored completed. It stays after a later job replaces `last_job`, so it is not the status of the current job. The reconciliation job does not store a completion timestamp. Nothing stores a remote high-water mark such as a directory USN, delta link, or sync token compared with a local applied generation. A completed controller job can still record `delivery` as `none`, `downstream_queued`, or `pending_prior_delivery`. When a schedule is due, `next_run` moves to the current time plus `interval_seconds` before the connector finishes, including when a queued or running job for that fingerprint already exists. A capacity conflict sets `next_run` 30 seconds ahead and records `last_error` instead of enqueueing. `now` minus `next_run` is therefore not time since a successful sync, and it is often negative while work is outstanding. An LDAP draft's `cookie` is the paged-results cookie, `phase` selects the user search or a configured group filter, and `sequence` counts pages already pulled in that draft. A cloud snapshot's `cursor`, `phase`, and `pages` resume that crawl. A provisioning `cursor` is progress through one reviewed plan. Queue oldest-pending age is the scrape time minus the earliest timestamp in that queue's pending age index, which is not a due time and not connector lag. `GET /api/operations/reconciliation` returns `next_run` as the next enqueue time. `last_completed_at` is written on the schedule at that local finish; this tree does not add a `riauth_connector_lag` series.

Reconciliation controllers have a separate read, `GET /api/operations/reconciliation`. It requires `operations.read` on `operations/reconciliation`. Essentials and Platform both serve it, on the same router as the schedule and job reads. The body is `riauth.reconciliation-diagnostics/v1`: counts for every stored schedule and every retained job, plus at most 50 redacted attention rows. An attention row is a schedule whose `last_error` is present, an enabled schedule whose local completion is overdue, a `failed` job, a `stale` job other than one marked stale because its schedule was disabled before dispatch, or a `queued` or `running` job that already has `last_error`. `has_error` records that presence. `next_action` is a fixed token from the row kind and status. The aggregate leaves stored error text, outcomes, authority, leases, actors and configuration fingerprints on `GET /api/reconciliation/schedules` and `GET /api/reconciliation/jobs`, which still require that controller's sync permission. `next_run` is the next time a periodic job is enqueued. It is not a measure of sync completion. A schedule attention row includes `last_completed_at` as a number or null: local unix seconds when that schedule's then-current `last_job` was stored completed. It can name an earlier job than the current `last_job`. It is not remote connector lag and not downstream delivery completion. The schedule read returns the number when it is set and omits the key when it is unset. Every schedule row also reports `completion_age_seconds` (local controller completion age, not connector lag) and `overdue`: an enabled schedule with no stored completion within `2 × interval_seconds + 300` seconds, or, with none stored, none since the retained last job's creation time, else since `next_run`; with no job queued or running the clock starts no earlier than `next_run - interval_seconds`, so a re-enable or interval change starts a new clock while a stuck job keeps its old one. An overdue schedule is listed without a stored error with `next_action` `check_worker_duty`: confirm that a process with the background-jobs duty is running (a `gateway` needs a separate `worker`; riAuth does not check duty coverage) and can reach the connector, then read the schedule and job reads. `counts` adds `schedules_overdue`, `schedules_never_completed` and `oldest_completion_age_seconds`; alert rules and the dashboard do not use them. `affects_readiness` is false, so `doctor.healthy`, `/readyz`, and `/livez` stay on their existing answers. There is no `riauth` diagnostics subcommand for this read and no Prometheus series. See [reconciliation diagnostics](roadmap/o06-reconciliation-diagnostics.md).

Outbound Shared Signals delivery has a Platform read, `GET /api/operations/ssf`. It requires `operations.read` on `operations/ssf`. The body is `riauth.ssf-delivery-diagnostics/v1`: counts for every stored delivery, plus at most 50 redacted attention rows whose state is `retrying`, `stopped`, or `cancelled`. The read pages `ssf_deliveries` 128 rows at a time and does not load stream records. An item also requires `ssf.configure` or `ssf.manage` on `ssf/<stored stream id>`; other rows stay in the counts as `withheld` and omit the stream id. `event` is `account_disabled`, `session_revoked`, `credential_change`, or `unknown`. `next_action` is a fixed token. The aggregate leaves the endpoint, subject, audience, JTI, credential type, and raw event string on the delivery record. `affects_readiness` is false. `queues.ssf_deliveries.failed` still counts retrying, stopped, and cancelled rows together. Essentials does not serve this route. The incident procedure is [SSF delivery](ssf-delivery.md). Mail delivery status remains `GET /api/operations/mail`, which already returns `stopped`, `attempts`, `next_attempt`, and `delivered_at` and omits the recipient, subject, body, proof, lease, and `dispatch_started`. See [SSF delivery diagnostics](roadmap/o06-ssf-delivery-diagnostics.md).

Use `/livez` for process liveness and `/readyz` for traffic readiness. Both are nested under a path-based issuer and bypass application quotas. Liveness performs no database work. Readiness verifies the storage schema, index revision and active release/capability record, checks that PostgreSQL is writable, and rejects saturated application workers on a role that serves authentication. Storage checks have separate bounded concurrency and a two-second response deadline; an outstanding database call keeps its permit until it ends. `/healthz` is a compatibility alias for readiness. A database outage should remove traffic through readiness rather than trigger a liveness restart loop.

A successful readiness response means that storage check passed. It does not mean every duty of a deployment is running in this process. The JSON adds `role` and `duties` (`authentication`, `protocol_listeners`, `background_jobs`). Integrated and gateway responses also include `issuer`. A worker omits `issuer`, serves no authentication or administration routes, and uses “Worker storage is not ready” when the storage check fails (the code remains `not_ready`). A gateway can be ready while `duties.background_jobs` is false. A disposable PostgreSQL exercise kept that response successful while the paired worker process was absent. See [process roles](roadmap/o01-process-roles.md).

HTTP handlers admit blocking crypto/database work through eight application workers, waiting up to two seconds for one; probes, background workers and protocol listeners have their own execution paths. Network limits are applied per effective client address; account failures additionally lock an existing username for 15 minutes after five failures. See [rate limits and admission](#rate-limits-and-admission). The maintenance loop runs about once per minute and removes expired protocol records in bounded pages; backlogs can take multiple passes. Audit retention is 90 days. Cleanup also removes retained PAM records and processes up to eight due offboarding jobs per pass; PAM access expires at the grant deadline independently of cleanup. Scheduled offboarding commits local revocation, RP logout and durable downstream SCIM deactivation intent together ([offboarding contract](enterprise/ENT-10.md)); remote delivery runs separately.

Logout delivery uses signed, audience-bound events, bounded concurrent requests, certificate verification, a five-second timeout and no redirects. The claim stores a lease until `next_attempt` (60 seconds) and pins that lease before the POST. A later worker sends after a new claim replaces an expired lease; a completion for the previous attempt is ignored. Failures retry with increasing delay for up to 24 hours; delivery records remain seven days. Inspect `riauth deliveries` with the corresponding permission. The RP must verify signatures/claims, target `sid`, and reject repeated `jti` values. Outbox delivery does not invalidate an RP that ignores logout events.

Serving responses for a stolen session, a lost passkey, a compromised agent or client credential, and a signing-key concern are in [credential compromise](credential-compromise.md).

For break-glass administrator recovery, stop every server connected to the store and use trusted local filesystem/key access. redb's exclusive file lock prevents another process from opening the store; PostgreSQL recovery refuses other connected riAuth clients. The command sets a new password, revokes sessions and grants, and records `admin.recover` in audit. Existing factors stay enrolled by default. For a passkey-only administrator with no authenticator app, a new password alone would bypass the passkey requirement, so recovery refuses it unless the operator explicitly chooses `--reset-mfa`. That flag removes enrolled passkeys, authenticator settings and recovery codes and records `admin.recover.factors_reset`:

```sh
riauth-maintenance --config /path/to/riauth.toml recover-admin admin --password-stdin --reset-mfa
riauth --config /path/to/riauth.toml serve
```

Supply a new password on standard input. Omit `--reset-mfa` when a password-backed administrator still has its authenticator code and needs the enrolled factors preserved. A passkey-only administrator with an authenticator app can also keep its factors and use that code with the new password. The last enabled administrator is protected during ordinary management. Verify the recovered login and applications, then enroll replacement passkeys before relying on passwordless access again.

Prometheus metrics are served at `/api/operations/prometheus` with `operations.read=operations/metrics`. `riauth metrics --prometheus` returns the exposition text in a JSON field. Requests rejected by header or trusted-proxy validation are included. Separate counters report 4xx client errors, 5xx server errors, 401/403 authentication rejections, rate limiting and worker admission failures. The original combined error counter remains for compatibility. Per-route latency uses registered route templates, finite method names and status classes; usernames, arbitrary paths and credentials are never metric labels.

Runtime histograms cover writer wait/hold time, PostgreSQL pool waits, signing, password verification and maintenance. Counters expose signing/maintenance errors, alert delivery failures, scanned records and optimistic transaction conflicts. A remote signing failure also counts under one fixed reason in `riauth_remote_signing_failures_total{reason}` and `runtime.remote_signing_failures`; see [failure reasons](kms.md#failure-reasons). An optional `alert_webhook` can POST selected local conditions (storage not ready, cleanup errors, signing failures). That route is not a substitute for deployment paging. See [alert routing](enterprise/PLATFORM-04.md). All counters reset on process restart. Metrics authentication still requires storage; monitor scrape availability and probes as well as metric values. [Scrape configuration](../deploy/prometheus.yml) and [alert rules](../deploy/riauth-alerts.yml) provide starting points; set thresholds from deployment load measurements. Keep the raw monitoring token in the referenced private secret file and rotate it before expiry.

[deploy/riauth-grafana.json](../deploy/riauth-grafana.json) is a Grafana dashboard document for the PromQL in the alert rules above. Select the Prometheus datasource that scrapes [deploy/prometheus.yml](../deploy/prometheus.yml). [scripts/check-grafana-dashboard.py](../scripts/check-grafana-dashboard.py) parses that JSON, requires each panel expression to be one of those rules with the comparison and any `{queue!="<queue>"}` exclusion removed, and requires every PromQL metric name to be rendered by this process. The check is source-only. Import into a running Grafana server is an operator step outside this repository. The original disposable loopback import and the corrected error-rate scale render are recorded in [o06 Grafana loopback](roadmap/o06-grafana-loopback.md).

The server-error panel is the five-minute rate of unlabeled `riauth_server_errors_total` divided by the five-minute rate of unlabeled `riauth_requests_total`, with the rule's `clamp_min`. That ratio is 5xx responses over all requests, so 401 and 429 responses remain in the denominator. Its field scale is min 0 and max 1 with unit `percentunit`, so the drawn axis is 0% through 100%. A ratio above 1, which `clamp_min` on the denominator can produce, is clipped at that max. The red threshold stays 0.05. The latency panel is the 95th percentile of unlabeled `riauth_request_duration_seconds`, the process-wide histogram in `src/api/observability.rs`. `riauth_http_duration_seconds` is the separate route, method, and status-class histogram. Rate-limit and worker panels are the five-minute rates of `riauth_rate_limited_total` (HTTP 429) and `riauth_worker_rejections_total` (a foreground request that found the eight application worker permits occupied). Signing and cleanup panels are the five-minute increase of `riauth_signing_errors_total` and the ten-minute increase of `riauth_cleanup_errors_total`. The write-wait panel is the 95th percentile of unlabeled `riauth_storage_write_wait_seconds`. Request, rate-limit, worker, and queue series are rendered in `src/api/observability.rs`. Signing errors, cleanup errors, and write wait are rendered in `src/telemetry.rs`. Those counters and histograms are process-local and reset when the process starts. Each scraped instance is its own series.

Queue panels draw `riauth_queue_pending`, `riauth_queue_failed`, and `riauth_queue_oldest_pending_seconds` as one series per `queue` label. The rules compare each series on its own. The labels are `logout_deliveries`, `mail_deliveries`, `provisioning_jobs`, `ssf_deliveries`, `offboard_jobs`, and `provisioning_deactivations`. `pending` and `failed` are maintained index counters, read from the store during the scrape, so a second scraped process repeats them. For the first four queues, pending means the row is not delivered, not stopped, and not completed, which keeps a retry pending. Failed on those queues includes a stopped or stale row, `last_failed`, a `next_attempt` of `u64::MAX`, a stored `error`, a non-2xx `last_status`, and an undelivered unleased mail row that already has an attempt. An SSF owner lease and pre-send pin are outside that predicate: a leased in-flight `ssf_deliveries` row stays pending until finish sets `last_failed`, `stopped`, or `delivered_at`. A mail owner lease and pre-send pin are outside the unleased-attempt clause the same way: a leased in-flight `mail_deliveries` row stays pending, and a failed finish clears the lease so the unleased attempt counts as failed while the row stays pending. A retry can increment both counters, and `ssf_deliveries` therefore folds retrying, stopped, and cancelled into `failed`. `offboard_jobs` counts scheduled and running as pending and `status` `failed` as failed. `provisioning_deactivations` counts `pending` and `running` as pending, and `failed` or `stale` as failed. Oldest pending age is the scrape timestamp minus the earliest timestamp in that queue's age index. For `offboard_jobs` that is creation age, not overdue time: a job scheduled up to 366 days ahead is already pending, and ages, from the moment it is created. Offboarding jobs and deactivations store `created_at`. Logout, mail, provisioning jobs, and SSF store `created_at`, and when that field is absent they store `plan.expires_at` minus 3600 seconds. The age index lists pending rows and is distinct from the due timestamp. The backlog rule `RiAuthDeliveryBacklog` is that age above 300 seconds while the same queue's pending gauge is above zero, for every queue except `offboard_jobs`. Both selectors carry `{queue!="offboard_jobs"}`, so a job scheduled more than a few minutes ahead no longer raises it. The other five queues are compared exactly as before. Scheduled offboarding therefore has no backlog alert. A job that stops after five attempts has `status` `failed` and raises `RiAuthFailedDeliveries` through `riauth_queue_failed{queue="offboard_jobs"}`. A scheduled, running or retrying job has no queue alert, including one that is due and has not been claimed, because no series measures time past `execute_at`. Read those through `GET /api/operations/offboarding`, which lists a scheduled or running job once it carries a `last_error` or is more than 300 seconds past its stored due time (`next_action` `check_worker_duty`; still no series or alert), and, when the maintenance pass itself returns an error, `RiAuthMaintenanceFailures` (`riauth_cleanup_errors_total`). The dashboard's age panel still draws the `offboard_jobs` series and its 300 second threshold line applies to it. That line is a display threshold, not an alert. Connector lag, key health, readiness, and physical allocated bytes stay off this dashboard. Prometheus scrape health remains the `up` rule in the alert file.

`riauth storage` and `GET /api/operations/storage` report physical allocated bytes for the store this process opened, with `operations.read` on `operations/storage`. On redb that is the apparent length of the opened database path, including free pages. On PostgreSQL it is the `riauth_store` tables and every partition or inheritance child below them, each counted once, with their indexes, TOAST and free space; the tables are shared by every process on that database. WAL, backups, other tables, other databases, filesystem capacity, and an occupancy ratio are not part of the reading. `capacity` stays `unknown` until a configured capacity is validated. The dedicated route is not cached: each call is one `stat` on redb, and on PostgreSQL one pooled connection and one catalog statement that stats the relation files on the database host and waits only behind an ACCESS EXCLUSIVE lock, up to `lock_timeout` per lock and the 15 second `statement_timeout` overall. The Prometheus series `riauth_storage_allocated_bytes` and `riauth_storage_allocation_age_seconds`, and the `storage_allocation` key of the JSON `metrics` response, are served from a cached sample for a caller that holds both `operations/metrics` and `operations/storage`, which includes a full administrator and an agent with `operations.read` on `*`; a scraper granted exactly `operations/metrics` is served as before and sees none of them. A scrape never waits for the size: one background thread per process reads it, at most once per 30 seconds while scraped, serves a sample up to 300 seconds old, and says how old it is. The first scrape after a restart, a sample older than 300 seconds, and a failed read have no series, and a table lock holds that one thread for at most about 15 seconds without slowing any scrape. A refresh that is still opening a PostgreSQL connection has no client-side bound after the TCP connect: it keeps the one thread, and a pool slot, until the process restarts, the series disappears after 300 seconds, and `refresh_running_seconds` in the JSON `storage_allocation` document keeps growing, so alert on that and on a missing series. The dedicated route can block the same way. See [storage allocation](roadmap/o06-storage-allocation.md) for the exact bounds. This dashboard does not plot the series. Writer wait, bytes materialized by reads, and queue pending or failed gauges are different readings. `riauth_signing_errors_total` counts signing failures in this process. `doctor` returns the active key id when `jwk()` succeeds, and `encrypted_at_rest` when `database_key_file` is set. `doctor.healthy` and `/readyz` do not use allocated bytes. The contract is [storage allocation](roadmap/o06-storage-allocation.md) and [storage and key diagnostics](roadmap/o06-storage-key-contract.md). What O06 does and does not yet cover, and why it stays open, is in the [O06 acceptance audit](roadmap/o06-acceptance-audit.md).

The public storage JSON also includes `pressure` (`riauth.storage-pressure/v1`), identifying the `storage` component, its safety limits and an operator remedy. `status` is always `unavailable` and `level` is null: neither backend measures capacity, even when allocated bytes are available. `unavailable_reasons` always includes `capacity_not_measured`, and adds `allocation_unavailable` for a missing or failed size sample or `allocation_sample_stale` for an available stale cache sample. Inspect the existing allocation status, `unavailable_reason` and cache fields when a sample needs attention, then verify capacity, free space and growth with filesystem monitoring for redb or database-host monitoring for PostgreSQL, including WAL and backups excluded from the sample. `safety.capacity_verified` is false; readiness and a successful size read establish no storage-headroom or write-safety guarantee. The same diagnosis is on `riauth storage`, `GET /api/operations/storage` and authorized `storage_allocation` in JSON metrics; it uses their existing permissions and adds no read, write, capacity, ratio or Prometheus pressure series. Raw Store allocation documents retain their existing schema. This availability diagnosis does not complete the physical pressure facet of O06.

JSON metrics and the existing `riauth metrics` command also return `key_health` (`riauth.key-health/v1`) under the existing `operations.read` permission on `operations/metrics`. It uses the already sampled `runtime.signing_errors` value: `observations.signing_failures` is the count in this process since its start, with `none_observed` at zero and `observed` above zero. It identifies neither a particular key nor the cause or current availability of a signer, and previous failures can remain in the counter after recovery. Full `status` remains `unavailable`, with the unmeasured primary signing material, signing domains, verification-key retention, remote signer and database encryption key listed explicitly; zero failures do not imply healthy keys. The diagnostic gives fixed, scoped operator triage for local signing configuration or an explicitly configured remote signer, and tells the operator to verify the remaining key-health checks through their existing procedures without exposing private material or credentials. It reads no key records or configuration, performs no new I/O and changes no state, readiness, doctor or Prometheus series. The read diagnoses evidence availability; it does not measure complete key health or close the original O06 key-problems facet.

**Contention measurements.** Writer wait and hold are also reported per activity in `riauth_storage_activity_write_wait_seconds` and `riauth_storage_activity_write_hold_seconds`: `foreground` is request work (HTTP, LDAP, RADIUS, CLI), while `maintenance` (cleanup passes, including due offboarding jobs, and alert checks), `provisioning`, `mail`, `logout_delivery` and `ssf_delivery` are the background workers. Logout and SSF polling each first read at most one due-index entry without taking the writer; if work is due, they re-read and claim the queue in a write transaction. An enqueue committed after an empty probe is picked up by a later delivery pass. Future retries and completed/stopped deliveries do not acquire the writer; due SSF rows that need retirement still do. `riauth_storage_write_waiters` and `_peak` count callers in this process queued for the writer; with PostgreSQL the advisory writer lock is shared by every node, so one node's wait includes other nodes' holds. PostgreSQL writer wait starts after the pool checkout. Pool metrics report checkout wait (including any new connection's setup) and connection hold time (the whole transaction, including a writer's lock wait) per activity, connections in use (and peak) against `riauth_storage_pool_capacity`, connection setup time and errors, checkout timeouts (`storage_busy`) and discarded closed connections. `riauth_storage_scan_rows` records how many records each range scan materialized, labelled by where it ran (`read`, `writer`, or `prepared` outside the writer) and whether the caller bounded it; point reads, bytes read and whole-keyspace snapshot records have their own counters. Optimistic preparation reports attempts, callback time outside the writer, validation time under it, records re-checked, expired authority and exhausted retries. Admission reports queue time, rejections, blocking work time and available permits for the `workers`, `credentials` and `forward` pools. The JSON `metrics` response carries the same values under `admission` and `runtime`. A labelled series appears after its first observation, and peaks are since process start. The added measurements are relaxed atomic counters and take no locks. [Testing](testing.md#contention-characterization) describes a local characterization workload.

Group writes maintain an encrypted per-user membership index in the same transaction as the group record. Session, authorization and directory membership reads use ordered 128-row index pages for that user, rather than reading every group. Opening a store with an older index version rebuilds the index from durable groups under the writer; allow time for this one-time pass when the group directory is large. A user who belongs to many groups still has to materialize every membership in the response. Temporary access grants remain separate and are evaluated at their live expiry time.

## Diagnostic next actions

Each attention row of the reconciliation, offboarding and provisioning-job diagnostic reads carries a fixed `next_action` token. The token names the follow-up; the read performs none of it. The tables below give, for each emitted token: when it appears, the existing read that holds the stored error, what is already true, and the existing command or decision. Deactivation-row tokens are in [deactivation delivery](deactivation-delivery.md) and SSF tokens in [SSF delivery](ssf-delivery.md). A waiver or an attestation is an operator record, never verified delivery.

**Reconciliation controllers** (`GET /api/operations/reconciliation`, both editions; no `riauth` command). The stored error is on `GET /api/reconciliation/jobs` and `GET /api/reconciliation/schedules`. Each row there needs the controller's own scope permission: `directory.sync` on `directory/<id>` for LDAP, on `workspace/<id>` or `entra/<id>` for Platform cloud controllers, or `provisioner.sync` on `provisioner/<id>` for SCIM. At each due time of an enabled schedule, the scheduler enqueues a new job unless one for that scope and configuration is already queued or running. No operator command re-runs a job: `POST /api/reconciliation/{kind}/{id}/events` accepts only the controller agent's own token. Sign-in and token paths do not call a controller; see [what still answers](connector-incidents.md#what-still-answers).

| Token | Emitted when | Read the cause | Already true | Action |
| --- | --- | --- | --- | --- |
| `inspect_connector_and_replan` | A job is `failed`: its fourth attempt failed (earlier retries wait 30, 60 and 120 seconds), or its worker lease expired after the final attempt. | Job `last_error` | The job is not retried. After an expired lease its outcome is unknown: it may already have committed local changes or queued SCIM delivery. | Fix the cause the error names, using the connector's section in [connector incidents](connector-incidents.md). Inspect the directory or target state and `riauth provision jobs` before trusting the next run. |
| `refresh_authority_and_replan` | A job is `stale`: the controller configuration or the controller agent's authority changed, or the call was refused as `access_denied`, `invalid_token` or `not_found`. A job made stale because its schedule was disabled before dispatch is not listed. | Job `last_error` | The job is not retried. Changes it committed before it went stale stay committed. | Confirm that the controller's `agent_id` names an enabled agent with the scope permission above and that its `credential_file` holds that agent's current token. Inspect what the job already changed; the next due run uses the current configuration and authority. |
| `wait_for_retry` | A `queued` job has the stored error of its previous attempt. | Job `last_error` and `next_attempt` | The worker retries at `next_attempt`, at most four attempts in all. | Wait. If the same error repeats, fix its cause before the attempts run out. |
| `wait_for_worker` | A `running` job has the stored error of an earlier attempt. | Job `last_error` | A worker holds the job's lease. | Wait for the lease to end. If the schedule then turns overdue, follow `check_worker_duty`. |
| `inspect_controller` | A schedule has a stored error: the scheduler could not enqueue its job, or it copied the error of its current job. Enqueue fails when the controller agent is missing, inactive or lacks the scope permission, or with `Too many active reconciliation jobs`. | Schedule `last_error` | No job is enqueued while that check fails. The scheduler tries again at the next due time, or after 30 seconds for the capacity conflict. For a capacity conflict, jobs already queued still run. After an agent failure, that controller's queued and running jobs have gone `stale`. | Restore the agent named by `agent_id`: enabled, unexpired, with the scope permission. A capacity conflict clears as active jobs finish. For a copied job error, follow that job's token. |
| `check_worker_duty` | A configured controller has no stored schedule, or an enabled schedule has no stored error and no completion within twice its interval plus 300 seconds. | Configured controller scope; schedule `last_completed_at` when present; `/livez` `duties` | A missing schedule leaves periodic completion unknown; event jobs may still exist. An overdue stored schedule has no completion within its local-time limit. Neither measures remote directory lag. | Confirm that a process with the background-jobs duty runs with this controller configured, then inspect current controller authority, credential setup and any stored schedule/job rows. |

A completed job is never an attention row, so `review_local_result` does not appear in this aggregate.

**Scheduled offboarding jobs** (`GET /api/operations/offboarding`, `riauth offboard diagnostics` or `riauthctl offboard diagnostics`, Platform). The stored error and the live downstream record are on `riauth offboard get <id>` (`GET /api/offboard/jobs/{id}`), which needs `user.offboard` on that user. Target tokens inside an item are in [deactivation delivery](deactivation-delivery.md).

| Token | Emitted when | Read the cause | Already true | Action |
| --- | --- | --- | --- | --- |
| `inspect_local_failure` | The job is `failed`: its fifth attempt failed, its 60-second lease expired after the fifth attempt (`Offboarding stopped after 5 attempts`), or an authorization or identity check ended it. Examples are an inactive scheduler, a changed user id, or a user who is now the last administrator. | `riauth offboard get <id>` `last_error` | None of the job's revocation committed: the job disabled nothing. | If access must end now, disable the account: `riauth --idempotency-key <key> --if-revision <revision> user disable <username>`, with `<revision>` from `riauth revision` (`PATCH /api/users/{username}` with `enabled: false`). Fix the cause, then schedule a new job with `riauth offboard schedule <username> --execute-at <instant> --timezone <label>`; a failed job is not active, so it does not block one. A failed job cannot be cancelled or rescheduled. It stays listed and keeps `riauth_queue_failed{queue="offboard_jobs"}` above zero. |
| `wait_for_local_retry` | A `scheduled` job has the stored error of a retryable attempt. | `riauth offboard get <id>` `last_error` and `next_attempt` | Nothing is revoked until an attempt commits. The fifth failed attempt is permanent. | Wait, or disable the account now if access must end. Fix the cause before the fifth attempt. |
| `wait_for_worker` | A `running` job has the stored error of an earlier attempt. | `riauth offboard get <id>` `last_error` | A maintenance pass holds the job's 60-second lease. Nothing is revoked until it commits. | Wait for the lease to end. A job left more than 300 seconds past it is overdue and shows `check_worker_duty`. |
| `check_worker_duty` | A `scheduled` or `running` job is more than 300 seconds past its stored due time. | `riauth offboard get <id>`; `/livez` `duties` | The job has not committed its local revocation. | Contain by disabling the account, then restore a background-jobs duty whose maintenance pass succeeds. |
| `inspect_hidden_targets` | A `done` job whose downstream state is `pending` or `incomplete` has at least one target hidden from the caller. This token replaces every downstream token. | `riauth offboard get <id>` with `provisioner.read` on the target | Hidden targets still decide the job's downstream state, so a hidden pending target never reads as delivered. | Obtain `provisioner.read` on `provisioner/<target>`, or ask an operator who has it, before acting on the downstream state. |
| `inspect_missing_delivery_evidence` | A `done` job is incomplete because some downstream records are unavailable and no retained target remains incomplete; no target is hidden from the caller. | `riauth offboard get <id>` and available retained audit/archive evidence | Local revocation committed, but missing records do not prove expiry, delivery or resolution. The job remains attention-worthy and unverified. | Inspect available retained evidence and the target state through the authorized incident process. No command reconstructs a missing record or converts it into verified delivery. |
| `confirm_waiver_not_delivery` | A `done` job is `incomplete`, no target is hidden from the caller, and every target that still needs attention was waived (`dismissed`) or attested (`resolved`). | `riauth offboard get <id>` `downstream.targets` (`dismissal`, `resolution`); `riauth provision deactivations` | Local revocation committed. riAuth verified none of those targets. | Confirm outside riAuth, against the recorded evidence, that each remote account is inactive. No command turns a waiver into delivery, so the job stays `incomplete`. |

A `done` job can also carry a deactivation token that summarizes its targets: `attest_remote_state`, `review_provisioning_plan`, `restore_controller_authority`, `inspect_deactivation` or `wait_for_deactivation`. Follow that token's row in [deactivation delivery](deactivation-delivery.md) for each target the job read lists.

**Outbound provisioning jobs** (`GET /api/operations/provisioning`, both editions). The stored error and `item` are on `riauth provision jobs` (`GET /api/provisioning/jobs`). The delivery states are defined in [SCIM delivery outcomes](scim.md#delivery-outcomes).

| Token | Emitted when | Read the cause | Already true | Action |
| --- | --- | --- | --- | --- |
| `inspect_provisioning_job` | `delivery_state` is `failed`: stale authority or configuration, an operator stop, or 12 attempts on one item. | `riauth provision jobs` | The job stopped and released the target for a new reviewed plan. riAuth does not undo items it delivered before the stop; delivery is at least once. | Inspect the target's remote state, then plan and review again: `riauth provision plan <target> --out <plan.json>`, then `riauth provision apply --plan <plan.json>`. Add `--confirm-removals` only for that plan's reviewed removals. |
| `review_ambiguous_delivery` | `delivery_state` is `ambiguous`: the current item's write was sent and its reply was lost, a 5xx, or not verified by read-back. | `riauth provision jobs` `item` | The next attempt reads the resource before any new write and clears the state when it observes the item. A stopped job keeps the state. | While the job runs, let it retry. Once it has stopped (`riauth provision stop <job-id>`, or after 12 attempts), inspect the target and run `riauth provision resolve <job-id> --observed <observed> --evidence <reference>`, with `applied`, `not_applied` or `absent`, before trusting a replacement plan. It needs `provisioner.sync` on the target and read access to the item, and conflicts while the item is still in flight. The command records what you observed and sends no remote request. |
| `wait_for_retry` | `delivery_state` is `pending` and the job has a stored error. | `riauth provision jobs` | The failed attempt either failed before sending or was refused with a 4xx other than 408, 425 or 429, so it changed nothing remotely. Backoff is capped at one hour. | Wait. If the error persists, fix it at the target. After 12 attempts on one item, the job stops and becomes `failed`. |

## Background capacity and overload

Both editions run scheduled and manual connector work on four dedicated Tokio
runtimes. Each has one async thread, the blocking-thread cap shown below, and a matching limit on
admitted passes. Foreground sign-in, session revocation and probes retain their
existing runtime and admission pools.

| Lane | Passes / blocking threads | Work |
| --- | --- | --- |
| `connectors` | 2 | Reconciliation; reviewed provisioning; manual LDAP/cloud/SCIM plans and applies; reconciliation event submission |
| `delivery` | 2 | Mail; logout followed by SSF (Platform); alert webhooks |
| `maintenance` | 1 | Cleanup, including scheduled local offboarding |
| `deactivation` | 1 | Due offboarding deactivation through the existing complete claim/dispatch/finish API |

Each scheduled worker admits at most one pass at a time; manual requests can
use up to the two shared connector slots for different targets. There is no
in-memory waiting queue: a busy lane defers the pass before it claims durable
work. Existing
outboxes, job leases, retry limits, removal review and authorization checks
remain authoritative. Mail and logout each retain their 16-item batch bound and
drain all child finishes before releasing the pass, including after an error.
Alert dispatch runs independently of maintenance, so a slow webhook cannot
hold up the next scheduled local revocation pass. Logout still precedes SSF
within a delivery pass.

Manual plan/apply and reconciliation event requests use the same connector
runtime and slots as scheduled work, including after bootstrap activation.
Routers sharing a store share one executor; creating a router starts no extra
runtime until needed. Delivery and maintenance retain their separate 2/1
budgets. An occupied connector lane or target returns `503 connector_overloaded` with
`Retry-After: 1` immediately, without queuing the operation, claiming a durable
job, or consuming a foreground worker. The one-second hint is not a completion
promise. Existing read/status, stop and deactivation retry operations remain
available through foreground admission; accepted durable retries still dispatch
through their scheduled lane.

Each configured target (`ldap/id`, `workspace/id`, `entra/id` or `scim/id`)
admits one active operation across manual requests, reconciliation, reviewed
SCIM delivery and offboarding deactivation. Apply requests resolve the target
from the stored plan or retained SCIM job, so different plan IDs cannot bypass
admission. The fixed registry has two general entries and one entry reserved
for deactivation, with no waiters. A cancelled or timed-out caller keeps its
target permit until the operation and
durable finish actually return. Busy scheduled candidates are skipped before
claiming: their status, leases, attempts and retry times are unchanged.

The deactivation runtime and target entry are reserved even when idle; manual,
reviewed and reconciliation work cannot borrow them. Two unrelated slow manual
targets therefore cannot prevent a due deactivation on a third target from
getting a service opportunity. Production starts independent reviewed and
deactivation loops at the same 250 ms cadence. Deactivation is checked even
without configured targets, and does not wait for reviewed provisioning to
succeed or return. The production reviewed worker calls `provisioning_step`
only, so it no longer schedules a second deactivation leg.

The reserved entry still excludes an active operation with the same
`scim/<target>` key in either lane. The existing synchronous `deactivation_step`
owns claim, every-send authority/dispatch fences and durable finish/ack. Its
target and pass permits live until that call returns. Scheduler cancellation,
timeout or shutdown does not settle a durable dispatch pin or initiate recovery.
Local revocation and intent enqueue retain their foreground transaction and
never wait for connector admission or remote delivery. A pass can update held
or closed rows without dispatching; runtime completion is not remote success.

The SCIM claim paths inspect at most 16 due index entries per pass. Two durable
`connector_due_cursors` records advance past inspected entries and wrap at a
fixed due-time cutoff, preventing a busy target's oldest page from hiding
unrelated due work. New retries do not extend the current sweep. Reconciliation
keeps its existing bounded 256-job, oldest-due selection and skips active
targets. Selection may interleave targets; it does not reorder resources inside
a reviewed job, split claim/dispatch/finish, bypass current authority or lease
fences, or change local revocation before downstream intent/delivery ordering.

Manual requests also have a 60-second response deadline. An admitted operation
that exceeds it returns `409 connector_operation_pending` without a retry hint;
it may still commit, so inspect durable state before retrying. The CLI treats
this as a conflict, not a retryable outage. Request cancellation and router
replacement retain both the shared executor and occupied slots until work
actually finishes. Audit attribution, revision guards, idempotency receipts,
removal confirmation and local-revocation/outbox ordering still execute inside
the original operation. This adds runtime admission errors, not delivery outcome
fields. Manual metrics use the fixed job label `manual_connector`.

A 60-second deadline bounds the scheduler's wait. It reports
`background_timeout`; it does **not** cancel a transaction, classify a remote
outcome, expire a lease or release capacity. The running pass keeps its job and
lane slots until all work finishes. Later ticks report `background_overloaded`
without starting a replacement. Completed late work retains its normal durable
result. A process stop leaves interrupted work and durable dispatch pins to
the queue's existing recovery rules; the runtime never initiates pin recovery.

Worker retries use the existing cadence: provisioning and deactivation 250 ms,
logout/SSF 2 s, mail and reconciliation 5 s, maintenance and alerts 60 s. Missed ticks are
skipped rather than replayed in a burst. These intervals are admission retry
hints, not promises of downstream completion, and do not override durable job
backoff. Warnings include the finite `job`, `lane`, error `code` and
`retry_after_ms`; unchanged admission errors are logged once until recovery.

Authenticated JSON metrics expose this policy and each worker's `active`,
`finished`, `failed`, `deferred`, `target_deferred`, `timeouts` and
`retry_after_ms` under `runtime.background`. Prometheus exposes `riauth_background_active`,
`riauth_background_finished_total`, `riauth_background_failed_total`,
`riauth_background_deferred_total`, `riauth_background_target_deferred_total`,
`riauth_background_timeouts_total`,
`riauth_background_retry_after_seconds` (job/lane labels) and
`riauth_background_capacity` (lane label). `finished` counts all terminated
passes, including failures; it is not a remote delivery outcome.
`target_deferred` counts target admission refusals and skipped candidates.
One counter folds three causes: the local target slot is busy, another process
holds the shared `connector_admissions` lease, and the lease ledger is full.
It does not say which. These use only fixed job/lane labels, never target IDs
or URLs. `failed` counts only passes whose work returned an error (a store or
serving-state failure, for example), and it stays set when the pass task is
cancelled or panics. A reconciliation job that is executed and stored as
`failed` or `stale` is a completed pass and does not increment
`riauth_background_failed_total`: `reconciliation_process` returns success once
the outcome is stored. Read that failure on `GET /api/operations/reconciliation`.
There is no reconciliation queue gauge, and `riauth_queue_failed` does not
include it. JSON policy
reports `connector_target_capacity: 1`. Rising timeouts/deferred counts with
occupied slots identify the lane to investigate.
Reserved-worker metrics use the fixed job/lane label `deactivation`; its capacity
is one. SCIM target deferrals are attributed to `provisioning` or `deactivation`
according to the claim path. Storage activity for both remains `provisioning`.
Check its connector timeout, network dependency and storage contention; do not
force a second claim or infer successful delivery from a runtime timeout.

OIDC end-session commits local revocation and its outbox before replying. It
now leaves back-channel dispatch to the durable worker, rather than waiting for
an inline network attempt; under normal capacity the next delivery tick is
within two seconds. Embedded users of `api::router` must run a delivery worker
(the production `serve` and bootstrap handoff do so).

Lane budgets, pass budgets and the local target slots are process-local runtime
isolation, not a latency or resource reservation for the whole deployment.
Storage locks/connections, CPU and memory remain shared; synchronous connector
calls are not forcibly interrupted. Separate store handles and processes have
separate lane and pass budgets and separate local slots.

Target exclusion also has a shared part. Each admission writes a
`connector_admissions` row keyed by a digest of the target. Scheduled work
writes it in the durable job claim's own transaction; manual work uses a short
writer before it starts. Processes on one shared PostgreSQL store therefore
refuse a target another process holds. The lease lasts 60 seconds and is not
renewed, and the ledger holds at most 128 live rows. A full ledger refuses a
new target and never evicts a live owner; an expired row is replaced. The lease
is not a fence around remote I/O. Another process can be admitted once 60
seconds have passed while the first pass is still running, and a suspended
process can resume after its lease expired, so each job's own dispatch and
finish fences remain the safety boundary. The source is
`src/background/targets.rs`.

Direct Core callers, protocol
login/federation exchanges and bulk administrative operations outside the listed
connector routes retain their existing limits (durable Core claim paths also
use target admission). This prevents one configured target occupying both
general connector slots; it does not reserve a turn for each caller. Multiple
busy target IDs can still fill the general lane. A deactivation for one of those
same targets must wait for exclusion, and a stalled deactivation occupies the
single reserve until it returns. One stalled scheduled pass still delays its
own serialized worker. Strict fairness between manual and scheduled work,
per-tenant or endpoint-wide quotas, reserved CPU/storage capacity, cross-node
quotas beyond the per-target lease, hard process isolation and production load/latency characterization
remain O05 follow-up work.

## Rate limits and admission

**Per-address limits.** Every request counts against one category per client address and 60-second window; the categories, defaults and routes are listed in the [API contract](api.md#proxy-authorization-and-limits). The client address is the socket peer, or the forwarded address from a trusted proxy. IPv4 addresses count individually, an IPv4-mapped IPv6 address counts as its IPv4 address, and other IPv6 addresses are grouped by /64, so one host cannot escape a limit by rotating addresses in its prefix. An exceeded limit returns 429 `rate_limited` ("Too many requests; retry in 60 seconds") and counts in `riauth_rate_limited_total`. With PostgreSQL, counters are shared by all nodes, including `forward_auth`, which counts through the reserved forward permits before its authorization check. With redb every counter stays in memory. The counter table holds up to 100,000 windows (one per address and category) in memory on each node, and as many in PostgreSQL. When it is full, the oldest window is dropped to make room: many addresses sending a request each can reset other clients' counts early, but they cannot lock clients the table has not seen out of the server.

**Overrides.** Optional `rate_limits` in `riauth.toml` replaces the default of any category; omitted categories keep their defaults. The allowed keys are `portal_start`, `portal_approve`, `login`, `passkey`, `account`, `source_start`, `source_callback`, `saml`, `mfa`, `device_start`, `device_verify`, `browser_decision`, `browser_state`, `forward_auth`, `outpost_start` and `general`. Values must be 1–100000 requests per minute; an unknown key or an out-of-range value stops the server at startup. Put the table after all top-level keys:

```toml
trusted_proxies = ["192.0.2.20"]  # replace with the actual proxy peer IP

[rate_limits]
login = 60            # default 20: shared by CLI and browser sign-in
outpost_start = 120   # default 30: forward-auth logins
browser_state = 3000  # default 1200: sign-in page polling
general = 2000        # default 600
```

Choose initial limits with shared office addresses in mind. An open sign-in page polls its state every 15 seconds while visible, and every 2 seconds while the terminal panel is open, so `browser_state` at 1200 allows about 300 visible pages, or 40 with the terminal panel open, per address. `login` counts every password attempt from the portal, the interaction pages and the CLI. The per-username lockout (5 failures within 15 minutes locks the name for 15 minutes) is separate and cannot be tuned; passkey sign-in is not affected by it.

**Shared policy and offline upgrade.** Security agreement format 3 records the
effective value of every category, including defaults for omitted overrides.
Omission and an explicit default agree. Startup compares that map, issuer,
active capabilities, token lifetimes and password history before migration,
backfill or edition stamping. A mismatch names its category and recorded limit:
set that `rate_limits` entry to the recorded value on every node and restart.
Existing format-3 policy is preserved; the record command does not overwrite a
different rate or authentication policy. A deliberate policy change needs a
separately reviewed migration, not deleting the agreement to bypass its checks.

Formats 1 and 2, and an initialized store with no agreement, now refuse startup
and remain unchanged. For an upgrade, verify a pre-upgrade backup with all
encryption keys, using the previous compatible release or a stopped-store
database/storage backup. Stop **every** server, gateway, worker, listener and
administration process on every node. Align issuer, active capabilities,
authentication policy and all effective rates, then use the maintenance binary
matching the source edition:

```sh
riauth-maintenance --config /etc/riauth/riauth.toml --json security-agreement-record --confirm-authentication-policy --confirm-rate-limits
```

Only for a deliberately reviewed missing-row adoption, add
`--adopt-missing-agreement`. It does not bypass a present incompatible row.
The command writes one complete agreement or nothing; a matching format-3 retry
reports `recorded: false`. redb requires exclusive ownership. PostgreSQL rejects
other connected sessions named `riauth`, but that check cannot prove every
process is stopped: checking and stopping all nodes is the operator's duty.
Keep the pre-upgrade backup. Older binaries refuse format 3; rollback requires
restoring the compatible backup and reconciling credentials/recovery, not editing
the format number. Restart only participating binaries that implement this
agreement and the accepted shared ledgers. The agreement does not discover peer
versions, revoke a paused old process, or establish deployed HA.

Shared connector admission still expires after 60 seconds and is not renewed.
It provides no atomic fence for a process paused after admission and before
external IO. Existing SCIM freshness/lease completion fences remain separate.

**Admission queues.** Blocking work runs on bounded queues, each with a two-second wait before 503 `temporarily_unavailable` ("Server busy; retry shortly"):

| Queue | Permits | Used by | On timeout |
| --- | ---: | --- | --- |
| Workers | 8 | Every blocking handler and upstream source callbacks | 503, counted in `riauth_worker_rejections_total` |
| Credentials | 4 | Password checks: `/api/login`, `/api/password`, `/api/portal/login/password`, `/api/portal/password`, `/api/portal/account/{accept,verify,reset}`, interaction `…/password`; taken before a worker | 503 (in `riauth_server_errors_total`) |
| Forward auth | 16 | `/outpost/{id}/auth` and `/outpost/{id}/traefik`, instead of a worker | 503 (in `riauth_server_errors_total`) |

A password check keeps its credential and worker permits until it finishes, even when the client disconnects first, so password hashing never occupies more than half the workers, and forward-auth checks never wait behind sign-ins. Occasional 503s during a morning sign-in rush mean the node is CPU-bound on Argon2: the portal and sign-in page retry idempotent reads, but never resubmit a password. Add CPU or nodes, or spread load, rather than raising limits.

**Failure floor.** Every failed password sign-in (`invalid_credentials` from `/api/login`, `/api/portal/login/password` or an interaction `…/password`) is answered no sooner than one second after the request started, so response time does not reveal whether an account exists, is locked, or uses an imported hash. The wait happens after the worker and credential permits are released, so it does not reduce capacity. Imported hashes that take longer than one second to verify remain distinguishable until their first successful login rehashes them. For an account bound to an LDAP directory, the whole directory exchange (connect, service bind, entry re-check and user bind) must finish within 0.8 seconds; a directory that is slower, hung or unreachable counts as unavailable, so it answers inside the floor like any other failure and holds a credential permit for at most that long. Keep the directory close to riAuth: a sign-in that needs longer against a healthy directory fails as `directory_unavailable` in the terminal and as the uniform 401 in the browser.

## Native HTTPS

Set `tls_cert_file` and `tls_key_file` together and use an HTTPS issuer. Paths resolve beside `riauth.toml`; PEM certificate chains and matching private keys are loaded by rustls. The server reloads both files every minute. When `client_certificates` is configured, that reload also refreshes the client trust anchors and CRLs; certificate login additionally reads them on each attempt. A failed replacement retains the active configuration. Graceful shutdown allows up to 30 seconds for native TLS connections to finish. CLI clients can trust a private CA with `--ca-cert`; disabling certificate verification is not an option.

HTTPS client-certificate login is optional and uses explicit user bindings. Both `mode = "optional"` and `mode = "required"` permit anonymous TLS handshakes; the certificate login endpoint rejects missing certificates. TLS session resumption is disabled when the profile is enabled. For proxy termination, accept certificate headers only from configured immediate proxy peers and ensure the proxy overwrites them. See the [certificate trust and revocation contract](enterprise/ENT-05.md).

See [PostgreSQL and availability](availability.md) for shared storage and failover, [external signing](kms.md) for Vault Transit, and [account delivery](lifecycle.md) for SMTP leases and status.

## Capacity planning

Measure the intended deployment before setting capacity expectations. Include TLS, network distance, upstream authentication, external signing, representative directory size, and actual storage topology. Record the tested build, CPU, memory, concurrency, completed requests, failures, and latency percentiles.

## Bounded work and schema 3

Authentication and OIDC token issuance prepare expensive work without holding the
writer or a PostgreSQL pool connection. Commit revalidates every point/range read,
checks tracked expiry boundaries, and atomically applies staged writes. Conflicts
retry at most four times, then return a retriable 503. Revocation and replay state
remain authoritative at commit. Management transactions and shared PostgreSQL HTTP
quota counters still use serialized writes; watch the lock/pool histograms before
raising concurrency or replica counts.

Cleanup visits at most 128 records per collection per pass, keeps durable cursors,
and releases the writer between protocol groups. Each sweep reads one final key
as a fixed boundary so continuing appends cannot prevent revisiting older records.
Backlogs can require multiple passes; expired records are rejected during
authentication independently of cleanup.
Grant issuance maintains a monotonic parent-session retention index, so a client
refresh TTL longer than the instance default cannot cause premature session removal.
Logout, email, SCIM, SCIM deactivation, SSF and offboarding claims read bounded due-time indexes;
quota counts are maintained atomically, with bounded expired-entry reclamation
when capacity is reached. Queue counts and oldest age need only point reads and
one index entry per queue. Offboarding handles at most eight jobs per maintenance
pass and rechecks the creator’s parent authority before committing. Effective PAM
groups use a per-user unrevoked-grant index; expired grants never confer access
while waiting for cleanup. Retained-history regressions bound records examined,
while deployment backlog deadlines still require representative measurement.

Schema 3 builds derived indexes atomically during the first upgrade. A separate index revision backfills newer indexes when opening an existing schema-3 store; restore also rebuilds them. Index revision 4 adds the per-account outbound-link index that records SCIM deactivation intent. An older writer left running would add links without that index, so their deactivation intent would be missing. Stop all
older writers before upgrading and keep a verified pre-upgrade backup. See the
[release compatibility contract](release-notes.md). With record encryption enabled,
index values are encrypted but timestamp/hash keys expose scheduling metadata.

Backup creation seals pages of up to 128 records inside one consistent snapshot.
The legacy v2 JSON writer limits serialized plaintext pages to 8 MiB and the complete
encrypted JSON archive to 64 MiB. Restore accepts older v2 chunks with larger
plaintext pages within that archive ceiling; v1 restore also materializes its
complete plaintext payload. Oversized backups fail explicitly; they do not truncate records.
The encrypted chunks, manifest and final JSON response remain buffered within the
archive bound. Restore rejects oversized files before reading them and decrypts
one chunk at a time without retaining decoded ciphertext copies. Version-1
archives remain readable within the archive limit; the legacy JSON endpoint writes version 2.
`riauth backup` and the streaming endpoint write the framed `riauth.backup/v3`
format, which `restore` recognises by its leading magic; only the legacy JSON
endpoint still writes v2.
A v3 archive is a sequence of AES-256-GCM frames whose associated data binds the
archive identity, frame position and kind, closed by a trailer that commits to the
record and frame counts and a transcript hash. Restore authenticates the complete
stream and validates schema and issuer before creating the output directory, then
re-reads and re-authenticates it while importing. The codec bounds its frame
buffers and applies configurable archive and frame quotas (default 4 GiB and
8 MiB), progress callbacks and cancellation. Source paging checks stored key
and value bytes before decoding, and restore/index validation scans one record
at a time. These checks do not establish a bound on process memory: decoded
JSON, redb caching and the restore write transaction still need peak-RSS
measurement on representative and adversarial data.
### Streamed backup export

`POST /api/operations/backup/stream` needs `operations.backup` on
`operations/backup` and sends one consistent snapshot as a v3 archive while it
is sealed. Authorization, the key and the quota are checked before the response
starts, so those failures are ordinary JSON errors; a process runs one export at
a time and answers 503 while another is running. The export hands frames to the
connection through a bounded queue of at most 1 MiB, so a slow client slows the
export rather than growing server memory. A client that disconnects cancels the
export and ends its read snapshot. The export is also cancelled when the client
accepts no data for `stall_timeout_seconds`, when it is still running after
`max_duration_seconds`, and when its server starts a graceful shutdown
(SIGTERM or Ctrl-C), after which that server answers new exports with 503.
The export holds a redb read transaction or a PostgreSQL `REPEATABLE READ`
transaction and pooled connection for its whole duration. A
failure after the response started aborts the body instead of ending it, and
the partial archive has no trailer, which verification and restore reject. On
shutdown the server also drops archive bytes still queued for the client.

Every export that passes authorization records durable audit events for its
actor with target `backup/<stream ID>`. `operations.backup.started` is committed
before the export opens its snapshot, so the archive contains it; if it cannot
be recorded, the export is refused. At most one terminal event follows, written
after the export released its snapshot: `operations.backup.completed` (frames,
records, bytes, transcript) only after every archive byte, trailer included,
was handed to the connection; otherwise `operations.backup.cancelled` (request
ended before the export began, client disconnected or stopped reading, deadline,
shutdown) or `operations.backup.failed`
(for example the quota), each with the reason and the bytes handed over. The
export writes a failure itself, so a client that keeps the connection open
without reading still gets a terminal event after `stall_timeout_seconds`; the
same holds when the export finished but its last queued bytes are not taken.
An export that finished sealing while its response was cut short is recorded
as cancelled, not completed, and its response then never completes; only the
client's verification confirms receipt. A started export can remain without a
terminal event if storage fails while it is written (the server logs the
failure) or if the process exits while the event for a disconnected client is
still being written in the background. Details never contain the backup key or
a credential. The server also logs each event and the export's progress every
10 seconds.

On PostgreSQL an export holds one pooled connection for its whole duration.
It works with `pool_size = 1`, but then other requests that need storage wait
for that connection (up to 5 seconds) and fail with 503 `storage_busy` until
the export ends; size the pool for backups taken while serving.

Server quotas live in an optional `[backup]` table; put it after all top-level keys:

```toml
[backup]
max_archive_bytes = 4294967296   # 1 MiB through 4 GiB (default 4 GiB)
max_frame_bytes = 8388608        # 4 KiB through 8 MiB (default 8 MiB)
stall_timeout_seconds = 60       # 5 through 3600
max_duration_seconds = 3600      # 60 through 86400
```

A request may lower the archive quota with `max_archive_bytes`; it cannot raise
it. An archive that would exceed the quota stops before the frame that crosses it.

`riauth backup` writes the stream to a new `0600` file beside `--out`, enforces
`--max-bytes` (default 4 GiB) while receiving, syncs the file, then
authenticates every frame, the record order, the trailer's counts and transcript,
EOF, schema and issuer with the backup key, as `restore` does before it creates
output. Only a verified archive is linked under `--out`, which is never
overwritten; the destination directory must support hard links, which
`riauth backup` checks before it transfers anything. Interrupted, oversized, cancelled (Ctrl-C) or unverifiable
transfers leave nothing under `--out`. The result reports `verified`,
`stream_id`, `frames`, `records`, `bytes`, `transcript` and `issuer`; the stream
ID matches the server's completion log. On an interactive terminal without
`--json`, progress appears on stderr.

The CLI deadline is configurable with `--request-timeout SECONDS` or
`RIAUTH_REQUEST_TIMEOUT` (default 30, range 1..=86,400), including body receipt.
For `riauth backup` it bounds the connection and each wait for more data
instead; the transfer has no total deadline, and the server's
`max_duration_seconds` bounds the export.
Measure HTTP backup and CLI restore with representative records, external keys, and the intended storage configuration. Record archive size, elapsed time, peak memory, and whether the restored service can sign in and serve applications. Set `backup.max_duration_seconds` from measured backup time when the default hour is insufficient. Keep results with the recovery procedure and repeat them after storage, configuration, or data-volume changes.

## Release verification

The [testing guide](testing.md) describes source and integration checks. A
tagged release may include Essentials, Platform, riauthctl and maintenance
archives built on Ubuntu 24.04 x86-64 and ARM64, plus matching container
archives, checksums and build provenance for each architecture. Build from
source when using another platform or an incompatible Linux environment.
Verify the architecture-specific checksums and match the provenance commit to the
version you intend to deploy. Test startup, proxy behavior and
restore in the target environment before directing users to the new version;
a container capabilities smoke test does not exercise those paths. See the
[release notes](release-notes.md) and [known limitations](limitations.md) for
version-specific compatibility and acceptance boundaries. A checkout-only
reading of the workflow and of the vulnerability intake text is the
[Q11 release-evidence check](roadmap/q11-release-evidence.md). It does not
download artifacts or replace checksum verification of a bundle you deploy.
That procedure also describes [scripts/spdx_sbom.py](../scripts/spdx_sbom.py).
`package-linux` binds the locked Essentials, Platform, and riauthctl graphs
to the exact Linux archive and image names, and the Linux packager source
calls it. The bundle checker and the installed gate require those document
bytes to match the files. This output is not a release SBOM. Reading this
page does not run the packager.
