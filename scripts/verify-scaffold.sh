#!/bin/bash
# Verify pane-vortex-v2 scaffold completeness
set -u

BASE="/home/louranicas/claude-code-workspace/pane-vortex-v2"
PASS=0; FAIL=0; WARN=0

pass() { echo "  [PASS] $1"; PASS=$((PASS+1)); }
fail() { echo "  [FAIL] $1"; FAIL=$((FAIL+1)); }
warn() { echo "  [WARN] $1"; WARN=$((WARN+1)); }

echo "=== PANE-VORTEX V2 SCAFFOLD VERIFICATION ==="
echo "  Date: $(date -Iseconds)"
echo ""

# 1. Core files
echo "--- Core Files ---"
for f in Cargo.toml CLAUDE.md CLAUDE.local.md MASTERPLAN.md src/lib.rs src/bin/main.rs src/bin/client.rs; do
  [ -f "$BASE/$f" ] && pass "$f" || fail "$f missing"
done

# 2. Layer mod.rs files (8)
echo ""
echo "--- Layer Modules (8 expected) ---"
for l in m1_foundation m2_services m3_field m4_coupling m5_learning m6_bridges m7_coordination m8_governance; do
  [ -f "$BASE/src/$l/mod.rs" ] && pass "src/$l/mod.rs" || fail "src/$l/mod.rs missing"
done

# 3. Module stubs (41)
echo ""
echo "--- Module Stubs (41 expected) ---"
count=$(/usr/bin/find "$BASE/src" -name 'm[0-9]*_*.rs' ! -name 'mod.rs' | wc -l)
[[ $count -ge 41 ]] && pass "Module stubs: $count" || fail "Module stubs: $count (expected 41)"

# 4. Config
echo ""
echo "--- Config ---"
[ -f "$BASE/config/default.toml" ] && pass "default.toml" || fail "default.toml missing"
[ -f "$BASE/config/production.toml" ] && pass "production.toml" || fail "production.toml missing"

# 5. Migrations
echo ""
echo "--- Migrations ---"
for m in 001_field_tables.sql 002_bus_tables.sql 003_governance_tables.sql; do
  [ -f "$BASE/migrations/$m" ] && pass "migrations/$m" || fail "migrations/$m missing"
done

# 6. .claude folder
echo ""
echo "--- .claude Operational ---"
for f in context.json status.json patterns.json anti_patterns.json; do
  [ -f "$BASE/.claude/$f" ] && pass ".claude/$f" || fail ".claude/$f missing"
done
for f in queries/field_state.sql queries/bus_tasks.sql queries/governance.sql schemas/bus_frame.schema.json; do
  [ -f "$BASE/.claude/$f" ] && pass ".claude/$f" || fail ".claude/$f missing"
done

# 7. ai_docs
echo ""
echo "--- ai_docs ---"
docs_count=$(/usr/bin/find "$BASE/ai_docs" -name '*.md' 2>/dev/null | wc -l)
[[ $docs_count -ge 10 ]] && pass "ai_docs: $docs_count files" || warn "ai_docs: $docs_count files (target 19)"

# 8. ai_specs
echo ""
echo "--- ai_specs ---"
specs_count=$(/usr/bin/find "$BASE/ai_specs" -name '*.md' 2>/dev/null | wc -l)
[[ $specs_count -ge 10 ]] && pass "ai_specs: $specs_count files" || warn "ai_specs: $specs_count files (target 20)"

# 9. Directories
echo ""
echo "--- Directories ---"
for d in data hooks tests scripts; do
  [ -d "$BASE/$d" ] && pass "$d/" || fail "$d/ missing"
done

# Summary
echo ""
echo "════════════════════════════════════════"
echo "  PASS: $PASS | FAIL: $FAIL | WARN: $WARN"
echo "  Total files: $(/usr/bin/find "$BASE" -type f ! -path '*/.git/*' | wc -l)"
echo "  Total dirs: $(/usr/bin/find "$BASE" -type d ! -path '*/.git/*' | wc -l)"
echo "════════════════════════════════════════"
