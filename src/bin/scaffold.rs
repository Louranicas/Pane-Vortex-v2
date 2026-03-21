//! Scaffold generator — plan-driven Rust microservice scaffold engine.
//!
//! Reads a `plan.toml` to generate custom scaffolds, or falls back to the
//! default 8-layer, 41-module structure for backward compatibility.
//!
//! Produces clippy-pedantic-clean code with zero warnings.
//! Used by the `scaffold-mastery` skill as a black-box script.
//!
//! ## Usage
//!
//! ```bash
//! scaffold-gen --from-plan plan.toml <project-dir>   # Plan-driven scaffold
//! scaffold-gen <project-dir> <project-name>           # Default 8L/41M scaffold
//! scaffold-gen --verify <project-dir>                 # Verify existing scaffold
//! scaffold-gen --help                                 # Show help
//! ```

use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

// ═══════════════════════════════════════════════════════════════
// Phase 1: Plan Data Model
// ═══════════════════════════════════════════════════════════════

/// Top-level plan parsed from `plan.toml`.
#[derive(Debug, Deserialize)]
struct Plan {
    metadata: Metadata,
    #[serde(default)]
    dependencies: BTreeMap<String, DepSpec>,
    features: Option<BTreeMap<String, toml::Value>>,
    quality: Option<QualityConfig>,
    consent: Option<ConsentConfig>,
    bin_targets: Option<Vec<BinTarget>>,
    layers: Vec<LayerDef>,
    modules: Vec<PlanModule>,
    implementation: Option<ImplOrder>,
    config: Option<BTreeMap<String, BTreeMap<String, toml::Value>>>,
}

/// Project metadata.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Metadata {
    name: String,
    description: String,
    version: String,
    edition: Option<String>,
    rust_version: Option<String>,
    authors: Option<Vec<String>>,
    license: Option<String>,
    repository: Option<String>,
    port: Option<u16>,
    service_id: Option<String>,
    devenv_batch: Option<u8>,
}

/// Crate dependency specification.
#[derive(Debug, Deserialize)]
struct DepSpec {
    version: String,
    features: Option<Vec<String>>,
    optional: Option<bool>,
}

/// Quality gate configuration.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct QualityConfig {
    gate: Option<Vec<String>>,
    min_tests_per_module: Option<u32>,
    deny_unwrap: Option<bool>,
    deny_unsafe: Option<bool>,
    k7_compliance: Option<bool>,
    k7_compliance_target: Option<f64>,
    hebbian_feedback: Option<bool>,
}

/// Consent configuration (NA-GAP-1).
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ConsentConfig {
    sphere_can_add_modules: Option<bool>,
    sphere_can_skip_layers: Option<bool>,
    modulation_not_command: Option<bool>,
    implementation_order: Option<String>,
}

/// Binary target definition.
#[derive(Debug, Deserialize)]
struct BinTarget {
    name: String,
    path: String,
    kind: Option<String>,
}

/// Layer definition — maps to a `src/` subdirectory.
#[derive(Debug, Deserialize)]
struct LayerDef {
    key: String,
    dir_name: String,
    name: String,
    description: String,
    depends_on: Vec<String>,
    feature_gate: Option<String>,
    rationale: Option<String>,
}

/// Module definition within a layer.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PlanModule {
    layer: String,
    name: String,
    description: String,
    test_kind: Option<String>,
    depends_on: Option<Vec<String>>,
    quality_override: Option<String>,
}

/// Implementation ordering recommendation.
#[derive(Debug, Deserialize)]
struct ImplOrder {
    order: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════
// Phase 1: Plan Loading & Validation
// ═══════════════════════════════════════════════════════════════

/// Load and parse a `plan.toml` file.
fn load_plan(path: &Path) -> Result<Plan, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    toml::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {e}", path.display()))
}

/// Validate plan consistency:
/// - All module layer refs match a `LayerDef.key`
/// - No duplicate layer keys or module names
/// - No circular layer dependencies (topological sort)
/// - All per-module `depends_on` refs exist as module names
fn validate_plan(plan: &Plan) -> Result<(), String> {
    // Collect layer keys
    let layer_keys: Vec<&str> = plan.layers.iter().map(|l| l.key.as_str()).collect();

    // Check for duplicate layer keys
    let mut seen_keys = std::collections::HashSet::new();
    for key in &layer_keys {
        if !seen_keys.insert(*key) {
            return Err(format!("Duplicate layer key: {key}"));
        }
    }

    // Check for duplicate module names
    let mut seen_modules = std::collections::HashSet::new();
    for m in &plan.modules {
        if !seen_modules.insert(m.name.as_str()) {
            return Err(format!("Duplicate module name: {}", m.name));
        }
    }

    // Validate module layer refs
    for m in &plan.modules {
        if !layer_keys.contains(&m.layer.as_str()) {
            return Err(format!(
                "Module {} references unknown layer: {}", m.name, m.layer
            ));
        }
    }

    // Validate per-module depends_on refs
    for m in &plan.modules {
        if let Some(deps) = &m.depends_on {
            for dep in deps {
                if !seen_modules.contains(dep.as_str()) {
                    return Err(format!(
                        "Module {} depends on unknown module: {dep}", m.name
                    ));
                }
            }
        }
    }

    // Validate layer depends_on refs
    for layer in &plan.layers {
        for dep in &layer.depends_on {
            if !layer_keys.contains(&dep.as_str()) {
                return Err(format!(
                    "Layer {} depends on unknown layer: {dep}", layer.key
                ));
            }
        }
    }

    // Topological sort to detect circular dependencies
    topo_sort_layers(plan)?;

    Ok(())
}

/// Kahn's algorithm for topological sort of layers.
/// Returns sorted layer keys or error if cycle detected.
fn topo_sort_layers(plan: &Plan) -> Result<Vec<String>, String> {
    let mut in_degree: BTreeMap<&str, usize> = BTreeMap::new();
    let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();

    for layer in &plan.layers {
        in_degree.entry(layer.key.as_str()).or_insert(0);
        adjacency.entry(layer.key.as_str()).or_default();
        for dep in &layer.depends_on {
            adjacency.entry(dep.as_str()).or_default().push(layer.key.as_str());
            *in_degree.entry(layer.key.as_str()).or_insert(0) += 1;
        }
    }

    let mut queue: Vec<&str> = in_degree.iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&k, _)| k)
        .collect();
    queue.sort_unstable(); // deterministic order

    let mut sorted = Vec::new();
    while let Some(node) = queue.pop() {
        sorted.push(node.to_string());
        if let Some(neighbors) = adjacency.get(node) {
            for &neighbor in neighbors {
                if let Some(deg) = in_degree.get_mut(neighbor) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push(neighbor);
                        queue.sort_unstable();
                    }
                }
            }
        }
    }

    if sorted.len() != plan.layers.len() {
        return Err("Circular dependency detected among layers".to_string());
    }

    Ok(sorted)
}

