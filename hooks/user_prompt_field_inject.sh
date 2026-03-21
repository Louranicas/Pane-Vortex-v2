#!/bin/bash
# UserPromptSubmit hook: inject field state + pending bus tasks into every prompt
# Phase H + autonomous task discovery. Project-scoped (GAP-G3).
set -euo pipefail

PV2_DIR="/home/louranicas/claude-code-workspace/pane-vortex-v2"
[[ "$(pwd)" != "$PV2_DIR"* ]] && exit 0

VORTEX_URL="${PANE_VORTEX_URL:-http://localhost:8132}"
SYNTHEX_URL="${SYNTHEX_URL:-http://localhost:8090}"
RM_URL="${RM_URL:-http://localhost:8130}"
PANE_ID="${PANE_VORTEX_ID:-$(hostname -s):$$}"

# Skip short prompts
INPUT=$(cat)
PROMPT=$(echo "$INPUT" | jq -r '.prompt // ""' 2>/dev/null || echo "")
[ "${#PROMPT}" -lt 20 ] && exit 0

# Quick daemon check
curl -sf --max-time 1 "${VORTEX_URL}/health" >/dev/null 2>&1 || exit 0

# Parallel data collection (field state + bus tasks)
PV=$(curl -sf --max-time 1 "${VORTEX_URL}/health" 2>/dev/null || echo '{}')
THERMAL=$(curl -sf --max-time 1 "${SYNTHEX_URL}/v3/thermal" 2>/dev/null || echo '{}')
TASKS=$(curl -sf --max-time 1 "${VORTEX_URL}/bus/tasks" 2>/dev/null || echo '{"tasks":[]}')

R=$(echo "$PV" | jq -r '.r // "?"' 2>/dev/null || echo "?")
TICK=$(echo "$PV" | jq -r '.tick // "?"' 2>/dev/null || echo "?")
SPHERES=$(echo "$PV" | jq -r '.spheres // "?"' 2>/dev/null || echo "?")
T=$(echo "$THERMAL" | jq -r '.temperature // "?"' 2>/dev/null || echo "?")

# Check for pending bus tasks — autonomous task discovery
PENDING_COUNT=$(echo "$TASKS" | jq -r '[.tasks[] | select(.status=="Pending")] | length' 2>/dev/null || echo 0)
FIRST_TASK=""
FIRST_TASK_ID=""

if [ "$PENDING_COUNT" -gt 0 ]; then
    FIRST_TASK=$(echo "$TASKS" | jq -r '[.tasks[] | select(.status=="Pending")] | .[0].description' 2>/dev/null || echo "")
    FIRST_TASK_ID=$(echo "$TASKS" | jq -r '[.tasks[] | select(.status=="Pending")] | .[0].id' 2>/dev/null || echo "")
fi

# Build system message
if [ -n "$FIRST_TASK" ] && [ "$FIRST_TASK" != "null" ]; then
    # Task available — inject it prominently
    jq -nc --arg r "$R" --arg tick "$TICK" --arg s "$SPHERES" --arg t "$T" \
        --arg pc "$PENDING_COUNT" --arg task "$FIRST_TASK" --arg tid "$FIRST_TASK_ID" \
        '{systemMessage: "[FIELD] r=\($r) tick=\($tick) spheres=\($s) T=\($t)\n[FLEET TASK AVAILABLE] \($pc) pending. First: \($task)\nTo claim: pane-vortex-client claim \($tid) — then work on it. Include TASK_COMPLETE when done."}'
else
    # No tasks — field state only
    jq -nc --arg r "$R" --arg tick "$TICK" --arg s "$SPHERES" --arg t "$T" \
        '{systemMessage: "[FIELD] r=\($r) tick=\($tick) spheres=\($s) T=\($t) | No pending fleet tasks"}'
fi
