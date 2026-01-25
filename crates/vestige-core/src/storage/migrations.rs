//! Database Migrations
//!
//! Schema migration definitions for the storage layer.

/// Migration definitions
pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        description: "Initial schema with FSRS-6 and embeddings",
        up: MIGRATION_V1_UP,
    },
    Migration {
        version: 2,
        description: "Add temporal columns",
        up: MIGRATION_V2_UP,
    },
    Migration {
        version: 3,
        description: "Add persistence tables for neuroscience features",
        up: MIGRATION_V3_UP,
    },
    Migration {
        version: 4,
        description: "GOD TIER 2026: Temporal knowledge graph, memory scopes, embedding versioning",
        up: MIGRATION_V4_UP,
    },
];

/// A database migration
#[derive(Debug, Clone)]
pub struct Migration {
    /// Version number
    pub version: u32,
    /// Description
    pub description: &'static str,
    /// SQL to apply
    pub up: &'static str,
}

/// V1: Initial schema
const MIGRATION_V1_UP: &str = r#"
CREATE TABLE IF NOT EXISTS knowledge_nodes (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    node_type TEXT NOT NULL DEFAULT 'fact',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_accessed TEXT NOT NULL,

    -- FSRS-6 state (21 parameters)
    stability REAL DEFAULT 1.0,
    difficulty REAL DEFAULT 5.0,
    reps INTEGER DEFAULT 0,
    lapses INTEGER DEFAULT 0,
    learning_state TEXT DEFAULT 'new',

    -- Dual-strength model (Bjork & Bjork 1992)
    storage_strength REAL DEFAULT 1.0,
    retrieval_strength REAL DEFAULT 1.0,
    retention_strength REAL DEFAULT 1.0,

    -- Sentiment for emotional memory weighting
    sentiment_score REAL DEFAULT 0.0,
    sentiment_magnitude REAL DEFAULT 0.0,

    -- Scheduling
    next_review TEXT,
    scheduled_days INTEGER DEFAULT 0,

    -- Provenance
    source TEXT,
    tags TEXT DEFAULT '[]',

    -- Embedding metadata
    has_embedding INTEGER DEFAULT 0,
    embedding_model TEXT
);

CREATE INDEX IF NOT EXISTS idx_nodes_retention ON knowledge_nodes(retention_strength);
CREATE INDEX IF NOT EXISTS idx_nodes_next_review ON knowledge_nodes(next_review);
CREATE INDEX IF NOT EXISTS idx_nodes_created ON knowledge_nodes(created_at);
CREATE INDEX IF NOT EXISTS idx_nodes_has_embedding ON knowledge_nodes(has_embedding);

-- Embeddings storage table (binary blob for efficiency)
CREATE TABLE IF NOT EXISTS node_embeddings (
    node_id TEXT PRIMARY KEY REFERENCES knowledge_nodes(id) ON DELETE CASCADE,
    embedding BLOB NOT NULL,
    dimensions INTEGER NOT NULL DEFAULT 768,
    model TEXT NOT NULL DEFAULT 'BAAI/bge-base-en-v1.5',
    created_at TEXT NOT NULL
);

-- FTS5 virtual table for full-text search
CREATE VIRTUAL TABLE IF NOT EXISTS knowledge_fts USING fts5(
    id,
    content,
    tags,
    content='knowledge_nodes',
    content_rowid='rowid'
);

-- Triggers to keep FTS in sync
CREATE TRIGGER IF NOT EXISTS knowledge_ai AFTER INSERT ON knowledge_nodes BEGIN
    INSERT INTO knowledge_fts(rowid, id, content, tags)
    VALUES (NEW.rowid, NEW.id, NEW.content, NEW.tags);
END;

CREATE TRIGGER IF NOT EXISTS knowledge_ad AFTER DELETE ON knowledge_nodes BEGIN
    INSERT INTO knowledge_fts(knowledge_fts, rowid, id, content, tags)
    VALUES ('delete', OLD.rowid, OLD.id, OLD.content, OLD.tags);
END;

CREATE TRIGGER IF NOT EXISTS knowledge_au AFTER UPDATE ON knowledge_nodes BEGIN
    INSERT INTO knowledge_fts(knowledge_fts, rowid, id, content, tags)
    VALUES ('delete', OLD.rowid, OLD.id, OLD.content, OLD.tags);
    INSERT INTO knowledge_fts(rowid, id, content, tags)
    VALUES (NEW.rowid, NEW.id, NEW.content, NEW.tags);
END;

-- Schema version tracking
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);

INSERT OR IGNORE INTO schema_version (version, applied_at) VALUES (1, datetime('now'));
"#;

