# Docker images

Version `0.1.2` is publicly available from these repositories, each with native
Linux `amd64` and `arm64` images. No registry login is required:

```text
ghcr.io/rhein-industries/riauth-essentials:0.1.2
ghcr.io/rhein-industries/riauth-platform:0.1.2
```

## Published manifests

| Edition | Multiarch manifest digest |
| --- | --- |
| Essentials | `sha256:bad89630e5d99c82a57c35e2e3c859f92bea05b968f5b8905d9ba499c89e8cb3` |
| Platform | `sha256:2074842ad67a84653e5e4ba18fe2c445b3787a9e1844d3ccb9d3ce6c54143bb9` |

These images were built from commit
`04eb0497c00ee198ec7ba616cdf9c77cd0e5c0c5`, which passed
[full Public CI](https://github.com/Rhein-Industries/riAuth/actions/runs/37232945709).
The [publication run](https://github.com/Rhein-Industries/riAuth/actions/runs/37236213488)
built and exercised both editions on native Linux AMD64 and ARM64 hosts, including
encrypted initialization, readiness, readonly themes and persistent restart.
Both packages were made public, and anonymous registry downloads verified the
manifest, config and every layer hash for all four images. The download check
used the OCI registry protocol; container execution was verified on the hosted
Linux runners.

Use these immutable manifest digests when deploying. Publication is separate
from the draft GitHub release workflow. No `latest` tag is created or changed
by this workflow.

## Start a single instance

Use a source checkout matching the published revision, Docker Engine and Docker
Compose on a Linux host. The image runs as UID/GID `10001:10001`. It includes the
server CLI, embedded browser frontend, MIT license and third-party notices.
Choose the edition before initialization; this example uses Essentials:

```sh
export RIAUTH_IMAGE=ghcr.io/rhein-industries/riauth-essentials:0.1.2
# For Platform instead:
# export RIAUTH_IMAGE=ghcr.io/rhein-industries/riauth-platform:0.1.2
docker pull "$RIAUTH_IMAGE"
```

For a pinned deployment, replace the tag with the corresponding
`ghcr.io/rhein-industries/riauth-essentials@sha256:<published-manifest-digest>`
or Platform equivalent. The manifest selects the native architecture.

From the checkout root, prepare the bind directories. The config directory is
private and writable by the container user only during initialization. The
theme contains public frontend files and may remain empty:

```sh
mkdir -p deployment-private/docker
sudo install -d -o 10001 -g 10001 -m 0700 deployment-private/docker/config
mkdir -p deployment-private/docker/theme
chmod 0755 deployment-private/docker/theme
```

Generate the database encryption key, then initialize once. `init` prompts for
the administrator password and its confirmation; the password is not a command
argument, environment variable or Compose value:

```sh
docker compose -f deploy/compose-docker.yml run --rm init \
  keygen --out /config/database.key
docker compose -f deploy/compose-docker.yml run --rm init \
  init --issuer http://localhost:9000 --listen 127.0.0.1:9000 \
  --data-dir /data --database-key-file /config/database.key
docker compose -f deploy/compose-docker.yml up -d riauth
curl --fail --max-time 5 http://localhost:9000/readyz
```

Open `http://localhost:9000/apps`. This Linux example uses host networking and
the configured listener binds **only on host loopback**. An HTTP issuer requires
that loopback listener; do not change it to `0.0.0.0`. `/readyz` checks storage
and required local capacity. It does not verify a proxy, another node or an
external protocol peer. `init` refuses existing config or initialized state;
do not delete those protections to make an initialization retry succeed.

For a different local port, set `RIAUTH_PORT` and use the matching issuer during
initialization, including `--listen 127.0.0.1:<port>`. For a public service, initialize with your final HTTPS issuer,
such as `https://login.example.org`, and configure a host TLS reverse proxy to
the loopback port; see [operations](operations.md) and [proxy SSO](proxy.md).
The issuer is instance identity and must remain fixed after initialization.
The loopback listener and readonly config do not configure TLS for you.
This host-network example targets Linux. On other engines, verify network support,
UID mappings and bind permissions independently;
the hosted native-Linux smoke is not Docker Desktop or Windows execution.

The named `riauth-data` volume persists the encrypted redb database across
container replacement. `/config` is readonly while serving and includes the
encryption key; `/data` stays writable. Do not use `docker compose down --volumes`
to update an existing instance. Keep the key and database backups together in
your authorized recovery process, outside Git and outside the theme directory.
One redb store has one process owner; this example is not a distributed or HA
installation. For those responsibilities see [availability](availability.md)
and [deployment examples](deployment-examples.md).

## Customize the frontend

Both editions use the same [frontend theme contracts](frontend-themes.md).
Copy the starter from the matching source checkout:

```sh
cp -R examples/frontend-theme/. deployment-private/docker/theme/
```

Using an authorized editor, append this table after top-level config keys in
`deployment-private/docker/config/riauth.toml` (retain its private owner/mode):

```toml
[frontend]
theme_dir = "/theme"
```

Keep theme files readable by UID 10001, restrict writes to the operator and
restart the service. Compose mounts `/theme` readonly. The server snapshots the
theme before opening storage; requests never read the directory. Missing
overrides keep embedded defaults. Custom scripts are trusted deployment code;
the existing CSP and backend guards still apply.

```sh
docker compose -f deploy/compose-docker.yml restart riauth
```

## Update, restore and rollback

Stop the service before offline maintenance or restore, make an encrypted
backup and preserve the original config/key. Select the new image by digest,
then start with the same volume and config. Review [operations](operations.md),
[migration](migration.md) and the compatibility rules in [operations](operations.md) before
changing version, edition, backend or issuer. Existing old-format shared policy
requires the explicit stopped-node agreement procedure with both confirmations;
there is no automatic adoption or policy overwrite in this deployment. Binaries supporting only agreement formats 1 or 2 refuse the current format;
selecting an older image is not a database rollback. Use a separately verified backup/restore path with compatible inputs.

## Maintainer publication procedure

After pushing the reviewed `0.1.2` source, identify **Public CI** for that exact
full commit on a push or manual CI run. A PR run is refused because its checkout
may test a synthetic merge. Manually run **Publish checked Docker images** with
`source_sha` set to that 40-character commit and `checked_ci_run_id` set to that
run ID. Native builds may run while CI is queued or active. A completed failed
CI run is refused. The publisher waits within its existing finite budget for
that exact run to finish successfully, revalidating its source and workflow
identity before any image load, registry login or write. The workflow revision
and product revision are recorded separately. Both manifests and all three applicable
lockfile package entries must match `0.1.2`.

Each native hosted job builds both editions from the pinned Dockerfile, with
one Cargo job and incremental/release debug disabled. A fresh owned fixture
checks image and executable architecture/version/edition, nonroot User and
licenses; initializes encrypted state; checks readiness and readonly theme
responses; and replaces the container while retaining its administrator
session and signing keys in the same volumes. It removes its owned containers
and volumes before declaring the smoke successful. These are small startup and
persistence checks, not a complete protocol suite or a deployment HA claim.
The CI fixture uses isolated bridge networking, a loopback-published random
port and a reserved `.invalid` HTTPS issuer identity; it does not contact that
hostname or claim an executed TLS reverse proxy.

Build jobs have readonly repository permissions and no registry credentials.
Only the final job receives `packages: write`, after both native jobs succeed.
It checks artifact hashes and provenance, loads those tested archives, refuses
any existing version/architecture tag, pushes `0.1.2-amd64` and `0.1.2-arm64`,
then creates and verifies each two-architecture `0.1.2` manifest. Remote config
digests bind the exact tested image metadata, including OCI revision/source
labels. An authenticated publication is not anonymous-pull evidence.

The run retains tested archives, checksums, fixed smoke/build receipts and
immutable post-push manifest digests. Failure can leave a partial registry
publication; inspect its receipt rather than rerunning to overwrite tags.
The single version concurrency group serializes this workflow, but registry
tag writes are not an atomic multi-repository transaction and outside writers
must not race the publication. Set both packages public through the authorized
organization settings, verify anonymous digest pulls for both architectures,
then share the pull names and digests. The workflow never publishes a draft
GitHub release, mutates production storage or changes historical release gates.
