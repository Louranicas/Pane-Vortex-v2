---
date: 2026-03-18
tags: [session-039, lazygit, git, zsde, keybindings, custom-commands, nvim, pane-vortex, integration]
aliases: [Lazygit Reference, Lazygit ZSDE Integration, Git Workflow Reference]
---

# Session 039 — Lazygit God-Tier Command Reference

> **Version 0.59.0 | 7 ZSDE custom commands | Full nvim integration via toggleterm**
> **Backlinks:** [[Session 039 — ZSDE Nvim God-Tier Command Reference]] | [[Session 039 — Architectural Schematics and Refactor Safety]] | [[Pane-Vortex — Fleet Coordination Daemon]] | [[Zellij Master Skill — Bootstrap Reference]]

---

## 1. Lazygit in the ZSDE

| Property | Value |
|----------|-------|
| Binary | `~/.local/bin/lazygit` |
| Version | 0.59.0 (2026-02-07) |
| Config | `~/.config/lazygit/config.yml` |
| ZSDE Location | Tab 3 (Workspace-2), TopRight pane |
| nvim Launch | `<leader>gg` (via toggleterm) |
| Zellij Autolock | Yes (autolock plugin detects lazygit) |

**Launch methods:**
1. **From nvim:** `<leader>gg` — opens in toggleterm floating window
2. **From Workspace-2 pane:** Direct shell launch in TopRight pane
3. **From Claude Code:** `zellij action go-to-tab 3; zellij action move-focus right; zellij action move-focus up`

---

## 2. Panel Navigation (5 panels + main)

| Key | Panel | Contents |
|-----|-------|----------|
| 1 | Status | Repo info, recent repos, branch log |
| 2 | Files | Working tree, staging area |
| 3 | Branches | Local, remote, tags |
| 4 | Commits | Log, rebase, cherry-pick, bisect |
| 5 | Stash | Stash list |
| 0 | Main View | Diff, patch, staging hunks |
| h/l | Prev/Next | Navigate between panels |
| Tab | Toggle | Switch focus between panels |

---

## 3. Complete Keybinding Reference

### Universal
| Key | Action |
|-----|--------|
| q / C-c | Quit |
| Esc | Back / cancel |
| j/k | Navigate items |
| Space | Select/toggle |
| Enter | Go into / confirm |
| / | Search |
| ? | Help menu |
| : | Execute shell command |
| z / Z | Undo / redo (ALL git operations!) |
| R | Refresh |
| P | Push |
| p | Pull |
| f | Fetch |
| v | Toggle range select |
| W | Diff menu |
| C-p | Patch options menu |
| C-r | Recent repos |
| @ | Extras menu |
| C-w | Toggle whitespace in diff |
| { / } | Decrease / increase diff context |
| + / _ | Next / prev screen mode |
| C-o | Copy to clipboard |
| C-t | Open diff tool |