/// V2: Add temporal columns
const MIGRATION_V2_UP: &str = r#"
ALTER TABLE knowledge_nodes ADD COLUMN valid_from TEXT;
ALTER TABLE knowledge_nodes ADD COLUMN valid_until TEXT;

CREATE INDEX IF NOT EXISTS idx_nodes_valid_from ON knowledge_nodes(valid_from);
CREATE INDEX IF NOT EXISTS idx_nodes_valid_until ON knowledge_nodes(valid_until);

UPDATE schema_version SET version = 2, applied_at = datetime('now');
"#;

/// V3: Add persistence tables for neuroscience features
/// Fixes critical gap: intentions, insights, and activation network were IN-MEMORY ONLY
const MIGRATION_V3_UP: &str = r#"
-- 1. INTENTIONS TABLE (Prospective Memory)
-- Stores future intentions/reminders with trigger conditions
CREATE TABLE IF NOT EXISTS intentions (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    trigger_type TEXT NOT NULL,  -- 'time', 'duration', 'event', 'context', 'activity', 'recurring', 'compound'
    trigger_data TEXT NOT NULL,  -- JSON: serialized IntentionTrigger
    priority INTEGER NOT NULL DEFAULT 2,  -- 1=Low, 2=Normal, 3=High, 4=Critical
    status TEXT NOT NULL DEFAULT 'active',  -- 'active', 'triggered', 'fulfilled', 'cancelled', 'expired', 'snoozed'
    created_at TEXT NOT NULL,
    deadline TEXT,
    fulfilled_at TEXT,
    reminder_count INTEGER DEFAULT 0,
    last_reminded_at TEXT,
    notes TEXT,
    tags TEXT DEFAULT '[]',
    related_memories TEXT DEFAULT '[]',
    snoozed_until TEXT,
    source_type TEXT NOT NULL DEFAULT 'api',
    source_data TEXT
);

CREATE INDEX IF NOT EXISTS idx_intentions_status ON intentions(status);
CREATE INDEX IF NOT EXISTS idx_intentions_priority ON intentions(priority);
CREATE INDEX IF NOT EXISTS idx_intentions_deadline ON intentions(deadline);
CREATE INDEX IF NOT EXISTS idx_intentions_snoozed ON intentions(snoozed_until);

