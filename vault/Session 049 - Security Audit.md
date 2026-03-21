# Session 049 — Security Audit

> **Date:** 2026-03-21 | **Scope:** hooks/*.sh + src/**/*.rs | **2 parallel agents**

---

## Rust Source: unwrap()/expect() Audit

### Result: ZERO Production Code Violations

All 563 `unwrap()`/`expect(` occurrences are inside `#[cfg(test)]` blocks. All 5 binaries (`main.rs`, `client.rs`, `probe.rs`, `scaffold.rs`, `fleet_verify.rs`) contain zero occurrences. Full compliance with the no-unwrap production rule.

Verified across all 41 modules (8 layers) and 5 binaries.

---

## Shell Hooks: Security Issues Found

### Summary: 14 findings (4 HIGH, 7 MEDIUM, 3 LOW)

| ID | File | Category | Severity |
|----|------|----------|----------|
| 001 | lib/task_queue.sh:35 | Command injection via sed `$2` interpolation | HIGH |
| 002 | lib/task_queue.sh:14-26 | Heredoc injection — unquoted delimiter, raw `$desc` expansion | HIGH |
| 003 | lib/task_queue.sh:53 | `ls` output for file path construction | MEDIUM |
| 004 | post_tool_use.sh:67-71 | Unvalidated server-returned TASK_ID in curl URL | HIGH |
| 005 | session_end.sh:21-22 | Unvalidated file-derived ACTIVE_TID in curl URL | HIGH |
| 006 | session_end.sh:58 | PID file in world-writable /tmp — arbitrary process kill | MEDIUM |
| 007 | session_start.sh:48-49 | Predictable temp file path — TOCTOU symlink race | MEDIUM |
| 008 | session_end.sh:32 | grep pattern injection via `.` in PANE_ID | MEDIUM |
| 009 | 3 hooks (povm/nexus/subagent) | Missing PANE_ID validation in 3 of 8 hooks | MEDIUM |
| 010 | subagent_field_aggregate.sh:22 | TSV injection — embedded tabs in $OUTPUT | MEDIUM |
| 011 | lib/rm_bus.sh:11,18,25,32 | TSV injection — all four RM write functions | MEDIUM |
| 012 | post_tool_use.sh:42 | Unanchored TASK_COMPLETE sentinel | LOW |
| 013 | session_start.sh:37-38 | Missing --connect-timeout on curl calls | LOW |
| 014 | session_end.sh:30-34 | Glob on empty/missing directory | LOW |

### HIGH Priority Details

**FINDING-001: sed command injection** (`lib/task_queue.sh:35`)
```bash
sed -i "s/^claimed_by:.*/claimed_by: $2/" "$dst"
```
`$2` (PANE_ID) interpolated into sed replacement. Characters like `&`, `\n` manipulate replacement. GNU sed `/e` suffix enables command execution.
**Fix:** Use awk: `awk -v r="$2" '/^claimed_by:/{print "claimed_by: " r; next} 1'`

**FINDING-002: heredoc injection** (`lib/task_queue.sh:14-26`)
```bash
cat > "$file" <<TASK
description: "${desc}"
TASK
```
Unquoted delimiter means `$(...)` in `$desc` is expanded by shell at write time.
**Fix:** Quote delimiter: `<<'TASK'`

**FINDING-004: URL path injection** (`post_tool_use.sh:67-71`)
```bash
TASK_ID=$(echo "$FIRST_PENDING" | jq -r '.id')
curl -sf -X POST "${VORTEX_URL}/bus/claim/${TASK_ID}"
```
Server-returned TASK_ID used in URL without validation.
**Fix:** `[[ ! "$TASK_ID" =~ ^[a-zA-Z0-9_-]{1,64}$ ]] && exit 0`

**FINDING-005: file-derived ID injection** (`session_end.sh:21-22`)
```bash
curl -sf -X POST "${VORTEX_URL}/bus/fail/${ACTIVE_TID}"
```
ACTIVE_TID read from `/tmp` file (world-writable) without validation.
**Fix:** Same UUID regex guard + move files out of `/tmp`

### Remediation Priority

1. **Immediate:** FINDING-004, 005 (URL injection), 002 (heredoc), 001 (sed injection)
2. **Near-term:** FINDING-009 (PANE_ID validation), 010/011 (TSV injection), 006 (PID), 007 (TOCTOU), 008 (grep -F), 003 (ls)
3. **Maintenance:** FINDING-012 (anchor grep), 013 (connect-timeout), 014 (nullglob)

---

## Cross-References

- [[Session 049 — Master Index]]
- [[ULTRAPLATE Master Index]]
- [[Session 049 - Quality Gate T810]]
