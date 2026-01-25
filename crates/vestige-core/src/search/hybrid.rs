//! Hybrid Search (Keyword + Semantic + RRF)
//!
//! Combines keyword (BM25/FTS5) and semantic (embedding) search
//! using Reciprocal Rank Fusion for optimal results.

use std::collections::HashMap;

// ============================================================================
// FUSION ALGORITHMS
// ============================================================================

/// Reciprocal Rank Fusion for combining search results
///
/// Combines keyword (BM25) and semantic search results using the RRF formula:
/// score(d) = sum of 1/(k + rank(d)) across all result lists
///
/// RRF is effective because:
/// - It normalizes across different scoring scales
/// - It rewards items appearing in multiple result lists
/// - The k parameter (typically 60) dampens the effect of high ranks
///
/// # Arguments
/// * `keyword_results` - Results from keyword search (id, score)
/// * `semantic_results` - Results from semantic search (id, score)
/// * `k` - Fusion constant (default 60.0)
///
/// # Returns
/// Combined results sorted by RRF score
pub fn reciprocal_rank_fusion(
    keyword_results: &[(String, f32)],
    semantic_results: &[(String, f32)],
    k: f32,
) -> Vec<(String, f32)> {
    let mut scores: HashMap<String, f32> = HashMap::new();

    // Add keyword search scores
    for (rank, (key, _)) in keyword_results.iter().enumerate() {
        *scores.entry(key.clone()).or_default() += 1.0 / (k + rank as f32);
    }

    // Add semantic search scores
    for (rank, (key, _)) in semantic_results.iter().enumerate() {
        *scores.entry(key.clone()).or_default() += 1.0 / (k + rank as f32);
    }

    // Sort by combined score
    let mut results: Vec<(String, f32)> = scores.into_iter().collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    results
}

/// Linear combination of search results with weights
///
/// Combines results using weighted sum of normalized scores.
/// Good when you have prior knowledge about relative importance.
///
/// # Arguments
/// * `keyword_results` - Results from keyword search
/// * `semantic_results` - Results from semantic search
/// * `keyword_weight` - Weight for keyword results (0.0 to 1.0)
/// * `semantic_weight` - Weight for semantic results (0.0 to 1.0)
pub fn linear_combination(
    keyword_results: &[(String, f32)],
    semantic_results: &[(String, f32)],
    keyword_weight: f32,
    semantic_weight: f32,
) -> Vec<(String, f32)> {
    let mut scores: HashMap<String, f32> = HashMap::new();

    // Normalize and add keyword search scores
    let max_keyword = keyword_results
        .first()
        .map(|(_, s)| *s)
        .unwrap_or(1.0)
        .max(0.001);
    for (key, score) in keyword_results {
        *scores.entry(key.clone()).or_default() += (score / max_keyword) * keyword_weight;
    }

    // Normalize and add semantic search scores
    let max_semantic = semantic_results
        .first()
        .map(|(_, s)| *s)
        .unwrap_or(1.0)
        .max(0.001);
    for (key, score) in semantic_results {
        *scores.entry(key.clone()).or_default() += (score / max_semantic) * semantic_weight;
    }

    // Sort by combined score
    let mut results: Vec<(String, f32)> = scores.into_iter().collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    results
}

// ============================================================================
// HYBRID SEARCH CONFIGURATION
// ============================================================================

/// Configuration for hybrid search
#[derive(Debug, Clone)]
pub struct HybridSearchConfig {
    /// Weight for keyword (BM25/FTS5) results
    pub keyword_weight: f32,
    /// Weight for semantic (embedding) results
    pub semantic_weight: f32,
    /// RRF constant (higher = more uniform weighting)
    pub rrf_k: f32,
    /// Minimum semantic similarity threshold
    pub min_semantic_similarity: f32,
    /// Number of results to fetch from each source before fusion
    pub source_limit_multiplier: usize,
}

impl Default for HybridSearchConfig {
    fn default() -> Self {
        Self {
            keyword_weight: 0.5,
            semantic_weight: 0.5,
            rrf_k: 60.0,
            min_semantic_similarity: 0.3,
            source_limit_multiplier: 2,
        }
    }
}

// ============================================================================
// HYBRID SEARCHER
// ============================================================================

/// Hybrid search combining keyword and semantic search
pub struct HybridSearcher {
    config: HybridSearchConfig,
}

