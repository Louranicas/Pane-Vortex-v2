//! # M10: API Server
//!
//! Axum 0.8 HTTP server with CORS, body limits, and route registration.
//! Feature-gated behind `api`.
//!
//! ## Layer: L2 (Services)
//! ## Module: M10
//! ## Dependencies: L1 (M02, M03, M06), L3 (M15 `SharedState`), L4 (M16, M18), L7 (M29, M30)
//!
//! ## Route Groups (36 total)
//!
//! | Group | Count | Description |
//! |-------|-------|-------------|
//! | Core | 3 | `/health`, `/spheres`, `/ghosts` |
//! | Field | 8 | `/field/*` — r, decision, tunnels, spectrum |
//! | Sphere CRUD | 6 | `/sphere/{pane_id}/*` — register, deregister, memory, status, heartbeat |
//! | Sphere Advanced | 4 | `/sphere/{pane_id}/neighbors`, inbox, send, ack |
//! | Coupling | 2 | `/coupling/matrix`, `/coupling/weight` |
//! | Bus | 9 | `/bus/info`, tasks, events, submit, claim, complete, fail, cascade, cascades |
//! | Bridges | 1 | `/bridges/health` |

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use parking_lot::RwLock;
use serde::Deserialize;
use tower_http::cors::CorsLayer;

use crate::m1_foundation::{
    m01_core_types::{
        GhostTrace, PaneId, PaneStatus,
    },
    m02_error_handling::PvError,
    m03_config::PvConfig,
    m04_constants,
    m06_validation::{
        validate_frequency, validate_pane_id, validate_persona, validate_summary,
        validate_tool_name, validate_weight,
    },
};
use crate::m3_field::m15_app_state::SharedState;
use crate::m3_field::{m11_sphere::PaneSphere, m12_field_state::FieldState};
use crate::m4_coupling::m16_coupling_network::CouplingNetwork;
use crate::m4_coupling::m18_topology::neighbors;
use crate::m7_coordination::m29_ipc_bus::BusState;
use crate::m7_coordination::m30_bus_types::{BusTask, TaskTarget};
use crate::m7_coordination::m33_cascade::CascadeTracker;

// ──────────────────────────────────────────────────────────────
// AppContext — multi-state extractor
// ──────────────────────────────────────────────────────────────

/// Application context holding all state references needed by API handlers.
///
/// Passed via `axum::extract::State<AppContext>` to handlers that need
/// more than just `SharedState`. Implements `Clone` via `Arc` sharing.
#[derive(Clone)]
pub struct AppContext {
    /// Shared application state (spheres, field, decisions).
    pub state: SharedState,
    /// Coupling network state (phases, weights, connections).
    pub network: Arc<RwLock<CouplingNetwork>>,
    /// IPC bus state (tasks, events, subscribers).
    pub bus: Arc<RwLock<BusState>>,
    /// Cascade handoff tracker.
    pub cascade: Arc<RwLock<CascadeTracker>>,
}

// ──────────────────────────────────────────────────────────────
// Request body types
// ──────────────────────────────────────────────────────────────

/// Request body for `POST /sphere/{pane_id}/register`.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    /// Human-readable persona description.
    pub persona: String,
    /// Natural frequency (Hz).
    pub frequency: f64,
}

/// Request body for `POST /sphere/{pane_id}/memory`.
#[derive(Debug, Deserialize)]
pub struct MemoryRequest {
    /// Name of the tool that produced this memory.
    pub tool_name: String,
    /// Human-readable summary.
    pub summary: String,
}

/// Request body for `POST /sphere/{pane_id}/accept-ghost`.
#[derive(Debug, Deserialize)]
pub struct AcceptGhostRequest {
    /// ID of the ghost to accept.
    pub ghost_id: String,
}

/// Request body for `POST /sphere/{pane_id}/status`.
#[derive(Debug, Deserialize)]
pub struct StatusRequest {
    /// Status string: "Idle", "Working", "Blocked", or "Complete".
    pub status: String,
    /// Optional name of the last tool used.
    pub last_tool: Option<String>,
}

/// Request body for `POST /coupling/weight`.
#[derive(Debug, Deserialize)]
pub struct WeightRequest {
    /// Source sphere ID.
    pub from: String,
    /// Target sphere ID.
    pub to: String,
    /// Coupling weight (clamped to valid range).
    pub weight: f64,
}

/// Request body for `POST /sphere/{pane_id}/inbox/send`.
#[derive(Debug, Deserialize)]
pub struct MessageRequest {
    /// Sender sphere ID.
    pub from: String,
    /// Message content.
    pub content: String,
}

/// Request body for `POST /sphere/{pane_id}/inbox/ack`.
#[derive(Debug, Deserialize)]
pub struct AckRequest {
    /// Message ID to acknowledge.
    pub message_id: u64,
}

/// Request body for `POST /bus/submit`.
#[derive(Debug, Deserialize)]
pub struct TaskSubmitRequest {
    /// Human-readable description of the work.
    pub description: String,
    /// Target strategy: `any_idle`, `field_driven`, `willing`, or `specific`.
    #[serde(default = "default_target")]
    pub target: String,
    /// Submitter sphere ID.
    pub submitter: String,
    /// For "specific" target, the target sphere ID.
    pub target_pane_id: Option<String>,
}

fn default_target() -> String {
    "any_idle".to_owned()
}

/// Request body for `POST /bus/cascade`.
#[derive(Debug, Deserialize)]
pub struct CascadeRequest {
    /// Source sphere ID.
    pub source: String,
    /// Target sphere ID.
    pub target: String,
    /// Markdown brief describing the work context.
    pub brief: String,
}

/// Request body for `POST /sphere/{pane_id}/phase`.
#[derive(Debug, Deserialize)]
pub struct PhaseRequest {
    /// New phase value (radians, will be wrapped to [0, 2π)).
    pub phase: Option<f64>,
    /// New frequency value (Hz, clamped to valid range).
    pub frequency: Option<f64>,
}

/// Request body for `POST /sphere/{pane_id}/steer`.
#[derive(Debug, Deserialize)]
pub struct SteerRequest {
    /// Target phase (radians).
    pub target_phase: f64,
    /// Steering strength (clamped to [0.0, 2.0]).
    pub strength: f64,
}

// ──────────────────────────────────────────────────────────────
// Error response
// ──────────────────────────────────────────────────────────────

/// Axum-compatible error response.
pub struct ApiError(PvError);

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self.0 {
            PvError::SphereNotFound(_)
            | PvError::BusTaskNotFound(_)
            | PvError::ProposalNotFound(_) => StatusCode::NOT_FOUND,
            PvError::SphereAlreadyRegistered(_) => StatusCode::CONFLICT,
            PvError::SphereCapReached(_)
            | PvError::CascadeRateLimit { .. } => StatusCode::TOO_MANY_REQUESTS,
            PvError::NonFinite { .. }
            | PvError::OutOfRange { .. }
            | PvError::EmptyString { .. }
            | PvError::StringTooLong { .. }
            | PvError::InvalidChars { .. }
            | PvError::ConfigValidation(_) => StatusCode::BAD_REQUEST,
            PvError::BridgeConsentDenied { .. }
            | PvError::VotingClosed(_) => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = serde_json::json!({
            "error": self.0.to_string(),
        });

        (status, Json(body)).into_response()
    }
}

impl From<PvError> for ApiError {
    fn from(err: PvError) -> Self {
        Self(err)
    }
}

// ──────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────

/// Parse a status string into `PaneStatus`.
fn parse_status(s: &str) -> Result<PaneStatus, PvError> {
    match s {
        "Idle" | "idle" => Ok(PaneStatus::Idle),
        "Working" | "working" => Ok(PaneStatus::Working),
        "Blocked" | "blocked" => Ok(PaneStatus::Blocked),
        "Complete" | "complete" => Ok(PaneStatus::Complete),
        other => Err(PvError::ConfigValidation(format!(
            "invalid status: '{other}' (expected Idle|Working|Blocked|Complete)"
        ))),
    }
}

// ──────────────────────────────────────────────────────────────
// Core route handlers (3)
// ──────────────────────────────────────────────────────────────

/// GET /health -- Basic health check (V1-compatible fields).
async fn health_handler(State(ctx): State<AppContext>) -> impl IntoResponse {
    let (r, sphere_count, tick, fleet_mode, warmup) = {
        let guard = ctx.state.read();
        // Use cached field state (computed every tick) for accurate r.
        // Falls back to r_history for backward compat during warmup.
        let r = guard
            .cached_field
            .as_ref()
            .map_or_else(
                || guard.r_history.back().copied().unwrap_or(0.0),
                |fs| fs.order_parameter.r,
            );
        (r, guard.spheres.len(), guard.tick, guard.fleet_mode(), guard.warmup_remaining)
    };

    let (k, k_mod) = {
        let net = ctx.network.read();
        (net.k, net.k_modulation)
    };

    Json(serde_json::json!({
        "status": "healthy",
        "r": r,
        "spheres": sphere_count,
        "tick": tick,
        "fleet_mode": format!("{fleet_mode:?}"),
        "k": k,
        "k_modulation": k_mod,
        "warmup_remaining": warmup,
    }))
}

/// GET /spheres -- List all registered spheres.
async fn spheres_handler(State(ctx): State<AppContext>) -> impl IntoResponse {
    let spheres: Vec<serde_json::Value> = {
        let guard = ctx.state.read();
        guard
            .spheres
            .iter()
            .map(|(id, s)| {
                serde_json::json!({
                    "id": id.as_str(),
                    "persona": s.persona,
                    "status": format!("{}", s.status),
                    "phase": s.phase,
                    "frequency": s.frequency,
                    "memories": s.memories.len(),
                    "receptivity": s.receptivity,
                    "total_steps": s.total_steps,
                })
            })
            .collect()
    };

    Json(serde_json::json!({ "spheres": spheres }))
}

/// GET /ghosts -- List ghost traces.
async fn ghosts_handler(State(ctx): State<AppContext>) -> impl IntoResponse {
    let ghosts: Vec<serde_json::Value> = {
        let guard = ctx.state.read();
        guard
            .ghosts
            .iter()
            .map(|g| {
                serde_json::json!({
                    "id": g.id.as_str(),
                    "persona": g.persona,
                    "deregistered_at": g.deregistered_at,
                    "total_steps_lived": g.total_steps_lived,
                    "memory_count": g.memory_count,
                    "top_tools": g.top_tools,
                })
            })
            .collect()
    };

    Json(serde_json::json!({ "ghosts": ghosts }))
}

// ──────────────────────────────────────────────────────────────
// Field route handlers (8)
// ──────────────────────────────────────────────────────────────

/// GET /field -- Full field state snapshot.
async fn field_handler(State(ctx): State<AppContext>) -> impl IntoResponse {
    let field = {
        let guard = ctx.state.read();
        let net = ctx.network.read();
        FieldState::compute(&guard.spheres, net.k_modulation, guard.tick)
    };
    Json(serde_json::json!(field))
}

