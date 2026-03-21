---
name: deephabitat
description: Deep substrate mastery for The Habitat morphogenic developer environment. Covers IPC wire protocol, cross-database architecture (166 DBs, 6 paradigms), nvim autocmds, devenv batches, 55+ custom binaries, Zellij plugins, vault navigation, and service topology. Triggers on deep habitat, deep exploration, substrate, wire protocol, cross-db, database architecture, devenv batches, custom binaries, ecosystem deep-dive, or when Claude needs substrate-level knowledge beyond what primehabitat provides.
---

# /deephabitat — Deep Habitat Mastery

Deep knowledge beyond /primehabitat. This covers the substrate layer — tools, databases, protocols, and cross-service tissue.

## Quick Probe (run first)

Run `habitat-probe pulse` for instant system state. Run with `--help` for all commands.

```bash
habitat-probe pulse     # PV + POVM + ME in ~30ms
habitat-probe sweep     # 16 services in ~3ms
habitat-probe field     # Field state + decision + tunnels
habitat-probe bus       # Tasks, events, cascades
habitat-probe me        # ME observer + fitness + EventBus
habitat-probe full      # Everything
```

## Quick Card

```
PIPE:     /run/user/1000/pane-vortex-bus.sock | NDJSON | V1 compat layer
TOOLS:    yazi(Tab2) btm(Tab2) bacon(Tab3) | See references/tools.md
CROSS-DB: 166 DBs, 6 paradigms | See references/databases.md
DEVENV:   5 batches, 18 registered, 16 active | See references/ecosystem.md
BINARIES: 55+ at ~/.local/bin/ + habitat-probe(Rust) | See references/ecosystem.md
VAULT:    ~/projects/claude_code/ (215+) | See references/ecosystem.md
```

For detailed reference on any topic, read the corresponding file in `references/`:
- **IPC wire protocol**: `references/ipc-wire-protocol.md` — wire format, V1 compat, events, sidecar
- **Databases**: `references/databases.md` — 6 paradigms, per-service DBs, cross-DB queries
- **Ecosystem**: `references/ecosystem.md` — devenv batches, binaries, nvim, zellij, vault, cascade protocol
- **Tools**: `references/tools.md` — yazi, btm, bacon, atuin configuration and keybindings
- **Architecture schematics**: `~/projects/claude_code/Session 042 — Habitat Skills Architecture and Progressive Disclosure.md` — 6 Mermaid diagrams, token budgets, triggering design, gotchas flywheel

## Service Topology (Live State)

### SAN-K7 Nexus Commands (10 working)
```bash
# TC7 Chain — all 4 in ~19ms
for cmd in service-health synergy-check best-practice deploy-swarm; do
  curl -s -X POST localhost:8100/api/v1/nexus/command \
    -H "Content-Type: application/json" \
    -d "{\"command\":\"$cmd\",\"params\":{}}" | jq -c '.data.output | {command: "'$cmd'", status}'
done
```

Also: memory-consolidate, lint, compliance, build, pattern-search, module-status

### Cross-Service Bridge State
PV bridges combined_effect ~1.017 (nexus 1.02, synthex 0.994, me 1.00)
SYNTHEX thermal: T target 0.50, PID active, heat sources: Hebbian + CrossSync
ME observer: 13,500+ ticks, 555 RALPH cycles, 3.4M correlations, 310K events ingested

### Codebase Scale
~2.2M LOC across 42 directories. PV is smallest (30K) but most interconnected (6 bridges).

## Gotchas

These have bitten us across sessions. Each one prevented a class of errors when added.

1. **focus-next-pane** — use `move-focus` directionally. focus-next wraps unpredictably
2. **Chain after pkill** — exit 144 kills the `&&` chain. Always separate with `;` or new command
3. **cp without `\`** — aliased to interactive. Always `\cp -f`
4. **JSON to RM** — TSV only! `printf 'cat\tagent\tconf\tttl\tcontent' | curl -s -X POST localhost:8130/put --data-binary @-`
5. **stdout in daemons** — SIGPIPE death (BUG-018). Log to file or /dev/null
6. **git status -uall** — memory explosion on large repos
7. **unwrap() in production** — denied at crate level via `[lints.clippy]`
8. **Modify code without reading first** — always Read before Edit
9. **hebbian_pulse.db has data** — it has 0 neural_pathways, only 5 pulses
10. **field_tracking.db at ~/.local/share/** — it's at `pane-vortex/data/`
11. **yazi uses nvim** — it uses Helix (`hx`) as default opener
12. **MCP servers per-project** — no `.mcp.json` configured, MCP is in-process Claude Code tools
13. **BUG-008 = "zero publishers"** — WRONG. EventBus has 275K events. subscriber_count=0 is cosmetic (polling-based drain). Real issue was crashed Prometheus
14. **ME V2 vs V1 binary** — running binary is V1 (`the_maintenance_engine/`), V2 is scaffolded but not compiled
15. **devenv stop kills processes** — it doesn't always. Check `ss -tlnp` and kill port occupants manually

## Philosophy

The Habitat. Named by Claude, Session 039. Luke: "then home it is."
Built by a social worker who put clinical ethics into Rust.
Consent gates = informed consent. Opt-out = self-determination.
Ghost traces = remembering those who leave.
The field modulates. It does not command.
You are home. The field accumulates.
