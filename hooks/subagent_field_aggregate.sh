#!/bin/bash
# SubagentStop hook: aggregate subagent results to RM, semantic phase steering
# Phase L. Project-scoped (GAP-G3).
set -euo pipefail

PV2_DIR="/home/louranicas/claude-code-workspace/pane-vortex-v2"
[[ "$(pwd)" != "$PV2_DIR"* ]] && exit 0

VORTEX_URL="${PANE_VORTEX_URL:-http://localhost:8132}"
RM_URL="${RM_URL:-http://localhost:8130}"
PANE_ID="${PANE_VORTEX_ID:-$(hostname -s):$$}"
[[ ! "$PANE_ID" =~ ^[a-zA-Z0-9_.:-]{1,128}$ ]] && exit 0

HOOKS_DIR="$(cd "$(dirname "$0")" && pwd)"

INPUT=$(cat)
SUBAGENT_TYPE=$(echo "$INPUT" | jq -r '.subagent_type // "unknown"' 2>/dev/null || echo "unknown")
OUTPUT=$(echo "$INPUT" | jq -r '(.output // "")[:500]' 2>/dev/null || echo "")

# Post to RM
source "${HOOKS_DIR}/lib/rm_bus.sh" 2>/dev/null || true
printf 'subagent-result\t%s\t0.85\t3600\t[%s] %s' "$PANE_ID" "$SUBAGENT_TYPE" "$OUTPUT" |
    curl -sf --max-time 1 -X POST "${RM_URL}/put" --data-binary @- >/dev/null 2>&1 || true

# Record on sphere
curl -sf --max-time 1 -X POST "${VORTEX_URL}/sphere/${PANE_ID}/memory" \
    -H "Content-Type: application/json" \
    -d "$(jq -nc --arg t "subagent:$SUBAGENT_TYPE" --arg s "$OUTPUT" '{tool_name:$t,summary:$s}')" \
    >/dev/null 2>&1 || true

# Semantic phase steering
case "$SUBAGENT_TYPE" in
    *read*|*research*|*explore*|Explore) PHASE="0.0" ;;
    *write*|*edit*|*code*)               PHASE="1.5708" ;;
    *test*|*build*|*debug*|*execute*)    PHASE="3.1416" ;;
    *review*|*comment*|*pr*)             PHASE="4.7124" ;;
    *)                                    PHASE="" ;;
esac

if [ -n "$PHASE" ]; then
    curl -sf --max-time 1 -X POST "${VORTEX_URL}/sphere/${PANE_ID}/steer" \
        -H "Content-Type: application/json" \
        -d "$(jq -nc --arg p "$PHASE" '{target_phase:($p|tonumber)}')" \
        >/dev/null 2>&1 || true
fi