/// Construct the default 8-layer, 41-module plan (backward compatibility).
#[allow(clippy::too_many_lines)]
fn default_plan(name: &str) -> Plan {
    let layers = vec![
        LayerDef {
            key: "L1".into(), dir_name: "m1_foundation".into(),
            name: "Foundation".into(),
            description: "Core types, errors, config, constants, traits, validation".into(),
            depends_on: vec![], feature_gate: None, rationale: None,
        },
        LayerDef {
            key: "L2".into(), dir_name: "m2_services".into(),
            name: "Services".into(),
            description: "Registry, health, lifecycle, API server".into(),
            depends_on: vec!["L1".into()], feature_gate: None, rationale: None,
        },
        LayerDef {
            key: "L3".into(), dir_name: "m3_field".into(),
            name: "Field".into(),
            description: "Sphere, field state, chimera, messaging, app state".into(),
            depends_on: vec!["L1".into()], feature_gate: None, rationale: None,
        },
        LayerDef {
            key: "L4".into(), dir_name: "m4_coupling".into(),
            name: "Coupling".into(),
            description: "Network, auto-K, topology".into(),
            depends_on: vec!["L1".into(), "L3".into()], feature_gate: None, rationale: None,
        },
        LayerDef {
            key: "L5".into(), dir_name: "m5_learning".into(),
            name: "Learning".into(),
            description: "Hebbian STDP, buoy network, memory manager".into(),
            depends_on: vec!["L1".into(), "L3".into(), "L4".into()],
            feature_gate: None, rationale: None,
        },
        LayerDef {
            key: "L6".into(), dir_name: "m6_bridges".into(),
            name: "Bridges".into(),
            description: "6 service bridges + consent gate".into(),
            depends_on: vec!["L1".into(), "L3".into()], feature_gate: None, rationale: None,
        },
        LayerDef {
            key: "L7".into(), dir_name: "m7_coordination".into(),
            name: "Coordination".into(),
            description: "IPC bus, conductor, executor, cascade, tick, persistence".into(),
            depends_on: vec!["L1".into(), "L3".into(), "L5".into(), "L6".into()],
            feature_gate: None, rationale: None,
        },
        LayerDef {
            key: "L8".into(), dir_name: "m8_governance".into(),
            name: "Governance".into(),
            description: "Proposals, voting, consent, sovereignty, evolution".into(),
            depends_on: vec!["L1".into(), "L3".into(), "L7".into()],
            feature_gate: None, rationale: None,
        },
    ];

    let module_defs: &[(&str, &str, &str)] = &[
        ("L1", "m01_core_types", "Core types, identifiers, newtypes"),
        ("L1", "m02_error_handling", "Error enum, Result alias"),
        ("L1", "m03_config", "Config from TOML + env"),
        ("L1", "m04_constants", "Named constants and thresholds"),
        ("L1", "m05_traits", "Core traits"),
        ("L1", "m06_validation", "Input validators"),
        ("L2", "m07_service_registry", "Service registry"),
        ("L2", "m08_health_monitor", "Circuit breaker"),
        ("L2", "m09_lifecycle", "Service FSM"),
        ("L2", "m10_api_server", "HTTP server"),
        ("L3", "m11_sphere", "Sphere entity"),
        ("L3", "m12_field_state", "Field computation"),
        ("L3", "m13_chimera", "Phase-gap detection"),
        ("L3", "m14_messaging", "Inter-sphere messages"),
        ("L3", "m15_app_state", "Shared state"),
        ("L4", "m16_coupling_network", "Coupling matrix"),
        ("L4", "m17_auto_k", "Adaptive coupling"),
        ("L4", "m18_topology", "Topology analysis"),
        ("L5", "m19_hebbian_stdp", "Hebbian STDP"),
        ("L5", "m20_buoy_network", "Buoy health"),
        ("L5", "m21_memory_manager", "Memory aggregation"),
        ("L6", "m22_synthex_bridge", "SYNTHEX bridge"),
        ("L6", "m23_nexus_bridge", "Nexus bridge"),
        ("L6", "m24_me_bridge", "ME bridge"),
        ("L6", "m25_povm_bridge", "POVM bridge"),
        ("L6", "m26_rm_bridge", "RM bridge"),
        ("L6", "m27_vms_bridge", "VMS bridge"),
        ("L6", "m28_consent_gate", "Consent gate"),
        ("L7", "m29_ipc_bus", "IPC bus"),
        ("L7", "m30_bus_types", "Bus frame types"),
        ("L7", "m31_conductor", "PI controller"),
        ("L7", "m32_executor", "Task routing"),
        ("L7", "m33_cascade", "Cascade handoff"),
        ("L7", "m34_suggestions", "Suggestions engine"),
        ("L7", "m35_tick", "Tick orchestrator"),
        ("L7", "m36_persistence", "Persistence layer"),
        ("L8", "m37_proposals", "Proposal lifecycle"),
        ("L8", "m38_voting", "Quorum voting"),
        ("L8", "m39_consent_declaration", "Consent declaration"),
        ("L8", "m40_data_sovereignty", "Data sovereignty"),
        ("L8", "m41_evolution", "Evolution chamber"),
    ];

    let modules: Vec<PlanModule> = module_defs.iter().map(|(layer, mname, desc)| {
        PlanModule {
            layer: (*layer).into(),
            name: (*mname).into(),
            description: (*desc).into(),
            test_kind: None,
            depends_on: None,
            quality_override: None,
        }
    }).collect();

    let mut deps = BTreeMap::new();
    for (k, v, feats) in [
        ("serde", "1", Some(vec!["derive".into()])),
        ("serde_json", "1", None),
        ("thiserror", "2", None),
        ("uuid", "1", Some(vec!["v4".into()])),
        ("parking_lot", "0.12", None),
        ("chrono", "0.4", Some(vec!["serde".into()])),
        ("tracing", "0.1", None),
    ] {
        deps.insert(k.into(), DepSpec {
            version: v.into(),
            features: feats,
            optional: None,
        });
    }

    Plan {
        metadata: Metadata {
            name: name.into(),
            description: "8-layer Rust microservice scaffold".into(),
            version: "0.1.0".into(),
            edition: Some("2021".into()),
            rust_version: Some("1.75".into()),
            authors: None,
            license: None,
            repository: None,
            port: Some(8132),
            service_id: Some(name.into()),
            devenv_batch: Some(5),
        },
        dependencies: deps,
        features: None,
        quality: Some(QualityConfig {
            gate: Some(vec!["check".into(), "clippy".into(), "pedantic".into(), "test".into()]),
            min_tests_per_module: Some(50),
            deny_unwrap: Some(true),
            deny_unsafe: Some(true),
            k7_compliance: None,
            k7_compliance_target: None,
            hebbian_feedback: None,
        }),
        consent: None,
        bin_targets: Some(vec![
            BinTarget { name: name.into(), path: "src/bin/main.rs".into(), kind: Some("daemon".into()) },
            BinTarget { name: format!("{name}-client"), path: "src/bin/client.rs".into(), kind: Some("client".into()) },
        ]),
        layers,
        modules,
        implementation: Some(ImplOrder {
            order: vec!["L1".into(), "L3".into(), "L4".into(), "L5".into(),
                        "L2".into(), "L6".into(), "L7".into(), "L8".into()],
        }),
        config: None,
    }
}

