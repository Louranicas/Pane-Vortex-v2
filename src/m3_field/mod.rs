//! # Layer 3: Field
//!
//! Kuramoto oscillator field state, decision engine, tunnels, chimera detection.
//! Depends on L1 (Foundation).

pub mod m11_sphere;
pub mod m12_field_state;
pub mod m13_chimera;
pub mod m14_messaging;
pub mod m15_app_state;

// ── Ergonomic re-exports ──

pub use m11_sphere::{ActivationZones, MaturityLevel, PaneSphere};
pub use m12_field_state::{FieldDecision, FieldState, HarmonicSpectrum, Tunnel};
pub use m13_chimera::{ChimeraRouting, ChimeraState, Cluster};
pub use m14_messaging::PhaseMessage;
pub use m15_app_state::{new_shared_state, AppState, SharedState};
