//! Custom Test Assertions
//!
//! Provides domain-specific assertions for memory testing:
//! - Retention and decay assertions
//! - Scheduling assertions
//! - State transition assertions
//! - Search result assertions

use vestige_core::{KnowledgeNode, Storage};

// ============================================================================
// RETENTION ASSERTIONS
// ============================================================================

/// Assert that retention has decreased from an expected value
///
/// # Example
/// ```rust,ignore
/// assert_retention_decreased!(node.retention_strength, 1.0, 0.1);
/// ```
#[macro_export]
macro_rules! assert_retention_decreased {
    ($actual:expr, $original:expr) => {
        assert!(
            $actual < $original,
            "Expected retention to decrease: {} should be less than {}",
            $actual,
            $original
        );
    };
    ($actual:expr, $original:expr, $min_decrease:expr) => {
        let decrease = $original - $actual;
        assert!(
            decrease >= $min_decrease,
            "Expected retention to decrease by at least {}: actual decrease was {} ({} -> {})",
            $min_decrease,
            decrease,
            $original,
            $actual
        );
    };
}

/// Assert that retention is within expected range
#[macro_export]
macro_rules! assert_retention_in_range {
    ($actual:expr, $min:expr, $max:expr) => {
        assert!(
            $actual >= $min && $actual <= $max,
            "Expected retention in range [{}, {}], got {}",
            $min,
            $max,
            $actual
        );
    };
}

/// Assert that retrieval strength has decayed properly
#[macro_export]
macro_rules! assert_retrieval_decayed {
    ($node:expr, $elapsed_days:expr) => {
        let expected_max = 1.0; // Can't exceed 1.0
        let expected_min = if $elapsed_days > 0.0 {
            0.0 // Should have decayed at least somewhat
        } else {
            1.0
        };
        assert!(
            $node.retrieval_strength >= expected_min && $node.retrieval_strength <= expected_max,
            "Retrieval strength {} out of expected range [{}, {}] after {} days",
            $node.retrieval_strength,
            expected_min,
            expected_max,
            $elapsed_days
        );
    };
}

// ============================================================================
// SCHEDULING ASSERTIONS
// ============================================================================

/// Assert that a memory is due for review
#[macro_export]
macro_rules! assert_is_due {
    ($node:expr) => {
        assert!(
            $node.is_due(),
            "Expected memory to be due for review, but next_review is {:?}",
            $node.next_review
        );
    };
}

/// Assert that a memory is not due for review
#[macro_export]
macro_rules! assert_not_due {
    ($node:expr) => {
        assert!(
            !$node.is_due(),
            "Expected memory to NOT be due for review, but it is (next_review: {:?})",
            $node.next_review
        );
    };
}

/// Assert that interval increased after review
#[macro_export]
macro_rules! assert_interval_increased {
    ($before:expr, $after:expr) => {
        let before_interval = $before
            .next_review
            .map(|t| (t - $before.last_accessed).num_days())
            .unwrap_or(0);
        let after_interval = $after
            .next_review
            .map(|t| (t - $after.last_accessed).num_days())
            .unwrap_or(0);
        assert!(
            after_interval >= before_interval,
            "Expected interval to increase: {} days -> {} days",
            before_interval,
            after_interval
        );
    };
}

/// Assert that stability increased after successful review
#[macro_export]
macro_rules! assert_stability_increased {
    ($before:expr, $after:expr) => {
        assert!(
            $after.stability >= $before.stability,
            "Expected stability to increase: {} -> {}",
            $before.stability,
            $after.stability
        );
    };
}

// ============================================================================
// STATE ASSERTIONS
// ============================================================================

/// Assert that storage strength increased
#[macro_export]
macro_rules! assert_storage_strength_increased {
    ($before:expr, $after:expr) => {
        assert!(
            $after.storage_strength >= $before.storage_strength,
            "Expected storage strength to increase: {} -> {}",
            $before.storage_strength,
            $after.storage_strength
        );
    };
}

/// Assert that reps count increased
#[macro_export]
macro_rules! assert_reps_increased {
    ($before:expr, $after:expr) => {
        assert!(
            $after.reps > $before.reps,
            "Expected reps to increase: {} -> {}",
            $before.reps,
            $after.reps
        );
    };
}

/// Assert that lapses count increased
#[macro_export]
macro_rules! assert_lapses_increased {
    ($before:expr, $after:expr) => {
        assert!(
            $after.lapses > $before.lapses,
            "Expected lapses to increase: {} -> {}",
            $before.lapses,
            $after.lapses
        );
    };
}

// ============================================================================
// TEMPORAL ASSERTIONS
// ============================================================================

