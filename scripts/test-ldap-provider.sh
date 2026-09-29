#!/usr/bin/env bash
# Loopback LDAPS and STARTTLS against riAuth's LDAP provider using an operator-supplied OpenLDAP ldapsearch.
# Does not start slapd, a system directory, or download OpenLDAP.
set -euo pipefail
riauth_cargo="${CARGO:-cargo}"
riauth_ldapsearch="${LDAPSEARCH:-}"
if [[ -z "$riauth_ldapsearch" && "$(uname -s)" == "Darwin" ]] && command -v brew >/dev/null 2>&1; then
  riauth_prefix="$(brew --prefix openldap 2>/dev/null || true)"
  if [[ -n "$riauth_prefix" && -x "$riauth_prefix/bin/ldapsearch" ]]; then
    riauth_ldapsearch="$riauth_prefix/bin/ldapsearch"
  fi
fi
if [[ -z "$riauth_ldapsearch" ]]; then
  if command -v ldapsearch >/dev/null 2>&1; then
    riauth_ldapsearch="$(command -v ldapsearch)"
  else
    echo "ldapsearch is required; set LDAPSEARCH to an OpenLDAP ldapsearch binary" >&2
    exit 1
  fi
fi
if [[ ! -x "$riauth_ldapsearch" ]]; then
  echo "ldapsearch is not executable: $riauth_ldapsearch" >&2
  exit 1
fi
"$riauth_ldapsearch" -VV
RIAUTH_TEST_LDAPSEARCH="$riauth_ldapsearch" "$riauth_cargo" test --locked --test ldap_provider_peer -- --ignored --nocapture