// ═══════════════════════════════════════════════════════════════
// Plan helper functions
// ═══════════════════════════════════════════════════════════════

/// Get modules belonging to a specific layer.
fn modules_for_layer<'a>(plan: &'a Plan, layer_key: &str) -> Vec<&'a PlanModule> {
    plan.modules.iter().filter(|m| m.layer == layer_key).collect()
}

/// Render architecture block for documentation.
fn render_architecture_block(plan: &Plan) -> String {
    let mut block = String::new();
    for layer in &plan.layers {
        let mods = modules_for_layer(plan, &layer.key);
        let range = module_range(&mods);

        let _ = writeln!(
            block,
            "{} {:<12} {}: {}",
            layer.key,
            layer.name,
            range,
            layer.description
        );
    }
    block
}

/// Compute module range string, e.g. "(m01-m06)".
fn module_range(modules: &[&PlanModule]) -> String {
    if modules.is_empty() {
        return "(empty)".into();
    }
    if modules.len() == 1 {
        return format!("({})", modules[0].name);
    }
    format!("({}-{})", modules[0].name, modules[modules.len() - 1].name)
}

/// Render text dependency chain for `MASTERPLAN.md`.
fn render_dep_chain(plan: &Plan) -> String {
    let mut lines = Vec::new();
    for layer in &plan.layers {
        if layer.depends_on.is_empty() {
            lines.push(format!("{} (no deps)", layer.key));
        } else {
            lines.push(format!("{} -> {}", layer.key, layer.depends_on.join(" & ")));
        }
    }
    lines.join("\n")
}

/// Render Mermaid dependency graph.
fn render_mermaid_dependencies(plan: &Plan) -> String {
    let mut mermaid = String::from("```mermaid\ngraph BT\n");
    for layer in &plan.layers {
        let count = plan.modules.iter().filter(|m| m.layer == layer.key).count();

        let _ = writeln!(
            mermaid,
            "    {}[{} {}<br/>{} modules]",
            layer.key, layer.key, layer.name, count
        );
    }
    mermaid.push('\n');
    for layer in &plan.layers {
        for dep in &layer.depends_on {
    
            let _ = writeln!(mermaid, "    {} --> {}", layer.key, dep);
        }
    }
    mermaid.push_str("```\n");
    mermaid
}

/// Return the appropriate test scaffold for a given `test_kind`.
///
/// When `has_tokio` is false, async tests fall back to unit tests.
fn test_scaffold(test_kind: &str, has_tokio: bool) -> &str {
    match test_kind {
        "async" if has_tokio => "#[tokio::test]\n    async fn async_scaffold() {\n        assert!(true);\n    }",
        "integration" => "#[test]\n    fn integration_scaffold() {\n        assert!(true);\n    }",
        "property" => "#[test]\n    fn property_scaffold() {\n        for i in 0..100_u32 {\n            assert!(i < 101);\n        }\n    }",
        _ => "#[test]\n    fn scaffold_compiles() {\n        assert!(true);\n    }",
    }
}

/// Determine whether a plan uses "should" (modulation) or "must" (command) language.
fn uses_modulation(plan: &Plan) -> bool {
    plan.consent.as_ref()
        .and_then(|c| c.modulation_not_command)
        .unwrap_or(false)
}

/// Determine the implementation order label.
fn impl_order_label(plan: &Plan) -> &str {
    let is_recommended = plan.consent.as_ref()
        .and_then(|c| c.implementation_order.as_deref())
        .unwrap_or("required");
    if is_recommended == "recommended" { "Recommended" } else { "Required" }
}

// ═══════════════════════════════════════════════════════════════
// Phase 2-5: Plan-Driven Generators
// ═══════════════════════════════════════════════════════════════

/// Generate a single module `.rs` file with test scaffold.
fn generate_module(src: &Path, module: &PlanModule, layer: &LayerDef, has_tokio: bool) {
    let file_path = src.join(&layer.dir_name).join(format!("{}.rs", module.name));
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    let test_kind = module.test_kind.as_deref().unwrap_or("unit");
    let test_body = test_scaffold(test_kind, has_tokio);

    // NA-GAP-3: sphere attribution stubs
    let attribution = "//! Implemented by: (sphere attribution)\n//! Session: (recorded on implementation)\n";

    // NA-GAP-5: consent check stub for bridge modules
    let is_bridge = layer.dir_name.contains("bridge") || module.name.contains("bridge");
    let consent_stub = if is_bridge {
        "\n/// Consent check — every external control must pass this gate.\nfn _consent_check() -> bool { true }\n"
    } else {
        ""
    };

    // Per-module deps comment (GAP 2) — backtick module names for clippy doc_markdown
    let deps_comment = module.depends_on.as_ref().map_or_else(String::new, |deps| {
        if deps.is_empty() {
            String::new()
        } else {
            let backticked: Vec<String> = deps.iter().map(|d| format!("`{d}`")).collect();
            format!("//! Dependencies: {}\n", backticked.join(", "))
        }
    });

    let content = format!(
        "//! `{}` — {}.\n//!\n//! Layer: `{}`\n{deps_comment}{attribution}\n\
         {consent_stub}\
         #[cfg(test)]\nmod tests {{\n    {test_body}\n}}\n",
        module.name, module.description, layer.dir_name
    );
    fs::write(&file_path, content).ok();
}

/// Generate `mod.rs` for a layer directory.
fn generate_mod_rs(src: &Path, layer: &LayerDef, plan: &Plan) {
    let layer_dir = src.join(&layer.dir_name);
    fs::create_dir_all(&layer_dir).ok();

    let mods = modules_for_layer(plan, &layer.key);
    let mut content = format!("//! `{}` layer modules.\n\n", layer.dir_name);
    for m in &mods {

        if let Some(fg) = &layer.feature_gate {
            let _ = writeln!(content, "#[cfg(feature = \"{fg}\")]");
        }
        let _ = writeln!(content, "pub mod {};", m.name);
    }
    fs::write(layer_dir.join("mod.rs"), content).ok();
}

/// Generate `lib.rs` from the plan.
fn generate_lib_rs(src: &Path, plan: &Plan) {
    let name = &plan.metadata.name;
    let layer_count = plan.layers.len();
    let module_count = plan.modules.len();

    let mut content = format!("//! `{name}` — {layer_count} layers, {module_count} modules.\n\n");
    for layer in &plan.layers {

        if let Some(fg) = &layer.feature_gate {
            let _ = writeln!(content, "#[cfg(feature = \"{fg}\")]");
        }
        let _ = writeln!(content, "pub mod {};", layer.dir_name);
    }
    fs::write(src.join("lib.rs"), content).ok();
}

