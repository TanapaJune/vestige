//! Sleep Consolidation
//!
//! Bio-inspired memory consolidation that mimics what happens during sleep:
//!
//! 1. **Decay Phase**: Apply forgetting curve to all memories
//! 2. **Replay Phase**: "Replay" important memories (boost storage strength)
//! 3. **Integration Phase**: Generate embeddings, find connections
//! 4. **Pruning Phase**: Remove very weak memories (optional)
//!
//! This should be run periodically (e.g., once per day, or on app startup).

use std::time::Instant;

use crate::memory::ConsolidationResult;

// ============================================================================
// CONSOLIDATION CONFIG
// ============================================================================

/// Configuration for sleep consolidation
#[derive(Debug, Clone)]
pub struct ConsolidationConfig {
    /// Whether to apply memory decay
    pub apply_decay: bool,
    /// Whether to promote emotional memories
    pub promote_emotional: bool,
    /// Minimum sentiment magnitude for promotion
    pub emotional_threshold: f64,
    /// Promotion boost factor
    pub promotion_factor: f64,
    /// Whether to generate missing embeddings
    pub generate_embeddings: bool,
    /// Maximum embeddings to generate per run
    pub max_embeddings_per_run: usize,
    /// Whether to prune weak memories
    pub enable_pruning: bool,
    /// Minimum retention to keep memory
    pub pruning_threshold: f64,
    /// Minimum age (days) before pruning
    pub pruning_min_age_days: i64,
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        Self {
            apply_decay: true,
            promote_emotional: true,
            emotional_threshold: 0.5,
            promotion_factor: 1.5,
            generate_embeddings: true,
            max_embeddings_per_run: 100,
            enable_pruning: false, // Disabled by default for safety
            pruning_threshold: 0.1,
            pruning_min_age_days: 30,
        }
    }
}

// ============================================================================
// SLEEP CONSOLIDATION
// ============================================================================

/// Sleep-inspired memory consolidation engine
pub struct SleepConsolidation {
    config: ConsolidationConfig,
}

impl Default for SleepConsolidation {
    fn default() -> Self {
        Self::new()
    }
}

impl SleepConsolidation {
    /// Create a new consolidation engine
    pub fn new() -> Self {
        Self {
            config: ConsolidationConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: ConsolidationConfig) -> Self {
        Self { config }
    }

    /// Get current configuration
    pub fn config(&self) -> &ConsolidationConfig {
        &self.config
    }

    /// Run consolidation (standalone, without storage)
    ///
    /// This performs calculations but doesn't actually modify storage.
    /// Use Storage::run_consolidation() for the full implementation.
    pub fn calculate_decay(&self, stability: f64, days_elapsed: f64, sentiment_mag: f64) -> f64 {
        const FSRS_DECAY: f64 = 0.5;
        const FSRS_FACTOR: f64 = 9.0;

        if days_elapsed <= 0.0 || stability <= 0.0 {
            return 1.0;
        }

        // Apply sentiment boost to effective stability
        let effective_stability = stability * (1.0 + sentiment_mag * 0.5);

        // FSRS-6 power law decay
        (1.0 + days_elapsed / (FSRS_FACTOR * effective_stability))
            .powf(-1.0 / FSRS_DECAY)
            .clamp(0.0, 1.0)
    }

    /// Calculate combined retention
    pub fn calculate_retention(&self, storage_strength: f64, retrieval_strength: f64) -> f64 {
        (retrieval_strength * 0.7) + ((storage_strength / 10.0).min(1.0) * 0.3)
    }

    /// Determine if a memory should be promoted
    pub fn should_promote(&self, sentiment_magnitude: f64, storage_strength: f64) -> bool {
        self.config.promote_emotional
            && sentiment_magnitude > self.config.emotional_threshold
            && storage_strength < 10.0
    }

    /// Calculate promotion boost
    pub fn promotion_boost(&self, current_strength: f64) -> f64 {
        (current_strength * self.config.promotion_factor).min(10.0)
    }

    /// Determine if a memory should be pruned
    pub fn should_prune(&self, retention: f64, age_days: i64) -> bool {
        self.config.enable_pruning
            && retention < self.config.pruning_threshold
            && age_days > self.config.pruning_min_age_days
    }

    /// Create a consolidation result tracker
    pub fn start_run(&self) -> ConsolidationRun {
        ConsolidationRun {
            start_time: Instant::now(),
            nodes_processed: 0,
            nodes_promoted: 0,
            nodes_pruned: 0,
            decay_applied: 0,
            embeddings_generated: 0,
        }
    }
}

/// Tracks a consolidation run in progress
pub struct ConsolidationRun {
    start_time: Instant,
    pub nodes_processed: i64,
    pub nodes_promoted: i64,
    pub nodes_pruned: i64,
    pub decay_applied: i64,
    pub embeddings_generated: i64,
}

impl ConsolidationRun {
    /// Record that decay was applied to a node
    pub fn record_decay(&mut self) {
        self.decay_applied += 1;
        self.nodes_processed += 1;
    }