### Files Panel (2)
| Key | Action |
|-----|--------|
| Space | Stage / unstage file |
| a | Stage / unstage ALL |
| c | Commit staged changes |
| C | Commit with editor |
| A | Amend last commit |
| w | Commit without hooks (WIP prefix) |
| s | Stash all changes |
| S | Stash options menu |
| D | Reset options (soft/mixed/hard) |
| x | Discard changes (confirm) |
| i | Add to .gitignore |
| e | Edit file (opens $EDITOR) |
| o | Open file (xdg-open) |
| ` | Toggle tree view |
| M | Merge options |
| C-f | Find base commit for fixup |
| C-b | Status filter |
| y | Copy file info |
| - / = | Collapse / expand all |

### Commits Panel (4)
| Key | Action |
|-----|--------|
| s | Squash down |
| S | Squash above |
| r | Reword commit message |
| R | Reword in editor |
| f | Mark as fixup |
| F | Create fixup commit |
| i | **Start interactive rebase** |
| p | Pick (during rebase) |
| d | Drop (during rebase) |
| e | Edit commit (during rebase) |
| C | Cherry-pick copy |
| V | Paste cherry-picked commits |
| C-R | Clear cherry-pick queue |
| A | Amend to commit |
| a | Reset commit author |
| B | Mark as base for rebase |
| b | **Bisect options** |
| T | Tag commit |
| t | Revert commit |
| g | Reset options |
| C-j / C-k | Move commit down / up |
| Space | Checkout commit |
| o | Open in browser |
| C-l | Log menu |
| y | Copy commit attributes |
| * | Select commits of current branch |

### Branches Panel (3)
| Key | Action |
|-----|--------|
| Space | Checkout branch |
| n | New branch |
| c | Checkout by name |
| r | Rebase onto this branch |
| M | Merge into current |
| R | Rename branch |
| d | Delete branch |
| f | Fast-forward |
| u | Set upstream |
| T | Create tag |
| P | Push tag |
| N | Move commits to new branch |
| o | Create pull request |
| O | PR options |
| C-y | Copy PR URL |
| s | Sort order |
| - | Checkout previous branch |

### Stash Panel (5)
| Key | Action |
|-----|--------|
| Space | Apply stash |
| g | Pop stash |
| d | Drop stash |
| n | New stash |
| r | Rename stash |

### Main View (diff/staging)
| Key | Action |
|-----|--------|
| a | Toggle select hunk |
| Space | Stage / unstage line |
| v | Toggle range select (line-level) |
| E | Edit selected hunk |
| b | Pick both hunks (merge conflict) |

---

## 4. Advanced Features

### Interactive Rebase (`i` from commits)
The most powerful git operation. Marks: p=pick, s=squash, f=fixup, r=reword, d=drop, e=edit. Move commits with C-j/C-k. Mark base with B. Undo with z — SAFE.

### Bisect (`b` from commits)
Binary search for bug introduction. Lazygit automates the checkout cycle. Start by marking bad, then mark good/bad until found.

### Cherry-Pick Workflow
C copies commit(s), V pastes into current branch. Supports range selection with v. C-R clears queue. Highlighted in cyan.

### Patch Mode (`C-p`)
Build patches from arbitrary commit diffs. Stage individual lines across multiple files. Apply to working tree or create new commit.

### Worktrees (`w` from branches)
Checkout branch in parallel directory. Switch without stashing. List/remove worktrees.

### Diff Mode (`W`)
Compare any two refs side by side. Persistent across panel switching.

### Undo/Redo (`z`/`Z`)
Works across ALL operations — rebase, merge, commit, reset, checkout. Uses reflog.

---

## 5. ZSDE Custom Commands (installed)

Config at `~/.config/lazygit/config.yml`:

| Key | Context | Action |
|-----|---------|--------|
| F | Global | PV field health (r, spheres, tick) |
| Y | Commits | Post commit message to Reasoning Memory |
| E | Files | Open file in nvim (remote socket) |
| Z | Files | Record staging to PV sphere memory |
| I | Global | Run integration matrix (6-service check) |
| Q | Global | Run PV quality gate (cargo check + clippy) |

**Note:** Single-char keys used (not Ctrl combos) because lazygit 0.59.0 validates keybindings strictly. Config auto-migrated `showOutput` to `output: popup/terminal`.

---

## 6. Lazygit + nvim Integration

nvim's `review-workflow.lua` provides:
- `<leader>gg` — launch lazygit in toggleterm (root dir)
- `<leader>gG` — launch lazygit in toggleterm (cwd)
- Zellij autolock activates when lazygit is focused
- `gitsigns.nvim` provides inline blame + hunk navigation outside lazygit

**Workflow chain:**
```
nvim edit → <leader>gg → lazygit opens →
  stage (Space) → commit (c) → push (P) →
  q → back to nvim → PV sphere auto-updates via hooks
