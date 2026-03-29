---
date: 2026-03-22
tags: [research, sidecar, features, architecture, coordination, fleet, observability, resilience]
session: 049b
---

# Sidecar Feature Research — Cutting-Edge Capabilities for AI Agent Fleet Coordination

Searched 2026-03-22 during Session 049b. This document catalogs features from modern
service mesh sidecars, multi-agent orchestration frameworks, and distributed systems
research that could differentiate pane-vortex's sidecar from a basic message relay.

Context: The sidecar is a native Rust binary bridging a WASM Zellij plugin and the
Kuramoto-coupled oscillator field coordination daemon (port 8132). It manages fleet
coordination of multiple Claude Code AI agent instances running in Zellij panes.

---

## Table of Contents

1. [Intelligent Routing & Dispatch](#1-intelligent-routing--dispatch)
2. [Resilience Patterns](#2-resilience-patterns)
3. [Observability & Telemetry](#3-observability--telemetry)
4. [Agent Coordination Protocols](#4-agent-coordination-protocols)
5. [Consensus Alternatives](#5-consensus-alternatives)
6. [Event-Driven Multi-Agent Patterns](#6-event-driven-multi-agent-patterns)
7. [Claude Code Hook Integration](#7-claude-code-hook-integration)
8. [Zellij Plugin IPC Advances](#8-zellij-plugin-ipc-advances)
9. [Swarm Intelligence Patterns](#9-swarm-intelligence-patterns)
10. [Deployment & Traffic Patterns](#10-deployment--traffic-patterns)
11. [Prioritized Feature Backlog](#11-prioritized-feature-backlog)

---

## 1. Intelligent Routing & Dispatch

### 1.1 Semantic / Content-Aware Routing

| Attribute | Detail |
|-----------|--------|
| **What** | Route tasks to agents based on semantic content analysis, not just round-robin. Analyze the task description, tool requirements, and domain affinity to select the best-matched agent. |
| **Sources** | AWS Prescriptive Guidance "Routing dynamic dispatch patterns"; Red Hat LLM Semantic Router (Envoy ExtProc, Rust+Go hybrid); vLLM Semantic Router v0.1 "Iris" (6 signal types); MarkTechPost Gemini self-correcting system. |
| **How it applies** | The sidecar intercepts `fleet-ctl dispatch` commands and IPC bus messages. Instead of dispatching to any idle pane, it scores agents by domain affinity (Rust expertise, infrastructure work, documentation), current Kuramoto phase alignment, and recent task history. The Hebbian weight matrix already encodes co-activation patterns; use those weights as routing scores. |
| **Complexity** | Medium |
| **Value** | **Must-have** — transforms the sidecar from a relay into an intelligent dispatcher. |

### 1.2 Health-Aware Routing

| Attribute | Detail |
|-----------|--------|
| **What** | Route only to agents that pass health checks. Factor in agent responsiveness, error rate, and resource consumption. Envoy calls this "outlier detection" with panic thresholds. |
| **Sources** | Envoy proxy outlier detection; Istio destination rules; `tower` middleware ecosystem. |
| **How it applies** | The sidecar already pings PV health. Extend to track per-pane metrics: last response time, consecutive failures, memory/CPU via `btm` scraping. Agents with degraded health get reduced routing weight (soft circuit break) before full exclusion. |
| **Complexity** | Low |
| **Value** | **Must-have** — prevents dispatching to stuck/crashed panes. |

### 1.3 Capability-Based Agent Cards (A2A-Inspired)

| Attribute | Detail |
|-----------|--------|
| **What** | Each agent pane publishes a capability card (JSON) describing what it can do, its tools, domain expertise, and current load. Inspired by Google A2A protocol's Agent Cards and MCP tool discovery. |
| **Sources** | Google A2A protocol (Agent Cards at `/.well-known/agent.json`); A2A survey (arxiv 2505.02279); AWS A2A blog; DigitalOcean A2A vs MCP comparison. |
| **How it applies** | Each Claude Code instance registers its available MCP servers, loaded skills, active CLAUDE.md rules, and domain tags with the sidecar. The sidecar maintains a capability registry. Dispatch decisions use capability matching: "this task needs GitHub MCP + Rust expertise" routes to the agent with those capabilities. |
| **Complexity** | Medium |
| **Value** | **Must-have** — foundational for intelligent routing. |

---

## 2. Resilience Patterns

### 2.1 Circuit Breaking

| Attribute | Detail |
|-----------|--------|
| **What** | Track failure rates per agent/destination. When failures exceed a threshold (e.g., 50%), stop routing to that agent. After a cooldown period, allow a probe request through. If it succeeds, close the circuit. Three states: Closed (normal), Open (failing, reject), Half-Open (probing). |
| **Sources** | Envoy circuit breaking (max_connections, max_pending_requests, max_retries); `tower-circuitbreaker` Rust crate (tower middleware); `rssafecircuit` crate; `revoke-resilience` crate. |
| **How it applies** | The sidecar wraps each pane connection with a circuit breaker. If a pane stops responding to IPC messages (3 consecutive timeouts), the circuit opens. The sidecar stops dispatching to that pane and notifies the conductor. After 30s, a lightweight health ping probes the pane. Existing `tower` ecosystem provides zero-cost abstraction layers. |
| **Complexity** | Low (using `tower-resilience` crate) |
| **Value** | **Must-have** — prevents cascading failures across the fleet. |

### 2.2 Bulkhead Isolation

| Attribute | Detail |
|-----------|--------|
| **What** | Partition resources so that failure in one agent/task domain cannot exhaust resources for others. Like ship bulkheads preventing one hull breach from sinking the whole vessel. Limit concurrent operations per partition. |
| **Sources** | `tower-resilience` BulkheadLayer (max_concurrent_calls); Envoy cluster circuit breaking; Microsoft resilience patterns. |
| **How it applies** | Partition the fleet into task domains (e.g., "code-editing", "research", "infrastructure"). Each partition has a max concurrent dispatch limit. If all 3 code-editing agents are busy, new code tasks queue rather than overwhelming the fleet. Critical tasks (governance votes, emergency repairs) get a reserved partition that is never exhausted. |
| **Complexity** | Medium |
| **Value** | **Nice-to-have** — important at scale (8+ agents), less critical for small fleets. |

### 2.3 Retry Budgets

| Attribute | Detail |
|-----------|--------|
| **What** | Instead of unbounded retries, allocate a fixed retry budget per time window (e.g., 20% of total requests can be retries). Prevents retry storms that amplify failures. |
| **Sources** | Envoy retry budgets; Istio retry policies; Google SRE "Handling Overload" chapter. |
| **How it applies** | When a dispatched task fails (agent crash, timeout, hook rejection), the sidecar can retry on a different agent. But total retries per 60s window are capped at 20% of total dispatches. Once the budget is exhausted, failures propagate to the conductor for human intervention rather than silently hammering the fleet. |
| **Complexity** | Low |
| **Value** | **Nice-to-have** — prevents retry storms in degraded fleet states. |

### 2.4 Request Hedging

| Attribute | Detail |
|-----------|--------|
| **What** | For critical tasks, dispatch to multiple agents simultaneously and use the first successful response. Cancel the remaining in-flight requests. Trades resource cost for latency reduction. |
| **Sources** | Envoy request hedging; Google "The Tail at Scale" (Dean & Barroso); AWS retry with hedging. |
| **How it applies** | For time-critical operations (governance votes with deadlines, emergency bug fixes), the sidecar dispatches to 2 agents simultaneously. First agent to produce a valid result wins; the other is cancelled. Since Claude Code instances are expensive (token cost), hedging should be reserved for high-priority tasks only. A priority field in the dispatch message controls whether hedging activates. |
| **Complexity** | Medium |
| **Value** | **Future** — high token cost makes this expensive; useful only for critical-path tasks. |

### 2.5 Chaos Engineering Hooks

| Attribute | Detail |
|-----------|--------|
| **What** | Built-in fault injection: add latency, drop messages, corrupt responses, kill agents. Validate that the fleet gracefully degrades under failure conditions. |
| **Sources** | Istio fault injection (VirtualService config); Chaos Mesh sidecar containers; Toxiproxy TCP proxy; Envoy fault filter. |
| **How it applies** | The sidecar exposes a `/chaos` API endpoint (disabled by default, enabled via config flag). Operators can inject: artificial latency on IPC messages, random message drops at configurable rates, forced circuit breaker trips, simulated agent crashes. This validates that Kuramoto field re-synchronization, Hebbian weight decay, and conductor failover all work correctly under stress. |
| **Complexity** | Medium |
| **Value** | **Future** — valuable for hardening but not needed until fleet is stable. |

---

## 3. Observability & Telemetry

### 3.1 OpenTelemetry Integration (Traces + Metrics + Logs)

| Attribute | Detail |
|-----------|--------|
| **What** | Emit structured telemetry using the OpenTelemetry standard. Traces show the full lifecycle of a task across agents. Metrics capture fleet health KPIs. Logs provide structured event streams. |
| **Sources** | `opentelemetry` Rust crate (API + SDK); `tracing-opentelemetry` bridge crate; `init-tracing-opentelemetry` for one-line setup; Datadog OTel Rust guide; SigNoz OTel Rust guide. |
| **How it applies** | The sidecar is the ideal telemetry collection point since all messages flow through it. Each task dispatch becomes a trace with spans for: routing decision, IPC delivery, agent processing, response receipt. Metrics include: dispatch_count, dispatch_latency_ms, agent_error_rate, circuit_breaker_state, kuramoto_order_parameter, hebbian_weight_distribution. The `tracing` crate already used in PV2 bridges directly to OTel via `tracing-opentelemetry`. |
| **Complexity** | Medium (Rust OTel ecosystem is mature) |
| **Value** | **Must-have** — without observability, fleet coordination is a black box. |

### 3.2 Agent-Specific Distributed Tracing

| Attribute | Detail |
|-----------|--------|
| **What** | Trace a task from dispatch through agent execution to completion, including all tool calls, sub-agent spawns, and hook firings within that agent. Each span carries token usage, latency, and outcome metadata. |
| **Sources** | Maxim AI agent observability (traces + spans for LLM calls, tool usage, retrieval); TrueFoundry AI model tracing; standard W3C Trace Context propagation. |
| **How it applies** | The sidecar generates a trace_id for each dispatched task and injects it via Claude Code's `SessionStart` hook as `additionalContext`. All subsequent hook firings (PreToolUse, PostToolUse, Stop) include the trace_id in their HTTP hook callbacks to the sidecar. The sidecar correlates all events into a single distributed trace. This gives unprecedented visibility: "Task X was dispatched to pane 3, called Bash 4 times, used 12K tokens, completed in 47s with 2 retries." |
| **Complexity** | High (requires hook integration + correlation engine) |
| **Value** | **Must-have** — differentiates from all existing agent coordination systems. |

### 3.3 Kuramoto Field Metrics Dashboard

| Attribute | Detail |
|-----------|--------|
| **What** | Real-time export of Kuramoto-specific metrics: order parameter R(t), per-sphere phase, coupling matrix weights, sync threshold crossings, chimera state detection, K_mod budget utilization. |
| **Sources** | arxiv 2508.12314 (Kuramoto for AI agent coordination, defines R(t) order parameter); existing PV2 `/coupling/matrix` and `/field/status` endpoints. |
| **How it applies** | The sidecar scrapes PV2's existing field endpoints every tick and exports as Prometheus-compatible metrics. Grafana dashboards (or terminal-based `btm`-style UI) show: R(t) over time, per-agent phase wheel, Hebbian weight heatmap, sync/desync events. This makes the Kuramoto coordination visible and debuggable rather than abstract. |
| **Complexity** | Low (PV2 already exposes the data; sidecar just reformats) |
| **Value** | **Must-have** — makes the Kuramoto field operationally useful. |

### 3.4 Token Usage Accounting

| Attribute | Detail |
|-----------|--------|
| **What** | Track token consumption per agent, per task, per session. Enable cost allocation and budget enforcement across the fleet. |
| **Sources** | Claude Code `PostToolUse` hooks expose token metadata; agent observability platforms (Maxim, TrueFoundry) track per-span token counts. |
| **How it applies** | The sidecar intercepts `PostToolUse` and `Stop` hook callbacks that include token usage. Accumulates per-agent and per-task totals. Enforces configurable budget limits: if agent X exceeds 100K tokens on a single task, the sidecar triggers a Stop hook to halt the agent and report to the conductor. Enables fleet-wide cost dashboards. |
| **Complexity** | Medium |
| **Value** | **Nice-to-have** — critical for cost management at scale. |

---

## 4. Agent Coordination Protocols

### 4.1 A2A-Style Agent Discovery & Negotiation

| Attribute | Detail |
|-----------|--------|
| **What** | Implement a lightweight version of Google's A2A protocol for agent-to-agent communication. Each agent publishes an Agent Card; agents can discover, negotiate, and delegate tasks to peers. |
| **Sources** | Google A2A protocol (April 2025); A2A specification (Agent Cards, task lifecycle, SSE streaming); arxiv 2505.02279 (protocol survey: MCP + ACP + A2A + ANP); AWS A2A blog. |
| **How it applies** | The sidecar acts as the A2A server for the fleet. Each agent's capabilities (loaded skills, MCP servers, domain tags) are registered as an Agent Card. The sidecar handles discovery requests ("who can do Rust refactoring?"), task delegation ("assign this to the best match"), and lifecycle management (pending/active/completed/failed). This provides a standardized interoperability layer if the fleet ever needs to coordinate with external agent systems. |
| **Complexity** | High |
| **Value** | **Future** — valuable for ecosystem interop but overkill for internal fleet today. |

### 4.2 MCP-Bridged Tool Sharing

| Attribute | Detail |
|-----------|--------|
| **What** | Agents in the fleet expose their MCP tools to each other via the sidecar. Agent A can invoke Agent B's unique MCP server (e.g., a specialized database connector) without both agents needing it installed. |
| **Sources** | MCP protocol (JSON-RPC client-server for tool invocation); A2A+MCP integration patterns (a2a-protocol.org); Confluent event-driven agent ebook. |
| **How it applies** | The sidecar maintains a registry of all MCP tools available across all fleet agents. When Agent A needs a tool only available on Agent B, the sidecar proxies the MCP call: serializes the request, routes it to Agent B's MCP server, returns the result to Agent A. This enables tool specialization without tool duplication. |
| **Complexity** | High |
| **Value** | **Future** — requires MCP proxy implementation; valuable when fleet has heterogeneous tool sets. |

### 4.3 Shared Blackboard / Knowledge Base

| Attribute | Detail |
|-----------|--------|
| **What** | A shared, persistent knowledge store that all agents read from and write to. Agents post findings, partial results, and coordination signals. Other agents consume relevant entries asynchronously. |
| **Sources** | Confluent/InfoWorld "Four Design Patterns for Event-Driven Multi-Agent Systems" (Blackboard Pattern); arxiv 2507.01701 (LLM Blackboard Architecture); NexaStack distributed cache/blackboard systems. |
| **How it applies** | The sidecar hosts an in-memory blackboard (backed by SQLite for persistence) where agents post discoveries: "found bug in module X", "file Y was modified", "test suite now passing". Other agents subscribe to relevant topics. This replaces ad-hoc file-based coordination (arena/ directory) with structured, queryable shared state. The Kuramoto field's phase alignment naturally prioritizes blackboard entries from synchronized agents. |
| **Complexity** | Medium |
| **Value** | **Must-have** — the arena/ directory is already an ad-hoc blackboard; formalize it. |

---

## 5. Consensus Alternatives

### 5.1 Raft (for Small Trusted Clusters)

| Attribute | Detail |
|-----------|--------|
| **What** | Simpler consensus protocol than PBFT for clusters where all participants are trusted (crash-fault tolerance only, not Byzantine). Leader-based with log replication. Much lower message complexity than PBFT. |
| **Sources** | Raft paper (Ongaro & Ousterhout); comparative analysis (anshadameenza.com); Nature Scientific Reports (Raft + Tendermint for energy efficiency). |
| **How it applies** | The fleet agents are all Claude Code instances on the same machine, controlled by the same user. Byzantine fault tolerance (protecting against malicious agents) is unnecessary. Raft provides crash-fault tolerance with O(n) message complexity vs PBFT's O(n^2). For governance votes and field parameter updates, Raft is sufficient and dramatically simpler. Keep PBFT as an option for multi-machine fleet deployments. |
| **Complexity** | Medium (Raft implementations exist in Rust: `raft-rs`, `openraft`) |
| **Value** | **Nice-to-have** — simplifies consensus for single-machine fleets. |

### 5.2 HotStuff (Linear View-Change BFT)

| Attribute | Detail |
|-----------|--------|
| **What** | BFT consensus with O(n) view-change complexity (vs PBFT's O(n^2)). HotStuff-1 further reduces latency by 2 network hops. Maintains Byzantine fault tolerance while being dramatically more scalable. |
| **Sources** | HotStuff paper (arxiv 1803.05069); HotStuff-1 (arxiv 2408.04728); Efficient-HotStuff (OpenReview). |
| **How it applies** | If the fleet grows beyond 7-10 agents, PBFT's quadratic view-change becomes a bottleneck. HotStuff provides the same safety guarantees with linear complexity. HotStuff-1's speculative execution maps naturally to the sidecar's request hedging feature. This is the upgrade path from PBFT if governance overhead becomes measurable. |
| **Complexity** | High (no mature Rust HotStuff crate; would need implementation) |
| **Value** | **Future** — only needed if fleet scales significantly. |

### 5.3 Optimistic Consensus with Rollback

| Attribute | Detail |
|-----------|--------|
| **What** | Assume consensus will be reached and execute optimistically. If consensus fails, roll back. Dramatically reduces latency for the common case. Used in speculative execution systems. |
| **Sources** | HotStuff-1 speculation mechanism; database optimistic concurrency control; Tendermint fast-path. |
| **How it applies** | For most governance proposals (non-destructive field parameter changes), the sidecar applies the change immediately and runs consensus in the background. If consensus fails (rare), the sidecar rolls back. This reduces governance latency from 2-3 round trips to near-zero for the 95% case. The Kuramoto field's natural damping absorbs small parameter oscillations from occasional rollbacks. |
| **Complexity** | Medium |
| **Value** | **Nice-to-have** — significant latency improvement for governance operations. |

---

## 6. Event-Driven Multi-Agent Patterns

### 6.1 Orchestrator-Worker Pattern (via Event Streams)

| Attribute | Detail |
|-----------|--------|
| **What** | Central orchestrator distributes command messages across partitions. Workers pull from assigned partitions as consumer groups. Output flows to a second topic. Key-based partitioning enables ordered processing per domain. |
| **Sources** | Confluent "Four Design Patterns" (Kafka-based); InfoWorld distributed state article; Codebridge multi-agent guide. |
| **How it applies** | The sidecar already acts as an orchestrator. Formalize this with a partitioned task queue: partitions by domain (code, infra, docs, research). Each partition maps to a subset of agents. The IPC bus becomes the event backbone. Workers (agents) consume tasks from their partition, produce results to a completion topic. The conductor reads the completion topic to update field state. |
| **Complexity** | Medium |
| **Value** | **Must-have** — this is essentially what the sidecar already does; formalize it. |

### 6.2 Market-Based Task Allocation

| Attribute | Detail |
|-----------|--------|
| **What** | Instead of centralized dispatch, tasks are posted as "asks" and agents bid on them. A market maker matches bids to tasks. Agents bid based on capability, current load, and domain affinity. Decentralized, avoids quadratic connections. |
| **Sources** | Confluent market-based pattern; Gurusup agent orchestration patterns; academic multi-agent systems literature. |
| **How it applies** | For non-urgent tasks, the sidecar posts a task announcement to all agents. Agents evaluate the task against their capabilities and current workload, then submit a bid (capability score + estimated completion time). The sidecar's market maker selects the best bid. This gives agents agency in task selection, improving the Kuramoto field's natural synchronization (agents self-select into aligned work). |
| **Complexity** | Medium |
| **Value** | **Nice-to-have** — elegant but adds latency vs direct dispatch; best for non-time-critical tasks. |

### 6.3 Stigmergy (Indirect Communication via Environment)

| Attribute | Detail |
|-----------|--------|
| **What** | Agents coordinate by modifying shared environment state (like ant pheromone trails). No direct agent-to-agent messages needed. Agents read environmental signals and adjust behavior. Scales linearly with agent count. |
| **Sources** | Nature article "Automatic design of stigmergy-based behaviours" (2024); academic swarm robotics literature; Kamran "Collective Stigmergic Optimization" (Medium). |
| **How it applies** | The Kuramoto field itself IS a stigmergic medium. Agent phases, coupling weights, and field order parameter are environmental signals that all agents read. The sidecar can amplify this: when Agent A edits file X, a "pheromone" (metadata tag) is deposited on file X. When Agent B encounters file X, the pheromone signals "recently modified, check for conflicts." The Hebbian weight matrix naturally provides stigmergic memory: frequently co-activated agent pairs develop stronger coupling without explicit coordination. |
| **Complexity** | Low (the Kuramoto field already does this) |
| **Value** | **Must-have** — recognize and amplify what the architecture already provides. |

---

## 7. Claude Code Hook Integration

### 7.1 Full Hook Event Coverage (22 Events)

| Attribute | Detail |
|-----------|--------|
| **What** | Claude Code now exposes 22 hook events with HTTP, command, prompt, and agent handler types. Key events for fleet coordination: SessionStart, UserPromptSubmit, PreToolUse, PostToolUse, Stop, SubagentStart, SubagentStop, TeammateIdle, TaskCompleted, Notification, StopFailure. |
| **Sources** | Claude Code Hooks Reference (code.claude.com/docs/en/hooks); Pixelmojo hooks guide; DataCamp hooks tutorial; Claude Agent SDK hooks documentation. |
| **How it applies** | The sidecar registers as an HTTP hook endpoint for ALL fleet-relevant events. This provides complete lifecycle visibility without modifying agent code. The sidecar's HTTP endpoint receives structured JSON for every significant agent action. |

**Critical new hooks since Session 049:**

| Hook Event | Sidecar Use | Priority |
|------------|-------------|----------|
| `TeammateIdle` | Detect when a fleet agent is about to go idle; reassign work or let it rest based on Kuramoto phase | **Must-have** |
| `TaskCompleted` | Validate task completion criteria before accepting; reject incomplete work back to agent | **Must-have** |
| `PermissionRequest` | Auto-approve safe operations fleet-wide via sidecar policy engine; deny dangerous ops centrally | **Must-have** |
| `StopFailure` | Detect rate limits, billing errors, auth failures across fleet; trigger circuit breaker on affected agent | **Must-have** |
| `ConfigChange` | Audit and optionally block configuration changes that would break fleet coordination | **Nice-to-have** |
| `PreCompact` / `PostCompact` | Track when agents compact context; inject critical fleet state into post-compact context | **Nice-to-have** |
| `Elicitation` | Auto-respond to MCP elicitation requests based on fleet policy | **Future** |
| `WorktreeCreate` / `WorktreeRemove` | Track worktree lifecycle for agents using isolated worktrees | **Future** |

**Complexity:** Low-Medium per hook (HTTP endpoint already exists; add handlers)
**Value:** **Must-have** — this is the primary integration surface.

### 7.2 HTTP Hook Endpoint Architecture

| Attribute | Detail |
|-----------|--------|
| **What** | The sidecar runs an HTTP server that receives hook callbacks from all fleet agents. Each agent's `.claude/settings.json` points its hooks to `http://localhost:SIDECAR_PORT/hooks/{event_name}`. |
| **Sources** | Claude Code HTTP hooks spec (type: "http", url, headers, allowedEnvVars, timeout). |
| **How it applies** | Single HTTP endpoint on the sidecar handles all hook events from all agents. The agent's `session_id` in the hook payload identifies which pane/sphere sent it. The sidecar correlates events into per-agent traces, updates the Kuramoto field state, and returns control decisions (allow/deny/block) based on fleet policy. This is zero-config for agents beyond the initial settings.json. |
| **Complexity** | Low (Axum HTTP server already in PV2) |
| **Value** | **Must-have** — the simplest and most powerful integration path. |

### 7.3 Permission Policy Engine

| Attribute | Detail |
|-----------|--------|
| **What** | Centralized permission decisions for the fleet. The sidecar's `PermissionRequest` hook handler auto-approves safe operations (read files in workspace, run tests, git status) and auto-denies dangerous ones (rm -rf, git push --force, writing outside workspace). Eliminates per-agent permission prompts that block fleet execution. |
| **Sources** | Claude Code PermissionRequest hook (hookSpecificOutput with behavior: allow/deny, updatedPermissions); managed policy settings. |
| **How it applies** | Every permission dialog across the fleet is intercepted by the sidecar. A policy engine (rules in TOML config) evaluates: tool name, command content, file paths, agent role. Safe patterns are auto-approved. Dangerous patterns are auto-denied with explanation. Ambiguous patterns are escalated to the human operator via Notification hook. This transforms fleet operation from "approve 50 permission dialogs" to "review 2 unusual requests." |
| **Complexity** | Medium |
| **Value** | **Must-have** — single biggest UX improvement for fleet operation. |

### 7.4 Stop / SubagentStop Quality Gates

| Attribute | Detail |
|-----------|--------|
| **What** | Use Stop and SubagentStop hooks to enforce completion criteria. If an agent tries to stop without running tests, the hook returns `decision: "block"` with a reason, forcing the agent to continue. |
| **Sources** | Claude Code Stop hook (decision: block forces continuation); SubagentStop hook (same pattern); TaskCompleted hook (exit 2 + stderr for rejection). |
| **How it applies** | The sidecar defines quality gates per task type: code tasks must pass `cargo check`, documentation tasks must include examples, infrastructure tasks must verify health endpoints. When an agent signals completion, the sidecar evaluates the gate. Failed gates return `decision: "block"` with specific feedback ("Tests not passing: 3 failures in m12_chimera"). The agent continues working until the gate passes. |
| **Complexity** | Medium |
| **Value** | **Must-have** — ensures fleet output quality without human review of every task. |

---

## 8. Zellij Plugin IPC Advances

### 8.1 Bidirectional Pipe Protocol

| Attribute | Detail |
|-----------|--------|
| **What** | Zellij pipes (0.40+) provide unidirectional message channels to/from/between plugins. Pipes support named channels with arbitrary serializable payloads. CLI-to-plugin and plugin-to-plugin communication. External data can be piped via stdin. |
| **Sources** | Zellij 0.40 pipes documentation; DeepWiki Zellij plugin communication; Zellij pipe CLI reference. |
| **How it applies** | The sidecar uses `zellij pipe` to send structured commands to the WASM plugin: "update phase for pane 3", "display field visualization", "highlight synchronized agents." The plugin uses `pipe_message_to_plugin` for plugin-to-plugin coordination without going through the sidecar. Protocol Buffers (prost) provide stable serialization across the WASM boundary. |
| **Complexity** | Low (pipes already exist; use them more) |
| **Value** | **Must-have** — the primary WASM plugin communication channel. |

### 8.2 Plugin Manager Integration (Zellij 0.41+)

| Attribute | Detail |
|-----------|--------|
| **What** | Zellij 0.41 added a built-in plugin manager for discovering, installing, and managing plugins. Config live reloading enables runtime reconfiguration. |
| **Sources** | Zellij 0.41 release notes; non-colliding keybindings preset; config live reload. |
| **How it applies** | The fleet coordination WASM plugin can be distributed and updated via the plugin manager. Config live reloading means the sidecar can dynamically reconfigure Zellij layouts (add/remove panes, change keybindings) without session restart. This enables dynamic fleet scaling: "add 2 more agent panes" at runtime. |
| **Complexity** | Low |
| **Value** | **Nice-to-have** — simplifies plugin distribution and runtime reconfiguration. |

### 8.3 Web Client Access (Zellij 0.43)

| Attribute | Detail |
|-----------|--------|
| **What** | Zellij 0.43 introduced web-based client access to terminal sessions. |
| **Sources** | Zellij 0.43 release notes. |
| **How it applies** | Remote fleet monitoring: operators can observe the fleet via web browser without SSH. The sidecar could expose a web dashboard that embeds Zellij web client views alongside Kuramoto field visualizations. This is especially valuable for multi-machine fleet deployments. |
| **Complexity** | Medium |
| **Value** | **Future** — valuable for remote operation but not critical for local fleet. |

### 8.4 Pinned Floating Panes (Zellij 0.42)

| Attribute | Detail |
|-----------|--------|
| **What** | Floating panes can be pinned to stay visible across tab switches. |
| **Sources** | Zellij 0.42 release notes (Stacked Resize, Pinned Floating Panes). |
| **How it applies** | A pinned floating pane running the field visualization plugin stays visible regardless of which agent tab the operator is viewing. This provides persistent fleet-wide situational awareness. The sidecar updates this pane with real-time field status via pipes. |
| **Complexity** | Low |
| **Value** | **Must-have** — trivial to implement, significant UX improvement. |

---

## 9. Swarm Intelligence Patterns

### 9.1 Separation-Alignment-Cohesion (Boids Rules)

| Attribute | Detail |
|-----------|--------|
| **What** | Three simple local rules produce emergent coordinated behavior: Separation (avoid crowding neighbors), Alignment (steer toward average heading), Cohesion (steer toward average position). No central controller. |
| **Sources** | Reynolds Boids (1987); arxiv drone swarm coordination using MAS+SI; Frontiers swarm robotics review; Nature SCM (Swarm Cooperation Model). |
| **How it applies** | Map to the Kuramoto field: Separation = maintain phase diversity (agents should not all work on the same thing); Alignment = Kuramoto coupling (agents trend toward phase coherence on related work); Cohesion = field order parameter R(t) pulls agents toward collective goals. The sidecar enforces separation by detecting task overlap and reassigning redundant agents. Alignment is automatic via Kuramoto. Cohesion is the R_TARGET parameter. |
| **Complexity** | Low (already implemented via Kuramoto; formalize the mapping) |
| **Value** | **Nice-to-have** — provides intuitive framework for tuning fleet behavior. |

### 9.2 Positive/Negative Feedback Loops

| Attribute | Detail |
|-----------|--------|
| **What** | Positive feedback amplifies successful patterns (ant pheromone trails get stronger with use). Negative feedback prevents runaway behavior (pheromones evaporate over time). Balance between the two produces adaptive behavior. |
| **Sources** | Nature Communications "collective intelligence model" (SCM); PMC animal collective behaviors review; MarkTechPost swarm architecture comparison. |
| **How it applies** | Positive feedback: Hebbian LTP strengthens coupling between agents that successfully collaborate. When agents A and B complete a task together, their coupling weight increases, making future co-assignment more likely. Negative feedback: Hebbian LTD and EMA decay (alpha=0.95) weaken stale connections. The sidecar can amplify these feedback loops: on task success, inject a Hebbian co-activation burst; on task failure, apply additional weight depression. |
| **Complexity** | Low (Hebbian STDP already implemented; sidecar adds triggers) |
| **Value** | **Must-have** — the sidecar is the feedback loop amplifier. |

### 9.3 Adaptive Coupling via Kuramoto Order Parameter

| Attribute | Detail |
|-----------|--------|
| **What** | Dynamically adjust global coupling strength K based on the order parameter R(t). When R is low (desynchronized fleet), increase K to pull agents together. When R is high (well-coordinated), relax K to allow individual exploration. |
| **Sources** | arxiv 2508.12314 (Kuramoto for AI agent coordination); PV2's existing auto-K module (m17_auto_k); SIAM adaptive couplings paper. |
| **How it applies** | The sidecar monitors R(t) from the field daemon. When R drops below SYNC_THRESHOLD (0.5), the sidecar signals auto-K to increase coupling, pulling the fleet back together. When R exceeds R_TARGET (0.93), the sidecar relaxes coupling to allow agents to explore independently. The IQR K-scaling (already implemented) provides the mechanism; the sidecar provides the decision intelligence. |
| **Complexity** | Low (auto-K exists; sidecar adds policy layer) |
| **Value** | **Must-have** — closes the feedback loop between field state and fleet behavior. |

---

## 10. Deployment & Traffic Patterns

### 10.1 Canary Dispatch

| Attribute | Detail |
|-----------|--------|
| **What** | When deploying updated agent configurations (new skills, updated CLAUDE.md, new hooks), roll out to one agent first. Monitor for regressions before fleet-wide deployment. |
| **Sources** | Kubernetes canary deployment; Istio traffic shifting; Envoy weighted routing; shadow deployment patterns. |
| **How it applies** | When updating fleet configuration (e.g., new skill deployment), the sidecar designates one agent as canary. Only that agent receives the new configuration. The sidecar monitors its error rate, completion time, and output quality for a configurable window (e.g., 5 tasks). If metrics are within tolerance, the sidecar rolls out to the remaining fleet. If not, it rolls back the canary and alerts the operator. |
| **Complexity** | Medium |
| **Value** | **Nice-to-have** — important for fleet stability during configuration changes. |

### 10.2 Shadow Dispatch (Dual Execution)

| Attribute | Detail |
|-----------|--------|
| **What** | Duplicate production tasks to a shadow agent running experimental configuration. Compare outputs without affecting production. The shadow agent's results are logged but not used. |
| **Sources** | Istio traffic mirroring; Gloo Edge shadowing; shadow deployment in microservices; ML model shadow deployment. |
| **How it applies** | When testing new agent capabilities (updated model, new skills, experimental prompts), the sidecar mirrors real tasks to a shadow agent. Both agents process the same task; only the production agent's output is used. The sidecar logs both outputs for comparison. This validates new configurations against real workloads without risk. Token cost is doubled for shadowed tasks, so use selectively. |
| **Complexity** | Medium |
| **Value** | **Future** — valuable for safe experimentation but expensive in tokens. |

### 10.3 Rate Limiting with Token Bucket

| Attribute | Detail |
|-----------|--------|
| **What** | Control the rate of task dispatches using a token bucket algorithm. Bucket fills at a steady rate; each dispatch consumes a token. Allows controlled bursts while enforcing sustained rate limits. |
| **Sources** | `async-rate-limiter` Rust crate; `leaky-bucket` crate; `rater` crate (lock-free, thread-safe); Envoy rate limiting. |
| **How it applies** | The sidecar rate-limits outbound dispatches to prevent overwhelming agents. Global rate limit (e.g., 10 dispatches/minute across fleet) prevents cascade overload. Per-agent rate limit (e.g., 2 concurrent tasks) prevents individual agent saturation. Burst allowance (bucket capacity) handles peak loads gracefully. The `rater` crate provides a lock-free, zero-overhead implementation ideal for the sidecar's hot path. |
| **Complexity** | Low (mature Rust crates available) |
| **Value** | **Must-have** — fundamental flow control for fleet operation. |

---

## 11. Prioritized Feature Backlog

### Tier 1: Must-Have (implement in next 2 sessions)

| # | Feature | Section | Complexity | Rationale |
|---|---------|---------|------------|-----------|
| 1 | HTTP Hook Endpoint (all 22 events) | 7.2 | Low | Primary integration surface; enables everything else |
| 2 | Permission Policy Engine | 7.3 | Medium | Biggest UX improvement; eliminates permission dialog spam |
| 3 | Circuit Breaking | 2.1 | Low | Prevents cascading failures; `tower-resilience` makes it trivial |
| 4 | Health-Aware Routing | 1.2 | Low | Prevents dispatching to dead agents |
| 5 | OpenTelemetry Traces + Metrics | 3.1 | Medium | Fleet is a black box without observability |
| 6 | Kuramoto Field Metrics Export | 3.3 | Low | Makes field state operationally visible |
| 7 | Rate Limiting (Token Bucket) | 10.3 | Low | Fundamental flow control |
| 8 | Semantic/Content-Aware Routing | 1.1 | Medium | Transforms sidecar from relay to intelligent dispatcher |
| 9 | Stop/SubagentStop Quality Gates | 7.4 | Medium | Ensures output quality without human review |
| 10 | Shared Blackboard | 4.3 | Medium | Formalizes arena/ into structured shared state |
| 11 | Bidirectional Pipe Protocol | 8.1 | Low | Already available; use more intentionally |
| 12 | Pinned Floating Pane Dashboard | 8.4 | Low | Trivial UX win for fleet visibility |
| 13 | Feedback Loop Amplifier | 9.2 | Low | Hebbian STDP already exists; sidecar adds triggers |
| 14 | Adaptive K Policy Layer | 9.3 | Low | Closes the feedback loop between field and fleet |

### Tier 2: Nice-to-Have (implement when Tier 1 stable)

| # | Feature | Section | Complexity | Rationale |
|---|---------|---------|------------|-----------|
| 15 | Agent Distributed Tracing | 3.2 | High | Deep visibility but requires hook correlation engine |
| 16 | Capability-Based Agent Cards | 1.3 | Medium | Enables intelligent matching but needs agent registration |
| 17 | Token Usage Accounting | 3.4 | Medium | Cost management at scale |
| 18 | Bulkhead Isolation | 2.2 | Medium | Important for 8+ agent fleets |
| 19 | Retry Budgets | 2.3 | Low | Prevents retry storms |
| 20 | Raft Consensus | 5.1 | Medium | Simpler than PBFT for trusted clusters |
| 21 | Market-Based Task Allocation | 6.2 | Medium | Elegant but adds dispatch latency |
| 22 | Canary Dispatch | 10.1 | Medium | Safe configuration rollouts |
| 23 | Optimistic Consensus | 5.3 | Medium | Reduces governance latency |
| 24 | Boids-Rule Mapping | 9.1 | Low | Intuitive fleet tuning framework |
| 25 | Plugin Manager Integration | 8.2 | Low | Simplifies distribution |

### Tier 3: Future (research and prototype)

| # | Feature | Section | Complexity | Rationale |
|---|---------|---------|------------|-----------|
| 26 | A2A Protocol Support | 4.1 | High | Ecosystem interop; overkill for internal fleet |
| 27 | MCP-Bridged Tool Sharing | 4.2 | High | Needs MCP proxy; valuable for heterogeneous fleets |
| 28 | Request Hedging | 2.4 | Medium | Expensive in tokens; critical-path only |
| 29 | Chaos Engineering Hooks | 2.5 | Medium | Hardening; needs stable fleet first |
| 30 | Shadow Dispatch | 10.2 | Medium | Safe experimentation but double token cost |
| 31 | HotStuff Consensus | 5.2 | High | Only needed at significant scale |
| 32 | Web Client Dashboard | 8.3 | Medium | Remote operation; not needed for local |
| 33 | Elicitation Auto-Response | 7.1 | Medium | Niche MCP automation |

---

## Key Architectural Insight

The research reveals a convergence point: **the sidecar should be an Envoy-like service proxy
specialized for AI agent traffic**. The same patterns that make Envoy powerful for microservices
(circuit breaking, health-aware routing, rate limiting, distributed tracing, fault injection)
apply directly to AI agent fleet coordination. The difference is that our "traffic" is task
dispatches rather than HTTP requests, and our "services" are Claude Code instances rather
than containers.

The Kuramoto oscillator field provides something no service mesh has: **a physics-based
coordination substrate** that naturally encodes agent relationships (coupling weights),
collective state (order parameter R(t)), and emergent behavior (synchronization, chimera
states). This is our differentiator. Every feature above should amplify the field's
intelligence rather than bypass it.

The Claude Code hooks system (22 events, HTTP handlers, structured JSON I/O) is the
**single most important integration surface**. It provides complete lifecycle visibility
into every agent without modifying agent code. The sidecar as an HTTP hook endpoint
is the architectural keystone that enables everything in this document.

---

## Sources

1. Google A2A Protocol: https://developers.googleblog.com/en/a2a-a-new-era-of-agent-interoperability/
2. Confluent Multi-Agent Patterns: https://www.confluent.io/blog/event-driven-multi-agent-systems/
3. arxiv 2508.12314 - Kuramoto for AI Agent Coordination: https://arxiv.org/html/2508.12314
4. arxiv 2505.02279 - Agent Protocol Survey: https://arxiv.org/html/2505.02279v2
5. Claude Code Hooks Reference: https://code.claude.com/docs/en/hooks
6. Envoy Proxy: https://www.envoyproxy.io/
7. tower-resilience (Rust): https://lib.rs/crates/tower-circuitbreaker
8. OpenTelemetry Rust: https://opentelemetry.io/docs/languages/rust/
9. Zellij Plugin Pipes: https://zellij.dev/documentation/plugin-pipes.html
10. HotStuff BFT: https://arxiv.org/abs/1803.05069
11. AWS Routing Dynamic Dispatch: https://docs.aws.amazon.com/prescriptive-guidance/latest/agentic-ai-patterns/routing-dynamic-dispatch-patterns.html
12. Red Hat LLM Semantic Router: https://developers.redhat.com/articles/2025/05/20/llm-semantic-router-intelligent-request-routing
13. Maxim Agent Observability: https://www.getmaxim.ai/products/agent-observability
14. Nature - Stigmergy Design: https://www.nature.com/articles/s44172-024-00175-7
15. Nature - Swarm Cooperation Model: https://www.nature.com/articles/s41467-025-61985-7
16. Gartner 1445% multi-agent surge: https://machinelearningmastery.com/7-agentic-ai-trends-to-watch-in-2026/