/// GET /field/r -- Current order parameter r.
async fn field_r_handler(State(ctx): State<AppContext>) -> impl IntoResponse {
    let (r, psi) = {
        let guard = ctx.state.read();
        // Use cached field state for accurate r (matches /field endpoint).
        guard
            .cached_field
            .as_ref()
            .map_or((0.0, 0.0), |fs| (fs.order_parameter.r, fs.order_parameter.psi))
    };
    Json(serde_json::json!({
        "r": r,
        "psi": psi,
    }))
}

/// GET /field/decision -- Current field decision.
async fn field_decision_handler(
    State(ctx): State<AppContext>,
) -> impl IntoResponse {
    let decision = {
        let guard = ctx.state.read();
        let net = ctx.network.read();
        let field = FieldState::compute(&guard.spheres, net.k_modulation, guard.tick);
        crate::m3_field::m12_field_state::FieldDecision::compute(
            &guard.spheres,
            &field,
            &guard.r_history,
            guard.tick,
        )
    };
    Json(serde_json::json!(decision))
}

/// GET /field/decisions -- Decision history.
async fn field_decisions_handler(State(ctx): State<AppContext>) -> impl IntoResponse {
    let history: Vec<serde_json::Value> = {
        let guard = ctx.state.read();
        guard
            .decision_history
            .iter()
            .map(|d| {
                serde_json::json!({
                    "tick": d.tick,
                    "action": format!("{}", d.action),
                    "r": d.r,
                    "k_mod": d.k_mod,
                    "sphere_count": d.sphere_count,
                })
            })
            .collect()
    };
    Json(serde_json::json!({ "decisions": history }))
}

/// GET /field/chimera -- Current chimera state.
async fn field_chimera_handler(State(ctx): State<AppContext>) -> impl IntoResponse {
    let chimera = {
        let guard = ctx.state.read();
        let net = ctx.network.read();
        let field = FieldState::compute(&guard.spheres, net.k_modulation, guard.tick);
        field.chimera
    };
    Json(serde_json::json!(chimera))
}

/// GET /field/tunnels -- Active tunnels (buoy overlaps).
async fn field_tunnels_handler(State(ctx): State<AppContext>) -> impl IntoResponse {
    let tunnels = {
        let guard = ctx.state.read();
        let net = ctx.network.read();
        let field = FieldState::compute(&guard.spheres, net.k_modulation, guard.tick);
        field.tunnels
    };
    Json(serde_json::json!({ "tunnels": tunnels, "count": tunnels.len() }))
}

/// GET `/field/k` -- Coupling K and `k_modulation`.
async fn field_k_handler(State(ctx): State<AppContext>) -> impl IntoResponse {
    let (k, k_mod, auto_k) = {
        let net = ctx.network.read();
        (net.k, net.k_modulation, net.auto_k)
    };
    Json(serde_json::json!({
        "k": k,
        "k_modulation": k_mod,
        "auto_k": auto_k,
    }))
}

/// GET /field/spectrum -- Harmonic spectrum.
async fn field_spectrum_handler(State(ctx): State<AppContext>) -> impl IntoResponse {
    let harmonics = {
        let guard = ctx.state.read();
        let net = ctx.network.read();
        let field = FieldState::compute(&guard.spheres, net.k_modulation, guard.tick);
        field.harmonics
    };
    Json(serde_json::json!(harmonics))
}

// ──────────────────────────────────────────────────────────────
// Sphere CRUD handlers (6)
// ──────────────────────────────────────────────────────────────

/// GET `/sphere/{pane_id}` -- Single sphere detail.
async fn sphere_detail_handler(
    State(ctx): State<AppContext>,
    Path(pane_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    validate_pane_id(&pane_id)?;
    let pid = PaneId::new(&pane_id);

    let sphere_json = {
        let guard = ctx.state.read();
        let sphere = guard
            .spheres
            .get(&pid)
            .ok_or_else(|| PvError::SphereNotFound(pane_id.clone()))?;
        serde_json::json!({
            "id": sphere.id.as_str(),
            "persona": sphere.persona,
            "status": format!("{}", sphere.status),
            "phase": sphere.phase,
            "frequency": sphere.frequency,
            "momentum": sphere.momentum,
            "memories": sphere.memories.len(),
            "buoys": sphere.buoys.len(),
            "receptivity": sphere.receptivity,
            "total_steps": sphere.total_steps,
            "has_worked": sphere.has_worked,
            "registered_at": sphere.registered_at,
            "last_heartbeat": sphere.last_heartbeat,
            "last_tool": sphere.last_tool,
            "inbox_count": sphere.inbox.len(),
            "activity_30s": sphere.activity_30s,
            "activity_5m": sphere.activity_5m,
            "activity_30m": sphere.activity_30m,
        })
    };

    Ok(Json(sphere_json))
}

/// POST `/sphere/{pane_id}/register` -- Register a new sphere.
async fn register_handler(
    State(ctx): State<AppContext>,
    Path(pane_id): Path<String>,
    Json(body): Json<RegisterRequest>,
) -> Result<impl IntoResponse, ApiError> {
    validate_pane_id(&pane_id)?;
    validate_persona(&body.persona)?;
    let freq = validate_frequency(body.frequency)?;

    let pid = PaneId::new(&pane_id);

    {
        let guard = ctx.state.read();
        if guard.spheres.contains_key(&pid) {
            return Err(PvError::SphereAlreadyRegistered(pane_id).into());
        }
        if guard.spheres.len() >= m04_constants::SPHERE_CAP {
            return Err(PvError::SphereCapReached(m04_constants::SPHERE_CAP).into());
        }
    }

    let mut sphere = PaneSphere::new(pid.clone(), body.persona.clone(), freq)?;

    // Ghost reincarnation: if a ghost trace exists for this ID, restore its phase
    let ghost_restored = {
        let mut guard = ctx.state.write();
        if let Some(ghost) = guard.accept_ghost(&pid) {
            sphere.phase = ghost.phase_at_departure;
            true
        } else {
            false
        }
    };

    let phase = sphere.phase;

    {
        let mut guard = ctx.state.write();
        guard.spheres.insert(pid.clone(), sphere);
        guard.state_changes += 1;
        guard.mark_dirty();
    }

    // Register in coupling network
    {
        let mut net = ctx.network.write();
        net.register(pid, phase, freq);
    }

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "registered": pane_id,
            "persona": body.persona,
            "frequency": freq,
            "ghost_restored": ghost_restored,
        })),
    ))
}

/// POST `/sphere/{pane_id}/deregister` -- Deregister a sphere (creates ghost).
async fn deregister_handler(
    State(ctx): State<AppContext>,
    Path(pane_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    validate_pane_id(&pane_id)?;
    let pid = PaneId::new(&pane_id);

    // Collect coupling neighbors BEFORE removing from network
    let strongest_neighbors = {
        let net = ctx.network.read();
        let mut neighbor_weights: Vec<(String, f64)> = net
            .connections
            .iter()
            .filter(|c| c.from == pid)
            .map(|c| (c.to.as_str().to_owned(), c.weight * c.type_weight))
            .collect();
        neighbor_weights.sort_unstable_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });
        neighbor_weights
    };

    let ghost = {
        let mut guard = ctx.state.write();
        let sphere = guard
            .spheres
            .remove(&pid)
            .ok_or_else(|| PvError::SphereNotFound(pane_id.clone()))?;

        let memory_count = sphere.memories.len();

        // Compute top tools from memory frequency
        let top_tools = {
            let mut tool_counts: std::collections::HashMap<&str, usize> =
                std::collections::HashMap::new();
            for mem in &sphere.memories {
                *tool_counts.entry(&mem.tool_name).or_insert(0) += 1;
            }
            let mut sorted: Vec<(&str, usize)> = tool_counts.into_iter().collect();
            sorted.sort_unstable_by(|a, b| b.1.cmp(&a.1));
            sorted.into_iter().take(10).map(|(t, _)| t.to_owned()).collect()
        };

        let ghost = GhostTrace {
            id: sphere.id,
            persona: sphere.persona,
            deregistered_at: guard.tick,
            total_steps_lived: sphere.total_steps,
            memory_count,
            top_tools,
            phase_at_departure: sphere.phase,
            receptivity: sphere.receptivity,
            work_signature: sphere.work_signature,
            strongest_neighbors,
        };
        guard.add_ghost(ghost.clone());
        guard.state_changes += 1;
        guard.mark_dirty();
        ghost
    };

    // Deregister from coupling network
    {
        let mut net = ctx.network.write();
        net.deregister(&PaneId::new(&pane_id));
    }

    Ok(Json(serde_json::json!({
        "deregistered": pane_id,
        "ghost": {
            "total_steps": ghost.total_steps_lived,
            "memory_count": ghost.memory_count,
        },
    })))
}

/// POST `/sphere/{pane_id}/accept-ghost` -- Accept a ghost trace.
///
/// Consumes the ghost (removes from ghost list) and returns its data.
/// The calling sphere can use the ghost's memory count and tool history
/// to bootstrap its own state.
async fn accept_ghost_handler(
    State(ctx): State<AppContext>,
    Path(pane_id): Path<String>,
    Json(body): Json<AcceptGhostRequest>,
) -> Result<impl IntoResponse, ApiError> {
    validate_pane_id(&pane_id)?;

    let pid = PaneId::new(&pane_id);
    let ghost_pid = PaneId::new(&body.ghost_id);

    let mut guard = ctx.state.write();

    // Verify the accepting sphere exists
    if !guard.spheres.contains_key(&pid) {
        return Err(ApiError(PvError::SphereNotFound(pane_id)));
    }

    // Consume the ghost
    let ghost = guard
        .accept_ghost(&ghost_pid)
        .ok_or_else(|| PvError::SphereNotFound(body.ghost_id.clone()))?;

    Ok(Json(serde_json::json!({
        "accepted_by": pane_id,
        "ghost_id": ghost.id.as_str(),
        "ghost_persona": ghost.persona,
        "ghost_steps": ghost.total_steps_lived,
        "ghost_memory_count": ghost.memory_count,
        "ghost_top_tools": ghost.top_tools,
    })))
}

/// POST `/sphere/{pane_id}/memory` -- Record a memory.
async fn memory_handler(
    State(ctx): State<AppContext>,
    Path(pane_id): Path<String>,
    Json(body): Json<MemoryRequest>,
) -> Result<impl IntoResponse, ApiError> {
    validate_pane_id(&pane_id)?;
    validate_tool_name(&body.tool_name)?;
    validate_summary(&body.summary)?;

    let pid = PaneId::new(&pane_id);

    let memory_id = {
        let mut guard = ctx.state.write();
        let sphere = guard
            .spheres
            .get_mut(&pid)
            .ok_or_else(|| PvError::SphereNotFound(pane_id.clone()))?;
        let mid = sphere.record_memory(body.tool_name.clone(), body.summary.clone());
        sphere.touch_heartbeat();
        guard.state_changes += 1;
        guard.mark_dirty();
        mid
    };

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "pane_id": pane_id,
            "memory_id": memory_id,
            "tool_name": body.tool_name,
        })),
    ))
}

