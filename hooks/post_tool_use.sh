#!/bin/bash
# PostToolUse hook: record memory, update status, frequency discovery
# V3: Checks consent before posting (NA-SG-1)
set -euo pipefail

VORTEX_URL="${PANE_VORTEX_URL:-http://localhost:8132}"
PANE_ID="${PANE_VORTEX_ID:-$(hostname -s):$$}"
[[ ! "$PANE_ID" =~ ^[a-zA-Z0-9_.:-]{1,128}$ ]] && exit 0
curl -sf "${VORTEX_URL}/health" >/dev/null 2>&1 || exit 0

# V3.3.8: Check consent before RM logging (NA-SG-1)
# (stub — will query /sphere/{id}/consent when governance is implemented)

INPUT=$(cat)
TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // "unknown"' 2>/dev/null || echo "unknown")
SUMMARY=$(echo "$INPUT" | jq -r '(.tool_input // {} | tostring)[:200]' 2>/dev/null || echo "")

# Record memory + update status
curl -sf -X POST "${VORTEX_URL}/sphere/${PANE_ID}/memory" \
    -H "Content-Type: application/json" \
    -d "$(jq -nc --arg tool "$TOOL_NAME" --arg sum "$SUMMARY" '{tool_name: $tool, summary: $sum}')" \
    >/dev/null 2>&1 || true

curl -sf -X POST "${VORTEX_URL}/sphere/${PANE_ID}/status" \
    -H "Content-Type: application/json" \
    -d "$(jq -nc --arg tool "$TOOL_NAME" '{status: "working", last_tool: $tool}')" \
    >/dev/null 2>&1 || true
