---
date: 2026-03-19
tags: [research, web-search, resources, kuramoto, axum, hebbian, governance, sqlite, ipc]
---

# Web Research — External Resources for V3 Deployment

Searched 2026-03-19 during Session 040 scaffolding phase.

## 1. Kuramoto Model

- [Higher-dimensional Kuramoto on networks (arxiv 2603.08352)](https://arxiv.org/html/2603.08352) — Matrix-weighted network framework for interactions described by orthonormal matrices. Relevant to our nested Kuramoto (inner+outer field).
- [Kuramoto model (Wikipedia)](https://en.wikipedia.org/wiki/Kuramoto_model) — Canonical reference for dθi/dt = ωi + K/N Σ sin(θj - θi).
- [Adaptive Couplings (SIAM)](https://epubs.siam.org/doi/10.1137/15M101484X) — Dynamics-preserving coupling adaptation. Directly relevant to our auto-K and consent-gated coupling.
- [Non-reciprocity effects (arxiv 2511.15845)](https://arxiv.org/html/2511.15845v1) — Asymmetric coupling. Relevant to our weighted w² topology.
- [Kuramoto paradigm for synchronization (Scala/UC3M)](https://scala.uc3m.es/publications_MANS/PDF/finalKura.pdf) — Comprehensive mathematical treatment.

**Gap:** No Rust implementations found. Our codebase is likely the most complete Rust Kuramoto implementation.

## 2. Axum 0.8 Best Practices

- [Axum 0.8.0 announcement (Tokio blog)](https://tokio.rs/blog/2025-01-01-announcing-axum-0-8-0) — Path syntax change: `/:single` → `/{single}`. OptionalFromRequestParts trait.
- [Ultimate Axum Guide (Shuttle)](https://www.shuttle.dev/blog/2023/12/06/using-axum-rust) — Production patterns, middleware, error handling.
- [Rust Web Development 2026 (Calmops)](https://calmops.com/programming/rust-web-development-2026/) — Framework comparison, Axum recommended for new projects.
- [Production REST APIs with Axum (OneUpTime)](https://oneuptime.com/blog/post/2026-01-07-rust-axum-rest-api/view) — Routing, middleware, validation, graceful shutdown.
- [Axum docs.rs](https://docs.rs/axum/latest/axum/) — Official API reference.

**Key change for V2:** Use `/{param}` syntax not `/:param`. OptionalFromRequestParts for optional extractors.

## 3. Hebbian STDP

- [STDP (Wikipedia)](https://en.wikipedia.org/wiki/Spike-timing-dependent_plasticity) — Temporal asymmetry: pre-before-post → LTP, post-before-pre → LTD.
- [STDP review (PubMed 18275283)](https://pubmed.ncbi.nlm.nih.gov/18275283/) — STDP as Hebbian learning rule.
- [Time-Integrated STDP (arxiv 2407.10028)](https://arxiv.org/html/2407.10028v1) — Integrates timing over windows rather than discrete spikes. Potentially relevant to our continuous coupling model.
- [Stable Hebbian from STDP (J Neuroscience)](https://www.jneurosci.org/content/20/23/8812) — Mathematical stability analysis. Our weight floor (0.15) and decay (0.995) ensure stability.
- [NEST STDP models](https://nest-simulator.readthedocs.io/en/v2.20.0/models/stdp.html) — Reference implementation in neural simulator.

**Insight:** Time-Integrated STDP could improve our model — we currently use discrete tick-based co-activation rather than continuous timing.

## 4. Collective Governance in Distributed Systems

- [Coordination transparency (Springer AI & Society 2026)](https://link.springer.com/article/10.1007/s00146-026-02853-w) — Four components: interaction logging, live monitoring, intervention hooks, boundary conditions. Directly maps to our conductor + bus events + consent gates.
- [Multi-Agent Collaboration Mechanisms (arxiv 2501.06322)](https://arxiv.org/html/2501.06322v1) — Actors, types, structures, strategies, coordination protocols taxonomy.
- [Consensus Algorithms for Agent Systems](https://notes.muthu.co/2025/11/consensus-algorithms-for-coordinating-agreement-in-distributed-agent-systems/) — Contract net (47%), market-based (29%), distributed constraint optimization (18%).
- [MCP for Multi-Agent Systems (arxiv 2504.21030)](https://arxiv.org/html/2504.21030v1) — MCP as standardized context sharing mechanism. Relevant to our planned MCP adapter.
- [Consensus and Cooperation (IEEE)](https://labs.engineering.asu.edu/acs/wp-content/uploads/sites/33/2016/09/Consensus-and-Cooperation-in-Networked-Multi-Agent-Systems-2007.pdf) — Olfati-Saber canonical reference.

**Key finding:** Our proposal+voting system (V3.4) maps to "contract net protocol" pattern — the most commonly implemented coordination mechanism (47% of systems).

## 5. SQLite WAL Mode (rusqlite)

- [Rusqlite GitHub](https://github.com/rusqlite/rusqlite) — Ergonomic SQLite bindings for Rust.
- [Limbo: SQLite rewrite in Rust (Turso)](https://turso.tech/blog/introducing-limbo-a-complete-rewrite-of-sqlite-in-rust) — Future migration path if needed.
- [Rusqlite Guide 2025](https://generalistprogrammer.com/tutorials/rusqlite-rust-crate-guide) — Connection patterns, WAL setup.
- [Rust ORMs 2026 comparison](https://aarambhdevhub.medium.com/rust-orms-in-2026-diesel-vs-sqlx-vs-seaorm-vs-rusqlite-which-one-should-you-actually-use-706d0fe912f3) — Rusqlite recommended for embedded/bundled SQLite.

**Best practices:** WAL + synchronous=NORMAL. Checkpoint every 1000 writes. busy_timeout(5000). Truncate WAL on startup.

## 6. Tokio Unix Domain Socket IPC

- [Axum Unix domain socket example](https://github.com/tokio-rs/axum/blob/main/examples/unix-domain-socket/src/main.rs) — Official example. Can serve axum over UDS.
- [Fast Unix Sockets with Tokio](https://ice.computer/blog/fast-unix-sockets-with-tokio) — Performance patterns.
- [tokio-unix-ipc crate](https://docs.rs/tokio-unix-ipc/latest/tokio_unix_ipc/) — Minimal IPC abstraction. Supports file handle passing.
- [Unix Domain Sockets in Tokio](https://app.studyraid.com/en/read/10838/332164/unix-domain-sockets) — Tutorial with UnixStream/UnixListener.

**Pattern:** Our NDJSON-over-UDS approach is standard. Use `BufReader::new(stream).lines()` for line-delimited framing.

## Summary: What's Novel About Our Approach

1. **Kuramoto + Hebbian + IPC bus** — no existing system combines oscillator synchronization with STDP learning and Unix socket coordination
2. **Consent-gated coupling** — unique NA approach to distributed agent governance
3. **Collective voting on field parameters** — maps to contract net protocol but applied to Kuramoto field dynamics
4. **Bimodal learning** — our POVM pathway distribution shows phase-transitive crystallization not seen in standard STDP literature