/// POST `/sphere/{pane_id}/status` -- Update sphere status.
async fn status_handler(
    State(ctx): State<AppContext>,
    Path(pane_id): Path<String>,
    Json(body): Json<StatusRequest>,
) -> Result<impl IntoResponse, ApiError> {
    validate_pane_id(&pane_id)?;
    let new_status = parse_status(&body.status)?;

    let pid = PaneId::new(&pane_id);

    {
        let mut guard = ctx.state.write();
        let sphere = guard
            .spheres
            .get_mut(&pid)
            .ok_or_else(|| PvError::SphereNotFound(pane_id.clone()))?;
        sphere.status = new_status;
        if let Some(ref tool) = body.last_tool {
            sphere.last_tool.clone_from(tool);
        }
        sphere.touch_heartbeat();
        guard.state_changes += 1;
    };

    Ok(Json(serde_json::json!({
        "pane_id": pane_id,
        "status": body.status,
    })))
}

/// POST `/sphere/{pane_id}/heartbeat` -- Touch heartbeat.
async fn heartbeat_handler(
    State(ctx): State<AppContext>,
    Path(pane_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    validate_pane_id(&pane_id)?;
    let pid = PaneId::new(&pane_id);

    {
        let mut guard = ctx.state.write();
        let sphere = guard
            .spheres
            .get_mut(&pid)
            .ok_or_else(|| PvError::SphereNotFound(pane_id.clone()))?;
        sphere.touch_heartbeat();
    };

    Ok(Json(serde_json::json!({
        "pane_id": pane_id,
        "heartbeat": "ok",
    })))
}

// ──────────────────────────────────────────────────────────────
// Gap fix handlers (3) — V1 compatibility
// ──────────────────────────────────────────────────────────────

/// POST `/sphere/{pane_id}/phase` -- Update phase and/or frequency (Gap 1).
async fn phase_handler(
    State(ctx): State<AppContext>,
    Path(pane_id): Path<String>,
    Json(body): Json<PhaseRequest>,
) -> Result<impl IntoResponse, ApiError> {
    validate_pane_id(&pane_id)?;
    let pid = PaneId::new(&pane_id);

    // Validate inputs before acquiring lock
    let validated_phase = body.phase.map(|p| {
        if p.is_finite() { Ok(p.rem_euclid(std::f64::consts::TAU)) }
        else { Err(PvError::NonFinite { field: "phase", value: p }) }
    }).transpose()?;
    let validated_freq = body.frequency.map(validate_frequency).transpose()?;

    let (new_phase, new_freq) = {
        let mut guard = ctx.state.write();
        if !guard.spheres.contains_key(&pid) {
            return Err(PvError::SphereNotFound(pane_id).into());
        }
        if let Some(phase) = validated_phase {
            if let Some(s) = guard.spheres.get_mut(&pid) { s.phase = phase; }
        }
        if let Some(freq) = validated_freq {
            if let Some(s) = guard.spheres.get_mut(&pid) {
                s.frequency = freq;
                s.base_frequency = freq;
            }
        }
        if let Some(s) = guard.spheres.get_mut(&pid) { s.touch_heartbeat(); }
        guard.state_changes += 1;
        guard.mark_dirty();
        let s = &guard.spheres[&pid];
        (s.phase, s.frequency)
    };

    // Sync to coupling network
    if let Some(freq) = body.frequency {
        let mut net = ctx.network.write();
        if let Some(net_freq) = net.frequencies.get_mut(&pid) {
            *net_freq = validate_frequency(freq)?;
        }
    }

    Ok(Json(serde_json::json!({
        "pane_id": pane_id,
        "phase": new_phase,
        "frequency": new_freq,
    })))
}

/// POST `/sphere/{pane_id}/steer` -- Steer phase toward target (Gap 2).
async fn steer_handler(
    State(ctx): State<AppContext>,
    Path(pane_id): Path<String>,
    Json(body): Json<SteerRequest>,
) -> Result<impl IntoResponse, ApiError> {
    validate_pane_id(&pane_id)?;
    if !body.target_phase.is_finite() {
        return Err(PvError::NonFinite { field: "target_phase", value: body.target_phase }.into());
    }
    let pid = PaneId::new(&pane_id);
    let target = body.target_phase.rem_euclid(std::f64::consts::TAU);
    let strength = body.strength.clamp(0.0, 2.0);

    {
        let mut guard = ctx.state.write();
        let sphere = guard
            .spheres
            .get_mut(&pid)
            .ok_or_else(|| PvError::SphereNotFound(pane_id.clone()))?;
        let effective = strength * sphere.receptivity;
        sphere.steer_toward(target, effective);
        sphere.touch_heartbeat();
        guard.state_changes += 1;
    };

    Ok(Json(serde_json::json!({
        "pane_id": pane_id,
        "target_phase": target,
        "strength": strength,
    })))
}

/// GET /bus/suggestions -- Field-driven suggestions (Gap 3 — stub).
async fn bus_suggestions_handler(State(ctx): State<AppContext>) -> impl IntoResponse {
    let bus = ctx.bus.read();
    let suggestions: Vec<&serde_json::Value> = bus.recent_suggestions(20);
    Json(serde_json::json!({
        "suggestions": suggestions,
        "total_generated": bus.total_suggestions(),
    }))
}

// ──────────────────────────────────────────────────────────────
// Sphere advanced handlers (4)
// ──────────────────────────────────────────────────────────────

/// GET `/sphere/{pane_id}/neighbors` -- Coupling neighbors.
async fn neighbors_handler(
    State(ctx): State<AppContext>,
    Path(pane_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    validate_pane_id(&pane_id)?;
    let pid = PaneId::new(&pane_id);

    // Verify sphere exists
    let exists = {
        let guard = ctx.state.read();
        guard.spheres.contains_key(&pid)
    };
    if !exists {
        return Err(PvError::SphereNotFound(pane_id).into());
    }

    let neighbor_list = {
        let net = ctx.network.read();
        neighbors(&net, &pid)
    };

    let json_list: Vec<serde_json::Value> = neighbor_list
        .iter()
        .map(|n| {
            serde_json::json!({
                "id": n.id.as_str(),
                "effective_weight": n.effective_weight,
                "phase_diff": n.phase_diff,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "pane_id": pane_id,
        "neighbors": json_list,
    })))
}

/// GET `/sphere/{pane_id}/inbox` -- Pending messages.
async fn inbox_handler(
    State(ctx): State<AppContext>,
    Path(pane_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    validate_pane_id(&pane_id)?;
    let pid = PaneId::new(&pane_id);

    let messages = {
        let guard = ctx.state.read();
        let sphere = guard
            .spheres
            .get(&pid)
            .ok_or_else(|| PvError::SphereNotFound(pane_id.clone()))?;
        sphere
            .inbox
            .iter()
            .map(|m| {
                serde_json::json!({
                    "id": m.id,
                    "from": m.from,
                    "content": m.content,
                    "received_at": m.received_at,
                    "acknowledged": m.acknowledged,
                })
            })
            .collect::<Vec<serde_json::Value>>()
    };

    Ok(Json(serde_json::json!({
        "pane_id": pane_id,
        "messages": messages,
    })))
}

/// POST `/sphere/{pane_id}/inbox/send` -- Send message to sphere.
async fn inbox_send_handler(
    State(ctx): State<AppContext>,
    Path(pane_id): Path<String>,
    Json(body): Json<MessageRequest>,
) -> Result<impl IntoResponse, ApiError> {
    validate_pane_id(&pane_id)?;
    validate_pane_id(&body.from)?;
    validate_summary(&body.content)?;

    let pid = PaneId::new(&pane_id);

    let message_id = {
        let mut guard = ctx.state.write();
        let sphere = guard
            .spheres
            .get_mut(&pid)
            .ok_or_else(|| PvError::SphereNotFound(pane_id.clone()))?;
        let mid = sphere.receive_message(body.from.clone(), body.content.clone());
        guard.state_changes += 1;
        mid
    };

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "pane_id": pane_id,
            "message_id": message_id,
            "from": body.from,
        })),
    ))
}

/// POST `/sphere/{pane_id}/inbox/ack` -- Acknowledge message.
async fn inbox_ack_handler(
    State(ctx): State<AppContext>,
    Path(pane_id): Path<String>,
    Json(body): Json<AckRequest>,
) -> Result<impl IntoResponse, ApiError> {
    validate_pane_id(&pane_id)?;
    let pid = PaneId::new(&pane_id);

    let acknowledged = {
        let mut guard = ctx.state.write();
        let sphere = guard
            .spheres
            .get_mut(&pid)
            .ok_or_else(|| PvError::SphereNotFound(pane_id.clone()))?;
        sphere.acknowledge_message(body.message_id)
    };

    if acknowledged {
        Ok(Json(serde_json::json!({
            "pane_id": pane_id,
            "message_id": body.message_id,
            "acknowledged": true,
        })))
    } else {
        Err(PvError::BusTaskNotFound(format!(
            "inbox message {} not found in sphere {pane_id}",
            body.message_id
        ))
        .into())
    }
}

// ──────────────────────────────────────────────────────────────
// Coupling handlers (2)
// ──────────────────────────────────────────────────────────────

/// GET /coupling/matrix -- Full coupling matrix.
async fn coupling_matrix_handler(State(ctx): State<AppContext>) -> impl IntoResponse {
    let matrix = {
        let net = ctx.network.read();
        let raw = net.coupling_matrix();
        raw.iter()
            .map(|((from, to), w)| {
                serde_json::json!({
                    "from": from.as_str(),
                    "to": to.as_str(),
                    "weight": w,
                })
            })
            .collect::<Vec<serde_json::Value>>()
    };

    Json(serde_json::json!({
        "matrix": matrix,
        "count": matrix.len(),
    }))
}

/// POST /coupling/weight -- Set weight between two spheres.
async fn coupling_weight_handler(
    State(ctx): State<AppContext>,
    Json(body): Json<WeightRequest>,
) -> Result<impl IntoResponse, ApiError> {
    validate_pane_id(&body.from)?;
    validate_pane_id(&body.to)?;
    let weight = validate_weight(body.weight)?;

    let from = PaneId::new(&body.from);
    let to = PaneId::new(&body.to);

    {
        let mut net = ctx.network.write();
        net.set_weight(&from, &to, weight);
    }

    Ok(Json(serde_json::json!({
        "from": body.from,
        "to": body.to,
        "weight": weight,
    })))
}

// ──────────────────────────────────────────────────────────────
// Bus handlers (3)
// ──────────────────────────────────────────────────────────────

/// GET /bus/info -- Bus state summary.
async fn bus_info_handler(State(ctx): State<AppContext>) -> impl IntoResponse {
    let (task_count, event_count, subscriber_count, cascade_count) = {
        let bus = ctx.bus.read();
        (
            bus.task_count(),
            bus.event_count(),
            bus.subscriber_count(),
            bus.cascade_count(),
        )
    };

    Json(serde_json::json!({
        "tasks": task_count,
        "events": event_count,
        "subscribers": subscriber_count,
        "cascade_count": cascade_count,
    }))
}