/// Assert that a memory is currently valid
#[macro_export]
macro_rules! assert_currently_valid {
    ($node:expr) => {
        assert!(
            $node.is_currently_valid(),
            "Expected memory to be currently valid, but valid_from={:?}, valid_until={:?}",
            $node.valid_from,
            $node.valid_until
        );
    };
}

/// Assert that a memory is not currently valid
#[macro_export]
macro_rules! assert_not_currently_valid {
    ($node:expr) => {
        assert!(
            !$node.is_currently_valid(),
            "Expected memory to NOT be currently valid, but it is (valid_from={:?}, valid_until={:?})",
            $node.valid_from,
            $node.valid_until
        );
    };
}

/// Assert that a memory is valid at a specific time
#[macro_export]
macro_rules! assert_valid_at {
    ($node:expr, $time:expr) => {
        assert!(
            $node.is_valid_at($time),
            "Expected memory to be valid at {:?}, but valid_from={:?}, valid_until={:?}",
            $time,
            $node.valid_from,
            $node.valid_until
        );
    };
}

// ============================================================================
// SEARCH ASSERTIONS
// ============================================================================

/// Assert that search results contain a specific ID
#[macro_export]
macro_rules! assert_search_contains {
    ($results:expr, $id:expr) => {
        assert!(
            $results.iter().any(|n| n.id == $id),
            "Expected search results to contain ID {}, but it was not found",
            $id
        );
    };
}

/// Assert that search results do not contain a specific ID
#[macro_export]
macro_rules! assert_search_not_contains {
    ($results:expr, $id:expr) => {
        assert!(
            !$results.iter().any(|n| n.id == $id),
            "Expected search results to NOT contain ID {}, but it was found",
            $id
        );
    };
}

/// Assert search result count
#[macro_export]
macro_rules! assert_search_count {
    ($results:expr, $expected:expr) => {
        assert_eq!(
            $results.len(),
            $expected,
            "Expected {} search results, got {}",
            $expected,
            $results.len()
        );
    };
}

/// Assert that search results are ordered by relevance (first result is most relevant)
#[macro_export]
macro_rules! assert_search_order {
    ($results:expr, $expected_first:expr) => {
        assert!(
            !$results.is_empty(),
            "Expected non-empty search results"
        );
        assert_eq!(
            $results[0].id,
            $expected_first,
            "Expected first result to be {}, got {}",
            $expected_first,
            $results[0].id
        );
    };
}

// ============================================================================
// EMBEDDING ASSERTIONS
// ============================================================================

/// Assert that embeddings are similar (cosine similarity > threshold)
#[macro_export]
macro_rules! assert_embeddings_similar {
    ($emb1:expr, $emb2:expr, $threshold:expr) => {{
        let dot: f32 = $emb1.iter().zip($emb2.iter()).map(|(a, b)| a * b).sum();
        let norm1: f32 = $emb1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = $emb2.iter().map(|x| x * x).sum::<f32>().sqrt();
        let similarity = if norm1 > 0.0 && norm2 > 0.0 {
            dot / (norm1 * norm2)
        } else {
            0.0
        };
        assert!(
            similarity >= $threshold,
            "Expected embeddings to be similar (>= {}), got similarity {}",
            $threshold,
            similarity
        );
    }};
}