    /// Record that a node was promoted
    pub fn record_promotion(&mut self) {
        self.nodes_promoted += 1;
    }

    /// Record that a node was pruned
    pub fn record_prune(&mut self) {
        self.nodes_pruned += 1;
    }

    /// Record that an embedding was generated
    pub fn record_embedding(&mut self) {
        self.embeddings_generated += 1;
    }

    /// Finish the run and create a result
    pub fn finish(self) -> ConsolidationResult {
        ConsolidationResult {
            nodes_processed: self.nodes_processed,
            nodes_promoted: self.nodes_promoted,
            nodes_pruned: self.nodes_pruned,
            decay_applied: self.decay_applied,
            duration_ms: self.start_time.elapsed().as_millis() as i64,
            embeddings_generated: self.embeddings_generated,
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
    fn test_consolidation_creation() {
        let consolidation = SleepConsolidation::new();
        assert!(consolidation.config().apply_decay);
        assert!(consolidation.config().promote_emotional);
    }

    #[test]
    fn test_decay_calculation() {
        let consolidation = SleepConsolidation::new();

        // No time elapsed = full retention
        let r0 = consolidation.calculate_decay(10.0, 0.0, 0.0);
        assert!((r0 - 1.0).abs() < 0.01);

        // Time elapsed = decay
        let r1 = consolidation.calculate_decay(10.0, 5.0, 0.0);
        assert!(r1 < 1.0);
        assert!(r1 > 0.0);

        // Emotional memory decays slower
        let r_neutral = consolidation.calculate_decay(10.0, 5.0, 0.0);
        let r_emotional = consolidation.calculate_decay(10.0, 5.0, 1.0);
        assert!(r_emotional > r_neutral);
    }

    #[test]
    fn test_retention_calculation() {
        let consolidation = SleepConsolidation::new();

        // Full retrieval, low storage
        let r1 = consolidation.calculate_retention(1.0, 1.0);
        assert!(r1 > 0.7);

        // Full retrieval, max storage
        let r2 = consolidation.calculate_retention(10.0, 1.0);
        assert!((r2 - 1.0).abs() < 0.01);

        // Low retrieval, max storage
        let r3 = consolidation.calculate_retention(10.0, 0.0);
        assert!((r3 - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_should_promote() {
        let consolidation = SleepConsolidation::new();

        // High emotion, low storage = promote
        assert!(consolidation.should_promote(0.8, 5.0));

        // Low emotion = don't promote
        assert!(!consolidation.should_promote(0.3, 5.0));

        // Max storage = don't promote
        assert!(!consolidation.should_promote(0.8, 10.0));
    }

    #[test]
    fn test_should_prune() {
        let consolidation = SleepConsolidation::new();

        // Pruning disabled by default
        assert!(!consolidation.should_prune(0.05, 60));

        // Enable pruning
        let config = ConsolidationConfig {
            enable_pruning: true,
            ..Default::default()
        };
        let consolidation = SleepConsolidation::with_config(config);

        // Low retention, old = prune
        assert!(consolidation.should_prune(0.05, 60));

        // Low retention, young = don't prune
        assert!(!consolidation.should_prune(0.05, 10));

        // High retention = don't prune
        assert!(!consolidation.should_prune(0.5, 60));
    }

    #[test]
    fn test_consolidation_run() {
        let consolidation = SleepConsolidation::new();
        let mut run = consolidation.start_run();

        run.record_decay();
        run.record_decay();
        run.record_promotion();
        run.record_embedding();

        let result = run.finish();

        assert_eq!(result.nodes_processed, 2);
        assert_eq!(result.decay_applied, 2);
        assert_eq!(result.nodes_promoted, 1);
        assert_eq!(result.embeddings_generated, 1);
        assert!(result.duration_ms >= 0);
    }
}
