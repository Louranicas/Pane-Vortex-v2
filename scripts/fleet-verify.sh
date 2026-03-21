#!/usr/bin/env bash
# fleet-verify.sh — Reliable fleet instance verification via PV bus + sidecar
# Uses the Habitat's own APIs instead of fragile screen dumps
#
# Sources: PV sphere API, IPC bus info, sidecar ring file, zellij tab names
# Output: structured fleet status with confidence levels
#
# Usage: fleet-verify.sh [--json] [--dispatch TASK]

set -euo pipefail

PV="localhost:8132"
JSON_MODE=false
DISPATCH_TASK=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --json) JSON_MODE=true; shift ;;
    --dispatch) DISPATCH_TASK="$2"; shift 2 ;;
    *) shift ;;
  esac
done

# ── 1. PV Sphere Status (most reliable — daemon-authoritative) ──
sphere_json=$(curl -s "$PV/spheres" 2>/dev/null || echo '{"spheres":[]}')
# Single jq pass to extract all sphere metrics
read -r total_spheres working idle blocked fleet_workers < <(
  echo "$sphere_json" | jq -r '[
    (.spheres | length),
    ([.spheres[] | select(.status == "Working")] | length),
    ([.spheres[] | select(.status == "Idle")] | length),
    ([.spheres[] | select(.status == "Blocked")] | length),
    ([.spheres[] | select(.id | test("^[456]:|^fleet-"))] | length)
  ] | @tsv'
)
total_spheres=${total_spheres:-0}
working=${working:-0}
idle=${idle:-0}
blocked=${blocked:-0}
fleet_workers=${fleet_workers:-0}

# ── 2. IPC Bus State (subscriber = connected client) ──
bus_data=$(curl -s "$PV/bus/info" 2>/dev/null || echo '{}')
subscribers=$(echo "$bus_data" | jq '.subscribers // 0')
pending_tasks=$(echo "$bus_data" | jq '.tasks // 0')
events=$(echo "$bus_data" | jq '.events // 0')

# ── 3. Sidecar (WASM bridge health) ──
sidecar_pid=$(pgrep -x swarm-sidecar 2>/dev/null || echo "")
sidecar_status="DOWN"
sidecar_events=0
if [[ -n "$sidecar_pid" ]]; then
  sidecar_status="UP"
  sidecar_events=$(wc -l < /tmp/swarm-events.jsonl 2>/dev/null || echo 0)
fi

# ── 4. Zellij Tab State (lightweight — tab names only) ──
tabs=$(zellij action query-tab-names 2>/dev/null | tr '\n' ',' || echo "unknown")

# ── 5. Health Composite ──
pv_health=$(curl -s "$PV/health" 2>/dev/null | jq -c '{r,tick,spheres}' || echo '{}')
pv_health_oneline=$(echo "$pv_health" | jq -c '.' 2>/dev/null || echo '{}')
bridges=$(curl -s "$PV/bridges/health" 2>/dev/null || echo '{}')
stale_count=$(echo "$bridges" | jq '[to_entries[] | select(.value == true)] | length')

# ── 6. Confidence Score ──
# High confidence if: PV healthy + subscribers > 0 + sidecar UP
confidence=0
[[ "$total_spheres" -gt 0 ]] && confidence=$((confidence + 30))
[[ "$working" -gt 0 ]] && confidence=$((confidence + 25))
[[ "$subscribers" -gt 0 ]] && confidence=$((confidence + 20))
[[ "$sidecar_status" == "UP" ]] && confidence=$((confidence + 15))
[[ "$stale_count" -eq 0 ]] && confidence=$((confidence + 10))

if $JSON_MODE; then
  cat <<ENDJSON | jq '.'
{
  "spheres": ${total_spheres:-0},
  "working": ${working:-0},
  "idle": ${idle:-0},
  "blocked": ${blocked:-0},
  "fleet_workers": ${fleet_workers:-0},
  "subscribers": ${subscribers:-0},
  "pending_tasks": ${pending_tasks:-0},
  "bus_events": ${events:-0},
  "sidecar": "${sidecar_status}",
  "sidecar_events": ${sidecar_events:-0},
  "stale_bridges": ${stale_count:-0},
  "confidence": ${confidence:-0},
  "tabs": "${tabs}",
  "pv": ${pv_health_oneline}
}
ENDJSON
else
  echo "╔══════════════════════════════════════════════╗"
  echo "║  FLEET VERIFY — $(date +%H:%M:%S)                    ║"
  echo "╠══════════════════════════════════════════════╣"
  echo "║ Spheres:    $total_spheres total ($working working, $idle idle, $blocked blocked)"
  echo "║ Fleet:      $fleet_workers fleet-tagged spheres"
  echo "║ Bus:        $subscribers subs, $pending_tasks tasks, $events events"
  echo "║ Sidecar:    $sidecar_status ($sidecar_events ring events)"
  echo "║ Bridges:    $stale_count/6 stale"
  echo "║ PV:         $pv_health"
  echo "║ Confidence: $confidence/100"
  echo "║ Tabs:       $tabs"
  echo "╚══════════════════════════════════════════════╝"
fi

# ── 7. Optional: dispatch task to idle fleet workers ──
if [[ -n "$DISPATCH_TASK" && "$idle" -gt 0 ]]; then
  PANE_VORTEX_ID="fleet-verify" pane-vortex-client submit \
    --description "$DISPATCH_TASK" --target any-idle 2>/dev/null && \
    echo "Task dispatched to idle worker" || echo "Dispatch failed"
fi
