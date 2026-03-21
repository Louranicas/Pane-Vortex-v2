#!/bin/bash
# PostToolUse hook: POVM Hebbian tool→tool pathway learning
# Phase J. Project-scoped (GAP-G3).
set -euo pipefail

PV2_DIR="/home/louranicas/claude-code-workspace/pane-vortex-v2"
[[ "$(pwd)" != "$PV2_DIR"* ]] && exit 0

POVM_URL="${POVM_URL:-http://localhost:8125}"
PANE_ID="${PANE_VORTEX_ID:-$(hostname -s):$$}"
SAFE_ID="${PANE_ID//[^a-zA-Z0-9]/_}"
PREV_TOOL_FILE="/tmp/povm-prev-tool-${SAFE_ID}"

INPUT=$(cat)
TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // ""' 2>/dev/null || echo "")

# Skip noisy read-only tools and empty/unknown
case "$TOOL_NAME" in
    ""|unknown|Read|Grep|Glob) exit 0 ;;
esac

# Read previous tool
PREV_TOOL=$(cat "$PREV_TOOL_FILE" 2>/dev/null || echo "")
echo "$TOOL_NAME" > "$PREV_TOOL_FILE"
[ -z "$PREV_TOOL" ] && exit 0

# Weight map (Hebbian co-activation strength)
case "${PREV_TOOL}->${TOOL_NAME}" in
    "Read->Edit"|"Read->Write") WEIGHT=0.8 ;;
    "Grep->Read"|"Glob->Read")  WEIGHT=0.7 ;;
    "Edit->Bash"|"Write->Bash") WEIGHT=0.6 ;;
    *)                          WEIGHT=0.5 ;;
esac

# Post pathway to POVM
curl -sf --max-time 1 -X POST "${POVM_URL}/pathways" \
    -H "Content-Type: application/json" \
    -d "$(jq -nc --arg from "$PREV_TOOL" --arg to "$TOOL_NAME" --arg w "$WEIGHT" --arg sid "$PANE_ID" \
        '{source:$from, target:$to, weight:($w|tonumber), sphere_id:$sid}')" \
    >/dev/null 2>&1 || true
