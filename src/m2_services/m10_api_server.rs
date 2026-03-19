//! # M10: API Server
//!
//! Axum 0.8 HTTP server with CORS, body limits, and route registration.
//! Feature-gated behind `api`.
//!
//! ## Layer: L2 (Services)
//! ## Module: M10
//! ## Dependencies: L1 (M02, M03), L3 (M15 `SharedState`)

use std::net::SocketAddr;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;

use crate::m1_foundation::{
    m02_error_handling::PvError,
    m03_config::PvConfig,
};
use crate::m3_field::m15_app_state::SharedState;

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
// Route handlers
// ──────────────────────────────────────────────────────────────

/// GET /health — Basic health check.
async fn health_handler(State(state): State<SharedState>) -> impl IntoResponse {
    let (r, sphere_count, tick, fleet_mode) = {
        let guard = state.read();
        let r = guard.r_history.back().copied().unwrap_or(0.0);
        (r, guard.spheres.len(), guard.tick, guard.fleet_mode())
    };

    Json(serde_json::json!({
        "status": "ok",
        "r": r,
        "spheres": sphere_count,
        "tick": tick,
        "fleet_mode": format!("{fleet_mode:?}"),
    }))
}

/// GET /spheres — List all registered spheres.
async fn spheres_handler(State(state): State<SharedState>) -> impl IntoResponse {
    let spheres: Vec<serde_json::Value> = {
        let guard = state.read();
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

/// GET /ghosts — List ghost traces.
async fn ghosts_handler(State(state): State<SharedState>) -> impl IntoResponse {
    let ghosts: Vec<serde_json::Value> = {
        let guard = state.read();
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
// Router construction
// ──────────────────────────────────────────────────────────────

/// Build the axum router with all routes.
pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/spheres", get(spheres_handler))
        .route("/ghosts", get(ghosts_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
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

    fn test_state() -> SharedState {
        new_shared_state()
    }

    // ── build_router ──

    #[test]
    fn build_router_creates_router() {
        let state = test_state();
        let _router = build_router(state);
    }

    // ── build_addr ──

    #[test]
    fn build_addr_default_config() {
        let config = PvConfig::default();
        let addr = build_addr(&config);
        assert!(addr.is_ok());
        assert_eq!(addr.unwrap().port(), 8132);
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

    // ── Route integration tests (async) ──

    #[tokio::test]
    async fn health_endpoint_returns_200() {
        let state = test_state();
        let app = build_router(state);

        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn spheres_endpoint_returns_200() {
        let state = test_state();
        let app = build_router(state);

        let req = Request::builder()
            .uri("/spheres")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn ghosts_endpoint_returns_200() {
        let state = test_state();
        let app = build_router(state);

        let req = Request::builder()
            .uri("/ghosts")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn unknown_route_returns_404() {
        let state = test_state();
        let app = build_router(state);

        let req = Request::builder()
            .uri("/nonexistent")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn health_returns_json() {
        let state = test_state();
        let app = build_router(state);

        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 64)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
    }

    #[tokio::test]
    async fn health_shows_tick() {
        let state = test_state();
        {
            let mut guard = state.write();
            guard.tick = 42;
        }
        let app = build_router(state);

        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 64)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["tick"], 42);
    }

    #[tokio::test]
    async fn spheres_empty_returns_empty_array() {
        let state = test_state();
        let app = build_router(state);

        let req = Request::builder()
            .uri("/spheres")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 64)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["spheres"].as_array().unwrap().is_empty());
    }
}
