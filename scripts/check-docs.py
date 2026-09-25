#!/usr/bin/env python3
"""Check tracked Markdown links and accidental build directories at repository root."""
import pathlib, re, subprocess
root=pathlib.Path(__file__).resolve().parents[1]
errors=[]
files=subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard'],cwd=root,text=True).splitlines()
for name in files:
    path=root/name
    if path.suffix != '.md' or name.startswith('vendor/') or not path.is_file(): continue
    for target in re.findall(r'\[[^\]\n]*\]\(([^)\s]+)(?:\s+"[^"]*")?\)',path.read_text()):
        if '://' in target or target.startswith(('#','mailto:','/')): continue
        location=target.split('#',1)[0]
        if location and not (path.parent/location).exists(): errors.append(f'{name}: missing {location}')
for path in root.glob('target-*'):
    if path.is_dir(): errors.append(f'{path.name}: use target/ or a temporary directory for build output')
if errors: raise SystemExit('\n'.join(errors))
print('Markdown links and build-directory layout checked')
