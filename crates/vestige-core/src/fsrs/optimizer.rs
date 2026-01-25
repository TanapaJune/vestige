//! FSRS-6 Parameter Optimizer
//!
//! Personalizes FSRS parameters based on user review history.
//! Uses gradient-free optimization to minimize prediction error.

use super::algorithm::{retrievability_with_decay, FSRS6_WEIGHTS};
use chrono::{DateTime, Utc};

// ============================================================================
// REVIEW LOG
// ============================================================================

/// A single review event for optimization
#[derive(Debug, Clone)]
pub struct ReviewLog {
    /// Review timestamp
    pub timestamp: DateTime<Utc>,
    /// Rating given (1-4)
    pub rating: i32,
    /// Stability at time of review
    pub stability: f64,
    /// Difficulty at time of review
    pub difficulty: f64,
    /// Days since last review
    pub elapsed_days: f64,
}

// ============================================================================
// OPTIMIZER
// ============================================================================

/// FSRS parameter optimizer
///
/// Personalizes the 21 FSRS-6 parameters based on user review history.
/// Uses the RMSE (Root Mean Square Error) of retrievability predictions
/// as the loss function.
pub struct FSRSOptimizer {
    /// Current weights being optimized
    weights: [f64; 21],
    /// Review history for training
    reviews: Vec<ReviewLog>,
    /// Minimum reviews required for optimization
    min_reviews: usize,
}

impl Default for FSRSOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl FSRSOptimizer {
    /// Create a new optimizer with default weights
    pub fn new() -> Self {
        Self {
            weights: FSRS6_WEIGHTS,
            reviews: Vec::new(),
            min_reviews: 100,
        }
    }

    /// Add a review to the training history
    pub fn add_review(&mut self, review: ReviewLog) {
        self.reviews.push(review);
    }

    /// Add multiple reviews
    pub fn add_reviews(&mut self, reviews: impl IntoIterator<Item = ReviewLog>) {
        self.reviews.extend(reviews);
    }

    /// Get current weights
    pub fn weights(&self) -> &[f64; 21] {
        &self.weights
    }

    /// Check if enough reviews for optimization
    pub fn has_enough_data(&self) -> bool {
        self.reviews.len() >= self.min_reviews
    }

    /// Get the number of reviews in history
    pub fn review_count(&self) -> usize {
        self.reviews.len()
    }

    /// Calculate RMSE loss for current weights
    pub fn calculate_loss(&self) -> f64 {
        if self.reviews.is_empty() {
            return 0.0;
        }

        let w20 = self.weights[20];
        let mut sum_squared_error = 0.0;

        for review in &self.reviews {
            // Calculate predicted retrievability
            let predicted_r = retrievability_with_decay(review.stability, review.elapsed_days, w20);

            // Convert rating to binary outcome (Again = 0, others = 1)
            let actual = if review.rating == 1 { 0.0 } else { 1.0 };

            let error = predicted_r - actual;
            sum_squared_error += error * error;
        }

        (sum_squared_error / self.reviews.len() as f64).sqrt()
    }

    /// Optimize the forgetting curve decay parameter (w20)
    ///
    /// This is the most personalizable parameter in FSRS-6.
    /// Uses golden section search for 1D optimization.
    pub fn optimize_decay(&mut self) -> f64 {
        if !self.has_enough_data() {
            return self.weights[20];
        }

        let (mut a, mut b) = (0.01, 1.0);
        let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;

        let mut x1 = b - (b - a) / phi;
        let mut x2 = a + (b - a) / phi;

        let mut f1 = self.loss_at_decay(x1);
        let mut f2 = self.loss_at_decay(x2);

        // Golden section iterations
        for _ in 0..50 {
            if f1 < f2 {
                b = x2;
                x2 = x1;
                f2 = f1;
                x1 = b - (b - a) / phi;
                f1 = self.loss_at_decay(x1);
            } else {
                a = x1;
                x1 = x2;
                f1 = f2;
                x2 = a + (b - a) / phi;
                f2 = self.loss_at_decay(x2);
            }

            if (b - a).abs() < 0.001 {
                break;
            }
        }

        let optimal_decay = (a + b) / 2.0;
        self.weights[20] = optimal_decay;
        optimal_decay
    }

    /// Calculate loss at a specific decay value
    fn loss_at_decay(&self, decay: f64) -> f64 {
        if self.reviews.is_empty() {
            return 0.0;
        }

        let mut sum_squared_error = 0.0;

        for review in &self.reviews {
            let predicted_r =
                retrievability_with_decay(review.stability, review.elapsed_days, decay);

            let actual = if review.rating == 1 { 0.0 } else { 1.0 };
            let error = predicted_r - actual;
            sum_squared_error += error * error;
        }

        (sum_squared_error / self.reviews.len() as f64).sqrt()
    }

    /// Reset optimizer state
    pub fn reset(&mut self) {
        self.weights = FSRS6_WEIGHTS;
        self.reviews.clear();
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn create_test_reviews(count: usize) -> Vec<ReviewLog> {
        let now = Utc::now();
        (0..count)
            .map(|i| ReviewLog {
                timestamp: now - Duration::days(i as i64),
                rating: if i % 5 == 0 { 1 } else { 3 },
                stability: 5.0 + (i as f64 * 0.1),
                difficulty: 5.0,
                elapsed_days: 1.0 + (i as f64 * 0.5),
            })
            .collect()
    }

    #[test]
    fn test_optimizer_creation() {
        let optimizer = FSRSOptimizer::new();
        assert_eq!(optimizer.weights().len(), 21);
        assert!(!optimizer.has_enough_data());
    }

    #[test]
    fn test_add_reviews() {
        let mut optimizer = FSRSOptimizer::new();
        let reviews = create_test_reviews(50);

        optimizer.add_reviews(reviews);
        assert_eq!(optimizer.review_count(), 50);
        assert!(!optimizer.has_enough_data()); // Need 100
    }

    #[test]
    fn test_calculate_loss() {
        let mut optimizer = FSRSOptimizer::new();
        let reviews = create_test_reviews(100);
        optimizer.add_reviews(reviews);

        let loss = optimizer.calculate_loss();
        assert!(loss >= 0.0);
        assert!(loss <= 1.0);
    }

    #[test]
    fn test_optimize_decay() {
        let mut optimizer = FSRSOptimizer::new();
        let reviews = create_test_reviews(200);
        optimizer.add_reviews(reviews);

        let original_decay = optimizer.weights()[20];
        let optimized_decay = optimizer.optimize_decay();

        // Decay should be a reasonable value
        assert!(optimized_decay > 0.01);
        assert!(optimized_decay < 1.0);

        // Optimization should have changed the value
        assert_ne!(original_decay, optimized_decay);
    }

    #[test]
    fn test_reset() {
        let mut optimizer = FSRSOptimizer::new();
        let reviews = create_test_reviews(100);
        optimizer.add_reviews(reviews);

        optimizer.reset();
        assert_eq!(optimizer.review_count(), 0);
        assert_eq!(optimizer.weights()[20], FSRS6_WEIGHTS[20]);
    }
}
