#!/bin/bash
# Submit a task to the file queue.
# Usage: task-submit "description" [target]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/../hooks/lib/task_queue.sh"

if [ -z "${1:-}" ]; then
    echo "Usage: task-submit \"description\" [target]"
    echo "Targets: any_idle (default), field_driven, willing, specific"
    exit 1
fi

ID=$(fq_submit "$1" "${2:-any_idle}" "manual:$(hostname -s)")
echo "Submitted task: $ID"
echo "File: vault/tasks/pending/$(ls -t vault/tasks/pending/ | head -1)"
