//! Dual-Strength Memory Model (Bjork & Bjork, 1992)
//!
//! Implements the new theory of disuse which distinguishes between:
//!
//! - **Storage Strength**: How well-encoded the memory is. Increases with
//!   each successful retrieval and never decays. Higher storage strength
//!   means the memory can be relearned faster if forgotten.
//!
//! - **Retrieval Strength**: How accessible the memory is right now.
//!   Decays over time following a power law (FSRS-6 compatible).
//!   Higher retrieval strength means easier recall.
//!
//! Key insight: Difficult retrievals (low retrieval strength + high storage
//! strength) lead to larger gains in both strengths ("desirable difficulties").

use serde::{Deserialize, Serialize};

// ============================================================================
// CONSTANTS
// ============================================================================

/// Maximum storage strength (caps accumulation)
pub const MAX_STORAGE_STRENGTH: f64 = 10.0;

/// FSRS-6 decay constant (power law exponent)
/// Slower decay than exponential for short intervals
pub const FSRS_DECAY: f64 = 0.5;

/// FSRS-6 factor (derived from decay optimization)
pub const FSRS_FACTOR: f64 = 9.0;

// ============================================================================
// DUAL STRENGTH MODEL
// ============================================================================

/// Dual-strength memory state
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DualStrength {
    /// Storage strength (1.0 - 10.0)
    pub storage: f64,
    /// Retrieval strength (0.0 - 1.0)
    pub retrieval: f64,
}

impl Default for DualStrength {
    fn default() -> Self {
        Self {
            storage: 1.0,
            retrieval: 1.0,
        }
    }
}

impl DualStrength {
    /// Create new dual strength with initial values
    pub fn new(storage: f64, retrieval: f64) -> Self {
        Self {
            storage: storage.clamp(0.0, MAX_STORAGE_STRENGTH),
            retrieval: retrieval.clamp(0.0, 1.0),
        }
    }

    /// Calculate combined retention strength
    ///
    /// Uses a weighted combination:
    /// - 70% retrieval strength (current accessibility)
    /// - 30% storage strength (normalized to 0-1 range)
    pub fn retention(&self) -> f64 {
        (self.retrieval * 0.7) + ((self.storage / MAX_STORAGE_STRENGTH) * 0.3)
    }

    /// Update strengths after a successful recall
    ///
    /// - Storage strength increases (memory becomes more durable)
    /// - Retrieval strength resets to 1.0 (just accessed)
    pub fn on_successful_recall(&mut self) {
        self.storage = (self.storage + 0.1).min(MAX_STORAGE_STRENGTH);
        self.retrieval = 1.0;
    }

    /// Update strengths after a failed recall (lapse)
    ///
    /// - Storage strength still increases (effort strengthens encoding)
    /// - Retrieval strength resets to 1.0 (just relearned)
    pub fn on_lapse(&mut self) {
        self.storage = (self.storage + 0.3).min(MAX_STORAGE_STRENGTH);
        self.retrieval = 1.0;
    }

    /// Apply time-based decay to retrieval strength
    ///
    /// Uses FSRS-6 power law formula which better matches human forgetting:
    /// R = (1 + t/(FACTOR * S))^(-1/DECAY)
    pub fn apply_decay(&mut self, days_elapsed: f64, stability: f64) {
        if days_elapsed > 0.0 && stability > 0.0 {
            self.retrieval = (1.0 + days_elapsed / (FSRS_FACTOR * stability))
                .powf(-1.0 / FSRS_DECAY)
                .clamp(0.0, 1.0);
        }
    }
}

// ============================================================================
// STRENGTH DECAY CALCULATOR
// ============================================================================

/// Calculates strength decay over time
pub struct StrengthDecay {
    /// FSRS stability (affects decay rate)
    stability: f64,
    /// Sentiment intensity (emotional memories decay slower)
    sentiment_boost: f64,
}

impl StrengthDecay {
    /// Create a new decay calculator
    pub fn new(stability: f64, sentiment_magnitude: f64) -> Self {
        Self {
            stability,
            sentiment_boost: 1.0 + sentiment_magnitude * 0.5,
        }
    }

    /// Calculate effective stability with sentiment boost
    pub fn effective_stability(&self) -> f64 {
        self.stability * self.sentiment_boost
    }

