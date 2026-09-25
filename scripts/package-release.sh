#!/usr/bin/env bash
set -euo pipefail
riauth_dist="target/dist"
mkdir -p "$riauth_dist"
tar -czf "$riauth_dist/riauth-linux-x86_64.tar.gz" -C target/release riauth -C "$PWD" LICENSE THIRD_PARTY_NOTICES.md
# No registry credentials required: ship the exact image tested by this run.
docker save "$RIAUTH_CI_IMAGE" | gzip > "$riauth_dist/riauth-linux-x86_64.docker.tar.gz"
python3 - <<'PY'
import hashlib, json, os, pathlib, platform, subprocess
root = pathlib.Path('target/dist')
os_release = platform.freedesktop_os_release()
provenance = {'schema': 'riauth.build/v1', 'commit': os.environ['RIAUTH_COMMIT'],
              'repository': os.environ['GITHUB_REPOSITORY'], 'run_id': os.environ['GITHUB_RUN_ID'],
              'run_attempt': os.environ['GITHUB_RUN_ATTEMPT'],
              'build_os': {'name': platform.system(), 'architecture': platform.machine(),
                           'distribution': os_release.get('ID'), 'version': os_release.get('VERSION_ID')},
              'docker_image_id': subprocess.check_output(['docker','image','inspect','--format','{{.Id}}',os.environ['RIAUTH_CI_IMAGE']],text=True).strip(),
              'rustc': subprocess.check_output(['rustc','--version'], text=True).strip(),
              'cargo_lock_sha256': hashlib.sha256(pathlib.Path('Cargo.lock').read_bytes()).hexdigest()}
(root/'build-provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')
files=sorted(p for p in root.iterdir() if p.name!='SHA256SUMS')
with (root/'SHA256SUMS').open('w') as output:
    for path in files:
        with path.open('rb') as source: checksum=hashlib.file_digest(source,'sha256').hexdigest()
        output.write(f'{checksum}  {path.name}\n')
PY
