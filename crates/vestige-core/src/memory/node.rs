//! Knowledge Node - The fundamental unit of memory
//!
//! Each node represents a discrete piece of knowledge with:
//! - Content and metadata
//! - FSRS-6 scheduling state
//! - Dual-strength retention model
//! - Temporal validity (bi-temporal)
//! - Embedding metadata

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// NODE TYPES
// ============================================================================

/// Types of knowledge nodes
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    /// A discrete fact or piece of information
    #[default]
    Fact,
    /// A concept or abstract idea
    Concept,
    /// A procedure or how-to knowledge
    Procedure,
    /// An event or experience
    Event,
    /// A relationship between entities
    Relationship,
    /// A quote or verbatim text
    Quote,
    /// Code or technical snippet
    Code,
    /// A question to be answered
    Question,
    /// User insight or reflection
    Insight,
}

impl NodeType {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::Fact => "fact",
            NodeType::Concept => "concept",
            NodeType::Procedure => "procedure",
            NodeType::Event => "event",
            NodeType::Relationship => "relationship",
            NodeType::Quote => "quote",
            NodeType::Code => "code",
            NodeType::Question => "question",
            NodeType::Insight => "insight",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "fact" => NodeType::Fact,
            "concept" => NodeType::Concept,
            "procedure" => NodeType::Procedure,
            "event" => NodeType::Event,
            "relationship" => NodeType::Relationship,
            "quote" => NodeType::Quote,
            "code" => NodeType::Code,
            "question" => NodeType::Question,
            "insight" => NodeType::Insight,
            _ => NodeType::Fact,
        }
    }
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// KNOWLEDGE NODE
// ============================================================================

/// A knowledge node in the memory graph
///
/// Combines multiple memory science models:
/// - FSRS-6 for optimal review scheduling
/// - Bjork dual-strength for realistic forgetting
/// - Temporal validity for time-sensitive knowledge
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeNode {
    /// Unique identifier (UUID v4)
    pub id: String,
    /// The actual content/knowledge
    pub content: String,
    /// Type of knowledge (fact, concept, procedure, etc.)
    pub node_type: String,
    /// When the node was created
    pub created_at: DateTime<Utc>,
    /// When the node was last modified
    pub updated_at: DateTime<Utc>,
    /// When the node was last accessed/reviewed
    pub last_accessed: DateTime<Utc>,

    // ========== FSRS-6 State (21 parameters) ==========
    /// Memory stability (days until 90% forgetting probability)
    pub stability: f64,
    /// Inherent difficulty (1.0 = easy, 10.0 = hard)
    pub difficulty: f64,
    /// Number of successful reviews
    pub reps: i32,
    /// Number of lapses (forgotten after learning)
    pub lapses: i32,

    // ========== Dual-Strength Model (Bjork & Bjork 1992) ==========
    /// Storage strength - accumulated with practice, never decays
    pub storage_strength: f64,
    /// Retrieval strength - current accessibility, decays over time
    pub retrieval_strength: f64,
    /// Combined retention score (0.0 - 1.0)
    pub retention_strength: f64,

    // ========== Emotional Memory ==========
    /// Sentiment polarity (-1.0 to 1.0)
    pub sentiment_score: f64,
    /// Sentiment intensity (0.0 to 1.0) - affects stability
    pub sentiment_magnitude: f64,

    // ========== Scheduling ==========
    /// Next scheduled review date
    pub next_review: Option<DateTime<Utc>>,

    // ========== Provenance ==========
    /// Source of the knowledge (URL, file, conversation, etc.)
    pub source: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,

    // ========== Temporal Memory (Bi-temporal) ==========
    /// When this knowledge became valid
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<DateTime<Utc>>,
    /// When this knowledge stops being valid
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<Utc>>,

    // ========== Semantic Embedding ==========
    /// Whether this node has an embedding vector
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_embedding: Option<bool>,
    /// Which model generated the embedding
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_model: Option<String>,
}

impl Default for KnowledgeNode {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: String::new(),
            content: String::new(),
            node_type: "fact".to_string(),
            created_at: now,
            updated_at: now,
            last_accessed: now,
            stability: 2.5,
            difficulty: 5.0,
            reps: 0,
            lapses: 0,
            storage_strength: 1.0,
            retrieval_strength: 1.0,
            retention_strength: 1.0,
            sentiment_score: 0.0,
            sentiment_magnitude: 0.0,
            next_review: None,
            source: None,
            tags: vec![],
            valid_from: None,
            valid_until: None,
            has_embedding: None,
            embedding_model: None,
        }
    }
}

