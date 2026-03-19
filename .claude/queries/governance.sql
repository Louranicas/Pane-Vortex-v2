-- Governance queries for pane-vortex-v2 (V3.4)

-- Active proposals
SELECT id, proposer_sphere, parameter, current_value, proposed_value,
       votes_for, votes_against, votes_abstain, status
FROM proposals WHERE status = 'open' ORDER BY created_at DESC;

-- Proposal history
SELECT id, parameter, proposed_value, status, votes_for, votes_against
FROM proposals ORDER BY created_at DESC LIMIT 20;

-- Consent declarations
SELECT sphere_id, accept_external_modulation, max_k_adjustment,
       accept_cascade, accept_observation, updated_at
FROM consent_declarations ORDER BY updated_at DESC;

-- Spheres that opted out of external modulation
SELECT sphere_id, max_k_adjustment, updated_at
FROM consent_declarations WHERE accept_external_modulation = 0;

-- Data manifest for a specific sphere
SELECT system, record_count, last_scanned_at
FROM data_manifests WHERE sphere_id = ? ORDER BY system;