/// GET /bus/tasks -- Pending tasks.
async fn bus_tasks_handler(State(ctx): State<AppContext>) -> impl IntoResponse {
    let tasks = {
        let bus = ctx.bus.read();
        bus.pending_tasks()
            .iter()
            .map(|t| {
                serde_json::json!({
                    "id": t.id.as_str(),
                    "description": t.description,
                    "target": format!("{}", t.target),
                    "status": format!("{}", t.status),
                    "submitted_by": t.submitted_by.as_str(),
                    "submitted_at": t.submitted_at,
                })
            })
            .collect::<Vec<serde_json::Value>>()
    };

    Json(serde_json::json!({ "tasks": tasks }))
}

/// GET /bus/events -- Recent events (last 50).
async fn bus_events_handler(State(ctx): State<AppContext>) -> impl IntoResponse {
    let events = {
        let bus = ctx.bus.read();
        bus.recent_events(50)
            .iter()
            .map(|e| {
                serde_json::json!({
                    "event_type": e.event_type,
                    "data": e.data,
                    "tick": e.tick,
                    "timestamp": e.timestamp,
                })
            })
            .collect::<Vec<serde_json::Value>>()
    };

    Json(serde_json::json!({ "events": events }))
}

/// POST /bus/submit -- Submit a task via HTTP (alternative to IPC socket).
async fn bus_submit_handler(
    State(ctx): State<AppContext>,
    Json(body): Json<TaskSubmitRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let submitter = PaneId::new(body.submitter);
    validate_pane_id(submitter.as_str())?;
    let target = match body.target.to_lowercase().as_str() {
        "any_idle" | "anyidle" => TaskTarget::AnyIdle,
        "field_driven" | "fielddriven" => TaskTarget::FieldDriven,
        "willing" => TaskTarget::Willing,
        "specific" => {
            let pane_id = body.target_pane_id.ok_or_else(|| {
                PvError::ConfigValidation(
                    "target_pane_id required for specific target".into(),
                )
            })?;
            TaskTarget::Specific {
                pane_id: PaneId::new(pane_id),
            }
        }
        other => {
            return Err(ApiError(PvError::ConfigValidation(
                format!("unknown target: {other}, expected any_idle|field_driven|willing|specific"),
            )));
        }
    };

    let task = BusTask::new(body.description, target, submitter);
    let task_id = {
        let mut bus = ctx.bus.write();
        bus.submit_task(task)?
    };

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "task_id": task_id.as_str(),
            "status": "Pending",
        })),
    ))
}

/// POST `/bus/tasks/{task_id}/claim` -- Claim a pending task.
async fn bus_task_claim_handler(
    State(ctx): State<AppContext>,
    Path(task_id): Path<String>,
    Json(body): Json<TaskClaimRequest>,
) -> Result<impl IntoResponse, ApiError> {
    validate_pane_id(&body.claimer)?;
    let claim_sphere = PaneId::new(body.claimer.clone());

    let ok = {
        let mut bus = ctx.bus.write();
        let task = bus.get_task_mut(&task_id).ok_or_else(|| {
            PvError::SphereNotFound(format!("task not found: {task_id}"))
        })?;
        task.claim(claim_sphere)
    };

    if ok {
        // Publish task.claimed event
        {
            let tick = ctx.state.read().tick;
            let event = crate::m7_coordination::m30_bus_types::BusEvent::new(
                "task.claimed".into(),
                serde_json::json!({
                    "task_id": task_id,
                    "claimer": body.claimer,
                }),
                tick,
            );
            ctx.bus.write().publish_event(event);
        }
        Ok(Json(serde_json::json!({
            "task_id": task_id,
            "status": "Claimed",
            "claimer": body.claimer,
        })))
    } else {
        Err(ApiError(PvError::ConfigValidation(
            format!("task {task_id} cannot be claimed (not pending)"),
        )))
    }
}

/// POST `/bus/tasks/{task_id}/complete` -- Mark a claimed task as completed.
async fn bus_task_complete_handler(
    State(ctx): State<AppContext>,
    Path(task_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let completed = {
        let mut bus = ctx.bus.write();
        let task = bus.get_task_mut(&task_id).ok_or_else(|| {
            PvError::SphereNotFound(format!("task not found: {task_id}"))
        })?;
        task.complete()
    };

    if completed {
        {
            let tick = ctx.state.read().tick;
            let event = crate::m7_coordination::m30_bus_types::BusEvent::new(
                "task.completed".into(),
                serde_json::json!({ "task_id": task_id }),
                tick,
            );
            ctx.bus.write().publish_event(event);
        }
        Ok(Json(serde_json::json!({
            "task_id": task_id,
            "status": "Completed",
        })))
    } else {
        Err(ApiError(PvError::ConfigValidation(
            format!("task {task_id} cannot be completed (not claimed)"),
        )))
    }
}

/// POST `/bus/tasks/{task_id}/fail` -- Mark a claimed task as failed.
async fn bus_task_fail_handler(
    State(ctx): State<AppContext>,
    Path(task_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let failed = {
        let mut bus = ctx.bus.write();
        let task = bus.get_task_mut(&task_id).ok_or_else(|| {
            PvError::SphereNotFound(format!("task not found: {task_id}"))
        })?;
        task.fail()
    };

    if failed {
        {
            let tick = ctx.state.read().tick;
            let event = crate::m7_coordination::m30_bus_types::BusEvent::new(
                "task.failed".into(),
                serde_json::json!({ "task_id": task_id }),
                tick,
            );
            ctx.bus.write().publish_event(event);
        }
        Ok(Json(serde_json::json!({
            "task_id": task_id,
            "status": "Failed",
        })))
    } else {
        Err(ApiError(PvError::ConfigValidation(
            format!("task {task_id} cannot be failed (not claimed)"),
        )))
    }
}

/// Request body for task claim.
#[derive(Debug, Deserialize)]
struct TaskClaimRequest {
    /// ID of the sphere claiming the task.
    claimer: String,
}

/// POST /bus/cascade -- Initiate a cascade handoff via HTTP.
async fn bus_cascade_handler(
    State(ctx): State<AppContext>,
    Json(body): Json<CascadeRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let source = PaneId::new(body.source);
    let target = PaneId::new(body.target);

    let index = {
        let mut cascade = ctx.cascade.write();
        cascade.initiate(source.clone(), target.clone(), body.brief)?
    };

    // Publish cascade event to bus
    {
        use crate::m7_coordination::m30_bus_types::BusEvent;
        let tick = {
            let guard = ctx.state.read();
            guard.tick
        };
        let event = BusEvent::new(
            "cascade.initiated".into(),
            serde_json::json!({
                "source": source.as_str(),
                "target": target.as_str(),
                "index": index,
            }),
            tick,
        );
        let mut bus = ctx.bus.write();
        bus.publish_event(event);
    }

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "cascade_index": index,
            "source": source.as_str(),
            "target": target.as_str(),
            "status": "dispatched",
        })),
    ))
}

/// GET /bus/cascades -- List pending cascades.
async fn bus_cascades_handler(State(ctx): State<AppContext>) -> impl IntoResponse {
    let cascades = {
        let tracker = ctx.cascade.read();
        tracker
            .pending_cascades()
            .iter()
            .map(|(idx, c)| {
                serde_json::json!({
                    "index": idx,
                    "source": c.source.as_str(),
                    "target": c.target.as_str(),
                    "brief_len": c.brief.len(),
                    "depth": c.depth,
                    "elapsed_secs": c.elapsed_secs(),
                })
            })
            .collect::<Vec<serde_json::Value>>()
    };

    Json(serde_json::json!({ "cascades": cascades }))
}

// ──────────────────────────────────────────────────────────────
// Bridge handlers (1)
// ──────────────────────────────────────────────────────────────

/// GET /bridges/health -- Bridge staleness summary.
async fn bridges_health_handler(State(ctx): State<AppContext>) -> impl IntoResponse {
    let staleness = {
        let guard = ctx.state.read();
        guard.prev_bridge_staleness.clone()
    };

    Json(serde_json::json!({
        "synthex_stale": staleness.synthex_stale,
        "nexus_stale": staleness.nexus_stale,
        "povm_stale": staleness.povm_stale,
        "rm_stale": staleness.rm_stale,
        "vms_stale": staleness.vms_stale,
        "me_stale": staleness.me_stale,
    }))
}

// ──────────────────────────────────────────────────────────────
// Governance handlers (feature-gated)
// ──────────────────────────────────────────────────────────────

/// Request body for `POST /field/propose`.
#[cfg(feature = "governance")]
#[derive(Debug, Deserialize)]
pub struct ProposeRequest {
    /// Proposer sphere ID.
    pub proposer: String,
    /// Parameter to change: `r_target`, `k_mod_budget_max`, or `coupling_steps`.
    pub parameter: String,
    /// Proposed value.
    pub value: f64,
    /// Human-readable reason.
    pub reason: String,
}

/// Request body for `POST /sphere/{pane_id}/vote/{proposal_id}`.
#[cfg(feature = "governance")]
#[derive(Debug, Deserialize)]
pub struct VoteRequest {
    /// Vote choice: `approve`, `reject`, or `abstain`.
    pub choice: String,
}

/// POST `/field/propose` -- Submit a governance proposal.
#[cfg(feature = "governance")]
async fn propose_handler(
    State(ctx): State<AppContext>,
    Json(body): Json<ProposeRequest>,
) -> Result<impl IntoResponse, ApiError> {
    use crate::m8_governance::m37_proposals::ProposableParameter;

    validate_pane_id(&body.proposer)?;

    let parameter = match body.parameter.as_str() {
        "r_target" => ProposableParameter::RTarget,
        "k_mod_budget_max" => ProposableParameter::KModBudgetMax,
        "coupling_steps" => ProposableParameter::CouplingSteps,
        "opt_out_policy" => ProposableParameter::OptOutPolicy,
        other if other.starts_with("sphere_override:") => {
            let target = other.strip_prefix("sphere_override:").unwrap_or("");
            ProposableParameter::SphereOverride {
                target_sphere: target.to_owned(),
            }
        }
        other => {
            return Err(PvError::ConfigValidation(format!(
                "unknown parameter: {other}"
            ))
            .into());
        }
    };

    let current_value = match &parameter {
        ProposableParameter::RTarget => m04_constants::R_TARGET_BASE,
        ProposableParameter::KModBudgetMax => m04_constants::K_MOD_BUDGET_MAX,
        ProposableParameter::CouplingSteps => {
            #[allow(clippy::cast_precision_loss)]
            let v = m04_constants::COUPLING_STEPS_PER_TICK as f64;
            v
        }
        ProposableParameter::SphereOverride { .. } => 1.0,
        ProposableParameter::OptOutPolicy => 0.0,
    };

    let proposal_id = {
        let mut guard = ctx.state.write();
        let tick = guard.tick;
        guard.proposal_manager.submit(
            PaneId::new(&body.proposer),
            parameter,
            body.value,
            current_value,
            body.reason.clone(),
            tick,
        )?
    };

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "proposal_id": proposal_id,
            "parameter": body.parameter,
            "proposed_value": body.value,
            "current_value": current_value,
        })),
    ))
}

