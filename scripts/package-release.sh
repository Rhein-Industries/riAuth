#!/usr/bin/env bash
set -euo pipefail
case "${RIAUTH_ARCH:?set RIAUTH_ARCH to x86_64 or aarch64}" in
  x86_64) target=x86_64-unknown-linux-gnu; oci_arch=amd64 ;;
  aarch64) target=aarch64-unknown-linux-gnu; oci_arch=arm64 ;;
  *) echo "Unsupported release architecture: $RIAUTH_ARCH" >&2; exit 2 ;;
esac
test "$(uname -s)" = Linux
test "$(uname -m)" = "$RIAUTH_ARCH"
: "${RIAUTH_COMMIT:?}"
: "${GITHUB_REPOSITORY:?}"
: "${GITHUB_RUN_ID:?}"
: "${GITHUB_RUN_ATTEMPT:?}"
: "${RIAUTH_ESSENTIALS_IMAGE:?}"
: "${RIAUTH_PLATFORM_IMAGE:?}"

riauth_dist=target/dist
mkdir -p "$riauth_dist"
if test -n "$(find "$riauth_dist" -mindepth 1 -maxdepth 1 -print -quit)"; then
  echo "Release output directory must be empty: $riauth_dist" >&2
  exit 1
fi
for edition in essentials platform; do
  binary="target/$edition/$target/release/riauth"
  test -x "$binary"
  tar -czf "$riauth_dist/riauth-$edition-linux-$RIAUTH_ARCH.tar.gz" -C "$(dirname "$binary")" riauth -C "$PWD" LICENSE THIRD_PARTY_NOTICES.md
done
maintenance_binary="target/platform/$target/release/riauth-maintenance"
test -x "$maintenance_binary"
tar -czf "$riauth_dist/riauth-maintenance-linux-$RIAUTH_ARCH.tar.gz" -C "$(dirname "$maintenance_binary")" riauth-maintenance -C "$PWD" LICENSE THIRD_PARTY_NOTICES.md
riauthctl_binary="crates/riauthctl/target/$target/release/riauthctl"
test -x "$riauthctl_binary"
tar -czf "$riauth_dist/riauthctl-linux-$RIAUTH_ARCH.tar.gz" -C "$(dirname "$riauthctl_binary")" riauthctl -C "$PWD" LICENSE THIRD_PARTY_NOTICES.md

for edition in essentials platform; do
  image_var="RIAUTH_${edition^^}_IMAGE"
  image="${!image_var}"
  test "$(docker image inspect --format '{{.Os}}/{{.Architecture}}' "$image")" = "linux/$oci_arch"
  test "$(docker image inspect --format '{{index .Config.Labels "org.opencontainers.image.revision"}}' "$image")" = "$RIAUTH_COMMIT"
  docker save "$image" | gzip -n > "$riauth_dist/riauth-$edition-linux-$RIAUTH_ARCH.docker.tar.gz"
done
python3 - <<'PY'
import hashlib, json, os, pathlib, platform, subprocess
root = pathlib.Path('target/dist')
arch = os.environ['RIAUTH_ARCH']
target = f'{arch}-unknown-linux-gnu'
oci_arch = {'x86_64': 'amd64', 'aarch64': 'arm64'}[arch]
os_release = platform.freedesktop_os_release()
provenance = {'schema': 'riauth.build/v3', 'commit': os.environ['RIAUTH_COMMIT'],
              'repository': os.environ['GITHUB_REPOSITORY'], 'run_id': os.environ['GITHUB_RUN_ID'],
              'run_attempt': os.environ['GITHUB_RUN_ATTEMPT'],
              'target_triple': target, 'oci_platform': f'linux/{oci_arch}',
              'build_os': {'name': platform.system(), 'architecture': platform.machine(),
                           'distribution': os_release.get('ID'), 'version': os_release.get('VERSION_ID')},
              'server_builds': {
                  edition: {'features': ['essentials'] if edition == 'essentials' else ['essentials', 'platform'],
                            'no_default_features': True,
                            'docker_image_id': subprocess.check_output(['docker','image','inspect','--format','{{.Id}}',os.environ[f'RIAUTH_{edition.upper()}_IMAGE']],text=True).strip()}
                  for edition in ('essentials', 'platform')},
              'maintenance_features': ['essentials', 'platform'],
              'rustc': subprocess.check_output(['rustc','--version'], text=True).strip(),
              'cargo_lock_sha256': hashlib.sha256(pathlib.Path('Cargo.lock').read_bytes()).hexdigest(),
              'riauthctl_cargo_lock_sha256': hashlib.sha256(pathlib.Path('crates/riauthctl/Cargo.lock').read_bytes()).hexdigest(),
              'riauthctl_features': 'no-default-features'}
(root/f'build-provenance-linux-{arch}.json').write_text(json.dumps(provenance,indent=2)+'\n')
files=sorted(p for p in root.iterdir() if p.is_file())
with (root/f'SHA256SUMS-linux-{arch}').open('w') as output:
    for path in files:
        with path.open('rb') as source: checksum=hashlib.file_digest(source,'sha256').hexdigest()
        output.write(f'{checksum}  {path.name}\n')
PY
