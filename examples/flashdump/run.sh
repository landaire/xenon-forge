#!/usr/bin/env bash
# Build flashdump, load it on a devkit with XBDM dbgextld and fetch the raw
# NAND dump and its summary.
#
#   examples/flashdump/run.sh [OUTDIR]
#
# OUTDIR defaults to /tmp/flashdump and receives flashraw.bin, flashraw.txt
# and debug.log. Environment: XEEDEE (client binary), XBOX (console address),
# TIMEOUT (seconds to wait for flashraw.done, default 3600), JOBS (Buck jobs,
# default 8), START_BLOCK (first block to read, default 0; flashraw.bin then
# starts at that block and is appended to an earlier dump on the host). The
# module stays loaded until the console reboots, so reboot before running it
# again. The console session holds /var/tmp/xbox360-console.lock (flock),
# which other users of the devkit share.
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
XB=${XEEDEE:-$HOME/dev/xeedee/target/debug/xeedee}
# every console command has a deadline, so a hung console cannot hold the lock
X="timeout 30 $XB"
# the dump is hundreds of MB
XGET="timeout 3600 $XB"
H=${XBOX:-192.168.1.27}
OUT=${1:-/tmp/flashdump}
mkdir -p "$OUT"
cd "$ROOT"

XEX=$(bin/buck2 build -j "${JOBS:-8}" //examples/flashdump:flashdump --show-simple-output)

exec 9>/var/tmp/xbox360-console.lock
flock 9

$X -H "$H" file mkdir 'DEVKIT:\DMEXT' >/dev/null 2>&1 || true
for name in flashraw.done flashraw.txt flashraw.bin; do
  $X -H "$H" file rm "DEVKIT:\\$name" >/dev/null 2>&1 || true
done
$X -H "$H" --no-progress file put "$XEX" 'DEVKIT:\DMEXT\flashdump.xex' >/dev/null
$X -H "$H" file rm 'DEVKIT:\flashdump.start' >/dev/null 2>&1 || true
if [ -n "${START_BLOCK:-}" ]; then
  START_FILE=$(mktemp)
  echo "$START_BLOCK" >"$START_FILE"
  $X -H "$H" --no-progress file put "$START_FILE" 'DEVKIT:\flashdump.start' >/dev/null
  rm -f "$START_FILE"
fi
if $X -H "$H" file stat 'DEVKIT:\flashraw.done' >/dev/null 2>&1; then
  echo "could not remove DEVKIT:\\flashraw.done from an earlier run" >&2
  exit 1
fi

# Debug output (progress lines) goes to debug.log.
$XB -H "$H" pix-notify >"$OUT/debug.log" 2>&1 &
NOTIFY=$!
trap 'kill $NOTIFY 2>/dev/null || true' EXIT

$X -H "$H" raw 'dbgextld name=flashdump.xex'
start=$(date +%s)
until $X -H "$H" file stat 'DEVKIT:\flashraw.done' >/dev/null 2>&1; do
  if [ $(($(date +%s) - start)) -gt "${TIMEOUT:-3600}" ]; then
    echo "timed out waiting for flashraw.done" >&2
    exit 1
  fi
  sleep 10
done

rm -f "$OUT/flashraw.txt" "$OUT/flashraw.bin"
$X -H "$H" --no-progress file get -o "$OUT/flashraw.txt" 'DEVKIT:\flashraw.txt' >/dev/null
grep -v '^error ' "$OUT/flashraw.txt"
grep -q '^result ok$' "$OUT/flashraw.txt"
$XGET -H "$H" --no-progress file get -o "$OUT/flashraw.bin" 'DEVKIT:\flashraw.bin' >/dev/null
echo "dump: $OUT/flashraw.bin ($(stat -c %s "$OUT/flashraw.bin") bytes)"
