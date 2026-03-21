#!/bin/bash
# File-based task queue for fleet coordination.
# Atomic claims via mv -n. No service dependency.

TASK_DIR="${PANE_VORTEX_V2_DIR:-/home/louranicas/claude-code-workspace/pane-vortex-v2}/vault/tasks"

# Submit a task to the file queue.
# Usage: fq_submit "description" [target] [submitter]
fq_submit() {
    local desc="$1" target="${2:-any_idle}" submitter="${3:-manual}"
    local ts id file
    ts=$(date +%s)
    id=$(head -c 8 /dev/urandom | xxd -p)
    file="${TASK_DIR}/pending/${ts}-${id}.md"
    cat > "$file" <<TASK
---
id: ${id}
description: "${desc}"
submitted_by: ${submitter}
submitted_at: $(date -Iseconds)
claimed_by: ""
target: ${target}
---
${desc}
TASK
    echo "$id"
}

# Claim a task atomically. Returns "claimed" or "failed".
# Usage: fq_claim "filename" "claimer"
fq_claim() {
    local src="${TASK_DIR}/pending/$1"
    local dst="${TASK_DIR}/claimed/$1"
    if mv -n "$src" "$dst" 2>/dev/null; then
        sed -i "s/^claimed_by:.*/claimed_by: $2/" "$dst" 2>/dev/null
        echo "claimed"
    else
        echo "failed"
    fi
}

# Complete a claimed task.
# Usage: fq_complete "filename"
fq_complete() {
    local src="${TASK_DIR}/claimed/$1"
    local dst="${TASK_DIR}/done/$1"
    mv -f "$src" "$dst" 2>/dev/null
}

# Poll for oldest pending task. Returns filename or empty.
# Usage: fq_poll
fq_poll() {
    ls "${TASK_DIR}/pending/" 2>/dev/null | head -1
}

# Get the description from a task file.
# Usage: fq_description "filepath"
fq_description() {
    grep '^description:' "$1" 2>/dev/null | sed 's/^description: *//' | sed 's/^"//;s/"$//'
}

# Prune done tasks older than N days.
# Usage: fq_prune_done [days]
fq_prune_done() {
    local days="${1:-7}"
    find "${TASK_DIR}/done/" -name "*.md" -mtime +"$days" -delete 2>/dev/null || true
}
