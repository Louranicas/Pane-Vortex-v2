---
date: 2026-03-18
tags: [session-039, nvim, bash, chaining, treesitter, command-reference, zsde, god-tier, pane-vortex, ultraplate]
aliases: [Nvim Command Reference, ZSDE Command Bible, Session 039 Commands]
---

# Session 039 — ZSDE Nvim God-Tier Command Reference

> **Complete command reference for the Zellij Synthetic Developer Environment.**
> **Built through 5 exploration iterations + 2 synthesis loops over ~35 minutes.**
> **Bidirectional links:** [[Pane-Vortex — Fleet Coordination Daemon]] | [[Session 036 — Nvim-Bash Command Chaining Analysis]] | [[Session 027 — Nvim Synergy and Tool Chaining]] | [[Zellij Master Skill — Bootstrap Reference]] | [[Zellij Navigation God-Tier — Session 035]]

---

## 1. Nvim Remote Socket Commands

**Socket:** `/tmp/nvim.sock` (auto-started by `~/.config/nvim/lua/config/options.lua`)

### Basic Operations
```bash
nvim --server /tmp/nvim.sock --remote-send ':e /path/to/file<CR>'
nvim --server /tmp/nvim.sock --remote-expr 'bufname("%")'
nvim --server /tmp/nvim.sock --remote-expr 'line("$")'
nvim --server /tmp/nvim.sock --remote-expr 'getline(42)'
nvim --server /tmp/nvim.sock --remote-expr 'join(getline(10, 20), "\n")'
nvim --server /tmp/nvim.sock --remote-send ':42<CR>'
nvim --server /tmp/nvim.sock --remote-expr 'expand("%:.") . ":" . line(".")'
```

### Buffer Management
```bash
nvim --server /tmp/nvim.sock --remote-expr 'execute("ls")'
nvim --server /tmp/nvim.sock --remote-expr 'luaeval("#vim.fn.getbufinfo({buflisted=1})")'
nvim --server /tmp/nvim.sock --remote-send ':args ~/path/src/*.rs<CR>'
nvim --server /tmp/nvim.sock --remote-expr 'argc()'
```

### Search and Quickfix
```bash
nvim --server /tmp/nvim.sock --remote-send ':vimgrep /pattern/j ~/path/src/*.rs<CR>'
nvim --server /tmp/nvim.sock --remote-expr 'len(getqflist())'
```

### Marks and Registers
```bash
nvim --server /tmp/nvim.sock --remote-send ':e /path/api.rs<CR>mA'
nvim --server /tmp/nvim.sock --remote-send "'A"
nvim --server /tmp/nvim.sock --remote-expr 'len(getjumplist()[0])'
```

---

## 2. LSP Commands (rust-analyzer)

```bash
# Diagnostics by severity (1=ERROR, 2=WARN, 4=HINT)
nvim --server /tmp/nvim.sock --remote-expr 'luaeval("vim.tbl_count(vim.diagnostic.get(nil,{severity=1}))")'

# Extract error messages with locations
nvim --server /tmp/nvim.sock --remote-expr 'luaeval("(function() local diags = vim.diagnostic.get(nil, {severity=1}) local out = {} for i, d in ipairs(diags) do if i > 5 then break end table.insert(out, vim.fn.fnamemodify(vim.fn.bufname(d.bufnr), \":t\") .. \":\" .. (d.lnum+1) .. \" \" .. d.message:sub(1,80)) end return table.concat(out, \"\\n\") end)()")'

# LSP-attached buffers
nvim --server /tmp/nvim.sock --remote-expr 'luaeval("(function() local bufs = vim.lsp.get_clients()[1].attached_buffers local names = {} for b in pairs(bufs) do local n = vim.fn.bufname(b) if n ~= \"\" then table.insert(names, vim.fn.fnamemodify(n, \":t\")) end end table.sort(names) return table.concat(names, \", \") end)()")'

# Inlay hints enabled?
nvim --server /tmp/nvim.sock --remote-expr 'luaeval("vim.lsp.inlay_hint.is_enabled({bufnr=0})")'
```

---

## 3. Treesitter AST Commands

### Struct Names + Field Counts
```bash
nvim --server /tmp/nvim.sock --remote-expr 'luaeval("(function() local parser = vim.treesitter.get_parser(0, \"rust\") local tree = parser:parse()[1] local root = tree:root() local result = {} for node in root:iter_children() do if node:type() == \"struct_item\" then local name, fields = \"\", 0 for child in node:iter_children() do if child:type() == \"type_identifier\" then name = vim.treesitter.get_node_text(child, 0) elseif child:type() == \"field_declaration_list\" then for f in child:iter_children() do if f:type() == \"field_declaration\" then fields = fields + 1 end end end end if name ~= \"\" then table.insert(result, name .. \"(\" .. fields .. \")\") end end end return table.concat(result, \", \") end)()")'
```

