//! # M29: IPC Bus
//! Unix domain socket at /run/user/1000/pane-vortex-bus.sock. NDJSON wire protocol.
//! `BufReader::lines()` for framing. Glob-pattern subscriptions. Max 50 connections.
//! ## Layer: L7 | Module: M29 | Dependencies: L1, L3 (M15 `BusState`)
//! ## Wire Protocol: Handshake → Welcome → Subscribe/Submit/Event frames
//! ## Design Constraints: C5 (lock ordering), C12 (bounded channel 256), I01-I10 (IPC patterns)
//! ## Related: [IPC Bus Spec](../../ai_specs/IPC_BUS_SPEC.md), [Wire Protocol](../../ai_specs/WIRE_PROTOCOL_SPEC.md)
