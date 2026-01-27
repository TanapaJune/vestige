//! Mock Embedding Service using FxHash
//!
//! Provides deterministic embeddings for testing without requiring
//! the actual fastembed model. Uses FxHash for fast, consistent hashing.
//!
//! Key properties:
//! - Deterministic: Same input always produces same embedding
//! - Fast: No ML model loading/inference
//! - Semantic similarity: Similar strings produce similar embeddings
//! - Normalized: All embeddings have unit length

use std::collections::HashMap;

/// Dimensions for mock embeddings (matches BGE-base-en-v1.5)
pub const MOCK_EMBEDDING_DIM: usize = 768;

/// FxHash implementation (fast, non-cryptographic hash)
/// Based on Firefox's hash function
fn fx_hash(data: &[u8]) -> u64 {
    const SEED: u64 = 0x517cc1b727220a95;
    let mut hash = SEED;
    for &byte in data {
        hash = hash.rotate_left(5) ^ (byte as u64);
        hash = hash.wrapping_mul(SEED);
    }
    hash
}

/// Mock embedding service for testing
///
/// Produces deterministic embeddings based on text content using FxHash.
/// Designed to approximate real embedding behavior:
/// - Similar texts produce similar embeddings
/// - Different texts produce different embeddings
/// - Embeddings are normalized to unit length
///
/// # Example
///
/// ```rust,ignore
/// let service = MockEmbeddingService::new();
///
/// let emb1 = service.embed("hello world");
/// let emb2 = service.embed("hello world");
/// let emb3 = service.embed("goodbye world");
///
/// // Same input = same output
/// assert_eq!(emb1, emb2);
///
/// // Different input = different output
/// assert_ne!(emb1, emb3);
///
/// // But similar inputs have higher similarity
/// let sim_same = service.cosine_similarity(&emb1, &emb2);
/// let sim_diff = service.cosine_similarity(&emb1, &emb3);
/// assert!(sim_same > sim_diff);
/// ```
pub struct MockEmbeddingService {
    /// Cache for computed embeddings
    cache: HashMap<String, Vec<f32>>,
    /// Whether to use word-level hashing for better semantic similarity
    semantic_mode: bool,
}

impl Default for MockEmbeddingService {
    fn default() -> Self {
        Self::new()
    }
}

