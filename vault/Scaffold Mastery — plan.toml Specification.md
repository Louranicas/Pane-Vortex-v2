# Scaffold Mastery — plan.toml Specification

**Created:** Session 043 (2026-03-20)
**Binary:** `scaffold-gen --from-plan plan.toml <dir>`
**Skill:** `.claude/skills/scaffold-mastery/`

## Complete Format

```toml
[metadata]
name = "my-service"                    # Required: crate name
description = "Description"            # Required
version = "0.1.0"                      # Required: semver
edition = "2021"                       # Default: "2021"
rust_version = "1.75"                  # Default: "1.75"
authors = ["Dev <dev@example.com>"]    # Optional
license = "MIT"                        # Optional
repository = ""                        # Optional
port = 8200                            # Optional: server port
service_id = "my-service"              # Optional: devenv ID
devenv_batch = 3                       # Optional: startup batch

[dependencies]
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }  # Needed for async test_kind
axum = { version = "0.8", optional = true }

[features]
default = ["api"]
api = ["axum"]

[quality]
gate = ["check", "clippy", "pedantic", "test"]
min_tests_per_module = 50
deny_unwrap = true
deny_unsafe = true
k7_compliance = true                   # Adds agent dispatch to MASTERPLAN
hebbian_feedback = true                # Record patterns to RM

[consent]
sphere_can_add_modules = true
sphere_can_skip_layers = true
modulation_not_command = true          # "should" not "must" for non-safety
implementation_order = "recommended"   # recommended|required

[[layers]]
key = "L1"
dir_name = "m1_core"
name = "Core"
description = "Core types"
depends_on = []
feature_gate = "api"                   # Optional: feature gate this layer
rationale = "Why this layer exists"    # Optional: flows into ai_specs/

[[modules]]
layer = "L1"                           # Must match a layer key
name = "m01_types"
description = "Core types"
test_kind = "unit"                     # unit|integration|async|property
depends_on = ["m01_types"]             # Per-module dependencies
quality_override = "experimental"      # strict|standard|experimental

[[bin_targets]]
name = "my-service"
path = "src/bin/main.rs"
kind = "daemon"                        # daemon|client|probe|tool

[implementation]
order = ["L1", "L2"]                   # Layer build order

[config.server]                        # -> config/default.toml
bind_addr = "\"127.0.0.1\""
port = 8200
```

## Validation Rules

1. All `module.layer` values must match a `layer.key`
2. No duplicate layer keys
3. No duplicate module names
4. No circular layer dependencies (topological sort)
5. All `module.depends_on` refs must exist as module names
6. All `layer.depends_on` refs must exist as layer keys

## Test Kinds

| Kind | Scaffold | Requires |
|------|----------|----------|
| `unit` (default) | `#[test]` | Nothing |
| `integration` | `#[test]` | Nothing |
| `async` | `#[tokio::test]` | `tokio` in deps |
| `property` | `#[test]` with loop | Nothing |

If `async` specified but `tokio` not in deps, falls back to unit test silently.

## Generated Output

- `src/<layer.dir_name>/<module.name>.rs` — module scaffolds with test
- `src/<layer.dir_name>/mod.rs` — layer module declarations
- `src/lib.rs` — top-level layer imports
- `src/bin/<bin_target>.rs` — binary stubs (daemon/client/probe/tool)
- `Cargo.toml` — from deps, features, quality
- `bacon.toml` — quality gate jobs
- `CLAUDE.md` — dynamic architecture block, rules, quality config
- `MASTERPLAN.md` — implementation order, dependency graph, agent dispatch
- `README.md`, `CHANGELOG.md`, `CONTRIBUTORS.md`
- `ai_docs/` — architecture, code module map, per-layer docs
- `ai_specs/` — per-layer specs with rationale, patterns, anti-patterns
- `schematics/` — dynamic Mermaid dependency graph, API map stubs
- `config/` — default.toml, production.toml, devenv-service.toml
- `.claude/` — context.json, patterns.json, anti_patterns.json

## Links

- [[Session 043 — Plan-Driven Scaffold Engine]]
- [[The Habitat — Integrated Master Plan V3]]
