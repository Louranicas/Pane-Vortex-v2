#!/bin/bash
# SessionStart hook: register sphere, connect IPC bus, start event listener
# Mirrors PV v1 hooks/session_start.sh with V3 improvements
# Install: add to .claude/settings.json hooks.SessionStart
set -euo pipefail

VORTEX_URL="${PANE_VORTEX_URL:-http://localhost:8132}"
PANE_ID="${PANE_VORTEX_ID:-$(hostname -s):$$}"
# C4/M06: Validate PANE_ID (security)
if [[ ! "$PANE_ID" =~ ^[a-zA-Z0-9_.:-]{1,128}$ ]]; then
    exit 1
fi

# V3.3: Check consent declaration before registering
# (stub — will check /sphere/{id}/consent when governance is implemented)

# Register sphere with retry + exponential backoff
MAX_RETRIES=3; RETRY_DELAY=2
for attempt in $(seq 1 "$MAX_RETRIES"); do
    curl -sf --max-time 2 "${VORTEX_URL}/health" >/dev/null 2>&1 && break
    (( attempt < MAX_RETRIES )) && sleep "$RETRY_DELAY" && RETRY_DELAY=$(( RETRY_DELAY * 2 ))
done

curl -sf --max-time 5 -X POST "${VORTEX_URL}/sphere/${PANE_ID}/register" \
    -H "Content-Type: application/json" \
    -d "$(jq -nc --arg persona "${PANE_VORTEX_PERSONA:-general}" --arg freq "${PANE_VORTEX_FREQUENCY:-0.1}" \
        '{persona: $persona, frequency: ($freq | tonumber)}')" \
    >/dev/null 2>&1 || true

# IPC bus persistent listener (with rotation)
if command -v pane-vortex-client >/dev/null 2>&1; then
    SAFE_ID="${PANE_ID//[^a-zA-Z0-9]/_}"
    EVENT_FILE="/tmp/pane-vortex-events-${SAFE_ID}.ndjson"
    # Rotate if >1MB
    [ -f "$EVENT_FILE" ] && [ "$(stat -c%s "$EVENT_FILE" 2>/dev/null || echo 0)" -gt 1048576 ] && \
        tail -n 1000 "$EVENT_FILE" > "${EVENT_FILE}.tmp" && mv -f "${EVENT_FILE}.tmp" "$EVENT_FILE"
    PANE_VORTEX_ID="$PANE_ID" pane-vortex-client subscribe '*' >> "$EVENT_FILE" 2>/dev/null &
    echo "$!" > "/tmp/pane-vortex-listener-${SAFE_ID}.pid"
fi
