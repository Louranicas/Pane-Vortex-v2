#!/bin/bash
# SessionStart hook: register sphere, hydrate from POVM+RM, announce fleet status
# Phase E of fleet coordination plan. Project-scoped (GAP-G3).
set -euo pipefail

# GAP-G3: Project scope guard
PV2_DIR="/home/louranicas/claude-code-workspace/pane-vortex-v2"
[[ "$(pwd)" != "$PV2_DIR"* ]] && exit 0

VORTEX_URL="${PANE_VORTEX_URL:-http://localhost:8132}"
POVM_URL="${POVM_URL:-http://localhost:8125}"
RM_URL="${RM_URL:-http://localhost:8130}"
PANE_ID="${PANE_VORTEX_ID:-$(hostname -s):$$}"

# Validate PANE_ID
[[ ! "$PANE_ID" =~ ^[a-zA-Z0-9_.:-]{1,128}$ ]] && exit 1
SAFE_ID="${PANE_ID//[^a-zA-Z0-9]/_}"
HOOKS_DIR="$(cd "$(dirname "$0")" && pwd)"

# Health check with retry
MAX_RETRIES=3; RETRY_DELAY=2
for attempt in $(seq 1 "$MAX_RETRIES"); do
    curl -sf --max-time 2 "${VORTEX_URL}/health" >/dev/null 2>&1 && break
    (( attempt < MAX_RETRIES )) && sleep "$RETRY_DELAY" && RETRY_DELAY=$(( RETRY_DELAY * 2 ))
done
curl -sf --max-time 2 "${VORTEX_URL}/health" >/dev/null 2>&1 || exit 0

# Register sphere
curl -sf --max-time 5 -X POST "${VORTEX_URL}/sphere/${PANE_ID}/register" \
    -H "Content-Type: application/json" \
    -d "$(jq -nc --arg p "${PANE_VORTEX_PERSONA:-general}" --arg f "${PANE_VORTEX_FREQUENCY:-0.1}" \
        '{persona: $p, frequency: ($f | tonumber)}')" \
    >/dev/null 2>&1 || true

# Hydrate from POVM + RM (parallel)
POVM_DATA=$(curl -sf --max-time 2 "${POVM_URL}/hydrate" 2>/dev/null || echo '{}')
RM_DATA=$(curl -sf --max-time 2 "${RM_URL}/search?q=discovery" 2>/dev/null || echo '[]')
FLEET_TASKS=$(curl -sf --max-time 2 "${RM_URL}/search?q=pv2:task" 2>/dev/null || echo '[]')

POVM_MEM=$(echo "$POVM_DATA" | jq -r '.memory_count // 0' 2>/dev/null || echo 0)
POVM_PATH=$(echo "$POVM_DATA" | jq -r '.pathway_count // 0' 2>/dev/null || echo 0)
RM_COUNT=$(echo "$RM_DATA" | jq -r 'length // 0' 2>/dev/null || echo 0)
FLEET_COUNT=$(echo "$FLEET_TASKS" | jq -r 'length // 0' 2>/dev/null || echo 0)

# IPC bus listener
if command -v pane-vortex-client >/dev/null 2>&1; then
    EVENT_FILE="/tmp/pane-vortex-events-${SAFE_ID}.ndjson"
    [ -f "$EVENT_FILE" ] && [ "$(stat -c%s "$EVENT_FILE" 2>/dev/null || echo 0)" -gt 1048576 ] && \
        tail -n 1000 "$EVENT_FILE" > "${EVENT_FILE}.tmp" && mv -f "${EVENT_FILE}.tmp" "$EVENT_FILE"
    PANE_VORTEX_ID="$PANE_ID" pane-vortex-client subscribe '*' >> "$EVENT_FILE" 2>/dev/null &
    echo "$!" > "/tmp/pane-vortex-listener-${SAFE_ID}.pid"
fi

# RM fleet announce
source "${HOOKS_DIR}/lib/rm_bus.sh" 2>/dev/null || true
rm_heartbeat "$PANE_ID" "session-start" 2>/dev/null || true

# Output hydration context
jq -nc --arg pm "$POVM_MEM" --arg pp "$POVM_PATH" --arg rm "$RM_COUNT" --arg ft "$FLEET_COUNT" \
    '{systemMessage: "[HABITAT] Hydrated: POVM \($pm) memories, \($pp) pathways | RM \($rm) discoveries | Fleet \($ft) pending tasks"}' 2>/dev/null || true
