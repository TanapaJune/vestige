//! Hybrid Multi-Model Embedding Fusion
//!
//! Combines multiple embedding models for improved semantic coverage:
//! - General text: all-MiniLM-L6-v2
//! - Code: code-specific models
//! - Scientific: domain-specific models
//!
//! Uses weighted fusion to combine embeddings from different models.

use super::local::Embedding;

// ============================================================================
// HYBRID EMBEDDING
// ============================================================================

/// Hybrid embedding combining multiple sources
#[derive(Debug, Clone)]
pub struct HybridEmbedding {
    /// Primary embedding (text)
    pub primary: Embedding,
    /// Secondary embeddings (specialized)
    pub secondary: Vec<(String, Embedding)>,
    /// Fusion weights
    pub weights: Vec<f32>,
}

impl HybridEmbedding {
    /// Create a hybrid embedding from a primary embedding
    pub fn from_primary(primary: Embedding) -> Self {
        Self {
            primary,
            secondary: Vec::new(),
            weights: vec![1.0],
        }
    }

    /// Add a secondary embedding with a model name
    pub fn add_secondary(
        &mut self,
        model_name: impl Into<String>,
        embedding: Embedding,
        weight: f32,
    ) {
        self.secondary.push((model_name.into(), embedding));
        self.weights.push(weight);
    }

    /// Compute fused similarity with another hybrid embedding
    pub fn fused_similarity(&self, other: &HybridEmbedding) -> f32 {
        // Normalize weights
        let total_weight: f32 = self.weights.iter().sum();
        if total_weight == 0.0 {
            return 0.0;
        }

        let mut total_sim = 0.0_f32;
        let mut weight_used = 0.0_f32;

        // Primary similarity
        total_sim += self.primary.cosine_similarity(&other.primary) * self.weights[0];
        weight_used += self.weights[0];

        // Secondary similarities (if models match)
        for (i, (name, emb)) in self.secondary.iter().enumerate() {
            if let Some((_, other_emb)) = other.secondary.iter().find(|(n, _)| n == name) {
                let weight = self.weights.get(i + 1).copied().unwrap_or(0.0);
                total_sim += emb.cosine_similarity(other_emb) * weight;
                weight_used += weight;
            }
        }

        if weight_used > 0.0 {
            total_sim / weight_used
        } else {
            0.0
        }
    }

    /// Get the primary embedding vector
    pub fn primary_vector(&self) -> &[f32] {
        &self.primary.vector
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_embedding() {
        let primary = Embedding::new(vec![1.0, 0.0, 0.0]);
        let mut hybrid = HybridEmbedding::from_primary(primary.clone());

        hybrid.add_secondary("code", Embedding::new(vec![0.0, 1.0, 0.0]), 0.5);

        assert_eq!(hybrid.secondary.len(), 1);
        assert_eq!(hybrid.weights.len(), 2);
    }

    #[test]
    fn test_fused_similarity() {
        let mut h1 = HybridEmbedding::from_primary(Embedding::new(vec![1.0, 0.0]));
        h1.add_secondary("code", Embedding::new(vec![1.0, 0.0]), 1.0);

        let mut h2 = HybridEmbedding::from_primary(Embedding::new(vec![1.0, 0.0]));
        h2.add_secondary("code", Embedding::new(vec![1.0, 0.0]), 1.0);

        let sim = h1.fused_similarity(&h2);
        assert!((sim - 1.0).abs() < 0.001);
    }
}