/// Generate `Cargo.toml` from plan dependencies and features.
fn generate_cargo_toml(dir: &Path, plan: &Plan) {
    let m = &plan.metadata;
    let name = &m.name;
    let version = &m.version;
    let edition = m.edition.as_deref().unwrap_or("2021");
    let rust_version = m.rust_version.as_deref().unwrap_or("1.75");

    let mut content = format!(
        "[package]\n\
         name = \"{name}\"\n\
         version = \"{version}\"\n\
         edition = \"{edition}\"\n\
         rust-version = \"{rust_version}\"\n\
         description = \"{}\"\n",
        m.description
    );

    if let Some(authors) = &m.authors {
        let _ = write!(content, "authors = [");
        for (i, a) in authors.iter().enumerate() {
            if i > 0 { let _ = write!(content, ", "); }
            let _ = write!(content, "\"{a}\"");
        }
        let _ = writeln!(content, "]");
    }
    if let Some(license) = &m.license {
        let _ = writeln!(content, "license = \"{license}\"");
    }
    if let Some(repo) = &m.repository {
        if !repo.is_empty() {
            let _ = writeln!(content, "repository = \"{repo}\"");
        }
    }

    let _ = writeln!(content, "\n[workspace]");

    // Binary targets
    if let Some(bins) = &plan.bin_targets {
        for bin in bins {
            let _ = writeln!(content, "\n[[bin]]\nname = \"{}\"\npath = \"{}\"", bin.name, bin.path);
        }
    }

    // Dependencies
    let _ = writeln!(content, "\n[dependencies]");
    for (dep_name, spec) in &plan.dependencies {
        if let Some(feats) = &spec.features {
            let feat_str: Vec<String> = feats.iter().map(|f| format!("\"{f}\"")).collect();
            let optional_str = if spec.optional.unwrap_or(false) { ", optional = true" } else { "" };
            let _ = writeln!(
                content,
                "{dep_name} = {{ version = \"{}\", features = [{}]{optional_str} }}",
                spec.version,
                feat_str.join(", ")
            );
        } else if spec.optional.unwrap_or(false) {
            let _ = writeln!(content, "{dep_name} = {{ version = \"{}\", optional = true }}", spec.version);
        } else {
            let _ = writeln!(content, "{dep_name} = \"{}\"", spec.version);
        }
    }

    // Features — from plan.features or auto-generated from layer feature gates
    let layer_features: Vec<&str> = plan.layers.iter()
        .filter_map(|l| l.feature_gate.as_deref())
        .collect();

    if let Some(features) = &plan.features {
        let _ = writeln!(content, "\n[features]");
        for (feat_name, feat_val) in features {
            let _ = writeln!(content, "{feat_name} = {feat_val}");
        }
    } else if !layer_features.is_empty() {
        let _ = writeln!(content, "\n[features]");
        for feat in &layer_features {
            let _ = writeln!(content, "{feat} = []");
        }
    }

    // Lints
    let quality = plan.quality.as_ref();
    let _ = writeln!(content, "\n[lints.clippy]");
    let _ = writeln!(content, "pedantic = {{ level = \"warn\", priority = -1 }}");
    if quality.and_then(|q| q.deny_unwrap).unwrap_or(true) {
        let _ = writeln!(content, "unwrap_used = \"deny\"");
        let _ = writeln!(content, "expect_used = \"deny\"");
    }

    // Profile
    let _ = writeln!(content, "\n[profile.release]\nopt-level = 3\nlto = \"thin\"\nstrip = \"symbols\"");

    fs::write(dir.join("Cargo.toml"), content).ok();
}

/// Generate binary stub files based on `bin_targets`.
fn generate_binaries(src: &Path, plan: &Plan) {
    let bin_dir = src.join("bin");
    fs::create_dir_all(&bin_dir).ok();

    let targets = plan.bin_targets.as_deref().unwrap_or(&[]);
    if targets.is_empty() {
        // Fallback: at least a main.rs
        fs::write(
            bin_dir.join("main.rs"),
            "//! Daemon scaffold.\n\nfn main() {\n    eprintln!(\"daemon — not yet implemented\");\n}\n",
        ).ok();
        return;
    }

    for target in targets {
        let filename = Path::new(&target.path)
            .file_name()
            .map_or("main.rs", |f| f.to_str().unwrap_or("main.rs"));
        let kind = target.kind.as_deref().unwrap_or("daemon");
        let stub = match kind {
            "daemon" => format!(
                "//! `{}` daemon scaffold.\n\nfn main() {{\n    eprintln!(\"{} daemon — not yet implemented\");\n}}\n",
                target.name, target.name
            ),
            "client" => format!(
                "//! `{}` client CLI scaffold.\n\nfn main() {{\n    eprintln!(\"{} client — not yet implemented\");\n}}\n",
                target.name, target.name
            ),
            "probe" => format!(
                "//! `{}` probe tool.\n\nfn main() {{\n    eprintln!(\"{} probe — not yet implemented\");\n}}\n",
                target.name, target.name
            ),
            _ => format!(
                "//! `{}` tool scaffold.\n\nfn main() {{\n    eprintln!(\"{} — not yet implemented\");\n}}\n",
                target.name, target.name
            ),
        };
        fs::write(bin_dir.join(filename), stub).ok();
    }
}

/// Generate `bacon.toml` (already generic).
fn generate_bacon_toml(dir: &Path) {
    let content = r#"default_job = "check"

[jobs.check]
command = ["cargo", "check"]
need_stdout = false

[jobs.clippy]
command = ["cargo", "clippy", "--", "-D", "warnings"]
need_stdout = false

[jobs.pedantic]
command = ["cargo", "clippy", "--", "-D", "warnings", "-W", "clippy::pedantic"]
need_stdout = false

[jobs.test]
command = ["cargo", "test", "--lib"]
need_stdout = true

[jobs.gate]
command = ["bash", "-c", "cargo check && cargo clippy -- -D warnings && cargo clippy -- -D warnings -W clippy::pedantic && cargo test --lib"]
need_stdout = true
"#;
    fs::write(dir.join("bacon.toml"), content).ok();
}

/// Generate all documentation from the plan.
fn generate_docs(dir: &Path, plan: &Plan) {
    generate_top_level_docs(dir, plan);
    generate_ai_docs(dir, plan);
    generate_ai_specs(dir, plan);
    generate_schematics(dir, plan);
    generate_config_files(dir, plan);
    generate_claude_context(dir, plan);
}

