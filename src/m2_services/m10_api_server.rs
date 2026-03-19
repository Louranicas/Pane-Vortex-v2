//! # M10: API Server
//!
//! Axum 0.8 HTTP server with CORS, body limits, and route registration.
//! Feature-gated behind `api`.
//!
//! ## Layer: L2 (Services)
//! ## Module: M10
//! ## Dependencies: L1 (M02, M03, M06), L3 (M15 `SharedState`), L4 (M16, M18), L7 (M29, M30)
//!
//! ## Route Groups (58 total)
//!
//! | Group | Count | Description |
//! |-------|-------|-------------|
//! | Core | 3 | `/health`, `/spheres`, `/ghosts` |
//! | Field | 8 | `/field/*` — r, decision, tunnels, spectrum |
//! | Sphere CRUD | 6 | `/sphere/{pane_id}/*` — register, deregister, memory, status, heartbeat |
//! | Sphere Advanced | 4 | `/sphere/{pane_id}/neighbors`, inbox, send, ack |
//! | Coupling | 2 | `/coupling/matrix`, `/coupling/weight` |
//! | Bus | 3 | `/bus/info`, `/bus/tasks`, `/bus/events` |
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
        let r = guard.r_history.back().copied().unwrap_or(0.0);
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
        let net = ctx.network.read();
        let op = net.order_parameter();
        (op.r, op.psi)
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

    let sphere = PaneSphere::new(pid.clone(), body.persona.clone(), freq)?;
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

    let ghost = {
        let mut guard = ctx.state.write();
        let sphere = guard
            .spheres
            .remove(&pid)
            .ok_or_else(|| PvError::SphereNotFound(pane_id.clone()))?;

        let memory_count = sphere.memories.len();
        let ghost = GhostTrace {
            id: sphere.id,
            persona: sphere.persona,
            deregistered_at: guard.tick,
            total_steps_lived: sphere.total_steps,
            memory_count,
            top_tools: Vec::new(),
            phase_at_departure: sphere.phase,
            receptivity: sphere.receptivity,
            work_signature: sphere.work_signature,
            strongest_neighbors: Vec::new(),
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
async fn bus_suggestions_handler(State(_ctx): State<AppContext>) -> impl IntoResponse {
    Json(serde_json::json!({
        "suggestions": [],
        "total_generated": 0,
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
// Router construction
// ──────────────────────────────────────────────────────────────

/// Build the axum router with all 27 routes across 7 groups.
///
/// Takes the full `AppContext` so handlers can access state, network, and bus.
pub fn build_router(ctx: AppContext) -> Router {
    Router::new()
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
        // Bus (3)
        .route("/bus/info", get(bus_info_handler))
        .route("/bus/tasks", get(bus_tasks_handler))
        .route("/bus/events", get(bus_events_handler))
        // Bridges (1)
        .route("/bridges/health", get(bridges_health_handler))
        .layer(CorsLayer::permissive())
        .with_state(ctx)
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
}
