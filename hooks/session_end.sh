#!/bin/bash
# Stop hook: fail active tasks, crystallize to POVM+RM, deregister, cleanup
# Phase G of fleet coordination plan. Project-scoped (GAP-G3).
set -euo pipefail

# GAP-G3: Project scope guard
PV2_DIR="/home/louranicas/claude-code-workspace/pane-vortex-v2"
[[ "$(pwd)" != "$PV2_DIR"* ]] && exit 0

VORTEX_URL="${PANE_VORTEX_URL:-http://localhost:8132}"
POVM_URL="${POVM_URL:-http://localhost:8125}"
PANE_ID="${PANE_VORTEX_ID:-$(hostname -s):$$}"
[[ ! "$PANE_ID" =~ ^[a-zA-Z0-9_.:-]{1,128}$ ]] && exit 0

SAFE_ID="${PANE_ID//[^a-zA-Z0-9]/_}"
HOOKS_DIR="$(cd "$(dirname "$0")" && pwd)"

# 1. Fail any active claimed task
ACTIVE_TASK_FILE="/tmp/pane-vortex-active-task-${SAFE_ID}"
if [ -f "$ACTIVE_TASK_FILE" ]; then
    ACTIVE_TID=$(cat "$ACTIVE_TASK_FILE")
    curl -sf --max-time 2 -X POST "${VORTEX_URL}/bus/fail/${ACTIVE_TID}" \
        -H "Content-Type: application/json" -d '{}' >/dev/null 2>&1 || true
    rm -f "$ACTIVE_TASK_FILE"
fi

# 2. Move any file-queue claimed tasks to done
source "${HOOKS_DIR}/lib/task_queue.sh" 2>/dev/null || true
TASK_DIR="${PANE_VORTEX_V2_DIR:-/home/louranicas/claude-code-workspace/pane-vortex-v2}/vault/tasks"
for f in "${TASK_DIR}/claimed/"*; do
    [ -f "$f" ] || continue
    if grep -q "claimed_by: $PANE_ID" "$f" 2>/dev/null; then
        fq_complete "$(basename "$f")" 2>/dev/null || true
    fi
done

# 3. Prune old done tasks (GAP-G8)
fq_prune_done 7 2>/dev/null || true

# 4. Mark complete + deregister
curl -sf --max-time 2 -X POST "${VORTEX_URL}/sphere/${PANE_ID}/status" \
    -H "Content-Type: application/json" -d '{"status":"complete"}' >/dev/null 2>&1 || true

# 5. Crystallize to POVM
R=$(curl -sf --max-time 1 "${VORTEX_URL}/health" 2>/dev/null | jq -r '.r // 0' 2>/dev/null || echo 0)
curl -sf --max-time 2 -X POST "${POVM_URL}/snapshots" \
    -H "Content-Type: application/json" \
    -d "$(jq -nc --arg r "$R" --arg sid "$PANE_ID" '{sphere_id:$sid,r:($r|tonumber),event:"session_end"}')" \
    >/dev/null 2>&1 || true

# 6. Crystallize to RM
source "${HOOKS_DIR}/lib/rm_bus.sh" 2>/dev/null || true
rm_complete_task "session" "$PANE_ID" "session-end r=${R}" 2>/dev/null || true
rm_heartbeat "$PANE_ID" "session-end" 2>/dev/null || true

# 7. Kill persistent listener
PID_FILE="/tmp/pane-vortex-listener-${SAFE_ID}.pid"
[ -f "$PID_FILE" ] && kill "$(cat "$PID_FILE")" 2>/dev/null; rm -f "$PID_FILE"

# 8. Deregister sphere (creates ghost trace)
curl -sf --max-time 2 -X POST "${VORTEX_URL}/sphere/${PANE_ID}/deregister" >/dev/null 2>&1 || true

# 9. Cleanup temp files
rm -f "/tmp/pane-vortex-ts-${SAFE_ID}"
rm -f "/tmp/pane-vortex-poll-counter-${SAFE_ID}"
rm -f "/tmp/povm-prev-tool-${SAFE_ID}"
rm -f "/tmp/nexus-pattern-counter-${SAFE_ID}"