/// Generate `CLAUDE.md` and `MASTERPLAN.md`.
fn generate_top_level_docs(dir: &Path, plan: &Plan) {
    let name = &plan.metadata.name;
    let layer_count = plan.layers.len();
    let module_count = plan.modules.len();
    let arch_block = render_architecture_block(plan);
    let min_tests = plan.quality.as_ref()
        .and_then(|q| q.min_tests_per_module)
        .unwrap_or(50);

    // NA-GAP-1: "should" vs "must" language
    let rule_verb = if uses_modulation(plan) { "should" } else { "must" };
    let safety_verb = "must"; // Safety rules always "must"

    let claude_md = format!(
        "# {name}\n\n\
         > {layer_count} layers | {module_count} modules | Quality gate: check + clippy + pedantic + test\n\n\
         ## Architecture\n\n\
         ```\n\
         {arch_block}\
         ```\n\n\
         ## Quality Gate (MANDATORY)\n\n\
         ```bash\n\
         cargo check && cargo clippy -- -D warnings && \\\n\
         cargo clippy -- -D warnings -W clippy::pedantic && cargo test --lib\n\
         ```\n\n\
         ## Rules\n\n\
         - {safety_verb}: No `unwrap()`/`expect()` outside tests — enforced via `[lints.clippy]`\n\
         - {safety_verb}: No `unsafe` — zero tolerance\n\
         - {rule_verb}: Doc comments on all public items\n\
         - {rule_verb}: {min_tests}+ tests per module minimum\n\
         - {rule_verb}: Backtick all identifiers in doc comments\n\n\
         ## Implementation Order\n\n\
         {}\n\n\
         ## Documentation Map\n\n\
         | Directory | Contents |\n\
         |-----------|----------|\n\
         | `ai_docs/` | Architecture, module docs, onboarding, schematics index |\n\
         | `ai_docs/modules/` | Per-layer module documentation |\n\
         | `ai_specs/` | Technical specifications, constraints, protocols |\n\
         | `ai_specs/layers/` | Per-layer implementation specs |\n\
         | `ai_specs/patterns/` | Rust patterns, anti-patterns, concurrency |\n\
         | `schematics/` | Mermaid diagrams: architecture, API, data flow |\n\
         | `config/` | TOML configs: default, production, devenv |\n\
         | `.claude/` | Claude Code context: patterns, schemas, queries |\n",
        plan.implementation.as_ref().map_or_else(
            || plan.layers.iter().map(|l| l.key.clone()).collect::<Vec<_>>().join(" -> "),
            |imp| imp.order.join(" -> ")
        )
    );
    fs::write(dir.join("CLAUDE.md"), claude_md).ok();

    // ── MASTERPLAN.md ──
    let order_label = impl_order_label(plan);
    let dep_chain = render_dep_chain(plan);

    let mut masterplan = format!(
        "# {name} — Master Plan\n\n\
         ## Phase 1: Scaffold (COMPLETE)\n\n\
         {layer_count} layers, {module_count} modules, full documentation tree, quality gate clean.\n\n\
         ## Phase 2: Implementation (bottom-up)\n\n\
         Quality gate after EVERY module. No exceptions.\n\n\
         ### {order_label} Order\n\n"
    );

    let impl_order: Vec<&str> = plan.implementation.as_ref().map_or_else(
        || plan.layers.iter().map(|l| l.key.as_str()).collect(),
        |imp| imp.order.iter().map(String::as_str).collect(),
    );

    for (i, key) in impl_order.iter().enumerate() {
        if let Some(layer) = plan.layers.iter().find(|l| l.key == *key) {
            let mods = modules_for_layer(plan, key);
            let range = module_range(&mods);
            let deps = if layer.depends_on.is_empty() {
                "zero dependencies".into()
            } else {
                format!("needs {}", layer.depends_on.join(", "))
            };
    
            let _ = writeln!(masterplan, "{}. {} {} {} — {}", i + 1, layer.key, layer.name, range, deps);
        }
    }

    let _ = writeln!(masterplan, "\n## Phase 3: Wiring\n");
    let _ = writeln!(masterplan, "- Daemon binary: server startup, health endpoint");
    let _ = writeln!(masterplan, "- Client binary: connect, query, disconnect");
    let _ = writeln!(masterplan, "\n## Dependency Graph\n\n```\n{dep_chain}\n```\n");

    // DevOps Synergy: agent dispatch strategy
    if plan.quality.as_ref().and_then(|q| q.k7_compliance).unwrap_or(false) {
        let _ = writeln!(masterplan, "\n## Agent Dispatch Strategy\n");
        for layer in &plan.layers {
            if layer.depends_on.is_empty() {
                let _ = writeln!(masterplan, "- Omega -> {} (foundation, highest priority)", layer.key);
            } else if layer.dir_name.contains("coord") || layer.dir_name.contains("bus") {
                let _ = writeln!(masterplan, "- Conductor -> {} (coordination)", layer.key);
            } else {
                let _ = writeln!(masterplan, "- Validator -> {} (quality gate per module)", layer.key);
            }
        }
        let _ = writeln!(masterplan, "- Scribe -> Documentation (all layers)");
    }

    fs::write(dir.join("MASTERPLAN.md"), masterplan).ok();
}

/// Generate `ai_docs/` tree.
fn generate_ai_docs(dir: &Path, plan: &Plan) {
    let name = &plan.metadata.name;
    let ai_docs = dir.join("ai_docs");
    let ai_docs_modules = ai_docs.join("modules");
    fs::create_dir_all(&ai_docs_modules).ok();

    fs::write(ai_docs.join("INDEX.md"), format!(
        "# {name} — Documentation Index\n\n\
         | Document | Purpose |\n\
         |----------|--------|\n\
         | `ARCHITECTURE_DEEP_DIVE.md` | System architecture overview |\n\
         | `CODE_MODULE_MAP.md` | Module-to-file mapping |\n\
         | `DEPLOYMENT_GUIDE.md` | Build, install, deploy |\n\
         | `ERROR_TAXONOMY.md` | Error types and handling |\n\
         | `MESSAGE_FLOWS.md` | Inter-module message paths |\n\
         | `ONBOARDING.md` | New developer quickstart |\n\
         | `PERFORMANCE.md` | Performance targets and profiling |\n\
         | `QUICKSTART.md` | 5-minute setup guide |\n\
         | `STATE_MACHINES.md` | FSM diagrams for lifecycle |\n\
         | `modules/` | Per-layer module documentation |\n"
    )).ok();

    // Dynamic architecture deep dive
    let mut arch = format!(
        "# {name} — Architecture Deep Dive\n\n\
         ## Layer Overview\n\n\
         {} layers, bottom-up dependency chain.\n\n",
        plan.layers.len()
    );
    for layer in &plan.layers {

        let _ = writeln!(arch, "## {} {}\n\n{}\n", layer.key, layer.name, layer.description);
    }
    fs::write(ai_docs.join("ARCHITECTURE_DEEP_DIVE.md"), arch).ok();

    // Dynamic code module map (GAP 2: per-module deps)
    let mut modmap = format!(
        "# {name} — Code Module Map\n\n\
         | Module | File | Layer | Description | Dependencies |\n\
         |--------|------|-------|-------------|-------------|\n"
    );
    for layer in &plan.layers {
        let mods = modules_for_layer(plan, &layer.key);
        for m in &mods {
    
            let deps = m.depends_on.as_ref()
                .map(|d| d.join(", "))
                .unwrap_or_default();
            let _ = writeln!(
                modmap,
                "| `{}` | `{}/{}.rs` | {} | {} | {} |",
                m.name, layer.dir_name, m.name, layer.key, m.description, deps
            );
        }
    }
    fs::write(ai_docs.join("CODE_MODULE_MAP.md"), modmap).ok();

    // Per-layer module docs
    for layer in &plan.layers {
        let mods = modules_for_layer(plan, &layer.key);
        let upper_key = format!("{}_{}", layer.key, layer.name.to_uppercase().replace(' ', "_"));
        let mut doc = format!("# {upper_key}\n\n{}\n\n## Modules\n\n", layer.description);
        for m in &mods {
    
            let _ = writeln!(doc, "- `{}` — {}", m.name, m.description);
        }

        let _ = writeln!(doc, "\nSee `ai_specs/layers/{upper_key}_SPEC.md` for implementation details.");
        fs::write(ai_docs_modules.join(format!("{upper_key}.md")), doc).ok();
    }

    for doc in ["DEPLOYMENT_GUIDE", "ERROR_TAXONOMY", "MESSAGE_FLOWS",
                "ONBOARDING", "PERFORMANCE", "QUICKSTART", "STATE_MACHINES"] {
        fs::write(ai_docs.join(format!("{doc}.md")), format!(
            "# {name} — {doc}\n\nTODO: Document {doc} during implementation.\n"
        )).ok();
    }
}