-- 2. INSIGHTS TABLE (From Consolidation/Dreams)
-- Stores AI-generated insights discovered during memory consolidation
CREATE TABLE IF NOT EXISTS insights (
    id TEXT PRIMARY KEY,
    insight TEXT NOT NULL,
    source_memories TEXT NOT NULL,  -- JSON array of memory IDs
    confidence REAL NOT NULL,
    novelty_score REAL NOT NULL,
    insight_type TEXT NOT NULL,  -- 'hidden_connection', 'recurring_pattern', 'generalization', 'contradiction', 'knowledge_gap', 'temporal_trend', 'synthesis'
    generated_at TEXT NOT NULL,
    tags TEXT DEFAULT '[]',
    feedback TEXT,  -- 'accepted', 'rejected', or NULL
    applied_count INTEGER DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_insights_type ON insights(insight_type);
CREATE INDEX IF NOT EXISTS idx_insights_confidence ON insights(confidence);
CREATE INDEX IF NOT EXISTS idx_insights_generated ON insights(generated_at);
CREATE INDEX IF NOT EXISTS idx_insights_feedback ON insights(feedback);

-- 3. MEMORY_CONNECTIONS TABLE (Activation Network Edges)
-- Stores associations between memories for spreading activation
CREATE TABLE IF NOT EXISTS memory_connections (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    strength REAL NOT NULL,
    link_type TEXT NOT NULL,  -- 'semantic', 'temporal', 'spatial', 'causal', 'part_of', 'user_defined', 'cross_reference', 'sequential', 'shared_concepts', 'pattern'
    created_at TEXT NOT NULL,
    last_activated TEXT NOT NULL,
    activation_count INTEGER DEFAULT 0,
    PRIMARY KEY (source_id, target_id),
    FOREIGN KEY (source_id) REFERENCES knowledge_nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (target_id) REFERENCES knowledge_nodes(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_connections_source ON memory_connections(source_id);
CREATE INDEX IF NOT EXISTS idx_connections_target ON memory_connections(target_id);
CREATE INDEX IF NOT EXISTS idx_connections_strength ON memory_connections(strength);
CREATE INDEX IF NOT EXISTS idx_connections_type ON memory_connections(link_type);

-- 4. MEMORY_STATES TABLE (Accessibility States)
-- Tracks lifecycle state of each memory (Active/Dormant/Silent/Unavailable)
CREATE TABLE IF NOT EXISTS memory_states (
    memory_id TEXT PRIMARY KEY,
    state TEXT NOT NULL DEFAULT 'active',  -- 'active', 'dormant', 'silent', 'unavailable'
    last_access TEXT NOT NULL,
    access_count INTEGER DEFAULT 1,
    state_entered_at TEXT NOT NULL,
    suppression_until TEXT,
    suppressed_by TEXT DEFAULT '[]',
    time_active_seconds INTEGER DEFAULT 0,
    time_dormant_seconds INTEGER DEFAULT 0,
    time_silent_seconds INTEGER DEFAULT 0,
    time_unavailable_seconds INTEGER DEFAULT 0,
    FOREIGN KEY (memory_id) REFERENCES knowledge_nodes(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_states_state ON memory_states(state);
CREATE INDEX IF NOT EXISTS idx_states_access ON memory_states(last_access);
CREATE INDEX IF NOT EXISTS idx_states_suppression ON memory_states(suppression_until);

-- 5. FSRS_CARDS TABLE (Extended Review State)
-- Stores complete FSRS-6 card state for spaced repetition
CREATE TABLE IF NOT EXISTS fsrs_cards (
    memory_id TEXT PRIMARY KEY,
    difficulty REAL NOT NULL DEFAULT 5.0,
    stability REAL NOT NULL DEFAULT 1.0,
    state TEXT NOT NULL DEFAULT 'new',  -- 'new', 'learning', 'review', 'relearning'
    reps INTEGER DEFAULT 0,
    lapses INTEGER DEFAULT 0,
    last_review TEXT,
    due_date TEXT,
    elapsed_days INTEGER DEFAULT 0,
    scheduled_days INTEGER DEFAULT 0,
    FOREIGN KEY (memory_id) REFERENCES knowledge_nodes(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_fsrs_due ON fsrs_cards(due_date);
CREATE INDEX IF NOT EXISTS idx_fsrs_state ON fsrs_cards(state);

-- 6. CONSOLIDATION_HISTORY TABLE (Dream Cycle Records)
-- Tracks when consolidation ran and what it accomplished
CREATE TABLE IF NOT EXISTS consolidation_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    completed_at TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    memories_replayed INTEGER DEFAULT 0,
    connections_found INTEGER DEFAULT 0,
    connections_strengthened INTEGER DEFAULT 0,
    connections_pruned INTEGER DEFAULT 0,
    insights_generated INTEGER DEFAULT 0,
    memories_transferred TEXT DEFAULT '[]',
    patterns_discovered TEXT DEFAULT '[]'
);

CREATE INDEX IF NOT EXISTS idx_consolidation_completed ON consolidation_history(completed_at);

-- 7. STATE_TRANSITIONS TABLE (Audit Trail)
-- Historical record of state changes for debugging and analytics
CREATE TABLE IF NOT EXISTS state_transitions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    memory_id TEXT NOT NULL,
    from_state TEXT NOT NULL,
    to_state TEXT NOT NULL,
    reason_type TEXT NOT NULL,  -- 'access', 'time_decay', 'cue_reactivation', 'competition_loss', 'interference_resolved', 'user_suppression', 'suppression_expired', 'manual_override', 'system_init'
    reason_data TEXT,
    timestamp TEXT NOT NULL,
    FOREIGN KEY (memory_id) REFERENCES knowledge_nodes(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_transitions_memory ON state_transitions(memory_id);
CREATE INDEX IF NOT EXISTS idx_transitions_timestamp ON state_transitions(timestamp);

UPDATE schema_version SET version = 3, applied_at = datetime('now');
"#;

/// V4: GOD TIER 2026 - Temporal Knowledge Graph, Memory Scopes, Embedding Versioning
/// Competes with Zep's Graphiti and Mem0's memory scopes
const MIGRATION_V4_UP: &str = r#"
-- ============================================================================
-- TEMPORAL KNOWLEDGE GRAPH (Like Zep's Graphiti)
-- ============================================================================

-- Knowledge edges for temporal reasoning
CREATE TABLE IF NOT EXISTS knowledge_edges (
    id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    edge_type TEXT NOT NULL,  -- 'semantic', 'temporal', 'causal', 'derived', 'contradiction', 'refinement'
    weight REAL NOT NULL DEFAULT 1.0,
    -- Temporal validity (bi-temporal model)
    valid_from TEXT,  -- When this relationship started being true
    valid_until TEXT, -- When this relationship stopped being true (NULL = still valid)
    -- Provenance
    created_at TEXT NOT NULL,
    created_by TEXT,  -- 'user', 'system', 'consolidation', 'llm'
    confidence REAL NOT NULL DEFAULT 1.0,  -- Confidence in this edge
    -- Metadata
    metadata TEXT,  -- JSON for edge-specific data
    FOREIGN KEY (source_id) REFERENCES knowledge_nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (target_id) REFERENCES knowledge_nodes(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_edges_source ON knowledge_edges(source_id);
CREATE INDEX IF NOT EXISTS idx_edges_target ON knowledge_edges(target_id);
CREATE INDEX IF NOT EXISTS idx_edges_type ON knowledge_edges(edge_type);
CREATE INDEX IF NOT EXISTS idx_edges_valid_from ON knowledge_edges(valid_from);
CREATE INDEX IF NOT EXISTS idx_edges_valid_until ON knowledge_edges(valid_until);

-- ============================================================================
-- MEMORY SCOPES (Like Mem0's User/Session/Agent)
-- ============================================================================

-- Add scope column to knowledge_nodes
ALTER TABLE knowledge_nodes ADD COLUMN scope TEXT DEFAULT 'user';
-- Values: 'session' (per-session, cleared on restart)
--         'user' (per-user, persists across sessions)
--         'agent' (global agent knowledge, shared)

CREATE INDEX IF NOT EXISTS idx_nodes_scope ON knowledge_nodes(scope);

-- Session tracking table
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL DEFAULT 'default',
    started_at TEXT NOT NULL,
    ended_at TEXT,
    context TEXT,  -- JSON: session metadata
    memory_count INTEGER DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_started ON sessions(started_at);

-- ============================================================================
-- EMBEDDING VERSIONING (Track model upgrades)
-- ============================================================================

-- Add embedding version to node_embeddings
ALTER TABLE node_embeddings ADD COLUMN version INTEGER DEFAULT 1;
-- Version 1 = all-MiniLM-L6-v2 (384d, pre-2026)
-- Version 2 = BGE-base-en-v1.5 (768d, GOD TIER 2026)

CREATE INDEX IF NOT EXISTS idx_embeddings_version ON node_embeddings(version);

-- Update existing embeddings to mark as version 1 (old model)
UPDATE node_embeddings SET version = 1 WHERE version IS NULL;

-- ============================================================================
-- MEMORY COMPRESSION (For old memories - Tier 3 prep)
-- ============================================================================

CREATE TABLE IF NOT EXISTS compressed_memories (
    id TEXT PRIMARY KEY,
    original_id TEXT NOT NULL,
    compressed_content TEXT NOT NULL,
    original_length INTEGER NOT NULL,
    compressed_length INTEGER NOT NULL,
    compression_ratio REAL NOT NULL,
    semantic_fidelity REAL NOT NULL,  -- How much meaning was preserved (0-1)
    compressed_at TEXT NOT NULL,
    model_used TEXT NOT NULL DEFAULT 'llm',
    FOREIGN KEY (original_id) REFERENCES knowledge_nodes(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_compressed_original ON compressed_memories(original_id);
CREATE INDEX IF NOT EXISTS idx_compressed_at ON compressed_memories(compressed_at);

-- ============================================================================
-- EPISODIC vs SEMANTIC MEMORY (Research-backed distinction)
-- ============================================================================

-- Add memory system classification
ALTER TABLE knowledge_nodes ADD COLUMN memory_system TEXT DEFAULT 'semantic';
-- Values: 'episodic' (what happened - events, conversations)
--         'semantic' (what I know - facts, concepts)
--         'procedural' (how-to - never decays)

CREATE INDEX IF NOT EXISTS idx_nodes_memory_system ON knowledge_nodes(memory_system);

UPDATE schema_version SET version = 4, applied_at = datetime('now');
"#;

/// Get current schema version from database
pub fn get_current_version(conn: &rusqlite::Connection) -> rusqlite::Result<u32> {
    conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |row| row.get(0),
    )
    .or(Ok(0))
}

/// Apply pending migrations
pub fn apply_migrations(conn: &rusqlite::Connection) -> rusqlite::Result<u32> {
    let current_version = get_current_version(conn)?;
    let mut applied = 0;

    for migration in MIGRATIONS {
        if migration.version > current_version {
            tracing::info!(
                "Applying migration v{}: {}",
                migration.version,
                migration.description
            );

            // Use execute_batch to handle multi-statement SQL including triggers
            conn.execute_batch(migration.up)?;

            applied += 1;
        }
    }

    Ok(applied)
}
