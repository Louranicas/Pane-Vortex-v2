# Workspace Tools Reference

## YAZI (File Navigator)
Tab 2 TopRight | `~/.config/yazi/yazi.toml` | Helix default opener

| Key | Action |
|-----|--------|
| z/Z | Zoxide/fzf jump |
| - | Parent dir |
| CR | Open file |
| gs/g. | Sort/toggle hidden |
| Space/v | Select/visual |
| d/y/p/r | Trash/yank/paste/rename |

## BTM (Process Monitor)
Tab 2 Bottom | `btm --regex_filter "pane-vortex|synthex|povm"`
Tab=cycle | /=search | t=tree | dd=kill | s=sort

## BACON (Continuous Compiler)
Tab 3 Left | `bacon.toml` in PV2 root
Jobs: check, clippy, pedantic, test, gate | Uses /tmp/cargo-pv2

## ATUIN (Shell History)
1,890 total, 721 unique | `atuin search --cwd <path> --limit 20`
SQLite: `~/.local/share/atuin/history.db`