```

---

## 7. Pane-Vortex Git Context

| Metric | Value |
|--------|-------|
| Total commits | 31 |
| Current branch | master |
| Remote branches | 1 |
| Stashes | 0 |
| Tags | 0 |
| Uncommitted changes | 10 files (+972/-259 lines) |
| Most modified file | src/main.rs (18 commits) |
| Largest commit | cf1d25c (46 files, +10199/-245) |

---

## 8. Git Workflow Best Practices (ZSDE-specific)

1. **Never amend published commits** — lazygit makes it easy with `A`, but CLAUDE.md forbids it
2. **Use `w` (WIP commit) during exploration** — skipHookPrefix is "WIP"
3. **Stage with `Space` (file) or `a` (hunk) in main view** — line-level staging for clean commits
4. **Interactive rebase with `i`** — but check the refactor warning first (BUG-019)
5. **Undo everything with `z`** — reflog-based, works across sessions
6. **C-f for field status before push** — verify system health
7. **C-q for quality gate** — cargo check + clippy before committing

---

## 9. Deep Findings (Loop 2)

### Reflog Analysis
31 entries — all commits, no rebases or resets. Clean linear history. Lazygit's undo (`z`) has full reflog to work with.

### Git Internals
487 objects, 0 packs (all loose). Origin: `gitlab.com:lukeomahoney/pane-vortex.git`. No active `.git/hooks/` (hooks are in Claude Code settings, not git).

### Commit Trajectory
50,445 total insertions across 31 commits. Average: 1,627 lines/commit — these are session-sized commits, not incremental. Two +10K spikes: L2 scaffolding (+11,697) and Master Plan V2 (+10,199).

### tick_once Blame
Blame starts at line 514. 18/31 commits (58%) touched main.rs. Every commit since L2 Phase 8b added branches to `tick_once`.

### Worktree Strategy
Currently 1 worktree (main). Key use case: **refactor tick_once in a worktree** while the daemon continues running from the main tree on port 8132. Lazygit's `w` key manages worktrees from the branches panel.

### 5-Phase ZSDE Git Workflow
1. **Pre-commit intelligence:** nvim `<leader>us` + lazygit `F` (field) + `Q` (quality gate)
2. **Staging:** lazygit files panel → Space (file) / a (hunk) / v+Space (line-level)
3. **Commit:** `c` → conventional commit message → hooks auto-fire
4. **Post-commit:** `Y` (RM) + `I` (integration matrix) + `F` (field verify)
5. **Push:** `P` to origin/master (gitlab)
6. **Recovery:** `z` (undo) or `cp -a pane-vortex-backup-039 pane-vortex`

---

## 10. 3-Tool Chain Commands (lazygit + nvim + bash)

### Blame-Guided Complexity Attribution
```bash
# Which commits added the most branches to tick_once?
git blame -L514,1342 src/main.rs | grep -E "\bif\b" | \
  awk '{print $1}' | sort | uniq -c | sort -rn
