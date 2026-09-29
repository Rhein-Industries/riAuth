# Linux image deployment examples

These examples use the released Essentials or Platform container **archive** for
the host CPU architecture. They do not build an image or start a registry. Run
them on Linux with Docker Compose. The public issuer below is
`https://id.example.com` with no path prefix; change the hostname everywhere if
needed. Keep all private files outside the build context in the ignored
`deployment-private/` directory. The image runs as UID/GID 10001. The commands
assume a standard rootful Docker Engine with direct host UID mapping; rootless,
user-namespace and SELinux hosts need their mount ownership or labels adapted
and checked before use.

Download the complete release asset set for that architecture into
`target/dist`, then select and verify one image on every service host:

```sh
cd /path/to/riAuth
ARCH=$(uname -m)                         # x86_64 or aarch64
EDITION=essentials                       # or platform; use the same edition on every node
case "$ARCH" in x86_64|aarch64) ;; *) exit 2 ;; esac
(cd target/dist && sha256sum --check "SHA256SUMS-linux-$ARCH")
LOAD_RESULT=$(docker load --input "target/dist/riauth-$EDITION-linux-$ARCH.docker.tar.gz")
printf '%s\n' "$LOAD_RESULT"
RIAUTH_IMAGE=$(printf '%s\n' "$LOAD_RESULT" | sed -n 's/^Loaded image: //p')
test -n "$RIAUTH_IMAGE" && export RIAUTH_IMAGE
docker image inspect --format '{{.Os}}/{{.Architecture}} {{index .Config.Labels "org.riauth.edition"}} {{index .Config.Labels "org.opencontainers.image.revision"}}' "$RIAUTH_IMAGE"
```

The displayed platform must be `linux/amd64` for `x86_64` or `linux/arm64`
for `aarch64`; the edition and revision must match the selected archive and
`build-provenance-linux-$ARCH.json`. Pin the chosen image tag in the operator's
environment on each restart. Compose refuses to render without `RIAUTH_IMAGE`.

## One small instance: redb and a host TLS proxy

[`compose-small.yml`](../deploy/compose-small.yml) runs one service with one
named `/data` volume. That volume has **one owning process**; never mount it in
another riAuth container or put it on shared storage. The example uses Linux
host networking so the listener and [host Caddy](../deploy/Caddyfile) share
loopback. The host Caddy process owns public port 443 and the certificate;
riAuth listens only on `127.0.0.1:9000`.

Provision the private directory and an administrator password file from your
secret manager. The password file belongs to the host operator, has mode 0600,
contains one password line, and is removed after initialization:

```sh
install -d -m 0700 deployment-private
sudo install -d -o 10001 -g 10001 -m 0700 deployment-private/small
install -m 0600 /path/to/admin-password deployment-private/admin-password
docker compose -f deploy/compose-small.yml --profile tools run --rm -T init \
  keygen --out /config/database.key
docker compose -f deploy/compose-small.yml --profile tools run --rm -T init \
  --json --non-interactive init --issuer https://id.example.com \
  --listen 127.0.0.1:9000 --data-dir /data \
  --database-key-file /config/database.key --password-stdin \
  < deployment-private/admin-password
rm deployment-private/admin-password
```

Review `deployment-private/small/riauth.toml`. Replace its top-level
`trusted_proxies` value with `["127.0.0.1"]` for host Caddy; keep one copy of
the field above any TOML table headers.
Keep that file and `database.key` readable by UID 10001 only; the runtime
mounts the directory read-only. Replace `id.example.com` in the Caddyfile,
install the certificate or arrange ACME. Mount any additional referenced TLS, mail,
directory, signer or connector files at their configured absolute paths; a
database backup does not preserve them. Start and check the local service:

```sh
docker compose -f deploy/compose-small.yml config --quiet
docker compose -f deploy/compose-small.yml up -d riauth
docker compose -f deploy/compose-small.yml ps
curl --fail http://127.0.0.1:9000/readyz
```

On a host with Caddy installed as a systemd service, install and edit the
example before enabling the public route:

```sh
sudo install -m 0644 deploy/Caddyfile /etc/caddy/Caddyfile
sudoedit /etc/caddy/Caddyfile             # replace id.example.com
sudo caddy validate --config /etc/caddy/Caddyfile
sudo systemctl enable --now caddy
sudo systemctl reload caddy
curl --fail https://id.example.com/.well-known/openid-configuration
```

