#!/usr/bin/env bash
# Peak RSS of the buffered v2 backup, the streamed v3 backup and v3 restore
# at several store sizes. Each step runs in its own process. The scan column
# pages through the same records without the codec: redb's read cache (1 GiB
# default) grows with the store, so v3 minus scan is a rough overhead comparison,
# not a proof of an end-to-end memory bound.
# Usage: scripts/measure-backup-memory.sh [records ...]  (default: 10000 40000 160000)
set -euo pipefail
cd "$(dirname "$0")/.."
sizes=("$@")
[[ ${#sizes[@]} -gt 0 ]] || sizes=(10000 40000 160000)

binary=$(cargo test --locked --features test-support --test backup_stream --no-run --message-format=json 2>/dev/null |
  python3 -c 'import json, sys
for line in sys.stdin:
    message = json.loads(line)
    if message.get("reason") == "compiler-artifact" and message["target"]["name"] == "backup_stream" and message.get("executable"):
        print(message["executable"])')

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# Runs one harness mode; prints "<peak MiB> <harness summary>".
step() {
  local dir=$1 mode=$2 log="$work/time.log" out="$work/step.out"
  local time_flag=-v
  [[ $(uname) == Darwin ]] && time_flag=-l
  RIAUTH_MEASURE_DIR=$dir RIAUTH_MEASURE_MODE=$mode RIAUTH_MEASURE_RECORDS=${3:-0} \
    /usr/bin/time "$time_flag" "$binary" measure_backup_memory --exact --ignored --nocapture \
    >"$out" 2>"$log"
  local peak
  if [[ $(uname) == Darwin ]]; then
    peak=$(awk '/maximum resident set size/ {printf "%.1f", $1 / 1048576}' "$log")
  else
    peak=$(awk -F: '/Maximum resident set size/ {printf "%.1f", $2 / 1024}' "$log")
  fi
  echo "$peak $(grep -o 'R01_MEASURE .*' "$out" | cut -d' ' -f2- || true)"
}

printf '%-9s %-8s %-10s %-10s %-10s %-10s %s\n' records db_MiB seed_MiB scan_MiB v2_MiB v3_MiB restore_MiB
for records in "${sizes[@]}"; do
  dir="$work/$records"
  mkdir -p "$dir"
  seed=$(step "$dir" seed "$records")
  db=$(du -m "$dir/data/riauth.redb" | cut -f1)
  scan=$(step "$dir" scan)
  v2=$(step "$dir" v2)
  v3=$(step "$dir" v3)
  restore=$(step "$dir" restore)
  printf '%-9s %-8s %-10s %-10s %-10s %-10s %s\n' "$records" "$db" "${seed%% *}" "${scan%% *}" "${v2%% *}" "${v3%% *}" "${restore%% *}"
  echo "  v2: ${v2#* }"
  echo "  v3: ${v3#* }"
done