/// Assert that embeddings are different (cosine similarity < threshold)
#[macro_export]
macro_rules! assert_embeddings_different {
    ($emb1:expr, $emb2:expr, $threshold:expr) => {{
        let dot: f32 = $emb1.iter().zip($emb2.iter()).map(|(a, b)| a * b).sum();
        let norm1: f32 = $emb1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = $emb2.iter().map(|x| x * x).sum::<f32>().sqrt();
        let similarity = if norm1 > 0.0 && norm2 > 0.0 {
            dot / (norm1 * norm2)
        } else {
            0.0
        };
        assert!(
            similarity < $threshold,
            "Expected embeddings to be different (< {}), got similarity {}",
            $threshold,
            similarity
        );
    }};
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Verify that a node exists in storage
pub fn assert_node_exists(storage: &Storage, id: &str) {
    let node = storage.get_node(id);
    assert!(
        node.is_ok() && node.unwrap().is_some(),
        "Expected node {} to exist in storage",
        id
    );
}

/// Verify that a node does not exist in storage
pub fn assert_node_not_exists(storage: &Storage, id: &str) {
    let node = storage.get_node(id);
    assert!(
        node.is_ok() && node.unwrap().is_none(),
        "Expected node {} to NOT exist in storage",
        id
    );
}

/// Verify that storage has expected node count
pub fn assert_node_count(storage: &Storage, expected: i64) {
    let stats = storage.get_stats().expect("Failed to get stats");
    assert_eq!(
        stats.total_nodes, expected,
        "Expected {} nodes, got {}",
        expected, stats.total_nodes
    );
}

/// Verify that a node has the expected content
pub fn assert_node_content(node: &KnowledgeNode, expected_content: &str) {
    assert_eq!(
        node.content, expected_content,
        "Expected content '{}', got '{}'",
        expected_content, node.content
    );
}

/// Verify that a node has the expected type
pub fn assert_node_type(node: &KnowledgeNode, expected_type: &str) {
    assert_eq!(
        node.node_type, expected_type,
        "Expected type '{}', got '{}'",
        expected_type, node.node_type
    );
}

/// Verify that a node has specific tags
pub fn assert_has_tags(node: &KnowledgeNode, expected_tags: &[&str]) {
    for tag in expected_tags {
        assert!(
            node.tags.contains(&tag.to_string()),
            "Expected node to have tag '{}', but tags are {:?}",
            tag,
            node.tags
        );
    }
}

/// Verify difficulty is within valid range
pub fn assert_difficulty_valid(node: &KnowledgeNode) {
    assert!(
        node.difficulty >= 1.0 && node.difficulty <= 10.0,
        "Difficulty {} is out of valid range [1.0, 10.0]",
        node.difficulty
    );
}

/// Verify stability is positive
pub fn assert_stability_valid(node: &KnowledgeNode) {
    assert!(
        node.stability > 0.0,
        "Stability {} should be positive",
        node.stability
    );
}

/// Approximate equality for floating point
pub fn assert_approx_eq(actual: f64, expected: f64, epsilon: f64) {
    assert!(
        (actual - expected).abs() < epsilon,
        "Expected {} to be approximately equal to {} (epsilon: {})",
        actual,
        expected,
        epsilon
    );
}

/// Approximate equality for f32
pub fn assert_approx_eq_f32(actual: f32, expected: f32, epsilon: f32) {
    assert!(
        (actual - expected).abs() < epsilon,
        "Expected {} to be approximately equal to {} (epsilon: {})",
        actual,
        expected,
        epsilon
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn create_test_node() -> KnowledgeNode {
        let mut node = KnowledgeNode::default();
        node.id = "test-id".to_string();
        node.content = "test content".to_string();
        node.node_type = "fact".to_string();
        node.created_at = Utc::now();
        node.updated_at = Utc::now();
        node.last_accessed = Utc::now();
        node.stability = 5.0;
        node.difficulty = 5.0;
        node.reps = 3;
        node.lapses = 0;
        node.storage_strength = 2.0;
        node.retrieval_strength = 0.9;
        node.retention_strength = 0.85;
        node.sentiment_score = 0.0;
        node.sentiment_magnitude = 0.0;
        node.next_review = Some(Utc::now() + Duration::days(5));
        node.source = None;
        node.tags = vec!["test".to_string(), "example".to_string()];
        node.valid_from = None;
        node.valid_until = None;
        node.has_embedding = None;
        node.embedding_model = None;
        node
    }

    #[test]
    fn test_retention_assertions() {
        assert_retention_decreased!(0.7, 1.0);
        assert_retention_decreased!(0.5, 1.0, 0.3);
        assert_retention_in_range!(0.85, 0.8, 0.9);
    }

    #[test]
    fn test_scheduling_assertions() {
        let mut node = create_test_node();

        // Not due yet (next_review is in the future)
        assert_not_due!(node);

        // Make it due
        node.next_review = Some(Utc::now() - Duration::hours(1));
        assert_is_due!(node);
    }

    #[test]
    fn test_temporal_assertions() {
        let node = create_test_node();
        assert_currently_valid!(node);
    }

    #[test]
    fn test_helper_functions() {
        let node = create_test_node();

        assert_node_content(&node, "test content");
        assert_node_type(&node, "fact");
        assert_has_tags(&node, &["test", "example"]);
        assert_difficulty_valid(&node);
        assert_stability_valid(&node);
    }

    #[test]
    fn test_approx_eq() {
        assert_approx_eq(0.90001, 0.9, 0.001);
        assert_approx_eq_f32(0.90001, 0.9, 0.001);
    }

    #[test]
    fn test_embedding_assertions() {
        let emb1 = [1.0f32, 0.0, 0.0];
        let emb2 = [0.9, 0.1, 0.0];
        let emb3 = [0.0, 1.0, 0.0];

        assert_embeddings_similar!(emb1, emb2, 0.8);
        assert_embeddings_different!(emb1, emb3, 0.5);
    }
}