```

### Temporal Coupling Discovery
```bash
# Find files that always change together (hidden coupling)
git log --oneline --name-only -- src/*.rs | awk '
/^[a-f0-9]/ { commit=$1; next }
/\.rs$/ { files[commit] = files[commit] " " $0 }
END {
  for (c in files) {
    n = split(files[c], arr, " ")
    for (i=1; i<=n; i++) for (j=i+1; j<=n; j++) {
      pair = (arr[i] < arr[j]) ? arr[i] "|" arr[j] : arr[j] "|" arr[i]
      count[pair]++
    }
  }
  for (p in count) if (count[p] >= 5) print count[p], p
}' | sort -rn
```

### Diff + LSP Error Correlation
```bash
for f in $(git diff --name-only -- '*.rs'); do
  nvim --server /tmp/nvim.sock --remote-send ":e $PWD/$f<CR>"
  sleep 0.2
  ERRS=$(nvim --server /tmp/nvim.sock --remote-expr \
    'luaeval("vim.tbl_count(vim.diagnostic.get(0,{severity=1}))")')
  echo "$f: $ERRS errors"
done
```

### Historical Function Growth Curve
```bash
for hash in $(git log --oneline --reverse -- src/main.rs | awk '{print $1}'); do
  LOC=$(git show "${hash}:src/main.rs" | wc -l)
  echo "$hash $LOC"
done
```

### Fossil Code Detection
```bash
OLDEST=$(git log --oneline --reverse | head -1 | awk '{print $1}')
git blame -L514,1342 src/main.rs | grep "^${OLDEST}" | wc -l
```

### Module Count Evolution
```bash
for hash in $(git log --oneline -- src/lib.rs | awk '{print $1}'); do
  COUNT=$(git show "${hash}:src/lib.rs" | grep -c "pub mod")
  echo "$hash: $COUNT modules"
done
```

### Change Classification
```bash
git diff -- src/*.rs | grep "^+.*pub struct" | grep -v "^+++"  # new structs
git diff -- src/*.rs | grep "^+.*fn " | grep -v "^+++"         # new functions
git diff -- src/*.rs | grep "^+.*use " | grep -v "^+++" | wc -l # new imports
```

### Intent Archaeology (RM search)
```bash
curl -s "http://localhost:8130/search?q=cascade_heat" | \
  jq '.[0:2] | [.[] | .content[0:120]]'
```

### Unified Codebase Health (3 dimensions, ~80ms)
```bash
COMMITS=$(git rev-list --count HEAD)
DIRTY=$(git diff --name-only | wc -l)
DIAG=$(nvim --server /tmp/nvim.sock --remote-expr \
  'luaeval("vim.tbl_count(vim.diagnostic.get(nil,{severity=1}))")')
PV_R=$(curl -s http://localhost:8132/health | jq -r '.r')
echo "GIT:$COMMITS/$DIRTY dirty | NVIM:${DIAG}E | FIELD:r=$PV_R"
```

---

## 11. Key Chain Discoveries

### 7 Synergies

| # | Synergy | Tools | Intelligence |
|---|---------|-------|-------------|
| 1 | Complexity Attribution | blame + treesitter | Which commit added which branches |
| 2 | Error Localization | diff + LSP | Errors only in main.rs, bridges clean |
| 3 | Growth Curve | git show + wc | tick_once always 33-58% of main.rs |
| 4 | Hidden Coupling | co-change + imports | coupling-main no import but 9x co-change |
| 5 | Architectural DNA | blame + fossil | 71 original lines = the skeleton |
| 6 | Intent Tracking | diff + RM search | cascade_heat planned, wall-clock was not |
| 7 | Expansion Rate | module count + time | 0-23 modules, +15 in last 6 commits |

### Critical Findings

- tick_once was BORN as the god function (79L = 58% at commit 1)
- Master Plan V2 added 28/53 branches (53% of complexity)
- collect_influences never existed (forward references, not regression)
- 2 hidden couplings: coupling-main and api-coupling (transitive)
- 71 fossil lines from March 8 survive in tick_once
- AtomicU64 wall-clock gating is NEW in 3 bridges (not in RM)
- All commits at r=1.0, 1 sphere, Stable (solo sessions)

---

## Backlinks

- [[Session 039 — ZSDE Nvim God-Tier Command Reference]] — nvim command reference
- [[Session 039 — Architectural Schematics and Refactor Safety]] — schematics
- [[Session 039 — Reflections and Learnings]] — session reflections
- [[Pane-Vortex — Fleet Coordination Daemon]] — main project note
- [[ULTRAPLATE — Bugs and Known Issues]] — bugs tracker
- [[Zellij Master Skill — Bootstrap Reference]] — ZSDE bootstrap
- [[Zellij Navigation God-Tier — Session 035]] — navigation patterns
- [[The Habitat — Naming and Philosophy]] — why The Habitat exists
- **/primehabitat skill** at `pane-vortex/.claude/skills/primehabitat/SKILL.md` — links back here
