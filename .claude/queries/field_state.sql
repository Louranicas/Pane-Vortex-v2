-- Field state queries for pane-vortex-v2

-- Latest N snapshots
SELECT tick, ROUND(r,3) as r, sphere_count, decision_action, ROUND(k_mod,3) as k_mod
FROM field_snapshots ORDER BY tick DESC LIMIT ?;

-- R trend over last hour (720 ticks at 5s)
SELECT MIN(r) as r_min, MAX(r) as r_max, AVG(r) as r_avg, COUNT(*) as samples
FROM field_snapshots WHERE tick > (SELECT MAX(tick) - 720 FROM field_snapshots);

-- Chimera events
SELECT tick, chimera_cluster_count, decision_action
FROM field_snapshots WHERE chimera_detected = 1 ORDER BY tick DESC LIMIT 20;

-- Sphere lifecycle
SELECT sphere_id, event_type, tick, status, persona
FROM sphere_history ORDER BY tick DESC LIMIT 30;

-- Most active spheres (by event count)
SELECT sphere_id, COUNT(*) as events, MIN(tick) as first_seen, MAX(tick) as last_seen
FROM sphere_history GROUP BY sphere_id ORDER BY events DESC LIMIT 15;

-- Decision distribution
SELECT decision_action, COUNT(*) as count
FROM field_snapshots GROUP BY decision_action ORDER BY count DESC;

-- Modulation breakdown (requires JSON1 extension)
SELECT tick, json_extract(modulation_breakdown, '$.synthex') as synthex,
       json_extract(modulation_breakdown, '$.nexus') as nexus,
       json_extract(modulation_breakdown, '$.me') as me,
       json_extract(modulation_breakdown, '$.conductor') as conductor
FROM field_snapshots WHERE modulation_breakdown IS NOT NULL
ORDER BY tick DESC LIMIT 10;
