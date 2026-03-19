---
date: 2026-03-18
tags: [session-039, atuin, yazi, shell-history, file-manager, zsoe, keybindings, sqlite, analytics]
aliases: [Atuin Reference, Yazi Reference, Shell History Intelligence]
---

# Session 039 — Atuin and Yazi God-Tier Reference

> **Atuin 18.10 (shell history intelligence) + Yazi 25.5.31 (TUI file manager)**
> **ZSOE Location: Tab 2 (Workspace-1) — Atuin in Left, Yazi in TopRight**
> **Backlinks:** [[Session 039 — ZSDE Nvim God-Tier Command Reference]] | [[Session 039 — Lazygit God-Tier Command Reference]] | [[Pane-Vortex — Fleet Coordination Daemon]] | [[Zellij Master Skill — Bootstrap Reference]]

---

## 1. Atuin — Shell History Intelligence

| Property | Value |
|----------|-------|
| Version | 18.10.0 |
| Binary | `~/.atuin/bin/atuin` |
| Config | `~/.config/atuin/config.toml` |
| Database | `~/.local/share/atuin/history.db` (1.3MB SQLite) |
| Total commands | 1,889 |
| Unique commands | 721 |
| Sessions | 548 over 67.4 days |
| Store index | 1,320 entries |

### Interactive Search (Ctrl+R in shell)
| Mode | Description |
|------|-------------|
| fuzzy | Fuzzy matching (default) |
| prefix | Starts-with |
| fulltext | Substring |
| skim | Skim-style fuzzy |

### Filter Modes
| Filter | Scope |
|--------|-------|
| global | All history everywhere |
| host | Current hostname only |
| session | Current shell session |
| directory | Current working directory |
| workspace | Current git repository |

### CLI Commands
```bash
atuin search [query]             # Interactive search
atuin history list [--cmd-only]  # List history
atuin stats                      # Command frequency chart
atuin kv list                    # Key-value store
atuin store status               # Store sync status
atuin search --after '2026-03-18' --cmd-only  # Time-scoped
atuin search --exit 0            # Only successful
atuin search --filter-mode workspace  # Git repo scope
```

### Direct SQLite Queries (the power move)
```bash
# Atuin's DB is standard SQLite — query it directly
sqlite3 ~/.local/share/atuin/history.db

# Schema: id, timestamp, duration, exit, command, cwd, session, hostname

# Commands by hour of day
SELECT strftime('%H', datetime(timestamp/1000000000, 'unixepoch', 'localtime')) as hour,
       COUNT(*) FROM history GROUP BY hour ORDER BY hour;

# Average duration by command type
SELECT CASE WHEN command LIKE 'cargo%' THEN 'cargo'
            WHEN command LIKE 'claude%' THEN 'claude'
            ELSE 'other' END as type,
       COUNT(*), ROUND(AVG(duration)/1e9, 1) as avg_secs
FROM history GROUP BY type ORDER BY avg_secs DESC;

# Most used directories
SELECT cwd, COUNT(*) FROM history GROUP BY cwd ORDER BY COUNT(*) DESC LIMIT 10;

# Failed commands
SELECT command, exit FROM history WHERE exit != 0 ORDER BY timestamp DESC LIMIT 10;

# Service interaction history
SELECT command FROM history WHERE command LIKE '%localhost:8%' ORDER BY timestamp DESC;
```

### Operator Analytics Discovered
- Peak activity: **20:00** (201 commands)
- Claude sessions average **9.6 hours** (max 6.4 days)
- Zellij sessions average **12.3 hours**
- Top directories: home(746), workspace(502), pane-vortex(75)
- 457 total claude invocations, 47 fleet dispatches
- 75 service-related curl commands
- Peak days: March 14-15 (154-158 commands) — POVM bridge sessions

---

## 2. Yazi — TUI File Manager

| Property | Value |
|----------|-------|
| Version | 25.5.31 |
| Binary | `~/.local/bin/yazi` |
| Config | `~/.config/yazi/yazi.toml` |
| Keymap | `~/.config/yazi/keymap.toml` (183 lines) |
| Theme | `~/.config/yazi/theme.toml` |
| Init | `~/.config/yazi/init.lua` (custom Lua) |
| Default editor | Helix (`hx`) |
| Layout | `[1, 3, 4]` (sidebar:main:preview) |

### Navigation
| Key | Action |
|-----|--------|
| j/k | Down/up |
| h/l | Parent/enter |
| gg/G | Top/bottom |
| H/L | Half-page up/down |
| C-u/C-d | Page up/down |
| u / C-o | History back |
| C-i | History forward |

### File Operations
| Key | Action |
|-----|--------|
| o/O | Open / open interactive |
| y | Yank (copy) |
| x | Cut |
| p | Paste |
| d | Trash |
| D | Permanent delete |
| r | Rename |
| a | Create file |
| A | Create directory |

### Selection
| Key | Action |
|-----|--------|
| Space | Toggle selection |
| v | Visual mode |
| V | Visual mode (unset) |
| C-a | Select all |
| C-r | Invert selection |

### Search and Jump
| Key | Action |
|-----|--------|
| / | Filter current directory |
| f | Search files (fzf) |
| s | Search content (ripgrep) |
| z | Jump with zoxide |
| Z | Jump with fzf |

### Goto Shortcuts (custom)
| Key | Destination |
|-----|-------------|
| gh | Home (~) |
| gc | ~/.config |
| gd | ~/Downloads |
| gp | ~/projects |
| gt | /tmp |

