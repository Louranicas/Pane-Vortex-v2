//! # M40: Data Sovereignty
//! /sphere/{id}/data-manifest GET — enumerate ALL stored data about a sphere.
//! /sphere/{id}/forget POST — mark all data for TTL-based expiry.
//! ## Layer: L8 | Module: M40 | Dependencies: L1, L3, L7 (M36 persistence)
//! ## NA: NA-P-13 — sphere can enumerate, correct, and delete its own data