    /// Calculate retrieval strength after elapsed time
    ///
    /// Uses FSRS-6 power law forgetting curve
    pub fn retrieval_at(&self, days_elapsed: f64) -> f64 {
        if days_elapsed <= 0.0 {
            return 1.0;
        }

        let effective_s = self.effective_stability();
        (1.0 + days_elapsed / (FSRS_FACTOR * effective_s))
            .powf(-1.0 / FSRS_DECAY)
            .clamp(0.0, 1.0)
    }

    /// Calculate combined retention at a given time
    pub fn retention_at(&self, days_elapsed: f64, storage_strength: f64) -> f64 {
        let retrieval = self.retrieval_at(days_elapsed);
        (retrieval * 0.7) + ((storage_strength / MAX_STORAGE_STRENGTH).min(1.0) * 0.3)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64, epsilon: f64) -> bool {
        (a - b).abs() < epsilon
    }

    #[test]
    fn test_dual_strength_default() {
        let ds = DualStrength::default();
        assert_eq!(ds.storage, 1.0);
        assert_eq!(ds.retrieval, 1.0);
        // retention = (retrieval * 0.7) + ((storage / MAX_STORAGE_STRENGTH) * 0.3)
        // = (1.0 * 0.7) + ((1.0 / 10.0) * 0.3) = 0.7 + 0.03 = 0.73
        assert!(approx_eq(ds.retention(), 0.73, 0.01));
    }

    #[test]
    fn test_dual_strength_retention() {
        // Full retrieval, low storage
        let ds1 = DualStrength::new(1.0, 1.0);
        assert!(approx_eq(ds1.retention(), 0.73, 0.01)); // 0.7*1.0 + 0.3*0.1

        // Full retrieval, max storage
        let ds2 = DualStrength::new(10.0, 1.0);
        assert!(approx_eq(ds2.retention(), 1.0, 0.01)); // 0.7*1.0 + 0.3*1.0

        // Zero retrieval, max storage
        let ds3 = DualStrength::new(10.0, 0.0);
        assert!(approx_eq(ds3.retention(), 0.3, 0.01)); // 0.7*0.0 + 0.3*1.0
    }

    #[test]
    fn test_successful_recall() {
        let mut ds = DualStrength::new(1.0, 0.5);

        ds.on_successful_recall();

        assert!(ds.storage > 1.0); // Storage increased
        assert_eq!(ds.retrieval, 1.0); // Retrieval reset
    }

    #[test]
    fn test_lapse() {
        let mut ds = DualStrength::new(1.0, 0.5);

        ds.on_lapse();

        assert!(ds.storage > 1.1); // Storage increased more
        assert_eq!(ds.retrieval, 1.0); // Retrieval reset
    }

    #[test]
    fn test_storage_cap() {
        let mut ds = DualStrength::new(9.9, 1.0);

        ds.on_successful_recall();

        assert_eq!(ds.storage, MAX_STORAGE_STRENGTH); // Capped at 10.0
    }

    #[test]
    fn test_decay_over_time() {
        let mut ds = DualStrength::new(1.0, 1.0);
        let stability = 10.0;

        // Apply decay for 1 day
        ds.apply_decay(1.0, stability);
        assert!(ds.retrieval < 1.0);
        assert!(ds.retrieval > 0.9);

        // Apply decay for 10 days
        ds.apply_decay(10.0, stability);
        assert!(ds.retrieval < 0.9);
    }

    #[test]
    fn test_strength_decay_calculator() {
        let decay = StrengthDecay::new(10.0, 0.0);

        // At time 0, full retrieval
        assert!(approx_eq(decay.retrieval_at(0.0), 1.0, 0.01));

        // Over time, retrieval decreases
        let r1 = decay.retrieval_at(1.0);
        let r10 = decay.retrieval_at(10.0);
        assert!(r1 > r10);
    }

    #[test]
    fn test_sentiment_boost() {
        let decay_neutral = StrengthDecay::new(10.0, 0.0);
        let decay_emotional = StrengthDecay::new(10.0, 1.0);

        // Emotional memories decay slower
        let r_neutral = decay_neutral.retrieval_at(10.0);
        let r_emotional = decay_emotional.retrieval_at(10.0);

        assert!(r_emotional > r_neutral);
    }
}
