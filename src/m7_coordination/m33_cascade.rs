//! # M33: Cascade System
//! CascadeHandoff/CascadeAck frames. Rate limiting: max 10/minute. Depth tracking (auto-summarize at >3).
//! Markdown fallback briefs for non-bus-aware recipients.
//! ## Layer: L7 | Module: M33 | Dependencies: L1, L7 (M29 bus, M30 types)
//! ## NA Gap: NA-P-7 (cascade rejection) — V3.3.3 adds `reject_cascade` frame