/// POST `/sphere/{pane_id}/vote/{proposal_id}` -- Vote on a proposal.
#[cfg(feature = "governance")]
async fn vote_handler(
    State(ctx): State<AppContext>,
    Path((pane_id, proposal_id)): Path<(String, String)>,
    Json(body): Json<VoteRequest>,
) -> Result<impl IntoResponse, ApiError> {
    use crate::m8_governance::m37_proposals::VoteChoice;

    validate_pane_id(&pane_id)?;

    let choice = match body.choice.as_str() {
        "approve" => VoteChoice::Approve,
        "reject" => VoteChoice::Reject,
        "abstain" => VoteChoice::Abstain,
        other => {
            return Err(
                PvError::ConfigValidation(format!("unknown vote choice: {other}")).into(),
            );
        }
    };

    {
        let mut guard = ctx.state.write();
        let tick = guard.tick;
        guard
            .proposal_manager
            .vote(&proposal_id, PaneId::new(&pane_id), choice, tick)?;
    }

    Ok(Json(serde_json::json!({
        "proposal_id": proposal_id,
        "voter": pane_id,
        "choice": body.choice,
    })))
}

/// GET `/field/proposals` -- List all proposals.
#[cfg(feature = "governance")]
async fn proposals_handler(State(ctx): State<AppContext>) -> impl IntoResponse {
    let proposals: Vec<serde_json::Value> = {
        let guard = ctx.state.read();
        guard
            .proposal_manager
            .all()
            .iter()
            .map(|p| {
                serde_json::json!({
                    "id": p.id,
                    "proposer": p.proposer.as_str(),
                    "parameter": format!("{:?}", p.parameter),
                    "proposed_value": p.proposed_value,
                    "current_value": p.current_value,
                    "reason": p.reason,
                    "status": format!("{:?}", p.status),
                    "votes": p.votes.len(),
                    "submitted_at_tick": p.submitted_at_tick,
                })
            })
            .collect()
    };

    Json(serde_json::json!({ "proposals": proposals }))
}

