#!/usr/bin/env bash
# Loopback PAP against an operator-supplied FreeRADIUS radclient.
# Does not start a system RADIUS service, listen on a public address, or download FreeRADIUS.
set -euo pipefail
riauth_cargo="${CARGO:-cargo}"
if [[ -n "${RADCLIENT:-}" ]]; then
  riauth_radclient="$RADCLIENT"
elif command -v radclient >/dev/null 2>&1; then
  riauth_radclient="$(command -v radclient)"
else
  echo "radclient is required; set RADCLIENT to a FreeRADIUS radclient binary" >&2
  exit 1
fi
if [[ ! -x "$riauth_radclient" ]]; then
  echo "radclient is not executable: $riauth_radclient" >&2
  exit 1
fi
echo "radclient: $("$riauth_radclient" -v)"
RIAUTH_TEST_RADCLIENT="$riauth_radclient" "$riauth_cargo" test --locked --test radius_peer -- --ignored --nocapture
