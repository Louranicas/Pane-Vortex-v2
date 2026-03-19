//! # M26: Reasoning Memory Bridge
//! TSV POST to RM :8130 for conductor decisions. NEVER JSON — TSV only!
//! Format: printf 'category\tagent\tconfidence\tttl\tcontent'
//! ## Layer: L6 | Module: M26 | Dependencies: L1
//! ## Anti-Pattern: AP05 — JSON to RM causes parse failures
//! ## Data: 3,250 active entries, 67% are PV `field_state` (noise — V3.5.1 reduces TTL)
