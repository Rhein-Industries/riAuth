#!/usr/bin/env bash
# Loopback GNU Lasso service-provider helper against riAuth's SAML IdP.
# Compiles scripts/lasso-saml-sp.c when pkg-config finds lasso. Does not start
# shibd, slapd, radiusd, or download Lasso. The helper is a separate program.
set -euo pipefail
riauth_root="$(cd "$(dirname "$0")/.." && pwd)"
riauth_cargo="${CARGO:-cargo}"
riauth_cc="${CC:-cc}"
if ! command -v pkg-config >/dev/null 2>&1 || ! pkg-config --exists lasso; then
  echo "pkg-config lasso is required; install GNU Lasso and set PKG_CONFIG_PATH" >&2
  exit 1
fi
pkg-config --modversion lasso
if command -v xmlsec1 >/dev/null 2>&1; then
  xmlsec1 --version
fi
riauth_out="${RIAUTH_LASSO_SP_BIN:-/tmp/riauth-i04-saml-sp/lasso-saml-sp}"
mkdir -p "$(dirname "$riauth_out")"
# Word splitting is the pkg-config argument list.
# shellcheck disable=SC2046
"$riauth_cc" -O2 -o "$riauth_out" "$riauth_root/scripts/lasso-saml-sp.c" $(pkg-config --cflags --libs lasso gobject-2.0)
chmod 755 "$riauth_out"
RIAUTH_TEST_LASSO_SP="$riauth_out" "$riauth_cargo" test --locked --test saml_sp_peer -- --ignored --nocapture