/// Generate `ai_specs/` tree.
#[allow(clippy::too_many_lines)]
fn generate_ai_specs(dir: &Path, plan: &Plan) {
    let name = &plan.metadata.name;
    let ai_specs = dir.join("ai_specs");
    let specs_layers = ai_specs.join("layers");
    let specs_patterns = ai_specs.join("patterns");
    fs::create_dir_all(&specs_layers).ok();
    fs::create_dir_all(&specs_patterns).ok();

    fs::write(ai_specs.join("INDEX.md"), format!(
        "# {name} — Specifications Index\n\n\
         | Spec | Purpose |\n\
         |------|--------|\n\
         | `API_SPEC.md` | HTTP API endpoints, request/response formats |\n\
         | `CONSENT_SPEC.md` | Consent gate, opt-out, receptivity |\n\
         | `DATABASE_SPEC.md` | SQLite schemas, WAL config |\n\
         | `DESIGN_CONSTRAINTS.md` | Non-negotiable constraints |\n\
         | `EVENT_SYSTEM_SPEC.md` | Bus events, types, routing |\n\
         | `IPC_BUS_SPEC.md` | Unix socket protocol |\n\
         | `MODULE_MATRIX.md` | Module dependency matrix |\n\
         | `SECURITY_SPEC.md` | Security model, permissions |\n\
         | `WIRE_PROTOCOL_SPEC.md` | NDJSON wire format |\n\
         | `layers/` | Per-layer implementation specs |\n\
         | `patterns/` | Rust patterns and anti-patterns |\n"
    )).ok();

    for spec in ["API_SPEC", "CONSENT_SPEC", "DATABASE_SPEC", "DESIGN_CONSTRAINTS",
                 "EVENT_SYSTEM_SPEC", "IPC_BUS_SPEC", "MODULE_MATRIX",
                 "SECURITY_SPEC", "WIRE_PROTOCOL_SPEC"] {
        fs::write(ai_specs.join(format!("{spec}.md")), format!(
            "# {name} — {spec}\n\nTODO: Specify during implementation.\n"
        )).ok();
    }

    let min_tests = plan.quality.as_ref()
        .and_then(|q| q.min_tests_per_module)
        .unwrap_or(50);
    let rule_verb = if uses_modulation(plan) { "should" } else { "must" };

    // Per-layer specs (dynamic, with NA-GAP-6 rationale)
    for layer in &plan.layers {
        let mods = modules_for_layer(plan, &layer.key);
        let upper_key = format!("{}_{}", layer.key, layer.name.to_uppercase().replace(' ', "_"));
        let mut spec = format!(
            "# {upper_key} — Implementation Spec\n\n\
             {}.\n\n",
            layer.description
        );


        // NA-GAP-6: rationale
        if let Some(rationale) = &layer.rationale {
            let _ = writeln!(spec, "## Rationale\n\n{rationale}\n");
        }

        let _ = writeln!(spec, "## Modules\n");
        for m in &mods {
            let _ = writeln!(spec, "- `{}` — {}", m.name, m.description);
        }

        let _ = writeln!(spec, "\n## Dependencies\n");
        if layer.depends_on.is_empty() {
            let _ = writeln!(spec, "No dependencies (foundation layer).\n");
        } else {
            let _ = writeln!(spec, "Depends on: {}\n", layer.depends_on.join(", "));
        }

        let _ = writeln!(spec, "## Constraints\n");
        let _ = writeln!(spec, "- {rule_verb}: {min_tests}+ tests per module");
        let _ = writeln!(spec, "- must: No `unwrap()`/`expect()` outside tests");
        let _ = writeln!(spec, "- must: Quality gate after every module");

        fs::write(specs_layers.join(format!("{upper_key}_SPEC.md")), spec).ok();
    }

    // Pattern docs (generic)
    fs::write(specs_patterns.join("INDEX.md"), format!(
        "# {name} — Patterns Index\n\n\
         | Pattern | Purpose |\n\
         |---------|--------|\n\
         | `RUST_CORE_PATTERNS.md` | Ownership, borrowing, lifetimes |\n\
         | `ERROR_PATTERNS.md` | Error propagation, `thiserror`, `?` operator |\n\
         | `CONCURRENCY_PATTERNS.md` | `Arc<RwLock>`, lock ordering, `parking_lot` |\n\
         | `ASYNC_PATTERNS.md` | Tokio, `spawn`, channel patterns |\n\
         | `BRIDGE_PATTERNS.md` | Fire-and-forget TCP, bridge health |\n\
         | `IPC_PATTERNS.md` | NDJSON, handshake, subscribe, V1 compat |\n\
         | `ANTIPATTERNS.md` | What NOT to do |\n"
    )).ok();

    for pattern in ["RUST_CORE_PATTERNS", "ERROR_PATTERNS", "CONCURRENCY_PATTERNS",
                    "ASYNC_PATTERNS", "BRIDGE_PATTERNS", "IPC_PATTERNS"] {
        fs::write(specs_patterns.join(format!("{pattern}.md")), format!(
            "# {pattern}\n\nTODO: Document during implementation.\n"
        )).ok();
    }

    let deny_unwrap = plan.quality.as_ref().and_then(|q| q.deny_unwrap).unwrap_or(true);
    let deny_unsafe = plan.quality.as_ref().and_then(|q| q.deny_unsafe).unwrap_or(true);
    let mut antipatterns = format!("# {name} — Anti-Patterns\n\n## NEVER\n\n");
    let mut n = 1;
    if deny_unwrap {

        let _ = writeln!(antipatterns, "{n}. `unwrap()`/`expect()` in production code");
        n += 1;
    }
    if deny_unsafe {

        let _ = writeln!(antipatterns, "{n}. `unsafe` blocks");
        n += 1;
    }
    {

        let _ = writeln!(antipatterns, "{n}. `stdout` in daemon binaries (SIGPIPE death)");
        let _ = writeln!(antipatterns, "{}. Global mutable state (use `Arc<RwLock<T>>`)", n + 1);
        let _ = writeln!(antipatterns, "{}. Suppressing clippy warnings (`#![allow]`) — fix the code", n + 2);
        let _ = writeln!(antipatterns, "{}. Panic-based error handling", n + 3);
        let _ = writeln!(antipatterns, "{}. `mod.rs` that re-exports everything blindly", n + 4);
        let _ = writeln!(antipatterns, "{}. Chaining after `pkill` (exit 144 kills `&&` chains)", n + 5);
        let _ = writeln!(antipatterns, "{}. `cp` without `\\` prefix (aliased to interactive)", n + 6);
        let _ = writeln!(antipatterns, "{}. `git status -uall` (memory explosion on large repos)", n + 7);
    }
    fs::write(specs_patterns.join("ANTIPATTERNS.md"), antipatterns).ok();
}

