#!/usr/bin/env bash
# Two disposable, loopback-only PostgreSQL nodes. Never touches an existing cluster.
set -euo pipefail
riauth_pg_bin="${PG_BIN:-$(dirname "$(command -v initdb)")}"
riauth_cargo="${CARGO:-cargo}"
riauth_pg_test="$(mktemp -d "${TMPDIR:-/tmp}/riauth-pg-test.XXXXXXXX")"
cleanup() {
  "$riauth_pg_bin/pg_ctl" -D "$riauth_pg_test/standby" stop -m immediate -w >/dev/null 2>&1 || true
  "$riauth_pg_bin/pg_ctl" -D "$riauth_pg_test/primary" stop -m immediate -w >/dev/null 2>&1 || true
  rm -rf "$riauth_pg_test"
}
trap cleanup EXIT
riauth_pg_port="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1])')"
riauth_pg_standby_port="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1])')"
if [[ "$riauth_pg_port" = "$riauth_pg_standby_port" ]]; then exit 1; fi
"$riauth_pg_bin/initdb" -D "$riauth_pg_test/primary" -U riauth_test --auth=trust --encoding=UTF8 --no-locale >"$riauth_pg_test/init.log"
cat >>"$riauth_pg_test/primary/postgresql.conf" <<CONF
listen_addresses = '127.0.0.1'
port = $riauth_pg_port
unix_socket_directories = '$riauth_pg_test'
wal_level = replica
max_wal_senders = 4
max_replication_slots = 4
CONF
"$riauth_pg_bin/pg_ctl" -D "$riauth_pg_test/primary" -l "$riauth_pg_test/primary.log" start -w >/dev/null
"$riauth_pg_bin/pg_basebackup" -h 127.0.0.1 -p "$riauth_pg_port" -U riauth_test -D "$riauth_pg_test/standby" -R -X stream
cat >>"$riauth_pg_test/standby/postgresql.conf" <<CONF
port = $riauth_pg_standby_port
synchronous_standby_names = ''
CONF
cat >>"$riauth_pg_test/standby/postgresql.auto.conf" <<CONF
primary_conninfo = 'host=127.0.0.1 port=$riauth_pg_port user=riauth_test application_name=riauth_test_standby'
CONF
"$riauth_pg_bin/pg_ctl" -D "$riauth_pg_test/standby" -l "$riauth_pg_test/standby.log" start -w >/dev/null
printf "\nsynchronous_standby_names = 'FIRST 1 (riauth_test_standby)'\n" >>"$riauth_pg_test/primary/postgresql.conf"
"$riauth_pg_bin/pg_ctl" -D "$riauth_pg_test/primary" reload >/dev/null
printf 'host=127.0.0.1,127.0.0.1 port=%s,%s dbname=postgres user=riauth_test sslmode=disable\n' "$riauth_pg_port" "$riauth_pg_standby_port" >"$riauth_pg_test/connection"
chmod 600 "$riauth_pg_test/connection"
printf 'riauth disposable integration cluster\n' >"$riauth_pg_test/marker"
RIAUTH_TEST_PG_CONNECTION="$riauth_pg_test/connection" RIAUTH_TEST_PG_ROOT="$riauth_pg_test" RIAUTH_TEST_PG_CTL="$riauth_pg_bin/pg_ctl" "$riauth_cargo" test --locked --test postgres -- --ignored --nocapture