### Function Names from Impl Blocks
```bash
nvim --server /tmp/nvim.sock --remote-expr 'luaeval("(function() local parser = vim.treesitter.get_parser(0, \"rust\") local tree = parser:parse()[1] local root = tree:root() local fns = {} for node in root:iter_children() do if node:type() == \"impl_item\" then for child in node:iter_children() do if child:type() == \"declaration_list\" then for fn_node in child:iter_children() do if fn_node:type() == \"function_item\" then local nm = fn_node:field(\"name\")[1] if nm then table.insert(fns, vim.treesitter.get_node_text(nm, 0)) end end end end end end end return table.concat(fns, \", \") end)()")'
```

### AST Parent Chain
```bash
nvim --server /tmp/nvim.sock --remote-expr 'luaeval("(function() local n=vim.treesitter.get_node() local c={} while n do table.insert(c, n:type()) n=n:parent() end return table.concat(c, \" -> \") end)()")'
```

### Import Graph Extraction
```bash
nvim --server /tmp/nvim.sock --remote-expr 'luaeval("(function() local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false) local uses = {} for _, line in ipairs(lines) do local mod = line:match(\"use crate::(%w+)\") if mod and not uses[mod] then uses[mod] = true end end local list = {} for k in pairs(uses) do table.insert(list, k) end table.sort(list) return table.concat(list, \", \") end)()")'
```

---

## 4. Custom Keymaps (800L, 8 prefix groups)

### Zellij Collaboration (`<leader>z`)
| Key | Action |
|-----|--------|
| z1-z6 | Go to tab 1-6 |
| za | Send line to agent pane |
| zf/zb/zg | Send visual selection to ALPHA/BETA/GAMMA |
| zd | Field-aware dispatch (PV decision -> best tab) |

### ULTRAPLATE Field (`<leader>u`)
| Key | Action |
|-----|--------|
| up | PV health | uf | Field decision | uc | Chimera |
| ut | Tunnels | uS | Spectrum | ug | Ghosts |
| uR | Register sphere | uE | Emergency coherence |
| uD | Diagnostics | uB | Bridge health | uI | Integration matrix |
| us | Full brief (4 services parallel) | uA | Cluster B brain |
| uF | Fleet status | uC | Conductor | uP | Post synthesis to RM |
| uT | SYNTHEX thermal | uV | POVM hydration |
| ue | Evolution patterns | ud | SX diagnostics | um | ME observer |
| ub | Bash safety | ux | Export diags JSON | uM | Inject observation |

### SAN-K7 Nexus (`<leader>n`)
| Key | Action |
|-----|--------|
| ns | Synergy | nl | Lint | nb | Best practice |
| nf | Pattern search | nh | Health | nm | Memory consolidate |
| n7 | TC7 chain (4 commands, 19ms) |

### Reasoning Memory (`<leader>u`)
| Key | Action |
|-----|--------|
| ur | Post code observation / visual discovery |
| uq | Search RM for word under cursor |

### Search (`<leader>s`) / Files (`<leader>f`) / Git (`<leader>g`)
| Prefix | Key highlights |
|--------|---------------|
| s | ss symbols, st todo, su undotree, sq quickfix |
| f | ff find, fr recent, fb buffers, fg git-files, fe explorer |
| g | gg lazygit, gs status, gb blame, gd diffview, gh history |

### Code (`<leader>c`) / Trouble (`<leader>x`) / Buffer (`<leader>b`)
| Prefix | Key highlights |
|--------|---------------|
| c | cr rename, ca action, cc codelens, cm mason, cF format |
| x | xx diagnostics, xt todo, xq quickfix |
| b | bj pick, bp pin, bd delete, bb alternate |

### Quick Open (`<leader>o`) / Yank (`<leader>y`) / Terminal (`<leader>t`)
| Key | Action |
|-----|--------|
| oP | pane-vortex main.rs | oV | VMS lib.rs | oC | CLAUDE.md |
| yf | relative path | yF | absolute path | yl | file:line |
| tt/tv/tf | horizontal/vertical/float terminal |