impl Default for HybridSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl HybridSearcher {
    /// Create a new hybrid searcher with default config
    pub fn new() -> Self {
        Self {
            config: HybridSearchConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: HybridSearchConfig) -> Self {
        Self { config }
    }

    /// Get current configuration
    pub fn config(&self) -> &HybridSearchConfig {
        &self.config
    }

    /// Fuse keyword and semantic results using RRF
    pub fn fuse_rrf(
        &self,
        keyword_results: &[(String, f32)],
        semantic_results: &[(String, f32)],
    ) -> Vec<(String, f32)> {
        reciprocal_rank_fusion(keyword_results, semantic_results, self.config.rrf_k)
    }

    /// Fuse results using linear combination
    pub fn fuse_linear(
        &self,
        keyword_results: &[(String, f32)],
        semantic_results: &[(String, f32)],
    ) -> Vec<(String, f32)> {
        linear_combination(
            keyword_results,
            semantic_results,
            self.config.keyword_weight,
            self.config.semantic_weight,
        )
    }

    /// Determine if semantic search should be used based on query
    ///
    /// Semantic search is more effective for:
    /// - Conceptual queries
    /// - Questions
    /// - Natural language
    ///
    /// Keyword search is more effective for:
    /// - Exact terms
    /// - Code/identifiers
    /// - Specific phrases
    pub fn should_use_semantic(&self, query: &str) -> bool {
        // Heuristics for when semantic search is useful
        let is_question = query.contains('?')
            || query.to_lowercase().starts_with("what ")
            || query.to_lowercase().starts_with("how ")
            || query.to_lowercase().starts_with("why ")
            || query.to_lowercase().starts_with("when ");

        let is_conceptual = query.split_whitespace().count() >= 3
            && !query.contains('(')
            && !query.contains('{')
            && !query.contains('=');

        is_question || is_conceptual
    }

    /// Calculate the effective limit for source queries
    pub fn effective_source_limit(&self, target_limit: usize) -> usize {
        target_limit * self.config.source_limit_multiplier
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reciprocal_rank_fusion() {
        let keyword = vec![
            ("doc-1".to_string(), 0.9),
            ("doc-2".to_string(), 0.8),
            ("doc-3".to_string(), 0.7),
        ];
        let semantic = vec![
            ("doc-2".to_string(), 0.95),
            ("doc-1".to_string(), 0.85),
            ("doc-4".to_string(), 0.75),
        ];

        let results = reciprocal_rank_fusion(&keyword, &semantic, 60.0);

        // doc-1 and doc-2 appear in both, should be at top
        assert!(results.iter().any(|(k, _)| k == "doc-1"));
        assert!(results.iter().any(|(k, _)| k == "doc-2"));

        // Results should be sorted by score descending
        for i in 1..results.len() {
            assert!(results[i - 1].1 >= results[i].1);
        }
    }

    #[test]
    fn test_linear_combination() {
        let keyword = vec![("doc-1".to_string(), 1.0), ("doc-2".to_string(), 0.5)];
        let semantic = vec![("doc-2".to_string(), 1.0), ("doc-3".to_string(), 0.5)];

        let results = linear_combination(&keyword, &semantic, 0.5, 0.5);

        // doc-2 appears in both with high scores, should be first or second
        let doc2_pos = results.iter().position(|(k, _)| k == "doc-2");
        assert!(doc2_pos.is_some());
    }

    #[test]
    fn test_hybrid_searcher() {
        let searcher = HybridSearcher::new();

        // Semantic queries
        assert!(searcher.should_use_semantic("What is the meaning of life?"));
        assert!(searcher.should_use_semantic("how does memory work"));

        // Keyword queries
        assert!(!searcher.should_use_semantic("fn main()"));
        assert!(!searcher.should_use_semantic("error"));
    }

    #[test]
    fn test_effective_source_limit() {
        let searcher = HybridSearcher::new();
        assert_eq!(searcher.effective_source_limit(10), 20);
    }

    #[test]
    fn test_rrf_with_empty_results() {
        let keyword: Vec<(String, f32)> = vec![];
        let semantic = vec![("doc-1".to_string(), 0.9)];

        let results = reciprocal_rank_fusion(&keyword, &semantic, 60.0);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "doc-1");
    }

    #[test]
    fn test_linear_with_unequal_weights() {
        let keyword = vec![("doc-1".to_string(), 1.0)];
        let semantic = vec![("doc-2".to_string(), 1.0)];

        // Heavy keyword weight
        let results = linear_combination(&keyword, &semantic, 0.9, 0.1);

        // doc-1 should have higher score
        let doc1_score = results.iter().find(|(k, _)| k == "doc-1").map(|(_, s)| *s);
        let doc2_score = results.iter().find(|(k, _)| k == "doc-2").map(|(_, s)| *s);

        assert!(doc1_score.unwrap() > doc2_score.unwrap());
    }
}
