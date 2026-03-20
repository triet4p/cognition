-- 1. Table for Memory Nodes (The 2-layer Schema)
CREATE TABLE IF NOT EXISTS nodes (
    id TEXT PRIMARY KEY NOT NULL,           -- UUID stored as String
    network_type TEXT NOT NULL,             -- world, experience, etc.
    narrative_fact TEXT NOT NULL,           -- Processed text
    raw_snippet TEXT,                       -- Original lossless source
    embedding BLOB,                         -- Vector data (Optional for persistence)
    confidence REAL NOT NULL DEFAULT 1.0,   -- Cognitive strength
    created_at DATETIME NOT NULL,           -- Temporal index point
    expires_at DATETIME                     -- Critical for Intention network
);

-- 2. Table for Memory Edges (Unified Graph Links)
CREATE TABLE IF NOT EXISTS edges (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    edge_type TEXT NOT NULL,                -- temporal, semantic, causal, etc.
    weight REAL NOT NULL,
    multiplier REAL NOT NULL DEFAULT 1.0,
    PRIMARY KEY (source_id, target_id, edge_type),
    FOREIGN KEY (source_id) REFERENCES nodes (id) ON DELETE CASCADE,
    FOREIGN KEY (target_id) REFERENCES nodes (id) ON DELETE CASCADE
);

-- 3. Performance Indexes
-- Speed up temporal queries (Recall by time)
CREATE INDEX IF NOT EXISTS idx_nodes_created_at ON nodes(created_at);
-- Speed up expiration checks (Intention network cleanup)
CREATE INDEX IF NOT EXISTS idx_nodes_expires_at ON nodes(expires_at);
-- Speed up network-specific filtering
CREATE INDEX IF NOT EXISTS idx_nodes_network ON nodes(network_type);
-- Speed up graph traversal (Finding neighbors)
CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source_id);