impl KnowledgeNode {
    /// Create a new knowledge node with the given content
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            ..Default::default()
        }
    }

    /// Check if this node is currently valid (within temporal bounds)
    pub fn is_valid_at(&self, time: DateTime<Utc>) -> bool {
        let after_start = self.valid_from.map(|t| time >= t).unwrap_or(true);
        let before_end = self.valid_until.map(|t| time <= t).unwrap_or(true);
        after_start && before_end
    }

    /// Check if this node is currently valid (now)
    pub fn is_currently_valid(&self) -> bool {
        self.is_valid_at(Utc::now())
    }

    /// Check if this node is due for review
    pub fn is_due(&self) -> bool {
        self.next_review.map(|t| t <= Utc::now()).unwrap_or(true)
    }

    /// Get the parsed node type
    pub fn get_node_type(&self) -> NodeType {
        NodeType::from_str(&self.node_type)
    }
}

// ============================================================================
// INPUT TYPES
// ============================================================================

/// Input for creating a new memory
///
/// Uses `deny_unknown_fields` to prevent field injection attacks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IngestInput {
    /// The content to memorize
    pub content: String,
    /// Type of knowledge (fact, concept, procedure, etc.)
    pub node_type: String,
    /// Source of the knowledge
    pub source: Option<String>,
    /// Sentiment polarity (-1.0 to 1.0)
    #[serde(default)]
    pub sentiment_score: f64,
    /// Sentiment intensity (0.0 to 1.0)
    #[serde(default)]
    pub sentiment_magnitude: f64,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    /// When this knowledge becomes valid
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<DateTime<Utc>>,
    /// When this knowledge stops being valid
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<Utc>>,
}

impl Default for IngestInput {
    fn default() -> Self {
        Self {
            content: String::new(),
            node_type: "fact".to_string(),
            source: None,
            sentiment_score: 0.0,
            sentiment_magnitude: 0.0,
            tags: vec![],
            valid_from: None,
            valid_until: None,
        }
    }
}

/// Search mode for recall queries
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SearchMode {
    /// Keyword search only (FTS5/BM25)
    Keyword,
    /// Semantic search only (embeddings)
    Semantic,
    /// Hybrid search with RRF fusion (default, best results)
    #[default]
    Hybrid,
}

/// Input for recalling memories
///
/// Uses `deny_unknown_fields` to prevent field injection attacks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecallInput {
    /// Search query
    pub query: String,
    /// Maximum results to return
    pub limit: i32,
    /// Minimum retention strength (0.0 to 1.0)
    #[serde(default)]
    pub min_retention: f64,
    /// Search mode (keyword, semantic, or hybrid)
    #[serde(default)]
    pub search_mode: SearchMode,
    /// Only return results valid at this time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_at: Option<DateTime<Utc>>,
}

impl Default for RecallInput {
    fn default() -> Self {
        Self {
            query: String::new(),
            limit: 10,
            min_retention: 0.0,
            search_mode: SearchMode::Hybrid,
            valid_at: None,
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_type_roundtrip() {
        for node_type in [
            NodeType::Fact,
            NodeType::Concept,
            NodeType::Procedure,
            NodeType::Event,
            NodeType::Code,
        ] {
            assert_eq!(NodeType::from_str(node_type.as_str()), node_type);
        }
    }

    #[test]
    fn test_knowledge_node_default() {
        let node = KnowledgeNode::default();
        assert!(node.id.is_empty());
        assert_eq!(node.node_type, "fact");
        assert!(node.is_due());
        assert!(node.is_currently_valid());
    }

    #[test]
    fn test_temporal_validity() {
        let mut node = KnowledgeNode::default();
        let now = Utc::now();

        // No bounds = always valid
        assert!(node.is_valid_at(now));

        // Set future valid_from = not valid now
        node.valid_from = Some(now + chrono::Duration::days(1));
        assert!(!node.is_valid_at(now));

        // Set past valid_from = valid now
        node.valid_from = Some(now - chrono::Duration::days(1));
        assert!(node.is_valid_at(now));

        // Set past valid_until = not valid now
        node.valid_until = Some(now - chrono::Duration::hours(1));
        assert!(!node.is_valid_at(now));
    }

    #[test]
    fn test_ingest_input_deny_unknown_fields() {
        // Valid input should parse
        let json = r#"{"content": "test", "nodeType": "fact", "tags": []}"#;
        let result: Result<IngestInput, _> = serde_json::from_str(json);
        assert!(result.is_ok());

        // Unknown field should fail (security feature)
        let json_with_unknown =
            r#"{"content": "test", "nodeType": "fact", "tags": [], "malicious_field": "attack"}"#;
        let result: Result<IngestInput, _> = serde_json::from_str(json_with_unknown);
        assert!(result.is_err());
    }
}
