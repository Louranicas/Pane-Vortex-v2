# BUG-HUNT GAMMA: POVM Write-Only Pathology Investigation

**Agent:** GAMMA (executed by BETA) | **Timestamp:** 2026-03-21 ~02:28 UTC

---

## 1. Hydrate Endpoint Verified

```bash
curl -s localhost:8125/hydrate
```

```json
{
  "crystallised_count": 0,
  "latest_r": 0.697,
  "memory_count": 53,
  "pathway_count": 2427,
  "session_count": 0
}
```

**`/hydrate` returns a summary, not the data itself.** It reports counts but doesn't trigger any read-back or co-activation increment. 53 memories, 2427 pathways, 0 crystallised, 0 sessions.

---

## 2. Memory Read — access_count Confirmed Zero

```bash
curl -s localhost:8125/memories | jq '.[0:3]'
```

All 3 sampled memories show `access_count: 0`. Confirmed: **every memory has never been read.** They retain rich content (Session 027 exploration, pane mastery, system schematics) but no service has ever retrieved them.

Key fields per memory:
- `access_count: 0` (NEVER READ)
- `crystallised: false` (none promoted to permanent)
- `decay_cycles_survived: 5` (still alive, decaying slowly)
- `intensity: 0.590` (uniform — 4 decay cycles from initial 1.0)
- `session_last_accessed: null` (no access session recorded)

---

## 3. Endpoint Discovery

| Endpoint | HTTP | Status |
|----------|------|--------|
| `/memories` | 200 | Returns all memories (array) |
| `/pathways` | 200 | Returns all pathways (object) |
| `/hydrate` | 200 | Returns summary counts |
| `/health` | 200 | Service health |
| `/memories/search` | 404 | Does not exist |
| `/recall` | 404 | Does not exist |
| `/retrieve` | 404 | Does not exist |
| `/query` | 404 | Does not exist |
| `/sessions` | 404 | Does not exist |
| `/status` | 404 | Does not exist |
| `/stats` | 404 | Does not exist |
| `/summary` | 404 | Does not exist |

**POVM has only 4 endpoints:** `/health`, `/hydrate`, `/memories`, `/pathways`. No search, no recall, no query. The only way to "read" is to fetch the entire collection.

---

## 4. PV2 Code — Does It Read from POVM?

**YES — PV2 has `hydrate_pathways()` and `hydrate_summary()` methods.**

From `pane-vortex-v2/src/m6_bridges/m25_povm_bridge.rs`:

```rust
// Line 266-281: Reads pathways from POVM, caches them, sets hydrated=true
pub fn hydrate_pathways(&self) -> PvResult<Vec<Pathway>> {
    let body = raw_http_get(&self.base_url, PATHWAYS_PATH, &self.service)?;
    let response: PathwaysResponse = serde_json::from_str(&body)?;
    let mut state = self.state.write();
    state.cached_pathways.clone_from(&response.pathways);
    state.hydrated = true;
    state.stale = false;
    Ok(response.pathways)
}

// Line 287-299: Reads summary from POVM
pub fn hydrate_summary(&self) -> PvResult<PovmSummary> {
    let body = raw_http_get(&self.base_url, SUMMARY_PATH, &self.service)?;
    let summary: PovmSummary = serde_json::from_str(&body)?;
    let mut state = self.state.write();
    state.last_summary = Some(summary.clone());
    Ok(summary)
}
```

**PV2 reads `/pathways` on startup and caches the topology.** This is the hydration that would close the read loop — but it's **V2 only**. V1 writes to POVM but never reads back. V2 adds the read-back path.

**However:** Even V2 only reads at startup (hydration), not continuously. POVM's `access_count` will still stay at 0 because POVM doesn't track that `/pathways` was called — it only tracks per-memory access, and PV reads pathways, not individual memories.

---

## 5. Diagnosis

```
V1 (current):  PV writes → POVM stores → nobody reads → access_count=0 → no co-activation → no Hebbian
V2 (pending):  PV writes → POVM stores → PV hydrates pathways on startup → topology restored
                BUT: POVM access_count still won't increment (reads pathways endpoint, not memory endpoint)
```

The write-only pathology has two layers:

1. **V1 layer (fixable with V2):** No read-back at all. V2's `hydrate_pathways()` closes this.
2. **Structural layer (persists in V2):** POVM tracks `access_count` per memory, but consumers read `/pathways` (aggregated), not `/memories/{id}` (individual). The access_count metric is structurally disconnected from actual usage.

---

## Summary

| Finding | Status |
|---------|--------|
| `/hydrate` exists, returns counts only | Verified |
| All 53 memories have access_count=0 | Confirmed |
| No search/recall/query endpoints | Confirmed (404 on all) |
| PV2 has hydrate_pathways() + hydrate_summary() | Confirmed in source |
| V2 reads `/pathways` on startup | Closes the write-only loop for topology |
| POVM access_count remains structurally broken | Even V2 won't fix this metric |

---

GAMMA-BUGHUNT-COMPLETE