/// Generate `schematics/` with dynamic Mermaid from plan.
fn generate_schematics(dir: &Path, plan: &Plan) {
    let name = &plan.metadata.name;
    let schematics = dir.join("schematics");
    fs::create_dir_all(&schematics).ok();

    fs::write(schematics.join("INDEX.md"), format!(
        "# {name} — Schematics Index\n\n\
         | Schematic | Purpose |\n\
         |-----------|--------|\n\
         | `LAYER_DEPENDENCIES.md` | Layer dependency graph (Mermaid) |\n\
         | `API_MAP.md` | HTTP API route map |\n\
         | `DATA_FLOW.md` | Data flow between modules |\n\
         | `DEPLOYMENT_FLOW.md` | Build, install, deploy pipeline |\n\
         | `IPC_PROTOCOL.md` | Wire protocol sequence diagram |\n\
         | `STATE_TRANSITIONS.md` | Service and sphere state machines |\n"
    )).ok();

    // Dynamic Mermaid dependency graph
    let mermaid = render_mermaid_dependencies(plan);
    fs::write(schematics.join("LAYER_DEPENDENCIES.md"), format!(
        "# {name} — Layer Dependencies\n\n{mermaid}"
    )).ok();

    // Dynamic API map from bin targets
    let mut api_map = format!("# {name} — API Route Map\n\n```mermaid\ngraph LR\n");
    let targets = plan.bin_targets.as_deref().unwrap_or(&[]);
    let has_daemon = targets.iter().any(|t| t.kind.as_deref() == Some("daemon"));
    if has_daemon {

        let _ = writeln!(api_map, "    subgraph Core");
        let _ = writeln!(api_map, "        H[/health]");
        let _ = writeln!(api_map, "        S[/status]");
        let _ = writeln!(api_map, "    end");
    }
    let _ = writeln!(api_map, "```\n\nTODO: Expand with full route table during implementation.");
    fs::write(schematics.join("API_MAP.md"), api_map).ok();

    for schematic in ["DATA_FLOW", "DEPLOYMENT_FLOW", "IPC_PROTOCOL", "STATE_TRANSITIONS"] {
        fs::write(schematics.join(format!("{schematic}.md")), format!(
            "# {name} — {schematic}\n\nTODO: Add Mermaid diagrams during implementation.\n"
        )).ok();
    }
}

/// Generate `config/` directory.
fn generate_config_files(dir: &Path, plan: &Plan) {
    let name = &plan.metadata.name;
    let config_dir = dir.join("config");
    fs::create_dir_all(&config_dir).ok();

    // Default config from plan or generic
    if let Some(config_sections) = &plan.config {
        let mut content = format!("# {name} — Default Configuration\n\n");
        for (section, kvs) in config_sections {
    
            let _ = writeln!(content, "[{section}]");
            for (k, v) in kvs {
                let _ = writeln!(content, "{k} = {v}");
            }
            let _ = writeln!(content);
        }
        fs::write(config_dir.join("default.toml"), content).ok();
    } else {
        let port = plan.metadata.port.unwrap_or(8080);
        fs::write(config_dir.join("default.toml"), format!(
            "# {name} — Default Configuration\n\n\
             [server]\n\
             bind_addr = \"127.0.0.1\"\n\
             port = {port}\n\n\
             [persistence]\n\
             data_dir = \"data/\"\n"
        )).ok();
    }

    fs::write(config_dir.join("production.toml"), format!(
        "# {name} — Production Overrides\n\n\
         [server]\n\
         bind_addr = \"127.0.0.1\"\n\n\
         [persistence]\n\
         data_dir = \"/var/lib/{name}/\"\n"
    )).ok();

    let sid = plan.metadata.service_id.as_deref().unwrap_or(name.as_str());
    let port = plan.metadata.port.unwrap_or(8080);
    fs::write(config_dir.join("devenv-service.toml"), format!(
        "[services.{sid}]\n\
         name = \"{name} v{}\"\n\
         command = \"./bin/{sid}\"\n\
         auto_start = true\n\
         auto_restart = true\n\
         max_restart_attempts = 5\n\
         health_check_url = \"http://localhost:{port}/health\"\n\
         health_check_interval_secs = 30\n\
         startup_timeout_secs = 60\n\n\
         [services.{sid}.env]\n\
         RUST_LOG = \"info\"\n\n\
         [services.{sid}.resource_limits]\n\
         max_memory_mb = 128\n\
         max_cpu_percent = 50\n",
        plan.metadata.version
    )).ok();
}

/// Generate `.claude/` context directory.
fn generate_claude_context(dir: &Path, plan: &Plan) {
    let name = &plan.metadata.name;
    let claude_dir = dir.join(".claude");
    let claude_schemas = claude_dir.join("schemas");
    let claude_queries = claude_dir.join("queries");
    fs::create_dir_all(&claude_schemas).ok();
    fs::create_dir_all(&claude_queries).ok();

    let impl_order = plan.implementation.as_ref().map_or_else(
        || {
            let items: Vec<String> = plan.layers.iter().map(|l| format!("\"{}\"", l.key)).collect();
            items.join(", ")
        },
        |imp| {
            let items: Vec<String> = imp.order.iter().map(|k| format!("\"{k}\"")).collect();
            items.join(", ")
        },
    );

    fs::write(claude_dir.join("context.json"), format!(
        "{{\n  \"project\": \"{name}\",\n  \"layers\": {},\n  \"modules\": {},\n  \
         \"quality_gate\": \"check + clippy + pedantic + test\",\n  \
         \"implementation_order\": [{impl_order}]\n}}\n",
        plan.layers.len(), plan.modules.len()
    )).ok();

    fs::write(claude_dir.join("patterns.json"),
        "{\n  \"error_handling\": \"thiserror + Result<T, Error>\",\n  \
         \"concurrency\": \"parking_lot::RwLock + Arc\",\n  \
         \"config\": \"Figment TOML + env\",\n  \
         \"serialization\": \"serde derive\",\n  \
         \"logging\": \"tracing (not log)\",\n  \
         \"identifiers\": \"uuid v4\",\n  \
         \"time\": \"chrono DateTime<Utc>\"\n}\n"
    ).ok();

    fs::write(claude_dir.join("anti_patterns.json"),
        "{\n  \"never\": [\n    \"unwrap() in production\",\n    \
         \"unsafe blocks\",\n    \"stdout in daemons\",\n    \
         \"global mutable state\",\n    \"suppress clippy warnings\",\n    \
         \"panic-based error handling\"\n  ]\n}\n"
    ).ok();
}

