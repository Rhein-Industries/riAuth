# Disaster recovery runbook

This runbook covers rebuilding an Essentials or Platform instance after its host,
volume or database is lost or damaged. It lists what must be kept **outside** the
authenticated backup archive, the order of a restore, the checks to run before
traffic returns, and the losses that no procedure can repair.

It builds on three guides. [Encrypted backup and restore](operations.md#encrypted-backup-and-restore)
covers taking and verifying archives. [Restored-state recovery](recovery.md)
defines the invalidation policy and the serving gate that every restore applies.
[PostgreSQL and availability](availability.md) covers database-native backups and
failover. This runbook adds no new commands or behaviour. Every command below exists
in the current `riauth` or `riauth-maintenance` build.
[Operational recovery decisions](operational-recovery.md) chooses which procedure
to run, lists the preflight and the failure stops, and separates the recorded
local drills from PostgreSQL PITR and peer recovery.
[Connector dependency incidents](connector-incidents.md) covers LDAP, outbound
SCIM, Workspace, Entra, SMTP, Vault Transit, and alert-webhook failures while
the store is still serving.

## What the archive holds

`riauth backup` writes a `riauth.backup/v3` stream. The legacy
`POST /api/operations/backup` endpoint writes `riauth.backup/v2`. Both are
authenticated with AES-256-GCM under the backup key and contain:

- **Every stored record** from one consistent snapshot, re-encrypted under the backup
  key. This includes users, credentials, clients, local signing private keys,
  upstream source secrets, agent credentials, audit and queued work.
- **The parsed configuration** of the running server, not the original TOML text.
  Comments and layout are lost, defaults are filled in, and
  `database_key_file` is removed before the configuration is written.

The archive does **not** contain:

- the backup key or the database key;
- the contents of any file that the configuration references (TLS keys, secret files,
  CA bundles, the PostgreSQL connection file). The archive holds only their paths;
- anything held by an external service: Vault Transit keys, the PostgreSQL server
  and its own backups, DNS, the TLS proxy, SMTP, upstream identity providers and
  downstream SCIM or cloud directories.

The archive records each path as the server resolved it at startup. A relative
path in `riauth.toml` is joined to the directory of the `--config` path. The shipped
[systemd unit](../deploy/riauth.service) and [Dockerfile](../Dockerfile) pass an
absolute `--config`, so their references are stored as **absolute paths on the
original host**. A path stays relative only if the server was started with a
relative `--config`, such as the default `riauth.toml`.

## Keep outside the archive

Keep these in a recovery secret store or record that does not depend on the
riAuth host, its volumes or its database. Review the list after any configuration
change.

### Keys and binaries (both editions)

| Item | Why it is needed | If it is lost |
| --- | --- | --- |
| **Backup key** (`riauth keygen` output, 32 random bytes as base64url) | Decrypts every archive made with it. Restore requires it as `--key-file`. | Every archive made with that key is unreadable. There is no passphrase, escrow or recovery path. See [Unrecoverable cases](#unrecoverable-cases). |
| **Database key** (`database_key_file`), when storage is encrypted | Opens the live store, a copied redb file and any PostgreSQL backup, dump or PITR of an encrypted store. **Not** needed to restore an archive: restore re-encrypts records under the key given as `--database-key-file`. | The live store and every database-native copy of it are unreadable. Only an archive and its backup key can rebuild the instance. |
| **A copy of `riauth.toml`** | Lists the referenced paths and settings you must provision *before* restoring. The archive's copy can be read only by restoring it. It omits `database_key_file` and comments. | Recoverable from the archive after restore, but some Platform files must exist before restore can finish. See [Restore order](#restore-order). |
| **The exact server and maintenance artifacts**: edition, release version and checksums | Restore and recovery commands must run with the deployment's edition, at the same release that made the archive. See [Choose the binary](#choose-the-binary). | Rebuild the same edition and revision from source, or download the release again. |

Make the backup key and the database key **different** keys. riAuth does not enforce
this. Both key files must be owner-only (mode 0600 or 0400), at most 128 bytes,
and must decode to 32 bytes. `riauth backup` sends the backup key to the running
server in its request body, and the server performs the encryption. Use the HTTPS
issuer for remote backups, and treat the serving host as able to read the key while
a backup runs.

### PostgreSQL deployments (both editions)

| Item | Notes |
| --- | --- |
| PostgreSQL connection file (`[postgres].connection_file`) | A libpq connection string that includes the database password. It must be an owner-only file of at most 16 KiB, with 1–8 hosts. |
| PostgreSQL CA (`[postgres].ca_file`) and the `postgres.json` given to `init`, `restore` or `migrate-postgres` | Relative paths in the JSON resolve beside it. |
| A dedicated database role and an empty database to restore into | Archive restore refuses a target that already holds records. |
| The database operator's own backups, WAL and PITR configuration | riAuth does not operate them. Restoring them follows the [database-native procedure](recovery.md#postgresql-responsibilities-and-boundary). They are encrypted with the database key when storage encryption is on. |

### Files referenced by `riauth.toml`

Save the **contents** of every file your configuration references. Provision them
at the same paths on the recovery host, or edit the paths in the restored
`riauth.toml`. Every credential is a `*_file` path; riAuth accepts no inline secrets
in `riauth.toml`.

Both editions:

| Configuration key | Contents |
| --- | --- |
| `tls_cert_file`, `tls_key_file` | Native HTTPS certificate chain and private key |
| `[mail].password_file` | SMTP password, required when `mail.username` is set. It is checked when `serve` starts. |
| `[alert_webhook].bearer_file` | Alert webhook bearer token |
| `[directories.<id>].password_file`, `ca_file` | LDAP bind password and upstream CA |
| `[scim_targets.<id>].token_file`, `ca_file` | Outbound SCIM bearer token and CA |
| `[scim_targets.<id>.oauth].client_secret_file`, `refresh_token_file`, `ca_file` | Outbound SCIM OAuth credentials |
| `[reconciliation_controllers."<kind>/<id>"].credential_file` | A `ri_agent_…` token issued **by this instance**. After a restore it may have been rotated or revoked; see [After serving resumes](#after-serving-resumes). |

Platform only. Essentials rejects all of these settings:

| Configuration key | Contents |
| --- | --- |
| `[signers.<name>].token_file`, `ca_file` | Vault token and Vault CA for a Transit signer; see [external signing](kms.md) |
| `[client_certificates].trust_anchors_file`, `crl_file` | HTTPS client-certificate trust anchors and CRLs |
| `[device_trust].pem_file` or `jwks_file` | Device-trust verification key. **Read during restore**; see [Restore order](#restore-order). |
| `[radius_listeners.<id>]`: `tls_cert_file`, `tls_key_file`, `client_ca_file`; `nas.<nas>.shared_secret_file`; `eap_tls.certificate_file`, `key_file`, `client_ca_file`, `client_crl_file`, `ocsp_response_file` | RadSec, RADIUS and EAP-TLS material. **Read during restore.** A stale CRL stops the listener. |
| `[ldap_listeners.<id>].tls_cert_file`, `tls_key_file` | LDAP provider listener certificate and key |
| `[proxy_listeners.<id>].tls_cert_file`, `tls_key_file`; `routes."<origin>".ca_file` | Embedded reverse-proxy TLS and upstream CAs |
| `[workspace_directories.<id>].client_secret_file`; `direct_auth.key_file` | Google Workspace broker secret or service-account key |
| `[entra_directories.<id>].client_secret_file`, or `certificate_file` + `private_key_file` | Microsoft Entra application credentials |

### External dependencies

Record how to reach and, if needed, rebuild each of these. None of them is in the archive.

- **Issuer, DNS and TLS front end.** The restored instance must serve the same issuer
  URL. Keep the proxy or load-balancer configuration and its certificate, for example
  the [Caddy](../deploy/Caddyfile) or [HAProxy](../deploy/haproxy-riauth.cfg.example)
  examples.
- **Vault Transit (Platform).** You need the Transit key, the exact bound key
  version, and a token that can sign with it. The database stores only the signer
  name, key version and pinned public key. Restore never contacts Vault. A missing or
  mismatched signer lets the server start, but every token issuance fails with
  `signer_unavailable`.
- **SMTP server, upstream identity providers, SCIM targets, and Workspace or Entra
  tenants.** Keep the application registrations and redirect URIs that match this
  issuer.
- **Monitoring.** The Prometheus scrape token is an `operations.read` credential
  issued by riAuth and stored on the monitoring side (see
  [prometheus.yml](../deploy/prometheus.yml)). Restored credential state may not
  match it.
- **Windows device hosts (Platform).** Device secrets live on each device. The server
  cannot return a lost secret; see [Windows device recovery](../windows/RECOVERY.md).
- **Your audit or SIEM export and incident records.** You need them to reconcile
  credentials that changed after the snapshot; see [serving gate](recovery.md#serving-gate).

## Choose the binary

Opening a store records the opening binary's edition and compiled capabilities in
`meta/edition_provenance` and `meta/version_activation`. This includes the reopen
that `restore` performs before it publishes `riauth.toml`. Run every command that
opens the store with the **deployment's own edition and release**.

- **Essentials.** Use the Essentials `riauth` binary or the released Essentials
  `riauth-maintenance` archive for `restore`, `recover-admin` and `init`. The
  `recovery` subcommands exist only in `riauth`. A Platform binary records the
  store as Platform, and the Essentials server then refuses it. If this happens
  during a restore, discard that output and restore again: restore always writes
  a new target. `keygen` does not open a store and is safe with either edition.
- **Platform.** Use the Platform `riauth` binary or the released Platform
  `riauth-maintenance` archive from the same release. The `recovery` subcommands
  exist only in `riauth`.
- **Crossing editions.** An Essentials binary cannot restore a Platform archive. Its
  configuration check rejects Platform settings, and the reopen refuses Platform
  provenance or retained Platform records. To move a Platform deployment to
  Essentials, restore it as Platform, complete the serving gate, then follow the
  explicit handoff in [server editions](editions.md). A Platform binary accepts an
  Essentials archive, but records the store as Platform.
- **Release.** A binary older than the release that last activated the store refuses
  it after import. A newer binary migrates the store and records its own release, so
  older releases can no longer open it. Restore with the release that made the
  archive, then upgrade separately under [upgrade and rollback](operations.md#upgrade-and-rollback).
- **Archive format.** `riauth.backup/v3` needs a v3-capable reader. The v0.1.1 release
  reads v1 and v2 only, and shares its version string with later unreleased builds.
  Record which build wrote each archive, and rehearse restore with that exact binary.

## Restore order

Before you start, make sure no other riAuth process can serve under the issuer or
write to the target. Never run two servers against one redb file. For PostgreSQL,
stop every node and fence a former database primary; see [availability](availability.md).

### A. Rebuild from an archive (redb or empty PostgreSQL)

1. **Install the binary.** Verify its checksum and edition with
   `riauth --json capabilities`.
2. **Provision keys.** Place the backup key and a database key as owner-only files.
   You may generate a new database key with `riauth keygen --out <path>`: restore
   re-encrypts the records. For a PostgreSQL target, also provision the connection
   file, the CA, the `postgres.json` and an empty dedicated database.
3. **Provision referenced files.** Place the saved files at the paths in your saved
   `riauth.toml` (see [Keep outside the archive](#keep-outside-the-archive)). On
   Platform, restore itself reads the `device_trust` key and all RADIUS listener
   material. If any is missing, restore fails, either as
   `Invalid backup configuration` or when the restored store is reopened. A relative
   `device_trust` path is checked from the working directory of the restore command.
   Files read only at `serve` or at use time (TLS, mail, signer, directory and SCIM
   credentials) can wait until step 6, but must exist before step 8.
4. **Restore into a new output directory** whose parent exists:

   ```sh
   riauth restore --backup backup.riauth --key-file backup.key \
     --out restored --database-key-file database.key
   # PostgreSQL target instead of redb:
   #   --postgres-config deployment-private/postgres.json
   ```

   Always pass `--database-key-file` if the store should be encrypted. Without it the
   restored store is **plaintext**, whatever the source used. Restore authenticates
   the whole archive before it creates `--out`. It refuses an existing directory or
   a non-empty PostgreSQL target.
5. **Check the output.** The output must show `"verified": true`, the expected
   `storage` (`redb` or `postgresql`), `"encrypted_at_rest": true` if intended, and
   `"serving_allowed": false`. Keep the `recovery` object: `recovery.id`, the
   per-collection removals, `reconcile` and any `unclassified` collections. If
   `restored/riauth.toml` is missing and only `.riauth.restore-pending.toml` exists,
   the reopen failed. Discard the directory, and recreate the PostgreSQL database,
   before retrying.
6. **Review the restored `riauth.toml`.** Restore rewrites only three fields:
   - `data_dir = "data"`, beside the restored file;
   - `database_key_file`, set to the absolute path you passed;
   - `[postgres]`, set from `--postgres-config` or removed.

   Every other path is copied exactly as recorded. Compare them with your saved
   copy. Then provision the files at those paths, or edit the paths. Adjust `listen`
   and `trusted_proxies` if the new host's addresses differ. Keep the issuer
   unchanged.
7. **Inspect and reconcile offline.** Run the read-only check with the edition's
   binary:

   ```sh
   riauth --config restored/riauth.toml recovery status
   ```

   Reconcile every listed credential class against your audit export or incident
   record, as described in [serving gate](recovery.md#serving-gate). Review any
   `unclassified` collection. Rotate a signing key that was retired for cause
   after the snapshot. If administrator credentials are also lost, run
   `riauth --config restored/riauth.toml recover-admin <name> --password-stdin`
   now. It works while the gate is closed; see [diagnostics and recovery](operations.md#diagnostics-and-recovery).
8. **Reopen the gate** with the reviewed ID:

   ```sh
   riauth --config restored/riauth.toml recovery complete \
     --recovery-id <recovery.id> --persistent-credentials-reconciled
   ```

   Do this within seven days of restore, so that queued back-channel logouts are
   still delivered.
9. **Start one process, not yet public.** Run `riauth --config restored/riauth.toml serve`
   and run the [validation checks](#validation-checks) before you move DNS or the
   load balancer.
10. **PostgreSQL with several nodes.** Copy the restored `riauth.toml`, the database
    key and every referenced file to each node at the **same absolute paths**:
    restore writes the database key and PostgreSQL paths as absolute paths from the
    restoring host. Compare checksums before starting the other nodes. Complete the
    gate only once, because it is stored in the shared database.

The [Compose examples](deployment-examples.md) use a fixed `/config` and `/data`
layout. A restored redb directory must be placed and mounted to match it, and the
data volume must stay writable. That arrangement is not a tested restore target.
Rehearse it before you depend on it.

### B. Database-native recovery (PostgreSQL PITR, base backup, dump, or a copied redb file)

Follow [PostgreSQL responsibilities and boundary](recovery.md#postgresql-responsibilities-and-boundary).
Stop every node, restore with the database's tooling, then run
`riauth recovery invalidate --database-restored` with the edition's binary before any
node serves. After that, continue at step 7 above. The live database key is
required. For the multi-host Compose example, run these commands through the
offline tool service; see [deployment examples](deployment-examples.md).

## Validation checks

Run these before directing users to the instance. Record the results with the
recovery ID.

| Check | How | What it proves |
| --- | --- | --- |
| Gate open | `riauth --config <file> recovery status` shows no pending recovery | `recovery complete` was recorded |
| Readiness | `curl --fail http://127.0.0.1:9000/readyz` on each node | Storage, schema, index and release activation match this binary |
| Edition | `riauth --json capabilities` on each node | The same edition and capabilities everywhere |
| Discovery and keys | `curl --fail https://<issuer>/.well-known/openid-configuration`, then fetch its `jwks_uri` | Issuer and published keys match what relying parties expect |
| Administrator | `riauth --server https://<issuer> login admin`, then `riauth --server https://<issuer> doctor` | Sign-in works; `doctor` reports schema, admin count, signing-key health and storage encryption |
| Token signing | Complete one real OIDC sign-in, and on Platform a SAML one, per signing domain | Signing works, including Vault Transit. Restore and `doctor` never contact Vault. |
| Mail, directories, provisioning | Send one invitation or reset mail; run one LDAP or SCIM plan; on Platform, one Workspace or Entra plan | The external secrets and endpoints are reachable |
| Listeners (Platform) | Exercise RADIUS, LDAP provider, proxy or client-certificate login, if configured | Listener material is current |
| Metrics | Scrape `/api/operations/prometheus` | The monitoring token is still valid |

A successful `restore` result (`verified`) covers only local checks: the archive is
authentic, the schema and issuer match, signing keys parse, and an enabled
administrator exists. It does not show that any external credential or service works.

## After serving resumes

- Take a new backup and verify it by restoring it into a scratch directory.
- Re-issue the reconciliation-controller agent tokens and the metrics token if they
  were rotated or revoked after the snapshot. The recovery gate lists
  `agent_credentials` for this reason.
- Expect every user to sign in again, and every application to request new grants.
  JWT access tokens and application sessions that were issued before the loss stay
  valid until they expire; see [what this does not establish](recovery.md#what-this-does-not-establish).

## Unrecoverable cases

No riAuth command can repair these:

- **Backup key lost.** Every archive encrypted with it is permanently unreadable. If
  the live store and its database key survive, generate a new backup key and take a
  new backup immediately.
- **Database key lost, for an encrypted store.** The live store, copied redb files, and
  every PostgreSQL backup, dump or PITR of that store are unreadable. Recovery is
  possible only from an archive and its backup key. If both keys are lost, the
  instance must be initialized again, and users, subjects, clients and signing keys
  cannot be preserved.
- **Changes after the snapshot.** Accounts, clients, credentials, enrollments and
  configuration written after the snapshot are gone. Revocations and rotations made
  after the snapshot are undone until you reconcile them; see
  [restored-state recovery](recovery.md).
- **Vault Transit key destroyed, or the bound version unusable.** That key can no longer
  sign. Configure a new signer version and bind it with `riauth keys bind`, or rotate
  to a locally generated key: `riauth rotate-key` for the default signing key,
  `riauth keys generate <domain>` for a named domain. Confirm the result with
  `riauth keys list`. Relying parties then need the new JWKS.
  Tokens signed by the old key are unaffected until they expire.
- **Referenced secret files lost.** riAuth cannot recover them. Re-issue them at the
  issuing service: SMTP, LDAP, SCIM, Workspace or Entra, the Vault token, TLS
  certificates. On Platform, a restore that needs device-trust or RADIUS material
  cannot finish until replacements exist.
- **Windows device secrets lost.** The server cannot return a lost device secret.
  Revoke the device and enroll it again, as described in
  [Windows device recovery](../windows/RECOVERY.md).
- **Freshness.** riAuth cannot tell whether an archive or a physical database restore
  is the newest available. Choosing the right point in time is the operator's
  decision.

## Rehearsal

Rehearse this runbook in an isolated environment that cannot serve under the
production issuer. Use the saved keys, the saved files and the recorded binary.
Record the archive size, elapsed times, the recovery ID and the validation results
with your recovery procedure, as the [capacity notes](operations.md#streamed-backup-export)
recommend. Repeat the rehearsal after changes to storage, edition, release or
referenced files.
