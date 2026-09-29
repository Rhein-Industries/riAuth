#!/usr/bin/env bash
# Shared contracts in new databases in a new loopback-only cluster.
# An optional first argument filters test names (default: all PostgreSQL contracts).
set -euo pipefail
riauth_repo="$(cd "$(dirname "$0")/.." && pwd)"
riauth_pg_bin="${PG_BIN:-$(dirname "$(command -v initdb)")}"
riauth_cargo="${CARGO:-cargo}"
riauth_filter="${1:-postgres}"
mkdir -p "$riauth_repo/target"
riauth_pg_test="$(mktemp -d "$riauth_repo/target/contract-postgres.XXXXXXXX")"
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
unix_socket_directories = ''
CONF
"$riauth_pg_bin/pg_ctl" -D "$riauth_pg_test/primary" -l "$riauth_pg_test/postgres.log" start -w >/dev/null
printf 'host=127.0.0.1 port=%s dbname=postgres user=riauth_test sslmode=disable\n' "$riauth_pg_port" >"$riauth_pg_test/connection"
chmod 600 "$riauth_pg_test/connection"
printf 'riauth disposable contract cluster\n' >"$riauth_pg_test/marker"
cd "$riauth_repo"
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$riauth_repo/target}" \
  RIAUTH_TEST_CONTRACT_PG_ROOT="$riauth_pg_test" \
  "$riauth_cargo" test --locked --features test-support --test contracts -- --ignored "$riauth_filter" --test-threads=2 --nocapture
