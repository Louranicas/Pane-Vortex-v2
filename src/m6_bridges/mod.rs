//! # Layer 6: Bridges
//!
//! External service bridges using raw TCP HTTP (fire-and-forget pattern).
//! Depends on L1 (Foundation), L3 (Field).
//!
//! ## Design Constraints: C1 C8 C14
//! - C8: ALL bridges route through `consent_gated_k_adjustment()` (M28)
//! - C14: Fire-and-forget for writes (`tokio::spawn`, no blocking)
//!
//! ## Bridge Pattern
//! `TcpStream::connect(addr) → write HTTP → read response → parse JSON`
//!
//! ## Modules
//!
//! | Module | LOC Target | Purpose |
//! |--------|-----------|---------|
//! | `m22_synthex_bridge` | ~300 | SYNTHEX :8090 thermal k_adjustment |
//! | `m23_nexus_bridge` | ~400 | SAN-K7 :8100 nested Kuramoto, strategy |
//! | `m24_me_bridge` | ~250 | ME :8080 observer fitness (BUG-008!) |
//! | `m25_povm_bridge` | ~200 | POVM :8125 snapshots + Hebbian weights |
//! | `m26_rm_bridge` | ~150 | RM :8130 TSV conductor decisions |
//! | `m27_vms_bridge` | ~150 | VMS :8120 field memory seeding |
//! | `m28_consent_gate` | ~200 | consent_gated_k_adjustment(), k_mod budget |

pub mod m22_synthex_bridge;
pub mod m23_nexus_bridge;
pub mod m24_me_bridge;
pub mod m25_povm_bridge;
pub mod m26_rm_bridge;
pub mod m27_vms_bridge;
pub mod m28_consent_gate;
