#!/bin/bash
# Reasoning Memory as cross-session message bus.
# TSV format only. Uses pv2: prefix for fleet categories.

RM_URL="${RM_URL:-http://localhost:8130}"

# Submit a fleet task via RM.
# Usage: rm_submit_task "description" [target] [submitter]
rm_submit_task() {
    local desc="$1" target="${2:-any_idle}" submitter="${3:-unknown}"
    printf 'pv2:task\t%s\t0.9\t3600\ttarget=%s desc=%s' "$submitter" "$target" "$desc" |
        curl -sf --max-time 1 -X POST "${RM_URL}/put" --data-binary @- >/dev/null 2>&1 || true
}

# Broadcast a task claim via RM.
# Usage: rm_claim_task "task_id" "claimer"
rm_claim_task() {
    printf 'pv2:claim\t%s\t0.85\t1800\ttask=%s claimer=%s' "$2" "$1" "$2" |
        curl -sf --max-time 1 -X POST "${RM_URL}/put" --data-binary @- >/dev/null 2>&1 || true
}

# Post task completion to RM.
# Usage: rm_complete_task "task_id" "completer" "summary"
rm_complete_task() {
    printf 'pv2:done\t%s\t0.95\t7200\ttask=%s summary=%s' "$2" "$1" "$3" |
        curl -sf --max-time 1 -X POST "${RM_URL}/put" --data-binary @- >/dev/null 2>&1 || true
}

# Post fleet heartbeat to RM.
# Usage: rm_heartbeat "sphere_id" "status"
rm_heartbeat() {
    printf 'pv2:status\t%s\t0.9\t300\tstatus=%s' "$1" "$2" |
        curl -sf --max-time 1 -X POST "${RM_URL}/put" --data-binary @- >/dev/null 2>&1 || true
}

# Check for fleet tasks from RM.
# Usage: rm_check_tasks
rm_check_tasks() {
    curl -sf --max-time 2 "${RM_URL}/search?q=pv2:task" 2>/dev/null
}

# Check fleet status from RM.
# Usage: rm_check_status
rm_check_status() {
    curl -sf --max-time 2 "${RM_URL}/search?q=pv2:status" 2>/dev/null
}
