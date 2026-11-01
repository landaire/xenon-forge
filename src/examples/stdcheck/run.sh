#!/usr/bin/env bash
# Build stdcheck, run it on a development kit with a network peer on this
# host, and print its report.
#
#   src/examples/stdcheck/run.sh [OUTDIR]
#
# OUTDIR defaults to /tmp/stdcheck and receives debug.log. Environment:
# XEEDEE (client binary), XBOX (console address), PORT (peer port, default
# 47360), TIMEOUT (seconds, default 180), JOBS (Buck jobs, default 8).
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/../../.." && pwd)
X=${XEEDEE:-$HOME/dev/xeedee/target/debug/xeedee}
H=${XBOX:-192.168.1.27}
PORT=${PORT:-47360}
OUT=${1:-/tmp/stdcheck}
DIR='DEVKIT:\stdcheck'
mkdir -p "$OUT"
OUT=$(cd "$OUT" && pwd)
cd "$ROOT"
XEX=$(bin/buck2 build -j "${JOBS:-8}" //src/examples/stdcheck:stdcheck --show-simple-output)
# The address of this host on the route to the console.
PEER=$(ip -4 route get "$H" | sed -n 's/.* src \([0-9.]*\).*/\1/p')

exec 9>/var/tmp/xbox360-console.lock
flock 9

# TCP: echo lines; on `CONNECT port`, connect back to the console and send
# `back`. UDP: echo datagrams.
python3 - "$PORT" <<'EOF' &
import socket, socketserver, sys, threading

port = int(sys.argv[1])

class Tcp(socketserver.StreamRequestHandler):
    def handle(self):
        for line in self.rfile:
            if line.startswith(b"CONNECT "):
                back = int(line.split()[1])
                with socket.create_connection((self.client_address[0], back), timeout=10) as s:
                    s.sendall(b"back\n")
                return
            self.wfile.write(line)

class Udp(socketserver.BaseRequestHandler):
    def handle(self):
        data, sock = self.request
        sock.sendto(data, self.client_address)

for server in (socketserver.ThreadingTCPServer, socketserver.UDPServer):
    server.allow_reuse_address = True
tcp = socketserver.ThreadingTCPServer(("0.0.0.0", port), Tcp)
udp = socketserver.UDPServer(("0.0.0.0", port), Udp)
threading.Thread(target=udp.serve_forever, daemon=True).start()
tcp.serve_forever()
EOF
peer=$!
"$X" -H "$H" pix-notify >"$OUT/debug.log" 2>&1 &
notify=$!
trap 'kill "$notify" "$peer" 2>/dev/null || true; wait "$notify" "$peer" 2>/dev/null || true' EXIT

"$X" -H "$H" file mkdir "$DIR" >/dev/null 2>&1 || true
"$X" -H "$H" --no-progress file put "$XEX" "$DIR\default.xex" >/dev/null
sleep 1
"$X" -H "$H" magicboot --title "$DIR\default.xex" --directory "$DIR" \
  --cmdline "$PEER:$PORT" >/dev/null
start=$(date +%s)
until grep -aq 'stdcheck: done' "$OUT/debug.log"; do
  if grep -aqE 'exception code=|ntassert|rip stop' "$OUT/debug.log" || \
     [ $(($(date +%s) - start)) -gt "${TIMEOUT:-180}" ]; then
    cat "$OUT/debug.log" >&2
    "$X" -H "$H" reboot >/dev/null || true
    exit 1
  fi
  sleep 1
done
grep -aoE '(PASS|FAIL) .*|stdcheck: .*' "$OUT/debug.log" | grep -v 'stdcheck: done'
grep -aqE 'stdcheck: [0-9]+ passed, 0 failed' "$OUT/debug.log"
