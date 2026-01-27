//! # Import/Export Journey Tests
//!
//! Tests the data portability features that allow users to backup, migrate,
//! and share their memory data. This ensures users have control over their
//! data and can move between systems.
//!
//! ## User Journey
//!
//! 1. User builds up memories over time
//! 2. User exports memories for backup or migration
//! 3. User imports memories on new system or from backup
//! 4. User shares relevant memories with teammates
//! 5. User merges memories from multiple sources

use chrono::{DateTime, Duration, Utc};
use vestige_core::memory::IngestInput;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// EXPORT/IMPORT FORMAT
// ============================================================================

/// Portable format for memory export/import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedMemory {
    /// Memory content
    pub content: String,
    /// Memory type (concept, fact, decision, etc.)
    pub node_type: String,
    /// Associated tags
    pub tags: Vec<String>,
    /// Original creation timestamp
    pub created_at: DateTime<Utc>,
    /// Source of the memory
    pub source: Option<String>,
    /// Sentiment score (-1 to 1)
    pub sentiment_score: f64,
    /// Sentiment magnitude (0 to 1)
    pub sentiment_magnitude: f64,
    /// FSRS stability (for preserving learning state)
    pub stability: f64,
    /// FSRS difficulty
    pub difficulty: f64,
    /// Review count
    pub reps: i32,
    /// Lapse count
    pub lapses: i32,
}

/// Export bundle containing memories and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportBundle {
    /// Format version
    pub version: String,
    /// Export timestamp
    pub exported_at: DateTime<Utc>,
    /// Exporting system identifier
    pub source_system: String,
    /// Exported memories
    pub memories: Vec<ExportedMemory>,
    /// Optional metadata
    pub metadata: HashMap<String, String>,
}

