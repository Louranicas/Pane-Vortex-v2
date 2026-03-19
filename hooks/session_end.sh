#!/bin/bash
# Stop hook: mark complete, kill listener, deregister
set -euo pipefail

VORTEX_URL="${PANE_VORTEX_URL:-http://localhost:8132}"
PANE_ID="${PANE_VORTEX_ID:-$(hostname -s):$$}"
[[ ! "$PANE_ID" =~ ^[a-zA-Z0-9_.:-]{1,128}$ ]] && exit 0
curl -sf --max-time 2 "${VORTEX_URL}/health" >/dev/null 2>&1 || exit 0

curl -sf --max-time 2 -X POST "${VORTEX_URL}/sphere/${PANE_ID}/status" \
    -H "Content-Type: application/json" -d '{"status":"complete"}' >/dev/null 2>&1 || true

# Kill persistent listener
SAFE_ID="${PANE_ID//[^a-zA-Z0-9]/_}"
PID_FILE="/tmp/pane-vortex-listener-${SAFE_ID}.pid"
[ -f "$PID_FILE" ] && kill "$(cat "$PID_FILE")" 2>/dev/null; rm -f "$PID_FILE"

curl -sf --max-time 2 -X POST "${VORTEX_URL}/sphere/${PANE_ID}/deregister" >/dev/null 2>&1 || true