The Compose healthcheck calls the image's `riauth status` command against the
local readiness alias; it checks storage, not just process existence. Back up
the named volume through an authenticated encrypted riAuth backup, using a
**different** backup key held outside this host. Rehearse offline restore into
a fresh store before relying on the volume. See [backup and recovery](operations.md#encrypted-backup-and-restore).

## Two service hosts: one external PostgreSQL authority

[`compose-distributed.yml`](../deploy/compose-distributed.yml) runs **one**
riAuth process on each Linux host. It binds port 9000 on that host for an
external TLS load balancer and mounts the same configuration and key material
read-only from each host's private directory. There is no shared filesystem or
local database volume. Its startup guard requires a `[postgres]` table, so a
missing PostgreSQL stanza cannot silently create redb in `/data`. Restrict
port 9000 to the load balancer IPs with the host firewall.

First provision a dedicated empty PostgreSQL database and role, a trusted CA,
and a **separately operated** HA design. The database operator must choose
synchronous replication, backup/PITR, writable-host routing, promotion policy
and fencing of the old primary. riAuth does not elect or fence a database
primary. Its `synchronous_commit=on` does not configure a synchronous standby;
the database operator must set the standby and durability policy. In
`deployment-private/distributed/`, owned by UID 10001 and mode
0700, create these files before initialization:

| File | Contents and mode |
| --- | --- |
| `postgres-connection` | Owner-only 0600 libpq connection string with one to eight explicit TLS hosts, database, user and password; for example `host=db-primary.example.com,db-secondary.example.com dbname=riauth user=riauth password=...`. |
| `postgres-ca.pem` | CA certificate that verifies each PostgreSQL hostname. |
| `postgres.json` | Copy [the PostgreSQL file-mount configuration](../deploy/postgres.json.example), which sets `pool_size` to 8 per node. |
| `database.key` | Generated by the image command below; same bytes on every service host. |
| `riauth.toml` | Generated once by `init` below; same bytes on every service host. |

Use an external secret store or protected transfer to provision the same private
files on each host. The connection file and database key must remain owner-only;
set their owner to UID 10001 and mode to 0600 or 0400. Pool size is **per
process**: two nodes with `pool_size:8` can use up to sixteen connections.
Initialize **once**, on one host, while no service nodes are running:

```sh
install -d -m 0700 deployment-private
sudo install -d -o 10001 -g 10001 -m 0700 deployment-private/distributed
sudo install -o 10001 -g 10001 -m 0600 /path/from/secret-manager/postgres-connection \
  deployment-private/distributed/postgres-connection
sudo install -o 10001 -g 10001 -m 0600 /path/to/postgres-ca.pem \
  deployment-private/distributed/postgres-ca.pem
sudo install -o 10001 -g 10001 -m 0600 deploy/postgres.json.example \
  deployment-private/distributed/postgres.json
install -m 0600 /path/to/admin-password deployment-private/admin-password
docker compose -f deploy/compose-distributed.yml --profile tools run --rm -T init \
  keygen --out /config/database.key
docker compose -f deploy/compose-distributed.yml --profile tools run --rm -T init \
  --json --non-interactive init --postgres-config /config/postgres.json \
  --issuer https://id.example.com --listen 0.0.0.0:9000 --data-dir /data \
  --database-key-file /config/database.key --password-stdin \
  < deployment-private/admin-password
rm deployment-private/admin-password
```

Replace the top-level `trusted_proxies` list in the generated `riauth.toml`
with the **exact load balancer source IPs**; keep the issuer, feature configuration,
database key, PostgreSQL connection policy and external signer/trust files
identical on every node. Mount any other referenced secret, certificate or
policy files at the same absolute paths on each host. Do not run `init` on the
second host. Copy the private directory securely, retain UID 10001 ownership,
and compare file checksums
before starting either node. Load the **same edition, revision and CPU-specific
image** on each host; their image IDs may differ across CPU architectures, but
the source revision and feature set must match. On **each** host:

```sh
docker compose -f deploy/compose-distributed.yml config --quiet
docker compose -f deploy/compose-distributed.yml up -d riauth
curl --fail http://127.0.0.1:9000/readyz
```

Replace the certificate, hostname and node IPs in
[`haproxy-riauth.cfg.example`](../deploy/haproxy-riauth.cfg.example). On a
dedicated load-balancer host with HAProxy installed as a systemd service:

```sh
sudo install -d -m 0700 /etc/haproxy/certs
sudo install -o root -g root -m 0600 /path/to/id.example.com.pem \
  /etc/haproxy/certs/id.example.com.pem
sudo install -m 0644 deploy/haproxy-riauth.cfg.example /etc/haproxy/haproxy.cfg
sudoedit /etc/haproxy/haproxy.cfg         # replace hostname and both node IPs
sudo haproxy -c -f /etc/haproxy/haproxy.cfg
sudo systemctl enable --now haproxy
sudo systemctl reload haproxy
```

Its `/readyz` checks remove a node when storage
is unavailable. Test public discovery, an administrator sign-in, one real
application flow and removal/re-addition of each node before cutover. Keep
clocks synchronized and protect the plaintext load-balancer-to-node network.
The example load balancer is itself one process; operate a redundant pair if
load-balancer availability is required.

Every process runs the HTTP listener and its built-in reconciliation,
provisioning, mail, logout, SSF and maintenance workers. These files provide no
worker-only or scheduler-only role and do not disable workers on a second
node. PostgreSQL shares core session, replay, rate-limit and delivery state;
the forward-auth limit remains per node. Platform connector, device,
certificate, offboarding and other newer enterprise flows need separate
multi-node acceptance; this example does not establish their distributed
behavior. See [availability](availability.md) and [limitations](limitations.md).

For a PostgreSQL outage, readiness becomes 503; keep liveness separate and
retry uncertain management writes with the original idempotency key. Before a
database-native restore or promotion that may have lost committed state, stop
**both** service nodes and fence the old primary. Reconcile the restored
database under the [restored-state procedure](recovery.md). Before either
service restarts, use the offline tool service on one host:

```sh
docker compose -f deploy/compose-distributed.yml --profile tools run --rm -T init \
  recovery invalidate --database-restored
docker compose -f deploy/compose-distributed.yml --profile tools run --rm -T init \
  recovery status
# After reconciling/rotating the listed persistent credentials:
docker compose -f deploy/compose-distributed.yml --profile tools run --rm -T init \
  recovery complete --recovery-id THE_REVIEWED_ID --persistent-credentials-reconciled
```

Record the reviewed recovery ID and reconciliation evidence before reopening
traffic. Database promotion without lost commits still requires the database
operator's fencing and verification; use the restored-state procedure when
commit loss is possible.
Keep encrypted riAuth backups, the distinct backup key, PostgreSQL backups,
TLS/CA material, configuration and external secrets in the recovery plan; the
[disaster recovery runbook](disaster-recovery.md) lists them and the restore order.