/// Generate `README.md` (GAP 1).
fn generate_readme(dir: &Path, plan: &Plan) {
    let name = &plan.metadata.name;
    let desc = &plan.metadata.description;
    let layer_count = plan.layers.len();
    let module_count = plan.modules.len();
    let license = plan.metadata.license.as_deref().unwrap_or("MIT");

    let content = format!(
        "# {name}\n\n\
         {desc}\n\n\
         ## Quick Start\n\n\
         ```bash\n\
         cargo build && cargo test\n\
         ```\n\n\
         ## Architecture\n\n\
         {layer_count} layers, {module_count} modules.\n\n\
         See `CLAUDE.md` for full architecture documentation.\n\n\
         ## License\n\n\
         {license}\n"
    );
    fs::write(dir.join("README.md"), content).ok();
}

/// Generate `CHANGELOG.md` (GAP 9).
fn generate_changelog(dir: &Path, plan: &Plan) {
    let layer_count = plan.layers.len();
    let module_count = plan.modules.len();

    let content = format!(
        "# Changelog\n\n\
         ## [{}] - {}\n\n\
         ### Added\n\n\
         - Initial scaffold: {layer_count} layers, {module_count} modules\n\
         - Full documentation tree: ai_docs, ai_specs, schematics\n\
         - Quality gate: check + clippy + pedantic + test\n",
        plan.metadata.version,
        chrono::Utc::now().format("%Y-%m-%d")
    );
    fs::write(dir.join("CHANGELOG.md"), content).ok();
}

/// Generate `CONTRIBUTORS.md` stub (NA-GAP-3).
fn generate_contributors(dir: &Path, plan: &Plan) {
    let name = &plan.metadata.name;
    let content = format!(
        "# {name} — Contributors\n\n\
         | Module | Implemented By | Session |\n\
         |--------|---------------|--------|\n\
         | (scaffold) | scaffold-gen | — |\n"
    );
    fs::write(dir.join("CONTRIBUTORS.md"), content).ok();
}

// ═══════════════════════════════════════════════════════════════
// Master Orchestrator
// ═══════════════════════════════════════════════════════════════

/// Generate the entire scaffold from a plan.
fn generate_all(dir: &Path, plan: &Plan) {
    let src = dir.join("src");
    fs::create_dir_all(&src).ok();

    let has_tokio = plan.dependencies.contains_key("tokio");

    // Generate all modules
    for layer in &plan.layers {
        let mods = modules_for_layer(plan, &layer.key);
        for m in mods {
            generate_module(&src, m, layer, has_tokio);
        }
    }

    // Generate mod.rs per layer
    for layer in &plan.layers {
        generate_mod_rs(&src, layer, plan);
    }

    // Generate lib.rs, binaries, config
    generate_lib_rs(&src, plan);
    generate_binaries(&src, plan);
    generate_cargo_toml(dir, plan);
    generate_bacon_toml(dir);
    generate_docs(dir, plan);
    generate_readme(dir, plan);
    generate_changelog(dir, plan);
    generate_contributors(dir, plan);
}

// ═══════════════════════════════════════════════════════════════
// Verify + CLI
// ═══════════════════════════════════════════════════════════════

/// Run the quality gate against a generated scaffold.
fn verify(dir: &Path) -> bool {
    use std::process::Command;
    let check = Command::new("cargo")
        .arg("check")
        .current_dir(dir)
        .output();
    let clippy = Command::new("cargo")
        .args(["clippy", "--", "-D", "warnings"])
        .current_dir(dir)
        .output();
    let pedantic = Command::new("cargo")
        .args(["clippy", "--", "-D", "warnings", "-W", "clippy::pedantic"])
        .current_dir(dir)
        .output();
    let test = Command::new("cargo")
        .args(["test", "--lib"])
        .current_dir(dir)
        .output();

    let mut pass = true;
    for (name, result) in [("check", check), ("clippy", clippy), ("pedantic", pedantic), ("test", test)] {
        match result {
            Ok(out) if out.status.success() => eprintln!("  {name}: PASS"),
            Ok(out) => {
                eprintln!("  {name}: FAIL");
                let stderr = String::from_utf8_lossy(&out.stderr);
                for line in stderr.lines().filter(|l| l.starts_with("error")) {
                    eprintln!("    {line}");
                }
                pass = false;
            }
            Err(e) => {
                eprintln!("  {name}: ERROR ({e})");
                pass = false;
            }
        }
    }
    pass
}

/// Print usage help.
fn print_help() {
    eprintln!("scaffold-gen — plan-driven Rust microservice scaffold generator");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  scaffold-gen --from-plan plan.toml <dir>   Plan-driven scaffold");
    eprintln!("  scaffold-gen <project-dir> <project-name>  Default 8L/41M scaffold");
    eprintln!("  scaffold-gen --verify <project-dir>        Verify existing scaffold");
    eprintln!("  scaffold-gen --help                        Show this help");
    eprintln!();
    eprintln!("PLAN FORMAT: See plan.toml specification in scaffold-mastery skill");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("--help" | "-h") => {
            print_help();
        }
        Some("--from-plan") => {
            let Some(plan_path) = args.get(2) else {
                eprintln!("Error: --from-plan requires a plan.toml path");
                eprintln!("Usage: scaffold-gen --from-plan plan.toml <project-dir>");
                std::process::exit(1);
            };
            let Some(dir_str) = args.get(3) else {
                eprintln!("Error: --from-plan requires a project directory");
                eprintln!("Usage: scaffold-gen --from-plan plan.toml <project-dir>");
                std::process::exit(1);
            };
            let plan_path = PathBuf::from(plan_path);
            let dir = PathBuf::from(dir_str);

            let plan = match load_plan(&plan_path) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Error loading plan: {e}");
                    std::process::exit(1);
                }
            };

            if let Err(e) = validate_plan(&plan) {
                eprintln!("Plan validation failed: {e}");
                std::process::exit(1);
            }

            eprintln!(
                "Scaffolding {} from {} in {}...",
                plan.metadata.name,
                plan_path.display(),
                dir.display()
            );

            generate_all(&dir, &plan);

            eprintln!(
                "Scaffold complete: {} layers, {} modules, ready for quality gate",
                plan.layers.len(),
                plan.modules.len()
            );
        }
        Some("--verify") => {
            let dir = args.get(2).map_or_else(|| PathBuf::from("."), PathBuf::from);
            eprintln!("Verifying scaffold in {}...", dir.display());
            let ok = verify(&dir);
            std::process::exit(i32::from(!ok));
        }
        Some(dir_str) => {
            let dir = PathBuf::from(dir_str);
            let name = args.get(2).map_or("scaffold-project", String::as_str);

            eprintln!("Scaffolding {name} (default 8L/41M) in {}...", dir.display());

            let plan = default_plan(name);
            generate_all(&dir, &plan);

            eprintln!(
                "Scaffold complete: {} layers, {} modules, ready for quality gate",
                plan.layers.len(),
                plan.modules.len()
            );
        }
        None => {
            print_help();
            std::process::exit(1);
        }
    }
}
