# Deployment, backup and recovery

This is the v0.1 operator runbook. The default redb backend has one owning process, snapshot reads and serialized commits. PostgreSQL supports multiple service processes with shared sessions, replay state, rate limits and delivery leases; see [availability](availability.md). Database election, replication policy and fencing belong to the deployment. Do not start two servers against the same redb file.

## Before exposing an instance

1. Choose redb for one service process or PostgreSQL for multiple processes. Keep the redb state directory private to one host and one running server.
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
sudo install -d -o riauth -g riauth -m 0700 /var/lib/riauth /etc/riauth
sudo -u riauth /usr/local/bin/riauth keygen --out /etc/riauth/database.key
sudo -u riauth sh -c 'cd /var/lib/riauth && /usr/local/bin/riauth init --issuer https://id.example.com --listen 127.0.0.1:9000 --database-key-file /etc/riauth/database.key'
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
riauth keygen --out database.key
riauth init --database-key-file database.key
riauth serve
```

While the service runs, use the same working directory in another terminal to sign in, generate a **separate backup key**, and take a backup. `backup` is an authenticated request to the running service; `restore` is a local offline operation that creates a new directory. Keep the backup key in your recovery secret store. The longer request deadline is useful for stores that exceed the default 30 seconds:

```sh
riauth login admin
riauth keygen --out backup.key
riauth --request-timeout 300 backup --key-file backup.key --out backup.json
riauth restore --backup backup.json --key-file backup.key \
  --out restored --database-key-file database.key
```

Check the `restored/` directory and the command's `verified` result. To rehearse service recovery, make the external keys and referenced files available in an isolated environment, then start that restored configuration there and test administrator login, JWKS and representative application flows. Do not expose a second server under the live issuer. For a remote service, use `riauth --server https://id.example.com` for `login` and `backup`; the account running `restore` needs read access to the archive and keys. Restore always creates a redb instance, even when the backup came from PostgreSQL. Returning to multi-node PostgreSQL service requires an offline migration into an empty database or a separately verified PostgreSQL recovery plan; see [availability](availability.md).

Backup takes a consistent database read snapshot while the server runs and encrypts the configuration and records with AES-256-GCM. New backups are authenticated chunks (`riauth.backup/v2`) so the plaintext snapshot is not assembled as one object. Restore still accepts that envelope and the earlier single-blob `riauth.backup/v1` envelope. Restore authenticates/decrypts the envelope, validates schema/issuer, requires a new output directory, checks signing keys and an enabled administrator, and reopens the restored store. A v2 manifest authenticates the chunk order, count and digests so truncated or reordered archives fail validation. Archives produced by the v2 writer require a reader that supports v2, even though the database schema remains 3. It does not start a server or overwrite an existing directory. A failed post-decryption validation can leave a new private directory for inspection.

The backup key, optional live-storage key and contents of external credential/certificate files are not included in the backup. Restore preserves their configuration references; re-provision and review those paths before starting the recovered service. Vault private keys remain in Vault. Losing the corresponding key makes those encrypted data unrecoverable. Record values are encrypted at rest; record names, sizes and access patterns are not hidden. Encryption does not protect a running compromised process. Backups contain operational secrets and should be treated as complete instance credentials.

To encrypt an existing plaintext database, take a backup, restore to a new directory with `--database-key-file`, verify the result, stop the old server and switch configuration. Never enable encryption by changing only the key field on an existing plaintext database. Key rotation for storage uses the same backup/restore procedure into a new directory.

## Upgrade and rollback

This version uses logical schema 3. Opening a schema-1 or schema-2 store performs an atomic migration. Schema 1 first backfills pairwise-subject seeds; schema 3 rebuilds due-time, age, quota-count/expiry and parent-session retention indexes. Credentials and signing keys are preserved, migrations are recorded, and the configuration revision advances. Unknown future schemas are rejected without mutation.

For an upgrade, use this order:

1. Record `riauth --version` and the deployed binary hash. While the old service is still running, take an encrypted backup with that version and run `restore` into a new directory to verify the archive. Preserve the old binary, backup key, database key and referenced external files.
2. Confirm the rollback binary can read that archive format. New `riauth.backup/v2` archives need a v2-capable reader; a compatible backup made by the old version may be necessary. Test the entire restore and startup path before depending on it.
3. Stop the old service and all other writers. For redb, never overlap the two server processes on one file. For PostgreSQL, coordinate all nodes and database failover/fencing before allowing a new writer.
4. Replace the binary and start the new service against the **existing** configuration and store. Its first open performs any required migration. Check `/readyz`, `riauth doctor`, administrator login and representative OIDC/SAML/application flows before restoring traffic.
5. If rollback is required, stop every new writer and restore the pre-upgrade backup into a **new** redb directory with the compatible older reader and keys. For a redb deployment, point the old binary at that restored configuration. For PostgreSQL, use a rehearsed database recovery or migrate the restored redb store offline into a fresh PostgreSQL database before resuming multi-node service. Verify login and applications again. Do not run an old binary against a store the new binary has opened or modified.

