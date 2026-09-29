#!/usr/bin/env bash
# Local contention characterization: a modest release-build workload that prints
# per-phase storage, preparation and admission measurements as JSON observations.
# --postgres starts one disposable loopback PostgreSQL node and never touches an existing cluster.
set -euo pipefail
riauth_postgres=0
riauth_out=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --postgres) riauth_postgres=1 ;;
    --out) riauth_out="$2"; shift ;;
    *) echo "usage: $0 [--postgres] [--out FILE]" >&2; exit 2 ;;
  esac
  shift
done
riauth_cargo="${CARGO:-cargo}"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
riauth_tests=()
if [[ "$riauth_postgres" = 1 ]]; then
  riauth_pg_bin="${PG_BIN:-$(dirname "$(command -v initdb)")}"
  riauth_pg_test="$(mktemp -d "${TMPDIR:-/tmp}/riauth-pg-contention.XXXXXXXX")"
  cleanup() {
    "$riauth_pg_bin/pg_ctl" -D "$riauth_pg_test/primary" stop -m immediate -w >/dev/null 2>&1 || true
    rm -rf "$riauth_pg_test"
  }
  trap cleanup EXIT
  riauth_pg_port="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1])')"
  "$riauth_pg_bin/initdb" -D "$riauth_pg_test/primary" -U riauth_test --auth=trust --encoding=UTF8 --no-locale >"$riauth_pg_test/init.log"
  cat >>"$riauth_pg_test/primary/postgresql.conf" <<CONF
listen_addresses = '127.0.0.1'
port = $riauth_pg_port
unix_socket_directories = '$riauth_pg_test'
CONF
  "$riauth_pg_bin/pg_ctl" -D "$riauth_pg_test/primary" -l "$riauth_pg_test/primary.log" start -w >/dev/null
  printf 'host=127.0.0.1 port=%s dbname=postgres user=riauth_test sslmode=disable\n' "$riauth_pg_port" >"$riauth_pg_test/connection"
  chmod 600 "$riauth_pg_test/connection"
  export RIAUTH_TEST_PG_CONNECTION="$riauth_pg_test/connection"
  # Plain storage format first; an encrypted characterization run formats the cluster differently.
  riauth_tests+=(postgres_pool_checkout_and_advisory_lock_waits_are_measured)
fi
riauth_tests+=(characterize_contention_baseline)
if [[ -n "$riauth_out" ]]; then
  export RIAUTH_CHARACTERIZE_OUT="$riauth_out"
fi
for riauth_test in "${riauth_tests[@]}"; do
  "$riauth_cargo" test --release --locked --test contention -- --ignored --exact --nocapture "$riauth_test"
done
