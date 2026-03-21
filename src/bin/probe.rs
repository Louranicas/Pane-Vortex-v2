//! # Habitat Probe
//!
//! Fast system intelligence CLI for The Habitat skills.
//! Replaces inline curl chains with typed, structured JSON output.
//!
//! ## Usage
//!
//! ```bash
//! habitat-probe pulse          # Quick 3-service check (~30ms)
//! habitat-probe sweep          # All 16 services health
//! habitat-probe field          # PV field state + decision
//! habitat-probe spheres        # Active sphere listing
//! habitat-probe bus            # Bus tasks, events, cascades
//! habitat-probe me             # ME observer + fitness
//! habitat-probe bridges        # Bridge staleness
//! habitat-probe full           # Everything in one shot
//! ```

use serde_json::{json, Value};
use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_secs(2);

/// GET JSON from a local service. Returns error object on failure.
fn get(url: &str) -> Value {
    match ureq::get(url).timeout(TIMEOUT).call() {
        Ok(resp) => {
            let body = resp.into_string().unwrap_or_default();
            serde_json::from_str(&body).unwrap_or_else(|_| json!({"raw": body}))
        }
        Err(e) => json!({"error": format!("{e}")}),
    }
}

/// Check a single port, return status code + latency.
fn probe_port(port: u16, path: &str) -> Value {
    let url = format!("http://127.0.0.1:{port}{path}");
    let start = Instant::now();
    let code = match ureq::get(&url).timeout(TIMEOUT).call() {
        Ok(r) => r.status(),
        Err(ureq::Error::Status(c, _)) => c,
        Err(_) => 0,
    };
    json!({"port": port, "status": code, "ms": start.elapsed().as_millis()})
}

const fn health_path(port: u16) -> &'static str {
    match port {
        8080 | 8090 => "/api/health",
        _ => "/health",
    }
}

// ──────────────────────────────────────────────────────────────
// Commands
// ──────────────────────────────────────────────────────────────

fn cmd_pulse() -> Value {
    let pv = get("http://127.0.0.1:8132/health");
    let povm = get("http://127.0.0.1:8125/hydrate");
    let me = get("http://127.0.0.1:8080/api/observer");

    json!({
        "pv": {
            "status": pv.get("status"),
            "tick": pv.get("tick"),
            "spheres": pv.get("spheres"),
            "r": pv.get("r"),
            "fleet_mode": pv.get("fleet_mode"),
            "k": pv.get("k"),
        },
        "povm": {
            "memories": povm.get("memory_count"),
            "pathways": povm.get("pathway_count"),
        },
        "me": {
            "fitness": me.pointer("/last_report/current_fitness"),
            "trend": me.get("fitness_trend"),
            "state": me.get("system_state"),
            "tick": me.get("tick_count"),
        },
    })
}

fn cmd_sweep() -> Value {
    let ports: &[u16] = &[
        8080, 8081, 8090, 8100, 8101, 8102, 8103, 8104, 8105, 8110, 8120, 8125, 8130, 8132,
        9001, 10001,
    ];

    let start = Instant::now();
    let results: Vec<Value> = ports.iter().map(|&p| probe_port(p, health_path(p))).collect();
    let healthy = results.iter().filter(|r| r["status"] == 200).count();

    json!({
        "services": results,
        "healthy": healthy,
        "total": ports.len(),
        "sweep_ms": start.elapsed().as_millis(),
    })
}

fn cmd_field() -> Value {
    let health = get("http://127.0.0.1:8132/health");
    let r = get("http://127.0.0.1:8132/field/r");
    let decision = get("http://127.0.0.1:8132/field/decision");

    json!({
        "tick": health.get("tick"),
        "spheres": health.get("spheres"),
        "fleet_mode": health.get("fleet_mode"),
        "r": r.get("r"),
        "psi": r.get("psi"),
        "k": health.get("k"),
        "k_modulation": health.get("k_modulation"),
        "action": decision.get("action"),
        "tunnel_count": decision.get("tunnel_count"),
        "working": decision.get("working_spheres"),
        "idle": decision.get("idle_spheres"),
        "strongest_tunnel": decision.get("strongest_tunnel"),
    })
}

fn cmd_spheres() -> Value {
    get("http://127.0.0.1:8132/spheres")
}

fn cmd_bus() -> Value {
    let info = get("http://127.0.0.1:8132/bus/info");
    let tasks = get("http://127.0.0.1:8132/bus/tasks");
    let cascades = get("http://127.0.0.1:8132/bus/cascades");

    json!({
        "info": info,
        "pending_tasks": tasks.get("tasks"),
        "pending_cascades": cascades.get("cascades"),
    })
}

fn cmd_me() -> Value {
    let observer = get("http://127.0.0.1:8080/api/observer");
    let eventbus = get("http://127.0.0.1:8080/api/eventbus/stats");

    json!({
        "fitness": observer.pointer("/last_report/current_fitness"),
        "trend": observer.get("fitness_trend"),
        "state": observer.get("system_state"),
        "tick": observer.get("tick_count"),
        "metrics": observer.get("metrics"),
        "eventbus_channels": eventbus.get("channels"),
        "eventbus_total_events": eventbus.get("total_events"),
    })
}

fn cmd_bridges() -> Value {
    get("http://127.0.0.1:8132/bridges/health")
}

fn cmd_full() -> Value {
    json!({
        "pulse": cmd_pulse(),
        "sweep": cmd_sweep(),
        "field": cmd_field(),
        "bus": cmd_bus(),
        "bridges": cmd_bridges(),
    })
}

fn print_help() {
    eprintln!("habitat-probe — Fast system intelligence for The Habitat");
    eprintln!();
    eprintln!("USAGE: habitat-probe <command>");
    eprintln!();
    eprintln!("COMMANDS:");
    eprintln!("  pulse     Quick 3-service check (PV + POVM + ME) ~30ms");
    eprintln!("  sweep     All 16 services health check");
    eprintln!("  field     PV field state, decision engine, tunnels");
    eprintln!("  spheres   Active sphere listing with phases");
    eprintln!("  bus       Bus state (tasks, events, cascades)");
    eprintln!("  me        ME observer + fitness + EventBus stats");
    eprintln!("  bridges   Bridge staleness check");
    eprintln!("  full      Everything above in one shot");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map_or("pulse", String::as_str);

    let result = match cmd {
        "pulse" => cmd_pulse(),
        "sweep" => cmd_sweep(),
        "field" => cmd_field(),
        "spheres" => cmd_spheres(),
        "bus" => cmd_bus(),
        "me" => cmd_me(),
        "bridges" => cmd_bridges(),
        "full" => cmd_full(),
        "--help" | "-h" | "help" => {
            print_help();
            return;
        }
        other => {
            eprintln!("Unknown command: {other}");
            eprintln!();
            print_help();
            std::process::exit(1);
        }
    };

    match serde_json::to_string_pretty(&result) {
        Ok(s) => println!("{s}"),
        Err(e) => eprintln!("JSON serialization error: {e}"),
    }
}
