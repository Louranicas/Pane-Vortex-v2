//! # M18: Coupling Topology
//!
//! Weight² amplification, neighborhood queries (/sphere/{id}/neighbors).
//! Adjacency index for O(degree) lookup. Topology-aware coupling (NA-25).
//!
//! ## Layer: L4 | Module: M18 | Dependencies: L1, M16
//! ## Design Constraints: C7 (owned Vec return from neighbor queries), C12 (bounded result sets)