### Other keys
| Key | Action |
|-----|--------|
| `-` | Oil file browser | `C-\` | Toggle terminal |
| `C-h/j/k/l` | Zellij-nav seamless | `]h/[h` | Gitsigns hunk nav |

---

## 5. Chained Command Recipes

### TC1: Full Service Intelligence (28-33ms)
```bash
PV=$(curl -s http://localhost:8132/health | jq -c '{r,spheres,tick}') && \
POVM=$(curl -s http://localhost:8125/hydrate | jq -c '{m:.memory_count,p:.pathway_count}') && \
K7=$(curl -s -X POST http://localhost:8100/api/v1/nexus/command -H "Content-Type: application/json" \
  -d '{"command":"synergy-check","params":{}}' | jq -r '.data.output.status') && \
SX=$(curl -s http://localhost:8090/v3/thermal | jq -r '.temperature') && \
ME=$(curl -s http://localhost:8080/api/observer | jq -r '.last_report.current_fitness') && \
echo "PV=$PV POVM=$POVM K7=$K7 SX=$SX ME=$ME"
```

### Diagnostic Pipeline (nvim -> services, 33ms)
```bash
DIAG_E=$(nvim --server /tmp/nvim.sock --remote-expr 'luaeval("vim.tbl_count(vim.diagnostic.get(nil,{severity=1}))")') && \
BRIDGE=$(curl -s http://localhost:8132/bridges/health | jq -r '.last_bridge_adjustments.combined_effect') && \
HI_PW=$(curl -s http://localhost:8125/pathways | jq '[.[] | select(.weight > 0.5)] | length') && \
echo "LSP:${DIAG_E}E Bridges:${BRIDGE} POVM_hi:${HI_PW}"
```

### ZSDE State Vector (single line)
```bash
TAB=$(zellij action query-tab-names | head -1) && \
BUF=$(nvim --server /tmp/nvim.sock --remote-expr 'expand("%:t")') && \
PV=$(curl -s http://localhost:8132/health | jq -r '"t\(.tick)/\(.spheres)sph"') && \
echo "tab=$TAB buf=$BUF pv=$PV"
```

### Multi-File Struct Map (treesitter, ~200ms/file)
```bash
for mod in types sphere state field coupling conductor bus executor; do
  nvim --server /tmp/nvim.sock --remote-send ":e ~/path/src/${mod}.rs<CR>"
  sleep 0.2
  nvim --server /tmp/nvim.sock --remote-expr '...treesitter struct extraction...'
done
```

### Module Fan-In
```bash
for target in types sphere coupling field state bus; do
  grep -rl "use crate::${target}" ~/path/src/*.rs | wc -l
done
```

### Cross-DB Correlation
```bash
sqlite3 ~/.local/share/pane-vortex/field_tracking.db "
SELECT fs.tick, ROUND(fs.r,4), fs.sphere_count,
       (SELECT COUNT(*) FROM sphere_history sh WHERE sh.tick = fs.tick)
FROM field_snapshots fs ORDER BY fs.tick DESC LIMIT 10;"
```

### Post to Reasoning Memory (TSV)
```bash
printf 'category\tagent\tconfidence\tttl\tcontent' | \
  curl -sf -X POST http://localhost:8130/put --data-binary @-
```

### Integration Matrix (live)
```bash
bash ~/claude-code-workspace/pane-vortex/arena/integration-matrix.sh
```

---

## 6. Structural Analysis Findings

### Struct Complexity (top 10, treesitter-derived)
| Struct | Fields | Module |
|--------|--------|--------|
| PaneSphere | 33 | sphere.rs |
| AppState | 24 | state.rs |
| SphereSummary | 22 | field.rs |
| FieldDecision | 12 | field.rs |
| BusState | 12 | bus.rs |
| ExecutorTaskRecord | 11 | executor.rs |
| GhostTrace | 10 | state.rs |
| NexusFieldState | 10 | nexus_bridge.rs |
| CouplingNetwork | 8 | coupling.rs |
| NestedKuramotoMetrics | 8 | nexus_bridge.rs |

### Enum Variants (10 enums, 55 total)
ClientFrame(12v) FieldAction(8v) ServerFrame(8v) PhaseMessage(5v) TaskStatus(5v) PaneStatus(4v) TaskTarget(4v) RTrend(3v) SuggestionType(3v) CascadeAckStatus(3v)

### Module Fan-In
types(9) > sphere(8) > coupling(5) > field(4) = state(4) > bus(2) = chimera(2)

### Dependency Graph (api.rs is the hub, imports 15 modules)
```
api -> {advanced_logging, bus, chimera, evolution_api, executor, field, me_bridge,
        messaging, nexus_bridge, nexus_bus, persistence, sphere, state, synthex_bridge, types}
```

### Risk Hotspots
field.rs: 51 branches / 18 tests (0.35 ratio) - HIGHEST RISK
coupling.rs: 36 branches / 9 tests (0.25 ratio) - HIGH
api.rs: 107 branches / 0 inline tests (integration-tested externally)

### Rust Metrics
383 .await | 22 Arc<RwLock> | 7 unsafe | 427 #[test] | 125 TODO/FIXME | 25 modules | ~24,983 LOC

---

## 7. Service API Map

| Service | Port | Key Endpoints |
|---------|------|---------------|
| PV | 8132 | /health /spheres /field/decision /field/chimera /field/tunnels /bridges/health /integration/matrix /nexus/metrics /bus/info /evolution/status /analytics/patterns |
| SX | 8090 | /api/health /v3/thermal /v3/diagnostics /v3/health |
| ME | 8080 | /api/health /api/status /api/observer |
| K7 | 8100 | /health /status POST /api/v1/nexus/command |
| POVM | 8125 | /health /memories /pathways /hydrate /consolidate /snapshots/latest |
| RM | 8130 | /health POST /put (TSV!) /search?q= |
| + 10 more | 8081-8110, 9001, 10001 | /health on all |

---

## 8. Database Reference

### bus_tracking.db (7 tables, ~/.local/share/pane-vortex/)
bus_tasks, task_tags, task_dependencies, bus_events, event_subscriptions, cascade_events, schema_versions

### field_tracking.db (4 tables)
field_snapshots (tick PK, r, k, k_mod, sphere_count, decision, chimera, breathing)
sphere_history (tick+sphere_id PK, phase, freq, status, tool, memory_count, steps)
coupling_history (tick+a+b PK, weight)

### povm_data.db (PV local copy, 4 tables)
memories (39), pathways (0 - engine has 2425), field_snapshots (1), sessions (0)

---

## 9. Memory Locations

| System | Location |
|--------|----------|
| Auto-Memory | `~/.claude/projects/-home-louranicas-claude-code-workspace-pane-vortex/memory/MEMORY.md` |
| PV Context | `pane-vortex/.claude/context.json` |
| PV Patterns | `pane-vortex/.claude/patterns.json` (15) + anti_patterns.json (20) |
| SQL Templates | `pane-vortex/.claude/queries/` |
| JSON Schemas | `pane-vortex/.claude/schemas/` |
| SQLite bus | `~/.local/share/pane-vortex/bus_tracking.db` |
| SQLite field | `~/.local/share/pane-vortex/field_tracking.db` |
| SQLite POVM | `~/.local/share/pane-vortex/povm_data.db` |
| POVM Engine | `http://localhost:8125` |
| Reasoning Memory | `http://localhost:8130` (TSV not JSON) |
| Obsidian | `~/projects/claude_code/` (257 notes) |
| Shared Context | `~/projects/shared-context/` |

---

## 10. Pioneer Loop Commands (Advanced Chains)

### Treesitter Cyclomatic Complexity Per Function
```bash
nvim --server /tmp/nvim.sock --remote-expr 'luaeval("(function() local parser = vim.treesitter.get_parser(0, \"rust\") local tree = parser:parse()[1] local fns = {} local function cb(node) local c=0 for child in node:iter_children() do local t=child:type() if t==\"if_expression\" or t==\"match_expression\" or t==\"for_expression\" or t==\"while_expression\" or t==\"loop_expression\" then c=c+1 end c=c+cb(child) end return c end local function scan(node) for child in node:iter_children() do if child:type()==\"function_item\" then local nm=child:field(\"name\")[1] if nm then local sr,_,er,_=child:range() local b=cb(child) table.insert(fns, {n=vim.treesitter.get_node_text(nm,0), b=b, l=er-sr}) end end scan(child) end end scan(tree:root()) table.sort(fns, function(a,b) return a.b > b.b end) local out={} for i=1,math.min(6,#fns) do table.insert(out, fns[i].n..\"(\"..fns[i].b..\"br/\"..fns[i].l..\"L)\") end return table.concat(out, \", \") end)()")'
```

### Treesitter Enum Variant Extraction
```bash
# Same pattern as struct extraction but with enum_item/enum_variant_list/enum_variant
```

### Treesitter Constant Map
```bash
# Iterate root children for const_item/static_item -> identifier
```

### POVM Weight Distribution Analysis
```bash
curl -s http://localhost:8125/pathways | jq '[.[] | .weight] |
  {total: length, below_0_2: [.[]|select(.<0.2)]|length,
   range_0_2_0_3: [.[]|select(.>=0.2 and .<0.3)]|length,
   above_0_8: [.[]|select(.>=0.8)]|length}'
```

### Cross-DB ATTACH Join (bus + field)
```bash
sqlite3 ~/.local/share/pane-vortex/bus_tracking.db "
ATTACH '$HOME/.local/share/pane-vortex/field_tracking.db' AS field;
SELECT be.tick, COUNT(*) as events, ROUND(fs.r, 4) as r
FROM bus_events be
JOIN field.field_snapshots fs ON be.tick = fs.tick
WHERE be.tick IS NOT NULL GROUP BY be.tick ORDER BY COUNT(*) DESC LIMIT 8;"
```

### Sphere Lifetime Statistics
```bash
sqlite3 ~/.local/share/pane-vortex/field_tracking.db "
WITH lives AS (SELECT sphere_id, MAX(tick)-MIN(tick) as lifetime, MAX(total_steps) as steps
FROM sphere_history GROUP BY sphere_id)
SELECT ROUND(AVG(lifetime)), ROUND(AVG(steps)), COUNT(*) FROM lives;"
```

### Module Fan-In (who depends on me?)
```bash
for target in types sphere coupling field state bus; do
  grep -rl "use crate::${target}" ~/path/src/*.rs | wc -l
done
```

### Zellij Log Inspection (live bug hunting)
```bash
tail -20 /tmp/zellij-1000/zellij-log/zellij.log
```

### K7 Full Command Audit
```bash
for cmd in service-health synergy-check module-status build compliance pattern-search \
  memory-consolidate best-practice deploy-swarm lint agent-status; do
  STATUS=$(curl -s --max-time 2 -X POST http://localhost:8100/api/v1/nexus/command \
    -H "Content-Type: application/json" -d "{\"command\":\"$cmd\",\"params\":{}}" | \
    jq -r '.data.output.status // .error // "timeout"')
  echo "$cmd: $STATUS"
done
```

---

## 11. Lazygit 3-Tool Chain Commands

See [[Session 039 — Lazygit God-Tier Command Reference#10. 3-Tool Chain Commands]] for the full set. Key chains:

| Chain | Command Pattern | Intelligence |
|-------|-----------------|-------------|
| Blame complexity | `git blame -L range file \| grep "if" \| uniq -c` | Which commit added which branches |
| Temporal coupling | `git log --name-only \| awk co-change counter` | Files that always change together |
| Diff + LSP | `git diff --name-only` + nvim diagnostics per file | Which changed files have errors |
| Growth curve | `git show hash:file \| wc -l` per commit | Historical function/file size |
| Fossil detection | `git blame \| grep oldest_hash` | Original lines still surviving |
| Change classification | `git diff \| grep "^+.*pub struct/fn/use"` | What types of changes are uncommitted |
| Intent archaeology | `curl RM/search?q=feature_name` | Did previous sessions plan this? |
| Codebase health | git stats + nvim diags + curl services | 3-dimensional health in 80ms |

---

## 12. Bugs Discovered This Session

See [[ULTRAPLATE — Bugs and Known Issues]] for full details:

| ID | Severity | Description |
|----|----------|-------------|
| BUG-019 | HIGH | tick_once god function (65br/829L) |
| BUG-020 | HIGH | 146 .unwrap() in production code |
| BUG-021 | MEDIUM | 4 LSP errors — forward references to unimplemented nexus_bus fns (not a regression) |
| BUG-022 | LOW | povm_data.db diverged from POVM Engine |
| BUG-023 | HIGH | SYNTHEX PID missing Kd — 87% thermal swings |
| BUG-024 | MEDIUM | Only 11/20 documented K7 commands work |
| BUG-025 | HIGH | Swarm orchestrator RunCommands permission denied |
| BUG-026 | MEDIUM | Zellij IPC broken pipe flood (BUG-18 recurrence) |

---

## Backlinks

- [[Pane-Vortex — Fleet Coordination Daemon]]
- [[Session 036 — Nvim-Bash Command Chaining Analysis]]
- [[Session 027 — Nvim Synergy and Tool Chaining]]
- [[Zellij Master Skill — Bootstrap Reference]]
- [[Zellij Navigation God-Tier — Session 035]]
- [[POVM Engine]]
- [[Vortex Sphere Brain-Body Architecture]]
- [[ULTRAPLATE — Integrated Master Plan V2]]
- [[Session 034d — Synthetic DevEnv Assessment]]
- [[Swarm Orchestrator v3.0 — IPC Bus Integration]]
- [[The Habitat — Naming and Philosophy]] — why The Habitat exists
- **/primehabitat skill** at `pane-vortex/.claude/skills/primehabitat/SKILL.md` — links back here
