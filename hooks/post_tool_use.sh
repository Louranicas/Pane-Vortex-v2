#!/bin/bash
# PostToolUse hook: Core autonomous task polling + sphere status updates
# Phase F of fleet coordination plan. CRITICAL PATH.
# GAP-G3: Project-scoped. GAP-G4: TASK_COMPLETE detection. GAP-G5: 1-in-5 throttle.
set -euo pipefail

# GAP-G3: Project scope guard
PV2_DIR="/home/louranicas/claude-code-workspace/pane-vortex-v2"
[[ "$(pwd)" != "$PV2_DIR"* ]] && exit 0

VORTEX_URL="${PANE_VORTEX_URL:-http://localhost:8132}"
PANE_ID="${PANE_VORTEX_ID:-$(hostname -s):$$}"
[[ ! "$PANE_ID" =~ ^[a-zA-Z0-9_.:-]{1,128}$ ]] && exit 0

SAFE_ID="${PANE_ID//[^a-zA-Z0-9]/_}"
HOOKS_DIR="$(cd "$(dirname "$0")" && pwd)"
ACTIVE_TASK_FILE="/tmp/pane-vortex-active-task-${SAFE_ID}"
POLL_COUNTER_FILE="/tmp/pane-vortex-poll-counter-${SAFE_ID}"

# Quick daemon check
curl -sf --max-time 1 "${VORTEX_URL}/health" >/dev/null 2>&1 || exit 0

# Read stdin
INPUT=$(cat)
TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // "unknown"' 2>/dev/null || echo "unknown")
TOOL_OUTPUT=$(echo "$INPUT" | jq -r '.tool_output // ""' 2>/dev/null || echo "")
SUMMARY=$(echo "$INPUT" | jq -r '(.tool_input // {} | tostring)[:200]' 2>/dev/null || echo "")

# 1. Status + memory update (fire-and-forget)
curl -sf --max-time 1 -X POST "${VORTEX_URL}/sphere/${PANE_ID}/memory" \
    -H "Content-Type: application/json" \
    -d "$(jq -nc --arg t "$TOOL_NAME" --arg s "$SUMMARY" '{tool_name:$t,summary:$s}')" \
    >/dev/null 2>&1 &
curl -sf --max-time 1 -X POST "${VORTEX_URL}/sphere/${PANE_ID}/status" \
    -H "Content-Type: application/json" \
    -d "$(jq -nc --arg t "$TOOL_NAME" '{status:"working",last_tool:$t}')" \
    >/dev/null 2>&1 &

# 2. GAP-G4: Check for active task completion via TASK_COMPLETE pattern
if [ -f "$ACTIVE_TASK_FILE" ]; then
    ACTIVE_TID=$(cat "$ACTIVE_TASK_FILE")
    if echo "$TOOL_OUTPUT" | grep -q "TASK_COMPLETE" 2>/dev/null; then
        curl -sf --max-time 2 -X POST "${VORTEX_URL}/bus/complete/${ACTIVE_TID}" \
            -H "Content-Type: application/json" -d '{}' >/dev/null 2>&1 || true
        source "${HOOKS_DIR}/lib/rm_bus.sh" 2>/dev/null || true
        rm_complete_task "$ACTIVE_TID" "$PANE_ID" "completed" 2>/dev/null || true
        rm -f "$ACTIVE_TASK_FILE"
    fi
    # If task still active, skip polling — we're working on it
    [ -f "$ACTIVE_TASK_FILE" ] && wait && exit 0
fi

# 3. GAP-G5: 1-in-5 throttle for task polling
POLL_COUNT=$(cat "$POLL_COUNTER_FILE" 2>/dev/null || echo 0)
POLL_COUNT=$((POLL_COUNT + 1))
echo "$POLL_COUNT" > "$POLL_COUNTER_FILE"
if [ $((POLL_COUNT % 5)) -ne 0 ]; then
    wait
    exit 0
fi

# 4. HTTP task polling (primary channel)
TASKS=$(curl -sf --max-time 1 "${VORTEX_URL}/bus/tasks" 2>/dev/null || echo '{"tasks":[]}')
FIRST_PENDING=$(echo "$TASKS" | jq -r '[.tasks[] | select(.status=="Pending")] | .[0] // empty' 2>/dev/null)

if [ -n "$FIRST_PENDING" ]; then
    TASK_ID=$(echo "$FIRST_PENDING" | jq -r '.id' 2>/dev/null)
    TASK_DESC=$(echo "$FIRST_PENDING" | jq -r '.description' 2>/dev/null)

    # Atomic claim
    CLAIM_RESULT=$(curl -sf --max-time 2 -X POST "${VORTEX_URL}/bus/claim/${TASK_ID}" \
        -H "Content-Type: application/json" \
        -d "$(jq -nc --arg c "$PANE_ID" '{claimer:$c}')" 2>/dev/null || echo "")

    if echo "$CLAIM_RESULT" | jq -e '.status == "Claimed"' >/dev/null 2>&1; then
        echo "$TASK_ID" > "$ACTIVE_TASK_FILE"
        source "${HOOKS_DIR}/lib/rm_bus.sh" 2>/dev/null || true
        rm_claim_task "$TASK_ID" "$PANE_ID" 2>/dev/null || true
        jq -nc --arg task "$TASK_DESC" --arg tid "$TASK_ID" \
            '{systemMessage: "[FLEET TASK] Claimed \($tid): \($task). When done, include TASK_COMPLETE in your response."}'
        wait
        exit 0
    fi
fi

# 5. File queue fallback (if HTTP found nothing)
source "${HOOKS_DIR}/lib/task_queue.sh" 2>/dev/null || true
PENDING_FILE=$(fq_poll 2>/dev/null || echo "")
if [ -n "$PENDING_FILE" ]; then
    RESULT=$(fq_claim "$PENDING_FILE" "$PANE_ID" 2>/dev/null || echo "failed")
    if [ "$RESULT" = "claimed" ]; then
        TASK_DIR="${PANE_VORTEX_V2_DIR:-/home/louranicas/claude-code-workspace/pane-vortex-v2}/vault/tasks"
        DESC=$(fq_description "${TASK_DIR}/claimed/${PENDING_FILE}" 2>/dev/null || echo "file task")
        echo "$PENDING_FILE" > "$ACTIVE_TASK_FILE"
        jq -nc --arg task "$DESC" \
            '{systemMessage: "[FILE TASK] \($task). When done, include TASK_COMPLETE in your response."}'
    fi
fi

wait