Backup frequency determines your recovery point. Restore, service restart, DNS/proxy updates and application verification determine recovery time; neither has an HA guarantee in this implementation.

## Diagnostics and recovery

`doctor` reports schema, revision, administrator count, signing-key health, storage encryption and pending logout deliveries. `metrics` returns JSON process counters and worker capacity; counters reset at process start. Operational details require permissions.

Use `/livez` for process liveness and `/readyz` for traffic readiness. Both are nested under a path-based issuer and bypass application quotas. Liveness performs no database work. Readiness verifies the storage schema, checks that PostgreSQL is writable, and rejects saturated application workers. Storage checks have separate bounded concurrency and a two-second response deadline; an outstanding database call keeps its permit until it ends. `/healthz` is a compatibility alias for readiness. A database outage should remove traffic through readiness rather than trigger a liveness restart loop.

HTTP handlers admit blocking crypto/database work through eight application workers, waiting up to two seconds for one; probes, background workers and protocol listeners have their own execution paths. Network limits are applied per effective client address; account failures additionally lock an existing username for 15 minutes after five failures. See [rate limits and admission](#rate-limits-and-admission). The maintenance loop runs about once per minute and removes expired protocol records in bounded pages; backlogs can take multiple passes. Audit retention is 90 days. Cleanup also removes retained PAM records and processes up to eight due offboarding jobs per pass; PAM access expires at the grant deadline independently of cleanup. Scheduled offboarding disables access in riAuth and queues RP logout; downstream SCIM deactivation remains unimplemented ([offboarding contract](enterprise/ENT-10.md)).

Logout delivery uses signed, audience-bound events, bounded concurrent requests, certificate verification, a five-second timeout and no redirects. Failures retry with increasing delay for up to 24 hours; delivery records remain seven days. Inspect `riauth deliveries` with the corresponding permission. The RP must verify signatures/claims, target `sid`, and reject repeated `jti` values. Outbox delivery does not invalidate an RP that ignores logout events.

For break-glass administrator recovery, stop the server and use trusted local filesystem/key access:

```sh
riauth --config /path/to/riauth.toml recover-admin admin --reset-mfa
riauth --config /path/to/riauth.toml serve
```

Recovery resets the password, restores enabled administrator access, clears MFA when requested, and invalidates old sessions/grants. The last enabled administrator is protected during ordinary management. Browser and offline OAuth grants have their own expiry rules; recovery must be verified through the applications as well as the CLI.

Prometheus metrics are served at `/api/operations/prometheus` with `operations.read=operations/metrics`. `riauth metrics --prometheus` returns the exposition text in a JSON field. Requests rejected by header or trusted-proxy validation are included. Separate counters report 4xx client errors, 5xx server errors, 401/403 authentication rejections, rate limiting and worker admission failures. The original combined error counter remains for compatibility. Per-route latency uses registered route templates, finite method names and status classes; usernames, arbitrary paths and credentials are never metric labels.

Runtime histograms cover writer wait/hold time, PostgreSQL pool waits, signing, password verification and maintenance. Counters expose signing/maintenance errors, alert delivery failures, scanned records and optimistic transaction conflicts. An optional `alert_webhook` can POST selected local conditions (storage not ready, cleanup errors, signing failures). That route is not a substitute for deployment paging. See [alert routing](enterprise/PLATFORM-04.md). All counters reset on process restart. Metrics authentication still requires storage; monitor scrape availability and probes as well as metric values. [Scrape configuration](../deploy/prometheus.yml) and [alert rules](../deploy/riauth-alerts.yml) provide starting points; set thresholds from deployment load measurements. Keep the raw monitoring token in the referenced private secret file and rotate it before expiry.

## Rate limits and admission

**Per-address limits.** Every request counts against one category per client address and 60-second window; the categories, defaults and routes are listed in the [API contract](api.md#proxy-authorization-and-limits). The client address is the socket peer, or the forwarded address from a trusted proxy. IPv4 addresses count individually, an IPv4-mapped IPv6 address counts as its IPv4 address, and other IPv6 addresses are grouped by /64, so one host cannot escape a limit by rotating addresses in its prefix. An exceeded limit returns 429 `rate_limited` ("Too many requests; retry in 60 seconds") and counts in `riauth_rate_limited_total`. With PostgreSQL, counters are shared by all nodes, except `forward_auth`, which is always counted in memory and so applies per node. The counter table holds up to 100,000 windows (one per address and category) in memory on each node, and as many in PostgreSQL. When it is full, the oldest window is dropped to make room: many addresses sending a request each can reset other clients' counts early, but they cannot lock clients the table has not seen out of the server.

**Overrides.** Optional `rate_limits` in `riauth.toml` replaces the default of any category; omitted categories keep their defaults. The allowed keys are `portal_start`, `portal_approve`, `login`, `passkey`, `account`, `source_start`, `source_callback`, `saml`, `mfa`, `device_start`, `device_verify`, `browser_decision`, `browser_state`, `forward_auth`, `outpost_start` and `general`. Values must be 1–100000 requests per minute; an unknown key or an out-of-range value stops the server at startup. Put the table after all top-level keys:

```toml
trusted_proxies = ["192.0.2.20"]  # replace with the actual proxy peer IP

[rate_limits]
login = 60            # default 20: shared by CLI and browser sign-in
outpost_start = 120   # default 30: forward-auth logins
browser_state = 3000  # default 1200: sign-in page polling
general = 2000        # default 600
```

Raise limits when many staff share one public address (office NAT). An open sign-in page polls its state every 15 seconds while visible, and every 2 seconds while the terminal panel is open, so `browser_state` at 1200 allows about 300 visible pages, or 40 with the terminal panel open, per address. `login` counts every password attempt from the portal, the interaction pages and the CLI. The per-username lockout (5 failures within 15 minutes locks the name for 15 minutes) is separate and cannot be tuned; passkey sign-in is not affected by it.

**Admission queues.** Blocking work runs on bounded queues, each with a two-second wait before 503 `temporarily_unavailable` ("Server busy; retry shortly"):

| Queue | Permits | Used by | On timeout |
| --- | ---: | --- | --- |
| Workers | 8 | Every blocking handler and upstream source callbacks | 503, counted in `riauth_worker_rejections_total` |
| Credentials | 4 | Password checks: `/api/login`, `/api/password`, `/api/portal/login/password`, interaction `…/password`; taken before a worker | 503 (in `riauth_server_errors_total`) |
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
Logout, email, SCIM, SSF and offboarding claims read bounded due-time indexes;
quota counts are maintained atomically, with bounded expired-entry reclamation
when capacity is reached. Queue counts and oldest age need only point reads and
one index entry per queue. Offboarding handles at most eight jobs per maintenance
pass and rechecks the creator’s parent authority before committing. Effective PAM
groups use a per-user unrevoked-grant index; expired grants never confer access
while waiting for cleanup. Retained-history regressions bound records examined,
while deployment backlog deadlines still require representative measurement.

Schema 3 builds derived indexes atomically during the first upgrade. A separate index revision backfills newer indexes when opening an existing schema-3 store; restore also rebuilds them. Stop all
older writers before upgrading and keep a verified pre-upgrade backup. See the
[release compatibility contract](release-notes.md). With record encryption enabled,
index values are encrypted but timestamp/hash keys expose scheduling metadata.

Backup creation seals pages of up to 128 records inside one consistent snapshot.
The current writer limits serialized plaintext pages to 8 MiB and the complete
encrypted JSON archive to 64 MiB. Restore accepts older v2 chunks with larger
plaintext pages within that archive ceiling; v1 restore also materializes its
complete plaintext payload. Oversized backups fail explicitly; they do not truncate records.
The encrypted chunks, manifest and final JSON response remain buffered within the
archive bound. Restore rejects oversized files before reading them and decrypts
one chunk at a time without retaining decoded ciphertext copies. Version-1
archives remain readable within the archive limit; new backups use version 2.
The CLI deadline is configurable with `--request-timeout SECONDS` or
`RIAUTH_REQUEST_TIMEOUT` (default 30, range 1..=86,400), including body receipt.
Measure HTTP backup and CLI restore with representative records, external keys, and the intended storage configuration. Record archive size, elapsed time, peak memory, and whether the restored service can sign in and serve applications. Set the request timeout from measured backup time when the default 30 seconds is insufficient. Keep results with the recovery procedure and repeat them after storage, configuration, or data-volume changes.

## Release verification

The [testing guide](testing.md) describes source and integration checks. A
tagged release may include a native binary archive built on Ubuntu 24.04 x86_64
for compatible Linux systems, a container archive, checksums and build provenance.
Build from source when using another platform or an incompatible Linux
environment. Verify the checksums and match the provenance commit to the
version you intend to deploy. Test startup, proxy behavior and
restore in the target environment before directing users to the new version;
a container capabilities smoke test does not exercise those paths. See the
[release notes](release-notes.md) and [known limitations](limitations.md) for
version-specific compatibility and acceptance boundaries.
