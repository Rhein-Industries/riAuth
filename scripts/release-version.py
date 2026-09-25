#!/usr/bin/env python3
"""Reject ambiguous/mismatched release tags before compiling or publishing."""
import pathlib, re, sys, tomllib
version = tomllib.loads(pathlib.Path('Cargo.toml').read_text())['package']['version']
if len(sys.argv) != 2 or not re.fullmatch(r'v[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?', sys.argv[1]) or sys.argv[1] != 'v'+version:
    raise SystemExit('Release tag must equal v' + version)
print('Validated release version:', version)