/// GET `/sphere/{pane_id}/consent` -- Get sphere consent posture.
#[cfg(feature = "governance")]
async fn sphere_consent_handler(
    State(ctx): State<AppContext>,
    Path(pane_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    validate_pane_id(&pane_id)?;
    let pid = PaneId::new(&pane_id);

    let consent_info = {
        let guard = ctx.state.read();
        let sphere = guard
            .spheres
            .get(&pid)
            .ok_or_else(|| PvError::SphereNotFound(pane_id.clone()))?;
        serde_json::json!({
            "pane_id": pane_id,
            "opt_out_hebbian": sphere.opt_out_hebbian,
            "opt_out_cross_activation": sphere.opt_out_cross_activation,
            "opt_out_external_modulation": sphere.opt_out_external_modulation,
            "opt_out_observation": sphere.opt_out_observation,
            "receptivity": sphere.receptivity,
            "preferred_r": sphere.preferred_r,
        })
    };

    Ok(Json(consent_info))
}

/// GET `/sphere/{pane_id}/data-manifest` -- Data sovereignty manifest.
#[cfg(feature = "governance")]
async fn data_manifest_handler(
    State(ctx): State<AppContext>,
    Path(pane_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    validate_pane_id(&pane_id)?;
    let pid = PaneId::new(&pane_id);

    let manifest = {
        let guard = ctx.state.read();
        let sphere = guard
            .spheres
            .get(&pid)
            .ok_or_else(|| PvError::SphereNotFound(pane_id.clone()))?;
        serde_json::json!({
            "pane_id": pane_id,
            "memories_count": sphere.memories.len(),
            "buoys_count": sphere.buoys.len(),
            "inbox_count": sphere.inbox.len(),
            "total_steps": sphere.total_steps,
            "registered_at": sphere.registered_at,
        })
    };

    Ok(Json(manifest))
}

// ──────────────────────────────────────────────────────────────
// Router construction
// ──────────────────────────────────────────────────────────────

/// Build the axum router with routes across all groups.
///
/// Takes the full `AppContext` so handlers can access state, network, and bus.
pub fn build_router(ctx: AppContext) -> Router {
    let router = Router::new()
        // Core (3)
        .route("/health", get(health_handler))
        .route("/spheres", get(spheres_handler))
        .route("/ghosts", get(ghosts_handler))
        // Field (8)
        .route("/field", get(field_handler))
        .route("/field/r", get(field_r_handler))
        .route("/field/decision", get(field_decision_handler))
        .route("/field/decisions", get(field_decisions_handler))
        .route("/field/chimera", get(field_chimera_handler))
        .route("/field/tunnels", get(field_tunnels_handler))
        .route("/field/k", get(field_k_handler))
        .route("/field/spectrum", get(field_spectrum_handler))
        // Sphere CRUD (6)
        .route("/sphere/{pane_id}", get(sphere_detail_handler))
        .route("/sphere/{pane_id}/register", post(register_handler))
        .route("/sphere/{pane_id}/deregister", post(deregister_handler))
        .route(
            "/sphere/{pane_id}/accept-ghost",
            post(accept_ghost_handler),
        )
        .route("/sphere/{pane_id}/memory", post(memory_handler))
        .route("/sphere/{pane_id}/status", post(status_handler))
        .route("/sphere/{pane_id}/heartbeat", post(heartbeat_handler))
        // Gap fix routes (3)
        .route("/sphere/{pane_id}/phase", post(phase_handler))
        .route("/sphere/{pane_id}/steer", post(steer_handler))
        .route("/bus/suggestions", get(bus_suggestions_handler))
        // Sphere advanced (4)
        .route("/sphere/{pane_id}/neighbors", get(neighbors_handler))
        .route("/sphere/{pane_id}/inbox", get(inbox_handler))
        .route("/sphere/{pane_id}/inbox/send", post(inbox_send_handler))
        .route("/sphere/{pane_id}/inbox/ack", post(inbox_ack_handler))
        // Coupling (2)
        .route("/coupling/matrix", get(coupling_matrix_handler))
        .route("/coupling/weight", post(coupling_weight_handler))
        // Bus (9)
        .route("/bus/info", get(bus_info_handler))
        .route("/bus/tasks", get(bus_tasks_handler))
        .route("/bus/events", get(bus_events_handler))
        .route("/bus/submit", post(bus_submit_handler))
        .route("/bus/claim/{task_id}", post(bus_task_claim_handler))
        .route("/bus/complete/{task_id}", post(bus_task_complete_handler))
        .route("/bus/fail/{task_id}", post(bus_task_fail_handler))
        .route("/bus/cascade", post(bus_cascade_handler))
        .route("/bus/cascades", get(bus_cascades_handler))
        // Bridges (1)
        .route("/bridges/health", get(bridges_health_handler));

    // Governance routes (feature-gated)
    #[cfg(feature = "governance")]
    let router = router
        .route("/field/propose", post(propose_handler))
        .route("/field/proposals", get(proposals_handler))
        .route(
            "/sphere/{pane_id}/vote/{proposal_id}",
            post(vote_handler),
        )
        .route("/sphere/{pane_id}/consent", get(sphere_consent_handler))
        .route(
            "/sphere/{pane_id}/data-manifest",
            get(data_manifest_handler),
        );

    router.layer(CorsLayer::permissive()).with_state(ctx)
}

/// Build the socket address from config.
///
/// # Errors
/// Returns `PvError::ConfigValidation` if the address cannot be parsed.
pub fn build_addr(config: &PvConfig) -> Result<SocketAddr, PvError> {
    let addr_str = format!("{}:{}", config.server.bind_addr, config.server.port);
    addr_str
        .parse()
        .map_err(|e| PvError::ConfigValidation(format!("invalid bind address: {e}")))
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m3_field::m15_app_state::new_shared_state;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    fn test_ctx() -> AppContext {
        AppContext {
            state: new_shared_state(),
            network: Arc::new(RwLock::new(CouplingNetwork::new())),
            bus: Arc::new(RwLock::new(BusState::new())),
            cascade: Arc::new(RwLock::new(CascadeTracker::new())),
        }
    }

    async fn get_json(app: Router, uri: &str) -> (StatusCode, serde_json::Value) {
        let req = Request::builder()
            .uri(uri)
            .body(Body::empty())
            .expect("test request build failed");
        let resp = app.oneshot(req).await.expect("oneshot failed");
        let status = resp.status();
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 64)
            .await
            .expect("body read failed");
        let json: serde_json::Value =
            serde_json::from_slice(&body).unwrap_or(serde_json::json!(null));
        (status, json)
    }

    async fn post_json(
        app: Router,
        uri: &str,
        body: serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).expect("serialize body")))
            .expect("test request build failed");
        let resp = app.oneshot(req).await.expect("oneshot failed");
        let status = resp.status();
        let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 64)
            .await
            .expect("body read failed");
        let json: serde_json::Value =
            serde_json::from_slice(&body_bytes).unwrap_or(serde_json::json!(null));
        (status, json)
    }

    // ── build_router ──

    #[test]
    fn build_router_creates_router() {
        let ctx = test_ctx();
        let _router = build_router(ctx);
    }

    // ── build_addr ──

    #[test]
    fn build_addr_default_config() {
        let config = PvConfig::default();
        let addr = build_addr(&config);
        assert!(addr.is_ok());
        assert_eq!(addr.expect("test addr failed").port(), 8132);
    }

    #[test]
    fn build_addr_invalid() {
        let mut config = PvConfig::default();
        config.server.bind_addr = "not-an-ip".into();
        let addr = build_addr(&config);
        assert!(addr.is_err());
    }

    // ── ApiError ──

    #[test]
    fn api_error_not_found() {
        let err = ApiError(PvError::SphereNotFound("test".into()));
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn api_error_conflict() {
        let err = ApiError(PvError::SphereAlreadyRegistered("test".into()));
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn api_error_bad_request() {
        let err = ApiError(PvError::EmptyString { field: "test" });
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn api_error_too_many_requests() {
        let err = ApiError(PvError::SphereCapReached(200));
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn api_error_forbidden() {
        let err = ApiError(PvError::BridgeConsentDenied {
            service: "test".into(),
            sphere: "test".into(),
        });
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn api_error_internal() {
        let err = ApiError(PvError::Internal("test".into()));
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn api_error_from_pv_error() {
        let pv_err = PvError::SphereNotFound("test".into());
        let _api_err: ApiError = pv_err.into();
    }

    // ── Core route integration tests (async) ──

    #[tokio::test]
    async fn health_endpoint_returns_200() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, _) = get_json(app, "/health").await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn spheres_endpoint_returns_200() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, _) = get_json(app, "/spheres").await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn ghosts_endpoint_returns_200() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, _) = get_json(app, "/ghosts").await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn unknown_route_returns_404() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let req = Request::builder()
            .uri("/nonexistent")
            .body(Body::empty())
            .expect("test request");
        let resp = app.oneshot(req).await.expect("oneshot");
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn health_returns_json() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (_, json) = get_json(app, "/health").await;
        assert_eq!(json["status"], "healthy");
    }

    #[tokio::test]
    async fn health_shows_tick() {
        let ctx = test_ctx();
        {
            let mut guard = ctx.state.write();
            guard.tick = 42;
        }
        let app = build_router(ctx);
        let (_, json) = get_json(app, "/health").await;
        assert_eq!(json["tick"], 42);
    }

    #[tokio::test]
    async fn spheres_empty_returns_empty_array() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (_, json) = get_json(app, "/spheres").await;
        assert!(json["spheres"].as_array().expect("array").is_empty());
    }

    // ── Field route tests ──

    #[tokio::test]
    async fn field_endpoint_returns_200() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, _) = get_json(app, "/field").await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn field_r_returns_r_value() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, json) = get_json(app, "/field/r").await;
        assert_eq!(status, StatusCode::OK);
        assert!(json.get("r").is_some());
    }

    #[tokio::test]
    async fn field_decision_returns_200() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, _) = get_json(app, "/field/decision").await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn field_decisions_returns_history() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, json) = get_json(app, "/field/decisions").await;
        assert_eq!(status, StatusCode::OK);
        assert!(json["decisions"].is_array());
    }

    #[tokio::test]
    async fn field_chimera_returns_200() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, _) = get_json(app, "/field/chimera").await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn field_tunnels_returns_200() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, json) = get_json(app, "/field/tunnels").await;
        assert_eq!(status, StatusCode::OK);
        assert!(json["tunnels"].is_array());
    }

    #[tokio::test]
    async fn field_k_returns_coupling_info() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, json) = get_json(app, "/field/k").await;
        assert_eq!(status, StatusCode::OK);
        assert!(json.get("k").is_some());
        assert!(json.get("k_modulation").is_some());
    }

    #[tokio::test]
    async fn field_spectrum_returns_200() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, _) = get_json(app, "/field/spectrum").await;
        assert_eq!(status, StatusCode::OK);
    }

    // ── Sphere CRUD tests ──

    #[tokio::test]
    async fn register_and_get_sphere() {
        let ctx = test_ctx();
        let app = build_router(ctx.clone());
        let (status, _) = post_json(
            app,
            "/sphere/test-alpha/register",
            serde_json::json!({ "persona": "explorer", "frequency": 0.1 }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);

        // Now GET the sphere
        let app2 = build_router(ctx);
        let (status2, json) = get_json(app2, "/sphere/test-alpha").await;
        assert_eq!(status2, StatusCode::OK);
        assert_eq!(json["persona"], "explorer");
    }

    #[tokio::test]
    async fn register_duplicate_returns_conflict() {
        let ctx = test_ctx();
        // Register first time
        let app = build_router(ctx.clone());
        let (status, _) = post_json(
            app,
            "/sphere/dupe-1/register",
            serde_json::json!({ "persona": "a", "frequency": 0.1 }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);

        // Register again
        let app2 = build_router(ctx);
        let (status2, _) = post_json(
            app2,
            "/sphere/dupe-1/register",
            serde_json::json!({ "persona": "b", "frequency": 0.2 }),
        )
        .await;
        assert_eq!(status2, StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn sphere_not_found_returns_404() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, _) = get_json(app, "/sphere/nonexistent").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn deregister_creates_ghost() {
        let ctx = test_ctx();
        // Register
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/ghost-test/register",
            serde_json::json!({ "persona": "departing", "frequency": 0.1 }),
        )
        .await;

        // Deregister
        let app2 = build_router(ctx.clone());
        let (status, json) = post_json(app2, "/sphere/ghost-test/deregister", serde_json::json!({}))
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["deregistered"], "ghost-test");

        // Check ghost exists
        let app3 = build_router(ctx);
        let (_, ghosts) = get_json(app3, "/ghosts").await;
        let ghosts_arr = ghosts["ghosts"].as_array().expect("ghosts array");
        assert!(!ghosts_arr.is_empty());
    }

    #[tokio::test]
    async fn record_memory() {
        let ctx = test_ctx();
        // Register first
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/mem-test/register",
            serde_json::json!({ "persona": "coder", "frequency": 0.1 }),
        )
        .await;

        // Record memory
        let app2 = build_router(ctx);
        let (status, json) = post_json(
            app2,
            "/sphere/mem-test/memory",
            serde_json::json!({ "tool_name": "Read", "summary": "read a file" }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert!(json.get("memory_id").is_some());
    }

    #[tokio::test]
    async fn update_status() {
        let ctx = test_ctx();
        // Register
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/status-test/register",
            serde_json::json!({ "persona": "worker", "frequency": 0.1 }),
        )
        .await;

        // Update status
        let app2 = build_router(ctx);
        let (status, json) = post_json(
            app2,
            "/sphere/status-test/status",
            serde_json::json!({ "status": "Working", "last_tool": "Bash" }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["status"], "Working");
    }

    #[tokio::test]
    async fn heartbeat() {
        let ctx = test_ctx();
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/hb-test/register",
            serde_json::json!({ "persona": "alive", "frequency": 0.1 }),
        )
        .await;

        let app2 = build_router(ctx);
        let (status, json) = post_json(app2, "/sphere/hb-test/heartbeat", serde_json::json!({}))
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["heartbeat"], "ok");
    }

    // ── Sphere advanced tests ──

    #[tokio::test]
    async fn neighbors_empty_for_single_sphere() {
        let ctx = test_ctx();
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/alone/register",
            serde_json::json!({ "persona": "solo", "frequency": 0.1 }),
        )
        .await;

        let app2 = build_router(ctx);
        let (status, json) = get_json(app2, "/sphere/alone/neighbors").await;
        assert_eq!(status, StatusCode::OK);
        assert!(json["neighbors"].is_array());
    }

    #[tokio::test]
    async fn inbox_empty_initially() {
        let ctx = test_ctx();
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/inbox-test/register",
            serde_json::json!({ "persona": "listener", "frequency": 0.1 }),
        )
        .await;

        let app2 = build_router(ctx);
        let (status, json) = get_json(app2, "/sphere/inbox-test/inbox").await;
        assert_eq!(status, StatusCode::OK);
        let messages = json["messages"].as_array().expect("messages array");
        assert!(messages.is_empty());
    }

    #[tokio::test]
    async fn send_and_ack_message() {
        let ctx = test_ctx();
        // Register target sphere
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/msg-target/register",
            serde_json::json!({ "persona": "target", "frequency": 0.1 }),
        )
        .await;

        // Send message
        let app2 = build_router(ctx.clone());
        let (status, json) = post_json(
            app2,
            "/sphere/msg-target/inbox/send",
            serde_json::json!({ "from": "msg-sender", "content": "hello there" }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        let msg_id = json["message_id"].as_u64().expect("message_id");

        // Acknowledge
        let app3 = build_router(ctx);
        let (status3, json3) = post_json(
            app3,
            "/sphere/msg-target/inbox/ack",
            serde_json::json!({ "message_id": msg_id }),
        )
        .await;
        assert_eq!(status3, StatusCode::OK);
        assert_eq!(json3["acknowledged"], true);
    }

    // ── Coupling tests ──

    #[tokio::test]
    async fn coupling_matrix_empty() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, json) = get_json(app, "/coupling/matrix").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["count"], 0);
    }

    #[tokio::test]
    async fn coupling_weight_set() {
        let ctx = test_ctx();
        // Register two spheres first
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/cw-a/register",
            serde_json::json!({ "persona": "a", "frequency": 0.1 }),
        )
        .await;
        let app2 = build_router(ctx.clone());
        post_json(
            app2,
            "/sphere/cw-b/register",
            serde_json::json!({ "persona": "b", "frequency": 0.2 }),
        )
        .await;

        let app3 = build_router(ctx);
        let (status, json) = post_json(
            app3,
            "/coupling/weight",
            serde_json::json!({ "from": "cw-a", "to": "cw-b", "weight": 0.8 }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["from"], "cw-a");
    }

    // ── Bus tests ──

    #[tokio::test]
    async fn bus_info_returns_200() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, json) = get_json(app, "/bus/info").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["tasks"], 0);
    }

    #[tokio::test]
    async fn bus_tasks_returns_empty() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, json) = get_json(app, "/bus/tasks").await;
        assert_eq!(status, StatusCode::OK);
        assert!(json["tasks"].is_array());
    }

    #[tokio::test]
    async fn bus_events_returns_empty() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, json) = get_json(app, "/bus/events").await;
        assert_eq!(status, StatusCode::OK);
        assert!(json["events"].is_array());
    }

    // ── Bridge tests ──

    #[tokio::test]
    async fn bridges_health_returns_200() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, json) = get_json(app, "/bridges/health").await;
        assert_eq!(status, StatusCode::OK);
        // All bridges should be stale initially (no smoke test in unit tests)
        assert!(json.get("synthex_stale").is_some());
    }

    // ── Validation tests ──

    #[tokio::test]
    async fn register_invalid_pane_id_returns_400() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, _) = post_json(
            app,
            "/sphere/bad%20id/register",
            serde_json::json!({ "persona": "test", "frequency": 0.1 }),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn register_empty_persona_returns_400() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, _) = post_json(
            app,
            "/sphere/good-id/register",
            serde_json::json!({ "persona": "", "frequency": 0.1 }),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn invalid_status_returns_400() {
        let ctx = test_ctx();
        // Register
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/val-test/register",
            serde_json::json!({ "persona": "tester", "frequency": 0.1 }),
        )
        .await;
        // Bad status
        let app2 = build_router(ctx);
        let (status, _) = post_json(
            app2,
            "/sphere/val-test/status",
            serde_json::json!({ "status": "Dancing" }),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    // ── parse_status ──

    #[test]
    fn parse_status_valid() {
        assert_eq!(parse_status("Idle").expect("idle"), PaneStatus::Idle);
        assert_eq!(parse_status("Working").expect("working"), PaneStatus::Working);
        assert_eq!(parse_status("Blocked").expect("blocked"), PaneStatus::Blocked);
        assert_eq!(parse_status("Complete").expect("complete"), PaneStatus::Complete);
        assert_eq!(parse_status("idle").expect("lower"), PaneStatus::Idle);
    }

    #[test]
    fn parse_status_invalid() {
        assert!(parse_status("Unknown").is_err());
    }

    // ── Bus submit tests ──

    #[tokio::test]
    async fn bus_submit_creates_task() {
        let ctx = test_ctx();
        let app = build_router(ctx.clone());
        let (status, json) = post_json(
            app,
            "/bus/submit",
            serde_json::json!({
                "description": "test task from HTTP",
                "target": "any_idle",
                "submitter": "http-test"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert!(json.get("task_id").is_some());
        assert_eq!(json["status"], "Pending");

        // Verify task appears in bus
        let app2 = build_router(ctx);
        let (_, tasks_json) = get_json(app2, "/bus/tasks").await;
        assert_eq!(tasks_json["tasks"].as_array().map_or(0, Vec::len), 1);
    }

    #[tokio::test]
    async fn bus_submit_field_driven() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, json) = post_json(
            app,
            "/bus/submit",
            serde_json::json!({
                "description": "field-driven task",
                "target": "field_driven",
                "submitter": "test-sphere"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert!(json.get("task_id").is_some());
    }

    #[tokio::test]
    async fn bus_submit_specific_target() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, json) = post_json(
            app,
            "/bus/submit",
            serde_json::json!({
                "description": "specific task",
                "target": "specific",
                "submitter": "test",
                "target_pane_id": "target-sphere"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert!(json.get("task_id").is_some());
    }

    #[tokio::test]
    async fn bus_submit_specific_missing_pane_id() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, _) = post_json(
            app,
            "/bus/submit",
            serde_json::json!({
                "description": "bad task",
                "target": "specific",
                "submitter": "test"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn bus_submit_invalid_target() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, _) = post_json(
            app,
            "/bus/submit",
            serde_json::json!({
                "description": "bad target",
                "target": "nonexistent",
                "submitter": "test"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    // ── Cascade tests ──

    #[tokio::test]
    async fn bus_cascade_creates_handoff() {
        let ctx = test_ctx();
        let app = build_router(ctx.clone());
        let (status, json) = post_json(
            app,
            "/bus/cascade",
            serde_json::json!({
                "source": "tab4-left",
                "target": "tab5-left",
                "brief": "Continue V3.2 inhabitation work"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert!(json.get("cascade_index").is_some());
        assert_eq!(json["status"], "dispatched");

        // Verify cascade appears in list
        let app2 = build_router(ctx.clone());
        let (_, cascades_json) = get_json(app2, "/bus/cascades").await;
        assert_eq!(cascades_json["cascades"].as_array().map_or(0, Vec::len), 1);

        // Verify event was published
        let app3 = build_router(ctx);
        let (_, events_json) = get_json(app3, "/bus/events").await;
        let events = events_json["events"].as_array().expect("events array");
        assert!(!events.is_empty());
        assert_eq!(events[0]["event_type"], "cascade.initiated");
    }

    #[tokio::test]
    async fn bus_cascades_empty() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, json) = get_json(app, "/bus/cascades").await;
        assert_eq!(status, StatusCode::OK);
        assert!(json["cascades"].as_array().map_or(false, Vec::is_empty));
    }

    // ── Ghost reincarnation tests ──

    #[tokio::test]
    async fn register_restores_ghost_phase() {
        let ctx = test_ctx();
        // Register and deregister to create a ghost
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/reborn-test/register",
            serde_json::json!({ "persona": "mortal", "frequency": 0.1 }),
        )
        .await;

        // Set a known phase before departure
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/reborn-test/phase",
            serde_json::json!({ "phase": 2.5 }),
        )
        .await;

        // Deregister (creates ghost with phase_at_departure ≈ 2.5)
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/reborn-test/deregister",
            serde_json::json!({}),
        )
        .await;

        // Re-register with same ID — should restore ghost phase
        let app = build_router(ctx.clone());
        let (status, json) = post_json(
            app,
            "/sphere/reborn-test/register",
            serde_json::json!({ "persona": "immortal", "frequency": 0.1 }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(json["ghost_restored"], true);

        // Ghost should be consumed (not in ghosts list)
        let app = build_router(ctx);
        let (_, ghosts_json) = get_json(app, "/ghosts").await;
        let ghosts = ghosts_json["ghosts"].as_array().expect("ghosts");
        let found = ghosts.iter().any(|g| g["id"] == "reborn-test");
        assert!(!found, "ghost should be consumed after reincarnation");
    }

    #[tokio::test]
    async fn register_no_ghost_returns_false() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, json) = post_json(
            app,
            "/sphere/fresh-test/register",
            serde_json::json!({ "persona": "new-soul", "frequency": 0.1 }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(json["ghost_restored"], false);
    }

    // ── Ghost enrichment tests ──

    #[tokio::test]
    async fn deregister_enriches_ghost_top_tools() {
        let ctx = test_ctx();
        // Register
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/enrich-test/register",
            serde_json::json!({ "persona": "enricher", "frequency": 0.1 }),
        )
        .await;

        // Record some memories
        for tool in &["Read", "Edit", "Read", "Bash", "Read"] {
            let app = build_router(ctx.clone());
            post_json(
                app,
                "/sphere/enrich-test/memory",
                serde_json::json!({ "tool_name": tool, "summary": "test" }),
            )
            .await;
        }

        // Deregister
        let app = build_router(ctx.clone());
        post_json(app, "/sphere/enrich-test/deregister", serde_json::json!({})).await;

        // Check ghost has top tools
        let app = build_router(ctx);
        let (_, json) = get_json(app, "/ghosts").await;
        let ghosts = json["ghosts"].as_array().expect("ghosts");
        assert!(!ghosts.is_empty());
        let ghost = &ghosts[0];
        let top_tools = ghost["top_tools"].as_array().expect("top_tools");
        assert!(!top_tools.is_empty(), "ghost should have top_tools");
        assert_eq!(top_tools[0], "Read", "most-used tool should be first");
    }

    #[tokio::test]
    async fn deregister_enriches_ghost_neighbors() {
        let ctx = test_ctx();
        // Register two spheres so there are coupling connections
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/nb-a/register",
            serde_json::json!({ "persona": "alpha", "frequency": 0.1 }),
        )
        .await;
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/nb-b/register",
            serde_json::json!({ "persona": "beta", "frequency": 0.2 }),
        )
        .await;

        // Deregister nb-a — should capture nb-b as neighbor
        let app = build_router(ctx.clone());
        post_json(app, "/sphere/nb-a/deregister", serde_json::json!({})).await;

        // Check ghost has strongest_neighbors from ghosts endpoint
        let guard = ctx.state.read();
        let ghost = guard.ghosts.iter().find(|g| g.id.as_str() == "nb-a");
        assert!(ghost.is_some(), "ghost should exist");
        let ghost = ghost.expect("ghost");
        assert!(
            !ghost.strongest_neighbors.is_empty(),
            "ghost should have neighbors"
        );
    }

    // ── Phase handler tests ──

    #[tokio::test]
    async fn phase_handler_updates_phase() {
        let ctx = test_ctx();
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/ph-test/register",
            serde_json::json!({ "persona": "phase-tester", "frequency": 0.1 }),
        )
        .await;

        let app2 = build_router(ctx.clone());
        let (status, json) = post_json(
            app2,
            "/sphere/ph-test/phase",
            serde_json::json!({ "phase": 1.57 }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["pane_id"], "ph-test");
        let returned_phase = json["phase"].as_f64().expect("phase");
        assert!((returned_phase - 1.57).abs() < 0.01);
    }

    #[tokio::test]
    async fn phase_handler_updates_frequency() {
        let ctx = test_ctx();
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/pf-test/register",
            serde_json::json!({ "persona": "freq-tester", "frequency": 0.1 }),
        )
        .await;

        let app2 = build_router(ctx.clone());
        let (status, json) = post_json(
            app2,
            "/sphere/pf-test/phase",
            serde_json::json!({ "frequency": 0.5 }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let returned_freq = json["frequency"].as_f64().expect("frequency");
        assert!((returned_freq - 0.5).abs() < 0.01);
    }

    #[tokio::test]
    async fn phase_handler_updates_both() {
        let ctx = test_ctx();
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/pb-test/register",
            serde_json::json!({ "persona": "both-tester", "frequency": 0.1 }),
        )
        .await;

        let app2 = build_router(ctx.clone());
        let (status, json) = post_json(
            app2,
            "/sphere/pb-test/phase",
            serde_json::json!({ "phase": 3.14, "frequency": 0.3 }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(json["phase"].as_f64().is_some());
        assert!(json["frequency"].as_f64().is_some());
    }

    #[tokio::test]
    async fn phase_handler_wraps_phase() {
        let ctx = test_ctx();
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/pw-test/register",
            serde_json::json!({ "persona": "wrap-tester", "frequency": 0.1 }),
        )
        .await;

        let app2 = build_router(ctx.clone());
        let (status, json) = post_json(
            app2,
            "/sphere/pw-test/phase",
            serde_json::json!({ "phase": 7.0 }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let phase = json["phase"].as_f64().expect("phase");
        assert!(phase >= 0.0 && phase < std::f64::consts::TAU);
    }

    #[tokio::test]
    async fn phase_handler_negative_phase_wraps() {
        let ctx = test_ctx();
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/pn-test/register",
            serde_json::json!({ "persona": "neg-tester", "frequency": 0.1 }),
        )
        .await;

        let app2 = build_router(ctx.clone());
        let (status, json) = post_json(
            app2,
            "/sphere/pn-test/phase",
            serde_json::json!({ "phase": -1.0 }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let phase = json["phase"].as_f64().expect("phase");
        assert!(phase >= 0.0 && phase < std::f64::consts::TAU);
    }

    #[tokio::test]
    async fn phase_handler_null_phase_is_noop() {
        let ctx = test_ctx();
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/null-test/register",
            serde_json::json!({ "persona": "null-tester", "frequency": 0.1 }),
        )
        .await;

        // JSON null → Option<f64>::None → no-op
        let app2 = build_router(ctx.clone());
        let (status, _) = post_json(
            app2,
            "/sphere/null-test/phase",
            serde_json::json!({ "phase": null }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn phase_handler_syncs_to_coupling_network() {
        let ctx = test_ctx();
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/sync-test/register",
            serde_json::json!({ "persona": "sync-tester", "frequency": 0.1 }),
        )
        .await;

        // Change frequency and verify coupling network picks it up
        let app2 = build_router(ctx.clone());
        let (status, _) = post_json(
            app2,
            "/sphere/sync-test/phase",
            serde_json::json!({ "frequency": 0.5 }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let net = ctx.network.read();
        let pid = PaneId::new("sync-test");
        if let Some(&freq) = net.frequencies.get(&pid) {
            assert!((freq - 0.5).abs() < 0.01);
        }
    }

    #[tokio::test]
    async fn phase_handler_sphere_not_found() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, _) = post_json(
            app,
            "/sphere/nonexistent/phase",
            serde_json::json!({ "phase": 1.0 }),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn phase_handler_empty_body_ok() {
        let ctx = test_ctx();
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/pe-test/register",
            serde_json::json!({ "persona": "empty-tester", "frequency": 0.1 }),
        )
        .await;

        // No phase or frequency — should still succeed (no-op)
        let app2 = build_router(ctx.clone());
        let (status, _) = post_json(
            app2,
            "/sphere/pe-test/phase",
            serde_json::json!({}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    // ── Steer handler tests ──

    #[tokio::test]
    async fn steer_handler_steers_phase() {
        let ctx = test_ctx();
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/st-test/register",
            serde_json::json!({ "persona": "steer-tester", "frequency": 0.1 }),
        )
        .await;

        let app2 = build_router(ctx.clone());
        let (status, json) = post_json(
            app2,
            "/sphere/st-test/steer",
            serde_json::json!({ "target_phase": 3.14, "strength": 0.5 }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["pane_id"], "st-test");
        let target = json["target_phase"].as_f64().expect("target_phase");
        assert!(target >= 0.0 && target < std::f64::consts::TAU);
    }

    #[tokio::test]
    async fn steer_handler_clamps_strength() {
        let ctx = test_ctx();
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/sc-test/register",
            serde_json::json!({ "persona": "clamp-tester", "frequency": 0.1 }),
        )
        .await;

        let app2 = build_router(ctx.clone());
        let (status, json) = post_json(
            app2,
            "/sphere/sc-test/steer",
            serde_json::json!({ "target_phase": 1.0, "strength": 5.0 }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let strength = json["strength"].as_f64().expect("strength");
        assert!((strength - 2.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn steer_handler_negative_strength_clamps() {
        let ctx = test_ctx();
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/sn-test/register",
            serde_json::json!({ "persona": "neg-str-tester", "frequency": 0.1 }),
        )
        .await;

        let app2 = build_router(ctx.clone());
        let (status, json) = post_json(
            app2,
            "/sphere/sn-test/steer",
            serde_json::json!({ "target_phase": 1.0, "strength": -1.0 }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let strength = json["strength"].as_f64().expect("strength");
        assert!(strength >= 0.0);
    }

    #[tokio::test]
    async fn steer_handler_null_target_rejected() {
        let ctx = test_ctx();
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/snan-test/register",
            serde_json::json!({ "persona": "nan-steer", "frequency": 0.1 }),
        )
        .await;

        // target_phase is required (f64, not Option<f64>), null → 422
        let app2 = build_router(ctx.clone());
        let (status, _) = post_json(
            app2,
            "/sphere/snan-test/steer",
            serde_json::json!({ "target_phase": null, "strength": 0.5 }),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn steer_handler_sphere_not_found() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, _) = post_json(
            app,
            "/sphere/nonexistent/steer",
            serde_json::json!({ "target_phase": 1.0, "strength": 0.5 }),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn steer_handler_wraps_target_phase() {
        let ctx = test_ctx();
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/sw-test/register",
            serde_json::json!({ "persona": "wrap-steer", "frequency": 0.1 }),
        )
        .await;

        let app2 = build_router(ctx.clone());
        let (status, json) = post_json(
            app2,
            "/sphere/sw-test/steer",
            serde_json::json!({ "target_phase": 10.0, "strength": 0.5 }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let target = json["target_phase"].as_f64().expect("target_phase");
        assert!(target >= 0.0 && target < std::f64::consts::TAU);
    }

    // ── Bus suggestions handler tests ──

    #[tokio::test]
    async fn bus_suggestions_returns_200() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, json) = get_json(app, "/bus/suggestions").await;
        assert_eq!(status, StatusCode::OK);
        assert!(json["suggestions"].is_array());
        assert_eq!(json["total_generated"], 0);
    }

    #[tokio::test]
    async fn bus_suggestions_empty_array() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (_, json) = get_json(app, "/bus/suggestions").await;
        let suggestions = json["suggestions"].as_array().expect("suggestions array");
        assert!(suggestions.is_empty());
    }

    // ── Governance API tests ──

    #[cfg(feature = "governance")]
    #[tokio::test]
    async fn propose_and_list() {
        let ctx = test_ctx();
        let app = build_router(ctx.clone());
        let (status, json) = post_json(
            app,
            "/field/propose",
            serde_json::json!({
                "proposer": "gov-test-1",
                "parameter": "r_target",
                "value": 0.85,
                "reason": "testing governance"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert!(json.get("proposal_id").is_some());

        // List proposals
        let app2 = build_router(ctx);
        let (status2, json2) = get_json(app2, "/field/proposals").await;
        assert_eq!(status2, StatusCode::OK);
        let proposals = json2["proposals"].as_array().expect("proposals");
        assert_eq!(proposals.len(), 1);
    }

    #[cfg(feature = "governance")]
    #[tokio::test]
    async fn propose_invalid_parameter_rejected() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, _) = post_json(
            app,
            "/field/propose",
            serde_json::json!({
                "proposer": "gov-test",
                "parameter": "invalid_param",
                "value": 1.0,
                "reason": "bad param"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[cfg(feature = "governance")]
    #[tokio::test]
    async fn vote_on_proposal() {
        let ctx = test_ctx();
        // Submit proposal
        let app = build_router(ctx.clone());
        let (_, json) = post_json(
            app,
            "/field/propose",
            serde_json::json!({
                "proposer": "vote-test-1",
                "parameter": "r_target",
                "value": 0.85,
                "reason": "vote test"
            }),
        )
        .await;
        let proposal_id = json["proposal_id"].as_str().expect("proposal_id").to_owned();

        // Vote
        let app2 = build_router(ctx);
        let (status, json2) = post_json(
            app2,
            &format!("/sphere/vote-test-2/vote/{proposal_id}"),
            serde_json::json!({ "choice": "approve" }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json2["choice"], "approve");
    }

    #[cfg(feature = "governance")]
    #[tokio::test]
    async fn sphere_consent_handler_returns_opt_outs() {
        let ctx = test_ctx();
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/consent-test/register",
            serde_json::json!({ "persona": "consent-tester", "frequency": 0.1 }),
        )
        .await;

        let app2 = build_router(ctx);
        let (status, json) = get_json(app2, "/sphere/consent-test/consent").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["opt_out_hebbian"], false);
        assert_eq!(json["opt_out_external_modulation"], false);
    }

    #[cfg(feature = "governance")]
    #[tokio::test]
    async fn data_manifest_handler_returns_counts() {
        let ctx = test_ctx();
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/manifest-test/register",
            serde_json::json!({ "persona": "manifest-tester", "frequency": 0.1 }),
        )
        .await;

        let app2 = build_router(ctx);
        let (status, json) = get_json(app2, "/sphere/manifest-test/data-manifest").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["memories_count"], 0);
        assert_eq!(json["buoys_count"], 3); // 3 buoys per sphere at creation
    }

    #[cfg(feature = "governance")]
    #[tokio::test]
    async fn proposals_empty_initially() {
        let ctx = test_ctx();
        let app = build_router(ctx);
        let (status, json) = get_json(app, "/field/proposals").await;
        assert_eq!(status, StatusCode::OK);
        let proposals = json["proposals"].as_array().expect("proposals");
        assert!(proposals.is_empty());
    }

    // ── E2E Integration tests ──

    #[cfg(feature = "governance")]
    #[tokio::test]
    async fn e2e_governance_proposal_lifecycle() {
        let ctx = test_ctx();

        // Register 3 spheres
        for name in &["gov-a", "gov-b", "gov-c"] {
            let app = build_router(ctx.clone());
            post_json(
                app,
                &format!("/sphere/{name}/register"),
                serde_json::json!({ "persona": "governor", "frequency": 0.1 }),
            )
            .await;
        }

        // Submit proposal
        let app = build_router(ctx.clone());
        let (status, json) = post_json(
            app,
            "/field/propose",
            serde_json::json!({
                "proposer": "gov-a",
                "parameter": "r_target",
                "value": 0.80,
                "reason": "E2E governance test"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        let proposal_id = json["proposal_id"].as_str().expect("id").to_owned();

        // Two spheres vote approve
        for voter in &["gov-b", "gov-c"] {
            let app = build_router(ctx.clone());
            let (status, _) = post_json(
                app,
                &format!("/sphere/{voter}/vote/{proposal_id}"),
                serde_json::json!({ "choice": "approve" }),
            )
            .await;
            assert_eq!(status, StatusCode::OK);
        }

        // Verify proposal has votes
        let app = build_router(ctx.clone());
        let (_, json) = get_json(app, "/field/proposals").await;
        let proposals = json["proposals"].as_array().expect("proposals");
        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0]["votes"], 2);
    }

    #[tokio::test]
    async fn e2e_ghost_reincarnation_full_cycle() {
        let ctx = test_ctx();

        // 1. Register sphere
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/e2e-ghost/register",
            serde_json::json!({ "persona": "ephemeral", "frequency": 0.1 }),
        )
        .await;

        // 2. Set phase and record memories
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/e2e-ghost/phase",
            serde_json::json!({ "phase": 2.0 }),
        )
        .await;

        for _ in 0..3 {
            let app = build_router(ctx.clone());
            post_json(
                app,
                "/sphere/e2e-ghost/memory",
                serde_json::json!({ "tool_name": "Read", "summary": "file" }),
            )
            .await;
        }

        // 3. Deregister — creates enriched ghost
        let app = build_router(ctx.clone());
        let (status, _) = post_json(
            app,
            "/sphere/e2e-ghost/deregister",
            serde_json::json!({}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        // 4. Verify ghost exists with enriched data
        let app = build_router(ctx.clone());
        let (_, json) = get_json(app, "/ghosts").await;
        let ghosts = json["ghosts"].as_array().expect("ghosts");
        let ghost = ghosts.iter().find(|g| g["id"] == "e2e-ghost");
        assert!(ghost.is_some(), "ghost should exist");
        let ghost = ghost.expect("ghost");
        assert_eq!(ghost["memory_count"], 3);
        let top_tools = ghost["top_tools"].as_array().expect("top_tools");
        assert!(!top_tools.is_empty());

        // 5. Re-register — ghost phase restored
        let app = build_router(ctx.clone());
        let (status, json) = post_json(
            app,
            "/sphere/e2e-ghost/register",
            serde_json::json!({ "persona": "reborn", "frequency": 0.1 }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(json["ghost_restored"], true);

        // 6. Verify ghost consumed
        let app = build_router(ctx.clone());
        let (_, json) = get_json(app, "/ghosts").await;
        let ghosts = json["ghosts"].as_array().expect("ghosts");
        assert!(
            !ghosts.iter().any(|g| g["id"] == "e2e-ghost"),
            "ghost should be consumed"
        );

        // 7. Verify sphere exists
        let app = build_router(ctx);
        let (status, _) = get_json(app, "/sphere/e2e-ghost").await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn e2e_consent_gate_integration() {
        let ctx = test_ctx();

        // Register sphere
        let app = build_router(ctx.clone());
        post_json(
            app,
            "/sphere/consent-e2e/register",
            serde_json::json!({ "persona": "consenter", "frequency": 0.1 }),
        )
        .await;

        // Check consent posture via consent endpoint
        #[cfg(feature = "governance")]
        {
            let app = build_router(ctx.clone());
            let (status, json) = get_json(app, "/sphere/consent-e2e/consent").await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(json["opt_out_hebbian"], false);
            assert_eq!(json["opt_out_external_modulation"], false);
        }

        // Check data manifest
        #[cfg(feature = "governance")]
        {
            let app = build_router(ctx.clone());
            let (status, json) = get_json(app, "/sphere/consent-e2e/data-manifest").await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(json["memories_count"], 0);
        }

        // Steer and verify sphere still responds
        let app = build_router(ctx.clone());
        let (status, _) = post_json(
            app,
            "/sphere/consent-e2e/steer",
            serde_json::json!({ "target_phase": 1.0, "strength": 0.5 }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }
}
