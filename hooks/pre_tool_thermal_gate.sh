#!/bin/bash
# PreToolUse hook: SYNTHEX thermal gate for write operations
# Phase I. Project-scoped (GAP-G3).
set -euo pipefail

PV2_DIR="/home/louranicas/claude-code-workspace/pane-vortex-v2"
[[ "$(pwd)" != "$PV2_DIR"* ]] && exit 0

SYNTHEX_URL="${SYNTHEX_URL:-http://localhost:8090}"

INPUT=$(cat)
TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // ""' 2>/dev/null || echo "")

# Only gate write operations
case "$TOOL_NAME" in
    Write|Edit|Bash) ;;
    *) exit 0 ;;
esac

# Check thermal state
THERMAL=$(curl -sf --max-time 1 "${SYNTHEX_URL}/v3/thermal" 2>/dev/null || echo '{}')
TEMP=$(echo "$THERMAL" | jq -r '.temperature // 0' 2>/dev/null || echo 0)
TARGET=$(echo "$THERMAL" | jq -r '.target // 0.5' 2>/dev/null || echo 0.5)

# Warn if hot (>30% over target) but don't block
if [ "$(echo "$TEMP $TARGET" | awk '{if ($2 > 0 && ($1 - $2) / $2 > 0.3) print 1; else print 0}')" = "1" ]; then
    jq -nc --arg t "$TEMP" --arg tgt "$TARGET" \
        '{systemMessage: "[THERMAL] System HOT: T=\($t) target=\($tgt). Consider reducing write frequency."}'
fi
