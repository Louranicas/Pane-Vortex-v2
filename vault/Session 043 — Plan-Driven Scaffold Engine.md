# Session 043 — Plan-Driven Scaffold Engine

**Date:** 2026-03-20
**POVM:** `6fb98218-8791-4020-a19d-6585786029da`
**Binary:** `scaffold-gen` (`src/bin/scaffold.rs`, 1456 LOC)
**Skill:** `scaffold-mastery`

## What Changed

`scaffold-gen` evolved from a fixed 8-layer/41-module generator into a **plan-driven scaffold engine** that reads `plan.toml` to generate custom project architectures.

### CLI

```bash
scaffold-gen --from-plan plan.toml <dir>   # Plan-driven scaffold
scaffold-gen <project-dir> <project-name>  # Default 8L/41M (backward compat)
scaffold-gen --verify <project-dir>        # Quality gate
```

### Architecture (747 -> 1456 LOC)

| Phase | What | LOC |
|-------|------|-----|
| 1 | Data model: Plan, Metadata, LayerDef, PlanModule, etc. | ~250 |
| 2 | Dynamic generators: module, mod.rs, lib.rs, Cargo.toml, binaries | ~200 |
| 3 | Dynamic documentation: CLAUDE.md, MASTERPLAN.md, ai_docs, ai_specs | ~300 |
| 4 | Dynamic Mermaid: layer dependency graph from plan | ~60 |
| 5 | CLI: --from-plan, backward compat, --help update | ~40 |
| 6 | Quality gate + skill update + arena testing | verification |

## MVP Features Integrated

### Gap Analysis (5 of 12)

| GAP | Feature | Status |
|-----|---------|--------|
| GAP-1 | README.md generation | DONE |
| GAP-2 | Per-module dependencies in docs | DONE |
| GAP-3 | Test kinds: unit/integration/async/property | DONE |
| GAP-9 | CHANGELOG.md generation | DONE |
| GAP-11 | Quality thresholds from plan | DONE |

### Non-Anthropocentric (5 of 7)

| NA-GAP | Feature | Status |
|--------|---------|--------|
| NA-1 | Consent: "should" vs "must" language | DONE |
| NA-3 | Sphere attribution stubs in modules | DONE |
| NA-5 | Bridge consent check stubs | DONE |
| NA-6 | Layer rationale in ai_specs | DONE |
| NA-7 | "Recommended" not "Required" order | DONE |

### DevOps Synergy

- K7 compliance flag adds agent dispatch strategy to MASTERPLAN
- Dynamic Mermaid dependency graphs from plan topology

## plan.toml Format

See [[Scaffold Mastery — plan.toml Specification]] for complete format.

Key sections: `[metadata]`, `[dependencies]`, `[features]`, `[quality]`, `[consent]`, `[[layers]]`, `[[modules]]`, `[[bin_targets]]`, `[implementation]`, `[config.*]`

## Arena Test Results

| Test | Layers | Modules | Quality Gate |
|------|--------|---------|-------------|
| Default (backward compat) | 8 | 41 | 4/4 PASS |
| Small plan | 2 | 5 | 4/4 PASS |
| Medium plan (features, consent, k7) | 5 | 20 | 4/4 PASS |

## Natural Language Plans

No code changes needed. Conversational workflow:
1. User describes project in plain English
2. Claude generates `plan.toml`
3. Claude runs `scaffold-gen --from-plan`

Works because Claude already speaks both languages.

## Tensor Encoding

```
[1456, 747, 3, 41, 20, 5, 1379, 117, 12, 7, 4, 1]
  |     |   |   |   |  |   |     |    |   |  |  |
  |     |   |   |   |  |   |     |    |   |  |  intensity
  |     |   |   |   |  |   |     |    |   |  quality_gate_stages
  |     |   |   |   |  |   |     |    |   na_gaps_integrated
  |     |   |   |   |  |   |     |    gaps_integrated
  |     |   |   |   |  |   |     files_generated_default
  |     |   |   |   |  |   tests_passing
  |     |   |   |   |  modules_small_test
  |     |   |   |   modules_medium_test
  |     |   |   modules_default
  |     |   cli_paths
  |     loc_before
  loc_after
```

## Links

- [[The Habitat — Integrated Master Plan V3]]
- [[ULTRAPLATE Master Index]]
- [[Session 041 — V2 Full Stack Implementation]]
- [[Scaffold Mastery — plan.toml Specification]]

## Key Decisions

1. **Single file** — all 1456 lines in `scaffold.rs`. No new dependencies.
2. **Backward compat first** — `default_plan()` produces identical output to old hardcoded code.
3. **Async test fallback** — if tokio not in deps, async test_kind silently falls back to unit.
4. **Feature gates auto-generated** — layer `feature_gate` values auto-added to `[features]` if no explicit features section.
5. **Consent is opt-in** — only activated via `[consent]` section in plan.toml.

## Phase 2-3 Future Work

| Feature | CLI Flag | Description |
|---------|----------|-------------|
| Extract plan from codebase | `--extract <dir>` | Reverse-engineer plan.toml from existing project |
| Diff/upgrade path | `--diff plan.toml <dir>` | Show changes without overwriting |
| Hooks generation | `[hooks]` in plan.toml | Session start/end stubs |
| K7 integration | `--with-k7` | K7 compliance + lint after generation |
| Workspace support | `[[sub_crates]]` | Multi-crate workspaces |
| Vault integration | `[vault]` | Obsidian vault links in docs |
