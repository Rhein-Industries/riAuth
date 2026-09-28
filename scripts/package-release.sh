#!/usr/bin/env bash
set -euo pipefail
riauth_dist="target/dist"
mkdir -p "$riauth_dist"
for edition in essentials platform; do
  binary="target/$edition/release/riauth"
  test -x "$binary"
  tar -czf "$riauth_dist/riauth-$edition-linux-x86_64.tar.gz" -C "$(dirname "$binary")" riauth -C "$PWD" LICENSE THIRD_PARTY_NOTICES.md
done
maintenance_binary="target/platform/release/riauth-maintenance"
test -x "$maintenance_binary"
tar -czf "$riauth_dist/riauth-maintenance-linux-x86_64.tar.gz" -C "$(dirname "$maintenance_binary")" riauth-maintenance -C "$PWD" LICENSE THIRD_PARTY_NOTICES.md
riauthctl_binary="crates/riauthctl/target/release/riauthctl"
test -x "$riauthctl_binary"
tar -czf "$riauth_dist/riauthctl-linux-x86_64.tar.gz" -C "$(dirname "$riauthctl_binary")" riauthctl -C "$PWD" LICENSE THIRD_PARTY_NOTICES.md
# No registry credentials required: ship the exact images tested by this run.
docker save "$RIAUTH_ESSENTIALS_IMAGE" | gzip > "$riauth_dist/riauth-essentials-linux-x86_64.docker.tar.gz"
docker save "$RIAUTH_PLATFORM_IMAGE" | gzip > "$riauth_dist/riauth-platform-linux-x86_64.docker.tar.gz"
python3 - <<'PY'
import hashlib, json, os, pathlib, platform, subprocess
root = pathlib.Path('target/dist')
os_release = platform.freedesktop_os_release()
provenance = {'schema': 'riauth.build/v2', 'commit': os.environ['RIAUTH_COMMIT'],
              'repository': os.environ['GITHUB_REPOSITORY'], 'run_id': os.environ['GITHUB_RUN_ID'],
              'run_attempt': os.environ['GITHUB_RUN_ATTEMPT'],
              'build_os': {'name': platform.system(), 'architecture': platform.machine(),
                           'distribution': os_release.get('ID'), 'version': os_release.get('VERSION_ID')},
              'server_builds': {
                  edition: {'features': ['essentials'] if edition == 'essentials' else ['essentials', 'platform'],
                            'no_default_features': True,
                            'docker_image_id': subprocess.check_output(['docker','image','inspect','--format','{{.Id}}',os.environ[f'RIAUTH_{edition.upper()}_IMAGE']],text=True).strip()}
                  for edition in ('essentials', 'platform')},
              'rustc': subprocess.check_output(['rustc','--version'], text=True).strip(),
              'cargo_lock_sha256': hashlib.sha256(pathlib.Path('Cargo.lock').read_bytes()).hexdigest(),
              'riauthctl_cargo_lock_sha256': hashlib.sha256(pathlib.Path('crates/riauthctl/Cargo.lock').read_bytes()).hexdigest(),
              'riauthctl_features': 'no-default-features'}
(root/'build-provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')
files=sorted(p for p in root.iterdir() if p.name!='SHA256SUMS')
with (root/'SHA256SUMS').open('w') as output:
    for path in files:
        with path.open('rb') as source: checksum=hashlib.file_digest(source,'sha256').hexdigest()
        output.write(f'{checksum}  {path.name}\n')
PY