### Tabs
| Key | Action |
|-----|--------|
| t | New tab |
| 1-6 | Switch to tab N |
| [/] | Prev/next tab |
| C-c | Close tab |

### View
| Key | Action |
|-----|--------|
| . | Toggle hidden files |
| S | Sort menu |
| w | Task manager |
| ~ | Help |
| Alt-j/k | Seek preview down/up |

### Lua Customizations (init.lua)
- `Status:name()` — shows hovered filename in status bar
- `Linemode:size()` — human-readable file sizes (B/K/M/G/T)
- `Header:host()` — user@hostname prefix in blue

---

## 3. ZSOE Integration Chains

### Atuin + Service APIs
```bash
# Mine all service interactions from history
sqlite3 ~/.local/share/atuin/history.db \
  "SELECT command FROM history WHERE command LIKE '%localhost:8%'" | head -20

# Find failed builds
atuin search cargo --exit 1 --cmd-only

# Workspace-scoped history (only pane-vortex commands)
cd ~/claude-code-workspace/pane-vortex && atuin search --filter-mode workspace
```

### Yazi + nvim + lazygit
```
Yazi (navigate to file) → o (open in Helix) → close →
lazygit (stage changed file) → commit → Y (post to RM)
```

### Atuin + PV Field
```bash
# Correlate command frequency with field activity
# (when are commands being run vs when is the field active?)
sqlite3 ~/.local/share/atuin/history.db \
  "SELECT strftime('%H', datetime(timestamp/1e9, 'unixepoch', 'localtime')), COUNT(*)
   FROM history GROUP BY 1" | \
  while read hour count; do
    echo "$hour: $count commands"
  done
```

---

## 4. ZSOE Tab 2 Layout

```
+────────────────+──────────────────────+
|                |    Yazi (TopRight)    |
|  Atuin (Left)  +──────────────────────+
|                |   btm (BotRight)     |
+────────────────+──────────────────────+
```

All three tools in Workspace-1 provide:
- **Atuin**: Command history search + analytics
- **Yazi**: Visual file navigation + preview
- **btm**: System resource monitoring

---

## 5. Deep Discoveries (Pioneer Loop)

### Bootstrap Ritual (command bigrams)
The operator's launch sequence, repeated 86-102 times:
```
cd ~/claude-code-workspace → python3 -m venv claude-code-env →
source claude-code-env/bin/activate → claude
```
Then: `alacritty → zellij` (38x) — terminal launch → multiplexer.

### Exit Code Archaeology
- `claude`: 54% fail rate (154/286) — non-zero on interrupt/compact (expected)
- `zellij`: 58% fail (45/78) — crash/interrupt
- `exit`: 100% fail (36/36) — captures PREVIOUS command's exit code
- `/exit`: 100% fail (24/24) — Claude Code exit returns non-zero

### Session Durations
Average session with 3+ commands: **15.8 hours**. Max: **153.6 hours** (6.4 days). Average 6 commands per session — most work happens inside Claude Code, not in the shell.

### March 15 Mega-Session (5-tool reconstruction)
- 158 shell commands, 10 fleet dispatches, 4 git commits
- 168 sphere registrations, 99 tasks submitted, 49 completed
- Commits: POVM bridge, executor, hooks, bridge simplification (84 files)
- Bootstrap: venv created 7 times (multiple terminal restarts)

### Command Complexity Evolution
- March 12-13: longest avg commands (212 chars) — scaffolding sessions
- March 18 (today): shortest avg (34 chars) — Claude Code does the typing

### Yazi Integration Capabilities
- `--chooser-file /tmp/chosen` → output selected files for nvim
- `--cwd-file /tmp/cwd` → output final directory on exit
- `--local-events rename,delete` → stream JSON events (pipe to PV sphere)
- Zoxide top frecent: pane-vortex (primary workspace confirmed)
- Catppuccin Mocha theme (113 lines, purple CWD, blue hover)

### Novel SQL Queries for Atuin
```sql
-- Command bigrams (what follows what)
WITH ordered AS (
  SELECT command, LAG(command) OVER (ORDER BY timestamp) as prev
  FROM history
)
SELECT substr(prev,1,20), substr(command,1,20), COUNT(*)
FROM ordered WHERE prev IS NOT NULL
GROUP BY 1, 2 HAVING COUNT(*) >= 5 ORDER BY 3 DESC;

-- Session duration analysis
WITH bounds AS (
  SELECT session, MIN(timestamp) as first, MAX(timestamp) as last, COUNT(*) as n
  FROM history GROUP BY session HAVING COUNT(*) >= 3
)
SELECT ROUND(AVG((last-first)/1e9/3600),1) as avg_hours,
       ROUND(MAX((last-first)/1e9/3600),1) as max_hours
FROM bounds;

-- Command complexity evolution
SELECT date(timestamp/1e9, 'unixepoch', 'localtime'),
       COUNT(*), ROUND(AVG(LENGTH(command)))
FROM history GROUP BY 1 ORDER BY 1 DESC LIMIT 10;
```

---

## Backlinks

- [[Session 039 — ZSDE Nvim God-Tier Command Reference]]
- [[Session 039 — Lazygit God-Tier Command Reference]]
- [[Session 039 — What I Learned]]
- [[Pane-Vortex — Fleet Coordination Daemon]]
- [[Zellij Master Skill — Bootstrap Reference]]
- [[The Habitat — Naming and Philosophy]] — why The Habitat exists
- **/primehabitat skill** at `pane-vortex/.claude/skills/primehabitat/SKILL.md` — links back here