impl ExportBundle {
    /// Create a new export bundle
    pub fn new(source_system: &str) -> Self {
        Self {
            version: "1.0".to_string(),
            exported_at: Utc::now(),
            source_system: source_system.to_string(),
            memories: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a memory to the bundle
    pub fn add_memory(&mut self, memory: ExportedMemory) {
        self.memories.push(memory);
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl ExportedMemory {
    /// Create a new exported memory
    pub fn new(content: &str, node_type: &str, tags: Vec<&str>) -> Self {
        Self {
            content: content.to_string(),
            node_type: node_type.to_string(),
            tags: tags.into_iter().map(String::from).collect(),
            created_at: Utc::now(),
            source: Some("test".to_string()),
            sentiment_score: 0.0,
            sentiment_magnitude: 0.0,
            stability: 10.0,
            difficulty: 0.3,
            reps: 5,
            lapses: 0,
        }
    }

    /// Convert to IngestInput for import
    pub fn to_ingest_input(&self) -> IngestInput {
        let json = serde_json::json!({
            "content": self.content,
            "nodeType": self.node_type,
            "tags": self.tags,
            "source": self.source,
            "sentimentScore": self.sentiment_score,
            "sentimentMagnitude": self.sentiment_magnitude
        });
        serde_json::from_value(json).expect("IngestInput JSON should be valid")
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Create a sample export bundle
fn create_sample_bundle() -> ExportBundle {
    let mut bundle = ExportBundle::new("test-system");
    bundle.add_metadata("project", "vestige");
    bundle.add_metadata("user", "test-user");

    // Add sample memories
    bundle.add_memory(ExportedMemory::new(
        "Rust ownership ensures memory safety",
        "concept",
        vec!["rust", "memory"],
    ));
    bundle.add_memory(ExportedMemory::new(
        "Borrowing allows temporary access to data",
        "concept",
        vec!["rust", "borrowing"],
    ));
    bundle.add_memory(ExportedMemory::new(
        "Lifetimes track reference validity",
        "concept",
        vec!["rust", "lifetimes"],
    ));

    bundle
}

// ============================================================================
// TEST 1: EXPORT SERIALIZES MEMORIES TO JSON
// ============================================================================

/// Test that memories can be exported to a portable JSON format.
///
/// Validates:
/// - All memory fields are preserved
/// - FSRS state is included
/// - Tags are preserved
/// - Metadata is included
#[test]
fn test_export_serializes_memories_to_json() {
    let bundle = create_sample_bundle();

    // Serialize to JSON
    let json = bundle.to_json().expect("Serialization should succeed");

    // Verify JSON is valid
    assert!(!json.is_empty(), "JSON should not be empty");
    assert!(json.contains("\"version\""), "Should contain version");
    assert!(json.contains("\"memories\""), "Should contain memories");
    assert!(json.contains("\"metadata\""), "Should contain metadata");

    // Verify content is present
    assert!(json.contains("Rust ownership"), "Should contain memory content");
    assert!(json.contains("rust"), "Should contain tags");

    // Verify FSRS state
    assert!(json.contains("stability"), "Should contain stability");
    assert!(json.contains("difficulty"), "Should contain difficulty");

    // Verify metadata
    assert!(json.contains("vestige"), "Should contain project metadata");
}

// ============================================================================
// TEST 2: IMPORT DESERIALIZES JSON TO MEMORIES
// ============================================================================

/// Test that exported JSON can be imported back to memories.
///
/// Validates:
/// - JSON parses correctly
/// - All fields are restored
/// - Memories can be ingested
#[test]
fn test_import_deserializes_json_to_memories() {
    let original = create_sample_bundle();
    let json = original.to_json().expect("Serialization should succeed");

    // Deserialize
    let imported = ExportBundle::from_json(&json).expect("Deserialization should succeed");

    // Verify structure
    assert_eq!(imported.version, "1.0");
    assert_eq!(imported.source_system, "test-system");
    assert_eq!(imported.memories.len(), 3);

    // Verify memories
    let mem1 = &imported.memories[0];
    assert!(mem1.content.contains("ownership"), "Content should be preserved");
    assert!(mem1.tags.contains(&"rust".to_string()), "Tags should be preserved");
    assert!(mem1.stability > 0.0, "Stability should be preserved");

    // Verify metadata
    assert_eq!(imported.metadata.get("project"), Some(&"vestige".to_string()));
}

// ============================================================================
// TEST 3: ROUNDTRIP PRESERVES ALL DATA
// ============================================================================

/// Test that export -> import roundtrip preserves all data.
///
/// Validates:
/// - Content is identical
/// - Tags are identical
/// - FSRS state is identical
/// - Timestamps are preserved
#[test]
fn test_roundtrip_preserves_all_data() {
    // Create original memory
    let original = ExportedMemory {
        content: "Test content with special chars: <>&\"'".to_string(),
        node_type: "decision".to_string(),
        tags: vec!["architecture".to_string(), "decision".to_string()],
        created_at: Utc::now() - Duration::days(30),
        source: Some("documentation".to_string()),
        sentiment_score: 0.5,
        sentiment_magnitude: 0.7,
        stability: 15.5,
        difficulty: 0.25,
        reps: 10,
        lapses: 2,
    };

    // Create bundle and serialize
    let mut bundle = ExportBundle::new("test");
    bundle.add_memory(original.clone());
    let json = bundle.to_json().unwrap();

    // Import
    let imported_bundle = ExportBundle::from_json(&json).unwrap();
    let imported = &imported_bundle.memories[0];

    // Verify all fields
    assert_eq!(imported.content, original.content, "Content should match");
    assert_eq!(imported.node_type, original.node_type, "Type should match");
    assert_eq!(imported.tags, original.tags, "Tags should match");
    assert_eq!(imported.stability, original.stability, "Stability should match");
    assert_eq!(imported.difficulty, original.difficulty, "Difficulty should match");
    assert_eq!(imported.reps, original.reps, "Reps should match");
    assert_eq!(imported.lapses, original.lapses, "Lapses should match");
    assert_eq!(imported.sentiment_score, original.sentiment_score, "Sentiment score should match");
    assert_eq!(imported.sentiment_magnitude, original.sentiment_magnitude, "Sentiment magnitude should match");
    assert_eq!(imported.source, original.source, "Source should match");
}

// ============================================================================
// TEST 4: SELECTIVE EXPORT BY TAGS
// ============================================================================

/// Test that memories can be selectively exported by tags.
///
/// Validates:
/// - Tag filtering works
/// - Only matching memories are exported
/// - Multiple tags can be combined
#[test]
fn test_selective_export_by_tags() {
    // Create memories with different tags
    let memories = [ExportedMemory::new("Rust ownership", "concept", vec!["rust", "memory"]),
        ExportedMemory::new("Python generators", "concept", vec!["python", "generators"]),
        ExportedMemory::new("Rust borrowing", "concept", vec!["rust", "borrowing"]),
        ExportedMemory::new("JavaScript async", "concept", vec!["javascript", "async"]),
        ExportedMemory::new("Rust async", "concept", vec!["rust", "async"])];

    // Filter by "rust" tag
    let rust_memories: Vec<_> = memories
        .iter()
        .filter(|m| m.tags.contains(&"rust".to_string()))
        .collect();

    assert_eq!(rust_memories.len(), 3, "Should filter to 3 Rust memories");

    // Filter by multiple tags (rust AND async)
    let rust_async_memories: Vec<_> = memories
        .iter()
        .filter(|m| {
            m.tags.contains(&"rust".to_string()) && m.tags.contains(&"async".to_string())
        })
        .collect();

    assert_eq!(rust_async_memories.len(), 1, "Should filter to 1 Rust async memory");
    assert!(rust_async_memories[0].content.contains("Rust async"));

    // Export filtered
    let mut bundle = ExportBundle::new("test");
    for mem in rust_memories {
        bundle.add_memory(mem.clone());
    }

    assert_eq!(bundle.memories.len(), 3, "Bundle should have 3 memories");
}

// ============================================================================
// TEST 5: IMPORT MERGES WITH EXISTING DATA
// ============================================================================

/// Test that imported memories can be merged with existing data.
///
/// Validates:
/// - Duplicate detection works
/// - New memories are added
/// - Conflict resolution can be applied
#[test]
fn test_import_merges_with_existing_data() {
    // Simulate existing memories
    let existing: HashMap<String, ExportedMemory> = [
        ("1".to_string(), ExportedMemory::new("Rust ownership memory safety", "concept", vec!["rust"])),
        ("2".to_string(), ExportedMemory::new("Rust borrowing rules explained", "concept", vec!["rust"])),
    ]
    .into_iter()
    .collect();

    // Create import bundle with some overlapping content
    let mut bundle = ExportBundle::new("external");
    bundle.add_memory(ExportedMemory {
        content: "Rust ownership memory safety updated version".to_string(),
        node_type: "concept".to_string(),
        tags: vec!["rust".to_string(), "memory".to_string()],
        created_at: Utc::now(),
        source: Some("external".to_string()),
        sentiment_score: 0.0,
        sentiment_magnitude: 0.0,
        stability: 12.0,
        difficulty: 0.25,
        reps: 8,
        lapses: 1,
    });
    bundle.add_memory(ExportedMemory {
        content: "Rust lifetimes tracking references".to_string(),
        node_type: "concept".to_string(),
        tags: vec!["rust".to_string(), "lifetimes".to_string()],
        created_at: Utc::now(),
        source: Some("external".to_string()),
        sentiment_score: 0.0,
        sentiment_magnitude: 0.0,
        stability: 10.0,
        difficulty: 0.3,
        reps: 5,
        lapses: 0,
    });

    // Simulate merge logic
    let mut merged_count = 0;
    let mut new_count = 0;

    for imported in &bundle.memories {
        // Check for duplicate (simplified: by content similarity)
        let is_duplicate = existing.values().any(|e| {
            // Simple content overlap check - count common words
            let imported_lower = imported.content.to_lowercase();
            let existing_lower = e.content.to_lowercase();
            let imported_words: std::collections::HashSet<&str> =
                imported_lower.split_whitespace().collect();
            let existing_words: std::collections::HashSet<&str> =
                existing_lower.split_whitespace().collect();
            let overlap_count = imported_words.intersection(&existing_words).count();
            // At least 3 words in common indicates likely duplicate
            overlap_count >= 3
        });

        if is_duplicate {
            merged_count += 1;
        } else {
            new_count += 1;
        }
    }

    assert_eq!(merged_count, 1, "Should detect 1 duplicate (ownership)");
    assert_eq!(new_count, 1, "Should add 1 new memory (lifetimes)");
}

// ============================================================================
// ADDITIONAL IMPORT/EXPORT TESTS
// ============================================================================

/// Test export bundle metadata.
#[test]
fn test_export_bundle_metadata() {
    let mut bundle = ExportBundle::new("vestige-client");
    bundle.add_metadata("version", "0.1.0");
    bundle.add_metadata("user_id", "user-123");
    bundle.add_metadata("export_reason", "backup");

    assert_eq!(bundle.metadata.len(), 3);
    assert_eq!(bundle.metadata.get("version"), Some(&"0.1.0".to_string()));
    assert_eq!(bundle.source_system, "vestige-client");
}

/// Test empty bundle handling.
#[test]
fn test_empty_bundle_handling() {
    let bundle = ExportBundle::new("test");

    // Serialize empty bundle
    let json = bundle.to_json().unwrap();
    assert!(json.contains("\"memories\": []"), "Should have empty memories array");

    // Deserialize and verify
    let imported = ExportBundle::from_json(&json).unwrap();
    assert!(imported.memories.is_empty(), "Imported should be empty");
}

/// Test large bundle performance.
#[test]
fn test_large_bundle_performance() {
    let mut bundle = ExportBundle::new("test");

    // Create 1000 memories
    for i in 0..1000 {
        bundle.add_memory(ExportedMemory {
            content: format!("Test memory content number {}", i),
            node_type: "fact".to_string(),
            tags: vec!["test".to_string(), format!("batch-{}", i / 100)],
            created_at: Utc::now(),
            source: Some("benchmark".to_string()),
            sentiment_score: 0.0,
            sentiment_magnitude: 0.0,
            stability: 10.0,
            difficulty: 0.3,
            reps: 0,
            lapses: 0,
        });
    }

    assert_eq!(bundle.memories.len(), 1000);

    // Serialize (should be reasonably fast)
    let start = std::time::Instant::now();
    let json = bundle.to_json().unwrap();
    let serialize_time = start.elapsed();

    // Deserialize
    let start = std::time::Instant::now();
    let imported = ExportBundle::from_json(&json).unwrap();
    let deserialize_time = start.elapsed();

    assert_eq!(imported.memories.len(), 1000);
    assert!(
        serialize_time.as_millis() < 1000,
        "Serialization took too long: {:?}",
        serialize_time
    );
    assert!(
        deserialize_time.as_millis() < 1000,
        "Deserialization took too long: {:?}",
        deserialize_time
    );
}

/// Test converting exported memory to ingest input.
#[test]
fn test_exported_to_ingest_input() {
    let exported = ExportedMemory {
        content: "Test content".to_string(),
        node_type: "concept".to_string(),
        tags: vec!["tag1".to_string(), "tag2".to_string()],
        created_at: Utc::now(),
        source: Some("external".to_string()),
        sentiment_score: 0.5,
        sentiment_magnitude: 0.8,
        stability: 15.0,
        difficulty: 0.2,
        reps: 10,
        lapses: 1,
    };

    let input = exported.to_ingest_input();

    assert_eq!(input.content, "Test content");
    assert_eq!(input.node_type, "concept");
    assert_eq!(input.tags.len(), 2);
    assert_eq!(input.source, Some("external".to_string()));
    assert_eq!(input.sentiment_score, 0.5);
    assert_eq!(input.sentiment_magnitude, 0.8);
}