impl MockEmbeddingService {
    /// Create a new mock embedding service
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            semantic_mode: true,
        }
    }

    /// Create a service without semantic mode (pure hash-based)
    pub fn new_simple() -> Self {
        Self {
            cache: HashMap::new(),
            semantic_mode: false,
        }
    }

    /// Embed text into a vector
    pub fn embed(&mut self, text: &str) -> Vec<f32> {
        // Check cache first
        if let Some(cached) = self.cache.get(text) {
            return cached.clone();
        }

        let embedding = if self.semantic_mode {
            self.semantic_embed(text)
        } else {
            self.simple_embed(text)
        };

        self.cache.insert(text.to_string(), embedding.clone());
        embedding
    }

    /// Simple hash-based embedding
    fn simple_embed(&self, text: &str) -> Vec<f32> {
        let mut embedding = vec![0.0f32; MOCK_EMBEDDING_DIM];
        let normalized = text.to_lowercase();

        // Use multiple hash seeds for different dimensions
        for (i, chunk) in embedding.chunks_mut(64).enumerate() {
            let seed_text = format!("{}:{}", i, normalized);
            let hash = fx_hash(seed_text.as_bytes());

            for (j, val) in chunk.iter_mut().enumerate() {
                // Generate pseudo-random float from hash
                let shifted = hash.rotate_left((j * 5) as u32);
                *val = ((shifted as f32 / u64::MAX as f32) * 2.0) - 1.0;
            }
        }

        normalize(&mut embedding);
        embedding
    }

    /// Semantic-aware embedding (word-level hashing)
    fn semantic_embed(&self, text: &str) -> Vec<f32> {
        let mut embedding = vec![0.0f32; MOCK_EMBEDDING_DIM];
        let normalized = text.to_lowercase();

        // Tokenize into words
        let words: Vec<&str> = normalized
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .collect();

        if words.is_empty() {
            // Fall back to simple embedding for empty text
            return self.simple_embed(text);
        }

        // Each word contributes to the embedding
        for word in &words {
            let word_hash = fx_hash(word.as_bytes());

            // Map word to a sparse set of dimensions
            for i in 0..16 {
                let dim = ((word_hash >> (i * 4)) as usize) % MOCK_EMBEDDING_DIM;
                let sign = if (word_hash >> (i + 48)) & 1 == 0 { 1.0 } else { -1.0 };
                let magnitude = ((word_hash >> (i * 2)) as f32 % 100.0) / 100.0 + 0.5;
                embedding[dim] += sign * magnitude;
            }
        }

        // Add position-aware component for word order sensitivity
        for (pos, word) in words.iter().enumerate() {
            let pos_hash = fx_hash(format!("{}:{}", pos, word).as_bytes());
            let dim = (pos_hash as usize) % MOCK_EMBEDDING_DIM;
            let weight = 1.0 / (pos as f32 + 1.0);
            embedding[dim] += weight;
        }

        // Add character n-gram features for subword similarity
        let chars: Vec<char> = normalized.chars().collect();
        for i in 0..chars.len().saturating_sub(2) {
            let trigram: String = chars[i..i + 3].iter().collect();
            let hash = fx_hash(trigram.as_bytes());
            let dim = (hash as usize) % MOCK_EMBEDDING_DIM;
            embedding[dim] += 0.1;
        }

        normalize(&mut embedding);
        embedding
    }

    /// Calculate cosine similarity between two embeddings
    pub fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot / (norm_a * norm_b)
    }

    /// Calculate euclidean distance between two embeddings
    pub fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return f32::MAX;
        }

        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    /// Find most similar embedding from a set
    pub fn find_most_similar<'a>(
        &self,
        query: &[f32],
        candidates: &'a [(String, Vec<f32>)],
    ) -> Option<(&'a str, f32)> {
        candidates
            .iter()
            .map(|(id, emb)| (id.as_str(), self.cosine_similarity(query, emb)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Clear the embedding cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Check if service is ready (always true for mock)
    pub fn is_ready(&self) -> bool {
        true
    }
}

/// Normalize a vector to unit length
fn normalize(v: &mut [f32]) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_embedding() {
        let mut service = MockEmbeddingService::new();

        let emb1 = service.embed("hello world");
        let emb2 = service.embed("hello world");

        assert_eq!(emb1, emb2);
    }

    #[test]
    fn test_different_texts_different_embeddings() {
        let mut service = MockEmbeddingService::new();

        let emb1 = service.embed("hello world");
        let emb2 = service.embed("goodbye universe");

        assert_ne!(emb1, emb2);
    }

    #[test]
    fn test_embedding_dimension() {
        let mut service = MockEmbeddingService::new();
        let emb = service.embed("test text");

        assert_eq!(emb.len(), MOCK_EMBEDDING_DIM);
    }

    #[test]
    fn test_normalized_embeddings() {
        let mut service = MockEmbeddingService::new();
        let emb = service.embed("test normalization");

        let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_semantic_similarity() {
        let mut service = MockEmbeddingService::new();

        let emb_dog = service.embed("the dog runs fast");
        let emb_cat = service.embed("the cat runs fast");
        let emb_car = service.embed("machine learning algorithms");

        let sim_animals = service.cosine_similarity(&emb_dog, &emb_cat);
        let sim_different = service.cosine_similarity(&emb_dog, &emb_car);

        // Similar sentences should have higher similarity
        assert!(sim_animals > sim_different);
    }

    #[test]
    fn test_cosine_similarity_range() {
        let mut service = MockEmbeddingService::new();

        let emb1 = service.embed("test one");
        let emb2 = service.embed("test two");

        let sim = service.cosine_similarity(&emb1, &emb2);

        // Cosine similarity should be in [-1, 1]
        assert!((-1.0..=1.0).contains(&sim));
    }

    #[test]
    fn test_self_similarity() {
        let mut service = MockEmbeddingService::new();
        let emb = service.embed("self similarity test");

        let sim = service.cosine_similarity(&emb, &emb);
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_caching() {
        let mut service = MockEmbeddingService::new();
        assert_eq!(service.cache_size(), 0);

        service.embed("text one");
        assert_eq!(service.cache_size(), 1);

        service.embed("text one"); // Should use cache
        assert_eq!(service.cache_size(), 1);

        service.embed("text two");
        assert_eq!(service.cache_size(), 2);

        service.clear_cache();
        assert_eq!(service.cache_size(), 0);
    }

    #[test]
    fn test_find_most_similar() {
        let mut service = MockEmbeddingService::new();

        let query = service.embed("programming code");
        let candidates = vec![
            ("doc1".to_string(), service.embed("python programming language")),
            ("doc2".to_string(), service.embed("cooking recipes")),
            ("doc3".to_string(), service.embed("software development code")),
        ];

        let result = service.find_most_similar(&query, &candidates);
        assert!(result.is_some());

        // Should find a programming-related document
        let (id, _) = result.unwrap();
        assert!(id == "doc1" || id == "doc3");
    }

    #[test]
    fn test_empty_text() {
        let mut service = MockEmbeddingService::new();
        let emb = service.embed("");

        assert_eq!(emb.len(), MOCK_EMBEDDING_DIM);
    }

    #[test]
    fn test_simple_mode() {
        let mut service = MockEmbeddingService::new_simple();
        let emb = service.embed("test simple mode");

        assert_eq!(emb.len(), MOCK_EMBEDDING_DIM);

        // Verify normalization
        let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001);
    }
}
