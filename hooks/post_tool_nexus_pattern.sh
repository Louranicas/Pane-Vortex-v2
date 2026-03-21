#!/bin/bash
# PostToolUse hook: SAN-K7 pattern recording (1-in-10 throttle)
# Phase K. Project-scoped (GAP-G3).
set -euo pipefail

PV2_DIR="/home/louranicas/claude-code-workspace/pane-vortex-v2"
[[ "$(pwd)" != "$PV2_DIR"* ]] && exit 0

NEXUS_URL="${NEXUS_URL:-http://localhost:8100}"
PANE_ID="${PANE_VORTEX_ID:-$(hostname -s):$$}"
SAFE_ID="${PANE_ID//[^a-zA-Z0-9]/_}"
COUNTER_FILE="/tmp/nexus-pattern-counter-${SAFE_ID}"

# 1-in-10 throttle
COUNT=$(cat "$COUNTER_FILE" 2>/dev/null || echo 0)
COUNT=$((COUNT + 1))
echo "$COUNT" > "$COUNTER_FILE"
[ $((COUNT % 10)) -ne 0 ] && exit 0

INPUT=$(cat)
TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // ""' 2>/dev/null || echo "")
TOOL_INPUT=$(echo "$INPUT" | jq -r '(.tool_input // {} | tostring)[:200]' 2>/dev/null || echo "")

[ -z "$TOOL_NAME" ] && exit 0

curl -sf --max-time 1 -X POST "${NEXUS_URL}/api/v1/nexus/command" \
    -H "Content-Type: application/json" \
    -d "$(jq -nc --arg t "$TOOL_NAME" --arg c "$TOOL_INPUT" \
        '{command:"pattern-search",params:{query:$t,context:$c}}')" \
    >/dev/null 2>&1 || true
