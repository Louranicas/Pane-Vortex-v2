# Session 049 — Layer Dimensions

> **8 layers, 46 files, 31,859 LOC, 1,471 tests**
> **Captured:** 2026-03-21 (3 parallel subagent analysis)

---

## Layer Breakdown

| Layer | Name | Modules | Files | LOC | Tests | Tests/LOC | LOC/File |
|-------|------|---------|-------|-----|-------|-----------|----------|
| L1 | Foundation | m01-m06 | 6 | 3,416 | 183 | 5.4% | 569 |
| L2 | Services | m07-m10 | 4 | 4,638 | 96 | 2.1% | 1,160 |
| L3 | Field | m11-m15 | 5 | 4,188 | 212 | 5.1% | 838 |
| L4 | Coupling | m16-m18 | 4 | 1,741 | 94 | 5.4% | 435 |
| L5 | Learning | m19-m21 | 4 | 1,441 | 77 | 5.3% | 360 |
| L6 | Bridges | m22-m28 | 8 | 6,792 | 390 | 5.7% | 849 |
| L7 | Coordination | m29-m36 | 9 | 7,539 | 311 | 4.1% | 838 |
| L8 | Governance | m37-m41 | 6 | 2,104 | 108 | 5.1% | 351 |
| **Total** | | **m01-m41** | **46** | **31,859** | **1,471** | **4.6%** | **693** |

## By Size (LOC descending)

```
L7 Coordination ████████████████████████████████████████  7,539 (23.7%)
L6 Bridges      ██████████████████████████████████████    6,792 (21.3%)
L2 Services     ████████████████████████                  4,638 (14.6%)
L3 Field        ██████████████████████                    4,188 (13.1%)
L1 Foundation   ██████████████████                        3,416 (10.7%)
L8 Governance   ███████████                               2,104  (6.6%)
L4 Coupling     █████████                                 1,741  (5.5%)
L5 Learning     ████████                                  1,441  (4.5%)
```

## By Test Count (descending)

```
L6 Bridges      ████████████████████████████████████████  390 (26.5%)
L7 Coordination ████████████████████████████████          311 (21.1%)
L3 Field        ██████████████████████████                212 (14.4%)
L1 Foundation   ███████████████████████                   183 (12.4%)
L8 Governance   ██████████████                            108  (7.3%)
L2 Services     ████████████                               96  (6.5%)
L4 Coupling     ████████████                               94  (6.4%)
L5 Learning     ██████████                                 77  (5.2%)
```

## Analysis

### Heaviest Layers
- **L7 Coordination** (7,539 LOC) — IPC bus, conductor, executor, tick orchestrator, persistence. The operational heart.
- **L6 Bridges** (6,792 LOC) — 7 service bridges + consent gate. Most files (8), most tests (390). The integration surface.

### Test Coverage
- **Best tested:** L6 Bridges (5.7% test density, 390 tests) — appropriate given bridges are the highest-risk integration surface
- **Under-tested:** L2 Services (2.1%, 96 tests) — API server, health, lifecycle. Consider adding more endpoint tests
- **All layers meet 50+ test minimum** except possibly per-module (L5 averages 19 tests/module)

### Architecture Balance
- L1-L3 (foundation + field): 12,242 LOC (38%) — the "what"
- L4-L5 (coupling + learning): 3,182 LOC (10%) — the "how" (compact algorithms)
- L6 (bridges): 6,792 LOC (21%) — the "connections"
- L7-L8 (coordination + governance): 9,643 LOC (30%) — the "orchestration"

The coupling/learning core (L4+L5) is intentionally compact at 10% — these are pure algorithms. The integration layer (L6+L7) dominates at 45%, reflecting the reality that coordination is harder than computation.

---

## Cross-References

- [[Session 049 — Master Index]]
- [[ULTRAPLATE Master Index]]
- [[The Habitat — Integrated Master Plan V3]]
