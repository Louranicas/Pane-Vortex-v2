-- Bus task queries for pane-vortex-v2

-- Recent tasks by status
SELECT id, status, source_sphere, description, target_type
FROM bus_tasks ORDER BY submitted_at DESC LIMIT 20;

-- Task completion rate
SELECT status, COUNT(*) as count FROM bus_tasks GROUP BY status;

-- Expired tasks (should be 0 if TTL enforcement works)
SELECT id, source_sphere, description,
       ROUND((julianday('now') - julianday(submitted_at)) * 86400) as age_secs
FROM bus_tasks WHERE status = 'expired' ORDER BY submitted_at DESC LIMIT 10;

-- Event type distribution
SELECT event_type, COUNT(*) as count
FROM bus_events GROUP BY event_type ORDER BY count DESC;

-- Cascade flow
SELECT source_sphere, target_sphere, status, depth
FROM cascade_events ORDER BY id DESC LIMIT 20;

-- Active subscriptions
SELECT sphere_id, pattern, created_at
FROM event_subscriptions ORDER BY created_at DESC;
